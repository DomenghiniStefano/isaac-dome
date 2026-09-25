//! From wikitext lines to blocks. One pass over the lines with three states in flight:
//! paragraph, list, table. Each line closes the states it doesn't belong to and opens
//! its own.
//!
//! The input is external data: no path may panic. Whatever isn't recognized degrades to
//! a paragraph.

use crate::inline::parse_inline;
use crate::resolver::Resolver;
use crate::template::{template_segments, Segment, Template};
use crate::{Block, Diagnostics, Inline, ListItem};

/// A list item still to be built: depth, whether the last marker is `#`, text.
struct RawItem {
    depth: usize,
    ordered: bool,
    text: String,
}

/// The state of the pass: the blocks already closed and the three states in flight.
struct Parser<'a> {
    r: &'a Resolver,
    d: &'a mut Diagnostics,
    out: Vec<Block>,
    para: Vec<String>,
    list: Vec<RawItem>,
    table: Option<Vec<String>>,
}

/// A template whose content is block-level, and what this pass is allowed to do with it.
/// Three kinds because three reasons — all of them ending in the same place, a shape the
/// contract can already say, which is why closing B49 added no `Block` variant.
enum Wrapper {
    /// Layout and nothing else: the wrapper decides how wide the columns are
    /// (`column list`) or that the box scrolls (`scroll box`), and the content it holds is
    /// already blocks. Dropped whole, because nothing it says is lost.
    Layout { param: &'static str },
    /// Content whose wrapper the tree already carries somewhere else. Every `{{bug|…}}`
    /// that spans lines sits under `== Bugs ==`, which is `SectionKind::Bugs`, and the
    /// single-line case has been dropped inline since `CONTENT_WRAPPERS` existed: keeping
    /// the multi-line one would model the same wrapper two ways. Its content is the
    /// **positional** argument, not a named one.
    Transparent,
    /// A sentence, then the blocks it introduces. `{{Book of Virtues synergy|description=…}}`
    /// means *with this item, this happens*, so dropping it whole loses the half that says
    /// with what. Re-closed at the end of its first line instead: the inline pass then sees
    /// the single-line shape it already models — the item's label keeps being built in one
    /// place — and the `**` lines below stay the children of the item that opened.
    Headed { param: &'static str },
}

/// The wrappers whose content is block-level, and nothing else. Measured on the committed
/// dataset on 2026-09-15, by spans that open on one line and close on another:
/// `column list` 55, `bug` 9, `book of virtues synergy` 6, `scroll box` 1,
/// `book of belial synergy` 1. A closed list on purpose — a family that is not here is
/// counted by `text_nodes_carry_no_raw_template_syntax` rather than guessed at.
fn wrapper(name: &str) -> Option<Wrapper> {
    match name {
        "column list" | "scroll box" => Some(Wrapper::Layout { param: "content" }),
        "bug" => Some(Wrapper::Transparent),
        "book of virtues synergy" | "book of belial synergy" => Some(Wrapper::Headed {
            param: "description",
        }),
        _ => None,
    }
}

/// A wrapper's content put back as the lines it is. A parameter's value arrives trimmed,
/// so the newline it opened with is gone: without putting one back, the first `**` lands
/// on the line the wrapper opened on and the list it holds becomes one item of text.
/// And one after it only when the page does not already continue on a new line — an
/// unconditional one leaves a blank line where the wrapper closed, and a blank line
/// flushes the list, which is the same cut this pass has a rule against, arriving from the
/// other side.
fn push_content(out: &mut String, content: Option<&String>, after: &str) {
    let Some(content) = content else {
        return;
    };
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(content);
    if !after.starts_with('\n') {
        out.push('\n');
    }
}

/// The body with those wrappers replaced by what the line pass can read, and nothing else
/// touched. `template_segments` is what says where one ends: a line-by-line pass cannot.
fn unwrapped(body: &str) -> String {
    template_segments(body).fold(String::new(), |mut out, segment| {
        match segment {
            Segment::Text(text) => out.push_str(text),
            Segment::Template {
                template,
                source,
                after,
            } => push_template(&mut out, &template, source, after),
        }
        out
    })
}

/// One top-level template, unwrapped if it is a block-level wrapper that spans lines.
///
/// Only a template that spans lines. One that closes where it opened is already visible to the
/// pass, and moving its content onto a line of its own would cut the list item it sits in —
/// which is where 538 of the 547 `{{bug|…}}` live. Every other template is content: it belongs
/// to the line it is on.
fn push_template(out: &mut String, t: &Template, source: &str, after: &str) {
    if !source.contains('\n') {
        out.push_str(source);
        return;
    }
    match wrapper(&t.name) {
        Some(Wrapper::Layout { param }) => push_content(out, t.named.get(param), after),
        Some(Wrapper::Transparent) => push_content(out, t.args.first(), after),
        Some(Wrapper::Headed { param }) => push_headed(out, t, param, source, after),
        None => out.push_str(source),
    }
}

/// A headed wrapper: the sentence goes back inside a template that closes on its line, so the
/// inline pass reads it; everything after the first newline is already the lines it introduced.
///
/// A headed wrapper whose sentence holds no newline of its own spans lines for some other
/// reason: left as it is rather than reshaped on a guess.
fn push_headed(out: &mut String, t: &Template, param: &str, source: &str, after: &str) {
    let Some((head, rest)) = t.named.get(param).and_then(|v| v.split_once('\n')) else {
        out.push_str(source);
        return;
    };
    out.push_str(&format!("{{{{{}|{param}={}}}}}\n", t.name, head.trim_end()));
    out.push_str(rest);
    if !after.starts_with('\n') {
        out.push('\n');
    }
}

pub fn parse_blocks(body: &str, r: &Resolver, d: &mut Diagnostics) -> Vec<Block> {
    let mut p = Parser {
        r,
        d,
        out: Vec::new(),
        para: Vec::new(),
        list: Vec::new(),
        table: None,
    };
    for raw in unwrapped(body).lines() {
        p.line(raw.trim_end());
    }
    p.finish()
}

impl Parser<'_> {
    fn line(&mut self, line: &str) {
        // Inside a table every line belongs to it, until `|}`.
        if let Some(rows) = self.table.as_mut() {
            if line.trim_start().starts_with("|}") {
                self.close_table();
            } else {
                rows.push(line.to_string());
            }
            return;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with("{|") {
            self.flush_para();
            self.flush_list();
            self.table = Some(Vec::new());
            return;
        }
        if let Some((level, text)) = heading(trimmed) {
            self.flush_para();
            self.flush_list();
            let inline = parse_inline(text, self.r, self.d);
            self.out.push(Block::Heading { level, inline });
            return;
        }
        let markers = trimmed
            .chars()
            .take_while(|c| *c == '*' || *c == '#')
            .count();
        if markers > 0 {
            self.flush_para();
            let ordered = trimmed.chars().nth(markers - 1) == Some('#');
            let text = trimmed.get(markers..).unwrap_or_default().trim();
            self.list.push(RawItem {
                depth: markers,
                ordered,
                text: text.to_string(),
            });
            return;
        }
        // A line that is nothing but closing braces belongs to a template opened further
        // up — `column list` wraps a nested list exactly that way. Left to fall through
        // it does two kinds of damage: it emits a `}}` paragraph, and it **cuts the list
        // in two**, because reaching `flush_list` below is what any non-list line does.
        // Dropping it keeps the list whole. It does not represent the wrapper, which
        // stays unmodelled and is counted instead of being hidden.
        if !trimmed.is_empty() && trimmed.chars().all(|c| c == '}') {
            self.d.orphan_closer();
            return;
        }
        self.flush_list();
        if trimmed.is_empty() {
            self.flush_para();
            return;
        }
        // `:` and `;` (definitions and indents) are paragraphs like any other.
        let text = trimmed
            .strip_prefix(':')
            .or_else(|| trimmed.strip_prefix(';'))
            .map(str::trim)
            .unwrap_or(trimmed);
        if text == "__TOC__" {
            return;
        }
        self.para.push(text.to_string());
    }

    fn finish(mut self) -> Vec<Block> {
        self.flush_para();
        self.flush_list();
        // A table without `|}` closes with whatever it has.
        self.close_table();
        self.out
    }

    /// A paragraph that ends up empty after inline parsing (`{{clear}}`, a comment) is not emitted.
    fn flush_para(&mut self) {
        if self.para.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.para).join(" ");
        let inline = parse_inline(&text, self.r, self.d);
        if !inline.is_empty() {
            self.out.push(Block::Paragraph { inline });
        }
    }

    fn flush_list(&mut self) {
        if self.list.is_empty() {
            return;
        }
        let items = std::mem::take(&mut self.list);
        let blocks = build_lists(&items, 1, self.r, self.d);
        self.out.extend(blocks);
    }

    fn close_table(&mut self) {
        if let Some(lines) = self.table.take() {
            let table = build_table(&lines, self.r, self.d);
            self.out.push(table);
        }
    }
}

/// `=== T ===` → `(3, "T")`, `==== T ====` → `(4, "T")`. Level 2 never appears here: the
/// sections are already split further upstream. Other levels stay as text.
fn heading(line: &str) -> Option<(u8, &str)> {
    for level in [4u8, 3] {
        let marks = "=".repeat(usize::from(level));
        let inner = line
            .strip_prefix(marks.as_str())
            .and_then(|s| s.strip_suffix(marks.as_str()));
        if let Some(inner) = inner {
            if !inner.starts_with('=') && !inner.ends_with('=') {
                return Some((level, inner.trim()));
            }
        }
    }
    None
}

/// Items at depth `depth` become a list; deeper items that follow an item end up in its
/// `children`. Multiple lists appear when `ordered` changes at the same level. Deeper
/// items with no item of this level in front of them hang from an empty item.
fn build_lists(items: &[RawItem], depth: usize, r: &Resolver, d: &mut Diagnostics) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut cur: Option<(bool, Vec<ListItem>)> = None;
    let mut i = 0;
    while let Some(item) = items.get(i) {
        if item.depth < depth {
            break;
        }
        let ordered = item.ordered;
        let inline = if item.depth > depth {
            Vec::new()
        } else {
            i += 1;
            parse_inline(&item.text, r, d)
        };
        let start = i;
        while items.get(i).is_some_and(|it| it.depth > depth) {
            i += 1;
        }
        let deeper = items.get(start..i).unwrap_or_default();
        let children = build_lists(deeper, depth + 1, r, d);
        let item = ListItem { inline, children };
        match cur.as_mut() {
            Some((o, v)) if *o == ordered => v.push(item),
            // The other kind of list, or no list open yet: close what is open and start one.
            Some(_) | None => {
                if let Some((o, v)) = cur.take() {
                    blocks.push(Block::List {
                        ordered: o,
                        items: v,
                    });
                }
                cur = Some((ordered, vec![item]));
            }
        }
    }
    if let Some((o, v)) = cur {
        blocks.push(Block::List {
            ordered: o,
            items: v,
        });
    }
    blocks
}

