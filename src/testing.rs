//! Helpers for the tests.

/// A temporary folder inside the user's own space (`~/.cache/xylem/tests`),
/// not in the shared `/tmp`: there, what other users leave around (a stray
/// `/tmp/.git`) gets in the way. Also outside Xylem's own repository, which
/// the Git tests need.
pub fn tempdir() -> tempfile::TempDir {
    let base = crate::platform::cache_dir()
        .expect("no home directory")
        .join("tests");
    std::fs::create_dir_all(&base).unwrap();
    tempfile::Builder::new()
        .prefix("xy-")
        .tempdir_in(base)
        .unwrap()
}

/// Hides a file or directory: the dot in the name is enough on Linux; on
/// Windows it takes the H attribute.
pub fn hide(path: &std::path::Path) {
    #[cfg(windows)]
    crate::platform::set_mode(path, 0x2).unwrap();
    let _ = path;
}

/// Waits, up to a second, until `dir` holds no `.nfs*`: on NFS a file removed
/// while open somewhere (by a process another test just forked, before its
/// `exec`) stays as one until it is closed.
pub fn wait_for_nfs(dir: &std::path::Path) {
    use std::time::{Duration, Instant};
    let deadline = Instant::now() + Duration::from_secs(1);
    let left = || {
        std::fs::read_dir(dir).is_ok_and(|mut entries| {
            entries.any(|e| e.is_ok_and(|e| e.file_name().to_string_lossy().starts_with(".nfs")))
        })
    };
    while left() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
}
