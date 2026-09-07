//! The plan queue: what you want to unlock, in the order you mean to do it.
//!
//! Pure — no SQL, no Tauri, no view-models. The order is yours; the only thing this crate
//! insists on is that it never contradicts the graph, and it repairs rather than refuses.
//!
//! Design: `docs/superpowers/specs/2026-09-07-plan-queue-design.md`.

pub mod edit;
pub mod model;
pub mod order;

pub use model::{Queue, QueueError, Row};
pub use order::Dependencies;
