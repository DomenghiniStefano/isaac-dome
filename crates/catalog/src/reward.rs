//! What completing a challenge grants. The game doesn't write it in `challenges.xml`:
//! it leaves it as a note on the reward achievement, in three different forms
//! depending on the file's era (measured on the Repentance+ `achievements.xml`,
//! 2026-09-05):
//!
//! - comment `Beat Challenge #N` (also `beat`), challenges 1..=20;
//! - comment `beat Challenge N (Name)`, sometimes with a tail (`unlocks
//!   Percs/Overdose`), challenges 21..=30;
//! - attribute `steam_description="Complete Challenge N."`, with no comment, challenges
//!   36..=44.
//!
//! Challenges 31..=35 and 45 leave no trace in the file, and can't be inferred by position.

use crate::ids::ChallengeId;

/// The challenge this text says was beaten, if it's a reward note.
///
/// Deliberately strict: verb `beat` or `complete`, the word `challenge`, an optional
/// `#`, a number; the rest of the line is ignored. So `You unlocked Challenge #4` and
/// `Unlocked a new challenge.` stay out, and a future comment like `unlock challenge
/// #N` wouldn't accidentally become a reward. No `regex`: it's three words and an integer.
pub(crate) fn challenge_beaten(text: &str) -> Option<ChallengeId> {
    let mut words = text.split_whitespace();
    let verb = words.next()?;
    if !(verb.eq_ignore_ascii_case("beat") || verb.eq_ignore_ascii_case("complete")) {
        return None;
    }
    if !words.next()?.eq_ignore_ascii_case("challenge") {
        return None;
    }
    let number = words.next()?;
    let digits: String = number
        .trim_start_matches('#')
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse::<u32>().ok().map(ChallengeId)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::ChallengeId;

    #[test]
    fn beat_with_hash_in_either_case() {
        assert_eq!(
            challenge_beaten("Beat Challenge #19"),
            Some(ChallengeId(19))
        );
        assert_eq!(
            challenge_beaten("beat Challenge #13"),
            Some(ChallengeId(13))
        );
    }

    #[test]
    fn beat_without_hash_with_a_name_and_a_tail() {
        assert_eq!(
            challenge_beaten("beat Challenge 21 (XXXXXXXXL)"),
            Some(ChallengeId(21))
        );
        assert_eq!(
            challenge_beaten("beat Challenge 24 (Pay to Play) unlocks Percs/Overdose"),
            Some(ChallengeId(24))
        );
    }

    #[test]
    fn complete_with_a_trailing_period_like_the_steam_description() {
        assert_eq!(
            challenge_beaten("Complete Challenge 36."),
            Some(ChallengeId(36))
        );
    }

    #[test]
    fn surrounding_whitespace_is_tolerated() {
        assert_eq!(
            challenge_beaten("  Beat Challenge #1  "),
            Some(ChallengeId(1))
        );
    }

    #[test]
    fn unlocking_a_challenge_is_not_beating_it() {
        // The text of the achievement that makes a challenge available, and its
        // steam_description: neither of them is a reward.
        assert_eq!(
            challenge_beaten("You unlocked Challenge #4 Darkness Falls"),
            None
        );
        assert_eq!(challenge_beaten("Unlocked a new challenge."), None);
    }

    #[test]
    fn a_condition_that_is_not_about_challenges_is_none() {
        assert_eq!(
            challenge_beaten("have 7 or more max red hearts at one time"),
            None
        );
        assert_eq!(challenge_beaten("beat Mom"), None);
        assert_eq!(challenge_beaten("Beat Challenge #"), None, "no number");
        assert_eq!(challenge_beaten(""), None);
    }
}
