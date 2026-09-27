//! `{{hearts|…}}` and `{{heart|…}}`: the starting-health breakdown on a character or
//! challenge's infobox. Both templates carry only named parameters (`red=3`) or a bare
//! positional one (`red`, meaning one), which the unknown-template fallback cannot read at
//! all — it keeps `args.first()`, and a call with nothing but named parameters has no
//! positional argument to keep. Measured 2026-09-26: 30 of 34 character pages and 44 of 45
//! challenges had an empty `health` for exactly this reason.

use crate::template::Template;
use crate::{Diagnostics, Inline};

use super::Out;

/// The heart-type parameter names the template accepts, each the container the game itself
/// calls it. `red`, `black`, `soul`, `bone`, `coin` and `unknown` are what `dataset/raw/`
/// measures on a `health` field (2026-09-26); `rotten`, `eternal`, `broken` and `golden` are
/// the wiki's own vocabulary for the same template used elsewhere on the site, kept here so a
/// page that starts using one is read on sight instead of landing in the unknown counter the
/// day it does. `unknown` is not a gap in this list: it is the wiki's own word for "of an
/// undetermined type" — Lazarus Risen keeps whatever non-red hearts Lazarus died holding, and
/// the page cannot say which they are.
const HEART_TYPES: &[&str] = &[
    "red", "soul", "black", "bone", "rotten", "eternal", "broken", "coin", "golden", "unknown",
];

/// The wiki's own page for every heart type: its body reads `Health#Red Heart Containers`,
/// `Health#Black Hearts`, `Health#Bone Hearts`… — one page, an anchor per type — and no
/// specific type has a standalone page the corpus links to instead. Concept resolution already
/// drops a link's `#fragment` (`link` in `inline.rs`), so a type-specific anchor would be lost
/// either way; naming the shared page is what the corpus's own wikilinks actually point at.
const HEALTH_PAGE: &str = "Health";

/// A type name against the closed list: kept and counted when it isn't on it, never dropped —
/// the count is still real information even for a type this parser doesn't recognize yet.
fn checked_type(kind: &str, d: &mut Diagnostics) -> String {
    let key = kind.trim().to_lowercase();
    if !HEART_TYPES.contains(&key.as_str()) {
        d.unknown_heart_type(&key);
    }
    key
}

/// One `{{hearts|…}}` segment as a reader would say it: "3 red hearts".
fn hearts_label(kind: &str, count: u32, d: &mut Diagnostics) -> String {
    let key = checked_type(kind, d);
    let word = if count == 1 { "heart" } else { "hearts" };
    format!("{count} {key} {word}")
}

/// One `{{heart|…}}`: always a single heart, empty (an unfilled container, drawn for a
/// character's *maximum* rather than current health) or not.
fn heart_label(kind: &str, empty: bool, d: &mut Diagnostics) -> String {
    let key = checked_type(kind, d);
    if empty {
        format!("1 empty {key} heart container")
    } else {
        format!("1 {key} heart")
    }
}

fn push_segment(out: &mut Out, first: &mut bool, label: String) {
    if !*first {
        out.buf.push_str(", ");
    }
    *first = false;
    out.push(Inline::Concept {
        page: HEALTH_PAGE.to_string(),
        label,
    });
}

/// `{{hearts|red|black=2}}`: a bare positional argument is one heart of that type (Lilith's
/// "one red heart container and two Black Hearts" writes the red one this way); a named one is
/// `type=count`. Positional arguments read first, then the named ones in their own (alphabetic)
/// order — the template call gives no other order to keep once a name enters `Template::named`.
pub(super) fn hearts(t: &Template, out: &mut Out, d: &mut Diagnostics) {
    let mut first = true;
    for kind in t.args.iter().map(|a| a.trim()).filter(|a| !a.is_empty()) {
        push_segment(out, &mut first, hearts_label(kind, 1, d));
    }
    for (kind, count) in &t.named {
        let count = count.trim().parse().unwrap_or(1);
        push_segment(out, &mut first, hearts_label(kind, count, d));
    }
}

/// `{{heart|red}}` or `{{heart|red|empty}}`: one heart icon, the second positional argument
/// naming the "empty container" variant used to draw a maximum-health outline (Have a Heart's
/// challenge health is a filled heart followed by eleven empty ones).
pub(super) fn heart(t: &Template, arg: &str, out: &mut Out, d: &mut Diagnostics) {
    let empty = t
        .args
        .get(1)
        .is_some_and(|a| a.trim().eq_ignore_ascii_case("empty"));
    out.push(Inline::Concept {
        page: HEALTH_PAGE.to_string(),
        label: heart_label(arg, empty, d),
    });
}

