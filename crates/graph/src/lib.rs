//! The unlock graph. Pure: no I/O, no Tauri, no network.
//!
//! Where the data comes from is the design's core decision: **what is needed** comes from
//! the wiki's typed refs, **who unlocks what** comes from the game's own files through
//! `catalog`. The graph never invents an edge from the wiki; it reads it from the game.
//!
//! Design: `docs/superpowers/specs/2026-09-07-unlock-graph-design.md`.

pub mod rules;

pub use rules::{target_key, Corrections, Requirements, Rules, RulesError, SCHEMA_VERSION};
