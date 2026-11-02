//! Remote machines over SSH: their files browsed through SFTP, spoken over
//! the system's own `ssh` (so `~/.ssh/config`, keys, the agent and
//! `known_hosts` all apply, and nothing is installed on the remote side).
//! A container is reached the same way, with `docker exec` running the
//! container's own `sftp-server` instead of ssh.
//!
//! A remote window's tree has a root like `andre@10.0.0.5:/home/andre`; the
//! remote path of anything below it is the remote root plus the components
//! joined with `/`, whatever the local system (as with archive entries).

pub mod askpass;
pub mod sftp;

use std::ffi::{OsStr, OsString};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, SystemTime};

use crate::model::tree::{FileEntry, FileKind, Lister};
use crate::platform;
use askpass::Askpass;
use sftp::{Attrs, Handle, Sftp};

/// How a remote machine is reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Via {
    Ssh,
    /// `docker exec` into a container (`host` is its name or id).
    Docker,
}

/// A remote machine and where to start, as typed:
/// `ssh://[user@]host[:port][/path]`, `user@host:[path]` or
/// `docker://[user@]container[/path]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    pub via: Via,
    pub user: Option<String>,
    pub host: String,
    pub port: Option<u16>,
    /// Empty: the home directory (a container's working directory).
    pub path: String,
}

/// Finds the container's `sftp-server` and becomes it. Fixed text: nothing
/// typed goes into it.
const DOCKER_SFTP: &str = "for p in /usr/lib/openssh/sftp-server /usr/libexec/openssh/sftp-server \
/usr/lib/ssh/sftp-server /usr/libexec/sftp-server; do [ -x \"$p\" ] && exec \"$p\"; done; \
echo 'no sftp-server in the container (install openssh-sftp-server)' >&2; exit 127";

/// The shell in a container: in the folder given as `$1`, the user's
/// shell, or bash if there is one, or sh.
const DOCKER_SHELL: &str = "cd \"$1\" 2>/dev/null; XY_TERMINAL=1 exec \
\"${SHELL:-$(command -v bash || echo /bin/sh)}\" -l";

impl Address {
    /// `None` for anything that is not a remote address. Plain `host:path`
    /// is not one: the user is required, so local names with a colon stay
    /// local.
    pub fn parse(text: &str) -> Option<Address> {
        let text = text.trim();
        if text.is_empty() || text.chars().any(char::is_whitespace) {
            return None;
        }
        if let Some(rest) = text.strip_prefix("docker://") {
            let (authority, path) = match rest.find('/') {
                Some(i) => (&rest[..i], &rest[i..]),
                None => (rest, ""),
            };
            let (user, name) = match authority.rsplit_once('@') {
                Some((u, h)) => (Some(u), h),
                None => (None, authority),
            };
            let mut address = Self::build(user, name, None, path)?;
            address.via = Via::Docker;
            return Some(address);
        }
        if let Some(rest) = text.strip_prefix("ssh://") {
            let (authority, path) = match rest.find('/') {
                Some(i) => (&rest[..i], &rest[i..]),
                None => (rest, ""),
            };
            let (user, host_port) = match authority.rsplit_once('@') {
                Some((u, h)) => (Some(u), h),
                None => (None, authority),
            };
            let (host, port) = match host_port.rsplit_once(':') {
                Some((h, p)) => (h, Some(p.parse().ok()?)),
                None => (host_port, None),
            };
            return Self::build(user, host, port, path);
        }
        let (user, rest) = text.split_once('@')?;
        let (host, path) = rest.split_once(':')?;
        Self::build(Some(user), host, None, path)
    }

    fn build(user: Option<&str>, host: &str, port: Option<u16>, path: &str) -> Option<Address> {
        // Names only: nothing that ssh could take as an option.
        let valid = |s: &str| {
            !s.is_empty()
                && !s.starts_with('-')
                && s.chars().all(|c| c.is_alphanumeric() || "-._".contains(c))
        };
        if !valid(host) || user.is_some_and(|u| !valid(u)) {
            return None;
        }
        Some(Address {
            via: Via::Ssh,
            user: user.map(String::from),
            host: host.to_string(),
            port,
            path: path.to_string(),
        })
    }

    /// What `ssh` gets: `user@host` or `host`; for a container, its name
    /// after `docker:`, as shown.
    pub fn destination(&self) -> String {
        let at = match &self.user {
            Some(user) => format!("{user}@{}", self.host),
            None => self.host.clone(),
        };
        match self.via {
            Via::Ssh => at,
            Via::Docker => format!("docker:{at}"),
        }
    }

