//! Degradation tests for find_steam / find_game.

use std::fs;
use std::path::Path;

use discovery::testing::{find_game, scan_override};
use discovery::{Diagnostic, Edition, Options, SteamInstall, SteamSource};

fn mkdir(path: &Path) {
    fs::create_dir_all(path).unwrap();
}

/// Minimal valid ACF with one dlcappid (Afterbirth).
const MINIMAL_ACF: &str = r#"
"AppState"
{
	"appid"		"250900"
	"installdir"	"The Binding of Isaac Rebirth"
	"InstalledDepots"
	{
		"250905"
		{
			"dlcappid"	"401920"
		}
	}
}
"#;

// ---------------------------------------------------------------------------
// MalformedManifest + fallback to the canonical folder
// ---------------------------------------------------------------------------

#[test]
fn malformed_manifest_falls_back_to_canonical_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let library = tmp.path().to_path_buf();

    // Garbage, non-VDF .acf
    let steamapps = library.join("steamapps");
    mkdir(&steamapps);
    fs::write(
        steamapps.join("appmanifest_250900.acf"),
        b"{{{{ non vdf ????",
    )
    .unwrap();

    // Canonical folder present
    let canonical = steamapps
        .join("common")
        .join("The Binding of Isaac Rebirth");
    mkdir(&canonical);

    let steam = SteamInstall {
        root: library.clone(),
        source: SteamSource::Override,
        libraries: vec![library.clone()],
    };

    let (game, diags) = find_game(&Options::default(), Some(&steam));

    // The game must be found with edition Rebirth (no DLC from the .acf)
    let game = game.expect("must find the game via the canonical fallback");
    assert_eq!(
        game.edition,
        Edition::Rebirth,
        "no DLC from the malformed manifest"
    );
    assert_eq!(
        game.updated_unix, None,
        "no update date without a readable manifest"
    );

    // MalformedManifest present
    let has_malformed = diags
        .iter()
        .any(|d| matches!(d, Diagnostic::MalformedManifest { .. }));
    assert!(has_malformed, "must emit MalformedManifest: {diags:?}");

    // GameNotFound must NOT be present alongside a found game
    let has_not_found = diags.iter().any(|d| matches!(d, Diagnostic::GameNotFound));
    assert!(
        !has_not_found,
        "GameNotFound must not appear when the game is found: {diags:?}"
    );
}

#[test]
fn malformed_manifest_without_canonical_dir_continues_to_next_library() {
    let tmp = tempfile::tempdir().unwrap();

    // First library: malformed .acf, canonical folder absent
    let lib1 = tmp.path().join("lib1");
    let steamapps1 = lib1.join("steamapps");
    mkdir(&steamapps1);
    fs::write(steamapps1.join("appmanifest_250900.acf"), b"spazzatura").unwrap();
    // We do NOT create the canonical folder in lib1

    // Second library: valid .acf + folder present
    let lib2 = tmp.path().join("lib2");
    let steamapps2 = lib2.join("steamapps");
    mkdir(&steamapps2);
    fs::write(
        steamapps2.join("appmanifest_250900.acf"),
        MINIMAL_ACF.as_bytes(),
    )
    .unwrap();
    mkdir(
        &steamapps2
            .join("common")
            .join("The Binding of Isaac Rebirth"),
    );

    let steam = SteamInstall {
        root: lib1.clone(),
        source: SteamSource::Override,
        libraries: vec![lib1, lib2],
    };

    let (game, diags) = find_game(&Options::default(), Some(&steam));

    let game = game.expect("must find the game in the second library");
    assert_eq!(
        game.edition,
        Edition::Afterbirth,
        "must read the DLC from the second library's manifest"
    );

    let has_malformed = diags
        .iter()
        .any(|d| matches!(d, Diagnostic::MalformedManifest { .. }));
    assert!(
        has_malformed,
        "MalformedManifest from the first library must be there: {diags:?}"
    );

    let has_not_found = diags.iter().any(|d| matches!(d, Diagnostic::GameNotFound));
    assert!(
        !has_not_found,
        "GameNotFound must not appear when the game is found: {diags:?}"
    );
}

// ---------------------------------------------------------------------------
// Multi-library selection
// ---------------------------------------------------------------------------

