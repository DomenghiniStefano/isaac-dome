use discovery::{
    Diagnostic, Discovery, Dlc, Edition, GameInstall, SaveCandidate, SavePrefix, SaveSource,
    SteamInstall, SteamSource,
};
use ipc::{
    candidates, profile_id, resolve_active, setup_state, ActiveProfile, CandidateSource,
    ChoiceReason, IconRef, MissingReason, SetupDiagnostic,
};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

#[test]
fn profile_id_is_deterministic() {
    let a = profile_id(Path::new(
        r"C:\Steam\userdata\1\250900\remote\rep+persistentgamedata1.dat",
    ));
    let b = profile_id(Path::new(
        r"C:\Steam\userdata\1\250900\remote\rep+persistentgamedata1.dat",
    ));
    assert_eq!(a, b);
}

#[test]
fn profile_id_ignores_case_and_separators() {
    let a = profile_id(Path::new(r"C:\Steam\Remote\rep+persistentgamedata1.dat"));
    let b = profile_id(Path::new("c:/steam/remote/rep+persistentgamedata1.dat"));
    assert_eq!(a, b, "same file written two different ways = same id");
}

#[test]
fn different_paths_give_different_ids() {
    let a = profile_id(Path::new("c:/steam/remote/rep+persistentgamedata1.dat"));
    let b = profile_id(Path::new("c:/steam/remote/rep+persistentgamedata2.dat"));
    assert_ne!(a, b);
}

#[test]
fn profile_id_is_not_the_path() {
    let id = profile_id(Path::new("c:/steam/remote/rep+persistentgamedata1.dat"));
    assert!(
        !id.as_str().contains('/'),
        "the id must not contain the path"
    );
    assert!(!id.as_str().contains("remote"));
}

fn candidate(name: &str, slot: u8, prefix: SavePrefix, secs: Option<u64>) -> SaveCandidate {
    SaveCandidate {
        path: PathBuf::from(format!("c:/steam/remote/{name}")),
        slot,
        source: SaveSource::SteamCloud {
            account_id: "123456789".into(),
        },
        prefix,
        modified: secs.map(|s| SystemTime::UNIX_EPOCH + Duration::from_secs(s)),
        size: 14491,
    }
}

#[test]
fn candidates_are_sorted_newest_first_and_one_is_suggested() {
    let views = candidates(&[
        candidate(
            "rep_persistentgamedata1.dat",
            1,
            SavePrefix::Rep,
            Some(1_000),
        ),
        candidate(
            "rep+persistentgamedata1.dat",
            1,
            SavePrefix::RepPlus,
            Some(2_000),
        ),
    ]);
    assert_eq!(views.len(), 2);
    assert_eq!(
        views[0].prefix,
        SavePrefix::RepPlus,
        "the most recent comes first"
    );
    assert!(views[0].suggested);
    assert!(!views[1].suggested);
}

#[test]
fn suggestion_is_stable_when_dates_are_missing() {
    let a = candidates(&[
        candidate("rep+persistentgamedata2.dat", 2, SavePrefix::RepPlus, None),
        candidate("rep+persistentgamedata1.dat", 1, SavePrefix::RepPlus, None),
    ]);
    let b = candidates(&[
        candidate("rep+persistentgamedata1.dat", 1, SavePrefix::RepPlus, None),
        candidate("rep+persistentgamedata2.dat", 2, SavePrefix::RepPlus, None),
    ]);
    assert_eq!(
        a[0].id, b[0].id,
        "without dates, the order doesn't depend on the input"
    );
    assert_eq!(a[0].slot, 1, "on a tie, the lowest slot wins");
}

#[test]
fn candidate_view_hides_the_steam_account_id() {
    let realistic = SaveCandidate {
        path: PathBuf::from(
            "c:/steam/userdata/123456789/250900/remote/rep+persistentgamedata1.dat",
        ),
        slot: 1,
        source: SaveSource::SteamCloud {
            account_id: "123456789".into(),
        },
        prefix: SavePrefix::RepPlus,
        modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        size: 14491,
    };
    let views = candidates(&[realistic]);
    let json = serde_json::to_string(&views[0]).unwrap();
    assert!(
        !json.contains("123456789"),
        "the Steam account id doesn't cross the IPC boundary"
    );
    assert!(json.contains("steamCloud"));
    assert!(json.contains("modifiedUnix"), "fields in camelCase");
    assert!(
        json.contains("<account>"),
        "path_hint redacts the account id"
    );
}

