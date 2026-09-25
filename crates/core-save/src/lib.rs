//! core-save — read-only, structural reading of Isaac Repentance+ `.dat` files.

mod bestiary;
mod diff;
mod marks;
mod parse;
mod section;

pub use bestiary::{Bestiary, EntityId, Record, Tally};
pub use diff::{diff, SaveDiff};
pub use marks::{cell_index, counter_index_of, group_of, CharacterGroup, Column, CounterKey, ROWS};
pub use parse::{Diagnostic, OpenError, Save};
pub use section::{Kind, Section};
