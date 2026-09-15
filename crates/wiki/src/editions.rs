//! The wiki's `{{dlc|…}}` codes, and the set of editions each one names.
//!
//! **A code is not a set of editions: it is a run of transitions over them.** `r` is not
//! "Repentance", it is "added in Repentance", so it names Repentance *and* Repentance+;
//! `nr` is "removed in Repentance", so it names the three before it. Reading one as a set
//! is what shipped 1078 of 1083 infobox values too narrow, and `n` as Rebirth, until
//! 2026-09-15.
//!
//! The dictionary below is transcribed from the wiki's own `Template:Dlcset`
//! (<https://bindingofisaacrebirth.wiki.gg/wiki/Template:Dlcset>, read 2026-09-15), which
//! is a `#switch` from the thirty codes to a five-bit mask, and which states the bit order
//! in a comment of its own:
//!
//! ```text
//! <!-- <Repentance †><Repentance><Afterbirth †><Afterbirth><Rebirth> -->
//!  |  1 | na       =  1 <!-- 00001 -->
//!  | 24 | r        = 24 <!-- 11000 -->
//!  | 31 | n | x |  = 31 <!-- 11111 -->
//!  | 0 <!-- invalid string! -->
//! ```
//!
//! So Rebirth is bit 0 and Repentance+ is bit 4; `n`, `x` and the empty string are the
//! code for "no restriction"; and anything outside the thirty is invalid to the wiki
//! itself, which is why it is `None` here rather than a guess.
//!
//! `Template:Dlc/format` names each row in prose, and that is where the meaning of `n` is
//! written down: row 7 is `nr`, `alt=(except in Repentance and Repentance+)`, titled
//! *Removed in Repentance*.

use crate::{Diagnostics, Dlc};

/// A set of the five editions. Not on the wire — `Vec<Dlc>` is, in release order — but the
/// intersection a page's context performs is a bit operation, and a `Vec` is the wrong
/// shape to do it on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Editions(u8);

/// The five editions, low bit first, exactly as `Template:Dlcset`'s comment orders them.
const BITS: [(u8, Dlc); 5] = [
    (1, Dlc::Rebirth),
    (2, Dlc::Afterbirth),
    (4, Dlc::AfterbirthPlus),
    (8, Dlc::Repentance),
    (16, Dlc::RepentancePlus),
];

/// `Template:Dlcset`, transcribed. Thirty codes and the mask each one names; `n`, `x` and
/// the empty string are `ALL` and are handled by `parse` rather than listed here, because
/// they are the absence of a restriction and not a range.
const CODES: [(&str, u8); 30] = [
    ("na", 1),
    ("ana+", 2),
    ("na+", 3),
    ("a+nr", 4),
    ("naa+nr", 5),
    ("anr", 6),
    ("nr", 7),
    ("rnr+", 8),
    ("narnr+", 9),
    ("ana+rnr+", 10),
    ("na+rnr+", 11),
    ("a+nr+", 12),
    ("naa+nr+", 13),
    ("anr+", 14),
    ("nr+", 15),
    ("r+", 16),
    ("nar+", 17),
    ("ana+r+", 18),
    ("na+r+", 19),
    ("a+nrr+", 20),
    ("naa+nrr+", 21),
    ("anrr+", 22),
    ("nrr+", 23),
    ("r", 24),
    ("nar", 25),
    ("ana+r", 26),
    ("na+r", 27),
    ("a+", 28),
    ("naa+", 29),
    ("a", 30),
];

impl Editions {
    /// Every edition: what a page with no `dlc` parameter says, and what `{{dlc|n}}` says.
    pub const ALL: Editions = Editions(31);
    /// No edition. Only produced by an intersection — no code names it.
    pub const NONE: Editions = Editions(0);

