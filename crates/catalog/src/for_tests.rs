//! Entry points that exist only so the repository's own dev-only targets can reach a shape
//! the public API doesn't build.
//!
//! One module per crate, and nothing dev-only anywhere else in the public surface: a name in
//! the crate's `pub use` list says "call me", which is the opposite of what these mean. The
//! module is named for tests because that is what the other five hold; this crate's one
//! entry point serves the `dump_heads` example, which is a dev-only target just the same.

use crate::diagnostics::Diagnostic;
use crate::heads;
use crate::sprite::Rect;

/// The `Main` layer of `coop menu.anm2` as frames, for the `dump_heads` spike: it prints the
/// crops so the character heads can be looked at one by one and the map written by eye.
pub fn parse_heads(anm2: &[u8], diagnostics: &mut Vec<Diagnostic>) -> Vec<Option<Rect>> {
    heads::parse(anm2, diagnostics)
}
