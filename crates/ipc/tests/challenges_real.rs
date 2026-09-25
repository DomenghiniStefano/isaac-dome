//! The 2026-09-17 measurement, turned into the property that guards it.
//!
//! Challenge `n` is cell `n`, and the instrument is the catalog's reward link: finishing a
//! challenge grants a known set of achievements, so "this row reads done" and "every reward it
//! names is earned" have to agree. Not a pinned number — it holds whatever the profile does
//! next, and it goes red if the mapping ever shifts by one.

mod support;

use core_save::{Kind, Save};
use ipc::{challenges_view, ChallengeStateView};
use support::real_catalog;
use test_support::{dated_series, REP_PLUS_SERIES};

#[test]
fn a_finished_challenge_has_earned_every_achievement_it_rewards() {
    let Some(c) = real_catalog() else {
        test_support::skip("samples/packed is missing, no catalog to join");
        return;
    };
    let files = dated_series(REP_PLUS_SERIES);
    let Some(path) = files.last() else {
        test_support::skip("no rep+ sample: the mapping has nothing to check");
        return;
    };
    let bytes = std::fs::read(path).expect("the sample reads");
    let save = Save::parse(&bytes).expect("the sample parses");
    let view = challenges_view(
        Some(&c),
        wiki::Dataset::embedded().ok(),
        save.flags(Kind::Challenges).as_deref(),
        save.flags(Kind::Achievements).as_deref(),
        |_| None,
    );

    // The vacuity guard: on a profile where nothing is done — or everything — the property
    // below holds without being able to fail, and would report coverage that isn't there.
    let done = view
        .challenges
        .iter()
        .filter(|r| r.state == ChallengeStateView::Done)
        .count();
    assert!(
        done > 0 && done < view.challenges.len(),
        "this profile has {done} of {} done: with none or all, the property is vacuous",
        view.challenges.len()
    );

    for row in &view.challenges {
        if row.state != ChallengeStateView::Done || row.rewards.is_empty() {
            continue;
        }
        let unearned: Vec<u32> = row
            .rewards
            .iter()
            .filter(|r| r.done != Some(true))
            .map(|r| r.achievement)
            .collect();
        assert!(
            unearned.is_empty(),
            "challenge {} reads done and {unearned:?} is not earned — the cell mapping shifted",
            row.number
        );
    }
}

#[test]
fn a_challenge_that_forces_a_character_can_name_it() {
    let Some(c) = real_catalog() else {
        test_support::skip("samples/packed is missing, no challenge list to check");
        return;
    };
    let view = challenges_view(Some(&c), wiki::Dataset::embedded().ok(), None, None, |_| {
        None
    });
    let forced: Vec<&ipc::ChallengeRow> = view
        .challenges
        .iter()
        .filter(|r| r.character.is_some())
        .collect();
    // The guard before the property: if no challenge named a character, the loop below would
    // pass over nothing and report a coverage it does not have.
    assert!(
        !forced.is_empty(),
        "no challenge names a character: the name lookup is never exercised"
    );
    let unnamed: Vec<u32> = forced
        .iter()
        .filter(|r| r.character_name.is_none())
        .map(|r| r.number)
        .collect();
    assert!(
        unnamed.is_empty(),
        "{unnamed:?} force a character the dataset cannot name — the row would have to invent a word"
    );
}

#[test]
fn every_challenge_has_a_wiki_page_in_the_committed_snapshot() {
    let Some(c) = real_catalog() else {
        test_support::skip("samples/packed is missing, no challenge list to check");
        return;
    };
    // No save at all: this asks about the dataset and the catalog, and the rows exist without
    // a profile.
    let view = challenges_view(Some(&c), wiki::Dataset::embedded().ok(), None, None, |_| {
        None
    });
    assert!(
        !view.challenges.is_empty(),
        "no challenges listed at all: the guard below would pass on an empty list"
    );
    let without: Vec<u32> = view
        .challenges
        .iter()
        .filter(|r| r.page.is_none())
        .map(|r| r.number)
        .collect();
    assert!(
        without.is_empty(),
        "the snapshot has no page for {without:?} — those rows lose their conditions in silence"
    );
}
