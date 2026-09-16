//! `savedatapath.txt` — the game's own answer to the question `discovery` has always had to
//! guess: which of the two Documents folders it saves into. The two spellings differ by one
//! character and which one exists depends on a history the app cannot see (B57).
//!
//! The strings below are the file's real contents on two different machines, recorded in B57 on
//! 2026-09-15 and 2026-09-16. The mixed separators are not a typo: the game writes
//! `C:\Users\stefa/Documents/...`, on both installs and two different Steam library roots, which
//! is what makes it something a parser may rely on rather than an observation.

use std::fs;

use discovery::for_tests::{declared_game_data, parse_save_data_path, scan_game_data};

const PLUS: &str = "This file is purely informational. Changing it will have no effect on saving or loading data.\r\n\r\nSave Data Path: C:\\Users\\stefa/Documents/My Games/Binding of Isaac Repentance+/\r\nModding Data Path: D:\\SteamLibrary\\steamapps\\common\\The Binding of Isaac Rebirth/mods/\r\n";

const NO_PLUS: &str = "This file is purely informational. Changing it will have no effect on saving or loading data.\r\n\r\nSave Data Path: C:\\Users\\stefa/Documents/My Games/Binding of Isaac Repentance/\r\nModding Data Path: C:\\Program Files (x86)\\Steam\\steamapps\\common\\The Binding of Isaac Rebirth/mods/\r\n";

#[test]
fn it_reads_the_folder_the_game_names() {
    let path = parse_save_data_path(PLUS).expect("the line is there");
    assert!(path.ends_with("Binding of Isaac Repentance+"));
}

#[test]
fn it_tells_the_two_spellings_apart() {
    // The whole point of the entry: the two differ by one character, and `discovery` chooses
    // between them by trying one and then the other.
    let plus = parse_save_data_path(PLUS).expect("the line is there");
    let plain = parse_save_data_path(NO_PLUS).expect("the line is there");
    assert!(plus.ends_with("Binding of Isaac Repentance+"));
    assert!(plain.ends_with("Binding of Isaac Repentance"));
    assert_ne!(plus, plain);
}

#[test]
fn the_trailing_separator_is_not_part_of_the_folder() {
    // The game writes a trailing `/`. Kept, it would make an empty last component and every
    // `ends_with` and `join` below it would be answering about the wrong thing.
    let path = parse_save_data_path(PLUS).expect("the line is there");
    assert_eq!(
        path.file_name().and_then(|n| n.to_str()),
        Some("Binding of Isaac Repentance+")
    );
}

#[test]
fn the_modding_path_is_not_mistaken_for_the_save_path() {
    // Both lines end in a folder and both are absolute. The label is the only thing telling
    // them apart, and the modding one names an install the launcher no longer lists.
    let path = parse_save_data_path(PLUS).expect("the line is there");
    assert!(!path.to_string_lossy().contains("mods"));
}

#[test]
fn a_file_without_the_line_says_nothing() {
    assert_eq!(
        parse_save_data_path("This file is purely informational."),
        None
    );
    assert_eq!(parse_save_data_path(""), None);
}

#[test]
fn a_line_with_nothing_after_the_label_says_nothing() {
    // Informational text, so a shape we have never seen is possible in principle. An empty
    // answer is not a folder, and returning one would be worse than returning none.
    assert_eq!(parse_save_data_path("Save Data Path:   \r\n"), None);
}

#[test]
fn the_file_is_read_from_the_install_when_it_is_there() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("savedatapath.txt"), PLUS).unwrap();
    let declared = declared_game_data(tmp.path()).expect("the file is there");
    assert!(declared.ends_with("Binding of Isaac Repentance+"));
}

#[test]
fn an_install_without_the_file_declares_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    assert_eq!(declared_game_data(tmp.path()), None);
}

#[test]
fn the_declared_folder_is_preferred_over_the_search() {
    // Both spellings exist — a machine that upgraded — and the search takes the `+` one first.
    // The game says it saves into the other, and the game is the one that cannot be wrong.
    let tmp = tempfile::tempdir().unwrap();
    let games = tmp.path().join("My Games");
    let plus = games.join("Binding of Isaac Repentance+");
    let plain = games.join("Binding of Isaac Repentance");
    fs::create_dir_all(&plus).unwrap();
    fs::create_dir_all(&plain).unwrap();

    let found = scan_game_data(tmp.path(), Some(&plain)).expect("the folder is there");
    assert_eq!(found.dir, plain);
}

#[test]
fn the_search_still_finds_the_folder_with_nothing_declared() {
    // **The test the entry asks for by name**: it turns red the day the file becomes the only
    // path that works. It exists only when the game is found, and the app has to work at a
    // stranger's house where it may not be.
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp
        .path()
        .join("My Games")
        .join("Binding of Isaac Repentance+");
    fs::create_dir_all(&dir).unwrap();

    let found = scan_game_data(tmp.path(), None).expect("the folder is there");
    assert_eq!(found.dir, dir);
}

#[test]
fn a_declared_folder_that_is_not_there_falls_back_to_the_search() {
    // The file is informational, so a mismatch is possible in principle — an old install, a
    // folder the user moved. It is a first candidate, checked like any other, never the answer.
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp
        .path()
        .join("My Games")
        .join("Binding of Isaac Repentance+");
    fs::create_dir_all(&dir).unwrap();

    let found =
        scan_game_data(tmp.path(), Some(&tmp.path().join("nowhere"))).expect("the search answers");
    assert_eq!(found.dir, dir);
}

#[test]
fn what_is_inside_the_declared_folder_is_reported_the_same_way() {
    // The declared folder is not a different kind of answer: the log, the online logs and the
    // backups are found in it exactly as in a searched one, and each only if it exists.
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("elsewhere").join("Isaac");
    fs::create_dir_all(dir.join("online_logs")).unwrap();
    fs::write(dir.join("log.txt"), b"[INFO] - hello").unwrap();

    let found = scan_game_data(tmp.path(), Some(&dir)).expect("the folder is there");
    assert_eq!(found.dir, dir);
    assert_eq!(found.log, Some(dir.join("log.txt")));
    assert_eq!(found.online_logs, Some(dir.join("online_logs")));
    assert_eq!(found.save_backups, None);
}
