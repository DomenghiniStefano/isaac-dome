//! Why a command could not answer, as variants rather than as a sentence.
//!
//! A `reason: String` built in Rust is not translatable, and an untyped field is what lets
//! the wording drift: before this, two of them answered in Italian and two in English, so
//! the language of an error depended on which line produced it. It is also the defect
//! `CLAUDE.md` charges `format!("{:?}")` with — the system's own message can name a path,
//! and a path carries the Steam account id and the Windows username.
//!
//! Every case is a variant here; the numbers travel as numbers and the wording lives in
//! `it.ts` / `en.ts`.

use serde::Serialize;

/// What the operating system said, reduced to the two cases a user can act on. Anything
/// else is `Other`: a third guess at what the OS meant would be wording, not information.
///
/// No variant carries a field, so this is a bare camelCase string on the wire and the
/// TypeScript mirrors it as a union of values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum IoReason {
    NotFound,
    PermissionDenied,
    Other,
}

impl From<std::io::ErrorKind> for IoReason {
    fn from(kind: std::io::ErrorKind) -> Self {
        match kind {
            std::io::ErrorKind::NotFound => IoReason::NotFound,
            std::io::ErrorKind::PermissionDenied => IoReason::PermissionDenied,
            _ => IoReason::Other,
        }
    }
}

/// Why the active profile's save could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SaveReason {
    /// Shorter than a header: there is no save in there at all.
    TooShort,
    /// The `ISAACNGSAVE` signature isn't where it has to be. The bytes found are not
    /// carried: they say nothing a user can act on, and they come from the user's file.
    BadMagic,
    Io {
        reason: IoReason,
    },
}

impl From<&core_save::OpenError> for SaveReason {
    fn from(e: &core_save::OpenError) -> Self {
        match e {
            core_save::OpenError::TooShort => SaveReason::TooShort,
            core_save::OpenError::BadMagic { .. } => SaveReason::BadMagic,
            core_save::OpenError::Io(io) => SaveReason::Io {
                reason: io.kind().into(),
            },
        }
    }
}

/// Why `settings.json` could not be written. The only state the app persists outside its
/// own database, and the one file whose loss the user notices immediately.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SettingsReason {
    /// The platform would not say where the config folder is: nothing to write into.
    ConfigDirUnknown,
    Io {
        reason: IoReason,
    },
    /// The settings would not serialize. Close to impossible, and still not an
    /// `unwrap()`: a case with no variant is a case that reads as another one.
    Encoding,
}

/// Why the app's database is unavailable. `NewerSchema` is the only one the user can act
/// on, which is why its two versions travel as numbers instead of inside a sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum StoreReason {
    /// The platform would not say where the data folder is.
    DataDirUnknown,
    /// It said, and the folder could not be created.
    DataDirNotCreatable,
    /// The file won't open, isn't SQLite, or a query failed. SQLite's own message stays
    /// in `store`: it can contain the file path.
    Unreadable,
    NewerSchema {
        found: u32,
        supported: u32,
    },
    /// The plan queue is one JSON document, and this one doesn't parse. Not the same as
    /// the database being unreadable: everything else in it reads.
    QueueUnparseable,
}