    /// One `{{dlc|…}}` code, or an infobox's `dlc` parameter: the same grammar, because
    /// the infobox hands its parameter to `{{section dlc}}` → `{{page dlc}}` → `{{dlcset}}`.
    /// `None` is the wiki's own `0 <!-- invalid string! -->`, and the caller counts it.
    pub fn parse(code: &str) -> Option<Editions> {
        let code = code.trim();
        if code.is_empty() || code == "n" || code == "x" {
            return Some(Editions::ALL);
        }
        CODES
            .iter()
            .find(|(c, _)| *c == code)
            .map(|(_, mask)| Editions(*mask))
    }

    /// The editions in release order, which is `Dlc`'s declaration order.
    pub fn list(self) -> Vec<Dlc> {
        BITS.iter()
            .filter(|(bit, _)| self.0 & bit != 0)
            .map(|(_, dlc)| *dlc)
            .collect()
    }

    /// The set a list names. An empty list is `ALL`: it is what an absent `dlc` parameter
    /// leaves behind, and an absent parameter is the wiki's "no restriction".
    pub fn of(list: &[Dlc]) -> Editions {
        if list.is_empty() {
            return Editions::ALL;
        }
        Editions(
            BITS.iter()
                .filter(|(_, dlc)| list.contains(dlc))
                .map(|(bit, _)| bit)
                .sum(),
        )
    }

