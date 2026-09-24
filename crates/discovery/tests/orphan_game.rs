//! The leftover game folder: `steamapps\common\The Binding of Isaac Rebirth` still on
//! disk with no executable and no `resources\`, which is what an uninstall or a moved
//! library leaves behind.
//!
//! It was seen for real on 2026-09-08 — the only Steam library on that machine held
//! exactly this, and `discovery` correctly reported nothing. But an observation on one
//! machine isn't a test: it disappears the moment that PC installs the game, and it never
//! ran anywhere else. These fixtures build the orphan wherever the suite runs.

use std::fs;
use std::path::{Path, PathBuf};

use discovery::for_tests::{find_game, find_steam};
use discovery::{Diagnostic, Options};

const INSTALLDIR: &str = "The Binding of Isaac Rebirth";

fn opts(steam_root: &Path) -> Options {
    Options {
        steam_root: Some(steam_root.to_path_buf()),
        game_dir: None,
        save_dir: None,
    }
}

/// The orphan as it actually appears on disk: the folder, its two subfolders, the stray
/// text file Steam leaves, and nothing that makes it a game — no executable, no
/// `resources\`.
fn orphan_folder(library: &Path) -> PathBuf {
    let dir = library.join("steamapps").join("common").join(INSTALLDIR);
    fs::create_dir_all(dir.join("data")).unwrap();
    fs::create_dir_all(dir.join("mods")).unwrap();
    fs::write(dir.join("savedatapath.txt"), b"C:\\Users\\x\\Documents").unwrap();
    dir
}

fn write_manifest(library: &Path, body: &str) {
    let dir = library.join("steamapps");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("appmanifest_250900.acf"), body).unwrap();
}

fn discover_game(steam_root: &Path) -> (Option<PathBuf>, Vec<Diagnostic>) {
    let o = opts(steam_root);
    let (steam, _) = find_steam(&o);
    let (game, diags) = find_game(&o, steam.as_ref());
    (game.map(|g| g.dir), diags)
}

/// The registered case. Steam's own record of what is installed is the appmanifest, so a
/// folder without one is a leftover, not an installation — reporting it would send the
/// rest of the app looking for archives inside an empty directory.
#[test]
fn a_leftover_folder_without_a_manifest_is_not_an_installation() {
    let tmp = tempfile::tempdir().unwrap();
    orphan_folder(tmp.path());

    let (dir, diags) = discover_game(tmp.path());

    assert_eq!(dir, None, "the leftover must not pass for an installation");
    assert!(
        diags.contains(&Diagnostic::GameNotFound),
        "and it must say so: {diags:?}"
    );
}

/// The other half of the pair, and what makes the one above mean something: with the
/// manifest there, the very same folder *is* the installation. So it's the manifest that
/// decides, not the folder's name.
#[test]
fn the_same_folder_with_a_manifest_is_an_installation() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = orphan_folder(tmp.path());
    write_manifest(
        tmp.path(),
        "\"AppState\"\n{\n\t\"installdir\"\t\t\"The Binding of Isaac Rebirth\"\n}\n",
    );

    let (found, _) = discover_game(tmp.path());

    assert_eq!(found.as_deref(), Some(dir.as_path()));
}

/// The opposite mistake, and the worse one: Steam's manifest says the game is installed
/// and the folder it names isn't there — an interrupted uninstall, or a library on a
/// drive that isn't plugged in.
///
/// Handing back a directory that doesn't exist isn't "degrade", it's **false data with a
/// confident face**: the rest of the app then goes looking for archives inside it and the
/// user is told extraction failed, when what actually happened is that the game isn't
/// there. Not finding it is an expected case, and expected cases travel as diagnostics.
///
/// The same function already applies this rule one branch over — the malformed-manifest
/// fallback checks `canonical.exists()` before returning it.
#[test]
fn a_manifest_pointing_at_a_missing_folder_is_not_an_installation() {
    let tmp = tempfile::tempdir().unwrap();
    write_manifest(
        tmp.path(),
        "\"AppState\"\n{\n\t\"installdir\"\t\t\"The Binding of Isaac Rebirth\"\n}\n",
    );

    let (found, diags) = discover_game(tmp.path());

    assert_eq!(found, None, "the folder the manifest names isn't on disk");
    assert!(
        diags.contains(&Diagnostic::GameNotFound),
        "and the absence has to be said: {diags:?}"
    );
}

/// Why a missing folder makes the scan *continue* rather than give up: two libraries is
/// the ordinary Windows setup — the system disk and a second drive — and the one holding
/// a stale manifest must not shadow the one holding the game. This is the case that ruled
/// out "return None on the first library that doesn't check out".
#[test]
fn a_stale_manifest_does_not_hide_the_library_that_has_the_game() {
    let tmp = tempfile::tempdir().unwrap();
    let stale = tmp.path().join("stale");
    let real = tmp.path().join("real");
    fs::create_dir_all(&stale).unwrap();
    fs::create_dir_all(&real).unwrap();
    let manifest = "\"AppState\"\n{\n\t\"installdir\"\t\t\"The Binding of Isaac Rebirth\"\n}\n";
    write_manifest(&stale, manifest);
    write_manifest(&real, manifest);
    let dir = orphan_folder(&real);

    let o = Options {
        steam_root: Some(tmp.path().to_path_buf()),
        game_dir: None,
        save_dir: None,
    };
    let steam = discovery::SteamInstall {
        root: tmp.path().to_path_buf(),
        source: discovery::SteamSource::Override,
        libraries: vec![stale, real],
    };
    let (game, _) = find_game(&o, Some(&steam));

    assert_eq!(
        game.map(|g| g.dir).as_deref(),
        Some(dir.as_path()),
        "the second library holds it"
    );
}

/// Card #80, P9: a game folder chosen by hand (B14) comes with no appmanifest, so nothing says
/// which edition it is. It used to read as Rebirth — a claim about the user's copy made from
/// the absence of a file. The edition is unknown there, and says so.
#[test]
fn a_game_folder_chosen_by_hand_has_no_edition_it_can_claim() {
    let tmp = tempfile::tempdir().unwrap();
    let chosen = Options {
        steam_root: None,
        game_dir: Some(tmp.path().to_path_buf()),
        save_dir: None,
    };
    let (game, _) = find_game(&chosen, None);
    let game = game.expect("a chosen folder is taken as the game");
    assert_eq!(game.edition, None);
}
