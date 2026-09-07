//! `challenges.xml`: 45 challenges with literal names. The unlock link here is
//! **plural** (`achievements="42,34,53"`): a list, not a single id. In the real file
//! the lists aren't always comma-separated: challenge 44 ("Red Redemption") has
//! `achievements="490 415"`, space-separated. And `startingitems` can contain
//! **negative** ids (challenges 37 and 38: `startingitems="-584,34,119,214,569"`): a
//! game signal that doesn't correspond to an item and should be dropped, not a format
//! error.

use crate::diagnostics::{Diagnostic, SkipReason, Source};
use crate::ids::{AchievementId, ChallengeId, ItemId};
use crate::xml::{elements, Element};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Challenge {
    pub id: ChallengeId,
    pub name: String,
    pub starting_items: Vec<ItemId>,
    pub unlocked_by: Vec<AchievementId>,
    /// The achievements earned by completing it. Not in `challenges.xml`: `Catalog::build`
    /// collects it from the notes on the achievements (see `reward`). Sorted.
    pub rewards: Vec<AchievementId>,
}

pub fn parse(bytes: &[u8], diagnostics: &mut Vec<Diagnostic>) -> Vec<Challenge> {
    let els = match elements(bytes) {
        Ok(els) => els,
        Err(_) => {
            diagnostics.push(Diagnostic::SourceUnreadable {
                source: Source::Challenges,
            });
            return Vec::new();
        }
    };
    els.iter()
        .filter(|e| e.name == "challenge")
        .filter_map(|e| challenge_from(e, diagnostics))
        .collect()
}

fn challenge_from(e: &Element, d: &mut Vec<Diagnostic>) -> Option<Challenge> {
    let skip = |id: Option<u32>, reason: SkipReason, d: &mut Vec<Diagnostic>| {
        d.push(Diagnostic::ElementSkipped {
            source: Source::Challenges,
            id,
            reason,
        });
        None
    };
    let Some(raw_id) = e.attr("id") else {
        return skip(None, SkipReason::MissingId, d);
    };
    let Ok(id) = raw_id.parse::<u32>() else {
        return skip(None, SkipReason::MalformedId, d);
    };
    let Some(name) = e.attr("name") else {
        return skip(Some(id), SkipReason::MissingName, d);
    };
    // The two lists degrade differently: an id <= 0 in `startingitems` is a game signal
    // (the id is dropped, the challenge stays valid), while a non-numeric token in
    // `achievements` is a corrupted file (the whole challenge is dropped).
    let Some(items) = item_id_list(e.attr("startingitems").unwrap_or("")) else {
        return skip(Some(id), SkipReason::MalformedList, d);
    };
    let Some(achievements) = id_list(e.attr("achievements").unwrap_or("")) else {
        return skip(Some(id), SkipReason::MalformedList, d);
    };
    Some(Challenge {
        id: ChallengeId(id),
        name: name.to_string(),
        starting_items: items.into_iter().map(ItemId).collect(),
        unlocked_by: achievements.into_iter().map(AchievementId).collect(),
        rewards: Vec::new(),
    })
}

/// `"1,2,3"` or `"1 2 3"` -> `[1,2,3]`; `""` -> `[]`; a non-numeric element -> `None`.
/// The file uses the comma almost everywhere but not always (challenges.xml, challenge
/// 44: `achievements="490 415"`): we split on both separators.
pub(crate) fn id_list(raw: &str) -> Option<Vec<u32>> {
    raw.split(|c: char| c == ',' || c.is_whitespace())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<u32>().ok())
        .collect()
}

