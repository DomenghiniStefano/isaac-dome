//! `wiki_progress`: the save's state for every wiki page that has one (design decision 6,
//! `2026-09-27-wiki-restyle-design.md`). A pure function over what `collection`, `challenges`,
//! `marks` and `graph_views` already read from the save and the catalog — never a second read
//! of the file (`crate::wiki::pages` is the one walk of the dataset, the same `wiki_index` and
//! `search` already share, so a page kind cannot go missing from one and not the other, B46).
//!
//! A page absent from `pages` has no state: a transformation, a stage, a version or a pickup
//! article (the save says nothing about any of them, decision 6's table), or a page whose only
//! source of state is a section that did not read and whose `PageProgress` shape has no way to
//! say "unknown" (`Achievement.done`, `Unlockable`'s two fields). Everywhere the shape *can*
//! say "unknown" (every `Option` field below), an unreadable section degrades that one field,
//! never the row it sits in — Review Focus 1.

use std::collections::HashMap;

use catalog::{Catalog, ChallengeId, CharacterId, Item, ItemKind};
use core_save::{Bestiary, Column};
use serde::Serialize;
use wiki::{ArticleCategory, Dataset, DatasetError, Entry, Infobox, Target};

use crate::challenges::{state_of as challenge_state_of, Profile as ChallengeProfile};
use crate::collection::{lock_of, LockView};
use crate::flags::{recorded, recorded_done};
use crate::marks::{cell_at, row_for_character, Cell};
use crate::ChallengeStateView;

