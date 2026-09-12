//! Search: one index over everything the app knows by name or by text (spec 3.5, Decisions
//! 5 and 6). Pure, like the rest of `ipc`.
//!
//! Two sources. The **wiki side** is built once — the dataset never changes — and each page
//! becomes its title plus one flattened string per section. The **catalog side** is read at
//! every query: 2,000 names cost nothing, and "the game isn't installed" is never cached.

use std::collections::BTreeMap;

use wiki::{Block, Dataset, DatasetError, Entry, Inline, SectionKind, Target};

use crate::wiki::boss_target;

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
        SearchIndex {
            loaded: true,
            pages,
        }
    }

    /// Whether the dataset loaded at all: `false` is the `noWiki` diagnostic.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn len(&self) -> usize {
        self.pages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    pub fn title(&self, target: &Target) -> Option<&str> {
        self.pages.get(target).map(|d| d.title.as_str())
    }

    /// The flattened text of one page's section: the measurable half of the index, so a test
    /// can state what a page reads as without going through a query.
    pub fn section_text(&self, target: &Target, kind: SectionKind) -> Option<&str> {
        self.pages
            .get(target)?
            .sections
            .iter()
            .find(|(k, _)| *k == kind)
            .map(|(_, text)| text.as_str())
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