    /// `docker exec` into the container, with `options` before its name
    /// and `command` after it.
    fn docker(&self, options: &[&str], command: &[&str]) -> Command {
        let mut cmd = Command::new(docker_program());
        cmd.arg("exec").args(options);
        if let Some(user) = &self.user {
            cmd.arg("-u").arg(user);
        }
        // The name is checked, and after `--` it could not be an option anyway.
        cmd.arg("--").arg(&self.host).args(command);
        cmd
    }

    /// `ssh` with the options every connection uses.
    fn ssh(&self, control: Option<&Path>, master: bool) -> Command {
        let mut cmd = Command::new(ssh_program());
        // A dead network ends ssh, which ends the calls waiting on it.
        cmd.args([
            "-o",
            "ServerAliveInterval=15",
            "-o",
            "ServerAliveCountMax=3",
        ]);
        if let Some(port) = self.port {
            cmd.arg("-p").arg(port.to_string());
        }
        if let Some(control) = control {
            let mode = if master { "auto" } else { "no" };
            cmd.arg("-o").arg(format!("ControlMaster={mode}"));
            cmd.arg("-o")
                .arg(format!("ControlPath={}", control.display()));
        }
        cmd
    }
}

/// The ssh program: `ssh`, or `$XY_SSH` (the tests point it at a wrapper
/// with a configuration of their own, away from the user's `~/.ssh`).
fn ssh_program() -> OsString {
    std::env::var_os("XY_SSH").unwrap_or_else(|| "ssh".into())
}

/// The docker program: `docker`, or `$XY_DOCKER` (`podman`, or the tests'
/// stand-in).
fn docker_program() -> OsString {
    std::env::var_os("XY_DOCKER").unwrap_or_else(|| "docker".into())
}

/// A connected remote machine.
pub struct Remote {
    pub address: Address,
    /// The window's root, as shown: `user@host:/home/user`.
    pub root: PathBuf,
    /// The remote path of the root.
    remote_root: Vec<u8>,
    sftp: Mutex<Sftp<ChildStdout, ChildStdin>>,
    child: Mutex<Child>,
    /// ssh's control socket, so the terminal reuses the login (Unix only).
    control: Option<PathBuf>,
}

type Session = (Sftp<ChildStdout, ChildStdin>, Vec<u8>);

/// A connection being made: ssh may be asking for a password meanwhile.
/// Dropping it before it is made kills ssh (Esc cancels).
pub struct Connecting {
    pub address: Address,
    pub askpass: Askpass,
    child: Option<Child>,
    control: Option<PathBuf>,
    result: Receiver<io::Result<Session>>,
}

impl Connecting {
    /// Starts `ssh` towards `address`; `run` is Xylem's private folder for
    /// this run and `n` numbers the connection.
    pub fn start(address: Address, run: &Path, n: usize) -> io::Result<Connecting> {
        let askpass = Askpass::new(run, n)?;
        let (mut cmd, control) = match address.via {
            Via::Ssh => {
                // Unix sockets have a short path limit (ssh adds a suffix while
                // creating it); with a long home, the terminal logs in again instead.
                let control = Some(run.join(format!("cm-{n}")))
                    .filter(|c| cfg!(unix) && c.as_os_str().len() <= 80);
                let mut cmd = address.ssh(control.as_deref(), true);
                cmd.arg("-s").arg(address.destination()).arg("sftp");
                // ssh asks through Xylem, never on the screen it is drawing.
                cmd.env("SSH_ASKPASS", std::env::current_exe()?)
                    .env("SSH_ASKPASS_REQUIRE", "force")
                    .env(askpass::DIR_VAR, askpass.dir());
                if std::env::var_os("DISPLAY").is_none() {
                    // Older ssh only uses SSH_ASKPASS with a DISPLAY.
                    cmd.env("DISPLAY", ":0");
                }
                (cmd, control)
            }
            // docker exec asks nothing.
            Via::Docker => (address.docker(&["-i"], &["sh", "-c", DOCKER_SFTP]), None),
        };
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        detach(&mut cmd);
        let program = cmd.get_program().to_string_lossy().into_owned();
        let mut child = cmd
            .spawn()
            .map_err(|e| io::Error::new(e.kind(), format!("cannot run {program}: {e}")))?;
        let pipes = (
            child.stdout.take().expect("piped stdout"),
            child.stdin.take().expect("piped stdin"),
            child.stderr.take().expect("piped stderr"),
        );
        let (tx, rx) = mpsc::channel();
        let path = address.path.clone();
        std::thread::spawn(move || {
            let _ = tx.send(handshake(pipes, &path));
        });
        Ok(Connecting {
            address,
            askpass,
            child: Some(child),
            control,
            result: rx,
        })
    }

