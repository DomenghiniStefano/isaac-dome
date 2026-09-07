//! The catalog: construction and access. The reading logic lives in the per-source modules.

use std::collections::{BTreeMap, HashSet};

use crate::achievements::{self, Achievement};
use crate::bossportraits::{self, Boss};
use crate::challenges::{self, Challenge};
use crate::diagnostics::{Diagnostic, Source};
use crate::heads;
use crate::ids::{AchievementId, BossId, ChallengeId, CharacterId, ItemId};
use crate::itempools::{self, Pool, PoolMembership};
use crate::items::{self, Item, ItemKind};
use crate::metadata;
use crate::players::{self, Character};
use crate::reward;
use crate::sprite::SpriteRef;
use crate::strings::Strings;
use crate::text::{Language, Text};
use crate::unlock::{self, Unlock};

/// The sources, by logical path. Public: callers and tests know what will be asked for.
pub const SOURCES: [(&str, Source); 9] = [
    ("items.xml", Source::Items),
    ("items_metadata.xml", Source::Metadata),
    ("stringtable.sta", Source::Strings),
    ("players.xml", Source::Players),
    ("gfx/ui/coop menu.anm2", Source::CoopMenuAnm2),
    ("achievements.xml", Source::Achievements),
    ("itempools.xml", Source::ItemPools),
    ("challenges.xml", Source::Challenges),
    ("bossportraits.xml", Source::BossPortraits),
];

/// The logical path of a source: `build` goes through here, so SOURCES is the only list.
fn path_of(source: Source) -> &'static str {
    // An exhaustive match: a new source without a path fails to compile.
    SOURCES
        .iter()
        .find(|(_, s)| *s == source)
        .map(|(p, _)| *p)
        .unwrap_or_else(|| match source {
            Source::Items
            | Source::Metadata
            | Source::Strings
            | Source::Players
            | Source::CoopMenuAnm2
            | Source::Achievements
            | Source::ItemPools
            | Source::Challenges
            | Source::BossPortraits => "",
        })
}

pub struct Catalog {
    items: BTreeMap<(ItemKind, ItemId), Item>,
    characters: BTreeMap<CharacterId, Character>,
    achievements: BTreeMap<AchievementId, Achievement>,
    pools: Vec<Pool>,
    challenges: BTreeMap<ChallengeId, Challenge>,
    bosses: BTreeMap<BossId, Boss>,
    strings: Option<Strings>,
    diagnostics: Vec<Diagnostic>,
    unlocks: unlock::Index,
}