// Tests extract one variant and panic on the rest: the wildcard is the assertion.
#[allow(clippy::wildcard_enum_match_arm)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inline::parse_inline;
    use crate::resolver::fixtures::test_resolver;
    use crate::Dlc;

    fn p(s: &str) -> (Vec<Inline>, Diagnostics) {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        (parse_inline(s, &r, &mut d), d)
    }

    /// Every `Health` concept's label, at any depth: an edition wrapper around a second
    /// `{{hearts|…}}` call must not hide it from this test any more than it hides it from a
    /// reader.
    fn labels(v: &[Inline]) -> Vec<&str> {
        v.iter()
            .flat_map(|i| match i {
                Inline::Concept { page, label } if page == "Health" => vec![label.as_str()],
                Inline::Edition { inline, .. } => labels(inline),
                Inline::Concept { .. } | Inline::Text { .. } | Inline::Ref { .. } => vec![],
            })
            .collect()
    }

    /// Magdalene's page: `{{hearts|red=4}}`. A single named parameter used to vanish whole,
    /// because the unknown-template fallback keeps only `args.first()` and a call with no
    /// positional argument has none.
    #[test]
    fn a_single_named_count_is_read() {
        let (v, d) = p("{{hearts|red=4}}");
        assert_eq!(labels(&v), vec!["4 red hearts"]);
        assert!(
            d.unknown_heart_types.is_empty(),
            "{:?}",
            d.unknown_heart_types
        );
        assert!(d.unknown_templates.is_empty(), "{:?}", d.unknown_templates);
    }

    /// Lilith: `{{hearts|red|black=2}}` — "one red heart container and two Black Hearts". The
    /// bare positional argument is a count of one, read before the named ones.
    #[test]
    fn a_bare_positional_argument_is_one_heart_before_the_named_ones() {
        let (v, _) = p("{{hearts|red|black=2}}");
        assert_eq!(labels(&v), vec!["1 red heart", "2 black hearts"]);
    }

    /// Cat Got Your Tongue: `{{hearts|red=3|black=3|soul=3}}`, all three types on one call.
    #[test]
    fn every_named_type_on_one_call_is_kept() {
        let (v, _) = p("{{hearts|red=3|black=3|soul=3}}");
        assert_eq!(
            labels(&v),
            vec!["3 black hearts", "3 red hearts", "3 soul hearts"]
        );
    }

    /// Judas: `{{heart|red}}`, one heart with no modifier.
    #[test]
    fn a_single_heart_reads_as_one() {
        let (v, _) = p("{{heart|red}}");
        assert_eq!(labels(&v), vec!["1 red heart"]);
    }

    /// Have a Heart's challenge health: a filled heart, then eleven empty ones.
    #[test]
    fn an_empty_heart_names_the_container_not_the_content() {
        let (v, _) = p("{{heart|red}}{{heart|red|empty}}");
        assert_eq!(
            labels(&v),
            vec!["1 red heart", "1 empty red heart container"]
        );
    }

    /// Lazarus Risen: `{{hearts|unknown=2}}`. `unknown` is the wiki's own word for a heart
    /// type the page cannot name, not a gap in the closed list — it must not be counted.
    #[test]
    fn the_unknown_heart_type_is_not_a_diagnostic() {
        let (v, d) = p("{{hearts|unknown=2}}");
        assert_eq!(labels(&v), vec!["2 unknown hearts"]);
        assert!(
            d.unknown_heart_types.is_empty(),
            "{:?}",
            d.unknown_heart_types
        );
    }

    /// A type name outside the closed list is kept — the count is still real — and counted,
    /// the same shape `unknown_entities` already gives `&name;`.
    #[test]
    fn a_type_outside_the_closed_list_is_kept_and_counted() {
        let (v, d) = p("{{hearts|glass=1}}");
        assert_eq!(labels(&v), vec!["1 glass heart"]);
        assert_eq!(d.unknown_heart_types.get("glass"), Some(&1));
    }

    /// Keeper's real markup: a base heart count, then a second count wrapped by `{{dlc|r}}`
    /// for the edition that unlocked a third one. The edition wrapper is a plain sibling
    /// template — `hearts` itself needs to know nothing about it — and the second call's
    /// label still comes through inside the `Edition` node it opens.
    #[test]
    fn a_second_call_inside_an_edition_wrapper_keeps_both() {
        let (v, _) = p("{{hearts|coin=2}} {{dlc|r}}{{hearts|coin=3}}");
        assert_eq!(labels(&v), vec!["2 coin hearts", "3 coin hearts"]);
        assert!(
            v.iter().any(|i| matches!(i, Inline::Edition { only, .. } if only == &vec![Dlc::Repentance, Dlc::RepentancePlus])),
            "{v:?}"
        );
    }
}
