//! How a `store` failure reaches the UI. Lives here because both halves do: `StoreError`
//! and `GoalsRead` are this crate's, and `ipc` cannot depend on it — `store` is the one
//! that depends on `ipc`, not the other way round.
//!
//! It is not wiring, so it does not belong in the Tauri crate: every function here has a
//! return value worth checking, and the tests next to it are what check it.

use ipc::{Goal, GoalId, IpcError, StoreReason};

use crate::{GoalsRead, StoreError};

/// A `store` failure as the UI receives it. SQLite's own message never gets this far:
/// `StoreReason` is a variant, so there is no string for a path to hide in.
pub fn store_error(e: StoreError) -> IpcError {
    store_unavailable((&e).into())
}

pub fn store_unavailable(reason: StoreReason) -> IpcError {
    IpcError::StoreUnavailable { reason }
}

/// The goals that could be read, the ones that couldn't be, and the reason there are none.
///
/// A database that won't open and a query that fails are the same case for the user — "the
/// goals can't be seen, and here's why" — and neither one is an `Err`: the Plan degrades,
/// it doesn't disappear.
pub fn plan_parts(
    read: Result<GoalsRead, StoreError>,
) -> (Vec<Goal>, Vec<GoalId>, Option<StoreReason>) {
    match read {
        Ok(r) => (r.goals, r.unreadable, None),
        Err(e) => (Vec::new(), Vec::new(), Some((&e).into())),
    }
}
