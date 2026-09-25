//! Search: one index over everything the app knows by name or by text (spec 3.5, Decisions
//! 5 and 6). Pure, like the rest of `ipc`.
//!
//! Two sources. The **wiki side** is built once — the dataset never changes — and each page
//! becomes its title plus one flattened string per section. The **catalog side** is read at
//! every query: 2,000 names cost nothing, and "the game isn't installed" is never cached.

mod rank;
mod text;

use std::collections::BTreeMap;

use catalog::{Catalog, Language};
use serde::Serialize;
use wiki::{Dataset, DatasetError, SectionKind, Target};

use crate::flags::recorded_done;
use crate::icon::IconRef;
use crate::target_sprite::{target_sprite, BossKeys, TargetSprite};
use crate::wiki_target;
use rank::Ranked;
use text::{fold, words, WikiDoc};

/// The wiki side of the index, built once from the embedded dataset.
pub struct SearchIndex {
    loaded: bool,
    pages: BTreeMap<Target, WikiDoc>,
}

impl SearchIndex {
    pub fn build(dataset: Result<&Dataset, &DatasetError>) -> SearchIndex {
        let Ok(ds) = dataset else {
            return SearchIndex {
                loaded: false,
                pages: BTreeMap::new(),
            };
        };
        // Every page the index lists, from the same walk (`crate::wiki::pages`): B46 was the
        // transformations missing from one of two hand-written lists.
        let pages = crate::wiki::pages(ds)
            .map(|(target, entry)| (target, text::doc(entry)))
            .collect();
        SearchIndex {
            loaded: true,
            pages,
        }
    }

    /// Whether the dataset loaded at all: `false` is the `noWiki` diagnostic.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    #[cfg(feature = "test-api")]
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    #[cfg(feature = "test-api")]
    pub fn title(&self, target: &Target) -> Option<&str> {
        self.pages.get(target).map(|d| d.title.as_str())
    }

    /// The flattened text of one page's section: the measurable half of the index, so a test
    /// can state what a page reads as without going through a query.
    #[cfg(feature = "test-api")]
    pub fn section_text(&self, target: &Target, kind: SectionKind) -> Option<&str> {
        self.pages
            .get(target)?
            .sections
            .iter()
            .find(|(k, _)| *k == kind)
            .map(|(_, text)| text.as_str())
    }
}

/// The two flag sections search reads. `None` for a section is "it didn't read", which is not
/// "nothing is done": the mark says `unknown` and a diagnostic says which section.
#[derive(Debug, Clone, Copy)]
pub struct SaveFlags<'a> {
    pub achievements: Option<&'a [bool]>,
    pub items: Option<&'a [bool]>,
}

/// Where a target stands in the profile. Fieldless: a bare string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum ProgressMark {
    Done,
    Pending,
    Unknown,
    None,
}

/// One searchable thing, with whatever the two sides know about it.
#[derive(Debug, Clone)]
pub struct Doc {
    pub title: String,
    /// The other name, when the two sides disagree: the wiki's title beside the game's.
    pub alias: Option<String>,
    /// An achievement's own wording of what to do.
    pub condition: Option<String>,
    pub has_page: bool,
}

/// Every document, keyed by target so the order is the kind's and then the id's, and a target
/// the two sides share is one row. The catalog's names are joined onto the wiki's pages one by
/// one, in the catalog's order.
pub(crate) fn documents(
    index: &SearchIndex,
    catalog: Option<&Catalog>,
    bosses: &BossKeys,
) -> BTreeMap<Target, Doc> {
    let pages: BTreeMap<Target, Doc> = index
        .pages
        .iter()
        .map(|(target, page)| {
            (
                target.clone(),
                Doc {
                    title: page.title.clone(),
                    alias: None,
                    condition: None,
                    has_page: true,
                },
            )
        })
        .collect();
    let Some(c) = catalog else {
        return pages;
    };
    catalog_names(c, bosses).fold(pages, |mut docs, (target, title, condition)| {
        let joined = join(docs.remove(&target), title, condition);
        docs.insert(target, joined);
        docs
    })
}

/// Every name the catalog gives a target, with an achievement's condition beside it.
fn catalog_names<'a>(
    c: &'a Catalog,
    bosses: &'a BossKeys,
) -> impl Iterator<Item = (Target, String, Option<String>)> + 'a {
    let en = Language::English;
    let items = c
        .items()
        .map(move |i| (wiki_target::item(i), c.text(&i.name, en).to_string(), None));
    let characters = c.characters().map(move |p| {
        (
            wiki_target::character(p),
            c.text(&p.name, en).to_string(),
            None,
        )
    });
    // The key is whatever `boss_keys` settles for the row — its page's, or the one its
    // portrait's file name declares. A row left without one names no target and is out.
    let bosses = c
        .bosses()
        .filter_map(move |b| Some((wiki_target::boss(bosses, b)?, b.name.clone(), None)));
    let challenges = c
        .challenges()
        .map(|ch| (wiki_target::challenge(ch), ch.name.clone(), None));
    let achievements = c.achievements().map(|a| {
        (
            wiki_target::achievement(a.id),
            a.text.clone(),
            a.unlock_condition.clone(),
        )
    });
    items
        .chain(characters)
        .chain(bosses)
        .chain(challenges)
        .chain(achievements)
}

/// One catalog name joined onto what the target already has. The catalog's name wins: it is
/// what the game itself calls the thing. The wiki title stays as an alias when it says
/// something else. An empty name is the game calling it nothing — Dead God's `text` is blank —
/// and it does not win.
fn join(existing: Option<Doc>, title: String, condition: Option<String>) -> Doc {
    match existing {
        Some(doc) if doc.title != title && !title.trim().is_empty() => Doc {
            alias: Some(doc.title),
            title,
            condition,
            has_page: doc.has_page,
        },
        Some(doc) => Doc { condition, ..doc },
        None => Doc {
            title,
            alias: None,
            condition,
            has_page: false,
        },
    }
}

