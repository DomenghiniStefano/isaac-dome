//! B42: the `player` Cargo table carries `parent` — the relation from an alternate form
//! (Black Judas, Lazarus Risen, a Tainted character's own Soul form) to the character it is
//! a variant of, on the same page. `Infobox::Character.parent` states the same relation,
//! read straight from the page's own `{{infobox character}}`. Downloaded since the table
//! was added, neither was ever checked against the other until this module.
//!
//! Measured 2026-09-14 on the committed snapshot: 32 named forms carry a character-infobox
//! *and* a `player` row, and all 32 agree. The other 8 rows of `player` (Jacob, Esau, The
//! Forgotten, The Soul, Tainted Forgotten, Tainted Soul, Tainted Lazarus, Dead Tainted
//! Lazarus) have no wiki character-infobox to compare against at all: four whole pages —
//! Jacob & Esau, The Forgotten, Tainted Forgotten, Tainted Lazarus — are missing from this
//! snapshot, which `player`, a Cargo table rather than a page fetch, still knows about.
//! Matching by `player`'s own `id` column instead of by name would have hidden this: that
//! column is exactly as unreliable as the infoboxes' (`Isaac` 14, `Magdalene` 2 — the same
//! bug `Resolver.characters` exists to route around), so an id match would have "agreed" by
//! accident on names it was never actually comparing (`Tainted Bethany`'s real id 36
//! colliding with `player`'s row for `Tainted Soul`).

use crate::infobox::{extract_infoboxes, infobox_from, InfoboxKind, RawInfobox};
use crate::page::PageKind;
use crate::raw::RawPage;
use crate::resolver::Resolver;
use crate::{Diagnostics, Infobox, Target};

/// One named form where the two sources disagree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParentMismatch {
    pub name: String,
    pub wiki_parent: Option<Target>,
    pub player_parent: Option<Target>,
}

/// The result of [`cross_check_character_parents`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParentCrossCheck {
    /// Named forms present on both sides: a wiki character-infobox, and a `player` row
    /// with that name as its `alias`.
    pub compared: u32,
    /// Same name, **both** sources state a parent, and the two differ. This is the
    /// disagreement the guard is about.
    pub mismatches: Vec<ParentMismatch>,
    /// Names where exactly one source states a parent, in page order. Not a disagreement
    /// and not nothing: silence is a third answer, and since B45 it has a known cause —
    /// `{{infobox characters}}` has **no `parent` parameter at all**, so the second form of
    /// a two-character page cannot say what `player` knows. Reported rather than tolerated,
    /// so the test can pin *which* names are silent and go red when a new one joins them.
    pub stated_by_one: Vec<ParentMismatch>,
}

/// Compares every character-infobox's own `parent` against the `player` table's `parent`
/// for the same name — the infobox's `name` parameter, or the page title when it carries
/// none (the base form; `player`'s `alias` column names it the same way). A name `player`
/// has no row for is left out of `compared`, not counted as a mismatch: there is nothing on
/// that side to disagree with (see the module doc).
pub fn cross_check_character_parents(pages: &[RawPage], r: &Resolver) -> ParentCrossCheck {
    let compared: Vec<ParentMismatch> = pages
        .iter()
        .filter(|p| p.index.kind == PageKind::Character)
        .flat_map(|p| {
            extract_infoboxes(&p.text)
                .into_iter()
                .map(move |ib| (p, ib))
        })
        .filter(|(_, ib)| InfoboxKind::of(&ib.name) == Some(InfoboxKind::Character))
        .filter_map(|(p, ib)| compare(p, &ib, r))
        .collect();
    let count = compared.len() as u32;
    let (mismatches, stated_by_one) = compared
        .into_iter()
        .filter(|c| c.wiki_parent != c.player_parent)
        .partition(|c| c.wiki_parent.is_some() && c.player_parent.is_some());
    ParentCrossCheck {
        compared: count,
        mismatches,
        stated_by_one,
    }
}

/// The two parents one character-infobox is compared on, agreeing or not; `None` when
/// `player` has no row for its name.
fn compare(p: &RawPage, ib: &RawInfobox, r: &Resolver) -> Option<ParentMismatch> {
    let name = form_name(p, ib);
    let player_parent = r.player_table_parent(&name)?;
    let mut d = Diagnostics::default();
    let Infobox::Character {
        parent: wiki_parent,
        ..
    } = infobox_from(InfoboxKind::Character, ib, &p.text, r, &mut d)
    else {
        return None;
    };
    Some(ParentMismatch {
        name,
        wiki_parent,
        player_parent,
    })
}

