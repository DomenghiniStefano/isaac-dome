//! The lines between `{|` and `|}`, turned into `Block::Table`. Split out of `blocks.rs`
//! because the `rowspan`/`colspan` bookkeeping (`CellSpec`, `Carry`, `apply_row`) and the
//! bare-template-row reading (`entity_row_minimal`) are a self-contained piece of it: nothing
//! here reaches back into the paragraph/list states `blocks.rs`'s own `Parser` keeps, and
//! nothing outside this file needs to know a table's rows are built from `CellSpec`s rather
//! than plain cells.

use crate::inline::{collectible, parse_inline};
use crate::resolver::Resolver;
use crate::template::parse_template_at;
use crate::{Block, Diagnostics, Inline, Style};

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

/// A cell's wrapped continuation joined onto the line that opened it: ordinary MediaWiki, a
/// `|`/`!`-led line opens a cell and everything up to the next `|`/`!`/`|-`/`|+` is that
/// cell's content, newlines included. Read one raw line at a time without this, a wrapped
/// sentence had no shape the table parser knew and was counted `unmodelled_table_row` —
/// character/Tainted ???'s "Poop Varieties" table wraps several of its descriptions this way.
///
/// A bare line with nothing open before it — no `|`/`!` line has been seen since the last
/// `|-`/`|+`, or the table has none yet — has nowhere to attach and is left as its own line:
/// IBS's second Effects table opens straight from `|-` into five lines of `[[file:…]] prose`,
/// which stays unmodelled rather than being merged into a row that never opened a cell.
///
/// `{{entity row minimal|…}}` is the other shape that starts a row on its own, unprefixed at
/// this level because MediaWiki's preprocessor expands the template — which itself begins
/// `|` — before the table is ever split into lines; this parser never sees that expansion, so
/// it treats the call by name instead. Camo Undies' two Notes tables open a header row with
/// `!` cells and no `|-` before the first `{{entity row minimal|…}}` line: read as an ordinary
/// bare line it would merge into the header's last cell instead of opening its own row.
fn merge_continuations(lines: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cell_open = false;
    for line in lines {
        let t = line.trim();
        if t.starts_with("|-") || t.starts_with("|+") || opens_entity_row_minimal(t) {
            cell_open = false;
        } else if t.starts_with('|') || t.starts_with('!') {
            cell_open = true;
        } else if cell_open && !t.is_empty() {
            if let Some(prev) = out.last_mut() {
                prev.push(' ');
                prev.push_str(t);
            }
            continue;
        }
        out.push(line.clone());
    }
    out
}

/// Whether `line` opens with `{{entity row minimal|…}}` (any case: the wiki's own templates
/// are written in several). Only the name is checked, the same test `entity_row_minimal`
/// itself makes after a full parse; a cheap prefix check is enough here, where the only
/// question is whether the line is a continuation target.
fn opens_entity_row_minimal(line: &str) -> bool {
    line.to_ascii_lowercase()
        .starts_with("{{entity row minimal")
}

/// The lines between `{|` and `|}`. The first line with `!` cells is `header`; every
/// other line, `!` cells included (the pill table's subheadings), goes into `rows`.
///
/// A line that opens with neither is not wikitable syntax at all — two pages write a row as a
/// bare `{{entity row minimal|Name}}`, which MediaWiki's own preprocessor still renders — and
/// is read as one where this parser knows the shape, counted where it does not.
pub(super) fn build_table(lines: &[String], r: &Resolver, d: &mut Diagnostics) -> Block {
    let mut table = TableBuilder::default();
    let lines = merge_continuations(lines);
    for line in &lines {
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
        } else if let Some(cells) = entity_row_minimal(l, r, d) {
            table.close_row();
            table.row = cells;
            table.close_row();
            continue;
        } else {
            if !l.is_empty() {
                d.unmodelled_table_row();
            }
            continue;
        };
        table.row_is_header |= is_header;
        let sep = if is_header { "!!" } else { "||" };
        for cell in split_cells(body, sep) {
            let (attrs, content) = split_attributes(cell.trim());
            table.row.push(CellSpec {
                content: parse_inline(content, r, d),
                colspan: span_value(attrs, "colspan"),
                rowspan: span_value(attrs, "rowspan"),
            });
        }
    }
    table.close_row();
    Block::Table {
        header: table.header,
        rows: table.rows,
    }
}

