//! Two independent derivations of the same relation: the wiki's `unlocks` and the
//! catalog's own `unlocked_by` links. They agreed on 397 of 404 on 2026-09-07. The seven
//! divergences below are not errors — six are two different id spaces, one is a genuine
//! ambiguity — and a new one is news, to be read before it is added here.

mod support;

/// Achievement ids whose wiki `unlocks` disagrees with the catalog, with the reason.
const KNOWN_DIVERGENCES: &[(u32, &str)] = &[
    (16, "entity id vs boss id: Steven is entity 79, boss 20"),
    (17, "entity id vs boss id: C.H.A.D. is entity 28, boss 21"),
    (18, "entity id vs boss id: Gish is entity 43, boss 19"),
    (34, "entity id vs boss id: It Lives! is entity 78, boss 25"),
    (66, "entity id vs boss id: Conquest is entity 65, boss 38"),
    (68, "entity id vs boss id: Triachnid is entity 101, boss 42"),
    (
        132,
        "The Soul: character 17 for the wiki, item 335 for the catalog",
    ),
];

#[test]
fn the_wiki_and_the_game_still_agree_where_they_both_speak() {
    let Some((catalog, _rs)) = support::real_catalog() else {
        return;
    };
    let dataset = match wiki::Dataset::embedded() {
        Ok(d) => d,
        Err(e) => panic!("the embedded wiki dataset must parse: {e}"),
    };
    let known: std::collections::BTreeMap<u32, &str> = KNOWN_DIVERGENCES
        .iter()
        .map(|(id, why)| (*id, *why))
        .collect();

    let (mut both, mut agreed) = (0u32, 0u32);
    let mut unexpected = Vec::new();
    for (&id, entry) in &dataset.achievements {
        let wiki::Infobox::Achievement {
            unlocks: Some(theirs),
            ..
        } = &entry.infobox
        else {
            continue;
        };
        let ours = catalog.unlocks(catalog::AchievementId(id));
        if ours.is_empty() {
            continue;
        }
        both += 1;
        if ours.iter().any(|o| same_thing(o, theirs)) {
            agreed += 1;
        } else if !known.contains_key(&id) {
            unexpected.push(format!(
                "achievement {id}: wiki says {theirs:?}, catalog says {ours:?}"
            ));
        }
    }
    eprintln!(
        "cross-check: {agreed} of {both} agree ({} known divergences)",
        known.len()
    );
    assert!(
        unexpected.is_empty(),
        "new divergences between the wiki and the game's files — read each one before \
         adding it to KNOWN_DIVERGENCES:\n{}",
        unexpected.join("\n")
    );
    assert!(
        both > 300,
        "only {both} achievements described by both sources: one of the two stopped \
         speaking, and the agreement rate below would be measured on nothing"
    );
}

/// Do the two sources point at the same thing? Item kinds collapse — the wiki has one
/// collectible id space, we key by `(kind, id)` — and an entity is never matched by id:
/// entity ids and boss ids are different spaces, which is why six of the seven known
/// divergences exist.
fn same_thing(ours: &catalog::Unlock, theirs: &wiki::Target) -> bool {
    match (ours, theirs) {
        (catalog::Unlock::Item { kind, id }, wiki::Target::Item { id: w }) => {
            *kind != catalog::ItemKind::Trinket && id.0 == *w
        }
        (catalog::Unlock::Item { kind, id }, wiki::Target::Trinket { id: w }) => {
            *kind == catalog::ItemKind::Trinket && id.0 == *w
        }
        (catalog::Unlock::Character { id }, wiki::Target::Character { id: w }) => id.0 == *w,
        (catalog::Unlock::Challenge { id }, wiki::Target::Challenge { number }) => id.0 == *number,
        // A pair of enums has 40 combinations of which five mean anything. The
        // exhaustiveness rule bans a catch-all that hides a new variant of one closed
        // enum; this arm hides nothing — the five that matter are written out above it.
        _ => false,
    }
}
