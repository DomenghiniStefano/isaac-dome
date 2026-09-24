//! Where `log.txt` and `online_logs\` are. Without this the watcher has nowhere to watch, and
//! `discovery` could not say: it reaches Documents only to look for `.dat` files, so with Steam
//! Cloud on — the ordinary case — the folder never surfaced at all.

use std::fs;

use discovery::for_tests::scan_game_data;

#[test]
fn the_folder_with_the_plus_is_found_with_everything_in_it() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp
        .path()
        .join("My Games")
        .join("Binding of Isaac Repentance+");
    fs::create_dir_all(dir.join("online_logs")).unwrap();
    fs::create_dir_all(dir.join("save_backups")).unwrap();
    fs::write(dir.join("log.txt"), b"[INFO] - hello").unwrap();

    let found = scan_game_data(Some(tmp.path()), None).expect("the folder is there");
    assert_eq!(found.dir, dir);
    assert_eq!(found.log, Some(dir.join("log.txt")));
    assert_eq!(found.online_logs, Some(dir.join("online_logs")));
    assert_eq!(found.save_backups, Some(dir.join("save_backups")));
}

#[test]
fn the_folder_without_the_plus_is_found_too() {
    // Both spellings exist on real machines, and the one without the `+` is the older install.
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp
        .path()
        .join("My Games")
        .join("Binding of Isaac Repentance");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("log.txt"), b"[INFO] - hello").unwrap();

    let found = scan_game_data(Some(tmp.path()), None).expect("the folder is there");
    assert_eq!(found.dir, dir);
    assert_eq!(found.log, Some(dir.join("log.txt")));
    assert_eq!(found.online_logs, None);
    assert_eq!(found.save_backups, None);
}

#[test]
fn the_one_with_the_plus_wins_when_both_exist() {
    // A machine that upgraded has both. The `+` one is the one the game writes to now.
    let tmp = tempfile::tempdir().unwrap();
    let games = tmp.path().join("My Games");
    fs::create_dir_all(games.join("Binding of Isaac Repentance")).unwrap();
    fs::create_dir_all(games.join("Binding of Isaac Repentance+")).unwrap();

    let found = scan_game_data(Some(tmp.path()), None).expect("a folder is there");
    assert_eq!(found.dir, games.join("Binding of Isaac Repentance+"));
}

#[test]
fn a_folder_that_exists_but_is_empty_is_still_the_folder() {
    // Degrade, never fail: the game was installed and never launched. The folder is the answer,
    // the three `None`s are the report.
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp
        .path()
        .join("My Games")
        .join("Binding of Isaac Repentance+");
    fs::create_dir_all(&dir).unwrap();

    let found = scan_game_data(Some(tmp.path()), None).expect("the folder is there");
    assert_eq!(found.log, None);
    assert_eq!(found.online_logs, None);
    assert_eq!(found.save_backups, None);
}

#[test]
fn nothing_there_is_none_and_not_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(scan_game_data(Some(tmp.path()), None).is_none());
}

#[test]
fn a_file_where_the_folder_should_be_is_not_a_folder() {
    // `read_dir` on a file is an error, and an error here would be a crash on someone's machine.
    let tmp = tempfile::tempdir().unwrap();
    let games = tmp.path().join("My Games");
    fs::create_dir_all(&games).unwrap();
    fs::write(games.join("Binding of Isaac Repentance+"), b"not a folder").unwrap();
    assert!(scan_game_data(Some(tmp.path()), None).is_none());
}

/// Card #80, P9: the folder the game says it writes to (B57) is its own answer, not a hint for
/// the Documents search. It was read only inside that search, so on a machine where
/// `dirs::document_dir()` answers nothing the declared folder was never tried.
#[test]
fn the_declared_folder_is_found_without_a_documents_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let declared = tmp.path().join("elsewhere");
    fs::create_dir_all(&declared).unwrap();
    fs::write(declared.join("log.txt"), b"[INFO] - hello").unwrap();

    let found = scan_game_data(None, Some(&declared)).expect("the declared folder is there");
    assert_eq!(found.dir, declared);
    assert_eq!(found.log, Some(declared.join("log.txt")));
}
