//! The typed reasons that replace the `String` a failing command used to hand the UI.
//!
//! N2: a reason built with `format!` is not translatable, and an untyped field is what let
//! the wording drift — two of them answered in Italian and two in English, so the language
//! of an error depended on which line produced it. Every case is a variant now and the
//! wording lives in `it.ts` / `en.ts`.
//!
//! A variant's name **is** the wire value the TypeScript switches on, so the names are
//! pinned here the way `summary_shape.rs` pins the section kinds: spelled out, not derived
//! from the enum, because a table generated from the enum renames along with it and asserts
//! nothing.

use core_save::OpenError;
use ipc::{IoReason, SaveReason, SettingsReason, StoreReason};

/// `IoReason` has no variant with a field, so by the repo's rule it is a **bare string**
/// and not a tagged object: the tag exists to tell variants carrying different data apart,
/// and without data it would add a key per row and hide that the field is a value.
#[test]
fn an_io_reason_is_a_bare_string() {
    let pairs = [
        (IoReason::NotFound, "notFound"),
        (IoReason::PermissionDenied, "permissionDenied"),
        (IoReason::Other, "other"),
    ];
    for (reason, expected) in pairs {
        let json = serde_json::to_value(reason).expect("an IoReason serializes");
        assert_eq!(json, serde_json::json!(expected), "{reason:?}");
    }
}

#[test]
fn every_save_reason_has_a_pinned_wire_name() {
    let pairs = [
        (SaveReason::TooShort, "tooShort"),
        (SaveReason::BadMagic, "badMagic"),
        (
            SaveReason::Io {
                reason: IoReason::Other,
            },
            "io",
        ),
    ];
    for (reason, expected) in pairs {
        let json = serde_json::to_value(reason).expect("a SaveReason serializes");
        assert_eq!(json["kind"], expected, "{reason:?}");
    }
}

#[test]
fn every_settings_reason_has_a_pinned_wire_name() {
    let pairs = [
        (SettingsReason::ConfigDirUnknown, "configDirUnknown"),
        (
            SettingsReason::Io {
                reason: IoReason::PermissionDenied,
            },
            "io",
        ),
        (SettingsReason::Encoding, "encoding"),
    ];
    for (reason, expected) in pairs {
        let json = serde_json::to_value(reason).expect("a SettingsReason serializes");
        assert_eq!(json["kind"], expected, "{reason:?}");
    }
}

#[test]
fn every_store_reason_has_a_pinned_wire_name() {
    let pairs = [
        (StoreReason::DataDirUnknown, "dataDirUnknown"),
        (StoreReason::DataDirNotCreatable, "dataDirNotCreatable"),
        (StoreReason::Unreadable, "unreadable"),
        (
            StoreReason::NewerSchema {
                found: 7,
                supported: 1,
            },
            "newerSchema",
        ),
        (StoreReason::QueueUnparseable, "queueUnparseable"),
    ];
    for (reason, expected) in pairs {
        let json = serde_json::to_value(reason).expect("a StoreReason serializes");
        assert_eq!(json["kind"], expected, "{reason:?}");
    }
}

/// The whole point of the change: the two versions travel as **numbers**, so the sentence
/// that names them is built in the locale file and not in Rust.
#[test]
fn a_newer_schema_carries_its_versions_as_numbers() {
    let json = serde_json::to_value(StoreReason::NewerSchema {
        found: 7,
        supported: 1,
    })
    .expect("a StoreReason serializes");
    assert_eq!(json["found"], 7);
    assert_eq!(json["supported"], 1);
}

/// Each `OpenError` maps to the reason that names it. `Io` keeps only the kind: the
/// system's own message is not translatable and is the thing this change removes.
#[test]
fn an_open_error_maps_to_the_reason_that_names_it() {
    assert_eq!(SaveReason::from(&OpenError::TooShort), SaveReason::TooShort);
    assert_eq!(
        SaveReason::from(&OpenError::BadMagic { found: [0; 16] }),
        SaveReason::BadMagic
    );
    assert_eq!(
        SaveReason::from(&OpenError::Io(std::io::Error::from(
            std::io::ErrorKind::PermissionDenied
        ))),
        SaveReason::Io {
            reason: IoReason::PermissionDenied
        }
    );
}

/// Only the two kinds a user can act on are kept apart. Everything else is `Other` rather
/// than a fourth guess at what the OS meant.
#[test]
fn an_io_error_keeps_only_the_kinds_a_user_can_act_on() {
    use std::io::ErrorKind;
    assert_eq!(IoReason::from(ErrorKind::NotFound), IoReason::NotFound);
    assert_eq!(
        IoReason::from(ErrorKind::PermissionDenied),
        IoReason::PermissionDenied
    );
    for other in [
        ErrorKind::AlreadyExists,
        ErrorKind::InvalidData,
        ErrorKind::UnexpectedEof,
        ErrorKind::Unsupported,
    ] {
        assert_eq!(IoReason::from(other), IoReason::Other, "{other:?}");
    }
}

/// The property behind all of it: whatever the operating system wrote, none of it reaches
/// the wire. A reason is a variant, so there is no string for a path to hide in.
#[test]
fn nothing_the_system_wrote_survives_into_the_reason() {
    let noisy = std::io::Error::other(r"C:\Users\someone\secret\rep+persistentgamedata1.dat");
    let json = serde_json::to_value(SaveReason::from(&OpenError::Io(noisy)))
        .expect("a SaveReason serializes");
    let text = json.to_string();
    assert!(!text.contains("secret"), "{text}");
    assert!(!text.contains("Users"), "{text}");
}

/// The two diagnostics that carried the same sentence from the same producer. A screen
/// that wants to say "your database is from a newer version, 7 against 1" now has the two
/// numbers, and the sentence is in the locale file.
#[test]
fn the_queue_diagnostic_carries_the_variant_and_its_numbers() {
    let json = serde_json::to_value(ipc::QueueDiagnostic::StoreUnavailable {
        reason: StoreReason::NewerSchema {
            found: 7,
            supported: 1,
        },
    })
    .expect("a QueueDiagnostic serializes");
    assert_eq!(
        json,
        serde_json::json!({
            "kind": "storeUnavailable",
            "reason": { "kind": "newerSchema", "found": 7, "supported": 1 },
        })
    );
}

#[test]
fn the_plan_diagnostic_carries_the_variant_and_its_numbers() {
    let json = serde_json::to_value(ipc::PlanDiagnostic::StoreUnavailable {
        reason: StoreReason::NewerSchema {
            found: 7,
            supported: 1,
        },
    })
    .expect("a PlanDiagnostic serializes");
    assert_eq!(
        json,
        serde_json::json!({
            "kind": "storeUnavailable",
            "reason": { "kind": "newerSchema", "found": 7, "supported": 1 },
        })
    );
}

/// The same defect outside `IpcError`: `NoSavesCard.vue` concatenated this reason onto a
/// label, and it was `io::Error::to_string()` — untranslatable, and written by the OS. The
/// name beside it is already only the path's last component, never the path.
#[test]
fn an_unreadable_path_says_which_io_case_it_was() {
    let json = serde_json::to_value(ipc::SetupDiagnostic::UnreadablePath {
        name: "userdata".to_string(),
        reason: IoReason::PermissionDenied,
    })
    .expect("a SetupDiagnostic serializes");
    assert_eq!(
        json,
        serde_json::json!({
            "kind": "unreadablePath",
            "name": "userdata",
            "reason": "permissionDenied",
        })
    );
}
