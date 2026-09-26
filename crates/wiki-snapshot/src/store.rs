//! Idempotent writing of the snapshot: a file is rewritten only if it changes, so `git
//! status` after a `fetch` shows only the pages that actually changed. The write itself
//! (`write_if_changed`) is `wiki::write_if_changed`, re-exported here: `Dataset::write_dir`
//! needs the same idempotent write for `dataset/wiki/`, and the dependency only runs one way
//! (this crate depends on `wiki`), so the one definition lives there.

use std::collections::BTreeSet;
use std::path::Path;

pub use wiki::write_if_changed;

/// Deletes the files in `dir` not present in `keep`; returns the deleted names, in order.
/// A directory that doesn't exist has nothing to delete. Subdirectories are ignored.
pub fn prune(dir: &Path, keep: &BTreeSet<String>) -> std::io::Result<Vec<String>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let mut removed = Vec::new();
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !keep.contains(&name) {
            std::fs::remove_file(entry.path())?;
            removed.push(name);
        }
    }
    removed.sort();
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_only_when_changed_and_prune() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("a.txt");
        assert!(write_if_changed(&f, b"x").unwrap());
        assert!(!write_if_changed(&f, b"x").unwrap());
        assert!(write_if_changed(&f, b"y").unwrap());
        std::fs::write(d.path().join("b.txt"), "z").unwrap();
        let removed = prune(d.path(), &["a.txt".to_string()].into_iter().collect()).unwrap();
        assert_eq!(removed, vec!["b.txt"]);
        assert!(f.exists());
    }

    #[test]
    fn write_creates_parents_and_prune_tolerates_missing_dir() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("deep/er/a.txt");
        assert!(write_if_changed(&f, b"x").unwrap());
        assert_eq!(std::fs::read(&f).unwrap(), b"x");
        let removed = prune(&d.path().join("nope"), &BTreeSet::new()).unwrap();
        assert!(removed.is_empty());
    }
}
