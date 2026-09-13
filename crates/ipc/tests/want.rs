//! B37: naming a thing and being told what to play for it. What these tests keep apart is
//! the set of ways the answer can be "nothing": a target nothing unlocks, a thing that is
//! not unlockable at all, and a game that isn't installed are three different sentences.

use ipc::{Target, WantDiagnostic, WantedView};

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

use ipc::{AchievementRef, UnlockTarget};

/// One node per achievement 1..=3, none done.
fn view_of(c: &catalog::Catalog) -> ipc::UnlockView {
    let flags = [false, false, false, false];
    ipc::unlock_view(Some(c), None, Some(&flags), None, None, None, |_| None)
}

#[test]
fn an_item_names_the_achievement_that_grants_it() {
    let c = catalog_with_achievements();
    let v = ipc::want_view(
        Some(&c),
        &view_of(&c),
        None,
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
        None,
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
        None,
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
        None,
        None,
        &Target::Character { id: 9 },
        |_| None,
    );
    assert_eq!(v.wanted, WantedView::Unresolved);
    assert!(v.routes.is_empty());
    assert_eq!(v.diagnostics, vec![WantDiagnostic::NothingUnlocks]);
}
