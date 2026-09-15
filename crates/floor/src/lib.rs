//! floor — the grid a player paints, and the game's own rules about where a secret room can be.
//!
//! Pure: no I/O, no clock, no network. The rules are a JSON file embedded at build time, and
//! every one of them carries the sentence it was read from — see
//! `docs/superpowers/reports/2026-09-15-secret-room-rules.md`.

mod grid;
mod room;

pub use grid::{neighbours, Grid, CELLS, HEIGHT, START, WIDTH};
pub use room::{Cell, RoomKind, Shape};
