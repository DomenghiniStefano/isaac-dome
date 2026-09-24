// A test that extracts one variant panics on every other, the ones added later included: here
// the wildcard *is* the assertion, and it fails loudly on a new variant instead of hiding it.
#![allow(clippy::wildcard_enum_match_arm)]

use std::fs;
use std::path::Path;

use discovery::for_tests::{scan_documents, scan_override, scan_userdata};
use discovery::{Diagnostic, SavePrefix, SaveSource};

fn touch(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"x").unwrap();
}

#[test]
fn scans_all_accounts_in_userdata() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    touch(&root.join("userdata/111/250900/remote/rep+persistentgamedata1.dat"));
    touch(&root.join("userdata/111/250900/remote/rep+persistentgamedata2.dat"));
    touch(&root.join("userdata/222/250900/remote/rep_persistentgamedata1.dat"));
    // Noise that must not get in:
    touch(&root.join("userdata/111/250900/remote/options.ini"));
    touch(&root.join("userdata/333/999999/remote/rep+persistentgamedata1.dat"));

    let (mut saves, diags) = scan_userdata(root);
    saves.sort_by_key(|c| c.path.clone());
    assert_eq!(saves.len(), 3, "two accounts with Isaac, three valid files");
    assert!(diags.is_empty(), "no errors: {diags:?}");

    let accounts: Vec<_> = saves
        .iter()
        .filter_map(|c| match &c.source {
            SaveSource::SteamCloud { account_id } => Some(account_id.clone()),
            _ => None,
        })
        .collect();
    assert!(accounts.contains(&"111".to_string()));
    assert!(accounts.contains(&"222".to_string()));
    assert!(
        !accounts.contains(&"333".to_string()),
        "999999 is not Isaac"
    );

    let slot2 = saves.iter().find(|c| c.slot == 2).unwrap();
    assert_eq!(slot2.prefix, SavePrefix::RepPlus);
    assert!(slot2.size > 0);
}

#[test]
fn missing_userdata_is_not_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    let (saves, diags) = scan_userdata(tmp.path());
    assert!(saves.is_empty());
    assert!(
        diags.is_empty(),
        "the absence of userdata is normal, not an error"
    );
}

#[test]
fn scans_documents_both_folder_names() {
    let tmp = tempfile::tempdir().unwrap();
    let docs = tmp.path();
    touch(&docs.join("My Games/Binding of Isaac Repentance+/rep+persistentgamedata1.dat"));
    touch(&docs.join("My Games/Binding of Isaac Repentance/rep_persistentgamedata1.dat"));
    // Dated backups must NOT get in:
    touch(&docs.join(
        "My Games/Binding of Isaac Repentance+/save_backups/20250626.rep+persistentgamedata1.dat",
    ));

    let (saves, diags) = scan_documents(docs);
    assert_eq!(saves.len(), 2, "one file per folder, backups excluded");
    assert!(saves
        .iter()
        .all(|c| matches!(c.source, SaveSource::Documents { .. })));
    assert!(diags.is_empty());
}

#[test]
fn override_dir_yields_override_source() {
    let tmp = tempfile::tempdir().unwrap();
    touch(&tmp.path().join("rep+persistentgamedata1.dat"));
    let (saves, diags) = scan_override(tmp.path());
    assert_eq!(saves.len(), 1);
    assert!(matches!(saves[0].source, SaveSource::Override));
    assert!(diags.is_empty());
    let _ = Diagnostic::NoSavesFound;
}
