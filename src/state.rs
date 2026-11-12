//! State that survives between runs: the prompt history, in
//! `~/.local/state/xylem/history.toml` (or `$XDG_STATE_HOME/xylem/...`).

use std::collections::{BTreeMap, HashMap};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::platform;

pub type History = HashMap<String, Vec<String>>;

/// A directory bookmark, optionally annotated with a friendly label.
///
/// A folder on a remote machine or in a container keeps its address in
/// `path` (`ssh://user@host/dir`, `docker://name/dir`): a version of Xylem that
/// does not know them reads and writes it back as a path that does not open,
/// and nothing is lost.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Bookmark {
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// The longest name a bookmark can be given.
pub const LABEL_MAX: usize = 60;

/// How the address of a remote bookmark starts. A local path never does:
/// bookmarks are absolute paths (`/x`, `C:\x`).
const REMOTE_PREFIXES: [&str; 2] = ["ssh://", "docker://"];

impl Bookmark {
    #[cfg(test)]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            label: None,
        }
    }

    #[cfg(test)]
    pub fn with_label(path: impl Into<PathBuf>, label: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            label: Some(label.into()),
        }
    }

    /// The address of a remote bookmark, as saved; it is checked only when
    /// the bookmark is used (`remote::Address::parse`).
    pub fn address(&self) -> Option<&str> {
        let text = self.path.to_str()?;
        REMOTE_PREFIXES
            .iter()
            .any(|p| text.starts_with(p))
            .then_some(text)
    }

    /// The name to show: whatever the file says, without control or
    /// direction characters and at most [`LABEL_MAX`] long.
    pub fn label(&self) -> Option<String> {
        let label = self.label.as_deref()?.trim();
        (!label.is_empty()).then(|| {
            label
                .chars()
                .take(LABEL_MAX)
                .map(|c| {
                    if c.is_control() || crate::bidi::is_bidi(c) {
                        '·'
                    } else {
                        c
                    }
                })
                .collect()
        })
    }
}

/// A name typed for a bookmark: `None` to remove it, an error for what a
/// name cannot have.
pub fn check_label(text: &str) -> Result<Option<String>, String> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    if text.chars().count() > LABEL_MAX {
        return Err(format!(
            "A bookmark's name has at most {LABEL_MAX} characters"
        ));
    }
    if text
        .chars()
        .any(|c| c.is_control() || crate::bidi::is_bidi(c))
    {
        return Err("A bookmark's name cannot have control or text direction characters".into());
    }
    Ok(Some(text.to_string()))
}

#[derive(Deserialize)]
#[serde(untagged)]
enum BookmarkHelper {
    Simple(PathBuf),
    Full {
        path: PathBuf,
        label: Option<String>,
    },
}

impl<'de> Deserialize<'de> for Bookmark {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        BookmarkHelper::deserialize(deserializer).map(|h| match h {
            BookmarkHelper::Simple(path) => Bookmark { path, label: None },
            BookmarkHelper::Full { path, label } => Bookmark { path, label },
        })
    }
}

pub type Bookmarks = BTreeMap<String, Bookmark>;

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct StateFile {
    #[serde(default)]
    pub history: History,
    #[serde(default)]
    pub bookmarks: Bookmarks,
}

pub fn history_path() -> Option<PathBuf> {
    Some(platform::state_dir()?.join("history.toml"))
}

/// Loads full state (history and bookmarks); empty if the file doesn't exist or can't be read.
pub fn load_state(path: &Path) -> StateFile {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| toml::from_str::<StateFile>(&text).ok())
        .unwrap_or_default()
}

/// Saves the full state atomically with private permissions.
pub fn save_state(path: &Path, state: &StateFile) -> io::Result<()> {
    let text = toml::to_string(state).map_err(io::Error::other)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("toml.tmp");
    let mut file =
        platform::private(OpenOptions::new().write(true).create(true).truncate(true)).open(&tmp)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    fs::rename(&tmp, path)
}

/// Saved history; empty if the file doesn't exist or can't be read.
#[allow(dead_code)]
pub fn load_history(path: &Path) -> History {
    load_state(path).history
}

/// Saves the history while preserving any existing bookmarks.
pub fn save_history(path: &Path, history: &History) -> io::Result<()> {
    let mut state = load_state(path);
    state.history = history.clone();
    save_state(path, &state)
}

/// Saved bookmarks; empty if the file doesn't exist or can't be read.
#[allow(dead_code)]
pub fn load_bookmarks(path: &Path) -> Bookmarks {
    load_state(path).bookmarks
}