/// A bare `{{entity row minimal|Name}}` line inside a table: not real wikitable syntax — no
/// leading `|` — but two pages write their per-monster notes this way (Camo Undies' Notes).
/// The template names the entity, the same reference `{{e|Name}}` would build; text the line
/// adds after it (`| Keeps spawning {{E|Spider}}s.`) becomes the row's second cell.
fn entity_row_minimal(line: &str, r: &Resolver, d: &mut Diagnostics) -> Option<Vec<CellSpec>> {
    let (t, end) = parse_template_at(line, 0)?;
    if t.name != "entity row minimal" {
        return None;
    }
    let name = t.args.first().cloned().unwrap_or_default();
    let reference = match collectible("e", &name, r, d) {
        Some(target) => Inline::Ref {
            target,
            label: name,
        },
        None => Inline::Text {
            text: name,
            style: Style::Plain,
        },
    };
    let mut cells = vec![CellSpec {
        content: vec![reference],
        colspan: 1,
        rowspan: 1,
    }];
    let rest = line.get(end..).unwrap_or_default().trim_start();
    if let Some(annotation) = rest
        .strip_prefix('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        cells.push(CellSpec {
            content: parse_inline(annotation, r, d),
            colspan: 1,
            rowspan: 1,
        });
    }
    Some(cells)
}

/// One cell as the wikitext writes it: its content, and how many columns/rows it reaches —
/// `colspan="2"`/`rowspan="2"`, both `1` when the cell declares neither.
struct CellSpec {
    content: Vec<Inline>,
    colspan: usize,
    rowspan: usize,
}

/// A cell whose `rowspan` still owes content to rows below the one that declared it: which
/// column it sits at, how wide, how many more rows it still covers, and its content.
struct Carry {
    col: usize,
    width: usize,
    remaining: usize,
    content: Vec<Inline>,
}

#[derive(Default)]
struct TableBuilder {
    header: Vec<Vec<Inline>>,
    rows: Vec<Vec<Vec<Inline>>>,
    row: Vec<CellSpec>,
    row_is_header: bool,
    /// At least one row already closed: from here on even `!` rows go into `rows`.
    closed_any: bool,
    /// `rowspan` cells still owed to the rows below the one that opened them.
    pending: Vec<Carry>,
}

impl TableBuilder {
    fn close_row(&mut self) {
        let is_header = std::mem::take(&mut self.row_is_header);
        if self.row.is_empty() {
            return;
        }
        let cells = apply_row(std::mem::take(&mut self.row), &mut self.pending);
        if is_header && !self.closed_any {
            self.header = cells;
        } else {
            self.rows.push(cells);
        }
        self.closed_any = true;
    }
}

/// Lays a row's own cells alongside whatever `rowspan` a row above still has running: at each
/// column, a `pending` carry from an earlier row is used before the row's next explicit cell
/// is — the same order the column would fill in if the spanned cell were written out by hand.
///
/// Birthright's Effects table is why this exists: Judas's row declares `rowspan="2"` on its
/// last two cells (a shared pickup quote and effect with Black Judas); Black Judas's row,
/// right after it, writes only its first cell. Dropping the span left Black Judas's row one
/// cell wide against a three-column header — this puts the other two back.
fn apply_row(cells: Vec<CellSpec>, pending: &mut Vec<Carry>) -> Vec<Vec<Inline>> {
    let mut op = std::mem::take(pending).into_iter().peekable();
    let mut ec = cells.into_iter();
    let mut out = Vec::new();
    let mut next_pending = Vec::new();
    let mut col = 0usize;
    loop {
        let carry_here = matches!(op.peek(), Some(carry) if carry.col <= col);
        if carry_here {
            let Some(carry) = op.next() else { break };
            let at = carry.col.max(col);
            out.push(carry.content.clone());
            if carry.remaining > 1 {
                next_pending.push(Carry {
                    col: at,
                    remaining: carry.remaining - 1,
                    ..carry
                });
            }
            col = at + carry.width.max(1);
            continue;
        }
        let Some(cell) = ec.next() else { break };
        out.push(cell.content.clone());
        if cell.rowspan > 1 {
            next_pending.push(Carry {
                col,
                width: cell.colspan.max(1),
                remaining: cell.rowspan - 1,
                content: cell.content,
            });
        }
        col += cell.colspan.max(1);
    }
    *pending = next_pending;
    out
}

