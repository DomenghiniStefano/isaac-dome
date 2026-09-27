//! View-model for the wiki dataset: the state of the embedded dataset (loaded or not,
//! and how up to date it is against the game installation) and the model types that
//! cross the IPC boundary unchanged — a single set of names shared between `wiki` and
//! the frontend.

use catalog::Catalog;
use serde::Serialize;

pub use wiki::{
    ArticleCategory, Block, Dlc, Entry, Infobox, Inline, ListItem, Section, SectionKind, Style,
    Target,
};
use wiki::{Dataset, DatasetError};

use crate::icon::IconRef;
use crate::target_sprite::{target_sprite, BossKeys, TargetSprite};

/// One page of the dataset: its identity, its own title, the link to its figure when
/// the catalog draws one, and which landing tile / sidebar category it belongs to.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct WikiPageRef {
    pub target: Target,
    pub title: String,
    pub icon_url: Option<String>,
    pub category: Option<WikiPageCategory>,
}

/// A page's landing tile / sidebar category (design decisions 5 and 7). Fieldless: a bare
/// string, and its values are chosen to equal the frontend's own `WikiCategory`
/// (`routeTable.ts`) member for member, so a value crossing the IPC needs no translation —
/// the two are structurally the same union, kept as two names because `WikiCategory` also
/// carries seven kinds this crate had no reason to name before.
///
/// **Why this exists at all, rather than being read from `Target` alone at the frontend**:
/// `Target::Entity` covers both a boss and a common enemy (design decision 2) and a
/// `Target::Article` covers four different landing tiles or none, and neither distinction
/// survives in the wire shape of `Target` — it is `entry.infobox`'s variant that says which,
/// and only `wiki_index` (here) still has the entry when it builds each page's reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum WikiPageCategory {
    Items,
    Trinkets,
    Achievements,
    Bosses,
    Challenges,
    Characters,
    Transformations,
    Monsters,
    CardsAndRunes,
    Pickups,
    Stages,
    Versions,
}

/// Every category, once, in the enum's declaration order: what the landing walks to build a
/// tile for each. Written out by hand, like `floor::ROOM_KINDS` — nothing but review holds it
/// complete, since a `#[serde(rename_all)]` fieldless enum has no `strum`-style iterator here.
pub const WIKI_PAGE_CATEGORIES: [WikiPageCategory; 12] = [
    WikiPageCategory::Items,
    WikiPageCategory::Trinkets,
    WikiPageCategory::Achievements,
    WikiPageCategory::Bosses,
    WikiPageCategory::Challenges,
    WikiPageCategory::Characters,
    WikiPageCategory::Transformations,
    WikiPageCategory::Monsters,
    WikiPageCategory::CardsAndRunes,
    WikiPageCategory::Pickups,
    WikiPageCategory::Stages,
    WikiPageCategory::Versions,
];

/// The category `target`'s own page belongs to, from `entry`'s infobox — which is what
/// tells a boss from a common enemy, and an article's declined-infobox category from
/// another article's. `None` for the three kinds with no page and for an article with no
/// category (mechanics, concepts, machines, list pages): neither has a landing tile.
fn page_category(target: &Target, entry: &Entry) -> Option<WikiPageCategory> {
    match target {
        Target::Item { .. } => Some(WikiPageCategory::Items),
        Target::Trinket { .. } => Some(WikiPageCategory::Trinkets),
        Target::Achievement { .. } => Some(WikiPageCategory::Achievements),
        Target::Challenge { .. } => Some(WikiPageCategory::Challenges),
        Target::Character { .. } => Some(WikiPageCategory::Characters),
        Target::Transformation { .. } => Some(WikiPageCategory::Transformations),
        Target::Entity { .. } => match &entry.infobox {
            Infobox::Boss { .. } => Some(WikiPageCategory::Bosses),
            Infobox::Entity { .. } => Some(WikiPageCategory::Monsters),
            Infobox::Item { .. }
            | Infobox::Trinket { .. }
            | Infobox::Achievement { .. }
            | Infobox::Challenge { .. }
            | Infobox::Transformation { .. }
            | Infobox::Character { .. }
            | Infobox::Article { .. } => None, // unreachable: an entity target's entry is always Boss or Entity
        },
        Target::Article { .. } => match &entry.infobox {
            Infobox::Article {
                category: Some(ArticleCategory::Card | ArticleCategory::Rune),
            } => Some(WikiPageCategory::CardsAndRunes),
            Infobox::Article {
                category: Some(ArticleCategory::Pickup),
            } => Some(WikiPageCategory::Pickups),
            Infobox::Article {
                category: Some(ArticleCategory::Stage),
            } => Some(WikiPageCategory::Stages),
            Infobox::Article {
                category: Some(ArticleCategory::Version),
            } => Some(WikiPageCategory::Versions),
            Infobox::Article { category: None } => None,
            Infobox::Item { .. }
            | Infobox::Trinket { .. }
            | Infobox::Achievement { .. }
            | Infobox::Boss { .. }
            | Infobox::Challenge { .. }
            | Infobox::Transformation { .. }
            | Infobox::Character { .. }
            | Infobox::Entity { .. } => None, // unreachable: an article target's entry is always Article
        },
        Target::Stage { .. } | Target::Room { .. } | Target::Concept { .. } => None,
    }
}