    /// The connection, once made (or why it failed).
    pub fn poll(&mut self) -> Option<io::Result<Remote>> {
        let session = match self.result.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                Err(io::Error::other("the connection was interrupted"))
            }
        };
        let mut child = self.child.take()?;
        Some(match session {
            Ok((sftp, remote_root)) => {
                let root = PathBuf::from(format!(
                    "{}:{}",
                    self.address.destination(),
                    String::from_utf8_lossy(&remote_root)
                ));
                Ok(Remote {
                    address: self.address.clone(),
                    root,
                    remote_root,
                    sftp: Mutex::new(sftp),
                    child: Mutex::new(child),
                    control: self.control.take(),
                })
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                Err(e)
            }
        })
    }
}

impl Drop for Connecting {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// On Unix, ssh runs in a session of its own, without the terminal: it can
/// only ask through SSH_ASKPASS.
#[cfg(unix)]
fn detach(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;
    // SAFETY: only setsid() runs between fork and exec, and it is
    // async-signal-safe.
    unsafe {
        cmd.pre_exec(|| nix::unistd::setsid().map(|_| ()).map_err(io::Error::from));
    }
}

#[cfg(not(unix))]
fn detach(_cmd: &mut Command) {}

/// The SFTP session over ssh's pipes, starting at `path` (empty: home).
fn handshake(
    (stdout, stdin, mut stderr): (ChildStdout, ChildStdin, std::process::ChildStderr),
    path: &str,
) -> io::Result<Session> {
    let connected = Sftp::init(stdout, stdin).and_then(|mut sftp| {
        let start = if path.is_empty() { "." } else { path };
        let remote_root = sftp.realpath(start.as_bytes())?;
        if !sftp.stat(&remote_root)?.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                format!(
                    "{} is not a directory",
                    String::from_utf8_lossy(&remote_root)
                ),
            ));
        }
        Ok((sftp, remote_root))
    });
    match connected {
        Ok(session) => {
            // Whatever ssh says from now on (warnings) must not fill the pipe.
            std::thread::spawn(move || io::copy(&mut stderr, &mut io::sink()));
            Ok(session)
        }
        Err(e) => {
            // ssh has ended (the pipe closed); its own words say more
            // ("Permission denied", "Could not resolve hostname").
            let mut said = String::new();
            let _ = stderr.read_to_string(&mut said);
            let last = said.lines().rev().find(|l| !l.trim().is_empty());
            Err(match last {
                Some(line) => io::Error::new(e.kind(), line.trim().to_string()),
                None => e,
            })
        }
    }
}

#[cfg(unix)]
fn os_bytes(s: &OsStr) -> Vec<u8> {
    std::os::unix::ffi::OsStrExt::as_bytes(s).to_vec()
}

#[cfg(not(unix))]
fn os_bytes(s: &OsStr) -> Vec<u8> {
    s.to_string_lossy().into_owned().into_bytes()
}

#[cfg(unix)]
fn bytes_os(b: Vec<u8>) -> OsString {
    std::os::unix::ffi::OsStringExt::from_vec(b)
}

#[cfg(not(unix))]
fn bytes_os(b: Vec<u8>) -> OsString {
    String::from_utf8_lossy(&b).into_owned().into()
}

/// A remote entry as a file of the tree.
fn file_entry(name: Vec<u8>, a: &Attrs) -> FileEntry {
    FileEntry {
        name: bytes_os(name),
        size: a.size.unwrap_or(0),
        disk: a.size.unwrap_or(0),
        link: None,
        modified: a
            .mtime
            .map(|s| SystemTime::UNIX_EPOCH + Duration::from_secs(s.into())),
        mode: platform::from_unix_mode(a.permissions.unwrap_or(0o100644)),
        kind: if a.is_symlink() {
            FileKind::Symlink
        } else if a.is_file() {
            FileKind::File
        } else {
            FileKind::Other
        },
        tagged: false,
    }
}

impl Remote {
    fn sftp(&self) -> MutexGuard<'_, Sftp<ChildStdout, ChildStdin>> {
        self.sftp.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The remote path of `path`, a path of the window's tree.
    pub fn remote_path(&self, path: &Path) -> Vec<u8> {
        let mut out = self.remote_root.clone();
        if let Ok(rel) = path.strip_prefix(&self.root) {
            for c in rel.components() {
                if let Component::Normal(name) = c {
                    if !out.ends_with(b"/") {
                        out.push(b'/');
                    }
                    out.extend(os_bytes(name));
                }
            }
        }
        out
    }

