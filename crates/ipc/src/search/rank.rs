//! How a document matches a query, and where its hit ranks.

use wiki::Target;

use super::text::{contains_all, fold, fragment};
use super::{Doc, ProgressMark, SearchIndex, SearchMatch};

/// How strongly a document matched, strongest first: the declaration order **is** the ranking.
/// The first four are on a name — the title or its alias; the last two on the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Tier {
    /// The name is the query.
    Exact,
    /// The name starts with the query.
    Prefix,
    /// A word of the name starts with the query.
    WordPrefix,
    /// The name holds every word, anywhere.
    Contains,
    /// The achievement's own wording of what to do holds every word.
    Condition,
    /// A section of the page holds every word.
    Section,
}

/// One document that matched, with everything its place in the answer is decided by.
pub(super) struct Ranked {
    pub(super) tier: Tier,
    pub(super) target: Target,
    pub(super) doc: Doc,
    pub(super) matched: SearchMatch,
    pub(super) progress: ProgressMark,
    folded_title: String,
}

impl Ranked {
    /// The document's hit, when any field of it holds every word.
    pub(super) fn new(
        index: &SearchIndex,
        target: Target,
        doc: Doc,
        words: &[String],
        folded_query: &str,
        progress: ProgressMark,
    ) -> Option<Ranked> {
        let (tier, matched) = best_field(index, &target, &doc, words, folded_query)?;
        Some(Ranked {
            tier,
            folded_title: fold(&doc.title),
            target,
            doc,
            matched,
            progress,
        })
    }

    /// Tier, then the profile, then the name, and last the target itself: the order is total,
    /// so a test can pin it and two runs can never disagree.
    pub(super) fn key(&self) -> (Tier, u8, &str, &Target) {
        (
            self.tier,
            progress_rank(self.progress),
            &self.folded_title,
            &self.target,
        )
    }
}

/// On a name: equal, prefix, a word's prefix, or merely containing the words.
fn title_tier(folded_title: &str, folded_query: &str) -> Tier {
    if folded_title == folded_query {
        Tier::Exact
    } else if folded_title.starts_with(folded_query) {
        Tier::Prefix
    } else if folded_title
        .split_whitespace()
        .any(|w| w.starts_with(folded_query))
    {
        Tier::WordPrefix
    } else {
        Tier::Contains
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
) -> Option<(Tier, SearchMatch)> {
    name_match(&doc.title, words, folded_query)
        .or_else(|| {
            doc.alias
                .as_deref()
                .and_then(|alias| name_match(alias, words, folded_query))
        })
        .or_else(|| condition_match(doc.condition.as_deref()?, words))
        .or_else(|| section_match(index, target, words))
}

fn name_match(name: &str, words: &[String], folded_query: &str) -> Option<(Tier, SearchMatch)> {
    let folded = fold(name);
    contains_all(&folded, words).then(|| (title_tier(&folded, folded_query), SearchMatch::Title))
}

fn condition_match(condition: &str, words: &[String]) -> Option<(Tier, SearchMatch)> {
    contains_all(&fold(condition), words).then(|| {
        (
            Tier::Condition,
            SearchMatch::Condition {
                text: condition.to_string(),
            },
        )
    })
}

fn section_match(
    index: &SearchIndex,
    target: &Target,
    words: &[String],
) -> Option<(Tier, SearchMatch)> {
    index
        .pages
        .get(target)?
        .sections
        .iter()
        .find_map(|(kind, text)| {
            let (before, matched, after) = fragment(text, words)?;
            Some((
                Tier::Section,
                SearchMatch::Section {
                    section: *kind,
                    before,
                    matched,
                    after,
                },
            ))
        })
}
