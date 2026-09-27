//! `wiki_progress` against the real catalog (`samples/packed`) and the newest dated profile in
//! `samples/`. Skips with a note when either is missing — `samples/` is git-ignored, and the
//! suite has to stay green for anyone who clones the repo.

use catalog::Catalog;
use core_save::{Kind, Save};
use ipc::{
    unlock_view, wiki_progress, AchievementRef, PageProgress, PageProgressEntry, Target,
    UnlockInputs, WikiProgressInputs,
};
use unpack::ResourceSet;

fn newest() -> Option<(Catalog, Save)> {
    let packed = test_support::packed_dir()?;
    let series = test_support::dated_series(test_support::REP_PLUS_SERIES);
    let path = series.last()?; // chronological order: the newest is last.
    let c = Catalog::build(|p| ResourceSet::open(&packed).read(p));
    match Save::open(path) {
        Ok(s) => Some((c, s)),
        Err(_) => {
            test_support::skip("the newest dated profile exists but doesn't read");
            None
        }
    }
}

#[test]
fn isaac_is_unlocked() {
    let Some((c, s)) = newest() else { return };
    let ds = wiki::Dataset::embedded().expect("embedded dataset");
    let achievements = s.flags(Kind::Achievements);
    let items = s.flags(Kind::Items);
    let challenges = s.flags(Kind::Challenges);
    let counters = s.u32s(Kind::Counters);
    let bestiary = s.bestiary_tallies();
    let out = wiki_progress(WikiProgressInputs {
        dataset: Ok(ds),
        catalog: Some(&c),
        achievements: achievements.as_deref(),
        items: items.as_deref(),
        challenges: challenges.as_deref(),
        counters: counters.as_deref(),
        bestiary: bestiary.as_ref(),
    });
    let isaac = out
        .pages
        .iter()
        .find(|p| p.target == Target::Character { id: 0 })
        .expect("Isaac has a wiki page and a catalog row");
    let PageProgress::Character { unlocked, .. } = &isaac.progress else {
        panic!("expected a character");
    };
    assert_eq!(*unlocked, Some(true), "Isaac has no unlock condition");
}

/// Achievement 1's `done` must agree with what `unlock_view` — the screen the Unlock page
/// already draws from the same section — says about the same slot. A property, not a pinned
/// value: it holds whichever way the real profile happens to have it.
#[test]
fn achievement_one_agrees_with_unlock_view() {
    let Some((c, s)) = newest() else { return };
    let ds = wiki::Dataset::embedded().expect("embedded dataset");
    let Some(flags) = s.flags(Kind::Achievements) else {
        test_support::skip("section 1 did not read on the newest profile");
        return;
    };

    let unlock = unlock_view(
        UnlockInputs {
            catalog: Some(&c),
            bosses: &ipc::for_tests::bosses(&c),
            dataset: Some(ds),
            flags: Some(&flags),
            graph: None,
            eval: None,
            progress: None,
        },
        |_| None,
    );
    let from_unlock_view = unlock
        .nodes
        .iter()
        .find_map(|n| match n.achievement {
            AchievementRef::Known { id: 1, .. } => Some(n.done),
            AchievementRef::Known { .. } | AchievementRef::Unknown { .. } => None,
        })
        .expect("slot 1 is a known achievement on the real catalog");

    let out = wiki_progress(WikiProgressInputs {
        dataset: Ok(ds),
        catalog: Some(&c),
        achievements: Some(&flags),
        items: None,
        challenges: None,
        counters: None,
        bestiary: None,
    });
    let PageProgress::Achievement { done } = out
        .pages
        .iter()
        .find(|p| p.target == Target::Achievement { id: 1 })
        .map(|p| p.progress.clone())
        .expect("achievement 1 has a wiki page")
    else {
        panic!("expected an achievement");
    };
    assert_eq!(done, from_unlock_view);
}

/// The 2026-09-16 measurement recorded in `docs/save-format.md`: Clotty (`15.0.0`) has been
/// killed by the player at least three times on the historical series' latest profile.
#[test]
fn clotty_has_been_killed_by_the_player_at_least_three_times() {
    let Some((c, s)) = newest() else { return };
    let ds = wiki::Dataset::embedded().expect("embedded dataset");
    let Some(bestiary) = s.bestiary_tallies() else {
        test_support::skip("section 10 did not read on the newest profile");
        return;
    };
    let out = wiki_progress(WikiProgressInputs {
        dataset: Ok(ds),
        catalog: Some(&c),
        achievements: None,
        items: None,
        challenges: None,
        counters: None,
        bestiary: Some(&bestiary),
    });
    let clotty = out
        .pages
        .iter()
        .find(|p| {
            p.target
                == Target::Entity {
                    id: 15,
                    variant: 0,
                    subtype: 0,
                }
        })
        .expect("Clotty has a wiki page");
    let PageProgress::Bestiary { killed_you, .. } = clotty.progress else {
        panic!("expected a bestiary row");
    };
    assert!(
        killed_you >= 3,
        "expected killed_you >= 3 on the 2026-09-16 measurement, got {killed_you}"
    );
}

/// Two-page bestiary keys are real on the embedded dataset too (Review Focus 4): whichever
/// keys collide there, each page must carry that key's own numbers, never a halved total.
#[test]
fn every_shared_bestiary_key_gives_each_page_the_same_numbers() {
    let Some((c, s)) = newest() else { return };
    let ds = wiki::Dataset::embedded().expect("embedded dataset");
    let Some(bestiary) = s.bestiary_tallies() else {
        test_support::skip("section 10 did not read on the newest profile");
        return;
    };
    let out = wiki_progress(WikiProgressInputs {
        dataset: Ok(ds),
        catalog: Some(&c),
        achievements: None,
        items: None,
        challenges: None,
        counters: None,
        bestiary: Some(&bestiary),
    });
    let bestiary_rows: Vec<&PageProgressEntry> = out
        .pages
        .iter()
        .filter(|p| matches!(p.progress, PageProgress::Bestiary { .. }))
        .collect();
    let key_of = |t: &Target| match t {
        Target::Entity {
            id,
            variant,
            subtype,
        } => Some((*id, *variant, *subtype)),
        Target::Item { .. }
        | Target::Trinket { .. }
        | Target::Character { .. }
        | Target::Achievement { .. }
        | Target::Challenge { .. }
        | Target::Transformation { .. }
        | Target::Stage { .. }
        | Target::Room { .. }
        | Target::Concept { .. }
        | Target::Article { .. } => None,
    };
    for a in &bestiary_rows {
        let Some(key) = key_of(&a.target) else {
            continue;
        };
        for b in &bestiary_rows {
            if key_of(&b.target) == Some(key) {
                assert_eq!(
                    a.progress, b.progress,
                    "same bestiary key {key:?}, different pages, must read the same numbers"
                );
            }
        }
    }
}