#[test]
fn source_is_mapped_to_a_provenance_without_data() {
    let views = candidates(&[candidate(
        "rep+persistentgamedata1.dat",
        1,
        SavePrefix::RepPlus,
        Some(1),
    )]);
    assert_eq!(views[0].source, CandidateSource::SteamCloud);
}

#[test]
fn documents_source_hides_the_windows_username() {
    // A save under Documents doesn't go through `userdata\`, so there's no account id
    // to redact — but the path always starts with `C:\Users\<name>`, and that name is
    // the person's name. `CLAUDE.md`: a path *always* carries the Windows username.
    // The expected value comes from that rule, not from what the code does today.
    let from_documents = SaveCandidate {
        path: PathBuf::from(
            "c:/users/alice/documents/my games/binding of isaac/rep+persistentgamedata1.dat",
        ),
        slot: 1,
        source: SaveSource::Documents {
            folder: "c:/users/alice/documents/my games/binding of isaac".into(),
        },
        prefix: SavePrefix::RepPlus,
        modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        size: 14491,
    };
    let views = candidates(&[from_documents]);
    let json = serde_json::to_string(&views[0]).unwrap();

    assert!(
        !json.contains("alice"),
        "the Windows username doesn't cross the IPC boundary"
    );
    // The rest of the path stays: `path_hint` exists to say "found here", and without
    // the recognizable folders it would say nothing at all.
    assert!(
        views[0].path_hint.contains("my games") && views[0].path_hint.contains("binding of isaac"),
        "the part that orients the user stays readable"
    );
    assert!(
        views[0].path_hint.contains("<utente>"),
        "the gap is declared, not silently removed"
    );
}

#[test]
fn the_username_is_masked_whatever_the_source() {
    // This also holds for a manual override: the rule is about the path, not the source.
    let manual = SaveCandidate {
        path: PathBuf::from("C:\\Users\\Bob\\Downloads\\rep+persistentgamedata2.dat"),
        slot: 2,
        source: SaveSource::Override,
        prefix: SavePrefix::RepPlus,
        modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        size: 14491,
    };
    let views = candidates(&[manual]);
    assert!(
        !views[0].path_hint.contains("Bob"),
        "override: the username is masked all the same"
    );
    assert!(views[0].path_hint.contains("Downloads"));
}

#[test]
fn a_path_without_a_user_directory_is_left_alone() {
    // A game on a second library doesn't go through `Users\`: there's nothing to mask
    // and the path must not be mangled.
    let second_library = SaveCandidate {
        path: PathBuf::from("d:/steamlibrary/steamapps/common/rep+persistentgamedata1.dat"),
        slot: 1,
        source: SaveSource::Override,
        prefix: SavePrefix::RepPlus,
        modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1)),
        size: 14491,
    };
    let views = candidates(&[second_library]);
    assert!(
        !views[0].path_hint.contains("<utente>"),
        "nothing to mask, no placeholder"
    );
    assert!(views[0].path_hint.contains("steamlibrary"));
}

#[test]
fn save_prefix_is_serialized_in_snake_case() {
    let views = candidates(&[candidate(
        "rep+persistentgamedata1.dat",
        1,
        SavePrefix::RepPlus,
        Some(1),
    )]);
    let json = serde_json::to_string(&views[0]).unwrap();
    // SavePrefix is snake_case because it comes from discovery; CandidateSource is
    // camelCase because it's defined in ipc. The expected TypeScript types are
    // 'rep' | 'rep_plus'.
    assert!(
        json.contains("\"prefix\":\"rep_plus\""),
        "SavePrefix::RepPlus must come out as 'rep_plus' in JSON"
    );
    assert!(
        json.contains("\"source\":\"steamCloud\""),
        "CandidateSource has no fields: on the wire it's a string, not an object"
    );
}

