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
use crate::metadata::{self, Metadata};
use crate::minimap;
use crate::players::{self, Character};
use crate::reward;
use crate::sprite::{Rect, SpriteRef};
use crate::strings::Strings;
use crate::text::{Language, Text};
use crate::unlock::{self, Unlock};
use crate::versusscreen::{self, PortraitCrops};

/// The sources, by logical path. Public: callers and tests know what will be asked for.
///
/// Built from [`path_of`], so a path is written once. What this list cannot do is grow by
/// itself: a new source read by `build` and left out of here fails
/// `a_reader_that_has_nothing_yields_one_missing_diagnostic_per_source`.
pub const SOURCES: [(&str, Source); 13] = [
    source(Source::Items),
    source(Source::Metadata),
    source(Source::Strings),
    source(Source::Players),
    source(Source::CoopMenuAnm2),
    source(Source::Achievements),
    source(Source::ItemPools),
    source(Source::Challenges),
    source(Source::BossPortraits),
    source(Source::MinimapIcons),
    source(Source::VersusScreen),
    source(Source::VersusScreenMother),
    source(Source::VersusScreenDogma),
];

const fn source(s: Source) -> (&'static str, Source) {
    (path_of(s), s)
}

/// The logical path of a source. An exhaustive match: a new source without a path fails
/// to compile.
const fn path_of(source: Source) -> &'static str {
    match source {
        Source::Items => "items.xml",
        Source::Metadata => "items_metadata.xml",
        Source::Strings => "stringtable.sta",
        Source::Players => "players.xml",
        Source::CoopMenuAnm2 => "gfx/ui/coop menu.anm2",
        Source::Achievements => "achievements.xml",
        Source::ItemPools => "itempools.xml",
        Source::Challenges => "challenges.xml",
        Source::BossPortraits => "bossportraits.xml",
        Source::MinimapIcons => "gfx/ui/minimap_icons.anm2",
        // The scene `bossportraits.xml` names in its root element, and the two the game puts
        // in its place for one boss each. Named here rather than looked for per row: the
        // catalog asks for the paths on this list and no others, and three files is the whole
        // of what the game has (`versusscreen*.anm2`, measured 2026-09-22).
        Source::VersusScreen => "gfx/ui/boss/versusscreen.anm2",
        Source::VersusScreenMother => "gfx/ui/boss/versusscreen_mother.anm2",
        Source::VersusScreenDogma => "gfx/ui/boss/versusscreen_dogma.anm2",
    }
}

type Items = BTreeMap<(ItemKind, ItemId), Item>;

pub struct Catalog {
    items: Items,
    characters: BTreeMap<CharacterId, Character>,
    achievements: BTreeMap<AchievementId, Achievement>,
    pools: Vec<Pool>,
    challenges: BTreeMap<ChallengeId, Challenge>,
    bosses: BTreeMap<BossId, Boss>,
    strings: Option<Strings>,
    diagnostics: Vec<Diagnostic>,
    unlocks: unlock::Index,
    /// The game's own minimap icons, by the name the game gave each one.
    minimap: BTreeMap<String, SpriteRef>,
}

