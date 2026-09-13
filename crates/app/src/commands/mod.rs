//! The Tauri commands, one file per screen family.
//!
//! Nothing in here is tested, and nothing in here should be worth testing: a command reads
//! state, calls a pure crate, and hands back a view-model. The moment one of them grows a
//! return value worth checking, that half belongs in `ipc` — which is the trip `GraphDeps`,
//! `plan_parts` and the degradation logic have already made.

pub(crate) mod completion;
pub(crate) mod graph;
pub(crate) mod plan;
pub(crate) mod profile;
pub(crate) mod queue;
pub(crate) mod session;
pub(crate) mod wiki;