/// Like [`id_list`], but for `startingitems`: an id `<= 0` (e.g. `-584`) is a game
/// signal that doesn't correspond to an item and is dropped without invalidating the
/// challenge. Stays `None` only if a token is not a number.
pub(crate) fn item_id_list(raw: &str) -> Option<Vec<u32>> {
    raw.split(|c: char| c == ',' || c.is_whitespace())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<i64>().ok())
        .collect::<Option<Vec<i64>>>()
        .map(|ids| {
            ids.into_iter()
                .filter(|&n| n > 0)
                // try_from, not `as u32`: an i64 beyond u32::MAX is dropped, not
                // silently truncated into a wrong id.
                .filter_map(|n| u32::try_from(n).ok())
                .collect()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CH: &[u8] = b"<challenges version=\"1\">
\t<challenge name=\"Pitch Black\" id=\"1\" endstage=\"6\" />
\t<challenge name=\"High Brow\" id=\"2\" startingitems=\"209,6,236,291\" achievements=\"42,34,53\" endstage=\"6\" />
\t<challenge name=\"Solo\" id=\"3\" achievements=\"8\" startingitems=\"\" endstage=\"1\" />
\t<challenge name=\"Bad\" id=\"4\" achievements=\"1,x\" endstage=\"1\" />
\t<challenge id=\"5\" endstage=\"1\" />
</challenges>";

    #[test]
    fn lists_are_split_on_commas_and_empty_lists_are_empty() {
        let mut d = Vec::new();
        let c = parse(CH, &mut d);
        assert_eq!(c[0].unlocked_by, Vec::<AchievementId>::new());
        assert_eq!(
            c[1].starting_items,
            vec![ItemId(209), ItemId(6), ItemId(236), ItemId(291)]
        );
        assert_eq!(
            c[1].unlocked_by,
            vec![AchievementId(42), AchievementId(34), AchievementId(53)]
        );
        assert_eq!(c[2].unlocked_by, vec![AchievementId(8)]);
        assert!(c[2].starting_items.is_empty());
    }

    #[test]
    fn a_malformed_list_skips_the_element_and_a_missing_name_too() {
        let mut d = Vec::new();
        let c = parse(CH, &mut d);
        assert_eq!(c.len(), 3);
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::Challenges,
            id: Some(4),
            reason: SkipReason::MalformedList
        }));
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::Challenges,
            id: Some(5),
            reason: SkipReason::MissingName
        }));
    }

    #[test]
    fn id_list_accepts_only_all_numeric() {
        assert_eq!(id_list("1,2,3"), Some(vec![1, 2, 3]));
        assert_eq!(id_list(""), Some(vec![]));
        assert_eq!(id_list("1, 2"), Some(vec![1, 2]), "whitespace tolerated");
        assert_eq!(id_list("1,x"), None);
    }

    #[test]
    fn id_list_splits_on_whitespace_too_like_the_sfida_44_achievements() {
        // challenges.xml, challenge 44 "Red Redemption": achievements="490 415".
        assert_eq!(id_list("490 415"), Some(vec![490, 415]));
    }

    #[test]
    fn item_id_list_drops_negative_ids_without_invalidating_the_list() {
        // challenges.xml, challenges 37 and 38: startingitems="-584,34,119,214,569" and similar.
        // -584 is a game signal, not an item: it's dropped, the list stays valid.
        assert_eq!(item_id_list("-584,34,119"), Some(vec![34, 119]));
        assert_eq!(
            item_id_list("abc"),
            None,
            "a non-numeric token stays an error"
        );
        assert_eq!(
            item_id_list("0,34"),
            Some(vec![34]),
            "0 is dropped too, not just negatives"
        );
    }

    #[test]
    fn a_space_separated_achievement_list_and_negative_starting_items_do_not_skip_the_challenge() {
        let ch = b"<challenges version=\"1\">
\t<challenge name=\"Red Redemption\" id=\"44\" achievements=\"490 415\" startingitems=\"175\" />
\t<challenge name=\"Bloody Mary\" id=\"37\" startingitems=\"-584,34,119,214,569\" achievements=\"34,147,404\" />
</challenges>";
        let mut d = Vec::new();
        let c = parse(ch, &mut d);
        assert_eq!(
            c.len(),
            2,
            "neither of the two challenges should be dropped"
        );
        assert!(
            d.is_empty(),
            "neither the space separator nor a negative id is a format error: {d:?}"
        );
        assert_eq!(
            c[0].unlocked_by,
            vec![AchievementId(490), AchievementId(415)]
        );
        assert_eq!(
            c[1].starting_items,
            vec![ItemId(34), ItemId(119), ItemId(214), ItemId(569)]
        );
    }
}