/// Splits a table row into cells, ignoring a separator nested inside `{{…}}` or `[[…]]`.
///
/// `||` is both the cell separator and, inside a template, an empty argument:
/// `{{e|Mask + Heart||Heart}}` is one cell, and cutting it in two leaves each half
/// holding template syntax that no longer parses — literal `{{` in a text node.
///
/// Depth counts the two-character openers and saturates at zero, so a stray `}}` on a
/// malformed row can't drive it negative: the row degrades into one cell, never into
/// none. Every delimiter here is ASCII, so slicing on these byte offsets stays on
/// character boundaries.
fn split_cells<'a>(body: &'a str, sep: &str) -> Vec<&'a str> {
    let bytes = body.as_bytes();
    let sep = sep.as_bytes();
    let mut cells = Vec::new();
    let mut depth: usize = 0;
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes.get(i..i + 2) {
            Some(b"{{") | Some(b"[[") => {
                depth += 1;
                i += 2;
            }
            Some(b"}}") | Some(b"]]") => {
                depth = depth.saturating_sub(1);
                i += 2;
            }
            Some(pair) if depth == 0 && pair == sep => {
                cells.push(&body[start..i]);
                i += 2;
                start = i;
            }
            _ => i += 1,
        }
    }
    cells.push(&body[start..]);
    cells
}

