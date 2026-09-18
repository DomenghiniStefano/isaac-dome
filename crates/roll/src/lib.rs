//! What to play tonight: one target drawn from a space of targets.
//!
//! **It knows nothing about the game.** No 34, no 12, no Greed: the caller brings the shape
//! and the cells and gets back a deck that names everything it left out. The counts stay
//! where they were measured (`ipc::marks`), which is the hardcoded-counts rule applied one
//! level up.
//!
//! Pure in the sense `floor` and `plan` are: no clock, no RNG of its own, no I/O. `draw` is a
//! function of `(deck, seed)` and the seed comes from `app`, which is what makes every test
//! here say something.

mod deck;
mod draw;
mod space;
mod target;

pub use deck::{contributions, deck, Contributions, Deck, Excluded, Preset, Selection};
pub use draw::draw;
pub use space::{CellValue, Space, SpaceError};
pub use target::{Status, Target};