#[test]
fn no_candidates_reports_where_the_chain_broke() {
    match resolve_active(None, &[], false, false) {
        ActiveProfile::None { reason } => assert_eq!(reason, MissingReason::SteamNotFound),
        other => panic!("expected None, got {other:?}"),
    }
    match resolve_active(None, &[], true, false) {
        ActiveProfile::None { reason } => assert_eq!(reason, MissingReason::GameNotFound),
        other => panic!("expected None, got {other:?}"),
    }
    match resolve_active(None, &[], true, true) {
        ActiveProfile::None { reason } => assert_eq!(reason, MissingReason::NoSaves),
        other => panic!("expected None, got {other:?}"),
    }
}

#[test]
fn a_single_candidate_is_selected_but_declared() {
    let views = candidates(&[candidate(
        "rep+persistentgamedata1.dat",
        1,
        SavePrefix::RepPlus,
        Some(1),
    )]);
    match resolve_active(None, &views, true, true) {
        ActiveProfile::Active {
            auto_selected,
            profile,
        } => {
            assert!(
                auto_selected,
                "must declare that the app chose, not the user"
            );
            assert_eq!(profile.slot, 1);
        }
        other => panic!("expected Active, got {other:?}"),
    }
}

#[test]
fn several_candidates_and_no_choice_asks_the_user() {
    let views = candidates(&[
        candidate(
            "rep+persistentgamedata1.dat",
            1,
            SavePrefix::RepPlus,
            Some(2_000),
        ),
        candidate(
            "rep_persistentgamedata1.dat",
            1,
            SavePrefix::Rep,
            Some(1_000),
        ),
    ]);
    match resolve_active(None, &views, true, true) {
        ActiveProfile::NeedsChoice { reason, suggested } => {
            assert_eq!(reason, ChoiceReason::NeverChosen);
            assert_eq!(
                suggested.as_ref(),
                Some(&views[0].id),
                "the most recent is suggested"
            );
        }
        other => panic!("expected NeedsChoice, got {other:?}"),
    }
}

#[test]
fn a_saved_choice_that_still_exists_wins() {
    let views = candidates(&[
        candidate(
            "rep+persistentgamedata1.dat",
            1,
            SavePrefix::RepPlus,
            Some(2_000),
        ),
        candidate(
            "rep_persistentgamedata1.dat",
            1,
            SavePrefix::Rep,
            Some(1_000),
        ),
    ]);
    let chosen = views[1].id.clone();
    match resolve_active(Some(&chosen), &views, true, true) {
        ActiveProfile::Active {
            auto_selected,
            profile,
        } => {
            assert!(!auto_selected, "the user chose");
            assert_eq!(
                profile.id, chosen,
                "the saved choice wins, not the suggestion"
            );
        }
        other => panic!("expected Active, got {other:?}"),
    }
}

#[test]
fn a_saved_choice_that_vanished_never_falls_back() {
    let views = candidates(&[
        candidate(
            "rep+persistentgamedata1.dat",
            1,
            SavePrefix::RepPlus,
            Some(2_000),
        ),
        candidate(
            "rep_persistentgamedata1.dat",
            1,
            SavePrefix::Rep,
            Some(1_000),
        ),
    ]);
    let gone = profile_id(Path::new("c:/steam/remote/rep+persistentgamedata3.dat"));
    match resolve_active(Some(&gone), &views, true, true) {
        ActiveProfile::NeedsChoice { reason, .. } => {
            assert!(matches!(reason, ChoiceReason::SavedProfileGone { .. }));
        }
        other => panic!("must never fall back to a different profile: {other:?}"),
    }
}

#[test]
fn a_single_candidate_that_is_not_the_saved_one_still_asks() {
    let views = candidates(&[candidate(
        "rep+persistentgamedata1.dat",
        1,
        SavePrefix::RepPlus,
        Some(1),
    )]);
    let gone = profile_id(Path::new("c:/steam/remote/rep+persistentgamedata3.dat"));
    match resolve_active(Some(&gone), &views, true, true) {
        ActiveProfile::NeedsChoice { reason, .. } => {
            assert!(matches!(reason, ChoiceReason::SavedProfileGone { .. }));
        }
        other => {
            panic!("a vanished choice must be declared even when a single file remains: {other:?}")
        }
    }
}

