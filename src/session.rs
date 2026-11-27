//! Where Xylem runs: as root (an elevated Administrator on Windows) and/or
//! inside an SSH session. Detected once at startup; the screen says so
//! (docs/plans/root-ssh-indicator.md).

use std::ffi::OsString;

use crate::platform;

/// Set by sshd for the session (and kept by `sudo -E`, not by `sudo`).
const SSH_VARS: [&str; 3] = ["SSH_CONNECTION", "SSH_CLIENT", "SSH_TTY"];
/// An ancestor process with one of these names: the session came in through
/// it (`sshd-session` is the per-session sshd of OpenSSH 9.8 and later).
const SSH_SERVERS: [&str; 3] = ["sshd", "sshd-session", "mosh-server"];
/// The machine's name in the badge, at most.
const HOST_MAX: usize = 24;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Session {
    /// Root on Unix, an elevated process on Windows.
    pub admin: bool,
    /// Inside an SSH session: the machine's short name (empty when unknown).
    pub ssh: Option<String>,
}

impl Session {
    /// The session Xylem runs in. Only `main` calls it: the tests may run as
    /// root, and their `App` must not say so.
    pub fn detect() -> Session {
        let ssh = in_ssh(|k| std::env::var_os(k), platform::ancestor_names);
        Session::from_parts(
            platform::is_elevated(),
            ssh,
            platform::hostname().as_deref(),
        )
    }

    pub fn from_parts(admin: bool, ssh: bool, host: Option<&str>) -> Session {
        Session {
            admin,
            ssh: ssh.then(|| host.map(short_host).unwrap_or_default()),
        }
    }
}

/// One of the SSH variables is set, or else an SSH server is among the
/// ancestors (`sudo` and `su -` clear the variables). `ancestors` is only
/// called without the variables.
pub fn in_ssh(
    var: impl Fn(&str) -> Option<OsString>,
    ancestors: impl FnOnce() -> Vec<String>,
) -> bool {
    SSH_VARS
        .iter()
        .any(|k| var(k).is_some_and(|v| !v.is_empty()))
        || ancestors()
            .iter()
            .any(|name| SSH_SERVERS.contains(&name.as_str()))
}

/// The host name up to the first dot, with only what a host name holds
/// (letters, digits, `-`, `_`): it comes from the system, and nothing else
/// may reach the screen.
pub fn short_host(raw: &str) -> String {
    raw.split('.')
        .next()
        .unwrap_or_default()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(HOST_MAX)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_from_any_of_the_variables() {
        for name in ["SSH_CONNECTION", "SSH_CLIENT", "SSH_TTY"] {
            let var = |k: &str| (k == name).then(|| OsString::from("10.0.0.1 22"));
            assert!(in_ssh(var, || panic!("ancestors read with {name} set")));
        }
    }

    #[test]
    fn without_the_variables_the_ancestors_decide() {
        let empty = |_: &str| Some(OsString::new());
        let names = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(in_ssh(empty, || names(&[
            "bash",
            "sudo",
            "bash",
            "sshd-session",
            "sshd"
        ])));
        assert!(in_ssh(|_| None, || names(&["zsh", "mosh-server"])));
        assert!(in_ssh(|_| None, || names(&["bash", "sshd"])));
        assert!(!in_ssh(|_| None, || names(&["bash", "login", "systemd"])));
        assert!(!in_ssh(|_| None, || names(&["xsshd", "sshd2"])));
    }

    #[test]
    fn short_host_keeps_the_first_label() {
        assert_eq!(short_host("srv01.example.com"), "srv01");
        assert_eq!(short_host("db-02_a"), "db-02_a");
    }

    #[test]
    fn short_host_drops_what_is_not_a_name() {
        assert_eq!(short_host("\u{202e}evil\u{1b}[31m"), "evil31m");
        assert_eq!(short_host(&"a".repeat(100)), "a".repeat(24));
        assert_eq!(short_host(".hidden"), "");
    }

    #[test]
    fn from_parts_names_the_machine_only_in_ssh() {
        assert_eq!(
            Session::from_parts(true, false, Some("srv01")),
            Session {
                admin: true,
                ssh: None
            }
        );
        assert_eq!(
            Session::from_parts(false, true, Some("srv01.lan"))
                .ssh
                .as_deref(),
            Some("srv01")
        );
        assert_eq!(
            Session::from_parts(false, true, None).ssh.as_deref(),
            Some("")
        );
    }
}
