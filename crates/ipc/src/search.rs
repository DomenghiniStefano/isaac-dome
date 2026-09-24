//! Search: one index over everything the app knows by name or by text (spec 3.5, Decisions
//! 5 and 6). Pure, like the rest of `ipc`.
//!
//! Two sources. The **wiki side** is built once — the dataset never changes — and each page
//! becomes its title plus one flattened string per section. The **catalog side** is read at
//! every query: 2,000 names cost nothing, and "the game isn't installed" is never cached.

use std::collections::BTreeMap;

use catalog::{Catalog, Language};
use serde::Serialize;
use wiki::{Block, Dataset, DatasetError, Entry, Inline, SectionKind, Target};

use crate::icon::IconRef;
use crate::target_sprite::{target_sprite, TargetSprite};
use crate::wiki::boss_target;
use crate::wiki_target;

/// One page as the search reads it: the title, and the text of each section in the order the
/// page has them.
struct WikiDoc {
    title: String,
    sections: Vec<(SectionKind, String)>,
}

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
        let mut pages = BTreeMap::new();
        let mut add = |target: Target, entry: &Entry| {
            pages.insert(target, doc(entry));
        };
        for (id, e) in &ds.items {
            add(Target::Item { id: *id }, e);
        }
        for (id, e) in &ds.trinkets {
            add(Target::Trinket { id: *id }, e);
        }
        for (id, e) in &ds.achievements {
            add(Target::Achievement { id: *id }, e);
        }
        for (key, e) in &ds.bosses {
            if let Some(t) = boss_target(key) {
                add(t, e);
            }
        }
        for (n, e) in &ds.challenges {
            add(Target::Challenge { number: *n }, e);
        }
        for (id, e) in &ds.characters {
            add(Target::Character { id: *id }, e);
        }
        // B46: the sixteen transformations are pages like the others. Indexed here as well
        // as in `wiki_index`, because a page that exists and cannot be found reads exactly
        // like a page that does not exist — which is how this was found, by typing a name
        // into the app and getting nothing.
        for (id, e) in &ds.transformations {
            add(Target::Transformation { id: *id }, e);
        }
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
/// the two sides share is one row.
pub(crate) fn documents(index: &SearchIndex, catalog: Option<&Catalog>) -> BTreeMap<Target, Doc> {
    let mut docs: BTreeMap<Target, Doc> = index
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
        return docs;
    };
    let en = Language::English;
    let mut join = |target: Target, title: String, condition: Option<String>| {
        match docs.get_mut(&target) {
            // The catalog's name wins: it is what the game itself calls the thing. The wiki
            // title stays as an alias when it says something else. An empty name is the game
            // calling it nothing — Dead God's `text` is blank — and it does not win.
            Some(doc) => {
                if doc.title != title && !title.trim().is_empty() {
                    doc.alias = Some(std::mem::replace(&mut doc.title, title));
                }
                doc.condition = condition;
            }
            None => {
                docs.insert(
                    target,
                    Doc {
                        title,
                        alias: None,
                        condition,
                        has_page: false,
                    },
                );
            }
        }
    };
    for i in c.items() {
        join(wiki_target::item(i), c.text(&i.name, en).to_string(), None);
    }
    for p in c.characters() {
        join(
            wiki_target::character(p),
            c.text(&p.name, en).to_string(),
            None,
        );
    }
    for b in c.bosses() {
        // The key is whatever `boss_keys` settles for the row — its page's, or the one its
        // portrait's file name declares. A row left without one names no target and is out.
        if let Some(target) = wiki_target::boss(c, b) {
            join(target, b.name.clone(), None);
        }
    }
    for ch in c.challenges() {
        join(wiki_target::challenge(ch), ch.name.clone(), None);
    }
    for a in c.achievements() {
        join(
            wiki_target::achievement(a.id),
            a.text.clone(),
            a.unlock_condition.clone(),
        );
    }
    docs
}

