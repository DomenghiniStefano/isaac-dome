//! Entry points that exist only so tests can reach a shape the public API doesn't build.
//!
//! One module per crate, and nothing test-only anywhere else in the public surface: a name
//! in the crate's `pub use` list says "call me", which is the opposite of what these mean.
//! Nothing outside a `tests/` target may call them.

use crate::{CollectibleTemplate, Dataset, Entry, Infobox};

/// An empty dataset for tests in the crates downstream: they need a `Dataset` to fill one
/// field of, and building `meta` by hand in every test would copy that block around. Not for
/// production code — `embedded()` and `from_json` are the ways in.
pub fn empty_dataset() -> Dataset {
    Dataset::empty()
}

/// An entry carrying only a title and an infobox: the three common facts empty, no
/// sections. Downstream tests care about one field at a time, and spelling out `Entry`'s
/// whole shape in each of them means every field added to the type edits a dozen tests
/// that never cared about it.
pub fn entry(title: &str, infobox: Infobox) -> Entry {
    Entry {
        title: title.into(),
        revid: 1,
        description: Vec::new(),
        dlc: Vec::new(),
        unlocked_by: None,
        infobox,
        sections: Vec::new(),
    }
}

/// A collectible infobox that says nothing. Same reason as `entry`: a test that wants "an
/// item, any item" should not have to name eight fields to get one.
pub fn empty_item() -> Infobox {
    Infobox::Item {
        quote: String::new(),
        template: CollectibleTemplate::Passive,
        quality: None,
        tags: Vec::new(),
        recharge: Vec::new(),
        devil_price: Vec::new(),
        shop_price: Vec::new(),
        pools: Vec::new(),
    }
}

/// A boss infobox that says nothing.
pub fn empty_boss() -> Infobox {
    Infobox::Boss {
        base_hp: None,
        stage_hp: Vec::new(),
        variant: None,
        environment: Vec::new(),
        pool: Vec::new(),
    }
}

/// A trinket infobox that says nothing.
pub fn empty_trinket() -> Infobox {
    Infobox::Trinket {
        quote: String::new(),
        tags: Vec::new(),
        pools: Vec::new(),
    }
}
