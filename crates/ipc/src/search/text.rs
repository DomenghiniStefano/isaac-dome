//! The text side of search: how a page flattens into one string per section, how a query
//! splits into words, and how a fragment is cut around a match.

use wiki::{Block, Entry, Inline, SectionKind};

/// One page as the search reads it: the title, and the text of each section in the order the
/// page has them.
pub(super) struct WikiDoc {
    pub(super) title: String,
    pub(super) sections: Vec<(SectionKind, String)>,
}

/// The characters kept before and after a match in a section fragment: enough to read the
/// sentence, counted in **characters**, so a cut never lands inside one.
const BEFORE: usize = 60;
const AFTER: usize = 90;

/// ASCII fold. Every ASCII byte maps to one byte and everything else is left alone, so an
/// offset in the folded string is an offset in the original: the fragment cuts the page's own
/// text without a second search.
pub(super) fn fold(s: &str) -> String {
    s.to_ascii_lowercase()
}

pub(super) fn words(query: &str) -> Vec<String> {
    fold(query).split_whitespace().map(str::to_string).collect()
}

pub(super) fn contains_all(folded: &str, words: &[String]) -> bool {
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

/// The text before the match, the match, and the text after it, when `text` holds every word.
pub(super) fn fragment(text: &str, words: &[String]) -> Option<(String, String, String)> {
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

pub(super) fn doc(entry: &Entry) -> WikiDoc {
    WikiDoc {
        title: entry.title.clone(),
        sections: entry
            .sections
            .iter()
            .map(|s| (s.kind, joined(s.blocks.iter().flat_map(block_pieces))))
            .collect(),
    }
}

/// The pieces as one line: each trimmed, the empty ones dropped, one space between the rest.
fn joined<'a>(pieces: impl Iterator<Item = &'a str>) -> String {
    pieces
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn inline_pieces(inline: &[Inline]) -> Vec<&str> {
    inline
        .iter()
        .flat_map(|i| match i {
            Inline::Text { text, .. } => vec![text.as_str()],
            Inline::Ref { label, .. } | Inline::Concept { label, .. } => vec![label.as_str()],
            // The words are on the page: an edition inline is unwrapped, not skipped.
            Inline::Edition { inline, .. } => inline_pieces(inline),
        })
        .collect()
}

fn block_pieces(block: &Block) -> Vec<&str> {
    match block {
        Block::Paragraph { inline } | Block::Heading { inline, .. } => inline_pieces(inline),
        Block::List { items, .. } => items
            .iter()
            .flat_map(|item| {
                inline_pieces(&item.inline)
                    .into_iter()
                    .chain(item.children.iter().flat_map(block_pieces))
            })
            .collect(),
        Block::Table { header, rows } => header
            .iter()
            .chain(rows.iter().flatten())
            .flat_map(|cell| inline_pieces(cell))
            .collect(),
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