#[test]
fn active_profile_serializes_with_camel_case_fields() {
    // `#[serde(rename_all = "camelCase")]` renames the variant names (`Active` → `"active"`),
    // but without `rename_all_fields = "camelCase"` it does NOT rename the fields inside
    // struct variants. So `auto_selected: bool` would serialize as snake_case in the JSON.
    // The frontend would read `undefined` with no error, on exactly the flag that
    // distinguishes auto-selection from the user's own choice.
    // This test guarantees that the tag and ALL the fields are in camelCase.

    let views = candidates(&[candidate(
        "rep+persistentgamedata1.dat",
        1,
        SavePrefix::RepPlus,
        Some(1),
    )]);
    let active = resolve_active(None, &views, true, true);

    let json = serde_json::to_string(&active).unwrap();
    assert!(
        json.contains("\"kind\":\"active\""),
        "the tag must be in camelCase"
    );
    assert!(
        json.contains("\"autoSelected\""),
        "the auto_selected field must serialize as autoSelected"
    );
    assert!(
        !json.contains("\"auto_selected\""),
        "the snake_case auto_selected must not appear"
    );
}

#[test]
fn setup_state_without_steam_reports_the_missing_chain() {
    let discovery = Discovery {
        steam: None,
        game: None,
        saves: vec![],
        game_data: None,
        diagnostics: vec![Diagnostic::SteamNotFound],
    };
    let state = setup_state(&discovery, None, |_: &Path| None, |_| None);

    assert!(state.steam.is_none());
    assert!(state.game.is_none());
    assert!(state.candidates.is_empty());
    match state.active {
        ActiveProfile::None { reason } => assert_eq!(reason, MissingReason::SteamNotFound),
        other => panic!("expected None, got {other:?}"),
    }
    assert_eq!(state.diagnostics, vec![SetupDiagnostic::SteamNotFound]);
}

#[test]
fn setup_state_never_leaks_the_steam_account_id_through_diagnostics() {
    // The full path of `UnreadablePath` carries the Steam account id under
    // `userdata/`: it's exactly the flaw already fixed on `path_hint` (see
    // `candidate_view_hides_the_steam_account_id`), resurfacing in setup diagnostics.
    // Only the last path component (here `remote`) must cross the IPC boundary.
    let discovery = Discovery {
        steam: None,
        game: None,
        saves: vec![],
        game_data: None,
        diagnostics: vec![Diagnostic::UnreadablePath {
            path: PathBuf::from("c:/steam/userdata/123456789/250900/remote"),
            // Was a `String` message when this test was written; `discovery` now carries the
            // `io` kind, which is the same information without the system's own wording.
            kind: std::io::ErrorKind::PermissionDenied,
        }],
    };
    let state = setup_state(&discovery, None, |_: &Path| None, |_| None);
    let json = serde_json::to_string(&state).unwrap();

    assert!(
        !json.contains("123456789"),
        "the Steam account id must not cross the IPC boundary in diagnostics"
    );
    assert!(
        json.contains("\"remote\""),
        "the last path component stays, for diagnostic value"
    );
}