/// The infobox's `name` parameter, or the page title when it carries none.
fn form_name(p: &RawPage, ib: &RawInfobox) -> String {
    ib.params
        .get("name")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| p.title.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw::IndexEntry;
    use crate::resolver::{Corrections, Row, Tables};
    use std::collections::BTreeMap;

    fn row(pairs: &[(&str, &str)]) -> Row {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn page(title: &str, text: &str) -> RawPage {
        RawPage {
            title: title.to_string(),
            index: IndexEntry {
                kind: PageKind::Character,
                pageid: 1,
                revid: 1,
                timestamp: "2026-01-01T00:00:00Z".into(),
            },
            text: text.to_string(),
        }
    }

    /// A resolver whose character map comes entirely from `corrections.json`, like
    /// `real.rs` builds one — `player`'s own `id` column is not the source of truth.
    fn resolver(player: Vec<Row>, corrections: Corrections) -> Resolver {
        let tables = Tables {
            player,
            ..Tables::default()
        };
        Resolver::new(&tables, &BTreeMap::new(), &corrections)
    }

    fn corrections(pairs: &[(&str, u32)]) -> Corrections {
        Corrections {
            characters: pairs.iter().map(|(n, id)| (n.to_string(), *id)).collect(),
            ..Corrections::default()
        }
    }

    #[test]
    fn agreeing_forms_are_compared_and_not_mismatched() {
        let pages = vec![
            page(
                "Judas",
                "{{infobox character | id = 3 }}\n\n== Black Judas ==\n{{infobox character | name = Black Judas | id = 12 | parent = Judas }}\n",
            ),
        ];
        let r = resolver(
            vec![
                row(&[("_pageName", "Judas"), ("alias", "Judas"), ("id", "3")]),
                row(&[
                    ("_pageName", "Judas"),
                    ("alias", "Black Judas"),
                    ("id", "12"),
                    ("parent", "Judas"),
                ]),
            ],
            corrections(&[("Judas", 3), ("Black Judas", 12)]),
        );
        let result = cross_check_character_parents(&pages, &r);
        assert_eq!(result.compared, 2);
        assert!(result.mismatches.is_empty(), "{:?}", result.mismatches);
    }

    #[test]
    fn a_disagreeing_parent_is_a_mismatch() {
        let pages = vec![page(
            "Black Judas",
            "{{infobox character | id = 12 | parent = Judas }}\n",
        )];
        // `player`'s row for the same name states a different parent.
        let r = resolver(
            vec![row(&[
                ("_pageName", "Black Judas"),
                ("alias", "Black Judas"),
                ("id", "12"),
                ("parent", "Lazarus"),
            ])],
            corrections(&[("Judas", 3), ("Lazarus", 8), ("Black Judas", 12)]),
        );
        let result = cross_check_character_parents(&pages, &r);
        assert_eq!(result.compared, 1);
        assert_eq!(
            result.mismatches,
            vec![ParentMismatch {
                name: "Black Judas".to_string(),
                wiki_parent: Some(Target::Character { id: 3 }),
                player_parent: Some(Target::Character { id: 8 }),
            }]
        );
    }

    /// B45. The page that states two characters cannot state the second one's parent —
    /// `{{infobox characters}}` has no `parent` parameter — while `player` does. That is a
    /// silence, not a contradiction, and it belongs in its own list: counting it among the
    /// disagreements would make the guard cry wolf four times and stop being read.
    #[test]
    fn a_parent_only_one_source_states_is_not_a_disagreement() {
        let pages = vec![page(
            "Jacob & Esau",
            "{{infobox character | name = Esau }}\n",
        )];
        let r = resolver(
            vec![row(&[
                ("_pageName", "Jacob & Esau"),
                ("alias", "Esau"),
                ("parent", "Jacob"),
            ])],
            corrections(&[("Jacob", 19), ("Esau", 20)]),
        );
        let result = cross_check_character_parents(&pages, &r);
        assert_eq!(result.compared, 1);
        assert!(result.mismatches.is_empty(), "{:?}", result.mismatches);
        assert_eq!(
            result.stated_by_one,
            vec![ParentMismatch {
                name: "Esau".to_string(),
                wiki_parent: None,
                player_parent: Some(Target::Character { id: 19 }),
            }]
        );
    }

    /// A name `player` has no row for at all is not compared: nothing to disagree with.
    #[test]
    fn a_name_absent_from_the_player_table_is_not_compared() {
        let pages = vec![page("The Forgotten", "{{infobox character | id = 16 }}\n")];
        let r = resolver(vec![], corrections(&[("The Forgotten", 16)]));
        let result = cross_check_character_parents(&pages, &r);
        assert_eq!(result.compared, 0);
        assert!(result.mismatches.is_empty());
    }

    /// A `player` row that states no parent (empty string) agrees with an infobox that
    /// carries no `parent` parameter either — the ordinary case for a base character.
    #[test]
    fn both_sides_stating_no_parent_agree() {
        let pages = vec![page("Isaac", "{{infobox character | id = 0 }}\n")];
        let r = resolver(
            vec![row(&[
                ("_pageName", "Isaac"),
                ("alias", "Isaac"),
                ("id", "14"),
                ("parent", ""),
            ])],
            corrections(&[("Isaac", 0)]),
        );
        let result = cross_check_character_parents(&pages, &r);
        assert_eq!(result.compared, 1);
        assert!(result.mismatches.is_empty());
    }
}
