use serde::{Deserialize, Serialize};

/// One thing to go and do tonight.
///
/// An enum and not `{ character, column, level }` because `Greedier` is legal on exactly one
/// column: a struct with a level field can hold "the second mark of Isaac's Boss Rush", which
/// is a thing this project refuses to name — bit 1 is measured in the Greed column and
/// nowhere else (`docs/save-format.md`, and the spec's §2).
///
/// Tagged with struct variants, and `rename_all_fields` beside `rename_all`: the first renames
/// the variants, only the second renames the fields inside them. It is stored in the document,
/// so this shape is a format and changing it is a document version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Target {
    /// "Mother, with Blue Baby" — one cell of the matrix.
    Mark { character: u8, column: u8 },
    /// "Ultra Greedier, with Judas" — the second level of the Greed column, the only one this
    /// project has measured.
    Greedier { character: u8 },
}

/// Derived on every read, never stored. That is what makes a drawn card close itself: it is
/// not a record of a task, it is a question re-asked against the current save.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The cell is readable and the target is not done.
    Missing,
    /// The cell is readable and the target is done.
    Taken,
    /// The file does not say.
    Unreadable,
}
