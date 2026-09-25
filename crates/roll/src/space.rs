//! The space a draw picks from: the completion matrix as a grid of cells, with its shape,
//! the one column that carries a second level, and which rows are a run you could start.
//!
//! The shape is the caller's, checked once in [`Space::new`] and never assumed: this crate
//! knows no count of the game's, and a shape that does not describe its cells is an error,
//! not a short deck. What a target's status is — missing, taken, unreadable — is read here,
//! from the cell's bits.

use crate::target::{Status, Target};

/// A cell of the space. Two variants and not three: `ipc::Cell` tells "not located" from
/// "suspicious value" because a diagnostics screen needs to, and a draw does not — both mean
/// the same thing here, which is that the file does not say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellValue {
    /// A valid mask. Bit 0 and bit 1 are levels, bit 2 is "won online".
    Known { bits: u8 },
    /// Not located, or outside the mask's range.
    Unreadable,
}

/// A shape that does not describe the cells it was given. Returned rather than clamped: a
/// caller that gets the shape wrong gets an error, not a silently short deck.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceError {
    CellCount {
        expected: usize,
        found: usize,
    },
    PlayableCount {
        expected: usize,
        found: usize,
    },
    GreedColumn {
        columns: usize,
        found: usize,
    },
    /// More rows or columns than a `Target` can name: its coordinates are `u8`.
    TooLarge {
        rows: usize,
        columns: usize,
    },
}

impl std::fmt::Display for SpaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpaceError::CellCount { expected, found } => {
                write!(f, "expected {expected} cells, got {found}")
            }
            SpaceError::PlayableCount { expected, found } => {
                write!(f, "expected {expected} playable flags, got {found}")
            }
            SpaceError::GreedColumn { columns, found } => {
                write!(f, "greed column {found} outside {columns} columns")
            }
            SpaceError::TooLarge { rows, columns } => {
                write!(f, "{rows}x{columns} is past what a target can address")
            }
        }
    }
}

impl std::error::Error for SpaceError {}

/// The space a draw picks from: a grid of cells, the one column that carries a second level,
/// and which rows are a run you could actually start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Space {
    rows: usize,
    columns: usize,
    greed_column: usize,
    cells: Vec<CellValue>,
    playable: Vec<bool>,
}

/// Bit 1: the second level, measured in the Greed column on 2026-09-12 and named from that
/// measurement. It is deliberately not called `HARD`.
const SECOND_LEVEL: u8 = 2;

impl Space {
    /// `cells` is row-major and `rows * columns` long; `playable` is one flag per row.
    pub fn new(
        rows: usize,
        columns: usize,
        greed_column: usize,
        cells: Vec<CellValue>,
        playable: Vec<bool>,
    ) -> Result<Space, SpaceError> {
        // First, so the product below cannot overflow and no cast in `deck` can wrap: a
        // target addresses its cell with two `u8` (card #80, P11c).
        const ADDRESSABLE: usize = u8::MAX as usize + 1;
        if rows > ADDRESSABLE || columns > ADDRESSABLE {
            return Err(SpaceError::TooLarge { rows, columns });
        }
        let expected = rows * columns;
        if cells.len() != expected {
            return Err(SpaceError::CellCount {
                expected,
                found: cells.len(),
            });
        }
        if playable.len() != rows {
            return Err(SpaceError::PlayableCount {
                expected: rows,
                found: playable.len(),
            });
        }
        if greed_column >= columns {
            return Err(SpaceError::GreedColumn {
                columns,
                found: greed_column,
            });
        }
        Ok(Space {
            rows,
            columns,
            greed_column,
            cells,
            playable,
        })
    }

    /// The one shape with nothing to draw — no rows, no columns, no cells — built directly
    /// rather than through `new`, so it cannot fail. Its one caller is the last fallback in
    /// `ipc::unreadable_space`, reached only if `new` refused a shape `ipc` built from its own
    /// tables: at that point there is no error left to report and no shape left to try.
    pub fn empty() -> Space {
        Space {
            rows: 0,
            columns: 0,
            greed_column: 0,
            cells: Vec::new(),
            playable: Vec::new(),
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn columns(&self) -> usize {
        self.columns
    }

    pub fn greed_column(&self) -> usize {
        self.greed_column
    }

    /// Every target the space holds: one per cell, plus one `Greedier` per row. This is the
    /// total the deck's accounting is checked against, in the tests.
    #[cfg(feature = "test-api")]
    pub fn target_count(&self) -> usize {
        self.rows * self.columns + self.rows
    }

    /// `None` for a row the space does not have.
    pub fn playable(&self, row: usize) -> Option<bool> {
        self.playable.get(row).copied()
    }

    fn cell(&self, row: usize, column: usize) -> Option<CellValue> {
        if row >= self.rows || column >= self.columns {
            return None;
        }
        self.cells.get(row * self.columns + column).copied()
    }

    /// `None` when the target is not in this space — a stored draw from a version whose matrix
    /// was a different size.
    pub fn status(&self, target: &Target) -> Option<Status> {
        match *target {
            Target::Mark { character, column } => {
                match self.cell(character as usize, column as usize)? {
                    // Any non-zero value: a cell's bits replace one another rather than
                    // accumulating (`docs/save-format.md`), so there is no bit that means
                    // "cleared" and every value but zero is a mark that was taken.
                    CellValue::Known { bits: 0 } => Some(Status::Missing),
                    CellValue::Known { .. } => Some(Status::Taken),
                    CellValue::Unreadable => Some(Status::Unreadable),
                }
            }
            Target::Greedier { character } => {
                match self.cell(character as usize, self.greed_column)? {
                    // Bit 1 alone, never bit 2: bit 2 is "won online", and reading it as a
                    // level would invent one the file never stated.
                    CellValue::Known { bits } if bits & SECOND_LEVEL == 0 => Some(Status::Missing),
                    CellValue::Known { .. } => Some(Status::Taken),
                    CellValue::Unreadable => Some(Status::Unreadable),
                }
            }
        }
    }
}
