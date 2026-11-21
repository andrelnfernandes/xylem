//! Telling that a new version is out (docs/plans/update-check.md): once a
//! day, if the user said yes, the system's `curl` fetches one line with the
//! latest version from the website, in a thread. The answer is hostile: only
//! three small numbers pass, and only they reach the screen. Nothing is
//! downloaded or installed.

use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Where the latest version is read. Fixed: nothing of the user goes in it.
pub const LATEST_URL: &str = "https://andrelnfernandes.github.io/xylem/latest.txt";
/// Where the releases are, for the message. Fixed, not from the answer.
pub const RELEASES_URL: &str = "https://github.com/andrelnfernandes/xylem/releases";
/// Answers the question without asking: `0` no, `1` yes (over the file).
pub const ENV_VAR: &str = "XYLEM_CHECK_UPDATES";
/// Whether this build checks at all (the `update-check` feature, for
/// distributions that do not want it).
pub const ENABLED: bool = cfg!(feature = "update-check");

/// The most read from the answer; a version needs far less.
const MAX_ANSWER: usize = 64;
/// One check a day.
const INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// A version as three numbers, compared in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u16, pub u16, pub u16);

impl Version {
    /// `N.N.N`, each number of one to four digits, with blanks only around
    /// it. Anything else (a captive portal's page, a proxy's error, a
    /// `-rc1`) is not a version.
    pub fn parse(text: &[u8]) -> Option<Version> {
        if text.len() > MAX_ANSWER {
            return None;
        }
        let text = std::str::from_utf8(text).ok()?.trim();
        let mut parts = text.split('.');
        let mut number = || -> Option<u16> {
            let part = parts.next()?;
            if part.is_empty() || part.len() > 4 || !part.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            part.parse().ok()
        };
        let version = Version(number()?, number()?, number()?);
        parts.next().is_none().then_some(version)
    }

    /// The version of this binary.
    pub fn current() -> Version {
        Version::parse(env!("CARGO_PKG_VERSION").as_bytes()).unwrap_or(Version(0, 0, 0))
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

/// What to do at the start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Check,
    DontCheck,
    /// Not answered yet: ask once.
    Ask,
}

/// The environment variable first, then the file; neither, ask.
pub fn decide(option: Option<bool>, env: Option<OsString>) -> Decision {
    let from_env = env.and_then(|v| match v.to_str()?.trim() {
        "0" => Some(false),
        "1" => Some(true),
        _ => None,
    });
    match from_env.or(option) {
        Some(true) => Decision::Check,
        Some(false) => Decision::DontCheck,
        None => Decision::Ask,
    }
}

/// When the last check was, and what it found: kept in the state folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    /// Seconds since 1970 of the last attempt (found or not).
    pub checked: u64,
    pub latest: Option<Version>,
}

impl State {
    /// `checked=<seconds>` and `latest=<version>`, one a line; a file that
    /// says something else counts as never checked.
    pub fn parse(text: &str) -> State {
        let mut state = State::default();
        for line in text.lines().take(4) {
            match line.split_once('=') {
                Some(("checked", n)) => state.checked = n.trim().parse().unwrap_or(0),
                Some(("latest", v)) => state.latest = Version::parse(v.as_bytes()),
                _ => {}
            }
        }
        state
    }

    fn text(&self) -> String {
        let latest = self.latest.map(|v| v.to_string()).unwrap_or_default();
        format!("checked={}\nlatest={latest}\n", self.checked)
    }

    /// Whether there was no check yet, a day has gone by since the last one,
    /// or the clock went back.
    pub fn due(&self, now: u64) -> bool {
        self.checked == 0 || now < self.checked || now - self.checked >= INTERVAL.as_secs()
    }
}

pub fn state_path() -> Option<PathBuf> {
    Some(crate::platform::state_dir()?.join("update-check"))
}

fn load(path: &Path) -> State {
    std::fs::read(path)
        .ok()
        .filter(|bytes| bytes.len() <= 256)
        .map(|bytes| State::parse(&String::from_utf8_lossy(&bytes)))
        .unwrap_or_default()
}