impl Catalog {
    /// Builds the catalog by asking `read` for the bytes of each source.
    /// Never fails: whatever is missing leaves its part empty and lands in `diagnostics`.
    pub fn build(mut read: impl FnMut(&str) -> Option<Vec<u8>>) -> Catalog {
        let mut diagnostics = Vec::new();
        let mut fetch = |source: Source, d: &mut Vec<Diagnostic>| {
            let bytes = read(path_of(source));
            if bytes.is_none() {
                d.push(Diagnostic::SourceMissing { source });
            }
            bytes
        };
        let strings = fetch(Source::Strings, &mut diagnostics)
            .and_then(|b| Strings::parse(&b, &mut diagnostics));
        let mut items: BTreeMap<(ItemKind, ItemId), Item> = fetch(Source::Items, &mut diagnostics)
            .map(|b| items::parse(&b, &mut diagnostics))
            .unwrap_or_default()
            .into_iter()
            .map(|i| ((i.kind, i.id), i))
            .collect();
        let metadata = fetch(Source::Metadata, &mut diagnostics)
            .map(|b| metadata::parse(&b, &mut diagnostics))
            .unwrap_or_default();
        for ((kind, id), item) in items.iter_mut() {
            // The metadata file doesn't distinguish passives, actives and familiars: all `<item>`.
            let lookup = match kind {
                ItemKind::Passive | ItemKind::Active | ItemKind::Familiar => ItemKind::Passive,
                ItemKind::Trinket => ItemKind::Trinket,
            };
            if let Some(m) = metadata.get(&(lookup, *id)) {
                item.quality = m.quality;
                item.tags = m.tags.clone();
            }
        }
        let mut characters: BTreeMap<CharacterId, Character> =
            fetch(Source::Players, &mut diagnostics)
                .map(|b| players::parse(&b, &mut diagnostics))
                .unwrap_or_default()
                .into_iter()
                .map(|c| (c.id, c))
                .collect();
        let frames = fetch(Source::CoopMenuAnm2, &mut diagnostics)
            .map(|b| heads::parse(&b, &mut diagnostics))
            .unwrap_or_default();
        if frames.is_empty() {
            diagnostics.push(Diagnostic::HeadSheetUnavailable);
        }
        let achievements: BTreeMap<AchievementId, Achievement> =
            fetch(Source::Achievements, &mut diagnostics)
                .map(|b| achievements::parse(&b, &mut diagnostics))
                .unwrap_or_default()
                .into_iter()
                .map(|a| (a.id, a))
                .collect();
        let pools = fetch(Source::ItemPools, &mut diagnostics)
            .map(|b| itempools::parse(&b, &mut diagnostics))
            .unwrap_or_default();
        for pool in &pools {
            for entry in &pool.entries {
                // Pools only contain collectibles (passives, actives, familiars), never
                // trinkets: we search among the three kinds and write to the first one found.
                for kind in [ItemKind::Passive, ItemKind::Active, ItemKind::Familiar] {
                    if let Some(item) = items.get_mut(&(kind, entry.item)) {
                        item.pools.push(PoolMembership {
                            pool: pool.name.clone(),
                            weight: entry.weight,
                        });
                        break;
                    }
                }
            }
        }
        let mut challenges: BTreeMap<ChallengeId, Challenge> =
            fetch(Source::Challenges, &mut diagnostics)
                .map(|b| challenges::parse(&b, &mut diagnostics))
                .unwrap_or_default()
                .into_iter()
                .map(|c| (c.id, c))
                .collect();
        let bosses: BTreeMap<BossId, Boss> = fetch(Source::BossPortraits, &mut diagnostics)
            .map(|b| bossportraits::parse(&b, &mut diagnostics))
            .unwrap_or_default()
            .into_iter()
            .map(|b| (b.id, b))
            .collect();
        for c in characters.values_mut() {
            c.head = heads::frame_for(c.id)
                .and_then(|f| frames.get(f).copied().flatten())
                .map(|rect| SpriteRef {
                    path: heads::SHEET.to_string(),
                    rect: Some(rect),
                });
        }
        assign_rewards(&achievements, &mut challenges, &mut diagnostics);
        let unlocks = build_unlocks(&items, &characters, &bosses, &challenges);
        diagnose_unresolved_keys(&items, &characters, strings.as_ref(), &mut diagnostics);
        Catalog {
            items,
            characters,
            achievements,
            pools,
            challenges,
            bosses,
            strings,
            diagnostics,
            unlocks,
        }
    }

    pub fn item(&self, kind: ItemKind, id: ItemId) -> Option<&Item> {
        self.items.get(&(kind, id))
    }

    /// All the items, ordered by kind and then by id.
    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.items.values()
    }

    pub fn character(&self, id: CharacterId) -> Option<&Character> {
        self.characters.get(&id)
    }

    pub fn characters(&self) -> impl Iterator<Item = &Character> {
        self.characters.values()
    }

    pub fn achievement(&self, id: AchievementId) -> Option<&Achievement> {
        self.achievements.get(&id)
    }

    /// All the achievements, ordered by id.
    pub fn achievements(&self) -> impl Iterator<Item = &Achievement> {
        self.achievements.values()
    }

    /// What an achievement unlocks. Empty if nothing cites it: that's data, not an error.
    pub fn unlocks(&self, id: AchievementId) -> &[Unlock] {
        self.unlocks.get(&id).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn pools(&self) -> &[Pool] {
        &self.pools
    }

    pub fn challenge(&self, id: ChallengeId) -> Option<&Challenge> {
        self.challenges.get(&id)
    }

    /// All the challenges, ordered by id.
    pub fn challenges(&self) -> impl Iterator<Item = &Challenge> {
        self.challenges.values()
    }

    pub fn boss(&self, id: BossId) -> Option<&Boss> {
        self.bosses.get(&id)
    }

    /// All the bosses, ordered by id.
    pub fn bosses(&self) -> impl Iterator<Item = &Boss> {
        self.bosses.values()
    }

    /// The text of `t`: in the requested language, else in English, else the key.
    /// Never `None`: there's always a label.
    pub fn text<'a>(&'a self, t: &'a Text, lang: Language) -> &'a str {
        match t {
            Text::Literal { text } => text,
            Text::Key { key } => self
                .strings
                .as_ref()
                .and_then(|s| s.get(key, lang).or_else(|| s.get(key, Language::English)))
                .unwrap_or(key),
        }
    }

    pub fn languages(&self) -> &[Language] {
        self.strings.as_ref().map(|s| s.languages()).unwrap_or(&[])
    }
}

