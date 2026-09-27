//! Recognizing a `<tabber>` tab's own opening line. Split out of `blocks.rs` for size, not for
//! any dependency of its own: `tabber_label` is a pure string check, called from
//! `Parser::line` (which stays in `blocks.rs` — the dispatch itself is state-machine code, not
//! this file's business) the same way `heading`/`list_item` are.

/// A `<tabber>` tab's own opening line: `<tabber>Label=` for the first tab, `|-|Label=` for
/// every one after it (Equality!'s "Before Repentance+" and "In Repentance+"). This does not
/// model the tab strip itself — which section belongs to which tab is decided by `sections.rs`
/// before this parser ever sees the text, and a heading is not a tab — but the label is real
/// content: it reads better as a heading of its own than as the literal `|-|…=` it would
/// otherwise fall through to as prose.
pub(super) fn tabber_label(trimmed: &str) -> Option<&str> {
    let label = trimmed
        .strip_prefix("<tabber>")
        .or_else(|| trimmed.strip_prefix("|-|"))?;
    let label = label.strip_suffix('=')?;
    (!label.is_empty()).then_some(label)
}

#[cfg(test)]
mod tests {
    use crate::resolver::fixtures::test_resolver;
    use crate::{parse_blocks, Block, Diagnostics, Inline, Style};

    fn p(s: &str) -> Vec<Block> {
        parse_blocks(s, &test_resolver(), &mut Diagnostics::default())
    }
    fn t(s: &str) -> Vec<Inline> {
        vec![Inline::Text {
            text: s.into(),
            style: Style::Plain,
        }]
    }

    /// `<tabber>` splits `|-|Label=` tabs into a heading per label instead of the literal
    /// syntax. This is a block-level mechanism only: which lines of an already-split section
    /// belong to which tab is not something `blocks` can know, and Equality!'s own tabber
    /// spans several `== Level 2 ==` headings that `sections.rs` splits before this text ever
    /// reaches `parse_blocks` — this test is the shape the mechanism produces on a section's
    /// body, not a claim that today's Equality! page reassembles correctly end to end.
    #[test]
    fn a_tabber_tab_label_becomes_a_heading_and_the_closing_tag_is_dropped() {
        let blocks = p(
            "<tabber>Before Repentance+=\nSome text\n|-|In Repentance+=\nMore text\n</tabber>\n",
        );
        assert_eq!(
            blocks,
            vec![
                Block::Heading {
                    level: 3,
                    inline: t("Before Repentance+")
                },
                Block::Paragraph {
                    inline: t("Some text")
                },
                Block::Heading {
                    level: 3,
                    inline: t("In Repentance+")
                },
                Block::Paragraph {
                    inline: t("More text")
                },
            ]
        );
    }
}
