//! The 2026-09-12 measurement, turned into the property that guards it.
//!
//! Ultra Greedier is not a boss the game gates by an achievement and not a tally anyone
//! has located: it is the **second level of the Greed column**, and that was measured on
//! three days of the series, each with a different character. Nothing in the code says so
//! except `corrections.json`, which is exactly the kind of claim that rots quietly — so
//! the series keeps answering for it.
//!
//! Not a pinned value: it holds whatever the profile does next, and it fails loudly if
//! the column moves, because a wrong column reads a neighbour's cell and lights up on the
//! wrong day.

use catalog::Catalog;
use core_save::marks::{cell_index, Column};
use core_save::{Kind, Save};
use graph::target_key;
use test_support::dated_series;

const SERIES: &str = "rep+persistentgamedata1.dat";

fn real_catalog() -> Option<Catalog> {
    let packed = test_support::packed_dir()?;
    let rs = unpack::ResourceSet::open(&packed);
    Some(Catalog::build(|p| rs.read(p)))
}

/// The character a reference names, resolved the way `Graph::build` resolves it: **by the
/// wiki's id first**, and only then by name. The two forms of a character share the game's
/// name, so the name can name the wrong one — which here would be a different row of the
/// matrix, checked with full confidence.
fn character_row(c: &Catalog, label: &str, wiki_id: u32) -> Option<usize> {
    let index = graph::resolve::NameIndex::new(c);
    let id = c
        .character(catalog::CharacterId(wiki_id))
        .map(|ch| ch.id)
        .or_else(|| index.character(label))?;
    (0..core_save::marks::ROWS).find(|&row| ipc::character_for(row, c).map(|ch| ch.id) == Some(id))
}

/// The (boss label, character label, wiki character id) an achievement's references name.
fn pair(rules: &graph::Rules, id: u32, boss: &str) -> Option<(String, u32)> {
    let refs = rules.refs(id);
    let keys: Vec<String> = refs
        .iter()
        .map(|r| target_key(&r.target, rules.alias(&r.label)))
        .collect();
    if !keys.iter().any(|k| k == boss) {
        return None;
    }
    refs.iter().find_map(|r| match &r.target {
        wiki::Target::Character { id } => Some((rules.alias(&r.label).to_string(), *id)),
        _ => None,
    })
}

/// Spec 2026-09-12 §2.2. On every day of the series where an achievement whose references
/// are (`entity:Ultra Greedier`, character *X*) flipped, *X*'s Greed cell gained bit 1.
#[test]
fn winning_greedier_sets_the_second_bit_of_that_characters_greed_cell() {
    let series = dated_series(SERIES);
    if series.len() < 2 {
        return; // dated_series has already declared why
    }
    let Some(c) = real_catalog() else {
        test_support::skip("samples/packed missing: no catalog to resolve characters with");
        return;
    };
    let rules = graph::rules::embedded().expect("embedded rules");

    let mut checked = 0u32;
    let mut prev: Option<(Vec<bool>, Vec<u32>)> = None;
    for path in &series {
        let Ok(s) = Save::open(path) else { continue };
        let (Some(flags), Some(counters)) = (s.flags(Kind::Achievements), s.u32s(Kind::Counters))
        else {
            continue;
        };
        if let Some((pflags, pcounters)) = &prev {
            for id in 0..flags.len().min(pflags.len()) {
                if !(flags[id] && !pflags[id]) {
                    continue;
                }
                let Some((label, wiki_id)) = pair(rules, id as u32, "entity:Ultra Greedier") else {
                    continue;
                };
                let Some(row) = character_row(&c, &label, wiki_id) else {
                    continue;
                };
                let cell = cell_index(row, Column::Greed).expect("the Greed column is located");
                let (before, after) = (
                    pcounters.get(cell).copied().unwrap_or(0),
                    counters.get(cell).copied().unwrap_or(0),
                );
                assert_eq!(
                    (before & 0b10, after & 0b10),
                    (0, 0b10),
                    "achievement {id} is Ultra Greedier as {label} (row {row}, cell {cell}): \
                     bit 1 had to turn on that day, saw {before} -> {after}"
                );
                checked += 1;
            }
        }
        prev = Some((flags, counters));
    }
    eprintln!("greedier days checked: {checked}");
    // A property over a series holds vacuously when the series contains none of the thing
    // it is about, and then it reports coverage that isn't there.
    assert!(
        checked >= 3,
        "the series held {checked} Ultra Greedier days; the measurement of 2026-09-12 had \
         three, so fewer means the samples shrank, not that the property got weaker"
    );
}

/// Spec §2.3, the other direction: the same walk against the two columns that were pinned
/// independently on 2026-09-08. If Mother's base or The Beast's had drifted, the cell this
/// finds would not be the cell that moved.
#[test]
fn beating_mother_or_the_beast_moves_that_characters_own_cell() {
    let series = dated_series(SERIES);
    if series.len() < 2 {
        return;
    }
    let Some(c) = real_catalog() else {
        test_support::skip("samples/packed missing: no catalog to resolve characters with");
        return;
    };
    let rules = graph::rules::embedded().expect("embedded rules");

    let mut checked = 0u32;
    let mut prev: Option<(Vec<bool>, Vec<u32>)> = None;
    for path in &series {
        let Ok(s) = Save::open(path) else { continue };
        let (Some(flags), Some(counters)) = (s.flags(Kind::Achievements), s.u32s(Kind::Counters))
        else {
            continue;
        };
        if let Some((pflags, pcounters)) = &prev {
            for id in 0..flags.len().min(pflags.len()) {
                if !(flags[id] && !pflags[id]) {
                    continue;
                }
                for (key, column) in [
                    ("entity:Mother", Column::Mother),
                    ("entity:The Beast", Column::TheBeast),
                ] {
                    let Some((label, wiki_id)) = pair(rules, id as u32, key) else {
                        continue;
                    };
                    let Some(row) = character_row(&c, &label, wiki_id) else {
                        continue;
                    };
                    // The 40 unlocated cells: nothing to check, and saying so is the point.
                    let Some(cell) = cell_index(row, column) else {
                        continue;
                    };
                    let (before, after) = (
                        pcounters.get(cell).copied().unwrap_or(0),
                        counters.get(cell).copied().unwrap_or(0),
                    );
                    assert!(
                        after > before,
                        "achievement {id} is {key} as {label} (row {row}, cell {cell}): the \
                         cell had to move that day, saw {before} -> {after}"
                    );
                    checked += 1;
                }
            }
        }
        prev = Some((flags, counters));
    }
    eprintln!("mother/beast days checked: {checked}");
    assert!(
        checked >= 2,
        "the series held {checked} Mother or The Beast days for a located character; the \
         walk of 2026-09-12 found four"
    );
}