/// A `#...` key that resolves in no language of the stringtable (or there's no
/// stringtable at all): diagnosed once per distinct key, not once per element (names
/// and descriptions of several items often share the same missing key).
fn diagnose_unresolved_keys(
    items: &BTreeMap<(ItemKind, ItemId), Item>,
    characters: &BTreeMap<CharacterId, Character>,
    strings: Option<&Strings>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let resolves = |key: &str| match strings {
        None => false,
        Some(s) => s.languages().iter().any(|&lang| s.get(key, lang).is_some()),
    };
    let mut seen = HashSet::new();
    let texts = items
        .values()
        .flat_map(|i| [&i.name, &i.description])
        .chain(characters.values().map(|c| &c.name));
    for t in texts {
        if let Text::Key { key } = t {
            if !resolves(key) && seen.insert(key.clone()) {
                diagnostics.push(Diagnostic::UnresolvedKey { key: key.clone() });
            }
        }
    }
}

/// The inverse index: for each achievement, what it unlocks. One pass over the four
/// files that carry `unlocked_by`; each vector is sorted, so the order doesn't depend
/// on reading order.
fn build_unlocks(
    items: &BTreeMap<(ItemKind, ItemId), Item>,
    characters: &BTreeMap<CharacterId, Character>,
    bosses: &BTreeMap<BossId, Boss>,
    challenges: &BTreeMap<ChallengeId, Challenge>,
) -> unlock::Index {
    let mut index: unlock::Index = BTreeMap::new();
    let mut push = |a: AchievementId, u: Unlock| index.entry(a).or_default().push(u);
    for ((kind, id), item) in items {
        if let Some(a) = item.unlocked_by {
            push(
                a,
                Unlock::Item {
                    kind: *kind,
                    id: *id,
                },
            );
        }
    }
    for (id, c) in characters {
        if let Some(a) = c.unlocked_by {
            push(a, Unlock::Character { id: *id });
        }
    }
    for (id, b) in bosses {
        if let Some(a) = b.unlocked_by {
            push(a, Unlock::Boss { id: *id });
        }
    }
    for (id, ch) in challenges {
        for a in &ch.unlocked_by {
            push(*a, Unlock::Challenge { id: *id });
        }
    }
    for v in index.values_mut() {
        v.sort();
    }
    index
}

/// Each challenge's reward: the achievements whose note says it was beaten. The note
/// lives in the comment (`unlock_condition`) or, where the comment is missing, in
/// `steam_description`; the comment wins if both are present. A challenge the file
/// doesn't have is a diagnostic, not an error. The vectors come out sorted by id.
fn assign_rewards(
    achievements: &BTreeMap<AchievementId, Achievement>,
    challenges: &mut BTreeMap<ChallengeId, Challenge>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (id, a) in achievements {
        let beaten = a
            .unlock_condition
            .as_deref()
            .and_then(reward::challenge_beaten)
            .or_else(|| {
                a.steam_description
                    .as_deref()
                    .and_then(reward::challenge_beaten)
            });
        let Some(challenge) = beaten else {
            continue;
        };
        match challenges.get_mut(&challenge) {
            Some(ch) => ch.rewards.push(*id),
            None => diagnostics.push(Diagnostic::RewardForUnknownChallenge {
                achievement: *id,
                challenge,
            }),
        }
    }
    for ch in challenges.values_mut() {
        ch.rewards.sort();
    }
}
