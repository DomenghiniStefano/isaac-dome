//! The forty-five as the UI sees them: section 7 joined with `challenges.xml`. What the tests
//! defend is that "unread" never reads as "not done", and that a gate which is not done is
//! *named* rather than summarised into a colour — the reading of `unlocked_by` as "all of
//! these" is not settled (spec 3.11 §4), so the claim has to be visible to be disbelieved.

use catalog::Catalog;
use ipc::{challenges_view, ChallengeStateView, ChallengesDiagnostic, ChallengesView};
use serde_json::{json, to_value};

// Challenge 1 is free, 2 is gated by achievement 1, 3 by achievements 1 and 2.
const CHALLENGES: &[u8] = b"<challenges><challenge id=\"1\" name=\"Pitch Black\" startingitems=\"1\" /><challenge id=\"2\" name=\"High Brow\" achievements=\"1\" /><challenge id=\"3\" name=\"Head Trauma\" achievements=\"1,2\" /></challenges>";
const ACH: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><achievement id=\"1\" text=\"t1\" gfx=\"1.png\" /><achievement id=\"2\" text=\"t2\" gfx=\"2.png\" /></achievements>";

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "challenges.xml" => Some(CHALLENGES.to_vec()),
        "achievements.xml" => Some(ACH.to_vec()),
        _ => None,
    })
}

// Section 7, cell 0 unused: challenge 1 done, 2 and 3 not.
const CELLS: [bool; 4] = [false, true, false, false];
// Achievement 1 done, 2 not.
const DONE: [bool; 3] = [false, true, false];

fn view(cells: Option<&[bool]>, achievements: Option<&[bool]>) -> ChallengesView {
    challenges_view(Some(&catalog()), None, cells, achievements, |_| None)
}

fn state(v: &ChallengesView, number: u32) -> ChallengeStateView {
    v.challenges
        .iter()
        .find(|c| c.number == number)
        .expect("the row is listed")
        .state
        .clone()
}

#[test]
fn the_state_shapes_are_pinned() {
    assert_eq!(
        to_value(ChallengeStateView::Done).expect("serializes"),
        json!({ "kind": "done" })
    );
    // The field is camelCased inside a struct variant: without `rename_all_fields` it comes
    // out snake_case and the screen reads `undefined`, with no error anywhere.
    assert_eq!(
        to_value(ChallengeStateView::Blocked { missing: vec![2] }).expect("serializes"),
        json!({ "kind": "blocked", "missing": [2] })
    );
    assert_eq!(
        to_value(ChallengeStateView::Unknown).expect("serializes"),
        json!({ "kind": "unknown" })
    );
}

#[test]
fn a_set_cell_is_done_and_the_cell_is_the_challenge_number() {
    let v = view(Some(&CELLS), Some(&DONE));
    assert_eq!(state(&v, 1), ChallengeStateView::Done);
    assert_ne!(
        state(&v, 2),
        ChallengeStateView::Done,
        "cell 2 is off: reading the cells one across would light this one"
    );
}

#[test]
fn a_challenge_with_no_gate_is_available_and_a_gated_one_is_not() {
    let none = [false, false, false];
    let v = challenges_view(
        Some(&catalog()),
        None,
        Some(&[false; 4]),
        Some(&none),
        |_| None,
    );
    assert_eq!(state(&v, 1), ChallengeStateView::Available);
    assert_eq!(
        state(&v, 2),
        ChallengeStateView::Blocked { missing: vec![1] }
    );
}

#[test]
fn a_blocked_challenge_names_the_gates_it_is_waiting_for() {
    // Achievement 1 done, 2 not: challenge 3 waits for 2 alone, and says which.
    let v = view(Some(&CELLS), Some(&DONE));
    assert_eq!(
        state(&v, 3),
        ChallengeStateView::Blocked { missing: vec![2] }
    );
}

#[test]
fn every_gate_done_makes_it_available() {
    let all = [false, true, true];
    let v = challenges_view(
        Some(&catalog()),
        None,
        Some(&[false; 4]),
        Some(&all),
        |_| None,
    );
    assert_eq!(state(&v, 3), ChallengeStateView::Available);
}

#[test]
fn an_unread_section_is_unknown_and_never_not_done() {
    let v = view(None, Some(&DONE));
    assert!(v
        .challenges
        .iter()
        .all(|c| c.state == ChallengeStateView::Unknown));
    assert_eq!(v.totals.done, 0);
    assert_eq!(v.totals.slots, 0, "no section, no length to state");
    assert!(v
        .diagnostics
        .contains(&ChallengesDiagnostic::NoChallengesSection));
}

#[test]
fn the_totals_read_the_section_length_from_the_file() {
    let v = view(Some(&CELLS), Some(&DONE));
    assert_eq!(v.totals.slots, 4, "the sample's own length, never 46");
    assert_eq!(v.totals.challenges, 3);
    assert_eq!(v.totals.done, 1);
}

#[test]
fn without_a_catalog_there_are_no_rows_and_the_cells_still_speak() {
    let v = challenges_view(None, None, Some(&CELLS), Some(&DONE), |_| None);
    assert!(v.challenges.is_empty());
    assert_eq!(
        v.totals.done, 1,
        "the set cells are countable without a single name"
    );
    assert!(v.diagnostics.contains(&ChallengesDiagnostic::NoCatalog));
}

#[test]
fn without_the_wiki_the_conditions_are_absent_and_said_once() {
    let v = view(Some(&CELLS), Some(&DONE));
    assert!(
        v.challenges.iter().all(|c| c.blindfolded.is_none()),
        "no page is not `not blindfolded`"
    );
    assert_eq!(
        v.diagnostics
            .iter()
            .filter(|d| **d == ChallengesDiagnostic::NoWiki)
            .count(),
        1,
        "one diagnostic, not one per row"
    );
}

#[test]
fn a_reward_carries_whether_it_is_earned_and_unread_is_not_earned() {
    let v = view(Some(&CELLS), None);
    assert!(
        v.challenges
            .iter()
            .flat_map(|c| &c.rewards)
            .all(|r| r.done.is_none()),
        "section 1 unread: whether a reward is earned is not known, and not `false`"
    );
    assert!(v
        .diagnostics
        .contains(&ChallengesDiagnostic::NoAchievementSection));
}
