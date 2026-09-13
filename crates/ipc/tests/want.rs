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
