//! core-save — read-only, structural reading of Isaac Repentance+ `.dat` files.

mod diff;
mod parse;
mod section;

pub use diff::{diff, SaveDiff};
pub use parse::{Diagnostic, OpenError, Save};
pub use section::{Kind, Section};
