//! Release notes: the markdown a release carries, read into the blocks a window draws.
//!
//! The notes come out of `latest.json`, which travels over TLS but is **not signed**, so the
//! text is whatever the server answered. It therefore never reaches the webview as markup:
//! it is read here into the same typed blocks the wiki uses, and what the reader does not
//! recognise stays text rather than disappearing.
//!
//! The subset is the one `docs/release.md`'s notes are written in: headings, paragraphs,
//! bullet and numbered lists, `**bold**`, `` `code` `` and a `---` rule.

use ipc::release_notes;
use wiki::{Block, Inline, ListItem, Style};

fn plain(text: &str) -> Inline {
    Inline::Text {
        text: text.into(),
        style: Style::Plain,
    }
}

fn bold(text: &str) -> Inline {
    Inline::Text {
        text: text.into(),
        style: Style::Bold,
    }
}

fn paragraph(inline: Vec<Inline>) -> Block {
    Block::Paragraph { inline }
}

fn item(inline: Vec<Inline>) -> ListItem {
    ListItem {
        inline,
        children: vec![],
    }
}

/// Every text run in the blocks, in order, lists included.
fn texts(blocks: &[Block]) -> Vec<String> {
    fn inline(runs: &[Inline], out: &mut Vec<String>) {
        for run in runs {
            match run {
                Inline::Text { text, .. } => out.push(text.clone()),
                Inline::Ref { label, .. } | Inline::Concept { label, .. } => {
                    out.push(label.clone())
                }
                Inline::Edition { inline: nested, .. } => inline(nested, out),
            }
        }
    }
    let mut out = Vec::new();
    for block in blocks {
        match block {
            Block::Paragraph { inline: runs } | Block::Heading { inline: runs, .. } => {
                inline(runs, &mut out)
            }
            Block::List { items, .. } => {
                for it in items {
                    inline(&it.inline, &mut out);
                    out.extend(texts(&it.children));
                }
            }
            Block::Table { header, rows } => {
                for cell in header {
                    inline(cell, &mut out);
                }
                for row in rows {
                    for cell in row {
                        inline(cell, &mut out);
                    }
                }
            }
        }
    }
    out
}

#[test]
fn nothing_but_whitespace_is_no_notes() {
    assert_eq!(release_notes(""), vec![]);
    assert_eq!(release_notes("  \n\n \r\n"), vec![]);
}

#[test]
fn a_heading_keeps_its_level_and_loses_its_hashes() {
    assert_eq!(
        release_notes("### What changed"),
        vec![Block::Heading {
            level: 3,
            inline: vec![plain("What changed")],
        }]
    );
}

#[test]
fn lines_of_one_paragraph_join_and_a_blank_line_separates_two() {
    assert_eq!(
        release_notes("first line\nsecond line\n\nanother"),
        vec![
            paragraph(vec![plain("first line second line")]),
            paragraph(vec![plain("another")]),
        ]
    );
}

#[test]
fn windows_line_endings_read_the_same() {
    assert_eq!(release_notes("a\r\nb\r\n\r\nc"), release_notes("a\nb\n\nc"));
}

#[test]
fn bold_is_a_style_and_the_stars_are_gone() {
    assert_eq!(
        release_notes("**Nothing changed.** The rest is plain."),
        vec![paragraph(vec![
            bold("Nothing changed."),
            plain(" The rest is plain."),
        ])]
    );
}

#[test]
fn a_star_pair_that_never_closes_stays_as_written() {
    assert_eq!(
        release_notes("half **open"),
        vec![paragraph(vec![plain("half **open")])]
    );
}

#[test]
fn code_keeps_its_text_and_drops_its_backticks() {
    assert_eq!(
        release_notes("key `FF27` in `tauri.conf.json`"),
        vec![paragraph(vec![plain("key FF27 in tauri.conf.json")])]
    );
}

#[test]
fn a_rule_separates_and_draws_nothing() {
    assert_eq!(
        release_notes("above\n\n---\n\nbelow"),
        vec![
            paragraph(vec![plain("above")]),
            paragraph(vec![plain("below")])
        ]
    );
}

#[test]
fn dashes_make_a_bullet_list_one_item_per_line() {
    assert_eq!(
        release_notes("- **One.** first\n- two"),
        vec![Block::List {
            ordered: false,
            items: vec![
                item(vec![bold("One."), plain(" first")]),
                item(vec![plain("two")]),
            ],
        }]
    );
}

#[test]
fn numbers_make_an_ordered_list() {
    assert_eq!(
        release_notes("1. build\n2. publish"),
        vec![Block::List {
            ordered: true,
            items: vec![item(vec![plain("build")]), item(vec![plain("publish")])],
        }]
    );
}

#[test]
fn an_indented_line_continues_the_item_above_it() {
    assert_eq!(
        release_notes("- a long item\n  that wraps"),
        vec![Block::List {
            ordered: false,
            items: vec![item(vec![plain("a long item that wraps")])],
        }]
    );
}

#[test]
fn a_list_ends_where_a_paragraph_starts() {
    assert_eq!(
        release_notes("- one\n\nafter"),
        vec![
            Block::List {
                ordered: false,
                items: vec![item(vec![plain("one")])],
            },
            paragraph(vec![plain("after")]),
        ]
    );
}

/// The notes 0.2.0 shipped with, as they reached a 0.1.3 on 2026-09-23 and were drawn as one
/// run of raw text with every marker showing. Properties rather than the whole tree: the
/// wording is the release's, the shape is what this reader owes it.
#[test]
fn the_notes_of_0_2_0_read_as_a_heading_a_list_and_paragraphs() {
    let notes = include_str!("fixtures/20260923.release-notes-0.2.0.md");
    let blocks = release_notes(notes);

    let headings = blocks
        .iter()
        .filter(|b| matches!(b, Block::Heading { .. }))
        .count();
    assert_eq!(headings, 1, "`### What changed` is the one heading");

    let lists: Vec<_> = blocks
        .iter()
        .filter_map(|b| match b {
            Block::List { ordered, items } => Some((*ordered, items.len())),
            _ => None,
        })
        .collect();
    assert_eq!(lists, vec![(false, 8)], "eight bullets, in one list");

    for text in texts(&blocks) {
        for marker in ["**", "###", "`", "---"] {
            assert!(
                !text.contains(marker),
                "`{marker}` is markup and reached the text: {text:?}"
            );
        }
    }
}