    /// The path in the window's tree for a remote path, if it is inside the
    /// window's root.
    pub fn virtual_path(&self, remote: &str) -> Option<PathBuf> {
        let root = String::from_utf8_lossy(&self.remote_root);
        let rel = remote.strip_prefix(root.as_ref())?;
        if !(rel.is_empty() || rel.starts_with('/') || root.ends_with('/')) {
            return None; // /home/andrex is not inside /home/andre
        }
        let mut path = self.root.clone();
        path.extend(rel.split('/').filter(|c| !c.is_empty()));
        Some(path)
    }

    /// A Windows machine: its paths start with a drive (`/C:/Users`).
    pub fn is_windows(&self) -> bool {
        let r = &self.remote_root;
        r.len() >= 3 && r[0] == b'/' && r[1].is_ascii_alphabetic() && r[2] == b':'
    }

    pub fn lstat(&self, path: &Path) -> io::Result<FileEntry> {
        let attrs = self.sftp().lstat(&self.remote_path(path))?;
        let name = path.file_name().map(os_bytes).unwrap_or_default();
        Ok(file_entry(name, &attrs))
    }

    /// The SFTP attributes of `path` (not following a symlink).
    pub fn attrs(&self, path: &Path) -> io::Result<Attrs> {
        self.sftp().lstat(&self.remote_path(path))
    }

    /// "uid:gid" of `path`: SFTP v3 sends ids, not names.
    pub fn owner(&self, path: &Path) -> Option<String> {
        let a = self.sftp().lstat(&self.remote_path(path)).ok()?;
        Some(format!("{}:{}", a.uid?, a.gid?))
    }

    /// Where a symlink points.
    pub fn read_link(&self, path: &Path) -> io::Result<String> {
        let target = self.sftp().read_link(&self.remote_path(path))?;
        Ok(String::from_utf8_lossy(&target).into_owned())
    }

    /// A file for reading, bit by bit.
    pub fn open(self: &Arc<Self>, path: &Path) -> io::Result<RemoteFile> {
        let handle = self.sftp().open(&self.remote_path(path))?;
        Ok(RemoteFile {
            remote: self.clone(),
            handle: Some(handle),
            offset: 0,
        })
    }

