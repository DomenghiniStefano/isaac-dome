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