#[test]
fn setup_state_with_two_candidates_needs_a_choice_and_hides_library_paths() {
    let steam = SteamInstall {
        root: PathBuf::from("c:/program files (x86)/steam"),
        source: SteamSource::SteamLocate,
        libraries: vec![
            PathBuf::from("c:/program files (x86)/steam/steamapps"),
            PathBuf::from("d:/games/steamlibrary/steamapps"),
        ],
    };
    let game = GameInstall {
        dir: PathBuf::from("c:/program files (x86)/steam/steamapps/common/the binding of isaac"),
        library: PathBuf::from("c:/program files (x86)/steam/steamapps"),
        manifest: PathBuf::from("c:/program files (x86)/steam/steamapps/appmanifest_250900.acf"),
        edition: Edition::RepentancePlus,
        dlcs: vec![Dlc::Repentance, Dlc::RepentancePlus],
        updated_unix: None,
    };
    let discovery = Discovery {
        steam: Some(steam),
        game: Some(game),
        saves: vec![
            candidate(
                "rep+persistentgamedata1.dat",
                1,
                SavePrefix::RepPlus,
                Some(2_000),
            ),
            candidate(
                "rep_persistentgamedata1.dat",
                1,
                SavePrefix::Rep,
                Some(1_000),
            ),
        ],
        game_data: None,
        diagnostics: vec![],
    };
    let state = setup_state(&discovery, None, |_: &Path| None, |_| None);

    assert_eq!(state.candidates.len(), 2);
    match state.active {
        ActiveProfile::NeedsChoice { .. } => {}
        other => panic!("expected NeedsChoice, got {other:?}"),
    }
    assert_eq!(
        state.steam.as_ref().map(|s| s.libraries),
        Some(2),
        "libraries is a count"
    );

    let json = serde_json::to_string(&state).unwrap();
    // "steamapps" legitimately appears in `dirHint` (the game's path, display-only):
    // the marker that must not appear is the secondary library, which has no reason
    // to show up anywhere if `libraries` is just a count.
    assert!(
        !json.contains("steamlibrary"),
        "the list of Steam library paths must not cross the IPC boundary"
    );
    assert!(
        !json.contains("d:/games"),
        "the list of Steam library paths must not cross the IPC boundary"
    );
}

/// Pins `MissingReason`'s JSON values. No other test serializes it: without this,
/// removing `rename_all` wouldn't fail anything here. It has no fields, so it's a
/// bare string, not an object with `kind`.
#[test]
fn missing_reason_json_values_are_pinned() {
    assert_eq!(
        serde_json::to_value(MissingReason::SteamNotFound).unwrap(),
        serde_json::json!("steamNotFound")
    );
    assert_eq!(
        serde_json::to_value(MissingReason::GameNotFound).unwrap(),
        serde_json::json!("gameNotFound")
    );
    assert_eq!(
        serde_json::to_value(MissingReason::NoSaves).unwrap(),
        serde_json::json!("noSaves")
    );
}

/// Same as above for `CandidateSource`, which ends up in every row of the profile list.
#[test]
fn candidate_source_json_values_are_pinned() {
    assert_eq!(
        serde_json::to_value(CandidateSource::SteamCloud).unwrap(),
        serde_json::json!("steamCloud")
    );
    assert_eq!(
        serde_json::to_value(CandidateSource::Documents).unwrap(),
        serde_json::json!("documents")
    );
    assert_eq!(
        serde_json::to_value(CandidateSource::Manual).unwrap(),
        serde_json::json!("manual")
    );
}

/// Pins `ChoiceReason`'s JSON tags and fields.
#[test]
fn choice_reason_json_tags_and_fields_are_pinned() {
    assert_eq!(
        serde_json::to_value(ChoiceReason::NeverChosen).unwrap(),
        serde_json::json!({"kind": "neverChosen"})
    );
    assert_eq!(
        serde_json::to_value(ChoiceReason::SavedProfileGone { was: "abc".into() }).unwrap(),
        serde_json::json!({"kind": "savedProfileGone", "was": "abc"})
    );
}

/// Pins `SetupDiagnostic`'s JSON tags and fields.
#[test]
fn setup_diagnostic_json_tags_and_fields_are_pinned() {
    assert_eq!(
        serde_json::to_value(SetupDiagnostic::SteamNotFound).unwrap(),
        serde_json::json!({"kind": "steamNotFound"})
    );
    assert_eq!(
        serde_json::to_value(SetupDiagnostic::GameNotFound).unwrap(),
        serde_json::json!({"kind": "gameNotFound"})
    );
    assert_eq!(
        serde_json::to_value(SetupDiagnostic::NoSavesFound).unwrap(),
        serde_json::json!({"kind": "noSavesFound"})
    );
    assert_eq!(
        serde_json::to_value(SetupDiagnostic::UnreadablePath {
            name: "remote".into(),
            // `reason` was a free `String` when this test was written and is now `IoReason`,
            // a fieldless enum: a bare camelCase string on the wire, per the `rename_all` on
            // its declaration. Pinned here as the wire value, the way it was before.
            reason: ipc::IoReason::PermissionDenied,
        })
        .unwrap(),
        serde_json::json!({
            "kind": "unreadablePath",
            "name": "remote",
            "reason": "permissionDenied"
        })
    );
    assert_eq!(
        serde_json::to_value(SetupDiagnostic::MalformedManifest {
            name: "appmanifest_250900.acf".into(),
        })
        .unwrap(),
        serde_json::json!({"kind": "malformedManifest", "name": "appmanifest_250900.acf"})
    );
}

