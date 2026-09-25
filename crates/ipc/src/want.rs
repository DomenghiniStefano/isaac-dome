//! "I want this — what do I have to play?", the graph read from the other end.
//!
//! Everything here is a view over what `graph.rs` already computed: this module adds no
//! traversal of its own. It resolves what you named, finds the achievements that grant it,
//! and asks for the chain — in the order the Plan would play it.

use catalog::{AchievementId, Catalog};
use serde::Serialize;
use wiki::Target;

use crate::graph::{AchievementRef, UnlockNode, UnlockTarget, UnlockView};
use crate::icon::IconRef;
use crate::target_sprite::BossKeys;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct WantView {
    pub wanted: WantedView,
    pub routes: Vec<WantRoute>,
    pub diagnostics: Vec<WantDiagnostic>,
}

/// A want is one of two things, and `UnlockTarget` can only be one of them: it has four
/// variants and none is an achievement. Naming *Greedier!* has to reach the node whose
/// target the catalog does not model, so the view carries both — and a third case for a name
/// the catalog no longer resolves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum WantedView {
    Target { target: UnlockTarget },
    Achievement { achievement: AchievementRef },
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct WantRoute {
    pub node: UnlockNode,
    pub state: WantState,
}

/// B37 — a want read from the other end of the graph: you name a thing, and these are the
/// ways to it. `routes` is a list because a challenge can be named by two achievements (14 of
/// 45 are, measured 2026-09-13); an empty list always travels with the diagnostic that says
/// which empty it is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum WantState {
    /// Already yours.
    Done,
    /// Nothing in the way: play it.
    AvailableNow,
    /// The prerequisites in the order the Plan would play them, the wanted node excluded.
    /// `unknown` counts the steps — the final node included — whose requirements the graph
    /// only partly interprets: a chain that cannot see everything says so in a number.
    Chain {
        steps: Vec<UnlockNode>,
        unknown: u32,
    },
    /// Section 1 was not read. The route is named; where you stand is not claimed.
    NoProfile,
}

/// Why the answer is empty or partial. Fieldless: a bare string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum WantDiagnostic {
    NoCatalog,
    NoProfile,
    /// `routes` is empty, and this is why: no node in the graph grants this.
    NothingUnlocks,
    /// A stage, a room, a pickup, a transformation: not a thing you unlock.
    NotUnlockable,
}

