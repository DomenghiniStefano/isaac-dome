//! Reading one slot of a flag section — achievements (section 1), challenges, collection —
//! with the default each question needs when the save holds no record of it.
//!
//! The same `flags.get(slot).copied()` was written out at eight call sites with three
//! different answers for "past the end of the section" (card #82, S4). The answers are right
//! for their questions and differ on purpose; what was wrong was that nothing named which one
//! a site meant. Each helper here is one of those answers, named for it.

/// Whether the save records the slot as done. A slot past the section's end reads as **not
/// done**: the save has no record of it, and "done" is a claim that needs one.
pub(crate) fn recorded_done(flags: &[bool], slot: u32) -> bool {
    recorded(Some(flags), slot).unwrap_or(false)
}

/// Whether nothing recorded says the slot is still locked. A slot past the section's end
/// reads as **unlocked**: an unknown must never hide a row somebody could play.
pub(crate) fn playable_unless_locked(flags: &[bool], slot: u32) -> bool {
    recorded(Some(flags), slot).unwrap_or(true)
}

/// What the save records for the slot, and `None` when it records nothing — the section did
/// not read, or the slot is past its end. For the screens that draw "unknown" as its own state.
pub(crate) fn recorded(flags: Option<&[bool]>, slot: u32) -> Option<bool> {
    flags?.get(slot as usize).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLAGS: [bool; 3] = [false, true, false];

    #[test]
    fn a_recorded_slot_answers_what_it_records() {
        assert!(recorded_done(&FLAGS, 1));
        assert!(!recorded_done(&FLAGS, 2));
        assert!(playable_unless_locked(&FLAGS, 1));
        assert!(!playable_unless_locked(&FLAGS, 2));
        assert_eq!(recorded(Some(&FLAGS), 1), Some(true));
        assert_eq!(recorded(Some(&FLAGS), 2), Some(false));
    }

    #[test]
    fn a_slot_past_the_end_answers_each_questions_own_default() {
        assert!(!recorded_done(&FLAGS, 3), "no record is not a done");
        assert!(
            playable_unless_locked(&FLAGS, 3),
            "no record is not a lock: an unknown never hides a row"
        );
        assert_eq!(recorded(Some(&FLAGS), 3), None);
        assert_eq!(recorded(None, 0), None, "a section that did not read");
    }
}
