//! Release notes: the markdown a release carries, read into the blocks the wiki already draws.
//!
//! **Why not markdown into HTML.** The notes arrive inside `latest.json`, which travels over
//! TLS but is not signed (`docs/release.md`, on `requireSignedVersion`): the text is whatever
//! the server answered. Rendered as markup it would be HTML from the network inside a webview
//! that can call Tauri commands. Read here into typed blocks, the worst a crafted response can
//! do is say something untrue in plain words.
//!
//! The subset is the one the notes are written in — headings, paragraphs, bullet and numbered
//! lists, `**bold**`, `` `code` ``, a `---` rule. Anything else **stays text**: a reader that
//! dropped what it did not know would turn a typo in the notes into a missing sentence.

use wiki::{Block, Inline, ListItem, Style};

pub fn release_notes(markdown: &str) -> Vec<Block> {
    let mut reader = Reader::default();
    // `lines` ends a line at `\n` and at `\r\n` alike, so a body edited on Windows reads the same.
    for raw in markdown.lines() {
        let line = raw.trim_end();
        let text = line.trim_start();
        if text.is_empty() || is_rule(text) {
            reader.end_block();
        } else if let Some((level, title)) = heading(text) {
            reader.end_block();
            reader.blocks.push(Block::Heading {
                level,
                inline: inline(title),
            });
        } else if let Some((ordered, first)) = list_item(text) {
            reader.item(ordered, first);
        } else if line.starts_with(char::is_whitespace) && reader.in_item() {
            // An indented line under an item is that item wrapping, not a new paragraph.
            reader.wrap_item(text);
        } else {
            reader.end_list();
            reader.paragraph.push(text.to_string());
        }
    }
    reader.end_block();
    reader.blocks
}

#[derive(Default)]
struct Reader {
    blocks: Vec<Block>,
    /// The lines of the paragraph being read, joined with a space when it ends.
    paragraph: Vec<String>,
    /// The list being read: ordered or not, and each item's text so far.
    list: Option<(bool, Vec<String>)>,
}

impl Reader {
    fn item(&mut self, ordered: bool, text: &str) {
        self.end_paragraph();
        match &mut self.list {
            Some((same, items)) if *same == ordered => items.push(text.to_string()),
            _ => {
                self.end_list();
                self.list = Some((ordered, vec![text.to_string()]));
            }
        }
    }

    /// Whether a list item is open for an indented line to continue.
    fn in_item(&self) -> bool {
        self.list
            .as_ref()
            .is_some_and(|(_, items)| !items.is_empty())
    }

    fn wrap_item(&mut self, text: &str) {
        if let Some(last) = self.list.as_mut().and_then(|(_, items)| items.last_mut()) {
            last.push(' ');
            last.push_str(text);
        }
    }

    fn end_paragraph(&mut self) {
        if !self.paragraph.is_empty() {
            let text = self.paragraph.join(" ");
            self.paragraph.clear();
            self.blocks.push(Block::Paragraph {
                inline: inline(&text),
            });
        }
    }

    fn end_list(&mut self) {
        if let Some((ordered, items)) = self.list.take() {
            self.blocks.push(Block::List {
                ordered,
                items: items
                    .iter()
                    .map(|text| ListItem {
                        inline: inline(text),
                        children: Vec::new(),
                    })
                    .collect(),
            });
        }
    }

    fn end_block(&mut self) {
        self.end_paragraph();
        self.end_list();
    }
}

/// `---`, `***` or `___`, three or more, spaces allowed between.
fn is_rule(text: &str) -> bool {
    let marks: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    marks.len() >= 3 && ['-', '*', '_'].iter().any(|m| marks.iter().all(|c| c == m))
}

/// `#` to `######` followed by a space.
fn heading(text: &str) -> Option<(u8, &str)> {
    let hashes = text.chars().take_while(|c| *c == '#').count();
    let rest = text.get(hashes..)?;
    if !(1..=6).contains(&hashes) || !rest.starts_with(' ') {
        return None;
    }
    Some((hashes as u8, rest.trim()))
}

/// `- `, `* `, `+ ` for a bullet, `1. ` or `1) ` for a numbered item.
fn list_item(text: &str) -> Option<(bool, &str)> {
    for bullet in ["- ", "* ", "+ "] {
        if let Some(rest) = text.strip_prefix(bullet) {
            return Some((false, rest.trim()));
        }
    }
    let digits = text.chars().take_while(|c| c.is_ascii_digit()).count();
    let rest = text.get(digits..)?;
    if digits == 0 {
        return None;
    }
    rest.strip_prefix(". ")
        .or_else(|| rest.strip_prefix(") "))
        .map(|item| (true, item.trim()))
}

/// A line's runs: `**…**` is bold, everything else plain, backticks dropped from both.
fn inline(text: &str) -> Vec<Inline> {
    let mut runs = Vec::new();
    let mut plain = String::new();
    let mut rest = text;
    while let Some(ch) = rest.chars().next() {
        if let Some(after) = rest.strip_prefix("**") {
            match after.find("**") {
                Some(end) if end > 0 => {
                    push(&mut runs, &plain, Style::Plain);
                    plain.clear();
                    push(&mut runs, &after[..end], Style::Bold);
                    rest = &after[end + 2..];
                }
                // Never closed, or closed on nothing: the stars are the text.
                _ => {
                    plain.push_str("**");
                    rest = after;
                }
            }
            continue;
        }
        plain.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    push(&mut runs, &plain, Style::Plain);
    runs
}

/// Appends a run, merged into the one before when the style is the same.
fn push(runs: &mut Vec<Inline>, text: &str, style: Style) {
    let text = without_code_marks(text);
    if text.is_empty() {
        return;
    }
    if let Some(Inline::Text {
        text: last,
        style: last_style,
    }) = runs.last_mut()
    {
        if *last_style == style {
            last.push_str(&text);
            return;
        }
    }
    runs.push(Inline::Text { text, style });
}

/// `` `code` `` keeps its words and loses its backticks. A backtick with no partner is kept,
/// because it was never a mark.
fn without_code_marks(text: &str) -> String {
    let parts: Vec<&str> = text.split('`').collect();
    let marks = parts.len() - 1;
    if marks.is_multiple_of(2) {
        return parts.concat();
    }
    let (paired, unpaired) = parts.split_at(parts.len() - 1);
    format!("{}`{}", paired.concat(), unpaired.concat())
}
