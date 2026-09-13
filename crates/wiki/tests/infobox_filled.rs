//! The defect this crate carried until 2026-09-13: 907 of its 1727 entries reached the
//! frontend as `{"kind":"item"}` and nothing else, because `Infobox::Item` and
//! `Infobox::Trinket` were unit variants and `infobox_from` dropped every parameter it had
//! just parsed.
//!
//! This is also the non-vacuity guard. If the parser ever stops finding the templates, the
//! dataset shrinks quietly and every "is it empty" question answers trivially — so the
//! counts are asserted before the emptiness is.

use wiki::{CollectibleTemplate, Dataset, Entry, Infobox};

/// Whether an entry's infobox says nothing at all. Each kind is asked about the fields the
/// wiki fills on nearly every page of that kind, not about all of them: a handful of pages
/// genuinely leave a field blank, and that is the wiki's editorial choice, not our bug.
fn says_nothing(e: &Entry) -> bool {
    match &e.infobox {
        Infobox::Item { quote, tags, .. } => quote.is_empty() && tags.is_empty(),
        Infobox::Trinket { quote, tags, .. } => quote.is_empty() && tags.is_empty(),
        Infobox::Achievement { requirements, .. } => requirements.is_empty(),
        Infobox::Boss {
            base_hp,
            environment,
            ..
        } => base_hp.is_none() && environment.is_empty(),
        Infobox::Challenge { goal, .. } => goal.is_empty(),
        Infobox::Character { health, .. } => health.is_empty(),
        // A transformation says nothing when it names neither a count nor a single item.
        // Adult is exactly that and it is not a defect: its page states its condition in
        // prose about pills, which this model does not carry.
        Infobox::Transformation {
            requires,
            contributors,
            ..
        } => requires.is_none() && contributors.is_empty(),
    }
}

#[test]
fn items_and_trinkets_carry_their_infobox() {
    let d = Dataset::embedded().expect("embedded dataset");

    // Non-vacuity first: these two maps are the ones the defect emptied.
    assert!(d.items.len() > 700, "only {} items", d.items.len());
    assert!(d.trinkets.len() > 180, "only {} trinkets", d.trinkets.len());

    let empty: Vec<&str> = d
        .items
        .values()
        .chain(d.trinkets.values())
        .filter(|e| says_nothing(e))
        .map(|e| e.title.as_str())
        .collect();
    assert!(
        empty.is_empty(),
        "{} of {} items and trinkets say nothing: {:?}",
        empty.len(),
        d.items.len() + d.trinkets.len(),
        &empty[..empty.len().min(10)]
    );
}

#[test]
fn every_kind_of_entry_says_something() {
    let d = Dataset::embedded().expect("embedded dataset");
    let all: Vec<&Entry> = d
        .items
        .values()
        .chain(d.trinkets.values())
        .chain(d.achievements.values())
        .chain(d.bosses.values())
        .chain(d.challenges.values())
        .chain(d.characters.values())
        .collect();

    assert!(all.len() > 1700, "only {} entries in total", all.len());
    let empty: Vec<&str> = all
        .iter()
        .filter(|e| says_nothing(e))
        .map(|e| e.title.as_str())
        .collect();
    // Not zero: the four kinds that were already parsed have a few pages where the wiki
    // fills nothing. One in twenty would mean the parser broke, not that the wiki is thin.
    assert!(
        empty.len() * 20 < all.len(),
        "{} of {} entries say nothing: {:?}",
        empty.len(),
        all.len(),
        &empty[..empty.len().min(10)]
    );
}

#[test]
fn the_template_field_separates_the_two_collectible_templates() {
    let d = Dataset::embedded().expect("embedded dataset");
    let activated = d
        .items
        .values()
        .filter(|e| {
            matches!(
                e.infobox,
                Infobox::Item {
                    template: CollectibleTemplate::Activated,
                    ..
                }
            )
        })
        .count();
    // Both halves have to be non-empty, or the field is a constant in disguise:
    // `InfoboxKind::of` merged the two templates into one kind until 2026-09-13.
    assert!(activated > 100, "only {activated} activated collectibles");
    assert!(
        activated < d.items.len() - 100,
        "{activated} of {} are activated: the flag is not discriminating",
        d.items.len()
    );
}