fn save(path: &Path, state: &State) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let tmp = path.with_extension("tmp");
    if std::fs::write(&tmp, state.text()).is_ok() {
        let _ = std::fs::rename(&tmp, path);
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Fetches the answer: at most `MAX_ANSWER + 1` bytes, or nothing.
pub type FetchFn = fn() -> Option<Vec<u8>>;

/// The arguments of `curl`, separate (no shell): HTTPS only, no redirect
/// followed, five seconds at most, a tiny file at most, and nothing said
/// about the user but the program's version.
pub fn curl_args() -> Vec<String> {
    [
        "--silent",
        "--fail",
        "--proto",
        "=https",
        "--tlsv1.2",
        "--max-time",
        "5",
        "--max-filesize",
        "64",
        "--user-agent",
        concat!("xylem/", env!("CARGO_PKG_VERSION")),
        LATEST_URL,
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect()
}

/// `curl` itself: on Windows only the one of the system, so that a
/// `curl.exe` in the current folder is never run; elsewhere the `PATH`'s, as
/// with git.
fn curl_program() -> Option<PathBuf> {
    if cfg!(windows) {
        let root = std::env::var_os("SystemRoot")?;
        Some(PathBuf::from(root).join("System32").join("curl.exe"))
    } else {
        Some(PathBuf::from("curl"))
    }
}

/// The real fetch, through the system's `curl`.
pub fn fetch_with_curl() -> Option<Vec<u8>> {
    let mut child = Command::new(curl_program()?)
        .args(curl_args())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut answer = Vec::new();
    let read = child
        .stdout
        .take()
        .map(|out| out.take(MAX_ANSWER as u64 + 1).read_to_end(&mut answer));
    // More than a version: not waited for.
    if answer.len() > MAX_ANSWER {
        let _ = child.kill();
    }
    let status = child.wait().ok()?;
    (read.is_some_and(|r| r.is_ok()) && status.success()).then_some(answer)
}

/// A check started at the start or from F12.
pub struct UpdateCheck {
    result: Option<Receiver<Option<Version>>>,
    /// The latest version known, newer or not.
    pub latest: Option<Version>,
}

impl UpdateCheck {
    /// Uses what was found today; otherwise fetches in a thread, which also
    /// keeps the date and what it found (a failure keeps the previous
    /// version, and the next try is tomorrow).
    pub fn start(path: Option<PathBuf>, fetch: FetchFn) -> UpdateCheck {
        UpdateCheck::start_at(path, fetch, now())
    }

    fn start_at(path: Option<PathBuf>, fetch: FetchFn, now: u64) -> UpdateCheck {
        let state = path.as_deref().map(load).unwrap_or_default();
        if !state.due(now) {
            return UpdateCheck {
                result: None,
                latest: state.latest,
            };
        }
        let (tx, rx) = channel();
        let spawned = std::thread::Builder::new()
            .name("update check".into())
            .spawn(move || {
                let found = fetch().and_then(|answer| Version::parse(&answer));
                let state = State {
                    checked: now,
                    latest: found.or(state.latest),
                };
                if let Some(path) = &path {
                    save(path, &state);
                }
                let _ = tx.send(state.latest);
            });
        UpdateCheck {
            result: spawned.ok().map(|_| rx),
            latest: state.latest,
        }
    }

    /// Takes the thread's answer when it has come. Whether it did now.
    pub fn poll(&mut self) -> bool {
        let Some(rx) = &self.result else {
            return false;
        };
        match rx.try_recv() {
            Ok(latest) => {
                self.latest = latest.or(self.latest);
                self.result = None;
                true
            }
            Err(TryRecvError::Empty) => false,
            Err(TryRecvError::Disconnected) => {
                self.result = None;
                false
            }
        }
    }

    /// A check that already knows the latest version.
    #[cfg(test)]
    pub(crate) fn known(latest: Version) -> UpdateCheck {
        UpdateCheck {
            result: None,
            latest: Some(latest),
        }
    }

    #[cfg(test)]
    pub(crate) fn wait(&mut self) {
        if let Some(rx) = self.result.take()
            && let Ok(latest) = rx.recv_timeout(Duration::from_secs(10))
        {
            self.latest = latest.or(self.latest);
        }
    }

    /// The latest version, when it is newer than this one.
    pub fn newer(&self) -> Option<Version> {
        self.latest.filter(|v| *v > Version::current())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_three_small_numbers_are_a_version() {
        assert_eq!(Version::parse(b"0.10.0"), Some(Version(0, 10, 0)));
        assert_eq!(Version::parse(b"1.2.3\n"), Some(Version(1, 2, 3)));
        assert_eq!(Version::parse(b" 1.2.3\r\n"), Some(Version(1, 2, 3)));
        for bad in [
            &b""[..],
            b"\n",
            b"1.2",
            b"1.2.3.4",
            b"0.10.0-rc1",
            b"v0.10.0",
            b"1..3",
            b"12345.0.0",
            b"+1.2.3",
            b"1.2.3 4",
            b"<html><body>Login</body></html>",
            b"1.2.\x1b[31m3",
            b"\xff\xfe1.2.3",
            "１.２.３".as_bytes(),
        ] {
            assert_eq!(
                Version::parse(bad),
                None,
                "{:?}",
                String::from_utf8_lossy(bad)
            );
        }
        let long = format!("1.2.3{}", " ".repeat(MAX_ANSWER));
        assert_eq!(Version::parse(long.as_bytes()), None, "65 bytes or more");
    }

    #[test]
    fn versions_compare_by_number() {
        assert!(Version(0, 9, 0) < Version(0, 10, 0));
        assert!(Version(1, 0, 0) > Version(0, 99, 99));
        assert_eq!(Version(0, 10, 0).to_string(), "0.10.0");
        assert!(Version::current() > Version(0, 0, 0));
    }

    #[test]
    fn the_environment_answers_before_the_file() {
        let env = |s: &str| Some(OsString::from(s));
        assert_eq!(decide(None, None), Decision::Ask);
        assert_eq!(decide(Some(true), None), Decision::Check);
        assert_eq!(decide(Some(false), None), Decision::DontCheck);
        assert_eq!(decide(Some(true), env("0")), Decision::DontCheck);
        assert_eq!(decide(None, env("1")), Decision::Check);
        assert_eq!(decide(Some(false), env("yes")), Decision::DontCheck);
        assert_eq!(decide(None, env("")), Decision::Ask);
    }

    #[test]
    fn the_state_says_when_to_check_again() {
        let state = State::parse("checked=1000\nlatest=0.10.0\n");
        assert_eq!(state.latest, Some(Version(0, 10, 0)));
        let hours = |h: u64| 1000 + h * 3600;
        assert!(!state.due(hours(23)));
        assert!(state.due(hours(25)));
        assert!(state.due(999), "the clock went back");
        for corrupt in ["", "garbage", "checked=x\nlatest=<script>", "\u{0}\u{1}"] {
            let s = State::parse(corrupt);
            assert_eq!(s.checked, 0);
            assert_eq!(s.latest, None);
            assert!(s.due(hours(0)));
        }
        assert_eq!(State::parse(&state.text()), state);
    }

    #[test]
    fn curl_gets_fixed_arguments() {
        let args = curl_args();
        assert_eq!(args.last().map(String::as_str), Some(LATEST_URL));
        for flag in ["--fail", "--tlsv1.2", "--max-time", "--max-filesize"] {
            assert!(args.iter().any(|a| a == flag), "{flag}");
        }
        let proto = args.iter().position(|a| a == "--proto").unwrap();
        assert_eq!(args[proto + 1], "=https");
        assert!(!args.iter().any(|a| a == "--location" || a == "-L"));
        assert!(LATEST_URL.starts_with("https://"));
    }

    fn newer_answer() -> Option<Vec<u8>> {
        Some(b"999.0.0\n".to_vec())
    }
    fn hostile_answer() -> Option<Vec<u8>> {
        Some(b"\x1b]0;owned\x07 999.0.0".to_vec())
    }
    fn no_answer() -> Option<Vec<u8>> {
        None
    }
    fn must_not_fetch() -> Option<Vec<u8>> {
        panic!("fetched within the day")
    }

    #[test]
    fn a_check_keeps_what_it_found_for_a_day() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("state").join("update-check");
        let mut check = UpdateCheck::start_at(Some(path.clone()), newer_answer, 5000);
        check.wait();
        assert_eq!(check.newer(), Some(Version(999, 0, 0)));
        let kept = load(&path);
        assert_eq!(kept.checked, 5000);
        assert_eq!(kept.latest, Some(Version(999, 0, 0)));

        let again = UpdateCheck::start_at(Some(path.clone()), must_not_fetch, 5000 + 3600);
        assert_eq!(again.newer(), Some(Version(999, 0, 0)), "from the file");

        // A day later the fetch fails: the version found stays.
        let mut later = UpdateCheck::start_at(Some(path.clone()), no_answer, 5000 + 90_000);
        later.wait();
        assert_eq!(later.newer(), Some(Version(999, 0, 0)));
        assert_eq!(load(&path).checked, 5000 + 90_000);
    }

    #[test]
    fn a_hostile_answer_tells_nothing() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("update-check");
        let mut check = UpdateCheck::start_at(Some(path.clone()), hostile_answer, 5000);
        check.wait();
        assert_eq!(check.newer(), None);
        assert_eq!(load(&path).latest, None);
        assert!(!std::fs::read_to_string(&path).unwrap().contains('\u{1b}'));
    }

    #[test]
    fn an_older_or_equal_version_is_not_newer() {
        let check = UpdateCheck {
            result: None,
            latest: Some(Version::current()),
        };
        assert_eq!(check.newer(), None);
        let check = UpdateCheck {
            result: None,
            latest: Some(Version(0, 0, 1)),
        };
        assert_eq!(check.newer(), None);
    }
}