#[test]
fn setup_state_hides_the_username_in_the_steam_and_game_hints() {
    // Steam isn't always in `Program Files`: it installs wherever you want, and a
    // library under the user's profile is extremely common. `rootHint` and `dirHint`
    // are full paths, so they fall under the same rule as `pathHint`.
    let steam = SteamInstall {
        root: PathBuf::from("c:/users/carol/steam"),
        source: SteamSource::SteamLocate,
        libraries: vec![PathBuf::from("c:/users/carol/steam/steamapps")],
    };
    let game = GameInstall {
        dir: PathBuf::from("c:/users/carol/steam/steamapps/common/the binding of isaac"),
        library: PathBuf::from("c:/users/carol/steam/steamapps"),
        manifest: PathBuf::from("c:/users/carol/steam/steamapps/appmanifest_250900.acf"),
        edition: Edition::RepentancePlus,
        dlcs: vec![Dlc::RepentancePlus],
        updated_unix: None,
    };
    let discovery = Discovery {
        steam: Some(steam),
        game: Some(game),
        saves: vec![],
        game_data: None,
        diagnostics: vec![],
    };
    let json =
        serde_json::to_string(&setup_state(&discovery, None, |_: &Path| None, |_| None)).unwrap();

    assert!(
        !json.contains("carol"),
        "the username doesn't leak out through rootHint or dirHint"
    );
    assert!(
        json.contains("the binding of isaac"),
        "what orients the user stays: the game's folder"
    );
}

// --- The preview travels with the candidates -------------------------------------------
//
// One command, not two: a save that appears or disappears between two calls would leave a
// card showing the numbers of a file no longer offered, and the join would sit in the
// frontend where nothing can check it (N8, M4's Live screen).

/// Four flags: slot 0, which is no achievement, plus three achievements, two of them done.
fn a_save() -> core_save::Save {
    core_save::Save {
        unknown_0x10: 0,
        sections: vec![core_save::Section {
            kind: core_save::Kind::Achievements,
            count: 4,
            f2: 0,
            offset: 0,
            bytes: vec![1, 1, 0, 1],
        }],
        diagnostics: Vec::new(),
    }
}

fn with_saves(saves: Vec<SaveCandidate>) -> Discovery {
    Discovery {
        steam: None,
        game: None,
        saves,
        game_data: None,
        diagnostics: vec![],
    }
}

#[test]
fn every_candidate_carries_what_its_own_file_says() {
    let d = with_saves(vec![candidate(
        "rep+persistentgamedata1.dat",
        1,
        SavePrefix::RepPlus,
        Some(1_000),
    )]);
    let state = setup_state(&d, None, |_: &Path| Some(a_save()), |_| None);
    let preview = state.candidates[0].preview.expect("the reader answered");
    assert_eq!(
        preview.achievements,
        ipc::PreviewCount::Read { done: 2, of: 3 }
    );
}

#[test]
fn a_candidate_whose_file_cannot_be_read_still_travels() {
    let d = with_saves(vec![candidate(
        "rep+persistentgamedata1.dat",
        1,
        SavePrefix::RepPlus,
        Some(1_000),
    )]);
    let state = setup_state(&d, None, |_: &Path| None, |_| None);
    assert_eq!(state.candidates.len(), 1, "it is still offered");
    assert!(
        state.candidates[0].preview.is_none(),
        "and it says it could not be read, rather than showing zeros"
    );
}