#[test]
fn finds_game_in_second_library_when_absent_from_first() {
    let tmp = tempfile::tempdir().unwrap();

    // First library: no appmanifest_250900.acf
    let lib1 = tmp.path().join("lib1");
    mkdir(&lib1.join("steamapps"));

    // Second library: valid manifest + folder
    let lib2 = tmp.path().join("lib2");
    let steamapps2 = lib2.join("steamapps");
    mkdir(&steamapps2);
    fs::write(
        steamapps2.join("appmanifest_250900.acf"),
        MINIMAL_ACF.as_bytes(),
    )
    .unwrap();
    mkdir(
        &steamapps2
            .join("common")
            .join("The Binding of Isaac Rebirth"),
    );

    let steam = SteamInstall {
        root: lib1.clone(),
        source: SteamSource::Override,
        libraries: vec![lib1, lib2.clone()],
    };

    let (game, diags) = find_game(&Options::default(), Some(&steam));

    let game = game.expect("must find the game in the second library");
    // The install folder must be under lib2
    assert!(
        game.dir.starts_with(&lib2),
        "the game must be in lib2, not lib1: {}",
        game.dir.display()
    );
    assert_eq!(game.edition, Edition::Afterbirth);

    let has_not_found = diags.iter().any(|d| matches!(d, Diagnostic::GameNotFound));
    assert!(!has_not_found, "GameNotFound must not appear: {diags:?}");
}

// ---------------------------------------------------------------------------
// UnreadablePath from scan_override on a FILE (not a directory)
// ---------------------------------------------------------------------------

#[test]
fn scan_override_on_file_path_yields_unreadable_path() {
    let tmp = tempfile::tempdir().unwrap();
    // Create a FILE and pass it as if it were a folder
    let file_path = tmp.path().join("not_a_directory.dat");
    fs::write(&file_path, b"contenuto").unwrap();

    let (_saves, diags) = scan_override(&file_path);

    let has_unreadable = diags
        .iter()
        .any(|d| matches!(d, Diagnostic::UnreadablePath { .. }));
    assert!(
        has_unreadable,
        "read_dir on a file must produce UnreadablePath: {diags:?}"
    );
}

// ---------------------------------------------------------------------------
// Update date from the appmanifest
// ---------------------------------------------------------------------------

#[test]
fn game_carries_last_updated_from_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    let library = tmp.path().to_path_buf();
    let steamapps = library.join("steamapps");
    mkdir(&steamapps);
    let acf = MINIMAL_ACF.replace(
        "\t\"installdir\"",
        "\t\"LastUpdated\"\t\"1757000000\"\n\t\"installdir\"",
    );
    fs::write(steamapps.join("appmanifest_250900.acf"), acf.as_bytes()).unwrap();
    mkdir(
        &steamapps
            .join("common")
            .join("The Binding of Isaac Rebirth"),
    );

    let steam = SteamInstall {
        root: library.clone(),
        source: SteamSource::Override,
        libraries: vec![library],
    };

    let (game, _) = find_game(&Options::default(), Some(&steam));
    let game = game.expect("must find the game");
    assert_eq!(game.updated_unix, Some(1_757_000_000));
}

// ---------------------------------------------------------------------------
// Leftover without a manifest: folder left behind after an uninstall
// ---------------------------------------------------------------------------

/// Steam leaves `steamapps\common\The Binding of Isaac Rebirth\` behind with some files
/// in it (screenshots, mods, mod saves) even after the game has been uninstalled. Without
/// `appmanifest_250900.acf` that folder **is not** an installation: reporting it would
/// have the app try to open archives that aren't there and tell the user the game is
/// present when it isn't.
///
/// The case is no longer reproducible on the dev machine — the game is installed there —
/// so it lives here, in a temp directory, instead of in a test on real data.
#[test]
fn a_leftover_game_folder_without_a_manifest_is_not_an_installation() {
    let tmp = tempfile::tempdir().unwrap();
    let library = tmp.path().to_path_buf();
    let residuo = library
        .join("steamapps")
        .join("common")
        .join("The Binding of Isaac Rebirth");
    mkdir(&residuo);
    // A plausible leftover: no executable, no `resources\packed`.
    fs::write(residuo.join("README.txt"), b"resti").unwrap();
    mkdir(&library.join("steamapps"));

    let steam = SteamInstall {
        root: library.clone(),
        source: SteamSource::Override,
        libraries: vec![library],
    };

    let (game, diags) = find_game(&Options::default(), Some(&steam));
    assert!(game.is_none(), "the leftover is not an installation");
    assert_eq!(diags, vec![Diagnostic::GameNotFound]);
}