    /// The first `limit` bytes of a file.
    pub fn read(self: &Arc<Self>, path: &Path, limit: u64) -> io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.open(path)?.take(limit).read_to_end(&mut data)?;
        Ok(data)
    }

    /// Creates `platform::tmp_path(dst)` on the remote machine for writing,
    /// never opening what is there: a leftover (or a symlink someone put
    /// there) is removed as itself and the creation tried once more.
    pub fn create_tmp(self: &Arc<Self>, dst: &Path) -> io::Result<(RemoteWriter, PathBuf)> {
        let tmp = platform::tmp_path(dst);
        let path = self.remote_path(&tmp);
        let created = self.sftp().create_new(&path);
        let handle = match created {
            Ok(h) => h,
            Err(e) => {
                let mut sftp = self.sftp();
                match sftp.lstat(&path) {
                    Ok(a) if !a.is_dir() => {
                        sftp.remove(&path)?;
                        sftp.create_new(&path)?
                    }
                    _ => return Err(e),
                }
            }
        };
        let writer = RemoteWriter {
            remote: self.clone(),
            handle: Some(handle),
            written: 0,
        };
        Ok((writer, tmp))
    }

    /// Whether `path` is a directory (following a symlink).
    pub fn is_dir(&self, path: &Path) -> bool {
        self.sftp()
            .stat(&self.remote_path(path))
            .is_ok_and(|a| a.is_dir())
    }

    /// Creates `path` and whatever is missing between it and the window's
    /// root.
    pub fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        let missing: Vec<&Path> = path
            .ancestors()
            .take_while(|d| d.starts_with(&self.root) && *d != self.root)
            .collect();
        if !path.starts_with(&self.root) {
            return Err(io::Error::other("outside the remote window"));
        }
        let mut sftp = self.sftp();
        for dir in missing.into_iter().rev() {
            let remote = self.remote_path(dir);
            match sftp.stat(&remote) {
                Ok(a) if a.is_dir() => {}
                Ok(_) => return Err(io::Error::from(io::ErrorKind::NotADirectory)),
                Err(e) if e.kind() == io::ErrorKind::NotFound => sftp.mkdir(&remote)?,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Puts a finished temporary file at `dst`. An existing `dst` (only
    /// with `replace`, after a "yes") is replaced in one step when the
    /// server can; otherwise it is removed first, and for a moment neither
    /// is there under that name (the copy is complete in `tmp` by then).
    pub fn put_in_place(&self, tmp: &Path, dst: &Path, replace: bool) -> io::Result<()> {
        let (tmp, dst) = (self.remote_path(tmp), self.remote_path(dst));
        let mut sftp = self.sftp();
        if replace && !sftp.can_replace() {
            sftp.remove(&dst)?;
            return sftp.rename(&tmp, &dst, false);
        }
        sftp.rename(&tmp, &dst, replace)
    }

    /// Removes a file (a symlink itself, never what it points to). What is a
    /// directory by now is left alone.
    pub fn remove_file(&self, path: &Path) -> io::Result<()> {
        let path = self.remote_path(path);
        let mut sftp = self.sftp();
        if sftp.lstat(&path)?.is_dir() {
            return Err(io::Error::from(io::ErrorKind::IsADirectory));
        }
        sftp.remove(&path)
    }

    /// Removes an empty directory; a symlink to one is not a directory.
    pub fn remove_dir(&self, path: &Path) -> io::Result<()> {
        let path = self.remote_path(path);
        let mut sftp = self.sftp();
        if !sftp.lstat(&path)?.is_dir() {
            return Err(io::Error::from(io::ErrorKind::NotADirectory));
        }
        sftp.rmdir(&path)
    }

    /// Sets the permission bits of `path`. A symlink has none of its own:
    /// setting them would change what it points to, so it is refused.
    pub fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()> {
        let path = self.remote_path(path);
        let mut sftp = self.sftp();
        if sftp.lstat(&path)?.is_symlink() {
            return Err(io::Error::other("symbolic link skipped"));
        }
        sftp.set_mode(&path, mode)
    }

    /// Sets the modification time of `path`, keeping its access time; a
    /// symlink is refused, as with [`Remote::set_mode`].
    pub fn set_modified(&self, path: &Path, when: SystemTime) -> io::Result<()> {
        let mtime = when
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .and_then(|d| u32::try_from(d.as_secs()).ok())
            .ok_or_else(|| io::Error::other("SFTP keeps dates from 1970 to 2106 only"))?;
        let path = self.remote_path(path);
        let mut sftp = self.sftp();
        let attrs = sftp.lstat(&path)?;
        if attrs.is_symlink() {
            return Err(io::Error::other(
                "symbolic link skipped (the date would change its target)",
            ));
        }
        sftp.set_times(&path, attrs.atime.unwrap_or(mtime), mtime)
    }

    /// The filesystem `path` is on, to tell mounted disks apart.
    pub fn filesystem(&self, path: &Path) -> io::Result<sftp::Filesystem> {
        self.sftp().filesystem(&self.remote_path(path))
    }

    /// Creates directory `path` (a path of the window's tree).
    pub fn create_dir(&self, path: &Path) -> io::Result<()> {
        self.sftp().mkdir(&self.remote_path(path))
    }

    /// Renames `from` to `to`, replacing an existing `to` only with
    /// `replace` (see [`Sftp::rename`]).
    pub fn rename(&self, from: &Path, to: &Path, replace: bool) -> io::Result<()> {
        let (from, to) = (self.remote_path(from), self.remote_path(to));
        self.sftp().rename(&from, &to, replace)
    }

    /// The command for a terminal on this machine, starting in `dir` (a
    /// path of the window's tree): `ssh -t`, reusing the login when it can.
    pub fn terminal_command(&self, dir: &Path) -> (OsString, Vec<String>) {
        if self.address.via == Via::Docker {
            let dir = String::from_utf8_lossy(&self.remote_path(dir)).into_owned();
            let cmd = self
                .address
                .docker(&["-it"], &["sh", "-c", DOCKER_SHELL, "sh", &dir]);
            let args = cmd.get_args().map(|a| a.to_string_lossy().into_owned());
            return (docker_program(), args.collect());
        }
        let mut args: Vec<String> = Vec::new();
        args.extend(
            [
                "-o",
                "ServerAliveInterval=15",
                "-o",
                "ServerAliveCountMax=3",
            ]
            .map(String::from),
        );
        if let Some(port) = self.address.port {
            args.extend(["-p".into(), port.to_string()]);
        }
        if let Some(control) = &self.control {
            args.extend([
                "-o".into(),
                "ControlMaster=no".into(),
                "-o".into(),
                format!("ControlPath={}", control.display()),
            ]);
        }
        args.push("-t".into());
        args.push(self.address.destination());
        // A Unix shell starts in the directory, with XY_TERMINAL=1 as in the
        // local terminal (so a ~/.bashrc can, say, skip starting tmux); a
        // Windows one, as it is.
        if !self.is_windows() {
            let dir = String::from_utf8_lossy(&self.remote_path(dir)).replace('\'', "'\\''");
            args.push(format!(
                "cd '{dir}' 2>/dev/null; XY_TERMINAL=1 exec \"${{SHELL:-/bin/sh}}\" -l"
            ));
        }
        (ssh_program(), args)
    }
}

impl Lister for Remote {
    fn list(&self, path: &Path) -> io::Result<(Vec<(OsString, bool)>, Vec<FileEntry>)> {
        let entries = self.sftp().list_dir(&self.remote_path(path))?;
        let (mut dirs, mut files) = (Vec::new(), Vec::new());
        for e in entries {
            if e.attrs.is_dir() {
                let hidden = e.name.first() == Some(&b'.');
                dirs.push((bytes_os(e.name), hidden));
            } else {
                files.push(file_entry(e.name, &e.attrs));
            }
        }
        Ok((dirs, files))
    }
}

impl Drop for Remote {
    fn drop(&mut self) {
        let mut child = self.child.lock().unwrap_or_else(|e| e.into_inner());
        let _ = child.kill();
        let _ = child.wait();
    }
}

/// A remote file being read.
pub struct RemoteFile {
    remote: Arc<Remote>,
    handle: Option<Handle>,
    offset: u64,
}

impl RemoteFile {
    /// Reads from `offset` on.
    pub fn skip_to(&mut self, offset: u64) {
        self.offset = offset;
    }
}

impl Read for RemoteFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let Some(handle) = &self.handle else {
            return Ok(0);
        };
        let want = buf.len().min(sftp::READ_CHUNK as usize) as u32;
        let data = self.remote.sftp().read(handle, self.offset, want)?;
        buf[..data.len()].copy_from_slice(&data);
        self.offset += data.len() as u64;
        Ok(data.len())
    }
}

