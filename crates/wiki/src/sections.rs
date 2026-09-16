//! A page = preamble + level-2 sections. Subsections stay in the body.

use crate::SectionKind;

/// A level-2 section as it is in the wikitext: raw title and body, subsections included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSection {
    pub title: String,
    pub body: String,
}

/// Splits the page into preamble (everything before the first `== … ==`) and level-2
/// sections. Level-3 headings and beyond stay in the body of the section that contains them.
pub fn split_page(text: &str) -> (String, Vec<RawSection>) {
    let mut pre = String::new();
    let mut secs: Vec<RawSection> = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        let is_l2 =
            t.starts_with("==") && !t.starts_with("===") && t.ends_with("==") && t.len() > 4;
        if is_l2 {
            let title = t.trim_matches('=').trim().to_string();
            secs.push(RawSection {
                title,
                body: String::new(),
            });
            continue;
        }
        match secs.last_mut() {
            Some(s) => {
                s.body.push_str(line);
                s.body.push('\n');
            }
            None => {
                pre.push_str(line);
                pre.push('\n');
            }
        }
    }
    (pre, secs)
}

/// Templates that render a marker or nothing at all, so a heading means the same without them.
/// **Measured, not guessed** (B54): across the 1113 pages of the snapshot, exactly six template
/// names appear in a heading — these four, plus `{{s|…}}` and `{{c|…}}`, which are a stage's and
/// a character's *name* and are the whole point of the distinction below.
const MARKER_TEMPLATES: [&str; 5] = ["dlc", "dlc+", "dlc-", "unlockable", "anchor"];

/// A comparable title: link brackets gone, marker templates gone, a **content** template replaced
/// by the text it renders, lowercased, whitespace collapsed.
///
/// **Public because the decision is made on this and `discardedSections` is keyed on the raw
/// title** (B54): anything reading that counter and not normalizing first sees more distinct
/// titles than there are — `{{dlc|nr}} Gallery` and `Gallery` are one title, and so are the two
/// capitalisations of `{{dlc+|r}} Behavior in Mausoleum/Gehenna`. `examples/probe_discarded.rs`
/// is the thing that needed it.
///
/// **A template carrying a name used to be stripped exactly like one carrying a marker**, so
/// `Interactions with {{c|Tainted Eve}}` came out as `interactions with` — a title truncated
/// rather than cleaned, and a diagnostic that named something no page has. It cannot change which
/// sections are kept: across the whole snapshot that is the only level-2 heading with a content
/// template in it, and it is dropped either way. `{{anchor|…}}` stays a marker because it renders
/// nothing at all, which is why a heading made only of one comes out **empty** — the wiki draws it
/// blank too, and The Forgotten's unlock section is the page that proves it.
pub fn normalize_title(title: &str) -> String {
    let chars: Vec<char> = title.chars().collect();
    let mut s = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' && chars.get(i + 1) == Some(&'{') {
            let (inner, next) = template_at(&chars, i);
            s.push_str(&rendered(&inner));
            i = next;
            continue;
        }
        if chars[i] == '[' || chars[i] == ']' {
            i += 1;
            continue;
        }
        s.push(chars[i]);
        i += 1;
    }
    s.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The text a `{{…}}` at `at` leaves behind, and the index just past it. Nesting is followed so
/// a template inside an argument does not end the outer one early.
fn template_at(chars: &[char], at: usize) -> (String, usize) {
    let mut depth = 0usize;
    let mut i = at;
    let mut inner = String::new();
    while i < chars.len() {
        if chars[i] == '{' && chars.get(i + 1) == Some(&'{') {
            depth += 1;
            i += 2;
            if depth > 1 {
                inner.push_str("{{");
            }
            continue;
        }
        if chars[i] == '}' && chars.get(i + 1) == Some(&'}') {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return (inner, i);
            }
            inner.push_str("}}");
            continue;
        }
        inner.push(chars[i]);
        i += 1;
    }
    // Unclosed: everything after it was inside the template as far as anyone can tell.
    (inner, chars.len())
}

/// What a template puts on screen: nothing for a marker, its first argument otherwise — which is
/// the name for `{{c|Tainted Eve}}` and `{{s|Ashpit}}`, the only two content templates any
/// heading in the snapshot uses.
fn rendered(inner: &str) -> String {
    let mut parts = inner.split('|');
    let name = parts.next().unwrap_or("").trim().to_lowercase();
    if MARKER_TEMPLATES.contains(&name.as_str()) {
        return String::new();
    }
    parts.next().unwrap_or("").trim().to_string()
}

