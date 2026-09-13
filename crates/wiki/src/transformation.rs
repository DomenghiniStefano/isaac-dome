//! The two facts a transformation page states beyond its prose: how many items it takes,
//! and which items count.
//!
//! Both are read from the transformation's own page. The item side cannot answer: since
//! `{{transformation contribution|X}}` resolves exactly like `{{tf|X}}`, an item page saying
//! "counts toward Guppy" is indistinguishable from one merely naming Guppy, and inverting
//! that index gives Guppy 28 pages and Conjoined 44 against real sets of about seven.

use crate::infobox::RawInfobox;
use crate::inline::parse_inline;
use crate::resolver::Resolver;
use crate::{Diagnostics, Inline, Target};

/// The sentence that states the count. The infobox's `requirement` is **not** a source: all
/// sixteen rows of the Cargo table hold the identical string `three items from this set`, a
/// template default that would hand every transformation the same number — including Adult,
/// which is about pills, and whose page has no such parameter at all.
const PICK_UP: &str = "Pick up";

/// The templates that state the set. Two shapes, because a page that splits its list by
/// edition writes one `rows` per edition under a shared header.
const TABLES: [&str; 4] = [
    "{{collectible table",
    "{{collectible rows",
    "{{trinket table",
    "{{trinket rows",
];

/// `"Pick up 3 [[item]]s or [[trinket]]s from the following list"` → 3, and Necromancer's
/// `"Pick up each of the 3 [[item]]s and [[trinket]]s"` → 3 as well: the digit is the first
/// one in that line, not the character straight after the phrase.
///
/// Narrow on purpose. Adult's page says "turns Isaac into an adult upon taking three
/// [[Puberty]] pills" — a different claim about a different kind of thing — and widening the
/// pattern until it matched would mean inventing a set of collectibles for an item that has
/// none. `None` is the honest answer there, and it must never become three: three is exactly
/// what a default would produce, so a wrong default here would be invisible.
pub fn requires(text: &str) -> Option<u32> {
    let at = text.find(PICK_UP)?;
    // The rest of that line only: a digit further down the page belongs to another sentence.
    let line = text[at..].lines().next()?;
    let digits: String = line
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

/// Every item and trinket target of an inline run, in order, edition wrappers included: a
/// list qualified by `dlc =` still names items that count, in that edition.
fn refs(inline: &[Inline], out: &mut Vec<Target>) {
    for i in inline {
        match i {
            Inline::Ref { target, .. } => match target {
                Target::Item { .. } | Target::Trinket { .. } => out.push(target.clone()),
                Target::Character { .. }
                | Target::Achievement { .. }
                | Target::Challenge { .. }
                | Target::Entity { .. }
                | Target::Transformation { .. }
                | Target::Stage { .. }
                | Target::Room { .. }
                | Target::Pickup { .. } => {}
            },
            Inline::Edition { inline, .. } => refs(inline, out),
            Inline::Text { .. } | Inline::Concept { .. } => {}
        }
    }
}

/// The union of the two sources on the page, deduplicated, the body's tables first because
/// they are the more complete of the two.
///
/// A union rather than a choice because each source fails on a different page and in the
/// opposite direction: the infobox loses Guppy's trinket, and ten of the sixteen pages have
/// an infobox list the Cargo table renders as empty. The body is read **only** from the
/// lines that carry one of `TABLES`, never from the page's prose: an Effects section naming
/// an item is a mention, and mentions are what make the item-side index useless.
/// Whether the two sources agreed is not returned: it is recorded in
/// `Diagnostics::transformation_sources_disagree`, which is where the snapshot's report
/// reads it and where a test can see it. Returning it as well would be a second copy of one
/// fact, free to drift from the first.
pub fn contributors(ib: &RawInfobox, text: &str, r: &Resolver, d: &mut Diagnostics) -> Vec<Target> {
    let mut body = Vec::new();
    for line in text
        .lines()
        .filter(|l| TABLES.iter().any(|t| l.contains(t)))
    {
        refs(&parse_inline(line, r, d), &mut body);
    }
    let mut from_infobox = Vec::new();
    let items = ib.params.get("items").map(String::as_str).unwrap_or("");
    refs(&parse_inline(items, r, d), &mut from_infobox);

    let disagree = body.iter().any(|t| !from_infobox.contains(t))
        || from_infobox.iter().any(|t| !body.contains(t));
    if disagree {
        d.transformation_sources_disagree += 1;
    }
    let mut targets: Vec<Target> = Vec::new();
    for t in body.into_iter().chain(from_infobox) {
        if !targets.contains(&t) {
            targets.push(t);
        }
    }
    targets
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::fixtures::test_resolver;
    use std::collections::BTreeMap;

    fn ib(pairs: &[(&str, &str)]) -> RawInfobox {
        RawInfobox {
            name: "infobox transformation".into(),
            params: pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<BTreeMap<_, _>>(),
        }
    }

    /// Guppy's and Super Bum's real sentences, and Necromancer's, which puts four words
    /// between the phrase and the number.
    #[test]
    fn the_count_is_the_first_digit_of_the_pick_up_line() {
        assert_eq!(
            requires("Pick up 3 [[item]]s or [[trinket]]s from the following list"),
            Some(3)
        );
        assert_eq!(
            requires("Pick up 3 [[item]]s from the following list."),
            Some(3)
        );
        assert_eq!(
            requires("Pick up each of the 3 [[item]]s and [[trinket]]s from the following list."),
            Some(3)
        );
    }

    /// Adult's real sentence, which the pattern deliberately does not reach, and a page whose
    /// count would have to come from a line other than the one the phrase is on.
    #[test]
    fn a_page_that_does_not_state_it_reads_none_and_never_three() {
        assert_eq!(
            requires("turns Isaac into an adult upon taking three [[Puberty]] pills"),
            None
        );
        assert_eq!(requires(""), None);
        assert_eq!(requires("three items from this set"), None);
        assert_eq!(requires("Pick up the items.\nThere are 3 of them."), None);
    }

    /// The body's tables carry what the infobox omits, and the union keeps both. The
    /// disagreement is recorded rather than resolved.
    #[test]
    fn contributors_are_the_union_of_the_infobox_and_the_body_tables() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let c = contributors(
            &ib(&[("items", "{{i|Breakfast}}")]),
            "== Collectibles ==\n{{collectible table | Breakfast }}\n{{trinket table | Swallowed Penny }}",
            &r,
            &mut d,
        );
        assert_eq!(c.len(), 2, "{c:?}");
        assert!(matches!(c[1], Target::Trinket { id: 1 }));

        assert_eq!(d.transformation_sources_disagree, 1);
    }

    /// The same set on both sides is not a disagreement: the counter has to be able to say
    /// "nothing to report" as well as "something".
    #[test]
    fn agreeing_sources_are_not_a_disagreement() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let c = contributors(
            &ib(&[("items", "{{i|Breakfast}}")]),
            "{{collectible table | Breakfast }}",
            &r,
            &mut d,
        );
        assert_eq!(c.len(), 1);

        assert_eq!(d.transformation_sources_disagree, 0);
    }

    /// The page's prose is not a source. An Effects section naming an item is a mention, and
    /// reading mentions is exactly what makes the item-side index unusable.
    #[test]
    fn prose_outside_the_tables_contributes_nothing() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let c = contributors(
            &ib(&[]),
            "== Effects ==\n* Works well with {{i|Breakfast}}.\n",
            &r,
            &mut d,
        );
        assert!(c.is_empty(), "{c:?}");
    }
}