impl Drop for RemoteFile {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = self.remote.sftp().close_file(&handle);
        }
    }
}

/// A file being written on a remote machine (a temporary one: see
/// [`Remote::create_tmp`]).
pub struct RemoteWriter {
    remote: Arc<Remote>,
    handle: Option<Handle>,
    written: u64,
}

impl RemoteWriter {
    /// The end of writing: checks that the server has every byte sent,
    /// gives the file its permissions and date, and closes it.
    pub fn finish(mut self, mode: u32, mtime: Option<SystemTime>) -> io::Result<()> {
        let handle = self.handle.take().expect("open until finished");
        let mut sftp = self.remote.sftp();
        let checked = sftp.fstat(&handle).and_then(|a| match a.size {
            Some(size) if size == self.written => Ok(()),
            size => Err(io::Error::other(format!(
                "the server has {} bytes of the {} sent",
                size.map_or("an unknown number of".into(), |s| s.to_string()),
                self.written
            ))),
        });
        let mtime = mtime
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .and_then(|d| u32::try_from(d.as_secs()).ok());
        let set = checked.and_then(|()| sftp.fsetstat(&handle, mode, mtime));
        let closed = sftp.close_file(&handle);
        set.and(closed)
    }
}

impl io::Write for RemoteWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let handle = self.handle.as_ref().expect("open until finished");
        self.remote.sftp().write_all(handle, self.written, buf)?;
        self.written += buf.len() as u64;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for RemoteWriter {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = self.remote.sftp().close_file(&handle);
        }
    }
}