#[test]
fn a_preview_follows_its_own_candidate_and_not_the_row_it_was_sorted_into() {
    // `candidates` sorts by date, so slot 1 comes first. The reader answers only for slot 2:
    // matched by position instead of by id, the preview would land on slot 1.
    let d = with_saves(vec![
        candidate(
            "rep+persistentgamedata1.dat",
            1,
            SavePrefix::RepPlus,
            Some(2_000),
        ),
        candidate(
            "rep+persistentgamedata2.dat",
            2,
            SavePrefix::RepPlus,
            Some(1_000),
        ),
    ]);
    let state = setup_state(
        &d,
        None,
        |p: &Path| p.ends_with("rep+persistentgamedata2.dat").then(a_save),
        |_| None,
    );
    assert_eq!(state.candidates[0].slot, 1, "the newer one sorts first");
    let by_slot = |slot: u8| {
        state
            .candidates
            .iter()
            .find(|c| c.slot == slot)
            .expect("both travel")
    };
    assert!(by_slot(1).preview.is_none());
    assert_eq!(
        by_slot(2)
            .preview
            .expect("the reader answered")
            .achievements,
        ipc::PreviewCount::Read { done: 2, of: 3 }
    );
}

#[test]
fn a_folder_you_pointed_at_that_holds_no_save_is_a_different_sentence() {
    // Two "nothing found" that a reader can act on differently: one is "we looked where saves
    // usually are", the other "you chose that folder, and there is nothing in it".
    let nowhere = Discovery {
        steam: None,
        game: None,
        saves: vec![],
        game_data: None,
        diagnostics: vec![Diagnostic::NoSavesFound],
    };
    let chosen = Discovery {
        diagnostics: vec![Diagnostic::NoSavesInChosenFolder],
        ..nowhere.clone()
    };
    let reason = |d: &Discovery| match setup_state(d, None, |_: &Path| None, |_| None).active {
        ActiveProfile::None { reason } => reason,
        other => panic!("expected None, got {other:?}"),
    };
    // With no Steam the chain breaks earlier, and that answer stays: this is about the last
    // link, so both machines below have Steam and the game.
    let with_chain = |d: &Discovery| Discovery {
        steam: Some(SteamInstall {
            root: PathBuf::from("c:/steam"),
            libraries: vec![],
            source: SteamSource::Registry,
        }),
        game: Some(GameInstall {
            dir: PathBuf::from("c:/steam/game"),
            library: PathBuf::from("c:/steam"),
            manifest: PathBuf::from("c:/steam/appmanifest_250900.acf"),
            edition: Edition::Repentance,
            dlcs: vec![],
            updated_unix: None,
        }),
        ..d.clone()
    };
    assert_eq!(reason(&with_chain(&nowhere)), MissingReason::NoSaves);
    assert_eq!(
        reason(&with_chain(&chosen)),
        MissingReason::NoSavesInChosenFolder
    );
}

/// The URL the Tauri crate would build for a reference, in the shape the webview sees off
/// Windows. What matters here is that the state carries *a* URL for the stand-in, and that
/// it is the one the icon protocol answers.
fn an_icon(r: &IconRef) -> Option<String> {
    Some(format!("isaac://localhost/{}", r.to_path()))
}

fn a_game() -> GameInstall {
    GameInstall {
        dir: PathBuf::from("c:/steam/game"),
        library: PathBuf::from("c:/steam"),
        manifest: PathBuf::from("c:/steam/appmanifest_250900.acf"),
        edition: Edition::Repentance,
        dlcs: vec![],
        updated_unix: None,
    }
}

#[test]
fn the_state_carries_the_picture_that_stands_in_for_an_icon_we_could_not_resolve() {
    // B69: a row whose icon did not resolve drew a hole, and a hole reads as "this thing has
    // no picture". The stand-in is the game's own question mark, and the frontend cannot
    // build its URL itself — no `isaac://` path is written outside the Tauri crate — so it
    // travels with the state the app already reads at startup.
    let d = Discovery {
        game: Some(a_game()),
        ..with_saves(vec![])
    };
    let state = setup_state(&d, None, |_: &Path| None, an_icon);
    assert_eq!(
        state.unknown_icon_url.as_deref(),
        Some("isaac://localhost/unknown")
    );
}

#[test]
fn without_the_game_there_is_no_stand_in_to_ask_for() {
    // Nothing to serve it from: every request would be a 404, one per row, and the interface
    // would end up drawing exactly what it draws today anyway. So it is not offered, and the
    // absence is the answer rather than a picture that never arrives.
    let state = setup_state(&with_saves(vec![]), None, |_: &Path| None, an_icon);
    assert!(state.unknown_icon_url.is_none());
}
