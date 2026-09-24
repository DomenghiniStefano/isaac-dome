//! From a generated ref to a typed requirement, against the catalog the user actually
//! has. Nothing here reads a file.

use std::collections::HashMap;

use catalog::{BossId, Catalog, ChallengeId, CharacterId, ItemId, ItemKind, Language};
use wiki::Target;

use crate::model::{Requirement, ThresholdItem};
use crate::rules::{target_key, RefRow, Rules, Verdict};

/// English names to ids. Built once per graph: resolution is by name, so this is the hot
/// path and a per-ref linear scan over 909 items would show.
pub struct NameIndex {
    characters: HashMap<String, CharacterId>,
    bosses: HashMap<String, BossId>,
    items: HashMap<String, (ItemKind, ItemId)>,
}

fn key(s: &str) -> String {
    s.trim().to_lowercase()
}

impl NameIndex {
    pub fn new(c: &Catalog) -> NameIndex {
        let en = Language::English;
        let mut characters = HashMap::new();
        for ch in c.characters() {
            characters.insert(key(c.text(&ch.name, en)), ch.id);
        }
        let mut bosses = HashMap::new();
        for b in c.bosses() {
            bosses.insert(key(&b.name), b.id);
        }
        let mut items = HashMap::new();
        for i in c.items() {
            items.insert(key(c.text(&i.name, en)), (i.kind, i.id));
        }
        NameIndex {
            characters,
            bosses,
            items,
        }
    }

    pub fn character(&self, name: &str) -> Option<CharacterId> {
        self.characters.get(&key(name)).copied()
    }

    pub fn boss(&self, name: &str) -> Option<BossId> {
        self.bosses.get(&key(name)).copied()
    }

    pub fn item(&self, name: &str) -> Option<(ItemKind, ItemId)> {
        self.items.get(&key(name)).copied()
    }
}

/// One ref, one outcome. Convenience for tests and single lookups: it builds the index
/// every time, so anything resolving many refs uses `requirement_with`.
#[cfg(feature = "test-api")]
pub fn requirement(
    c: &Catalog,
    rules: &Rules,
    row: &RefRow,
    character: Option<CharacterId>,
) -> Requirement {
    let index = NameIndex::new(c);
    requirement_with(c, rules, &index, row, character)
}

/// The form used when resolving many refs: the caller builds the index once.
///
/// Never `None` by omission — a target we can't judge becomes `Requirement::Unknown` and
/// demotes its node to `Partial`.
///
/// `character` is the character the **same achievement** names, if any. It is a property of
/// the sentence, not of this reference: "defeat Mother as Magdalene" is one requirement
/// spread over two references, and the boss reference is the one that needs to know.
pub fn requirement_with(
    c: &Catalog,
    rules: &Rules,
    index: &NameIndex,
    row: &RefRow,
    character: Option<CharacterId>,
) -> Requirement {
    let label = rules.alias(&row.label).to_string();
    let verdict_key = target_key(&row.target, &label);
    let unknown = || Requirement::Unknown {
        label: label.clone(),
    };
    match &row.target {
        Target::Character { id } => index
            .character(&label)
            .or_else(|| c.character(CharacterId(*id)).map(|ch| ch.id))
            .map(|id| Requirement::Character { id })
            .unwrap_or_else(unknown),
        // By name, never by id: the wiki's entity id is the game's entity type, ours comes
        // from `bossportraits.xml`, and the two don't line up (Gish is entity 43, boss 19).
        //
        // Resolving to a boss is **not** enough to stop here. A boss the game says nothing
        // about — Hush, Delirium, Mother, The Beast have no `achievement=` in
        // `bossportraits.xml` — would otherwise produce no edge and no unknown, and the
        // node would read as "nothing in the way" while Delirium sits behind The Void.
        // Only a boss the game itself gates by an achievement short-circuits the verdict
        // table; everything else has to be judged.
        Target::Entity { .. } => match index.boss(&label) {
            Some(id) if c.boss(id).and_then(|b| b.unlocked_by).is_some() => {
                Requirement::Boss { id }
            }
            // A boss the game does not gate, or no boss by that name: the verdict decides.
            Some(_) | None => from_verdict(rules, &verdict_key, character, unknown),
        },
        Target::Challenge { number } => c
            .challenge(ChallengeId(*number))
            .map(|ch| Requirement::Challenge { id: ch.id })
            .unwrap_or_else(unknown),
        Target::Item { id } | Target::Trinket { id } => index
            .item(&label)
            .or_else(|| {
                [
                    ItemKind::Passive,
                    ItemKind::Active,
                    ItemKind::Familiar,
                    ItemKind::Trinket,
                ]
                .into_iter()
                .find_map(|k| c.item(k, ItemId(*id)).map(|i| (i.kind, i.id)))
            })
            .map(|(kind, id)| Requirement::Item { kind, id })
            .unwrap_or_else(unknown),
        // An achievement referenced directly is already a node. It travels as a gate key
        // so that `build` has one place that turns requirements into edges.
        Target::Achievement { id } => Requirement::Gate {
            gate: format!("achievement:{id}"),
        },
        // A transformation no longer reaches the verdict table: the rules file carries its
        // count and its items, so it can be answered rather than judged.
        Target::Transformation { id } => threshold(c, rules, *id, &label, unknown),
        Target::Stage { .. } | Target::Room { .. } | Target::Concept { .. } => {
            from_verdict(rules, &verdict_key, character, unknown)
        }
    }
}