    /// What a span means inside a page: the wiki narrows one by the other
    /// (`{{context test}}`), so a line marked `{{dlc|nr+}}` on an item that exists only
    /// from Repentance names Repentance alone, not the four editions the code lists.
    pub fn intersect(self, other: Editions) -> Editions {
        Editions(self.0 & other.0)
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn is_all(self) -> bool {
        self == Editions::ALL
    }
}

/// A code, parsed, with an unreadable one counted instead of dropped. The counter is the
/// only thing that would make a thirty-first code visible instead of shipped.
pub fn parse_code(code: &str, d: &mut Diagnostics) -> Editions {
    match Editions::parse(code) {
        Some(e) => e,
        None => {
            d.unknown_dlc_code(code.trim());
            Editions::ALL
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The thirty rows were copied by hand out of a `#switch`, and a transcription is the
    /// one kind of table a typo hides in. So they are re-derived here from what the codes
    /// *mean* — a run of transitions, `n` turning the following edition and everything
    /// after it off, a bare code turning it and everything after it on — and the two have
    /// to agree row for row. The grammar is the explanation; `CODES` stays the parser,
    /// because a code that tokenizes and is not in the wiki's switch (`ra`, `rn`) is
    /// invalid there and must be invalid here.
    #[test]
    fn every_transcribed_row_is_the_run_of_transitions_its_code_spells() {
        for (code, mask) in CODES {
            assert_eq!(
                Editions(mask),
                by_transitions(code).unwrap_or_else(|| panic!("{code} does not tokenize")),
                "{code} = {mask}",
            );
        }
    }

    /// `n?` then one of `a+`, `a`, `r+`, `r`, longest first: `na+r` is `na+` then `r`,
    /// while `nar+` is `na` then `r+`. Rebirth is never named, because it is where the
    /// series starts: what holds before the first transition is simply its opposite, which
    /// is why `na` ("removed in Afterbirth") means Rebirth alone and `a` ("added in
    /// Afterbirth") does not include it.
    fn by_transitions(code: &str) -> Option<Editions> {
        let mut transitions = Vec::new();
        let mut rest = code;
        while !rest.is_empty() {
            let (removes, tail) = match rest.strip_prefix('n') {
                Some(tail) => (true, tail),
                None => (false, rest),
            };
            let (at, tail) = ["a+", "a", "r+", "r"]
                .iter()
                .find_map(|c| tail.strip_prefix(c).map(|t| (index_of(c), t)))?;
            transitions.push((at?, removes));
            rest = tail;
        }
        let mut mask = 0u8;
        let mut present = transitions.first().is_some_and(|(_, removes)| *removes);
        let mut at = 0usize;
        for (next, removes) in transitions {
            if present {
                mask |= BITS[at..next].iter().map(|(b, _)| b).sum::<u8>();
            }
            at = next;
            present = !removes;
        }
        if present {
            mask |= BITS[at..].iter().map(|(b, _)| b).sum::<u8>();
        }
        Some(Editions(mask))
    }

    fn index_of(code: &str) -> Option<usize> {
        let dlc = match code {
            "a" => Dlc::Afterbirth,
            "a+" => Dlc::AfterbirthPlus,
            "r" => Dlc::Repentance,
            "r+" => Dlc::RepentancePlus,
            _ => return None,
        };
        BITS.iter().position(|(_, d)| *d == dlc)
    }

    /// The four rows `Template:Dlc/format` states in prose, which is where `n`'s meaning
    /// is written: row 24 is *Added in Repentance*, "(in Repentance and Repentance+)", and
    /// row 7 is *Removed in Repentance*, "(except in Repentance and Repentance+)".
    #[test]
    fn a_code_names_a_range_and_not_one_edition() {
        assert_eq!(
            Editions::parse("r").map(Editions::list),
            Some(vec![Dlc::Repentance, Dlc::RepentancePlus])
        );
        assert_eq!(
            Editions::parse("nr").map(Editions::list),
            Some(vec![Dlc::Rebirth, Dlc::Afterbirth, Dlc::AfterbirthPlus])
        );
        // Tonsil's collectible: added in Afterbirth †, removed in Repentance. One edition,
        // and the only page in the snapshot that says so.
        assert_eq!(
            Editions::parse("a+nr").map(Editions::list),
            Some(vec![Dlc::AfterbirthPlus])
        );
        assert_eq!(
            Editions::parse("na").map(Editions::list),
            Some(vec![Dlc::Rebirth])
        );
    }

    /// `n` alone is row 31, beside `x` and the empty string: no restriction at all. Read
    /// as a code for Rebirth — which is what `Dlc::from_code` did — it said the opposite
    /// of what it means, on the one value that names every edition.
    #[test]
    fn the_bare_n_is_no_restriction_and_not_rebirth() {
        assert_eq!(Editions::parse("n"), Some(Editions::ALL));
        assert_eq!(Editions::parse("x"), Some(Editions::ALL));
        assert_eq!(Editions::parse(""), Some(Editions::ALL));
        assert_eq!(Editions::ALL.list().len(), 5);
    }

    /// Outside the thirty the wiki answers `0 <!-- invalid string! -->`. A parser that
    /// tokenized instead would accept `ra` and `rn` and invent a mask for them.
    #[test]
    fn a_code_outside_the_switch_is_not_read_at_all() {
        assert_eq!(Editions::parse("zz"), None);
        assert_eq!(Editions::parse("ra"), None);
        assert_eq!(Editions::parse("rn"), None);
    }

    /// Abyss: the item exists from Repentance (`dlc = r`), and carries a line marked
    /// `{{dlc|nr+}}`. On its own that code names four editions, three of which the item
    /// does not exist in; inside the page it names Repentance, which is what the line says
    /// — removed in Repentance+.
    #[test]
    fn a_span_is_narrowed_by_the_page_it_sits_on() {
        let page = Editions::parse("r").expect("r is a code");
        let span = Editions::parse("nr+").expect("nr+ is a code");
        assert_eq!(span.intersect(page).list(), vec![Dlc::Repentance]);
    }

    /// An absent parameter narrows nothing — which is the common case, 492 of the 1113
    /// pages in the snapshot.
    #[test]
    fn a_page_that_declares_nothing_narrows_nothing() {
        let span = Editions::parse("nr").expect("nr is a code");
        assert_eq!(span.intersect(Editions::of(&[])), span);
    }

    /// An unreadable code is counted and reads as no restriction: the words stay, the
    /// badge does not appear, and `meta` says a code was dropped. Until 2026-09-15 it read
    /// as *no edition at all*, and a span valid in no edition draws nothing either — so
    /// 1690 spans in the shipped dataset said nothing, and nothing said they had.
    #[test]
    fn an_unreadable_code_is_counted_and_restricts_nothing() {
        let mut d = Diagnostics::default();
        assert_eq!(parse_code("zz", &mut d), Editions::ALL);
        assert_eq!(d.unknown_dlc_codes.get("zz"), Some(&1));
    }
}