/// Every page the dataset has, once per window (spec 3.5, Decision 2): what the tab labels,
/// the category lists and the icon of every reference inside a page are read from.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct WikiIndex {
    pub info: WikiInfo,
    pub pages: Vec<WikiPageRef>,
    /// One representative picture per landing tile (design decision 5's "as many pictures as
    /// the game gives"), in `WIKI_PAGE_CATEGORIES` order. `icon_url: None` is the fallback
    /// icon, the same drawing the tile has without the game — not a broken image.
    pub samples: Vec<CategorySample>,
}

/// A landing tile's own picture, by its category: not a random find, a checked one — see
/// `category_sample`.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CategorySample {
    pub category: WikiPageCategory,
    pub icon_url: Option<String>,
}

/// The index, in the dataset's order: by kind, then by id. Bosses are keyed by a string
/// (`Dataset::boss_key`), so they come out in string order — the screen sorts by title
/// anyway. A link goes out only when the catalog resolves the figure: `None` is "no
/// picture", drawn as the placeholder, never as a broken image.
pub fn wiki_index(
    dataset: Result<&Dataset, &DatasetError>,
    catalog: Option<&Catalog>,
    bosses: &BossKeys,
    game_updated_unix: Option<u64>,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> WikiIndex {
    let info = wiki_info(dataset, game_updated_unix);
    let Ok(ds) = dataset else {
        return WikiIndex {
            info,
            pages: Vec::new(),
            samples: Vec::new(),
        };
    };
    let pages = pages(ds)
        .map(|(target, entry)| WikiPageRef {
            icon_url: catalog.and_then(|c| match target_sprite(c, bosses, Some(ds), &target) {
                TargetSprite::Found(_) | TargetSprite::Entity(_) => icon(&IconRef::Page {
                    target: target.clone(),
                }),
                TargetSprite::NoArt | TargetSprite::Unknown => None,
            }),
            title: entry.title.clone(),
            category: page_category(&target, entry),
            target,
        })
        .collect();
    let samples = WIKI_PAGE_CATEGORIES
        .into_iter()
        .map(|category| CategorySample {
            category,
            icon_url: category_icon_url(catalog, bosses, ds, category, &mut icon),
        })
        .collect();
    WikiIndex {
        info,
        pages,
        samples,
    }
}

/// A deliberate, checked choice of one game thing per landing tile — never "whatever page
/// happens to come first", which would draw a different tile picture depending on the
/// dataset's own iteration order. `None` for a kind the game draws no picture for at all
/// (transformations: B50; stages: no title-art reader exists yet; versions: a patch note has
/// no picture) — the tile falls back to its plain icon, same as without the game.
///
/// The two "raw" choices (`CardsAndRunes`, `Pickups`) are `IconRef::Entity`, not
/// `IconRef::Page`: individual card faces have no data-file mapping at all (measured — see
/// `target_sprite`'s `Target::Article` arm), so a card page can never draw a picture through
/// this pipeline. The tile still can, honestly: `5.300.1` is a real `entities2.xml` row, a
/// tarot card's own face-down back — "a card", not "the right card" — and `5.10.1` is a
/// heart pickup, the same distinction the task drew.
pub fn category_sample(category: WikiPageCategory) -> Option<IconRef> {
    let entity = |id, variant, subtype| {
        Some(IconRef::Page {
            target: Target::Entity {
                id,
                variant,
                subtype,
            },
        })
    };
    match category {
        // Sad Onion: item 105, in the game since Rebirth, always drawn.
        WikiPageCategory::Items => Some(IconRef::Page {
            target: Target::Item { id: 105 },
        }),
        // Swallowed Penny: the first trinket a new run can find.
        WikiPageCategory::Trinkets => Some(IconRef::Page {
            target: Target::Trinket { id: 1 },
        }),
        WikiPageCategory::Achievements => Some(IconRef::Page {
            target: Target::Achievement { id: 1 },
        }),
        // Isaac: id 0, the character the game itself opens on.
        WikiPageCategory::Characters => Some(IconRef::Page {
            target: Target::Character { id: 0 },
        }),
        WikiPageCategory::Challenges => Some(IconRef::Page {
            target: Target::Challenge { number: 1 },
        }),
        // Monstro: the first boss's entity key.
        WikiPageCategory::Bosses => entity(20, 0, 0),
        // Gaper: the game's own first common enemy.
        WikiPageCategory::Monsters => entity(10, 0, 0),
        // A tarot card's shared back — see this function's own doc comment.
        WikiPageCategory::CardsAndRunes => Some(IconRef::Entity {
            id: 5,
            variant: 300,
            subtype: 1,
        }),
        // A heart pickup.
        WikiPageCategory::Pickups => Some(IconRef::Entity {
            id: 5,
            variant: 10,
            subtype: 1,
        }),
        WikiPageCategory::Transformations
        | WikiPageCategory::Stages
        | WikiPageCategory::Versions => None,
    }
}

/// `category_sample`'s picture, only when the catalog really has one — never a URL to a
/// picture that would 404. `icon_source` is the one function that already knows how to check
/// every kind of `IconRef` this can produce, `Page` and `Entity` alike.
fn category_icon_url(
    catalog: Option<&Catalog>,
    bosses: &BossKeys,
    ds: &Dataset,
    category: WikiPageCategory,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> Option<String> {
    let c = catalog?;
    let r = category_sample(category)?;
    crate::icon::icon_source(c, bosses, Some(ds), &r)?;
    icon(&r)
}

/// Every page of the dataset with its identity, by kind and then by id: the one walk the
/// index and the search both read. Two walks is how B46 happened — the sixteen
/// transformations entered the dataset on 2026-09-13 and one of the two lists did not learn
/// about them, and a page that exists and cannot be found reads exactly like a page that
/// does not exist.
pub(crate) fn pages(ds: &Dataset) -> impl Iterator<Item = (Target, &Entry)> {
    let items = ds.items.iter().map(|(id, e)| (Target::Item { id: *id }, e));
    let trinkets = ds
        .trinkets
        .iter()
        .map(|(id, e)| (Target::Trinket { id: *id }, e));
    let achievements = ds
        .achievements
        .iter()
        .map(|(id, e)| (Target::Achievement { id: *id }, e));
    let bosses = ds
        .bosses
        .iter()
        .filter_map(|(key, e)| Some((boss_target(key)?, e)));
    let challenges = ds
        .challenges
        .iter()
        .map(|(n, e)| (Target::Challenge { number: *n }, e));
    let characters = ds
        .characters
        .iter()
        .map(|(id, e)| (Target::Character { id: *id }, e));
    let transformations = ds
        .transformations
        .iter()
        .map(|(id, e)| (Target::Transformation { id: *id }, e));
    // Bosses and common enemies are keyed the same way (`Dataset::boss_key`) and read back
    // into the same `Target::Entity` shape: one reader for both, `boss_target` despite the
    // name.
    let entities = ds
        .entities
        .iter()
        .filter_map(|(key, e)| Some((boss_target(key)?, e)));
    let articles = ds.articles.iter().map(|(title, e)| {
        (
            Target::Article {
                title: title.clone(),
            },
            e,
        )
    });
    items
        .chain(trinkets)
        .chain(achievements)
        .chain(bosses)
        .chain(challenges)
        .chain(characters)
        .chain(transformations)
        .chain(entities)
        .chain(articles)
}

/// The inverse of `Dataset::boss_key`: `"20.0.0"` → the entity. A key that isn't three
/// numbers is one the build never wrote, and the page is left out rather than guessed.
/// Shared by `ds.bosses` and `ds.entities` (design decision 2): both collections key their
/// entries the same way, by the game's bestiary triple.
fn boss_target(key: &str) -> Option<Target> {
    let (id, variant, subtype) = Dataset::parse_boss_key(key)?;
    Some(Target::Entity {
        id,
        variant,
        subtype,
    })
}

/// A game patch, as the wiki knows it.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct PatchView {
    pub number: String,
    pub date: String,
}

/// Per-type entry counts in the embedded dataset.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct WikiCounts {
    pub items: u32,
    pub trinkets: u32,
    pub achievements: u32,
    pub bosses: u32,
    pub challenges: u32,
    pub characters: u32,
    pub transformations: u32,
    /// The `entities` collection (design decision 2): common enemies and pickup entities,
    /// what the landing's "Monsters" tile counts.
    pub monsters: u32,
    /// Articles under `ArticleCategory::Card` or `::Rune` (design decision 5).
    pub cards_and_runes: u32,
    /// Articles under `ArticleCategory::Pickup`.
    pub pickups: u32,
    /// Articles under `ArticleCategory::Stage`.
    pub stages: u32,
    /// Articles under `ArticleCategory::Version`: the "Added in …" patch and version pages.
    pub versions: u32,
    /// The whole `articles` collection, category or none: what search counts against.
    pub articles: u32,
}

