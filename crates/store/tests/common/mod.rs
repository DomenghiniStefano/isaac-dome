//! What every test file here opens: a real file in a temporary directory, because
//! `Store::open` is what runs the migrations and that is half of what these tests are about.
//! The directory is returned with the store: dropping it would take the file with it.

use store::Store;
use tempfile::TempDir;

pub fn temp_store() -> (TempDir, Store) {
    let dir = tempfile::tempdir().expect("a temp dir");
    let store = Store::open(&dir.path().join("isaacdome.db")).expect("a fresh database opens");
    (dir, store)
}
