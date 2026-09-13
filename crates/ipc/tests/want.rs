//! B37: naming a thing and being told what to play for it. What these tests keep apart is
//! the set of ways the answer can be "nothing": a target nothing unlocks, a thing that is
//! not unlockable at all, and a game that isn't installed are three different sentences.

use ipc::{AchievementRef, GraphInfo, Target, UnlockTarget, WantDiagnostic, WantState, WantedView};

mod support;
use support::{catalog_with_achievements, empty_view};

#[test]
fn a_stage_is_not_a_thing_you_unlock() {
    let c = catalog_with_achievements();
    let v = ipc::want_view(
        Some(&c),
        &empty_view(),
        None,
        None,
        &Target::Stage {
            name: "Basement".into(),
        },
        |_| None,
    );
    assert_eq!(v.wanted, WantedView::Unresolved);
    assert!(v.routes.is_empty());
    assert_eq!(v.diagnostics, vec![WantDiagnostic::NotUnlockable]);
}

#[test]
fn without_a_catalog_nothing_resolves_and_the_view_says_so() {
    let v = ipc::want_view(
        None,
        &empty_view(),
        None,
        None,
        &Target::Item { id: 2 },
        |_| None,
    );
    assert_eq!(v.wanted, WantedView::Unresolved);
    assert!(v.routes.is_empty());
    assert_eq!(v.diagnostics, vec![WantDiagnostic::NoCatalog]);
}

/// One node per achievement 1..=3, none done, all playable right now. These four tests are
/// about *which* nodes a name reaches; they read a profile so that the states they make no
/// claim about don't become the `NoProfile` diagnostic and drown the ones they do.
fn view_of(c: &catalog::Catalog) -> ipc::UnlockView {
    view_with(c, &[], COMPUTED_NOW)
}

/// The profile those four tests read: four slots, nothing done.
const READ: [bool; 4] = [false; 4];

#[test]
fn an_item_names_the_achievement_that_grants_it() {
    let c = catalog_with_achievements();
    let v = ipc::want_view(
        Some(&c),
        &view_of(&c),
        Some(&READ),
        None,
        &Target::Item { id: 2 },
        |_| None,
    );
    assert!(matches!(
        &v.wanted,
        WantedView::Target {
            target: UnlockTarget::Item { id: 2, .. }
        }
    ));
    assert_eq!(v.routes.len(), 1);
    assert!(matches!(
        &v.routes[0].node.achievement,
        AchievementRef::Known { id: 1, .. }
    ));
    assert!(v.diagnostics.is_empty());
}

#[test]
fn a_challenge_named_by_two_achievements_shows_two_routes() {
    let c = catalog_with_achievements();
    let v = ipc::want_view(
        Some(&c),
        &view_of(&c),
        Some(&READ),
        None,
        &Target::Challenge { number: 4 },
        |_| None,
    );
    let ids: Vec<u32> = v
        .routes
        .iter()
        .filter_map(|r| match r.node.achievement {
            AchievementRef::Known { id, .. } => Some(id),
            AchievementRef::Unknown { .. } => None,
        })
        .collect();
    assert_eq!(ids, vec![1, 2], "both ways in, neither hidden");
}

#[test]
fn an_achievement_named_directly_is_its_own_route() {
    // *Greedier!* is the case: the catalog models no target for it, so the only way to reach
    // that node is to name the achievement itself.
    let c = catalog_with_achievements();
    let v = ipc::want_view(
        Some(&c),
        &view_of(&c),
        Some(&READ),
        None,
        &Target::Achievement { id: 2 },
        |_| None,
    );
    assert!(matches!(
        &v.wanted,
        WantedView::Achievement {
            achievement: AchievementRef::Known { id: 2, .. }
        }
    ));
    assert_eq!(v.routes.len(), 1);
    assert!(v.diagnostics.is_empty());
}

#[test]
fn a_thing_no_achievement_grants_says_which_empty_it_is() {
    let c = catalog_with_achievements();
    // Character 7 is granted by achievement 2; character 9 is in no file at all.
    let v = ipc::want_view(
        Some(&c),
        &view_of(&c),
        Some(&READ),
        None,
        &Target::Character { id: 9 },
        |_| None,
    );
    assert_eq!(v.wanted, WantedView::Unresolved);
    assert!(v.routes.is_empty());
    assert_eq!(v.diagnostics, vec![WantDiagnostic::NothingUnlocks]);
}

/// The same view, with the achievements in `done` marked and the graph's verdict forced.
fn view_with(c: &catalog::Catalog, done: &[u32], info: GraphInfo) -> ipc::UnlockView {
    let mut flags = [false; 4];
    for id in done {
        flags[*id as usize] = true;
    }
    let mut v = ipc::unlock_view(Some(c), None, Some(&flags), None, None, None, |_| None);
    for n in v.nodes.iter_mut() {
        n.graph = info.clone();
    }
    v
}

const COMPUTED_NOW: GraphInfo = GraphInfo::Computed {
    available_now: true,
    blocked_by: 0,
    fan_out: 0,
    steps_missing: 0,
};

#[test]
fn a_want_you_already_have_says_so() {
    let c = catalog_with_achievements();
    let v = view_with(&c, &[1], COMPUTED_NOW);
    let w = ipc::want_view(
        Some(&c),
        &v,
        Some(&[false; 4]),
        None,
        &Target::Item { id: 2 },
        |_| None,
    );
    assert_eq!(w.routes[0].state, WantState::Done);
}

#[test]
fn a_want_with_nothing_in_the_way_is_available_now() {
    let c = catalog_with_achievements();
    let v = view_with(&c, &[], COMPUTED_NOW);
    let w = ipc::want_view(
        Some(&c),
        &v,
        Some(&[false; 4]),
        None,
        &Target::Item { id: 2 },
        |_| None,
    );
    assert_eq!(w.routes[0].state, WantState::AvailableNow);
}

#[test]
fn without_the_section_the_view_names_the_route_and_claims_nothing() {
    let c = catalog_with_achievements();
    let v = view_with(&c, &[], COMPUTED_NOW);
    let w = ipc::want_view(Some(&c), &v, None, None, &Target::Item { id: 2 }, |_| None);
    assert_eq!(w.routes[0].state, WantState::NoProfile);
    assert_eq!(w.diagnostics, vec![WantDiagnostic::NoProfile]);
}

#[test]
fn the_no_profile_diagnostic_and_the_rows_cannot_disagree() {
    // The banner and the rows are two readings of one fact: the diagnostic is emitted if and
    // only if every route is NoProfile. A screen that trusted the banner while a row said
    // something else would draw a lie either way round.
    let c = catalog_with_achievements();
    for flags in [None, Some(&[false; 4][..])] {
        let v = view_with(&c, &[], COMPUTED_NOW);
        let w = ipc::want_view(Some(&c), &v, flags, None, &Target::Item { id: 2 }, |_| None);
        let all_rows = w.routes.iter().all(|r| r.state == WantState::NoProfile);
        let banner = w.diagnostics.contains(&WantDiagnostic::NoProfile);
        assert_eq!(all_rows, banner);
    }
}