/// Why the embedded dataset failed to load. Fieldless: a bare string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum WikiMissingReason {
    SchemaMismatch,
    Malformed,
}

/// The state of the wiki dataset: what the verification screen shows, and the basis
/// for saying "is it worth updating?".
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum WikiInfo {
    Loaded {
        snapshot_at: String,
        last_known_patch: Option<PatchView>,
        counts: WikiCounts,
        unresolved: u32,
        unknown_templates: u32,
        /// `None` when the game's installation date isn't known: there's no way to
        /// tell whether the snapshot is behind.
        game_newer_than_snapshot: Option<bool>,
    },
    Missing {
        reason: WikiMissingReason,
    },
}

/// Translates the outcome of `Dataset::embedded()` into a view. The dataset never
/// crosses the IPC boundary in full: only the summary needed to tell if it's current.
pub fn wiki_info(
    dataset: Result<&Dataset, &DatasetError>,
    game_updated_unix: Option<u64>,
) -> WikiInfo {
    match dataset {
        Ok(ds) => {
            let meta = &ds.meta;
            WikiInfo::Loaded {
                snapshot_at: meta.snapshot_at.clone(),
                last_known_patch: meta.last_known_patch.as_ref().map(|p| PatchView {
                    number: p.number.clone(),
                    date: p.date.clone(),
                }),
                counts: WikiCounts {
                    items: meta.counts.items,
                    trinkets: meta.counts.trinkets,
                    achievements: meta.counts.achievements,
                    bosses: meta.counts.bosses,
                    challenges: meta.counts.challenges,
                    characters: meta.counts.characters,
                    transformations: meta.counts.transformations,
                    monsters: meta.counts.entities,
                    cards_and_runes: articles_by_category(
                        ds,
                        &[ArticleCategory::Card, ArticleCategory::Rune],
                    ),
                    pickups: articles_by_category(ds, &[ArticleCategory::Pickup]),
                    stages: articles_by_category(ds, &[ArticleCategory::Stage]),
                    versions: articles_by_category(ds, &[ArticleCategory::Version]),
                    articles: meta.counts.articles,
                },
                unresolved: meta.diagnostics.unresolved.values().sum(),
                unknown_templates: meta.diagnostics.unknown_templates.values().sum(),
                game_newer_than_snapshot: game_updated_unix
                    .zip(rfc3339_to_unix(&meta.snapshot_at))
                    .map(|(g, s)| g > s),
            }
        }
        Err(DatasetError::SchemaMismatch { .. }) => WikiInfo::Missing {
            reason: WikiMissingReason::SchemaMismatch,
        },
        Err(DatasetError::Malformed { .. }) => WikiInfo::Missing {
            reason: WikiMissingReason::Malformed,
        },
    }
}

