use crate::deck::Deck;
use crate::target::Target;

/// splitmix64: six lines, and no dependency for a modulo over a few hundred entries. The
/// constants are the reference ones (Steele, Lea & Flood, 2014); it is a mixer, not a
/// generator, which is exactly what turning a seed into an index needs.
fn mix(seed: u64) -> u64 {
    let z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// One target of the deck, or `None` when there is nothing to draw — which is a state of its
/// own on the screen, not a disabled button.
///
/// The modulo's bias over a few hundred entries out of 2^64 is far below anything a
/// measurement could see, and a rejection loop would buy nothing here.
pub fn draw(deck: &Deck, seed: u64) -> Option<Target> {
    let len = deck.targets.len();
    if len == 0 {
        return None;
    }
    let index = (mix(seed) % len as u64) as usize;
    deck.targets.get(index).copied()
}
