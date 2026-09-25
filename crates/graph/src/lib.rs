//! The unlock graph. Pure: no I/O, no Tauri, no network.
//!
//! Where the data comes from is the design's core decision: **what is needed** comes from
//! the wiki's typed refs, **who unlocks what** comes from the game's own files through
//! `catalog`. The graph never invents an edge from the wiki; it reads it from the game.
//!
//! Design: `docs/superpowers/specs/2026-09-07-unlock-graph-design.md`.

pub mod build;
pub mod evaluate;
#[cfg(feature = "test-api")]
pub mod for_tests;
pub mod generate;
pub mod model;
pub mod resolve;
pub mod rules;

// Every item of this crate has one path, through the module that defines it:
// `graph::build::Graph`, `graph::rules::Rules`. The one re-export is another crate's type.
//
// The id every public signature here speaks in (card #81, V12), re-exported so `plan` reads it
// from the crate it already depends on.
pub use catalog::AchievementId;