/// How many articles carry one of `wanted`'s categories. An article with no category
/// (mechanics, concepts, machines, list pages) never counts here: it has no landing tile
/// (design decision 5).
fn articles_by_category(ds: &Dataset, wanted: &[ArticleCategory]) -> u32 {
    let n = ds
        .articles
        .values()
        .filter(|e| match &e.infobox {
            Infobox::Article {
                category: Some(category),
            } => wanted.contains(category),
            Infobox::Article { category: None } => false,
            Infobox::Item { .. }
            | Infobox::Trinket { .. }
            | Infobox::Achievement { .. }
            | Infobox::Boss { .. }
            | Infobox::Challenge { .. }
            | Infobox::Transformation { .. }
            | Infobox::Character { .. }
            | Infobox::Entity { .. } => false,
        })
        .count();
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// Seconds since the Unix epoch for a `YYYY-MM-DDTHH:MM:SSZ` timestamp. Only this exact
/// format: anything else (no `Z`, no time, a different length) yields `None`.
pub fn rfc3339_to_unix(s: &str) -> Option<u64> {
    let b = s.as_bytes();
    if b.len() != 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
        || b[19] != b'Z'
    {
        return None;
    }
    let field = |range: std::ops::Range<usize>| -> Option<i64> {
        let slice = s.get(range)?;
        if slice.is_empty() || !slice.bytes().all(|c| c.is_ascii_digit()) {
            return None;
        }
        slice.parse().ok()
    };
    let year = field(0..4)?;
    let month = field(5..7)?;
    let day = field(8..10)?;
    let hour = field(11..13)?;
    let minute = field(14..16)?;
    let second = field(17..19)?;
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute)
        || !(0..=59).contains(&second)
    {
        return None;
    }
    let days = days_from_civil(year, month, day);
    let secs = days * 86_400 + hour * 3_600 + minute * 60 + second;
    u64::try_from(secs).ok()
}

