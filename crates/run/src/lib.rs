//! run — what was played, from the game's own log. Pure: no disk, no clock, no screen.
//!
//! The split is the repo's rule — *if a return value is worth checking, it lives in a pure
//! crate* — applied to a file watcher, where the parts worth checking are not the ones that
//! touch the disk. [`Tail`] turns bytes into lines and recognizes a relaunch; the rules turn a
//! line into an event; the fold makes the judgments the log does not make.

mod tail;

pub use tail::Tail;
