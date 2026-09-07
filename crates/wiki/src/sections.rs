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
        "notes" => SectionKind::Notes,
        "synergies" => SectionKind::Synergies,
        "interactions" | "item interactions" => SectionKind::Interactions,
        "bugs" => SectionKind::Bugs,
        "behavior" => SectionKind::Behavior,
        "champion versions" => SectionKind::ChampionVersions,
        "damage scaling" => SectionKind::DamageScaling,
        "strategies" | "strategy" | "tips" => SectionKind::Strategies,
        "difficulty" => SectionKind::Difficulty,
        "reward" => SectionKind::Reward,
        "unlockable achievements" | "unlockable achievement" | "unlockable starting items" => {
            SectionKind::Unlockable
        }
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