/// Howard Hinnant's "days from civil": days since 1970-01-01 for any civil date (even
/// before the epoch), with no calendar library and no hand-written leap-year tables.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = if m > 2 { m - 3 } else { m + 9 }; // [0, 11], March = 0
    let doy = (153 * mp + 2) / 5 + d - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Strict: exactly the three numbers `Dataset::boss_key` writes, and nothing else.
    #[test]
    fn a_boss_page_key_is_three_numbers_and_nothing_else() {
        assert_eq!(
            boss_target("20.0.0"),
            Some(Target::Entity {
                id: 20,
                variant: 0,
                subtype: 0
            })
        );
        assert_eq!(
            boss_target("19.2.1"),
            Some(Target::Entity {
                id: 19,
                variant: 2,
                subtype: 1
            })
        );
        for refused in [
            "20.0", "20.0.0.0", "20.0.x", "x.0.0", "20..0", "", "20.0.0.",
        ] {
            assert_eq!(boss_target(refused), None, "{refused:?}");
        }
    }

    #[test]
    fn rfc3339() {
        assert_eq!(rfc3339_to_unix("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(rfc3339_to_unix("2026-09-05T14:20:41Z"), Some(1_788_618_041));
        assert_eq!(rfc3339_to_unix("2026-09-05"), None);
    }

    #[test]
    fn info_shapes() {
        let missing = wiki_info(Err(&DatasetError::Malformed { reason: "x".into() }), None);
        assert_eq!(
            serde_json::to_value(&missing).unwrap(),
            serde_json::json!({"kind":"missing","reason":"malformed"})
        );
        let ds = Dataset::embedded().expect("embedded dataset");
        let loaded = wiki_info(Ok(ds), Some(0));
        let v = serde_json::to_value(&loaded).unwrap();
        assert_eq!(v["kind"], "loaded");
        assert_eq!(v["gameNewerThanSnapshot"], false);
        assert_eq!(v["counts"]["trinkets"], 188);
        let v = serde_json::to_value(wiki_info(Ok(ds), None)).unwrap();
        assert_eq!(v["gameNewerThanSnapshot"], serde_json::Value::Null);
        let v = serde_json::to_value(wiki_info(Ok(ds), Some(u64::MAX / 2))).unwrap();
        assert_eq!(v["gameNewerThanSnapshot"], true);
    }

    #[test]
    fn schema_mismatch_maps_to_its_own_reason() {
        let missing = wiki_info(
            Err(&DatasetError::SchemaMismatch {
                found: 2,
                expected: 1,
            }),
            None,
        );
        assert_eq!(
            serde_json::to_value(&missing).unwrap(),
            serde_json::json!({"kind":"missing","reason":"schemaMismatch"})
        );
    }
}
