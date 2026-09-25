//! The clock, read in one place. The pure crates read none, so every timestamp and every seed
//! the app needs is drawn here.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Time since the epoch. A clock set before 1970 reads as zero: it is not a reason to refuse a
/// goal, a draw or a write.
pub(crate) fn since_epoch() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
}

/// Seconds since the epoch, the unit every stored timestamp uses.
pub(crate) fn now_unix() -> i64 {
    since_epoch().as_secs() as i64
}