/// For tests: a "remote machine" that is OpenSSH's sftp-server spoken to
/// directly (no ssh, no network), rooted at `dir`. `None` without it.
#[cfg(all(test, unix))]
pub fn for_tests(dir: &Path) -> Option<Arc<Remote>> {
    let (child, mut sftp) = sftp::tests::server()?;
    let dir = os_bytes(dir.as_os_str());
    let remote_root = sftp.realpath(&dir).ok()?;
    let root = PathBuf::from(format!(
        "test@testhost:{}",
        String::from_utf8_lossy(&remote_root)
    ));
    Some(Arc::new(Remote {
        address: Address::parse("test@testhost:").unwrap(),
        root,
        remote_root,
        sftp: Mutex::new(sftp),
        child: Mutex::new(child),
        control: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn maps_paths_both_ways() {
        let tmp = crate::testing::tempdir();
        std::fs::create_dir(tmp.path().join("sub")).unwrap();
        let Some(remote) = for_tests(tmp.path()) else {
            return;
        };
        let inside = remote.root.join("sub").join("a b.txt");
        let real = tmp.path().canonicalize().unwrap();
        assert_eq!(
            remote.remote_path(&inside),
            format!("{}/sub/a b.txt", real.display()).into_bytes()
        );
        let back = remote.virtual_path(&format!("{}/sub", real.display()));
        assert_eq!(back, Some(remote.root.join("sub")));
        assert_eq!(remote.virtual_path("/etc"), None);
        // A sibling whose name starts the same is not inside.
        assert_eq!(remote.virtual_path(&format!("{}x", real.display())), None);
        assert!(!remote.is_windows());

        // The terminal: ssh -t, starting in the directory, with XY_TERMINAL.
        let (_, args) = remote.terminal_command(&remote.root.join("sub"));
        let command = args.last().unwrap();
        assert!(args.contains(&"-t".to_string()), "{args:?}");
        assert!(
            command.starts_with(&format!("cd '{}/sub'", real.display())),
            "{command}"
        );
        assert!(command.contains("XY_TERMINAL=1 exec"), "{command}");
    }

    #[cfg(unix)]
    #[test]
    fn removing_never_follows_a_symlink() {
        let tmp = crate::testing::tempdir();
        let outside = crate::testing::tempdir();
        std::fs::write(outside.path().join("keep"), "k").unwrap();
        std::fs::create_dir(tmp.path().join("dir")).unwrap();
        std::os::unix::fs::symlink(outside.path(), tmp.path().join("to_dir")).unwrap();
        std::os::unix::fs::symlink(outside.path().join("keep"), tmp.path().join("to_file"))
            .unwrap();
        let Some(remote) = for_tests(tmp.path()) else {
            return;
        };
        let at = |n: &str| remote.root.join(n);
        // A symlink to a directory is not a directory to remove...
        assert!(remote.remove_dir(&at("to_dir")).is_err());
        // ...and a directory is not a file to remove.
        assert!(remote.remove_file(&at("dir")).is_err());
        assert!(tmp.path().join("dir").is_dir());
        // A symlink goes as itself; what it pointed to stays.
        remote.remove_file(&at("to_dir")).unwrap();
        remote.remove_file(&at("to_file")).unwrap();
        assert_eq!(
            std::fs::read_to_string(outside.path().join("keep")).unwrap(),
            "k"
        );
        remote.remove_dir(&at("dir")).unwrap();
        assert_eq!(std::fs::read_dir(tmp.path()).unwrap().count(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn permissions_and_dates_never_go_through_a_symlink() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = crate::testing::tempdir();
        let outside = crate::testing::tempdir();
        let target = outside.path().join("t");
        std::fs::write(&target, "t").unwrap();
        std::fs::write(tmp.path().join("f"), "f").unwrap();
        std::os::unix::fs::symlink(&target, tmp.path().join("l")).unwrap();
        let Some(remote) = for_tests(tmp.path()) else {
            return;
        };
        let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o7777;
        let before = mode(&target);
        remote.set_mode(&remote.root.join("f"), 0o640).unwrap();
        assert_eq!(mode(&tmp.path().join("f")), 0o640);
        assert!(remote.set_mode(&remote.root.join("l"), 0o777).is_err());
        assert_eq!(mode(&target), before);

        let when = SystemTime::UNIX_EPOCH + Duration::from_secs(1_234_567_890);
        let atime = remote.attrs(&remote.root.join("f")).unwrap().atime;
        remote.set_modified(&remote.root.join("f"), when).unwrap();
        let a = remote.attrs(&remote.root.join("f")).unwrap();
        assert_eq!((a.mtime, a.atime), (Some(1_234_567_890), atime));
        let target_time = std::fs::metadata(&target).unwrap().modified().unwrap();
        assert!(remote.set_modified(&remote.root.join("l"), when).is_err());
        assert_eq!(
            std::fs::metadata(&target).unwrap().modified().unwrap(),
            target_time
        );
        // SFTP v3 cannot say a date before 1970.
        let old = SystemTime::UNIX_EPOCH - Duration::from_secs(1);
        assert!(remote.set_modified(&remote.root.join("f"), old).is_err());
    }

    #[test]
    fn parses_addresses() {
        let a = Address::parse("andre@10.0.0.5:/srv/www").unwrap();
        assert_eq!(a.destination(), "andre@10.0.0.5");
        assert_eq!((a.port, a.path.as_str()), (None, "/srv/www"));
        let a = Address::parse("andre@host:").unwrap();
        assert_eq!(a.path, "", "the home directory");
        let a = Address::parse("ssh://andre@host.lan:2222/tmp").unwrap();
        assert_eq!((a.port, a.path.as_str()), (Some(2222), "/tmp"));
        let a = Address::parse("ssh://myserver").unwrap();
        assert_eq!(
            (a.destination().as_str(), a.path.as_str()),
            ("myserver", "")
        );
        // Local paths stay local.
        for local in [
            "/home/a",
            "notes:2026.txt",
            "C:\\Users",
            "~/x",
            "a b@c:d",
            "@host:x",
        ] {
            assert_eq!(Address::parse(local), None, "{local}");
        }
        // Nothing ssh would read as an option.
        assert_eq!(Address::parse("ssh://-oProxyCommand=x"), None);
        assert_eq!(Address::parse("-oX@host:/"), None);
        assert_eq!(Address::parse("andre@host:").unwrap().via, Via::Ssh);
    }

    #[test]
    fn parses_containers() {
        let a = Address::parse("docker://web").unwrap();
        assert_eq!(
            (a.via, a.host.as_str(), a.path.as_str()),
            (Via::Docker, "web", "")
        );
        assert_eq!((&a.user, a.port), (&None, None));
        assert_eq!(a.destination(), "docker:web");
        let a = Address::parse("docker://root@my_app.1/srv/www").unwrap();
        assert_eq!(a.destination(), "docker:root@my_app.1");
        assert_eq!(a.path, "/srv/www");
        // No port, no options, no shell.
        for bad in [
            "docker://",
            "docker://web:2222",
            "docker://-it",
            "docker://-u@web",
            "docker://a;b",
            "docker://$(id)",
            "docker://a b",
            "docker://@web",
        ] {
            assert_eq!(Address::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn a_container_is_reached_with_docker_exec_and_nothing_else() {
        let args = |cmd: Command| -> Vec<String> {
            cmd.get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect()
        };
        let a = Address::parse("docker://web").unwrap();
        let got = args(a.docker(&["-i"], &["sh", "-c", DOCKER_SFTP]));
        assert_eq!(got, ["exec", "-i", "--", "web", "sh", "-c", DOCKER_SFTP]);
        let a = Address::parse("docker://deploy@web").unwrap();
        let got = args(a.docker(&["-it"], &["true"]));
        assert_eq!(got, ["exec", "-it", "-u", "deploy", "--", "web", "true"]);
        // The scripts are fixed: no name or path is ever part of them.
        for script in [DOCKER_SFTP, DOCKER_SHELL] {
            assert!(!script.contains("web") && !script.contains("deploy"));
        }
    }

    #[cfg(unix)]
    #[test]
    fn the_shell_in_a_container_gets_its_folder_as_an_argument() {
        let tmp = crate::testing::tempdir();
        let Some(remote) = for_tests(tmp.path()) else {
            return;
        };
        let mut remote = Arc::try_unwrap(remote).ok().unwrap();
        remote.address = Address::parse("docker://web").unwrap();
        let odd = remote.root.join("it's $(here)");
        let (_, args) = remote.terminal_command(&odd);
        let real = tmp.path().canonicalize().unwrap();
        assert_eq!(&args[..4], ["exec", "-it", "--", "web"]);
        assert_eq!(&args[4..7], ["sh", "-c", DOCKER_SHELL]);
        assert_eq!(
            &args[7..],
            ["sh", &format!("{}/it's $(here)", real.display())]
        );
        // And the script does go there, and starts a shell.
        std::fs::create_dir(tmp.path().join("it's $(here)")).unwrap();
        let shell = tmp.path().join("shell");
        std::fs::write(&shell, "#!/bin/sh\npwd\n").unwrap();
        let mode = std::os::unix::fs::PermissionsExt::from_mode(0o755);
        std::fs::set_permissions(&shell, mode).unwrap();
        let out = Command::new("sh")
            .args(["-c", DOCKER_SHELL, "sh"])
            .arg(&args[8])
            .env("SHELL", &shell)
            .output()
            .unwrap();
        let said = String::from_utf8_lossy(&out.stdout);
        assert!(said.trim().ends_with("it's $(here)"), "{said}");
    }
}
