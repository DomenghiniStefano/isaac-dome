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

/// A comparable title: `{{…}}` gone (even nested), link brackets gone, lowercased,
/// whitespace collapsed.
fn normalize_title(title: &str) -> String {
    let mut s = String::new();
    let mut skip = 0usize;
    let mut chars = title.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'{') {
            skip += 1;
            chars.next();
            continue;
        }
        if c == '}' && chars.peek() == Some(&'}') {
            skip = skip.saturating_sub(1);
            chars.next();
            continue;
        }
        if skip > 0 {
            continue;
        }
        if c == '[' || c == ']' {
            continue;
        }
        s.push(c);
    }
    s.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The section kind for a wiki title; `None` for titles that aren't kept.
pub fn section_kind(title: &str) -> Option<SectionKind> {
    Some(match normalize_title(title).as_str() {
        "effects" | "effect" | "no effect" => SectionKind::Effects,
        // "Excluded Items" is a note about the subject ("the following items cannot be
        // found while playing as…"); its list comes from a table template we don't expand.
        "notes" | "excluded items" => SectionKind::Notes,
        "synergies" => SectionKind::Synergies,
        "interactions" | "item interactions" | "interaction" => SectionKind::Interactions,
        "bugs" | "bug" => SectionKind::Bugs,
        "behavior" => SectionKind::Behavior,
        "champion versions" => SectionKind::ChampionVersions,
        "damage scaling" => SectionKind::DamageScaling,
        "strategies" | "strategy" | "tips" => SectionKind::Strategies,
        "difficulty" => SectionKind::Difficulty,
        "reward" | "rewards" => SectionKind::Reward,
        "unlockable achievements"
        | "unlockable achievement"
        | "unlockable starting items"
        | "unlockable items"
        // "How to Acquire" describes how a thing is reached, which is what this kind is.
        | "how to acquire" => SectionKind::Unlockable,
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
