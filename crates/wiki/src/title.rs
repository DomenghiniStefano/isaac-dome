//! MediaWiki's own title normalization, not a lookup key: `bindingofisaacrebirth.wiki.gg`
//! treats an underscore in a title as a space and the **first** character as
//! case-insensitive, like every standard namespace on a MediaWiki install — `[[familiar]]`
//! and `[[Familiar]]` are one page, `Blood_Donation_Machine` and `Blood Donation Machine`
//! are one page. `resolver::key` looks similar but answers a different question: it
//! lowercases the *whole* string for a lookup that must not care about case at all, which
//! would conflate `Reroll` and `REROLL` as the same page — right for a lookup key, wrong for
//! the title MediaWiki itself would show.

/// A title as MediaWiki's own normalization would spell it: `_` read as a space, runs of
/// whitespace collapsed to one and trimmed, and only the first character uppercased — with
/// `char::to_uppercase`, which is Unicode-aware and can turn one character into several
/// (`ß` → `SS`), the same rule MediaWiki applies to every alphabet, not just ASCII.
pub fn canonical_title(title: &str) -> String {
    let spaced: String = title
        .chars()
        .map(|c| if c == '_' { ' ' } else { c })
        .collect();
    let collapsed = spaced.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = collapsed.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => collapsed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_first_character_changes_case() {
        assert_eq!(canonical_title("familiar"), "Familiar");
        assert_eq!(canonical_title("Familiar"), "Familiar");
        assert_eq!(canonical_title("dAMAGE"), "DAMAGE");
    }

    #[test]
    fn an_underscore_is_a_space() {
        assert_eq!(
            canonical_title("Blood_Donation_Machine"),
            "Blood Donation Machine"
        );
        assert_eq!(canonical_title("_boss"), "Boss");
    }

    #[test]
    fn whitespace_runs_collapse_and_the_ends_trim() {
        assert_eq!(canonical_title("  Hard   mode  "), "Hard mode");
        assert_eq!(canonical_title("hard\u{a0}mode"), "Hard mode");
    }

    #[test]
    fn empty_and_single_character_titles_are_not_a_special_case() {
        assert_eq!(canonical_title(""), "");
        assert_eq!(canonical_title("_"), "");
        assert_eq!(canonical_title("a"), "A");
    }

    /// `char::to_uppercase` is Unicode-aware, and one input character can become several
    /// (German `ß`, as the first character, uppercases to two letters, `SS`) — the same
    /// rule MediaWiki applies, not an ASCII shortcut that would leave an accented title's
    /// first letter alone.
    #[test]
    fn unicode_uppercasing_is_not_ascii_only() {
        assert_eq!(canonical_title("école"), "École");
        assert_eq!(canonical_title("ßraße"), "SSraße");
    }
}
