//! A read-only walk of every inline leaf an [`Entry`] carries. The mutating twin,
//! `page::narrow_to_page`'s pass over the same shape, stays there — it changes the tree, this
//! only looks at it. Two callers need exactly that look (`dead_links`, tallying what cannot
//! open, and the real-dataset guard that no `Text` leaves the parser carrying raw
//! `{{`/`}}`), so it is written once here instead of twice.

use crate::model::{Block, Entry, Inline};

impl Entry {
    /// Every inline leaf the entry carries — a `Text`, a `Ref` or a `Concept` — from the
    /// infobox's own fields, the description, and every section's blocks. An
    /// `Inline::Edition` wrapper is unwrapped rather than returned: a caller that only wants
    /// what a reader reads never has to recurse into one itself.
    pub fn inlines(&self) -> Vec<&Inline> {
        let mut out = Vec::new();
        for field in self.infobox.inlines() {
            leaves_of(field, &mut out);
        }
        leaves_of(&self.description, &mut out);
        for section in &self.sections {
            for block in &section.blocks {
                block_leaves_of(block, &mut out);
            }
        }
        out
    }
}

/// One inline run's leaves, recursing into an edition wrapper's words.
fn leaves_of<'a>(inline: &'a [Inline], out: &mut Vec<&'a Inline>) {
    for node in inline {
        match node {
            Inline::Edition { only: _, inline } => leaves_of(inline, out),
            leaf @ (Inline::Text { .. } | Inline::Ref { .. } | Inline::Concept { .. }) => {
                out.push(leaf);
            }
        }
    }
}

/// One block's leaves, recursing into a list's children and a table's cells.
fn block_leaves_of<'a>(block: &'a Block, out: &mut Vec<&'a Inline>) {
    match block {
        Block::Paragraph { inline } | Block::Heading { inline, level: _ } => {
            leaves_of(inline, out);
        }
        Block::List { ordered: _, items } => {
            for item in items {
                leaves_of(&item.inline, out);
                for child in &item.children {
                    block_leaves_of(child, out);
                }
            }
        }
        Block::Table { header, rows } => {
            for cell in header {
                leaves_of(cell, out);
            }
            for row in rows {
                for cell in row {
                    leaves_of(cell, out);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        CollectibleTemplate, Dlc, Infobox, ListItem, Section, SectionKind, Style, Target,
    };

    fn text(t: &str) -> Inline {
        Inline::Text {
            text: t.into(),
            style: Style::Plain,
        }
    }

    fn a_ref(id: u32, label: &str) -> Inline {
        Inline::Ref {
            target: Target::Item { id },
            label: label.into(),
        }
    }

    fn entry(description: Vec<Inline>, sections: Vec<Section>) -> Entry {
        Entry {
            title: "X".into(),
            revid: 1,
            description,
            dlc: vec![],
            unlocked_by: None,
            infobox: Infobox::Item {
                quote: vec![text("quote")],
                template: CollectibleTemplate::Passive,
                quality: None,
                tags: vec![],
                recharge: vec![],
                devil_price: vec![],
                shop_price: vec![],
                pools: vec![],
            },
            sections,
        }
    }

    /// The infobox's own fields and the description both contribute leaves: naming an
    /// entry's fields by hand instead of going through `Infobox::inlines` is exactly how a
    /// pass over its text can skip some of them.
    #[test]
    fn infobox_fields_and_the_description_are_both_walked() {
        let e = entry(vec![text("desc")], vec![]);
        let leaves = e.inlines();
        assert!(leaves.contains(&&text("quote")), "{leaves:?}");
        assert!(leaves.contains(&&text("desc")), "{leaves:?}");
    }

    /// An `Edition` wrapper is unwrapped: its own words come out as ordinary leaves, and the
    /// wrapper node itself never appears in the result.
    #[test]
    fn an_edition_wrapper_is_unwrapped_to_its_words() {
        let e = entry(
            vec![Inline::Edition {
                only: vec![Dlc::Repentance],
                inline: vec![a_ref(25, "Breakfast")],
            }],
            vec![],
        );
        let leaves = e.inlines();
        assert!(leaves.contains(&&a_ref(25, "Breakfast")), "{leaves:?}");
        assert!(
            !leaves.iter().any(|i| matches!(i, Inline::Edition { .. })),
            "{leaves:?}"
        );
    }

    /// A list's nested children and a table's header and cells all contribute leaves, not
    /// only a paragraph or a heading.
    #[test]
    fn sections_are_walked_through_lists_and_tables() {
        let e = entry(
            vec![],
            vec![Section {
                kind: SectionKind::Effects,
                title: vec![],
                blocks: vec![
                    Block::List {
                        ordered: false,
                        items: vec![ListItem {
                            inline: vec![text("item")],
                            children: vec![Block::Paragraph {
                                inline: vec![text("child")],
                            }],
                        }],
                    },
                    Block::Table {
                        header: vec![vec![text("head")]],
                        rows: vec![vec![vec![text("cell")]]],
                    },
                ],
            }],
        );
        let leaves = e.inlines();
        for expected in ["item", "child", "head", "cell"] {
            assert!(
                leaves.contains(&&text(expected)),
                "missing {expected:?}: {leaves:?}"
            );
        }
    }
}
