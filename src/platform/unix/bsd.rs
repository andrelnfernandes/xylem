//! macOS and FreeBSD, which have no `/proc`: what Linux reads there comes
//! from `ps` and `sysctl`. The system is told by `std::env::consts::OS` when
//! Xylem runs, not by `#[cfg]`, so this code builds and is tested on Linux
//! too.

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

/// `ps` on macOS and FreeBSD.
const PS: &str = "/bin/ps";
/// Of `ps -A`, at most.
const PS_MAX: u64 = 4 << 20;
/// How long a program may take.
const LIMIT: Duration = Duration::from_secs(1);

/// Runs `program` and returns what it prints, at most `cap` bytes. There is
/// no shell and no `PATH`: the environment is empty but for `LC_ALL=C`, and
/// stdin and stderr go nowhere. A program that does not start, or that has
/// not finished printing within `limit`, gives `None`; one that prints more
/// than `cap` is ended.
pub(super) fn run_capped(
    program: &Path,
    args: &[&str],
    cap: u64,
    limit: Duration,
) -> Option<String> {
    let mut child = Command::new(program)
        .args(args)
        .env_clear()
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let (tx, rx) = mpsc::channel();
    if let Some(stdout) = child.stdout.take() {
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let read = stdout.take(cap).read_to_end(&mut bytes);
            let _ = tx.send(read.map(|_| bytes));
        });
    }
    let out = rx.recv_timeout(limit);
    // Ends one still running (past the cap or the limit), then reaps it.
    let _ = child.kill();
    let _ = child.wait();
    let bytes = out.ok()?.ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// The lines of `ps -o pid=,ppid=,comm=`, as pid → (name, parent's pid). The
/// name is what follows the last `/` (macOS gives the whole path). A line
/// without both numbers and a name is skipped, and a repeated pid keeps its
/// first line.
fn parse_ps(text: &str) -> HashMap<u32, (String, u32)> {
    let mut table = HashMap::new();
    for (pid, parent, name) in text.lines().filter_map(ps_line) {
        table
            .entry(pid)
            .or_insert_with(|| (name.to_string(), parent));
    }
    table
}

fn ps_line(line: &str) -> Option<(u32, u32, &str)> {
    let (pid, rest) = line.trim_start().split_once(char::is_whitespace)?;
    let (parent, comm) = rest.trim_start().split_once(char::is_whitespace)?;
    let name = comm.trim().rsplit('/').next()?;
    if name.is_empty() {
        return None;
    }
    Some((pid.parse().ok()?, parent.parse().ok()?, name))
}

/// The names of the ancestors from `start` up, from one `ps -A`.
pub(super) fn ancestor_names(start: u32) -> Vec<String> {
    let args = ["-A", "-o", "pid=,ppid=,comm="];
    let Some(text) = run_capped(Path::new(PS), &args, PS_MAX, LIMIT) else {
        return Vec::new();
    };
    let table = parse_ps(&text);
    super::ancestors_with(start, |pid| table.get(&pid).cloned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::time::{Duration, Instant};

    #[test]
    fn run_capped_returns_what_the_program_prints() {
        let out = run_capped(
            Path::new("/bin/sh"),
            &["-c", "echo a; echo b"],
            1024,
            Duration::from_secs(5),
        );
        assert_eq!(out.as_deref(), Some("a\nb\n"));
    }

    #[test]
    fn run_capped_stops_at_the_cap() {
        let start = Instant::now();
        let out = run_capped(Path::new("/usr/bin/yes"), &[], 10, Duration::from_secs(5)).unwrap();
        assert_eq!(out.len(), 10);
        assert!(start.elapsed() < Duration::from_secs(3));
    }

    #[test]
    fn run_capped_gives_up_after_the_limit() {
        let start = Instant::now();
        assert_eq!(
            run_capped(
                Path::new("/bin/sleep"),
                &["5"],
                1024,
                Duration::from_millis(200)
            ),
            None
        );
        assert!(start.elapsed() < Duration::from_secs(3));
    }

    #[test]
    fn run_capped_passes_only_lc_all() {
        // env_clear also removes PATH, so the programs are called by full path.
        let env = run_capped(Path::new("/usr/bin/env"), &[], 4096, Duration::from_secs(5)).unwrap();
        assert!(env.lines().any(|l| l == "LC_ALL=C"), "{env}");
        assert!(!env.lines().any(|l| l.starts_with("HOME=")), "{env}");
    }

    #[test]
    fn run_capped_without_the_program_is_none() {
        assert_eq!(
            run_capped(
                Path::new("/nonexistent/ps"),
                &[],
                1024,
                Duration::from_secs(1)
            ),
            None
        );
    }

    #[test]
    fn parse_ps_reads_pid_parent_and_name() {
        let t = parse_ps(
            "    1     0 /sbin/launchd\n  612     1 /usr/sbin/sshd\n  640   612 sshd-session\n  700   640 -zsh\n  800   700 /Applications/Visual Studio Code.app/Contents/MacOS/Electron\n  810   700 Google Chrome Helper\n",
        );
        assert_eq!(t[&612], ("sshd".into(), 1));
        assert_eq!(t[&640], ("sshd-session".into(), 612));
        assert_eq!(t[&700], ("-zsh".into(), 640));
        assert_eq!(t[&800], ("Electron".into(), 700));
        assert_eq!(t[&810], ("Google Chrome Helper".into(), 700));
    }

    #[test]
    fn parse_ps_ignores_lines_without_a_name() {
        let t = parse_ps("garbage\n  900\n  901   1\n  902   1 /\nx 1 sshd\n  903   1 ok\n  90");
        assert_eq!(t.len(), 1);
        assert_eq!(t[&903], ("ok".into(), 1));
    }

    #[test]
    fn parse_ps_keeps_the_first_of_a_repeated_pid() {
        let t = parse_ps("  5  1 first\n  5  1 second\n");
        assert_eq!(t[&5].0, "first");
    }

    #[test]
    fn the_walk_over_ps_finds_sshd() {
        let t = parse_ps(
            "    1     0 /sbin/launchd\n  612     1 /usr/sbin/sshd\n  640   612 sshd-session\n  700   640 -zsh\n",
        );
        assert_eq!(
            super::super::ancestors_with(700, |pid| t.get(&pid).cloned()),
            ["-zsh", "sshd-session", "sshd"]
        );
    }

    #[test]
    fn ancestor_names_runs_the_real_ps() {
        // Linux also has /bin/ps (procps), with the same options.
        let parent = std::os::unix::process::parent_id();
        if !Path::new("/bin/ps").exists() || parent <= 1 {
            return;
        }
        assert!(!ancestor_names(parent).is_empty());
    }
}