pub fn want_view(
    catalog: Option<&Catalog>,
    bosses: &BossKeys,
    view: &UnlockView,
    flags: Option<&[bool]>,
    g: Option<&graph::build::Graph>,
    target: &Target,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> WantView {
    resolved(catalog, bosses, view, flags, g, target, &mut icon).unwrap_or_else(|d| WantView {
        wanted: WantedView::Unresolved,
        routes: Vec::new(),
        diagnostics: vec![d],
    })
}

/// The answer, or the one reason there is none.
fn resolved(
    catalog: Option<&Catalog>,
    bosses: &BossKeys,
    view: &UnlockView,
    flags: Option<&[bool]>,
    g: Option<&graph::build::Graph>,
    target: &Target,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> Result<WantView, WantDiagnostic> {
    if !unlockable(target) {
        return Err(WantDiagnostic::NotUnlockable);
    }
    let c = catalog.ok_or(WantDiagnostic::NoCatalog)?;
    let (wanted, ids) = wanted_of(c, bosses, view, target, icon)?;
    let routes: Vec<WantRoute> = ids
        .iter()
        .filter_map(|id| node_of(view, *id))
        .map(|node| WantRoute {
            state: route_state(node, flags, g, view),
            node: node.clone(),
        })
        .collect();
    if routes.is_empty() {
        return Err(WantDiagnostic::NothingUnlocks);
    }
    // The banner and the rows are two readings of one fact, so one produces the other: the
    // screen never has to scan the rows to know whether it may say where you stand.
    let diagnostics = routes
        .iter()
        .all(|r| r.state == WantState::NoProfile)
        .then_some(WantDiagnostic::NoProfile)
        .into_iter()
        .collect();
    Ok(WantView {
        wanted,
        routes,
        diagnostics,
    })
}

/// What is wanted, and the achievements that would grant it.
///
/// Naming an achievement reaches the node directly: it is the only way to ask for what the
/// catalog models no target for — a mode, an event. *Greedier!* is that case.
fn wanted_of(
    c: &Catalog,
    bosses: &BossKeys,
    view: &UnlockView,
    target: &Target,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> Result<(WantedView, Vec<u32>), WantDiagnostic> {
    match target {
        Target::Achievement { id } => node_of(view, *id)
            .map(|n| {
                (
                    WantedView::Achievement {
                        achievement: n.achievement.clone(),
                    },
                    vec![*id],
                )
            })
            .ok_or(WantDiagnostic::NothingUnlocks),
        Target::Item { .. }
        | Target::Trinket { .. }
        | Target::Character { .. }
        | Target::Challenge { .. }
        | Target::Entity { .. }
        | Target::Transformation { .. }
        | Target::Stage { .. }
        | Target::Room { .. }
        | Target::Concept { .. } => {
            let key = key_of(c, bosses, target).ok_or(WantDiagnostic::NothingUnlocks)?;
            let ids = crate::queue::achievements_unlocking(c, &key);
            match crate::graph::resolve_target(c, bosses, &key, None, icon) {
                Some(t) if !ids.is_empty() => Ok((WantedView::Target { target: t }, ids)),
                _ => Err(WantDiagnostic::NothingUnlocks),
            }
        }
    }
}

/// The state of one route, read from what the node already says. Never from the length of a
/// chain: `missing_chain` answers with an empty list for four different situations, and
/// telling those apart is this view's whole job.
fn route_state(
    node: &UnlockNode,
    flags: Option<&[bool]>,
    g: Option<&graph::build::Graph>,
    view: &UnlockView,
) -> WantState {
    let Some(flags) = flags else {
        return WantState::NoProfile;
    };
    if node.done {
        return WantState::Done;
    }
    if matches!(
        node.graph,
        crate::graph::GraphInfo::Computed {
            available_now: true,
            ..
        }
    ) {
        return WantState::AvailableNow;
    }
    let AchievementRef::Known { id, .. } = node.achievement else {
        return WantState::NoProfile;
    };
    let steps = planned_steps(AchievementId(id), flags, g, view);
    let unknown = steps
        .iter()
        .chain(std::iter::once(node))
        .filter(|n| matches!(n.graph, crate::graph::GraphInfo::Partial { .. }))
        .count() as u32;
    // An empty chain with nothing unreadable is not a chain: it is the node being playable
    // right now, which is a different sentence and has its own state.
    if steps.is_empty() && unknown == 0 {
        return WantState::AvailableNow;
    }
    WantState::Chain { steps, unknown }
}

/// The prerequisites of `wanted` in the order the Plan would play them, `wanted` itself left
/// out.
///
/// No graph is not a fifth situation: `unlock_view` already writes `Partial { unknown: 1 }`
/// for a slot the graph says nothing about, so the chain comes out empty and `unknown`
/// counts it. The route then reads "I can't tell you the series", never "nothing missing".
///
/// The order is the queue's, asked rather than reinvented: `enqueue` appends the chain,
/// then the wish, then runs the repair that pulls the prerequisites above it. An empty
/// throwaway queue makes this preview and the write the Plan performs one computation.
fn planned_steps(
    wanted: AchievementId,
    flags: &[bool],
    g: Option<&graph::build::Graph>,
    view: &UnlockView,
) -> Vec<UnlockNode> {
    let chain = g
        .map(|g| g.missing_chain(wanted, &graph::evaluate::FlagsOnly(Some(flags))))
        .unwrap_or_default();
    let rows: Vec<AchievementId> = chain.iter().copied().chain([wanted]).collect();
    let deps = match g {
        Some(g) => crate::queue::GraphDeps::new(g, Some(flags), &rows),
        None => crate::queue::GraphDeps::from_chains([]),
    };
    let mut queue = plan::Queue::from_rows(Vec::new());
    queue.enqueue(wanted, &chain, &deps);
    queue
        .rows()
        .iter()
        .filter(|r| r.achievement != wanted)
        .filter_map(|r| node_of(view, r.achievement.0))
        .cloned()
        .collect()
}

/// The node for an achievement, or nothing: `UnlockView` has one node per save slot, so an
/// achievement the catalog knows and this save has no slot for simply has no route.
fn node_of(view: &UnlockView, achievement: u32) -> Option<&UnlockNode> {
    view.nodes
        .iter()
        .find(|n| matches!(n.achievement, AchievementRef::Known { id, .. } if id == achievement))
}

/// The name you typed, as the key the catalog indexes unlocks by. The conversion lives here
/// and not on the frontend: an item's kind and a boss's entity triple are things only the
/// catalog knows, and a key assembled from a page identity would be a second mapping.
fn key_of(c: &Catalog, bosses: &BossKeys, t: &Target) -> Option<crate::goals::TargetKey> {
    use crate::catalog_view::{kind_view, ItemKindView};
    use crate::goals::TargetKey;
    use catalog::{ChallengeId, CharacterId, ItemId, ItemKind};
    match t {
        Target::Item { id } => crate::catalog_view::collectible(c, *id).map(|i| TargetKey::Item {
            item_kind: kind_view(i.kind),
            id: i.id.0,
        }),
        Target::Trinket { id } => c
            .item(ItemKind::Trinket, ItemId(*id))
            .map(|i| TargetKey::Item {
                item_kind: ItemKindView::Trinket,
                id: i.id.0,
            }),
        Target::Character { id } => c
            .character(CharacterId(*id))
            .map(|ch| TargetKey::Character { id: ch.id.0 }),
        Target::Challenge { number } => c
            .challenge(ChallengeId(*number))
            .map(|ch| TargetKey::Challenge { id: ch.id.0 }),
        // The boss is found by the same key `wiki_target` writes: one mapping, read in both
        // directions, so a row and its page can never disagree about which is which.
        Target::Entity { .. } => c
            .bosses()
            .find(|b| crate::wiki_target::boss(bosses, b).as_ref() == Some(t))
            .map(|b| TargetKey::Boss { id: b.id.0 }),
        Target::Achievement { .. }
        | Target::Transformation { .. }
        | Target::Stage { .. }
        | Target::Room { .. }
        | Target::Concept { .. } => None,
    }
}

/// The four kinds the graph never grants. Written as a `match` with no `_` arm so that a new
/// `Target` variant breaks this build rather than falling silently into "not unlockable".
fn unlockable(t: &Target) -> bool {
    match t {
        Target::Item { .. }
        | Target::Trinket { .. }
        | Target::Character { .. }
        | Target::Achievement { .. }
        | Target::Challenge { .. }
        | Target::Entity { .. } => true,
        Target::Transformation { .. }
        | Target::Stage { .. }
        | Target::Room { .. }
        | Target::Concept { .. } => false,
    }
}