/// `attr="N"` or `attr=N`, quoted or not (`rowspan=10` on D10's own table): the value inside a
/// cell's attribute prefix, `1` — spans nothing extra — when the attribute is absent or its
/// value can't be read, the same shape an unreadable `dlc` code restricts nothing.
fn span_value(attrs: &str, name: &str) -> usize {
    let lower = attrs.to_ascii_lowercase();
    let Some(pos) = lower.find(name) else {
        return 1;
    };
    let after = attrs.get(pos + name.len()..).unwrap_or_default();
    let Some(after) = after.trim_start().strip_prefix('=') else {
        return 1;
    };
    let digits: String = after
        .trim_start()
        .trim_start_matches(['"', '\''])
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok().filter(|n| *n > 0).unwrap_or(1)
}

/// The cell's attribute prefix and its content: `colspan="2"| text` → `("colspan=\"2\"",
/// "text")`. A `|` inside `[[…]]`/`{{…}}` is not a separator, and a `|` with no `=` before it
/// is not an attribute: the whole cell is content, with no attribute prefix.
fn split_attributes(cell: &str) -> (&str, &str) {
    let mut depth = 0i32;
    for (i, ch) in cell.char_indices() {
        match ch {
            '[' | '{' => depth += 1,
            ']' | '}' => depth -= 1,
            '|' if depth == 0 => {
                let before = cell.get(..i).unwrap_or_default();
                if before.contains('=') {
                    return (before, cell.get(i + 1..).unwrap_or_default().trim());
                }
                return ("", cell);
            }
            _ => {}
        }
    }
    ("", cell)
}

// Tests extract one variant and panic on the rest: the wildcard is the assertion.
#[allow(clippy::wildcard_enum_match_arm)]
#[cfg(test)]
mod tests {
    use crate::parse_blocks;
    use crate::resolver::fixtures::test_resolver;
    use crate::{Block, Diagnostics, Inline, Style, Target};

