//! A page = preamble + level-2 sections. Subsections stay in the body.

use crate::template::{template_segments, Segment, Template};
use crate::SectionKind;

/// A level-2 section as it is in the wikitext: raw title and body, subsections included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSection {
    pub title: String,
    pub body: String,
}

/// Splits the page into preamble (everything before the first `== … ==`) and level-2
/// sections. Level-3 headings and beyond stay in the body of the section that contains them.
///
/// A `<tabber>…</tabber>` span is opaque to this cut: MediaWiki's tab syntax reuses `==` to
/// open each tab's content on some pages (collectible/Equality!, one tab per edition), and a
/// line-by-line pass reading those as page sections merges two editions' `Effects` into one
/// and loses which tab either half belongs to. Whatever is inside the tags stays raw text in
/// whichever section (or the preamble) was already open when `<tabber>` appeared — the same
/// place any other body text on that line would land — so the tab markers and the headings
/// they carry reach `parse_blocks` intact, for the block parser to read as it renders tabs.
pub fn split_page(text: &str) -> (String, Vec<RawSection>) {
    let mut pre = String::new();
    let mut secs: Vec<RawSection> = Vec::new();
    let mut in_tabber = false;
    for line in text.lines() {
        let t = line.trim();
        if in_tabber {
            in_tabber = !t.contains("</tabber>");
            push_line(&mut pre, &mut secs, line);
            continue;
        }
        if t.contains("<tabber>") && !t.contains("</tabber>") {
            in_tabber = true;
            push_line(&mut pre, &mut secs, line);
            continue;
        }
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
        push_line(&mut pre, &mut secs, line);
    }
    (pre, secs)
}