/// One contributor of a transformation, by **id alone**. Unlike a reference in a sentence
/// there is no label to resolve by: the id comes from the wiki's own collectible table,
/// which numbers items the way the game does.
fn contributor(c: &Catalog, t: &Target) -> Option<ThresholdItem> {
    let id = match t {
        Target::Item { id } | Target::Trinket { id } => ItemId(*id),
        Target::Character { .. }
        | Target::Achievement { .. }
        | Target::Challenge { .. }
        | Target::Entity { .. }
        | Target::Transformation { .. }
        | Target::Stage { .. }
        | Target::Room { .. }
        | Target::Concept { .. } => return None,
    };
    [
        ItemKind::Passive,
        ItemKind::Active,
        ItemKind::Familiar,
        ItemKind::Trinket,
    ]
    .into_iter()
    .find_map(|k| {
        c.item(k, id).map(|i| ThresholdItem {
            kind: i.kind,
            id: i.id,
            unlocked_by: i.unlocked_by,
        })
    })
}

/// A transformation becomes a `Threshold` only when the rules file states a count **and**
/// names at least that many items. Two refusals, and each has its reason:
///
/// - **no count** — `at_least` is a number, not an option, because a threshold that can be
///   built without one would let "unknown" be mistaken for "zero", which everything meets;
/// - **fewer items than the count** — Stompy's shape on the real snapshot: two collectibles
///   and a count of three, because its third contributor is a pill and this model carries
///   no pills. A threshold nothing can ever meet would say "you are one item away" forever.
///
/// The length checked is the **wiki's** list, not the resolved one: an item missing from
/// this catalog is `unresolved`, which evaluation treats as "can only help", and is a
/// different thing from a page that never named enough items at all.
fn threshold(
    c: &Catalog,
    rules: &Rules,
    id: u32,
    label: &str,
    unknown: impl Fn() -> Requirement,
) -> Requirement {
    let Some(row) = rules.transformation(id) else {
        return unknown();
    };
    let Some(at_least) = row.at_least else {
        return unknown();
    };
    if (row.items.len() as u32) < at_least {
        return unknown();
    }
    let mut of = Vec::new();
    let mut unresolved = 0;
    for t in &row.items {
        match contributor(c, t) {
            Some(item) => of.push(item),
            None => unresolved += 1,
        }
    }
    Requirement::Threshold {
        transformation: id,
        label: label.to_string(),
        at_least,
        of,
        unresolved,
    }
}

fn from_verdict(
    rules: &Rules,
    key: &str,
    character: Option<CharacterId>,
    unknown: impl Fn() -> Requirement,
) -> Requirement {
    match rules.verdict(key) {
        Some(Verdict::AlwaysAvailable(_)) | Some(Verdict::NotAPrerequisite(_)) => Requirement::None,
        Some(Verdict::Behind { .. }) => Requirement::Gate {
            gate: key.to_string(),
        },
        // Which half answers is decided by the reference: one that names a character asks
        // about that character's cell, one that names only the boss asks whether it was
        // ever done at all. The half it needs may be absent — Ultra Greedier has no located
        // tally — and then this is `Unknown` like anything else the crate can't say. It
        // must not fall back to the other half, which would answer a question nobody asked.
        Some(Verdict::Progress { mark, counter }) => match (character, mark, counter) {
            (Some(ch), Some(m), _) => Requirement::Mark {
                character: ch,
                column: m.column,
                level: m.level,
            },
            (None, _, Some(c)) => Requirement::Counter {
                name: c.name,
                at_least: c.at_least,
            },
            (Some(_), None, _) | (None, _, None) => unknown(),
        },
        // Judged inexpressible and never judged land in the same place: the node drops to
        // `Partial` either way. The difference is recorded in `corrections.json`, for
        // whoever reads it next, not in the value.
        Some(Verdict::Unknown { .. }) | None => unknown(),
    }
}