/// The lines between `{|` and `|}`. The first line with `!` cells is `header`; every
/// other line, `!` cells included (the pill table's subheadings), goes into `rows`.
fn build_table(lines: &[String], r: &Resolver, d: &mut Diagnostics) -> Block {
    let mut table = TableBuilder::default();
    for line in lines {
        let l = line.trim();
        if l.starts_with("|-") {
            table.close_row();
            continue;
        }
        if l.starts_with("|+") {
            continue; // caption
        }
        let (is_header, body) = if let Some(b) = l.strip_prefix('!') {
            (true, b)
        } else if let Some(b) = l.strip_prefix('|') {
            (false, b)
        } else {
            continue;
        };
        table.row_is_header |= is_header;
        let sep = if is_header { "!!" } else { "||" };
        for cell in split_cells(body, sep) {
            let cell = strip_attributes(cell.trim());
            table.row.push(parse_inline(cell, r, d));
        }
    }
    table.close_row();
    Block::Table {
        header: table.header,
        rows: table.rows,
    }
}

#[derive(Default)]
struct TableBuilder {
    header: Vec<Vec<Inline>>,
    rows: Vec<Vec<Vec<Inline>>>,
    row: Vec<Vec<Inline>>,
    row_is_header: bool,
    /// At least one row already closed: from here on even `!` rows go into `rows`.
    closed_any: bool,
}