/// One line, into the last opened section's body, or the preamble when none has opened yet.
fn push_line(pre: &mut String, secs: &mut [RawSection], line: &str) {
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

/// Templates that render a marker or nothing at all, so a heading means the same without them.
/// **Measured, not guessed** (B54, re-read 2026-09-25): across the 1113 pages of the snapshot,
/// exactly six template names appear in a heading — four of these (`dlc`, `dlc+`, `unlockable`,
/// `anchor`), plus `{{s|…}}` and `{{c|…}}`, which are a stage's and a character's *name* and are
/// the whole point of the distinction below. `dlc-` is in no heading: it is listed because it
/// is the closer of `dlc+`, and renders nothing wherever it is.
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
    let flat: String = template_segments(title)
        .map(|segment| match segment {
            Segment::Text(text) => text.replace(['[', ']'], ""),
            Segment::Template { template, .. } => rendered(&template),
        })
        .collect();
    flat.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// What a template puts on screen: nothing for a marker, its first argument otherwise — which is
/// the name for `{{c|Tainted Eve}}` and `{{s|Ashpit}}`, the only two content templates any
/// heading in the snapshot uses.
fn rendered(t: &Template) -> String {
    if MARKER_TEMPLATES.contains(&t.name.as_str()) {
        return String::new();
    }
    t.args.first().cloned().unwrap_or_default()
}

/// Titles dropped by name, with the reason recorded in `corrections.json`'s
/// `excluded.sections` (design decision 10): images and video the constraints forbid
/// shipping (`Gallery`, the `In-game Footage`/`Ingame Footage`/`In-Game Footage` spellings,
/// `Audio`), citations that point off the wiki (`References`), and the owner's call to leave
/// the least useful part of a page out (`Trivia`, decision 9). Everything else that used to
/// fall through `section_kind` now lands there instead, as `SectionKind::Other` — this list
/// is the one place a heading is still thrown away.
///
/// Checked against the raw corpus by
/// `every_discarded_heading_is_listed_and_every_listed_heading_occurs`: a title dropped here
/// and missing from `corrections.json`, or listed there and no longer dropped, fails the
/// build.
pub fn is_excluded_section(title: &str) -> bool {
    matches!(
        normalize_title(title).as_str(),
        "gallery" | "in-game footage" | "ingame footage" | "references" | "trivia" | "audio"
    )
}

/// The section kind for a wiki title. A title the thirteen named kinds below don't
/// recognize, and that isn't on [`is_excluded_section`]'s list, is [`SectionKind::Other`]:
/// design decision 2 keeps it under its own heading rather than discarding it, so this
/// function itself never has a "throw it away" answer any more — that decision now belongs
/// entirely to `is_excluded_section`, read first by the caller.
pub fn section_kind(title: &str) -> SectionKind {
    match normalize_title(title).as_str() {
        "effects" | "effect" | "no effect" => SectionKind::Effects,
        // "Excluded Items" is a note about the subject ("the following items cannot be
        // found while playing as…"); its list comes from a table template we don't expand.
        // "Item Exclusion" (character/Tainted Lost) is the same section under another name.
        "notes" | "excluded items" | "item exclusion" => SectionKind::Notes,
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
        "unlockable starting items" => SectionKind::StartingItems,
        "unlockable achievements"
        | "unlockable achievement"
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
        // A free-form wiki title: kept under its own heading (`Section::title`), not one of
        // the thirteen named kinds.
        _ => SectionKind::Other,
    }
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

    /// collectible/Equality!: the whole page is one `<tabber>`, opened before any `== … ==`
    /// of the page's own, and each tab's content carries `==Effects==`/`==Synergies==`
    /// headings of its own — MediaWiki's tab syntax, not this page's sections. Read as
    /// ordinary headings they merge two editions' `Effects` into one section and lose which
    /// tab either half came from; opaque, none of it becomes a `RawSection`, and the raw
    /// tags and headings both survive in the preamble text for the block parser to read.
    #[test]
    fn a_tabber_before_any_heading_is_not_split_on_its_own_headings() {
        let src = "{{infobox trinket|id=103}}\n\n<tabber>{{dlc|nr+}} Before=\n== Effects ==\n* a\n== Synergies ==\n* b\n|-|{{dlc|r+}} After=\n== Effects ==\n* c\n</tabber>\n\n{{nav|x}}\n";
        let (pre, secs) = split_page(src);
        assert!(secs.is_empty(), "{secs:?}");
        assert!(pre.contains("<tabber>"));
        assert!(pre.contains("== Effects =="));
        assert!(pre.contains("== Synergies =="));
        assert!(pre.contains("</tabber>"));
    }

    /// The common shape (article/Range, article/Completion Marks): a `<tabber>` with no
    /// heading inside it at all. Opacity changes nothing here — there is nothing to protect
    /// against — and this pins that the plain case still behaves like before.
    #[test]
    fn a_tabber_with_no_headings_inside_is_unaffected() {
        let src = "intro\n\n<tabber>\n Post-Repentance=\n{| \n|a\n|}\n|-|\n Pre-Repentance=\n{|\n|b\n|}\n</tabber>\n\n== Notes ==\nc\n";
        let (pre, secs) = split_page(src);
        assert_eq!(secs.len(), 1);
        assert_eq!(secs[0].title, "Notes");
        assert!(pre.contains("Post-Repentance"));
        assert!(pre.contains("Pre-Repentance"));
    }

    /// A `<tabber>` that opens **inside** an already-titled section stays there: its content,
    /// headings included, is the section's body, and the page's real next heading still cuts
    /// a new section right after `</tabber>` closes.
    #[test]
    fn a_tabber_inside_a_section_stays_in_that_section() {
        let src = "== Effects ==\n<tabber>Before=\n== Sub ==\nx\n|-|After=\n== Sub ==\ny\n</tabber>\n== Notes ==\nz\n";
        let (_pre, secs) = split_page(src);
        assert_eq!(secs.len(), 2);
        assert_eq!(secs[0].title, "Effects");
        assert!(secs[0].body.contains("<tabber>"));
        assert!(secs[0].body.contains("== Sub =="));
        assert!(secs[0].body.contains("</tabber>"));
        assert_eq!(secs[1].title, "Notes");
        assert_eq!(secs[1].body.trim(), "z");
    }

    /// `discardedSections` in the built dataset mixes two different things, and only one of
    /// them was a decision. These six were falling through by oversight: five are a plural
    /// or singular of a title the mapping already knew, and "How to Acquire" describes how
    /// a thing is reached, which is what `Unlockable` is.
    #[test]
    fn titles_that_used_to_fall_through_now_map() {
        assert_eq!(section_kind("Unlockable Items"), SectionKind::Unlockable);
        assert_eq!(section_kind("How to Acquire"), SectionKind::Unlockable);
        assert_eq!(section_kind("Bug"), SectionKind::Bugs);
        assert_eq!(section_kind("Interaction"), SectionKind::Interactions);
        assert_eq!(section_kind("Rewards"), SectionKind::Reward);
        // "The following items cannot be found while playing as Tainted Lost" — a note
        // about the subject, whose list comes from a table template we do not expand.
        assert_eq!(section_kind("Excluded Items"), SectionKind::Notes);
        // character/Tainted Lost: the same note, under the name decision 2 maps it by.
        assert_eq!(section_kind("Item Exclusion"), SectionKind::Notes);
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
    /// wiki's side, and the fix is a correction to the page, not to this function. An empty
    /// title still isn't excluded: it is `Other`, the same as any other title none of the
    /// thirteen kinds names.
    #[test]
    fn a_heading_that_is_only_an_anchor_is_empty() {
        assert_eq!(
            normalize_title("{{anchor|Unlocking the Forgotten|Unlocking The Forgotten}}"),
            ""
        );
        assert_eq!(section_kind(""), SectionKind::Other);
        assert!(!is_excluded_section(""));
    }

    /// B54's near-miss family, with the line the reading drew: **a spelling of a kind comes in,
    /// a qualifier does not.** Six came in; the refusals below are the same reading and are what
    /// stop the list from growing by reflex.
    #[test]
    fn a_spelling_of_a_kind_comes_in() {
        // collectible/Blood Bombs, the snapshot's only typo of the word.
        assert_eq!(section_kind("Syngergies"), SectionKind::Synergies);
        // character/Tainted Lost, trinket/Found Soul, trinket/Broken Remote.
        for t in [
            "Items Interactions",
            "Active Item Interactions",
            "Other Interactions",
        ] {
            assert_eq!(section_kind(t), SectionKind::Interactions, "{t}");
        }
        // character/The Lost, collectible/Isaac's Heart.
        for t in ["General Strategies", "Tips and strategies"] {
            assert_eq!(section_kind(t), SectionKind::Strategies, "{t}");
        }
    }

    /// The refusals, each with the page that refused it. A qualifier is content: folding a
    /// spelling that carries one into the kind would make the kind assert something the page
    /// distinguishes. Refused **as a kind**, not discarded: decision 2 keeps every one of these
    /// under its own title, as `SectionKind::Other`.
    #[test]
    fn a_qualifier_is_not_a_spelling() {
        // collectible/Sumptorium: a kind that names one subject stops being a kind.
        assert_eq!(
            section_kind("Interactions with {{c|Tainted Eve}}"),
            SectionKind::Other
        );
        // trinket/Broken Remote keeps four of these apart on one page, and "infinite",
        // "conditional", "pre-Repentance" and "with delay" are the whole of what they say.
        for t in [
            "Infinite Synergies",
            "Infinite Synergies (Conditional)",
            "Infinite Synergies (Pre-Repentance)",
            "Infinite Synergies (With Delay)",
        ] {
            assert_eq!(section_kind(t), SectionKind::Other, "{t}");
        }
        // boss/Mom and boss/Mom's Heart: where the behaviour applies is what the heading adds.
        assert_eq!(
            section_kind("{{dlc+|r}} Behavior in Mausoleum/Gehenna"),
            SectionKind::Other
        );
        // challenge/Bloody Mary and others: an editorial verdict on items, which `Notes` does
        // not make. The entry says so and the pages say so.
        for t in [
            "Good Items",
            "Bad items",
            "Neutral Items",
            "Detrimental items",
        ] {
            assert_eq!(section_kind(t), SectionKind::Other, "{t}");
        }
    }

    /// B54's third family, **read on 2026-09-17 and recorded here rather than summarised**.
    ///
    /// The entry asks for these to carry "the record that they were read", and a test is where
    /// that record survives: prose in a document does not fail when somebody adds one of these
    /// spellings by reflex, and this does. Each line is one page's own heading, seen exactly
    /// once in the whole snapshot, with the page it is on and what the section actually holds.
    ///
    /// None of them is a near-miss, and none of them is a kind: `SectionKind` stays the
    /// thirteen closed names it already has. Decision 2 changed what happens to a heading in
    /// none of them — kept under its own title as `Other`, not thrown away — not which
    /// headings those are, so every one of these is still `Other` and none of the four
    /// families below earns a kind of its own.
    #[test]
    fn a_page_own_heading_is_read_and_kept_as_other() {
        // 1. Game data in a table or list, with no kind in `SectionKind` that names it. Adding
        //    one would mean inventing a kind per page, which is the opposite of a closed list.
        for (title, page) in [
            (
                "Component Types and Qualities",
                "collectible/Bag of Crafting",
            ),
            ("Recipes", "collectible/Bag of Crafting"),
            ("Uncraftable Items", "collectible/Bag of Crafting"),
            ("{{dlc+|r}} Monster Replacement Tables", "collectible/D10"),
            ("Poop Varieties", "character/Tainted ???"),
            ("Random pool choices", "collectible/Lemegeton"),
            ("Survival", "collectible/Damocles"),
            ("Notable Rerolls", "collectible/Spindown Dice"),
            ("Specific Item Effects", "collectible/Metronome"),
            ("Drops", "collectible/Bum Friend"),
            ("Modifiers", "challenge/Ultra Hard"),
            ("Everything Is Terrible!!! changes", "boss/Mom's Heart"),
        ] {
            assert_eq!(section_kind(title), SectionKind::Other, "{title} — {page}");
        }

        // 2. The heading is not a heading. `[[Monsters]]` (boss/Great Gideon) is a **link**, and
        //    its section has no body at all: the 54 lines are two `=== … Waves ===` tables under
        //    it. `Videos` is one `{{#ev:youtube}}` and `Sounds` is a table of `.wav` files —
        //    media, the same reason `Gallery` and in-game footage are excluded, but this
        //    heading's own spelling is not on that list, so it is kept as `Other` rather than
        //    dropped — its body is a table, not the media itself. The two `Combinations` are a
        //    single list template each, so the content is not on the page to keep either way.
        for (title, page) in [
            ("[[Monsters]]", "boss/Great Gideon"),
            ("Videos", "trinket/Super Bum"),
            ("Sounds", "trinket/Dog Tooth"),
            ("Combinations", "collectible/Book of Virtues"),
            (
                "{{dlc+|r}} Judas' Birthright Combinations",
                "collectible/The Book of Belial",
            ),
        ] {
            assert_eq!(section_kind(title), SectionKind::Other, "{title} — {page}");
        }

        // 3. Research that lives off the wiki. `Algorithm` is a Lua listing credited to a wiki
        //    user, `Puzzle Pieces` is lore pointing at imgur and reddit. Neither is the game
        //    saying something about itself, but neither is excluded by name either: kept as
        //    `Other`, under its own title, like the rest of this list.
        for (title, page) in [
            ("Algorithm", "collectible/GB Bug"),
            ("Puzzle Pieces {{dlc|na}}", "collectible/Missing Poster"),
        ] {
            assert_eq!(section_kind(title), SectionKind::Other, "{title} — {page}");
        }

        // 4. An editorial verdict, which is the family `Good Items` and its three were already
        //    refused a *kind* for. `Items` on character/Tainted Eden opens "there are some that
        //    should be of special notice", which is the verdict in a shorter word — still kept,
        //    still `Other`.
        assert_eq!(
            section_kind("Items"),
            SectionKind::Other,
            "character/Tainted Eden"
        );

        // **The closest call of the 22, recorded as one so it can be overturned cheaply.**
        // `Strategy and Items` (character/Tainted Apollyon, 22 lines) is strategy prose from the
        // first bullet to the last, and `General Strategies` and `Tips and strategies` are both
        // in. What keeps it out of `Strategies` is the line collectible/Sumptorium drew: the
        // heading names what the strategy is *about*, and every accepted spelling names only the
        // kind. If that line ever moves, this is the page to move it on — and the reason it is
        // written here rather than decided quietly is that the entry's own rule is to read the
        // page first, which is exactly what makes this one arguable.
        assert_eq!(
            section_kind("Strategy and Items"),
            SectionKind::Other,
            "character/Tainted Apollyon"
        );
    }

    /// Three spellings read on their own pages before being added (B54): Mega Satan's `Unlock`,
    /// and The Lost's two editions. Each is a procedure for reaching something, which is what
    /// `Unlockable` is — the same reasoning that let `How to Acquire` in.
    #[test]
    fn the_unlock_procedures_are_unlockable() {
        assert_eq!(section_kind("Unlock"), SectionKind::Unlockable);
        assert_eq!(
            section_kind("{{dlc|na}} Unlocking The Lost in Rebirth"),
            SectionKind::Unlockable
        );
        assert_eq!(
            section_kind("{{dlc|a}} Unlocking The Lost in Afterbirth"),
            SectionKind::Unlockable
        );
        // Not a prefix rule: a spelling nobody has read stays out of the kind — it is `Other`,
        // not discarded.
        assert_eq!(section_kind("Unlocking Tainted Eden"), SectionKind::Other);
    }

    /// Isaac's page has "Unlockable Achievements" and, under it, "Unlockable Starting Items"
    /// ("Defeat Isaac as ??? - Start with The D6"); so do seven more base characters. Read as
    /// one kind they were two sections with the same name.
    #[test]
    fn the_starting_items_are_a_kind_of_their_own() {
        assert_eq!(
            section_kind("Unlockable Starting Items"),
            SectionKind::StartingItems
        );
        assert_eq!(
            section_kind("Unlockable [[Achievement]]s"),
            SectionKind::Unlockable
        );
    }

    /// The other half of `discardedSections`, and the half worth protecting: these are out
    /// on purpose, and this test is what keeps a later "let's map everything" from taking
    /// 2248 lines of trivia and video embeds into the dataset. `Blood Clots`, a page-specific
    /// heading with no reason to be dropped, is the contrast: kept, as `Other`.
    #[test]
    fn the_deliberate_discards_stay_discarded() {
        for t in [
            "Trivia",
            "Gallery",
            "In-game Footage",
            "In-Game Footage",
            "Ingame Footage",
            "References",
            "Audio",
        ] {
            assert!(is_excluded_section(t), "{t}");
        }
        assert!(!is_excluded_section("Blood Clots"));
        assert_eq!(section_kind("Blood Clots"), SectionKind::Other);
    }

    #[test]
    fn section_kinds() {
        assert_eq!(section_kind("Effects"), SectionKind::Effects);
        assert_eq!(section_kind("No Effect"), SectionKind::Effects);
        assert_eq!(
            section_kind("Unlockable [[Achievement]]s"),
            SectionKind::Unlockable
        );
        assert_eq!(
            section_kind("Champion Versions {{unlockable}}"),
            SectionKind::ChampionVersions
        );
        // Excluded before `section_kind` is ever asked, by `is_excluded_section` — see
        // `the_deliberate_discards_stay_discarded`. Read on its own, `section_kind` has no
        // "throw it away" answer left, so these come back `Other` like any other free title.
        assert_eq!(section_kind("In-game Footage"), SectionKind::Other);
        assert_eq!(section_kind("Trivia"), SectionKind::Other);
    }

    /// Design decision 10's completeness check, scoped to sections: every level-2 heading the
    /// raw corpus actually drops has to be named in `corrections.json`'s `excluded.sections`,
    /// with its reason, and a name listed there that the corpus no longer drops is stale and
    /// fails too. Read independently of `resolver::Corrections` — another branch adds an
    /// `excluded.templates` key under the same top-level `excluded` object, and the two
    /// readers merge into one struct once both land.
    #[test]
    fn every_discarded_heading_is_listed_and_every_listed_heading_occurs() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dataset");
        let raw = crate::Raw::load(&root.join("raw")).expect("dataset/raw/");
        let corrections: SectionCorrections = serde_json::from_str(
            &std::fs::read_to_string(root.join("corrections.json")).expect("corrections.json"),
        )
        .expect("json");

        let listed: std::collections::BTreeSet<String> = corrections
            .excluded
            .sections
            .keys()
            .map(|title| normalize_title(title))
            .collect();

        let mut occurring: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for page in &raw.pages {
            let (_pre, secs) = split_page(&page.text);
            for s in &secs {
                if is_excluded_section(&s.title) {
                    occurring.insert(normalize_title(&s.title));
                }
            }
        }

        let unlisted: Vec<&String> = occurring.difference(&listed).collect();
        assert!(
            unlisted.is_empty(),
            "discarded but missing from corrections.json's excluded.sections: {unlisted:?}"
        );
        let stale: Vec<&String> = listed.difference(&occurring).collect();
        assert!(
            stale.is_empty(),
            "listed in corrections.json's excluded.sections but no longer discarded: {stale:?}"
        );
    }

    /// The slice of `corrections.json` this module reads on its own: `excluded.sections`,
    /// title (as a page writes it, not normalized) → reason. A small, separate struct rather
    /// than a field on `resolver::Corrections` (see the note on `is_excluded_section` and the
    /// test above) — reconciled into one reader at merge, once the `excluded.templates`
    /// branch lands. Test-only: nothing at runtime reads this half of the file, the same way
    /// `section_kind`'s decision of what to keep is a closed list that needs no file at all.
    #[derive(Debug, Clone, Default, serde::Deserialize)]
    struct SectionCorrections {
        #[serde(default)]
        excluded: ExcludedKinds,
    }

    #[derive(Debug, Clone, Default, serde::Deserialize)]
    struct ExcludedKinds {
        #[serde(default)]
        sections: std::collections::BTreeMap<String, String>,
    }
}
