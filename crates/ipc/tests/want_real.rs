//! B37 against the real catalog and real profiles. Three facts a synthetic catalog cannot
//! give: that a deep want exists at all, that the preview's order is the queue's own, and how
//! many targets really have more than one way in.
//!
//! Every one carries a vacuity guard. A property about chains holds trivially on a profile
//! with no chains, and "3 passed" reads exactly like "3 verified".
//!
//! **The profile is not the usual `live` one, and that is the result of a measurement.** The
//! graph is flat: chains of achievements are short, and they get shorter as a profile
//! advances, because a prerequisite already earned is not in the chain. Measured 2026-09-13
//! over five samples, the deepest chain anywhere is achievement 509's —
//!
//! ```text
//! 20240606 (286 done)      3 steps
//! 20250626 (302 done)      3
//! 20260629 (305 done)      3
//! 20260912 co-op (128)     4     <- the one these tests read
//! live     (379 done)      1
//! ```
//!
//! so the ordering property is checked where there is an order to check.

// A test that extracts one variant panics on every other, the ones added later included: here
// the wildcard *is* the assertion, and it fails loudly on a new variant instead of hiding it.
#![allow(clippy::wildcard_enum_match_arm)]

use catalog::Catalog;
use core_save::{Kind, Save};
use ipc::{AchievementRef, TargetKey, UnlockView, WantState};
use unpack::ResourceSet;

/// The young profile: the other participant's, from an online session (see `CLAUDE.md`,
/// "Real-world paths"). It is the only sample early enough in the progression to still have
/// a chain worth ordering.
const YOUNG: &str = "20260912.coop-partner.persistentgamedata1.dat";

/// Everything the view needs, built the way `crates/app/src/commands/graph.rs` builds it.
fn setup(name: &str) -> Option<(Catalog, graph::build::Graph, Vec<bool>, UnlockView)> {
    let packed = test_support::packed_dir()?;
    let path = test_support::sample(name)?;
    let rs = ResourceSet::open(&packed);
    let c = Catalog::build(|p| rs.read(p));
    let s = match Save::open(&path) {
        Ok(s) => s,
        Err(_) => {
            test_support::skip(&format!("{name} exists but doesn't read"));
            return None;
        }
    };
    let flags = s.flags(Kind::Achievements)?;
    let counters = s.u32s(Kind::Counters)?;
    let g = graph::build::Graph::build(&c, graph::rules::embedded().expect("embedded rules"));
    let progress = ipc::SaveProgress::new(Some(&flags), Some(&counters), Some(&c));
    let e = g.evaluate(&progress);
    let view = ipc::unlock_view(
        Some(&c),
        &ipc::for_tests::bosses(&c),
        wiki::Dataset::embedded().ok(),
        Some(&flags),
        Some(&g),
        Some(&e),
        Some(&progress),
        |_| None,
    );
    Some((c, g, flags, view))
}

fn known(node: &ipc::UnlockNode) -> Option<u32> {
    match node.achievement {
        AchievementRef::Known { id, .. } => Some(id),
        AchievementRef::Unknown { .. } => None,
    }
}

/// The not-done node whose chain is longest, with that chain.
fn deepest(
    view: &UnlockView,
    g: &graph::build::Graph,
    flags: &[bool],
) -> Option<(u32, Vec<graph::AchievementId>)> {
    view.nodes
        .iter()
        .filter(|n| !n.done)
        .filter_map(known)
        .map(|id| {
            (
                id,
                g.missing_chain(
                    graph::AchievementId(id),
                    &graph::evaluate::FlagsOnly(Some(flags)),
                ),
            )
        })
        .max_by_key(|(_, chain)| chain.len())
}

#[test]
fn the_young_profile_has_a_want_with_a_real_chain() {
    let Some((_, g, flags, view)) = setup(YOUNG) else {
        return;
    };
    let (id, chain) = deepest(&view, &g, &flags).expect("the catalog has achievements");
    eprintln!(
        "measured: the deepest want is achievement {id}, {} steps",
        chain.len()
    );
    assert!(
        chain.len() >= 3,
        "vacuity guard: with no chain longer than two, the ordering property below proves \
         nothing. Longest chain found: {}",
        chain.len()
    );
}

#[test]
fn a_wants_chain_is_what_the_queue_would_hold() {
    let Some((c, g, flags, view)) = setup(YOUNG) else {
        return;
    };
    let (id, chain) = deepest(&view, &g, &flags).expect("the catalog has achievements");
    // The want is named the way a player would name it: the thing the node unlocks, when the
    // catalog knows one; otherwise the achievement itself, which is the case 231 nodes are in.
    let node = view
        .nodes
        .iter()
        .find(|n| known(n) == Some(id))
        .expect("the deepest node is in the view");
    let target = match node.unlocks.first() {
        Some(ipc::UnlockTarget::Item { item_kind, id, .. }) => match item_kind {
            ipc::ItemKindView::Trinket => wiki::Target::Trinket { id: *id },
            _ => wiki::Target::Item { id: *id },
        },
        Some(ipc::UnlockTarget::Character { id, .. }) => wiki::Target::Character { id: *id },
        Some(ipc::UnlockTarget::Challenge { id, .. }) => wiki::Target::Challenge { number: *id },
        // A boss's page key is its portrait's, which this test has no business rebuilding;
        // the achievement names the same node.
        Some(ipc::UnlockTarget::Boss { .. }) | None => wiki::Target::Achievement { id },
    };
    let w = ipc::want_view(
        Some(&c),
        &ipc::for_tests::bosses(&c),
        &view,
        Some(&flags),
        Some(&g),
        &target,
        |_| None,
    );
    let route = w
        .routes
        .iter()
        .find(|r| known(&r.node) == Some(id))
        .expect("the want reaches the node it was taken from");
    let WantState::Chain { steps, .. } = &route.state else {
        panic!("expected a chain for a node {} steps deep", chain.len());
    };
    let preview: Vec<u32> = steps.iter().filter_map(known).collect();

    let mut q = plan::Queue::from_rows(vec![]);
    let mut rows = chain.clone();
    rows.push(graph::AchievementId(id));
    q.enqueue(
        graph::AchievementId(id),
        &chain,
        &ipc::GraphDeps::new(&g, Some(&flags), &rows),
    );
    let queued: Vec<u32> = q
        .rows()
        .iter()
        .map(|r| r.achievement.0)
        .filter(|a| *a != id)
        .collect();
    assert_eq!(preview, queued, "the preview is the queue's own order");
}

#[test]
fn how_many_targets_have_more_than_one_way_in() {
    let Some((c, _, _, _)) = setup(YOUNG) else {
        return;
    };
    // A measurement, not a threshold. `routes` is a list because a challenge's `unlocked_by`
    // is one in the game's own file; how often that matters is a number, and it belongs in
    // the report rather than in someone's assumption.
    let total = c.challenges().count();
    let several = c
        .challenges()
        .filter(|ch| {
            ipc::achievements_unlocking(&c, &TargetKey::Challenge { id: ch.id.0 }).len() > 1
        })
        .count();
    eprintln!("measured: {several} of {total} challenges are named by more than one achievement");
    assert!(
        total > 0,
        "vacuity guard: no challenges were read, so the count above says nothing"
    );
    assert!(
        several > 0,
        "vacuity guard: if no target has two routes, the list shape is untested on real data"
    );
}