/// Where the target stands in the profile: section 1 for an achievement, section 4 for a
/// collectible, nothing for anything else. A slot past a section's end reads as not done —
/// the save has no record of it — while a section that didn't read is `Unknown`.
pub(crate) fn progress(target: &Target, flags: Option<SaveFlags<'_>>) -> ProgressMark {
    let mark = |slots: Option<&[bool]>, id: u32| match slots {
        None => ProgressMark::Unknown,
        Some(f) => {
            if recorded_done(f, id) {
                ProgressMark::Done
            } else {
                ProgressMark::Pending
            }
        }
    };
    match target {
        Target::Achievement { id } => match flags {
            None => ProgressMark::Unknown,
            Some(f) => mark(f.achievements, *id),
        },
        Target::Item { id } => match flags {
            None => ProgressMark::Unknown,
            Some(f) => mark(f.items, *id),
        },
        // A trinket has no slot in section 4, and a boss, a character or a challenge has no
        // slot to read at all: no mark is not an unknown one.
        Target::Trinket { .. }
        | Target::Character { .. }
        | Target::Challenge { .. }
        | Target::Entity { .. }
        | Target::Transformation { .. }
        | Target::Stage { .. }
        | Target::Room { .. }
        | Target::Concept { .. } => ProgressMark::None,
    }
}

/// A ranked answer. `total` is how many documents matched before the limit: the screen says
/// "300 of N" from it, and the palette's last row counts with it.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct SearchView {
    pub query: String,
    pub hits: Vec<SearchHit>,
    pub total: u32,
    pub diagnostics: Vec<SearchDiagnostic>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub target: Target,
    pub title: String,
    pub icon_url: Option<String>,
    /// The dataset has this page: a Wiki destination exists for the hit.
    pub has_page: bool,
    #[serde(rename = "match")]
    pub matched: SearchMatch,
    pub progress: ProgressMark,
}

/// Why the hit matched, in the words the row shows. Tagged: two of the three carry data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SearchMatch {
    Title,
    /// The achievement's own wording of what to do.
    Condition {
        text: String,
    },
    Section {
        section: SectionKind,
        before: String,
        matched: String,
        after: String,
    },
}

/// What the answer couldn't take into account. Fieldless: a bare string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum SearchDiagnostic {
    /// No profile chosen: every mark is unknown. Expected before a save is picked, not a
    /// failure.
    NoProfile,
    NoCatalog,
    NoWiki,
    NoAchievementSection,
    NoCollectionSection,
}

/// The ranked answer to one query. Pure: the caller supplies the index, the catalog, the
/// profile's two sections and the icon link.
pub fn search(
    index: &SearchIndex,
    catalog: Option<&Catalog>,
    bosses: &BossKeys,
    flags: Option<SaveFlags<'_>>,
    query: &str,
    limit: usize,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> SearchView {
    let words = words(query);
    if words.is_empty() {
        return SearchView {
            query: query.to_string(),
            hits: Vec::new(),
            total: 0,
            diagnostics: Vec::new(),
        };
    }
    let folded_query = fold(query.trim());
    let mut ranked: Vec<Ranked> = documents(index, catalog, bosses)
        .into_iter()
        .filter_map(|(target, doc)| {
            let progress = progress(&target, flags);
            Ranked::new(index, target, doc, &words, &folded_query, progress)
        })
        .collect();
    ranked.sort_by(|a, b| a.key().cmp(&b.key()));
    let total = ranked.len() as u32;
    let hits = ranked
        .into_iter()
        .take(limit)
        .map(|r| hit(r, catalog, bosses, &mut icon))
        .collect();
    SearchView {
        query: query.to_string(),
        hits,
        total,
        diagnostics: diagnostics(index, catalog, flags),
    }
}

/// A ranked document as the row shows it. The icon is asked for only when the catalog has art
/// for the target: a link nothing can serve draws a broken image.
fn hit(
    r: Ranked,
    catalog: Option<&Catalog>,
    bosses: &BossKeys,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> SearchHit {
    let icon_url = catalog.and_then(|c| match target_sprite(c, bosses, &r.target) {
        TargetSprite::Found(_) => icon(&IconRef::Page {
            target: r.target.clone(),
        }),
        TargetSprite::NoArt | TargetSprite::Unknown => None,
    });
    SearchHit {
        icon_url,
        has_page: r.doc.has_page,
        title: r.doc.title,
        target: r.target,
        matched: r.matched,
        progress: r.progress,
    }
}

fn diagnostics(
    index: &SearchIndex,
    catalog: Option<&Catalog>,
    flags: Option<SaveFlags<'_>>,
) -> Vec<SearchDiagnostic> {
    [
        catalog.is_none().then_some(SearchDiagnostic::NoCatalog),
        (!index.is_loaded()).then_some(SearchDiagnostic::NoWiki),
    ]
    .into_iter()
    .flatten()
    .chain(profile_diagnostics(flags))
    .collect()
}

/// One word, not two, without a profile: there is no section to miss.
fn profile_diagnostics(flags: Option<SaveFlags<'_>>) -> Vec<SearchDiagnostic> {
    match flags {
        None => vec![SearchDiagnostic::NoProfile],
        Some(f) => [
            f.achievements
                .is_none()
                .then_some(SearchDiagnostic::NoAchievementSection),
            f.items
                .is_none()
                .then_some(SearchDiagnostic::NoCollectionSection),
        ]
        .into_iter()
        .flatten()
        .collect(),
    }
}