impl TableBuilder {
    fn close_row(&mut self) {
        let is_header = std::mem::take(&mut self.row_is_header);
        if self.row.is_empty() {
            return;
        }
        let cells = std::mem::take(&mut self.row);
        if is_header && !self.closed_any {
            self.header = cells;
        } else {
            self.rows.push(cells);
        }
        self.closed_any = true;
    }
}

/// `colspan="2"| text` → `text`. A `|` inside `[[…]]`/`{{…}}` is not a separator, and a
/// `|` with no `=` before it is not an attribute: the cell stays whole.
fn strip_attributes(cell: &str) -> &str {
    let mut depth = 0i32;
    for (i, ch) in cell.char_indices() {
        match ch {
            '[' | '{' => depth += 1,
            ']' | '}' => depth -= 1,
            '|' if depth == 0 => {
                let before = cell.get(..i).unwrap_or_default();
                if before.contains('=') {
                    return cell.get(i + 1..).unwrap_or_default().trim();
                }
                return cell;
            }
            _ => {}
        }
    }
    cell
}

// Tests extract one variant and panic on the rest: the wildcard is the assertion.
#[allow(clippy::wildcard_enum_match_arm)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::fixtures::test_resolver;
    use crate::{Block, Diagnostics, Inline, ListItem, Style};

    fn p(s: &str) -> Vec<Block> {
        parse_blocks(s, &test_resolver(), &mut Diagnostics::default())
    }
    fn t(s: &str) -> Vec<Inline> {
        vec![Inline::Text {
            text: s.into(),
            style: Style::Plain,
        }]
    }

    /// B49: `{{column list | width = … | content = …}}` opens in the middle of a list item
    /// and closes on a line of its own several lines below, which a line-by-line pass cannot
    /// see as one template: the opener stayed in the text as itself and the `}}` became a
    /// paragraph that cut the list in two. Reported from the running app on 2026-09-14,
    /// where Beelzebub's page printed the wrapper's own source where the flies should be.
    ///
    /// It is pure layout — the content it wraps is already a wiki list — so it is unwrapped
    /// before the pass, and what is left is the list the page meant.
    #[test]
    fn a_column_list_is_unwrapped_into_the_list_it_holds() {
        let blocks =
            p("* The flies: {{column list | width = 15em | content =\n** one\n** two\n}}\n");
        let Block::List { items, .. } = &blocks[0] else {
            panic!("a list, got {blocks:?}")
        };
        assert_eq!(items.len(), 1, "{items:?}");
        let Block::List { items: inner, .. } = &items[0].children[0] else {
            panic!("a nested list, got {:?}", items[0].children)
        };
        assert_eq!(inner.len(), 2);
        assert_eq!(inner[0].inline, t("one"));
        // Nothing of the wrapper survives, neither its opener nor the line that closed it.
        assert_eq!(blocks.len(), 1, "{blocks:?}");
        assert_eq!(items[0].inline, t("The flies:"));
    }

    /// The half of B49 that is not layout. The two `X synergy` templates carry their
    /// sentence in `description=`, and when that sentence ends in a colon the list it
    /// introduces runs below it — inside the template, which therefore spans lines.
    /// Dropping the wrapper whole would lose the half that says *with what*, so it is
    /// re-closed at the end of its first line instead: the inline pass then sees the
    /// single-line shape it already models, and the `**` lines stay what they already
    /// are, the children of the item that opened.
    #[test]
    fn a_headed_wrapper_keeps_its_sentence_and_the_list_it_introduces() {
        let blocks = p("* {{Book of Virtues synergy|description=One of these:\n** one\n** two}}\n");
        let Block::List { items, .. } = &blocks[0] else {
            panic!("a list, got {blocks:?}")
        };
        assert_eq!(blocks.len(), 1, "{blocks:?}");
        assert_eq!(items.len(), 1, "{items:?}");
        let flat: String = items[0]
            .inline
            .iter()
            .filter_map(|i| match i {
                Inline::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert!(
            flat.contains("One of these:"),
            "the sentence is lost: {:?}",
            items[0].inline
        );
        // …and it arrives as text, not as the source that produced it: the raw opener
        // contains the sentence too, so a `contains` on its own passes while broken.
        assert!(!flat.contains("{{"), "raw source in the item: {flat:?}");
        let Block::List { items: inner, .. } = &items[0].children[0] else {
            panic!("a nested list, got {:?}", items[0].children)
        };
        assert_eq!(inner.len(), 2);
        assert_eq!(inner[1].inline, t("two"));
    }

    /// `{{bug|…}}` wraps content, and the single-line case has been dropped inline since
    /// `CONTENT_WRAPPERS` existed: the wrapper says "this is a defect", which every one of
    /// the nine that span lines already says by sitting under `== Bugs ==` —
    /// `SectionKind::Bugs` in the tree. So they are dropped too, rather than the same
    /// wrapper being modelled two ways, and what is left is the paragraph and the list the
    /// page meant. `dlc=` goes with it, exactly as it does in the single-line case.
    #[test]
    fn a_bug_that_spans_lines_becomes_the_blocks_it_holds() {
        let blocks = p("{{bug|dlc=r|Certain monsters do not:\n* one\n* two}}\n");
        assert_eq!(
            blocks[0],
            Block::Paragraph {
                inline: t("Certain monsters do not:")
            },
            "{blocks:?}"
        );
        let Block::List { items, .. } = &blocks[1] else {
            panic!("a list, got {blocks:?}")
        };
        assert_eq!(items.len(), 2, "{items:?}");
        assert_eq!(blocks.len(), 2, "{blocks:?}");
    }

    /// The second layout wrapper, found by the same census that closed the other two
    /// families: one use, on The Lost's page, where it holds a section of seeds. It decides
    /// that the box scrolls and nothing else, and unlike `column list` what it holds is not
    /// a list — which costs nothing, because headings and paragraphs are what the pass
    /// reads anyway.
    #[test]
    fn a_scroll_box_is_unwrapped_like_the_other_layout_wrapper() {
        let blocks = p("{{scroll box | content =\n=== Seeds ===\nGPE3 2T1H\n}}\n");
        assert_eq!(
            blocks,
            vec![
                Block::Heading {
                    level: 3,
                    inline: t("Seeds")
                },
                Block::Paragraph {
                    inline: t("GPE3 2T1H")
                }
            ]
        );
    }

    /// Only a template that spans lines is this pass's business, and this is what the rule
    /// costs when it is missing: 538 of the 547 `{{bug|…}}` in the dataset close on the
    /// line they opened on, most of them inside a list item. Unwrapping one of those would
    /// put its content on a line of its own — and any non-list line flushes the list — so
    /// the item would be cut in two and the list with it. The pass would be repairing one
    /// family by breaking five hundred.
    #[test]
    fn a_wrapper_that_closes_on_its_own_line_is_left_where_it_is() {
        let blocks = p("* before {{bug|a known defect}} after\n* second\n");
        let Block::List { items, .. } = &blocks[0] else {
            panic!("a list, got {blocks:?}")
        };
        assert_eq!(blocks.len(), 1, "{blocks:?}");
        assert_eq!(items.len(), 2, "{items:?}");
    }

    #[test]
    fn paragraphs_join_lines_and_split_on_blank() {
        assert_eq!(
            p("a\nb\n\nc\n"),
            vec![
                Block::Paragraph { inline: t("a b") },
                Block::Paragraph { inline: t("c") }
            ]
        );
    }

    #[test]
    fn nested_list() {
        let v = p("* one\n** two\n** three\n* four\n# n1\n");
        assert_eq!(
            v,
            vec![
                Block::List {
                    ordered: false,
                    items: vec![
                        ListItem {
                            inline: t("one"),
                            children: vec![Block::List {
                                ordered: false,
                                items: vec![
                                    ListItem {
                                        inline: t("two"),
                                        children: vec![]
                                    },
                                    ListItem {
                                        inline: t("three"),
                                        children: vec![]
                                    }
                                ]
                            }]
                        },
                        ListItem {
                            inline: t("four"),
                            children: vec![]
                        },
                    ]
                },
                Block::List {
                    ordered: true,
                    items: vec![ListItem {
                        inline: t("n1"),
                        children: vec![]
                    }]
                },
            ]
        );
    }

    #[test]
    fn table() {
        let src = "{| class=\"wikitable\"\n ! Pill !! Changes Into\n |-\n | Stat up pill\n | Stat down pill\n |-\n ! colspan=\"2\"| Neutral pills\n |-\n |colspan=\"2\"| ''I Found Pills''\n|}\n";
        let v = p(src);
        let Block::Table { header, rows } = &v[0] else {
            panic!("table")
        };
        assert_eq!(header, &vec![t("Pill"), t("Changes Into")]);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], vec![t("Stat up pill"), t("Stat down pill")]);
        assert_eq!(rows[1], vec![t("Neutral pills")]);
        assert_eq!(
            rows[2],
            vec![vec![Inline::Text {
                text: "I Found Pills".into(),
                style: Style::Italic
            }]]
        );
    }

    #[test]
    fn headings_and_definition_lines() {
        let v = p("=== Phase 1 ===\ntext\n==== Phase 2-1 (100-80% HP) ====\n: indented\n");
        assert_eq!(
            v[0],
            Block::Heading {
                level: 3,
                inline: t("Phase 1")
            }
        );
        assert_eq!(v[1], Block::Paragraph { inline: t("text") });
        assert_eq!(
            v[2],
            Block::Heading {
                level: 4,
                inline: t("Phase 2-1 (100-80% HP)")
            }
        );
        assert_eq!(
            v[3],
            Block::Paragraph {
                inline: t("indented")
            }
        );
    }

    #[test]
    fn empty_paragraphs_are_dropped() {
        assert_eq!(p("__TOC__\n{{cit|p}}\n"), vec![]);
    }

    #[test]
    fn orphan_deep_item_hangs_from_empty_parent() {
        let v = p("** deep\n* top\n");
        assert_eq!(
            v,
            vec![Block::List {
                ordered: false,
                items: vec![
                    ListItem {
                        inline: vec![],
                        children: vec![Block::List {
                            ordered: false,
                            items: vec![ListItem {
                                inline: t("deep"),
                                children: vec![]
                            }]
                        }]
                    },
                    ListItem {
                        inline: t("top"),
                        children: vec![]
                    },
                ]
            }]
        );
    }

    /// A lone `}}` closes a template that opened on an earlier list line — this is how
    /// `column list` wraps a nested list, on some fifty pages. Emitted as a paragraph it
    /// does two kinds of damage: a junk block, and a list **cut in two**, because any
    /// non-list line flushes the list. The items after the closer belong to the same
    /// list as those before it.
    #[test]
    fn a_lone_template_closer_does_not_cut_the_list_in_two() {
        let v = p("* first {{column list|content =\n** nested\n}}\n* second\n");
        assert_eq!(v.len(), 1, "one list, not list + paragraph + list: {v:?}");
        let Block::List { items, .. } = &v[0] else {
            panic!("list")
        };
        assert_eq!(
            items.len(),
            2,
            "`first` and `second` are siblings: {items:?}"
        );
    }

    /// `||` inside a template is an empty argument, not a cell separator. Mystery Egg's
    /// table carries `{{e|Mask + Heart||Heart}}`; split on the bare `||` it becomes two
    /// cells, each holding half a template that no longer parses — which is how literal
    /// `{{` ends up in a text node.
    #[test]
    fn a_pipe_pair_inside_a_template_is_an_argument_not_a_cell_separator() {
        let v = p("{|\n| {{e|Mask + Heart||Heart}} || after\n");
        let Block::Table { rows, .. } = &v[0] else {
            panic!("table")
        };
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].len(),
            2,
            "the template is one cell, `after` is the other: {rows:?}"
        );
    }

    #[test]
    fn table_cell_keeps_pipe_inside_link_and_survives_missing_close() {
        let v = p("{|\n| [[Shot Speed|speed]] || x=1|kept\n");
        let Block::Table { header, rows } = &v[0] else {
            panic!("table")
        };
        assert!(header.is_empty());
        assert_eq!(
            rows,
            &vec![vec![
                vec![Inline::Concept {
                    page: "Shot Speed".into(),
                    label: "speed".into()
                }],
                t("kept")
            ]]
        );
    }
}