/// Saves bookmarks while preserving any existing history.
pub fn save_bookmarks(path: &Path, bookmarks: &Bookmarks) -> io::Result<()> {
    let mut state = load_state(path);
    state.bookmarks = bookmarks.clone();
    save_state(path, &state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_and_loads_privately() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("xylem/history.toml");
        assert!(load_history(&path).is_empty());

        let mut history = History::new();
        history.insert("execute".into(), vec!["tar czf 'a b.tgz' %t".into()]);
        history.insert("filespec".into(), vec!["*.rs".into(), "*.md -x\"y".into()]);
        save_history(&path, &history).unwrap();
        assert_eq!(load_history(&path), history);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        assert!(!path.with_extension("toml.tmp").exists());
    }

    #[test]
    fn ignores_broken_files() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("history.toml");
        fs::write(&path, "not [valid").unwrap();
        assert!(load_history(&path).is_empty());
        assert!(load_bookmarks(&path).is_empty());
    }

    #[test]
    fn bookmarks_persistence_and_preservation() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("history.toml");

        let mut history = History::new();
        history.insert("filespec".into(), vec!["*.rs".into()]);
        save_history(&path, &history).unwrap();

        let mut bookmarks = Bookmarks::new();
        bookmarks.insert("1".into(), Bookmark::new("/tmp/one"));
        bookmarks.insert("p".into(), Bookmark::with_label("/tmp/proj", "Projects"));
        save_bookmarks(&path, &bookmarks).unwrap();

        // Check both are preserved
        assert_eq!(load_history(&path), history);
        assert_eq!(load_bookmarks(&path), bookmarks);

        // Update history, verify bookmarks are still there
        history.insert("execute".into(), vec!["cargo check".into()]);
        save_history(&path, &history).unwrap();
        assert_eq!(load_bookmarks(&path), bookmarks);
        assert_eq!(load_history(&path), history);
    }

    #[test]
    fn bookmarks_deserializes_simple_and_full_formats() {
        let toml_str = r#"
[bookmarks]
"1" = "/home/user/simple"
"p" = { path = "/home/user/full", label = "Projects" }
"#;
        let state: StateFile = toml::from_str(toml_str).unwrap();
        assert_eq!(
            state.bookmarks.get("1"),
            Some(&Bookmark::new("/home/user/simple"))
        );
        assert_eq!(
            state.bookmarks.get("p"),
            Some(&Bookmark::with_label("/home/user/full", "Projects"))
        );
    }

    #[test]
    fn a_remote_bookmark_is_kept_in_its_path() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("history.toml");
        let mut bookmarks = Bookmarks::new();
        bookmarks.insert(
            "o".into(),
            Bookmark::with_label("ssh://oracle@db01:2222/u01/app", "Servidor do Oracle"),
        );
        bookmarks.insert("p".into(), Bookmark::new("docker://pg-amb1/var/lib"));
        save_bookmarks(&path, &bookmarks).unwrap();
        // Read as an older Xylem reads it: a path and a label, nothing else.
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            text.contains("path = \"ssh://oracle@db01:2222/u01/app\""),
            "{text}"
        );
        let back = load_bookmarks(&path);
        assert_eq!(back, bookmarks);
        assert_eq!(back["o"].address(), Some("ssh://oracle@db01:2222/u01/app"));
        assert_eq!(back["p"].address(), Some("docker://pg-amb1/var/lib"));
        assert_eq!(Bookmark::new("/home/a@b:c").address(), None);
        assert_eq!(Bookmark::new(r"C:\ssh\x").address(), None);
    }

    #[test]
    fn a_name_is_checked_when_typed_and_cleaned_when_read() {
        assert_eq!(check_label("  "), Ok(None));
        assert_eq!(
            check_label(" Docker do Postgres Amb 1 "),
            Ok(Some("Docker do Postgres Amb 1".into()))
        );
        assert!(check_label(&"x".repeat(LABEL_MAX)).is_ok());
        assert!(check_label(&"x".repeat(LABEL_MAX + 1)).is_err());
        assert!(check_label("a\u{1b}[31mb").is_err());
        assert!(check_label("fatura\u{202E}fdp").is_err());

        // From the file, as it may have been edited by hand.
        let bm = Bookmark::with_label("/x", "a\u{1b}b\u{202E}c");
        assert_eq!(bm.label().as_deref(), Some("a·b·c"));
        let long = Bookmark::with_label("/x", "y".repeat(500));
        assert_eq!(long.label().unwrap().chars().count(), LABEL_MAX);
        assert_eq!(Bookmark::with_label("/x", "  ").label(), None);
    }

    #[test]
    fn retrocompatible_with_old_history_without_bookmarks() {
        let toml_str = r#"
[history]
filespec = ["*.rs"]
"#;
        let state: StateFile = toml::from_str(toml_str).unwrap();
        assert_eq!(state.history["filespec"], vec!["*.rs"]);
        assert!(state.bookmarks.is_empty());
    }
}