/// Where the target stands in the profile: section 1 for an achievement, section 4 for a
/// collectible, nothing for anything else. A slot past a section's end reads as not done —
/// the save has no record of it — while a section that didn't read is `Unknown`.
pub(crate) fn progress(target: &Target, flags: Option<SaveFlags<'_>>) -> ProgressMark {
    let mark = |slots: Option<&[bool]>, id: u32| match slots {
        None => ProgressMark::Unknown,
        Some(f) => {
            if f.get(id as usize).copied().unwrap_or(false) {
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

/// The characters kept before and after a match in a section fragment: enough to read the
/// sentence, counted in **characters**, so a cut never lands inside one.
const BEFORE: usize = 60;
const AFTER: usize = 90;

/// ASCII fold. Every ASCII byte maps to one byte and everything else is left alone, so an
/// offset in the folded string is an offset in the original: the fragment cuts the page's own
/// text without a second search.
fn fold(s: &str) -> String {
    s.to_ascii_lowercase()
}

fn words(query: &str) -> Vec<String> {
    fold(query).split_whitespace().map(str::to_string).collect()
}

fn contains_all(folded: &str, words: &[String]) -> bool {
    !words.is_empty() && words.iter().all(|w| folded.contains(w.as_str()))
}

/// The earliest occurrence among the words: where the fragment is cut, and how long the
/// matched text is.
fn first_at(folded: &str, words: &[String]) -> Option<(usize, usize)> {
    words
        .iter()
        .filter_map(|w| folded.find(w.as_str()).map(|at| (at, w.len())))
        .min_by_key(|(at, _)| *at)
}

fn fragment(text: &str, words: &[String]) -> Option<(String, String, String)> {
    let folded = fold(text);
    if !contains_all(&folded, words) {
        return None;
    }
    let (at, len) = first_at(&folded, words)?;
    let head = text.get(..at)?;
    let start = head
        .char_indices()
        .rev()
        .nth(BEFORE - 1)
        .map_or(0, |(i, _)| i);
    let matched = text.get(at..at + len)?;
    let tail = text.get(at + len..)?;
    let end = tail
        .char_indices()
        .nth(AFTER)
        .map_or(tail.len(), |(i, _)| i);
    Some((
        head[start..].to_string(),
        matched.to_string(),
        tail[..end].to_string(),
    ))
}

fn doc(entry: &Entry) -> WikiDoc {
    WikiDoc {
        title: entry.title.clone(),
        sections: entry
            .sections
            .iter()
            .map(|s| {
                let mut text = String::new();
                for block in &s.blocks {
                    flatten_block(block, &mut text);
                }
                (s.kind, text)
            })
            .collect(),
    }
}

fn push(out: &mut String, piece: &str) {
    let piece = piece.trim();
    if piece.is_empty() {
        return;
    }
    if !out.is_empty() {
        out.push(' ');
    }
    out.push_str(piece);
}

fn flatten_inline(inline: &[Inline], out: &mut String) {
    for i in inline {
        match i {
            Inline::Text { text, .. } => push(out, text),
            Inline::Ref { label, .. } | Inline::Concept { label, .. } => push(out, label),
            // The words are on the page: an edition inline is unwrapped, not skipped.
            Inline::Edition { inline, .. } => flatten_inline(inline, out),
        }
    }
}

fn flatten_block(block: &Block, out: &mut String) {
    match block {
        Block::Paragraph { inline } | Block::Heading { inline, .. } => flatten_inline(inline, out),
        Block::List { items, .. } => {
            for item in items {
                flatten_inline(&item.inline, out);
                for child in &item.children {
                    flatten_block(child, out);
                }
            }
        }
        Block::Table { header, rows } => {
            for cell in header {
                flatten_inline(cell, out);
            }
            for row in rows {
                for cell in row {
                    flatten_inline(cell, out);
                }
            }
        }
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

/// Tier 0 to 3, on a name: equal, prefix, a word's prefix, or merely containing the words.
fn title_tier(folded_title: &str, folded_query: &str) -> u8 {
    if folded_title == folded_query {
        0
    } else if folded_title.starts_with(folded_query) {
        1
    } else if folded_title
        .split_whitespace()
        .any(|w| w.starts_with(folded_query))
    {
        2
    } else {
        3
    }
}

/// Not done before done: that is what makes the profile part of the ranking (B5).
fn progress_rank(p: ProgressMark) -> u8 {
    match p {
        ProgressMark::Pending => 0,
        ProgressMark::Unknown => 1,
        ProgressMark::None => 2,
        ProgressMark::Done => 3,
    }
}

/// The first field of this document that holds every word, and the tier it earns. Fields in
/// order: title, alias, condition, then each section — so one target is one hit, named by the
/// strongest reason it matched.
fn best_field(
    index: &SearchIndex,
    target: &Target,
    doc: &Doc,
    words: &[String],
    folded_query: &str,
) -> Option<(u8, SearchMatch)> {
    let title = fold(&doc.title);
    if contains_all(&title, words) {
        return Some((title_tier(&title, folded_query), SearchMatch::Title));
    }
    if let Some(alias) = &doc.alias {
        let alias = fold(alias);
        if contains_all(&alias, words) {
            return Some((title_tier(&alias, folded_query), SearchMatch::Title));
        }
    }
    if let Some(condition) = &doc.condition {
        if contains_all(&fold(condition), words) {
            return Some((
                4,
                SearchMatch::Condition {
                    text: condition.clone(),
                },
            ));
        }
    }
    for (kind, text) in index.pages.get(target).map(|p| &p.sections)? {
        if let Some((before, matched, after)) = fragment(text, words) {
            return Some((
                5,
                SearchMatch::Section {
                    section: *kind,
                    before,
                    matched,
                    after,
                },
            ));
        }
    }
    None
}

/// The ranked answer to one query. Pure: the caller supplies the index, the catalog, the
/// profile's two sections and the icon link.
pub fn search(
    index: &SearchIndex,
    catalog: Option<&Catalog>,
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
    let docs = documents(index, catalog);
    let mut ranked: Vec<(u8, u8, String, Target, Doc, SearchMatch, ProgressMark)> = docs
        .into_iter()
        .filter_map(|(target, doc)| {
            let (tier, matched) = best_field(index, &target, &doc, &words, &folded_query)?;
            let progress = progress(&target, flags);
            Some((
                tier,
                progress_rank(progress),
                fold(&doc.title),
                target,
                doc,
                matched,
                progress,
            ))
        })
        .collect();
    // Tier, then the profile, then the name, and last the target itself: the order is total,
    // so a test can pin it and two runs can never disagree.
    ranked.sort_by(|a, b| (a.0, a.1, &a.2, &a.3).cmp(&(b.0, b.1, &b.2, &b.3)));
    let total = ranked.len() as u32;
    ranked.truncate(limit);
    let hits = ranked
        .into_iter()
        .map(|(_, _, _, target, doc, matched, progress)| SearchHit {
            icon_url: catalog.and_then(|c| match target_sprite(c, &target) {
                TargetSprite::Found(_) => icon(&IconRef::Page {
                    target: target.clone(),
                }),
                TargetSprite::NoArt | TargetSprite::Unknown => None,
            }),
            has_page: doc.has_page,
            title: doc.title,
            target,
            matched,
            progress,
        })
        .collect();
    SearchView {
        query: query.to_string(),
        hits,
        total,
        diagnostics: diagnostics(index, catalog, flags),
    }
}

fn diagnostics(
    index: &SearchIndex,
    catalog: Option<&Catalog>,
    flags: Option<SaveFlags<'_>>,
) -> Vec<SearchDiagnostic> {
    let mut out = Vec::new();
    if catalog.is_none() {
        out.push(SearchDiagnostic::NoCatalog);
    }
    if !index.is_loaded() {
        out.push(SearchDiagnostic::NoWiki);
    }
    match flags {
        // One word, not two: without a profile there is no section to miss.
        None => out.push(SearchDiagnostic::NoProfile),
        Some(f) => {
            if f.achievements.is_none() {
                out.push(SearchDiagnostic::NoAchievementSection);
            }
            if f.items.is_none() {
                out.push(SearchDiagnostic::NoCollectionSection);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folding_keeps_byte_offsets_aligned() {
        // ASCII fold only: a non-ASCII letter matches itself, and every byte keeps its
        // place, so an offset found in the folded string cuts the original correctly.
        let source = "Brimstone è Rosso";
        let folded = fold(source);
        assert_eq!(folded, "brimstone è rosso");
        assert_eq!(folded.len(), source.len());
    }

    #[test]
    fn every_word_must_be_in_the_same_field() {
        let w = words("  blood  BOMB ");
        assert_eq!(w, vec!["blood".to_string(), "bomb".to_string()]);
        assert!(contains_all("a bomb full of blood", &w));
        assert!(!contains_all("a bomb", &w));
        assert!(words("   ").is_empty());
        // An empty query matches nothing at all, rather than everything.
        assert!(!contains_all("anything", &words("")));
    }

    #[test]
    fn a_fragment_is_cut_around_the_earliest_word_on_character_boundaries() {
        let text = "The D6 rerolls the items in the Treasure Room, and Blood Bombs stay";
        let (before, matched, after) =
            fragment(text, &words("bombs treasure")).expect("the text holds both words");
        // The earliest of the two words names the place, whatever order they were typed in.
        assert_eq!(matched, "Treasure");
        assert!(text.starts_with(&before), "before: {before:?}");
        assert!(text.ends_with(&after), "after: {after:?}");
        assert!(before.chars().count() <= 60);
        assert!(after.chars().count() <= 90);
        assert_eq!(fragment(text, &words("nothing")), None);
    }

    #[test]
    fn a_fragment_never_splits_a_character() {
        // Eighty accented characters, two bytes each: a cut counted in bytes would land
        // inside one and panic.
        let text = format!("{} bomba", "è".repeat(80));
        let (before, matched, _) = fragment(&text, &words("bomba")).expect("matches");
        assert_eq!(matched, "bomba");
        assert!(before.chars().count() <= 60);
    }
}