    fn p(s: &str) -> Vec<Block> {
        parse_blocks(s, &test_resolver(), &mut Diagnostics::default())
    }
    fn t(s: &str) -> Vec<Inline> {
        vec![Inline::Text {
            text: s.into(),
            style: Style::Plain,
        }]
    }
    fn pd(s: &str) -> (Vec<Block>, Diagnostics) {
        let mut d = Diagnostics::default();
        (parse_blocks(s, &test_resolver(), &mut d), d)
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

    /// Camo Undies' Notes: `{{entity row minimal|Name}}` alone on its line, no leading `|`,
    /// no `|-` between rows. Two such tables sit on that page and neither carries a header.
    #[test]
    fn a_bare_entity_row_minimal_line_is_its_own_row() {
        let blocks = p("{|\n {{entity row minimal|Mom}}\n {{entity row minimal|Uriel}}\n|}\n");
        let Block::Table { header, rows } = &blocks[0] else {
            panic!("table, got {blocks:?}")
        };
        assert!(header.is_empty());
        assert_eq!(rows.len(), 2, "{rows:?}");
        assert_eq!(
            rows[0],
            vec![vec![Inline::Ref {
                target: Target::Entity {
                    id: 45,
                    variant: 0,
                    subtype: 0
                },
                label: "Mom".into()
            }]]
        );
        assert_eq!(
            rows[1][0],
            vec![Inline::Ref {
                target: Target::Entity {
                    id: 271,
                    variant: 0,
                    subtype: 0
                },
                label: "Uriel".into()
            }]
        );
    }

    /// The other shape on Camo Undies' second table: an annotation after the template,
    /// separated by a bare `|` — not wikitable syntax (there was no leading `|` to begin
    /// with), just how the page happens to write "and here is a note about it".
    #[test]
    fn text_after_the_entity_row_minimal_template_is_a_second_cell() {
        let blocks = p("{|\n {{entity row minimal|Mom}} | still confused\n|}\n");
        let Block::Table { rows, .. } = &blocks[0] else {
            panic!("table, got {blocks:?}")
        };
        assert_eq!(rows[0].len(), 2, "{rows:?}");
        assert_eq!(rows[0][1], t("still confused"));
    }

    /// A name the resolver does not know keeps its own words, the same fallback every other
    /// reference template has, rather than vanishing along with the row.
    #[test]
    fn an_unresolved_entity_row_minimal_name_is_text_in_its_own_row() {
        let (blocks, d) = pd("{|\n {{entity row minimal|Not An Entity}}\n|}\n");
        let Block::Table { rows, .. } = &blocks[0] else {
            panic!("table, got {blocks:?}")
        };
        assert_eq!(rows[0], vec![t("Not An Entity")]);
        assert_eq!(d.unresolved.get("e"), Some(&1));
    }

    /// IBS's second Effects table: five lines of `[[file:…]] prose`, neither wikitable syntax
    /// nor a template line at all. They stay dropped — nothing here can turn prose glued to a
    /// file link into a row — but the drop is now counted instead of silent.
    #[test]
    fn a_bare_line_that_is_neither_syntax_nor_a_known_template_is_counted_not_silent() {
        let (blocks, d) = pd("{|\n! Poop\n|-\n[[file:x.png|32px]] Black Poop: slows enemies.\n[[file:y.png|32px]] Toxic cloud: poisons enemies.\n|}\n");
        let Block::Table { header, rows } = &blocks[0] else {
            panic!("table, got {blocks:?}")
        };
        assert_eq!(header, &vec![t("Poop")]);
        assert!(rows.is_empty(), "the two lines produced no row: {rows:?}");
        assert_eq!(d.unmodelled_table_rows, 2);
    }

    /// character/Tainted ???'s "Poop Varieties" table: a cell's description wraps onto a line
    /// of its own, real MediaWiki (a cell's content runs until the next `|`/`!`/`|-`/`|+`, not
    /// until the next raw newline). Read one line at a time without `merge_continuations`, the
    /// wrapped sentence had no shape this parser knew and was counted `unmodelled_table_row`;
    /// joined, it is the second sentence of the cell above it.
    #[test]
    fn a_cell_s_wrapped_continuation_joins_the_cell_above_it() {
        let (blocks, d) = pd(concat!(
            "{|\n",
            "! Icon\n",
            "|-\n",
            "| A\n",
            "| A normal poop.\n",
            "Can drop pick-ups like normal.\n",
            "|-\n",
            "| B\n",
            "| Something else.\n",
            "|}\n",
        ));
        let Block::Table { rows, .. } = &blocks[0] else {
            panic!("table, got {blocks:?}")
        };
        assert_eq!(rows.len(), 2, "{rows:?}");
        assert_eq!(
            rows[0][1],
            t("A normal poop. Can drop pick-ups like normal.")
        );
        assert_eq!(rows[1][1], t("Something else."));
        assert_eq!(d.unmodelled_table_rows, 0, "{d:?}");
    }

    /// Camo Undies' actual shape: a two-cell `!` header with no `|-` before the first
    /// `{{entity row minimal|…}}` line. The continuation merge must not read that line as the
    /// header's own wrapped text — it opens its own row, same as
    /// `a_bare_entity_row_minimal_line_is_its_own_row` pins for the simpler case with no
    /// header at all.
    #[test]
    fn an_entity_row_minimal_line_right_after_a_header_still_opens_its_own_row() {
        let blocks = p(
            "{|\n ! Name\n ! ID\n {{entity row minimal|Mom}}\n {{entity row minimal|Uriel}}\n|}\n",
        );
        let Block::Table { header, rows } = &blocks[0] else {
            panic!("table, got {blocks:?}")
        };
        assert_eq!(header, &vec![t("Name"), t("ID")]);
        assert_eq!(rows.len(), 2, "{rows:?}");
        assert_eq!(
            rows[0],
            vec![vec![Inline::Ref {
                target: Target::Entity {
                    id: 45,
                    variant: 0,
                    subtype: 0
                },
                label: "Mom".into()
            }]]
        );
    }

    /// The line right after `|-`, with no `|`/`!` cell opened yet, has nowhere to attach: it
    /// stays exactly what `a_bare_line_that_is_neither_syntax_nor_a_known_template_is_counted_not_silent`
    /// already pins, unaffected by the continuation merge.
    #[test]
    fn a_bare_line_straight_after_a_row_separator_still_has_nowhere_to_attach() {
        let (blocks, d) = pd("{|\n|-\nprose with no cell before it\n|}\n");
        let Block::Table { rows, .. } = &blocks[0] else {
            panic!("table, got {blocks:?}")
        };
        assert!(rows.is_empty(), "{rows:?}");
        assert_eq!(d.unmodelled_table_rows, 1);
    }

    /// Birthright's Effects table: Judas's row declares `rowspan="2"` on its last two cells,
    /// shared with Black Judas right after — a name cell of its own, then the two it does not
    /// repeat in the source.
    #[test]
    fn a_rowspan_cell_is_repeated_onto_the_row_it_still_covers() {
        let blocks = p(concat!(
            "{|\n",
            "! Character\n",
            "! Quote\n",
            "! Effect\n",
            "|-\n",
            "| Judas\n",
            "| rowspan=\"2\" | Belial incarnate\n",
            "| rowspan=\"2\" | shared effect text\n",
            "|-\n",
            "| Black Judas\n",
            "|-\n",
            "| Cain\n",
            "| own quote\n",
            "| own effect\n",
            "|}\n",
        ));
        let Block::Table { rows, .. } = &blocks[0] else {
            panic!("table, got {blocks:?}")
        };
        assert_eq!(rows.len(), 3, "{rows:?}");
        // The row that declares the span keeps all three cells, unaffected.
        assert_eq!(rows[0][1], t("Belial incarnate"));
        assert_eq!(rows[0][2], t("shared effect text"));
        // The row right after it, which the source writes with only one cell, reads the
        // other two back from the span instead of losing them.
        assert_eq!(
            rows[1].len(),
            3,
            "the spanned row is one cell wide against a three-column header: {:?}",
            rows[1]
        );
        assert_eq!(rows[1][0], t("Black Judas"));
        assert_eq!(rows[1][1], t("Belial incarnate"));
        assert_eq!(rows[1][2], t("shared effect text"));
        // A row after the span has run its course is unaffected: its own three cells, no
        // carry-over left to apply.
        assert_eq!(rows[2], vec![t("Cain"), t("own quote"), t("own effect")]);
    }

    /// PHD's real shape: `colspan="2"` spans the **whole** row (one cell, not two columns'
    /// worth of the same text), and it does not spill into the next row the way `rowspan`
    /// does. Dropping the attribute already gave the right cell count; this pins that a
    /// `colspan` alone creates no carry.
    #[test]
    fn a_colspan_alone_does_not_create_a_carry_into_the_next_row() {
        let blocks = p(concat!(
            "{|\n",
            "! Pill !! Changes Into\n",
            "|-\n",
            "| Stat up\n",
            "| Stat down\n",
            "|-\n",
            "| colspan=\"2\" | ''I Found Pills''\n",
            "|-\n",
            "| Amnesia\n",
            "| Wide-brimmed hat\n",
            "|}\n",
        ));
        let Block::Table { rows, .. } = &blocks[0] else {
            panic!("table, got {blocks:?}")
        };
        assert_eq!(rows.len(), 3, "{rows:?}");
        assert_eq!(rows[1].len(), 1, "one merged cell: {:?}", rows[1]);
        assert_eq!(
            rows[2].len(),
            2,
            "not shortened by the row above: {:?}",
            rows[2]
        );
    }
}
