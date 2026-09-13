//! Three fields exist on both sides of the app: the wiki says them, and so does the game.
//!
//! | wiki | game | what it settles |
//! |---|---|---|
//! | `Infobox::Item.quality` | `items_metadata.xml` | the field is read correctly |
//! | `Infobox::Item.tags` | `items_metadata.xml` | the same, on a multi-valued field |
//! | `Infobox::Item.quote` | `items.xml` `description` | that `quote` is the pickup quote at all |
//!
//! The third is the point. `Infobox::Item.quote`'s doc comment asserts it is the same string
//! the game shows when you pick the item up; that assertion is what a future "wiki says X,
//! your game says Y" comparison rests on, and this is the test that earns it.
//!
//! A disagreement is **not** a failure. The dataset's snapshot is 2026-09-04 and its
//! `lastKnownPatch` is v1.9.7.17 (2026-04-20), while the player's install is whatever they
//! have; the two drifting apart is the thing a compare-button exists to show. A *systematic*
//! disagreement is a failure, because it means we are reading the field wrong.

use catalog::{ItemKind, Language};
use unpack::ResourceSet;
use wiki::{Infobox, Inline};

/// The words of an inline run, edition wrappers unwrapped. The wiki writes the quote with
/// per-edition markup (`Boomerang tears {{dlc|r|+ DMG up + luck down}}`), and it is the
/// flattened reading that should equal what the game shows.
fn flatten(inline: &[Inline]) -> String {
    let mut out = String::new();
    for i in inline {
        match i {
            Inline::Text { text, .. } => out.push_str(text),
            Inline::Ref { label, .. } | Inline::Concept { label, .. } => out.push_str(label),
            Inline::Edition { inline, .. } => out.push_str(&flatten(inline)),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Per field, over the items where both sides say something. Five months of patches move a
/// handful of qualities and a few quotes; they do not move one item in twenty.
const MAX_DISAGREEMENT: f64 = 0.05;

struct Tally {
    both: u32,
    bad: u32,
    examples: Vec<String>,
}

impl Tally {
    fn new() -> Tally {
        Tally {
            both: 0,
            bad: 0,
            examples: Vec::new(),
        }
    }

    fn disagree(&mut self, example: String) {
        self.bad += 1;
        if self.examples.len() < 10 {
            self.examples.push(example);
        }
    }

    fn rate(&self) -> f64 {
        if self.both == 0 {
            return 0.0;
        }
        f64::from(self.bad) / f64::from(self.both)
    }

    fn report(&self, name: &str) {
        eprintln!(
            "{name}: {} of {} disagree ({:.2}%)",
            self.bad,
            self.both,
            self.rate() * 100.0
        );
        for e in &self.examples {
            eprintln!("    {e}");
        }
    }

    fn assert_within(&self, name: &str) {
        assert!(
            self.rate() < MAX_DISAGREEMENT,
            "{name} disagrees on {} of {} ({:.2}%), over the {:.0}% a patch gap explains: \
             we are reading the field wrong, not reading an old wiki",
            self.bad,
            self.both,
            self.rate() * 100.0,
            MAX_DISAGREEMENT * 100.0
        );
    }
}

#[test]
fn the_wiki_and_the_game_agree_on_quality_tags_and_the_pickup_quote() {
    let Some(packed) = test_support::packed_dir() else {
        return;
    };
    let rs = ResourceSet::open(&packed);
    let catalog = catalog::Catalog::build(|p| rs.read(p));
    let Ok(dataset) = wiki::Dataset::embedded() else {
        test_support::skip("the embedded wiki dataset does not load");
        return;
    };

    // Every tag the game uses anywhere. A wiki tag outside this set is not a tag at all —
    // it is wikitext that reached the dataset dressed as one.
    //
    // Measured 2026-09-13: the two vocabularies are the same 32 words, and the wiki adds
    // exactly one the game never uses, `devilsacrifice`. So the per-item disagreements are
    // not a vocabulary mismatch — they are the wiki qualifying a tag by edition where the
    // installed `items_metadata.xml` simply lists what holds today.
    let vocabulary: std::collections::BTreeSet<String> = catalog
        .items()
        .flat_map(|i| i.tags.iter().cloned())
        .collect();
    assert!(
        vocabulary.len() > 20,
        "the game's tag vocabulary has only {} entries: the catalog did not load",
        vocabulary.len()
    );

    let (mut quality, mut tags, mut quote) = (Tally::new(), Tally::new(), Tally::new());
    let mut matched = 0u32;

    for item in catalog.items() {
        // Trinkets are keyed separately in the dataset and have no quality of their own in
        // the wiki's box; this test is about collectibles.
        if item.kind == ItemKind::Trinket {
            continue;
        }
        let Some(entry) = dataset.items.get(&item.id.0) else {
            continue;
        };
        let Infobox::Item {
            quality: w_quality,
            tags: w_tags,
            quote: w_quote,
            ..
        } = &entry.infobox
        else {
            continue;
        };
        matched += 1;

        if let (Some(w), Some(g)) = (w_quality, item.quality) {
            quality.both += 1;
            if *w != g {
                quality.disagree(format!("{} {}: wiki {w}, game {g}", item.id.0, entry.title));
            }
        }

        // Not equality. The wiki qualifies some tags by edition (`{{dlc|r+|fly}}`) and
        // `Infobox::Item.tags` flattens that qualification away on purpose, so the wiki can
        // legitimately list a tag the installed game does not. What must hold is that every
        // tag the wiki states is a **word the game's own vocabulary uses** — which is what
        // fails the moment an unparsed `{{dlc|r+|fly}}` is passed off as a tag, the bug this
        // test found on 2026-09-13.
        if !w_tags.is_empty() && !item.tags.is_empty() {
            tags.both += 1;
            let junk: Vec<&String> = w_tags
                .iter()
                .filter(|t| !vocabulary.contains(t.as_str()))
                .collect();
            if !junk.is_empty() {
                tags.disagree(format!(
                    "{} {}: not tags the game ever uses: {junk:?}",
                    item.id.0, entry.title
                ));
            }
        }

        // Not equality either, and for a reason the data forced: `{{dlcalt|Kills heal|r=DMG
        // up + kills heal}}` states two readings, one per era, and the game shows the one
        // for the edition installed. Flattened, the wiki's quote *contains* the game's.
        // Containment still catches the original defect — raw `{{dlc|r|…}}` markup does not
        // contain the game's sentence — without pretending the wiki says only one thing.
        let g_quote = catalog.text(&item.description, Language::English);
        let w_quote = flatten(w_quote);
        if !w_quote.is_empty() && !g_quote.is_empty() {
            quote.both += 1;
            if !w_quote.contains(g_quote) {
                quote.disagree(format!(
                    "{} {}: wiki {w_quote:?} does not contain game {g_quote:?}",
                    item.id.0, entry.title
                ));
            }
        }
    }

    // Non-vacuity: an empty join answers every question below trivially. The game has 733
    // collectibles and the dataset 719, so anything under 500 means the join is broken.
    assert!(
        matched > 500,
        "only {matched} collectibles matched between the catalog and the dataset: \
         the join is broken, and the agreement below would mean nothing"
    );
    eprintln!("matched {matched} collectibles");
    quality.report("quality");
    tags.report("tags");
    quote.report("quote");

    // Each field also has to have been compared at all: a field that is always empty on one
    // side would report a perfect 0% while testing nothing.
    for (name, t) in [("quality", &quality), ("tags", &tags), ("quote", &quote)] {
        assert!(
            t.both > 300,
            "{name} was compared on only {} items: one of the two sides is not speaking",
            t.both
        );
    }

    quality.assert_within("quality");
    tags.assert_within("tags");
    quote.assert_within("quote");
}