impl Catalog {
    /// Builds the catalog by asking `read` for the bytes of each source.
    /// Never fails: whatever is missing leaves its part empty and lands in `diagnostics`.
    ///
    /// The sources are read in a fixed order, and the diagnostics come out in it: first what
    /// each file says about itself, then what the files say about each other.
    pub fn build(read: impl FnMut(&str) -> Option<Vec<u8>>) -> Catalog {
        let mut src = Reader {
            read,
            diagnostics: Vec::new(),
        };
        let strings = src
            .fetch(Source::Strings)
            .and_then(|b| Strings::parse(&b, &mut src.diagnostics));
        let mut items = keyed(src.parse(Source::Items, items::parse), |i| (i.kind, i.id));
        merge_metadata(&mut items, &src.parse(Source::Metadata, metadata::parse));
        let mut characters = keyed(src.parse(Source::Players, players::parse), |c| c.id);
        let head_frames = src.parse(Source::CoopMenuAnm2, heads::parse);
        if head_frames.is_empty() {
            src.diagnostics.push(Diagnostic::HeadSheetUnavailable);
        }
        attach_heads(&mut characters, &head_frames);
        let achievements = keyed(src.parse(Source::Achievements, achievements::parse), |a| {
            a.id
        });
        let pools = src.parse(Source::ItemPools, itempools::parse);
        attach_pools(&mut items, &pools);
        let mut challenges = keyed(src.parse(Source::Challenges, challenges::parse), |c| c.id);
        let crops = src.portrait_crops();
        let bosses = keyed(
            src.parse(Source::BossPortraits, |b, d| {
                bossportraits::parse(b, &crops, d)
            }),
            |b| b.id,
        );
        let minimap = src.parse(Source::MinimapIcons, minimap::parse);

        let mut diagnostics = src.diagnostics;
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
            minimap,
        }
    }

    /// One of the game's own minimap icons, by the name the game gave it.
    ///
    /// `None` covers both "no such icon" and "the game is not installed": the caller draws
    /// its own symbol either way, and nothing here invents a crop.
    pub fn minimap_icon(&self, name: &str) -> Option<&SpriteRef> {
        self.minimap.get(name)
    }

    pub fn item(&self, kind: ItemKind, id: ItemId) -> Option<&Item> {
        self.items.get(&(kind, id))
    }

    /// The collectible numbered `id`, whichever of the three collectible kinds it is.
    ///
    /// The wiki's `Item { id }`, a run's `Adding collectible N` and a pool entry all name one
    /// by number alone. Passives, actives and familiars share one id space, so at most one
    /// kind matches; a trinket never does, because it can carry the same number as a
    /// collectible and is not the thing any of them means.
    pub fn collectible(&self, id: ItemId) -> Option<&Item> {
        self.items.get(&(collectible_kind(&self.items, id)?, id))
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

/// The sources as `build` asks for them, and the diagnostics of the asking.
struct Reader<R> {
    read: R,
    diagnostics: Vec<Diagnostic>,
}

impl<R: FnMut(&str) -> Option<Vec<u8>>> Reader<R> {
    /// The bytes of a source, or a `SourceMissing` for it.
    fn fetch(&mut self, source: Source) -> Option<Vec<u8>> {
        let bytes = (self.read)(path_of(source));
        if bytes.is_none() {
            self.diagnostics.push(Diagnostic::SourceMissing { source });
        }
        bytes
    }

    /// A source through its parser, or the parser's empty answer when it is not there.
    fn parse<T: Default>(
        &mut self,
        source: Source,
        parse: impl FnOnce(&[u8], &mut Vec<Diagnostic>) -> T,
    ) -> T {
        match self.fetch(source) {
            Some(bytes) => parse(&bytes, &mut self.diagnostics),
            None => T::default(),
        }
    }

    /// The scenes that say where to cut a portrait: a portrait is a piece of its file, and
    /// which piece is the versus screen's to say (B70). A scene that is not there costs the
    /// crop and nothing else — the row keeps the whole file, the way every build before did.
    fn portrait_crops(&mut self) -> PortraitCrops {
        let default_scene = self.fetch(Source::VersusScreen);
        let own_scenes: Vec<Vec<u8>> = [Source::VersusScreenMother, Source::VersusScreenDogma]
            .into_iter()
            .filter_map(|s| self.fetch(s))
            .collect();
        versusscreen::crops(
            default_scene.as_deref(),
            &own_scenes.iter().map(Vec::as_slice).collect::<Vec<_>>(),
        )
    }
}

/// Rows by the key each one carries.
fn keyed<K: Ord, V>(rows: Vec<V>, key: impl Fn(&V) -> K) -> BTreeMap<K, V> {
    rows.into_iter().map(|v| (key(&v), v)).collect()
}

/// Quality and tags, from `items_metadata.xml` onto the items `items.xml` declared.
fn merge_metadata(items: &mut Items, metadata: &BTreeMap<(ItemKind, ItemId), Metadata>) {
    for ((kind, id), item) in items.iter_mut() {
        if let Some(m) = metadata.get(&(metadata_kind(*kind), *id)) {
            item.quality = m.quality;
            item.tags = m.tags.clone();
        }
    }
}

/// The key the metadata file files a kind under: it doesn't distinguish passives, actives
/// and familiars, all `<item>`.
fn metadata_kind(kind: ItemKind) -> ItemKind {
    match kind {
        ItemKind::Passive | ItemKind::Active | ItemKind::Familiar => ItemKind::Passive,
        ItemKind::Trinket => ItemKind::Trinket,
    }
}

/// Each pool's entries, written on the items they name.
fn attach_pools(items: &mut Items, pools: &[Pool]) {
    let memberships = pools.iter().flat_map(|pool| {
        pool.entries.iter().map(|entry| {
            let membership = PoolMembership {
                pool: pool.name.clone(),
                weight: entry.weight,
            };
            (entry.item, membership)
        })
    });
    for (id, membership) in memberships {
        if let Some(item) = collectible_mut(items, id) {
            item.pools.push(membership);
        }
    }
}

/// The collectible with a bare id, for writing while the catalog is built. Pools only
/// contain collectibles, never trinkets.
fn collectible_mut(items: &mut Items, id: ItemId) -> Option<&mut Item> {
    items.get_mut(&(collectible_kind(items, id)?, id))
}

/// Which of the three collectible kinds carries `id`: they are searched in order and the
/// first one found is the one. The one lookup behind [`Catalog::collectible`] and the pools.
fn collectible_kind(items: &Items, id: ItemId) -> Option<ItemKind> {
    ItemKind::COLLECTIBLES
        .into_iter()
        .find(|&kind| items.contains_key(&(kind, id)))
}

/// Each character's cell of `coop menu.png`, where the map has one and the sheet holds it.
fn attach_heads(characters: &mut BTreeMap<CharacterId, Character>, frames: &[Option<Rect>]) {
    for c in characters.values_mut() {
        c.head = head_of(c.id, frames);
    }
}

fn head_of(id: CharacterId, frames: &[Option<Rect>]) -> Option<SpriteRef> {
    let rect = heads::frame_for(id).and_then(|f| frames.get(f).copied().flatten())?;
    Some(SpriteRef {
        path: heads::SHEET.to_string(),
        rect: Some(rect),
    })
}

/// A `#...` key that resolves in no language of the stringtable (or there's no
/// stringtable at all): diagnosed once per distinct key, not once per element (names
/// and descriptions of several items often share the same missing key).
fn diagnose_unresolved_keys(
    items: &Items,
    characters: &BTreeMap<CharacterId, Character>,
    strings: Option<&Strings>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let resolves = |key: &str| match strings {
        None => false,
        Some(s) => s.languages().iter().any(|&lang| s.get(key, lang).is_some()),
    };
    let keys = items
        .values()
        .flat_map(|i| [&i.name, &i.description])
        .chain(characters.values().map(|c| &c.name))
        .filter_map(|t| match t {
            Text::Key { key } => Some(key),
            Text::Literal { .. } => None,
        });
    let mut seen = HashSet::new();
    diagnostics.extend(
        keys.filter(|key| !resolves(key) && seen.insert(key.as_str()))
            .map(|key| Diagnostic::UnresolvedKey { key: key.clone() }),
    );
}

/// The inverse index: for each achievement, what it unlocks. One pass over the four
/// files that carry `unlocked_by`; each vector is sorted, so the order doesn't depend
/// on reading order.
fn build_unlocks(
    items: &Items,
    characters: &BTreeMap<CharacterId, Character>,
    bosses: &BTreeMap<BossId, Boss>,
    challenges: &BTreeMap<ChallengeId, Challenge>,
) -> unlock::Index {
    let item_links = items
        .iter()
        .filter_map(|(&(kind, id), item)| Some((item.unlocked_by?, Unlock::Item { kind, id })));
    let character_links = characters
        .iter()
        .filter_map(|(&id, c)| Some((c.unlocked_by?, Unlock::Character { id })));
    let boss_links = bosses
        .iter()
        .filter_map(|(&id, b)| Some((b.unlocked_by?, Unlock::Boss { id })));
    let challenge_links = challenges.iter().flat_map(|(&id, ch)| {
        ch.unlocked_by
            .iter()
            .map(move |&a| (a, Unlock::Challenge { id }))
    });
    let mut index = item_links
        .chain(character_links)
        .chain(boss_links)
        .chain(challenge_links)
        .fold(unlock::Index::new(), |mut index, (a, u)| {
            index.entry(a).or_default().push(u);
            index
        });
    index.values_mut().for_each(|v| v.sort());
    index
}

/// Each challenge's reward: the achievements whose note says it was beaten. A challenge the
/// file doesn't have is a diagnostic, not an error. The vectors come out sorted by id.
fn assign_rewards(
    achievements: &BTreeMap<AchievementId, Achievement>,
    challenges: &mut BTreeMap<ChallengeId, Challenge>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let rewards = achievements
        .iter()
        .filter_map(|(&id, a)| Some((id, challenge_rewarded(a)?)));
    for (achievement, challenge) in rewards {
        match challenges.get_mut(&challenge) {
            Some(ch) => ch.rewards.push(achievement),
            None => diagnostics.push(Diagnostic::RewardForUnknownChallenge {
                achievement,
                challenge,
            }),
        }
    }
    for ch in challenges.values_mut() {
        ch.rewards.sort();
    }
}

/// The challenge an achievement's note says was beaten. The note lives in the comment
/// (`unlock_condition`) or, where the comment is missing, in `steam_description`; the
/// comment wins if both are present.
fn challenge_rewarded(a: &Achievement) -> Option<ChallengeId> {
    a.unlock_condition
        .as_deref()
        .and_then(reward::challenge_beaten)
        .or_else(|| {
            a.steam_description
                .as_deref()
                .and_then(reward::challenge_beaten)
        })
}