/// One page's state, one variant per kind decision 6's table gives any.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PageProgress {
    Achievement {
        done: bool,
    },
    Item {
        collected: Option<bool>,
        unlocked: Option<bool>,
        unlocked_by: Option<u32>,
    },
    /// A trinket, a card or a rune article: unlocked from the single achievement that gates
    /// it. Both fields are plain, not `Option`, because the variant itself only appears once
    /// there is a concrete achievement id to name — see `unlockable_from_catalog` and
    /// `unlockable_from_wiki`.
    Unlockable {
        unlocked: bool,
        unlocked_by: u32,
    },
    Character {
        unlocked: Option<bool>,
        marks_done: u32,
        marks_total: u32,
    },
    Challenge {
        state: ChallengeStateView,
    },
    /// A boss or a common enemy, keyed by its own `type.variant.subtype` — the bestiary's own
    /// key, the same one `Target::Entity` carries. Two pages that reach the same key (a boss
    /// and an entity page both naming Peep Eye, Review Focus 4) each read this key
    /// independently and get the same numbers, never a shared, halved total.
    Bestiary {
        met: u32,
        killed: u32,
        killed_you: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct PageProgressEntry {
    pub target: Target,
    pub progress: PageProgress,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct WikiProgress {
    pub pages: Vec<PageProgressEntry>,
}

/// Everything `wiki_progress` reads: the same sections `collection_view`, `challenges_view`
/// and `marks_matrix` take, plus the bestiary and the catalog. A struct and not seven
/// parameters, the way `UnlockInputs` already is: a list that long is hard to call correctly.
#[derive(Clone, Copy)]
pub struct WikiProgressInputs<'a> {
    pub dataset: Result<&'a Dataset, &'a DatasetError>,
    pub catalog: Option<&'a Catalog>,
    /// Section 1.
    pub achievements: Option<&'a [bool]>,
    /// Section 4.
    pub items: Option<&'a [bool]>,
    /// Section 7.
    pub challenges: Option<&'a [bool]>,
    /// The completion matrix's counters, the same slice `marks_matrix` reads.
    pub counters: Option<&'a [u32]>,
    /// Section 10, already read as its declared tallies.
    pub bestiary: Option<&'a Bestiary>,
}

/// The save's state for every page the dataset has one for. `Err(dataset)` gives no pages, the
/// same degrade `wiki_index` gives the rest of the page list.
pub fn wiki_progress(inputs: WikiProgressInputs<'_>) -> WikiProgress {
    let Ok(ds) = inputs.dataset else {
        return WikiProgress { pages: Vec::new() };
    };
    let collectibles = items_by_id(inputs.catalog, false);
    let trinkets = items_by_id(inputs.catalog, true);
    let pages = crate::wiki::pages(ds)
        .filter_map(|(target, entry)| {
            let progress = progress_of(&inputs, &collectibles, &trinkets, &target, entry)?;
            Some(PageProgressEntry { target, progress })
        })
        .collect();
    WikiProgress { pages }
}

/// The catalog's items, by id, split by whether they are a trinket: `Target::Item` and
/// `Target::Trinket` are two separate id spaces in the wiki, but one `items()` list in the
/// catalog (`collection.rs`'s own doc comment). Built once per call, not once per page.
fn items_by_id(catalog: Option<&Catalog>, trinket: bool) -> HashMap<u32, &Item> {
    catalog
        .into_iter()
        .flat_map(Catalog::items)
        .filter(|i| (i.kind == ItemKind::Trinket) == trinket)
        .map(|i| (i.id.0, i))
        .collect()
}

/// One page's progress, dispatched by its own infobox kind — the same thing decision 6's
/// table dispatches on. Exhaustive over `Infobox`: a ninth kind fails to build here until it
/// says which arm it takes.
fn progress_of(
    inputs: &WikiProgressInputs<'_>,
    collectibles: &HashMap<u32, &Item>,
    trinkets: &HashMap<u32, &Item>,
    target: &Target,
    entry: &Entry,
) -> Option<PageProgress> {
    match &entry.infobox {
        Infobox::Achievement { .. } => {
            let Target::Achievement { id } = target else {
                return None; // unreachable: an achievement target's entry is always Achievement
            };
            achievement_progress(*id, inputs.achievements)
        }
        Infobox::Item { .. } => {
            let Target::Item { id } = target else {
                return None; // unreachable: an item target's entry is always Item
            };
            Some(item_progress(inputs, collectibles, *id))
        }
        Infobox::Trinket { .. } => {
            let Target::Trinket { id } = target else {
                return None; // unreachable: a trinket target's entry is always Trinket
            };
            unlockable_from_catalog(trinkets, *id, inputs.achievements)
        }
        Infobox::Challenge { .. } => {
            let Target::Challenge { number } = target else {
                return None; // unreachable: a challenge target's entry is always Challenge
            };
            challenge_progress(inputs, *number)
        }
        Infobox::Character { .. } => {
            let Target::Character { id } = target else {
                return None; // unreachable: a character target's entry is always Character
            };
            character_progress(inputs, *id)
        }
        Infobox::Boss { .. } | Infobox::Entity { .. } => {
            let Target::Entity {
                id,
                variant,
                subtype,
            } = target
            else {
                return None; // unreachable: a boss/entity target's entry is always Entity
            };
            bestiary_progress(inputs.bestiary, *id, *variant, *subtype)
        }
        Infobox::Article { category } => article_progress(*category, entry, inputs.achievements),
        // The save states nothing about a transformation (decision 6's table).
        Infobox::Transformation { .. } => None,
    }
}

/// `done` only when section 1 read: a false claim of "not done" is worse than no row.
fn achievement_progress(id: u32, achievements: Option<&[bool]>) -> Option<PageProgress> {
    achievements.map(|f| PageProgress::Achievement {
        done: recorded_done(f, id),
    })
}

/// `collected` reads section 4 directly, by the game's own id — no catalog needed. `unlocked`
/// and `unlocked_by` need the catalog's own record of the item, for the same tri-state
/// `collection_view` already draws (`lock_of`): degrade one field at a time, never the row.
fn item_progress(
    inputs: &WikiProgressInputs<'_>,
    collectibles: &HashMap<u32, &Item>,
    id: u32,
) -> PageProgress {
    let collected = recorded(inputs.items, id);
    let (unlocked, unlocked_by) = match (inputs.catalog, collectibles.get(&id)) {
        (Some(c), Some(item)) => lock_tuple(lock_of(
            c,
            inputs.dataset.ok(),
            item.unlocked_by,
            inputs.achievements,
        )),
        _ => (None, None),
    };
    PageProgress::Item {
        collected,
        unlocked,
        unlocked_by,
    }
}

/// `LockView`'s four cases as the two-field shape `PageProgress::Item` carries: free reads as
/// unlocked with no gate to name, same as an item with nothing standing in front of it.
fn lock_tuple(lock: LockView) -> (Option<bool>, Option<u32>) {
    match lock {
        LockView::Free => (Some(true), None),
        LockView::Unlocked { achievement, .. } => (Some(true), Some(achievement)),
        LockView::Locked { achievement, .. } => (Some(false), Some(achievement)),
        LockView::Unknown { achievement, .. } => (None, Some(achievement)),
    }
}

/// A trinket's gate, from the catalog's own `unlocked_by` — the same field `collection_view`
/// reads for a passive or active item, just never joined for a trinket there (trinkets have no
/// collected state, `collection.rs`'s own doc comment). `None` when the trinket is free or the
/// catalog doesn't have it: `PageProgress::Unlockable` has no field to carry "free" in.
fn unlockable_from_catalog(
    trinkets: &HashMap<u32, &Item>,
    id: u32,
    achievements: Option<&[bool]>,
) -> Option<PageProgress> {
    let item = *trinkets.get(&id)?;
    let achievement = item.unlocked_by?.0;
    Some(PageProgress::Unlockable {
        unlocked: achievements.is_some_and(|f| recorded_done(f, achievement)),
        unlocked_by: achievement,
    })
}

/// A card or a rune: the catalog has no record of either at all (they're not in
/// `items.xml`), so the only stated gate is the wiki's own `Entry::unlocked_by` — "what the
/// wiki states has to be unlocked first" (`model.rs`'s own doc comment on the field). `None`
/// when the wiki names no achievement, or names something else: never a guessed gate.
fn unlockable_from_wiki(entry: &Entry, achievements: Option<&[bool]>) -> Option<PageProgress> {
    let Some(Target::Achievement { id }) = &entry.unlocked_by else {
        return None;
    };
    let id = *id;
    Some(PageProgress::Unlockable {
        unlocked: achievements.is_some_and(|f| recorded_done(f, id)),
        unlocked_by: id,
    })
}

/// Only a card or a rune article carries a gate worth showing (decision 6's table); a pickup,
/// a stage, a version or a plain mechanics page (`category: None`) says nothing.
fn article_progress(
    category: Option<ArticleCategory>,
    entry: &Entry,
    achievements: Option<&[bool]>,
) -> Option<PageProgress> {
    match category {
        Some(ArticleCategory::Card | ArticleCategory::Rune) => {
            unlockable_from_wiki(entry, achievements)
        }
        Some(ArticleCategory::Pickup | ArticleCategory::Stage | ArticleCategory::Version)
        | None => None,
    }
}

/// A challenge's state, from the same gating `challenges_view` draws — without a catalog
/// there is no `catalog::Challenge` to read the gate off, so the page carries no state at all
/// rather than a state with the gate missing.
fn challenge_progress(inputs: &WikiProgressInputs<'_>, number: u32) -> Option<PageProgress> {
    let c = inputs.catalog?;
    let ch = c.challenge(ChallengeId(number))?;
    let profile = ChallengeProfile {
        challenges: inputs.challenges,
        achievements: inputs.achievements,
    };
    Some(PageProgress::Challenge {
        state: challenge_state_of(ch, profile),
    })
}

/// A character's unlock (free, or gated by its own achievement) and its marks, looked up by
/// id and Tainted flag — never by name, which the base and Tainted forms share (Review Focus
/// 3, B28). Without a catalog there is no way to resolve either, so the page carries no state.
fn character_progress(inputs: &WikiProgressInputs<'_>, id: u32) -> Option<PageProgress> {
    let c = inputs.catalog?;
    let ch = c.character(CharacterId(id))?;
    let unlocked = match ch.unlocked_by {
        None => Some(true),
        Some(a) => inputs.achievements.map(|f| recorded_done(f, a.0)),
    };
    let (marks_done, marks_total) = marks_of(inputs.counters, row_for_character(ch));
    Some(PageProgress::Character {
        unlocked,
        marks_done,
        marks_total,
    })
}

/// The marks taken and the columns there are to take, for the roster row a character
/// occupies. `(0, 0)` for a hidden form with no row (Lazarus 2, Black Judas, The Soul): that
/// is the true count of columns a form outside the matrix has, not a degrade standing in for
/// one.
fn marks_of(counters: Option<&[u32]>, row: Option<usize>) -> (u32, u32) {
    let Some(row) = row else {
        return (0, 0);
    };
    let counters = counters.unwrap_or(&[]);
    let done = Column::ALL
        .iter()
        .filter(|&&column| {
            matches!(cell_at(counters, row, column), Cell::Known { level, .. } if level.reached())
        })
        .count() as u32;
    (done, Column::ALL.len() as u32)
}

/// A boss or entity's tallies, keyed by its own bestiary triple. `None` only when the section
/// didn't read at all; an entity the section read but never recorded is a real zero, not an
/// unknown (decision 7: the numbers are shown as the save holds them).
fn bestiary_progress(
    bestiary: Option<&Bestiary>,
    id: u32,
    variant: u32,
    subtype: u32,
) -> Option<PageProgress> {
    let b = bestiary?;
    let count = |tally_id: u32| -> u32 {
        b.tallies
            .iter()
            .find(|t| t.id == tally_id)
            .and_then(|t| {
                t.records.iter().find(|r| {
                    r.entity.kind == id
                        && r.entity.variant == variant
                        && r.entity.subtype == subtype
                })
            })
            .map(|r| r.count)
            .unwrap_or(0)
    };
    // Tally 1 = met, 2 = killed, 4 = killed you; tally 3 is never named or shown
    // (`docs/save-format.md`, "The bestiary (section 10)").
    Some(PageProgress::Bestiary {
        met: count(1),
        killed: count(2),
        killed_you: count(4),
    })
}
