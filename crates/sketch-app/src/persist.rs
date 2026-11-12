//! The browser's `localStorage` stand-in: small files under
//! `%APPDATA%\SketchRust\`, read when the app starts and written when the
//! setting changes (`sketch.defaultOptions`, `sketch.areaCodes`,
//! `sketch.textLibraryLabels`, `sketch.symbolFavorites`; the recent files, the
//! interface theme and the default comments have their own, older helpers).
//!
//! Everything is opt-in: the real app's `main()` sets `SKETCH_PERSIST`, so the
//! test harness never reads or writes the user's files. A missing, unreadable
//! or invalid file just means "use the bundled default", as the original's
//! `try { JSON.parse(localStorage.getItem(...)) } catch (_) {}` does.

use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::Serialize;

// Where a test's private store lives (thread-local, so tests running side by
// side never see each other's files and the user's profile is never touched).
#[cfg(test)]
thread_local! {
    static TEST_DIR: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// Whether persistence is on: the real app's `main()` sets `SKETCH_PERSIST`;
/// a test turns it on only for itself, through `with_temp_store`.
pub fn enabled() -> bool {
    #[cfg(test)]
    {
        TEST_DIR.with(|d| d.borrow().is_some())
    }
    #[cfg(not(test))]
    {
        std::env::var_os("SKETCH_PERSIST").is_some()
    }
}

/// `%APPDATA%\SketchRust\<name>`.
pub fn path(name: &str) -> Option<PathBuf> {
    #[cfg(test)]
    {
        TEST_DIR.with(|d| d.borrow().as_ref().map(|dir| dir.join(name)))
    }
    #[cfg(not(test))]
    {
        let base = std::env::var("APPDATA").ok()?;
        Some(std::path::Path::new(&base).join("SketchRust").join(name))
    }
}

/// The stored text, if persistence is on and the file reads.
pub fn read(name: &str) -> Option<String> {
    if !enabled() {
        return None;
    }
    // A file edited by hand (Notepad, PowerShell) may start with a BOM.
    std::fs::read_to_string(path(name)?)
        .ok()
        .map(|t| t.trim_start_matches('\u{feff}').to_string())
}

/// The stored JSON value; `None` when absent or not valid for `T`.
pub fn read_json<T: DeserializeOwned>(name: &str) -> Option<T> {
    serde_json::from_str(&read(name)?).ok()
}

/// Writes `body`; `Some(true)` when stored, `Some(false)` when the write
/// failed, `None` when persistence is off.
pub fn write(name: &str, body: &str) -> Option<bool> {
    if !enabled() {
        return None;
    }
    let target = path(name)?;
    if let Some(dir) = target.parent() {
        if std::fs::create_dir_all(dir).is_err() {
            return Some(false);
        }
    }
    Some(crate::autosave::atomic_write(&target, body.as_bytes()).is_ok())
}

/// `write` of `value` as JSON.
pub fn write_json<T: Serialize>(name: &str, value: &T) -> Option<bool> {
    write(name, &serde_json::to_string(value).ok()?)
}

/// Runs `test` against a private, empty store with persistence on - for this
/// thread only.
#[cfg(test)]
pub fn with_temp_store(test: impl FnOnce()) {
    let test_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/test-profiles");
    std::fs::create_dir_all(&test_root).unwrap();
    let test_root = test_root.canonicalize().unwrap();
    let dir = test_root.join(format!(
        "profile-{}-{:?}-{}",
        std::process::id(),
        std::thread::current().id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    TEST_DIR.with(|d| *d.borrow_mut() = Some(dir.clone()));
    test();
    TEST_DIR.with(|d| *d.borrow_mut() = None);
    assert!(dir.canonicalize().unwrap().starts_with(&test_root));
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_round_trip_and_bad_files_read_as_nothing() {
        with_temp_store(|| {
            assert_eq!(read("x.json"), None, "nothing stored yet");
            assert_eq!(write_json("x.json", &vec!["a", "b"]), Some(true));
            assert_eq!(
                read_json::<Vec<String>>("x.json"),
                Some(vec!["a".to_string(), "b".to_string()])
            );
            // A file that is not valid for the type is ignored, not an error.
            write("x.json", "{not json").unwrap();
            assert_eq!(read_json::<Vec<String>>("x.json"), None);
        });
    }

    #[test]
    fn a_byte_order_mark_from_a_hand_edit_is_ignored() {
        with_temp_store(|| {
            write("bom.json", "\u{feff}[\"a\"]").unwrap();
            assert_eq!(
                read_json::<Vec<String>>("bom.json"),
                Some(vec!["a".to_string()])
            );
        });
    }

    #[test]
    fn nothing_is_read_or_written_unless_persistence_is_on() {
        assert_eq!(write("y.txt", "z"), None);
        assert_eq!(read("y.txt"), None);
    }
}
