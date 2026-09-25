//! floor — the grid a player paints, and the game's own rules about where a secret room can be.
//!
//! Pure: no I/O, no clock, no network. The rules are a JSON file embedded at build time, and
//! every one of them carries the sentence it was read from — see
//! `docs/superpowers/reports/2026-09-15-secret-room-rules.md`.

mod grid;
mod room;
mod rules;
mod solve;

#[cfg(feature = "test-api")]
pub use grid::START;
pub use grid::{neighbours, Grid, CELLS, HEIGHT, WIDTH};
pub use room::{Cell, RoomKind, Shape};
pub use rules::{is_special, Constraint, Rule, Rules, RulesError, Target, SPECIAL_KINDS};
pub use solve::{distance_from_start, solve, Candidate, Solution, Unresolved};
