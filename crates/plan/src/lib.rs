//! The plan queue: what you want to unlock, in the order you mean to do it.
//!
//! Pure — no SQL, no Tauri, no view-models. The order is yours; the only thing this crate
//! insists on is that it never contradicts the graph, and it repairs rather than refuses.
//!
//! Design: `docs/superpowers/specs/2026-09-07-plan-queue-design.md`.

// Private modules, so every item has one path: the crate root.
mod edit;
mod model;
mod order;

pub use model::{Queue, QueueError, Row};
pub use order::Dependencies;
// The type of `Row::achievement` and `Row::origins`, so a reader of the queue needs no second
// crate to name it (card #81, V12).
pub use graph::AchievementId;