/// The section kind for a wiki title; `None` for titles that aren't kept.
pub fn section_kind(title: &str) -> Option<SectionKind> {
    Some(match normalize_title(title).as_str() {
        "effects" | "effect" | "no effect" => SectionKind::Effects,
        // "Excluded Items" is a note about the subject ("the following items cannot be
        // found while playing as…"); its list comes from a table template we don't expand.
        "notes" | "excluded items" => SectionKind::Notes,
        // "Syngergies" is a typo, on collectible/Blood Bombs, and the only one in the snapshot.
        "synergies" | "syngergies" => SectionKind::Synergies,
        // Six spellings read on their own pages (B54), and the line drawn there: **a spelling of
        // a kind comes in, a qualifier does not.** "Items Interactions" is the plural of one
        // already here (character/Tainted Lost); "Active Item Interactions" is the same kind
        // narrowed to actives (trinket/Found Soul); "Other Interactions" is the leftovers of a
        // page that has several (trinket/Broken Remote).
        //
        // Refused in the same reading, and each with its page: "Interactions with
        // {{c|Tainted Eve}}" (collectible/Sumptorium) names **one subject**, and a kind that
        // names a subject stops being a kind.
        "interactions" | "item interactions" | "items interactions" | "active item interactions"
        | "other interactions" | "interaction" => SectionKind::Interactions,
        "bugs" | "bug" => SectionKind::Bugs,
        "behavior" => SectionKind::Behavior,
        "champion versions" => SectionKind::ChampionVersions,
        "damage scaling" => SectionKind::DamageScaling,
        // "General Strategies" (character/The Lost, which opens with `{{main|The Lost (Strategy)}}`)
        // and "Tips and strategies" (collectible/Isaac's Heart) are both spellings of the two
        // words already here.
        "strategies" | "strategy" | "tips" | "general strategies" | "tips and strategies" => {
            SectionKind::Strategies
        }
        "difficulty" => SectionKind::Difficulty,
        "reward" | "rewards" => SectionKind::Reward,
        "unlockable achievements"
        | "unlockable achievement"
        | "unlockable starting items"
        | "unlockable items"
        // "How to Acquire" describes how a thing is reached, which is what this kind is.
        | "how to acquire"
        // Three more of the same, each **read on its page first** — which is the rule B54 sets
        // against adding spellings by reflex, and the reason these are three exact strings and
        // not a prefix that would accept headings nobody has looked at.
        //   "Unlock"                           — boss/Mega Satan, the criteria for the gate
        //   "Unlocking The Lost in Rebirth"    — character/The Lost, the four deaths in order
        //   "Unlocking The Lost in Afterbirth" — the same, for the earlier edition
        | "unlock"
        | "unlocking the lost in rebirth"
        | "unlocking the lost in afterbirth" => SectionKind::Unlockable,
        _ => return None, // allowed: free-form wiki title
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SectionKind;

    #[test]
    fn splits_level_two_and_keeps_subsections_inside() {
        let src = "{{infobox boss\n | id = 1\n}}\n'''X''' intro\n\n== Behavior ==\n=== Phase 1 ===\na\n== Notes ==\nb\n";
        let (pre, secs) = split_page(src);
        assert!(pre.contains("intro"));
        assert_eq!(secs.len(), 2);
        assert_eq!(secs[0].title, "Behavior");
        assert_eq!(secs[0].body.trim(), "=== Phase 1 ===\na");
        assert_eq!(secs[1].title, "Notes");
    }

    /// `discardedSections` in the built dataset mixes two different things, and only one of
    /// them was a decision. These six were falling through by oversight: five are a plural
    /// or singular of a title the mapping already knew, and "How to Acquire" describes how
    /// a thing is reached, which is what `Unlockable` is.
    #[test]
    fn titles_that_used_to_fall_through_now_map() {
        assert_eq!(
            section_kind("Unlockable Items"),
            Some(SectionKind::Unlockable)
        );
        assert_eq!(
            section_kind("How to Acquire"),
            Some(SectionKind::Unlockable)
        );
        assert_eq!(section_kind("Bug"), Some(SectionKind::Bugs));
        assert_eq!(section_kind("Interaction"), Some(SectionKind::Interactions));
        assert_eq!(section_kind("Rewards"), Some(SectionKind::Reward));
        // "The following items cannot be found while playing as Tainted Lost" — a note
        // about the subject, whose list comes from a table template we do not expand.
        assert_eq!(section_kind("Excluded Items"), Some(SectionKind::Notes));
    }

    /// B54. A template carrying a **name** used to be stripped exactly like one carrying a
    /// **marker**, so a heading came out truncated rather than cleaned — and the diagnostic then
    /// named a title no page has.
    #[test]
    fn a_template_carrying_a_name_leaves_the_name_behind() {
        assert_eq!(
            normalize_title("Interactions with {{c|Tainted Eve}}"),
            "interactions with tainted eve"
        );
        assert_eq!(normalize_title("{{s|Ashpit}} Waves"), "ashpit waves");
    }

    /// The other half of the same rule, and the reason it is a list and not "strip nothing":
    /// these render a marker or nothing, so a heading means the same without them.
    #[test]
    fn a_template_carrying_a_marker_leaves_nothing_behind() {
        assert_eq!(normalize_title("{{dlc|nr}} Gallery"), "gallery");
        assert_eq!(
            normalize_title("Champion Versions {{unlockable}}"),
            "champion versions"
        );
        // Two capitalisations of one heading, which `discardedSections` counts as two titles.
        assert_eq!(
            normalize_title("{{Dlc+|r}} Behavior in Mausoleum/Gehenna"),
            normalize_title("{{dlc+|r}} Behavior in Mausoleum/Gehenna")
        );
    }

    /// `{{anchor}}` renders nothing, so a heading made only of one **is** empty — the wiki draws
    /// it blank too. The Forgotten's unlock section sits under one, and this pins that the
    /// emptiness is the answer rather than a parse failure: what is lost there is lost on the
    /// wiki's side, and the fix is a correction to the page, not to this function.
    #[test]
    fn a_heading_that_is_only_an_anchor_is_empty() {
        assert_eq!(
            normalize_title("{{anchor|Unlocking the Forgotten|Unlocking The Forgotten}}"),
            ""
        );
        assert_eq!(section_kind(""), None);
    }

    /// B54's near-miss family, with the line the reading drew: **a spelling of a kind comes in,
    /// a qualifier does not.** Six came in; the refusals below are the same reading and are what
    /// stop the list from growing by reflex.
    #[test]
    fn a_spelling_of_a_kind_comes_in() {
        // collectible/Blood Bombs, the snapshot's only typo of the word.
        assert_eq!(section_kind("Syngergies"), Some(SectionKind::Synergies));
        // character/Tainted Lost, trinket/Found Soul, trinket/Broken Remote.
        for t in [
            "Items Interactions",
            "Active Item Interactions",
            "Other Interactions",
        ] {
            assert_eq!(section_kind(t), Some(SectionKind::Interactions), "{t}");
        }
        // character/The Lost, collectible/Isaac's Heart.
        for t in ["General Strategies", "Tips and strategies"] {
            assert_eq!(section_kind(t), Some(SectionKind::Strategies), "{t}");
        }
    }

    /// The refusals, each with the page that refused it. A qualifier is content: folding it in
    /// would make the kind assert something the page distinguishes.
    #[test]
    fn a_qualifier_is_not_a_spelling() {
        // collectible/Sumptorium: a kind that names one subject stops being a kind.
        assert_eq!(section_kind("Interactions with {{c|Tainted Eve}}"), None);
        // trinket/Broken Remote keeps four of these apart on one page, and "infinite",
        // "conditional", "pre-Repentance" and "with delay" are the whole of what they say.
        for t in [
            "Infinite Synergies",
            "Infinite Synergies (Conditional)",
            "Infinite Synergies (Pre-Repentance)",
            "Infinite Synergies (With Delay)",
        ] {
            assert_eq!(section_kind(t), None, "{t}");
        }
        // boss/Mom and boss/Mom's Heart: where the behaviour applies is what the heading adds.
        assert_eq!(
            section_kind("{{dlc+|r}} Behavior in Mausoleum/Gehenna"),
            None
        );
        // challenge/Bloody Mary and others: an editorial verdict on items, which `Notes` does
        // not make. The entry says so and the pages say so.
        for t in [
            "Good Items",
            "Bad items",
            "Neutral Items",
            "Detrimental items",
        ] {
            assert_eq!(section_kind(t), None, "{t}");
        }
    }

    /// Three spellings read on their own pages before being added (B54): Mega Satan's `Unlock`,
    /// and The Lost's two editions. Each is a procedure for reaching something, which is what
    /// `Unlockable` is — the same reasoning that let `How to Acquire` in.
    #[test]
    fn the_unlock_procedures_are_unlockable() {
        assert_eq!(section_kind("Unlock"), Some(SectionKind::Unlockable));
        assert_eq!(
            section_kind("{{dlc|na}} Unlocking The Lost in Rebirth"),
            Some(SectionKind::Unlockable)
        );
        assert_eq!(
            section_kind("{{dlc|a}} Unlocking The Lost in Afterbirth"),
            Some(SectionKind::Unlockable)
        );
        // Not a prefix rule: a spelling nobody has read stays out.
        assert_eq!(section_kind("Unlocking Tainted Eden"), None);
    }

    /// The other half of `discardedSections`, and the half worth protecting: these are out
    /// on purpose, and this test is what keeps a later "let's map everything" from taking
    /// 2248 lines of trivia and video embeds into the dataset.
    #[test]
    fn the_deliberate_discards_stay_discarded() {
        assert_eq!(section_kind("Trivia"), None);
        assert_eq!(section_kind("Gallery"), None);
        assert_eq!(section_kind("In-game Footage"), None);
        assert_eq!(section_kind("References"), None);
        // A page-specific heading is not a kind, however often it appears.
        assert_eq!(section_kind("Blood Clots"), None);
    }

    #[test]
    fn section_kinds() {
        assert_eq!(section_kind("Effects"), Some(SectionKind::Effects));
        assert_eq!(section_kind("No Effect"), Some(SectionKind::Effects));
        assert_eq!(
            section_kind("Unlockable [[Achievement]]s"),
            Some(SectionKind::Unlockable)
        );
        assert_eq!(
            section_kind("Champion Versions {{unlockable}}"),
            Some(SectionKind::ChampionVersions)
        );
        assert_eq!(section_kind("In-game Footage"), None);
        assert_eq!(section_kind("Trivia"), None);
    }
}
