//! The Unlock view: one node per slot of section 1, and the pieces a node is made of.

use catalog::{AchievementId, Catalog, Unlock};
use wiki::Dataset;

use super::missing::missing_view;
use super::target::target_of;
use super::types::{
    AchievementRef, GraphInfo, OriginView, UnlockDiagnostic, UnlockNode, UnlockTotals, UnlockView,
};
use crate::goals::UnlockTarget;
use crate::icon::IconRef;
use crate::target_sprite::BossKeys;
use crate::wiki_target;

/// Everything `unlock_view` reads, apart from the icon link. A struct rather than seven
/// parameters, as `QueueInputs` is: a list that long is hard to call correctly.
///
/// `graph` and `eval` travel together or not at all: without a catalog there is no graph,
/// and a node then carries `Partial` with one unknown — never `Computed`, which would read
/// as "nothing is in the way".
#[derive(Clone, Copy)]
pub struct UnlockInputs<'a> {
    pub catalog: Option<&'a Catalog>,
    /// The catalog's boss keys (`boss_keys`), settled once beside it.
    pub bosses: &'a BossKeys,
    /// The embedded wiki dataset, for a condition the game's file does not state and for the
    /// page a requirement links to. `None` links nothing.
    pub dataset: Option<&'a Dataset>,
    /// Section 1 of the save. `None` is "the section wasn't read" — see `unlock_view`.
    pub flags: Option<&'a [bool]>,
    pub graph: Option<&'a graph::build::Graph>,
    pub eval: Option<&'a graph::evaluate::Eval>,
    /// What the save says about marks and tallies, for the requirements the graph answers
    /// from the profile. `None` draws none of them.
    pub progress: Option<&'a dyn graph::evaluate::Profile>,
}

/// The Unlock view: one node per slot 1..=N of section 1 of the save. `flags[i]` is
/// slot i; slot 0 is unused (the `slot[id]` mapping, verified on 2026-09-05: 169 items
/// out of 171 seen with the achievement done).
///
/// `flags: None` means "section 1 wasn't read", and it is not the same thing as a save
/// with no achievements: flattening the two cases would make the view claim the catalog
/// has 638 more achievements than the file, which is false. `Some(&[])` stays the
/// degenerate save, with its own diagnostic.
pub fn unlock_view(
    inputs: UnlockInputs<'_>,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> UnlockView {
    let read = inputs.flags.unwrap_or(&[]);
    let nodes: Vec<UnlockNode> = read
        .iter()
        .enumerate()
        .skip(1)
        .map(|(slot, &done)| node_at(&inputs, read, slot as u32, done, &mut icon))
        .collect();
    let totals = totals_of(read.len() as u32, &nodes);
    let diagnostics = diagnostics(inputs.catalog, inputs.flags, totals);
    UnlockView {
        nodes,
        totals,
        diagnostics,
    }
}

/// The node for one slot: what the catalog names there, whether the save has it, what the
/// graph says about it and what is still in its way.
fn node_at(
    inputs: &UnlockInputs<'_>,
    read: &[bool],
    slot: u32,
    done: bool,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> UnlockNode {
    let id = AchievementId(slot);
    let known = inputs
        .catalog
        .and_then(|c| c.achievement(id).map(|a| (c, a)));
    let (achievement, unlocks, origin) = match known {
        Some((c, a)) => {
            // The edges first and the achievement's own icon after, the order the links have
            // always been asked for in.
            let unlocks = unlocks_of(c, inputs.bosses, a.id, inputs.dataset, icon);
            let achievement = known_ref(a, inputs.dataset, icon);
            (achievement, unlocks, first_item_origin(c, c.unlocks(a.id)))
        }
        None => (AchievementRef::Unknown { slot }, Vec::new(), None),
    };
    let missing = inputs
        .catalog
        .zip(inputs.graph.and_then(|g| g.node(id)))
        .map(|(c, n)| missing_view(c, inputs.bosses, inputs.dataset, n, read, inputs.progress))
        .unwrap_or_default();
    UnlockNode {
        achievement,
        done,
        unlocks,
        origin,
        missing,
        graph: graph_info(inputs.eval.and_then(|e| e.node(id))),
    }
}

fn known_ref(
    a: &catalog::Achievement,
    dataset: Option<&Dataset>,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> AchievementRef {
    AchievementRef::Known {
        id: a.id.0,
        text: a.text.clone(),
        condition: condition_of(a, dataset),
        icon_url: icon(&IconRef::Achievement { id: a.id.0 }),
    }
}

fn unlocks_of(
    c: &Catalog,
    bosses: &BossKeys,
    id: AchievementId,
    dataset: Option<&Dataset>,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> Vec<UnlockTarget> {
    c.unlocks(id)
        .iter()
        .map(|u| target_of(c, bosses, u, dataset, icon))
        .collect()
}

/// The graph's verdict on a node, as the wire carries it.
fn graph_info(node: Option<&graph::evaluate::NodeInfo>) -> GraphInfo {
    match node {
        Some(graph::evaluate::NodeInfo::Computed {
            available_now,
            blocked_by,
            fan_out,
            steps_missing,
        }) => GraphInfo::Computed {
            available_now: *available_now,
            blocked_by: *blocked_by,
            fan_out: *fan_out,
            steps_missing: *steps_missing,
        },
        Some(graph::evaluate::NodeInfo::Partial {
            blocked_by,
            fan_out,
            unknown,
        }) => GraphInfo::Partial {
            blocked_by: *blocked_by,
            fan_out: *fan_out,
            unknown: *unknown,
        },
        // No graph for this slot: no catalog, or a slot beyond it. The graph has
        // nothing to say, which is `Partial` with one unknown — never `Computed`.
        None => GraphInfo::Partial {
            blocked_by: 0,
            fan_out: 0,
            unknown: 1,
        },
    }
}

/// The totals, read off the nodes once they are built: every count is a count of nodes, so it
/// can never disagree with the list beside it.
fn totals_of(slots: u32, nodes: &[UnlockNode]) -> UnlockTotals {
    let known = nodes
        .iter()
        .filter(|n| matches!(n.achievement, AchievementRef::Known { .. }))
        .count() as u32;
    UnlockTotals {
        slots,
        done: nodes.iter().filter(|n| n.done).count() as u32,
        known,
        unknown: nodes.len() as u32 - known,
    }
}

/// What the view could not account for.
///
/// Without a catalog every slot is `unknown`: `SlotsBeyondCatalog` would just repeat
/// `NoCatalog` with a number attached. Without section 1 there's nothing to compare.
fn diagnostics(
    catalog: Option<&Catalog>,
    flags: Option<&[bool]>,
    totals: UnlockTotals,
) -> Vec<UnlockDiagnostic> {
    let compared = match (catalog, flags) {
        (Some(c), Some(_)) => catalog_against_slots(c, totals),
        (None, _) => vec![UnlockDiagnostic::NoCatalog],
        (Some(_), None) => Vec::new(),
    };
    compared
        .into_iter()
        .chain(
            flags
                .is_none()
                .then_some(UnlockDiagnostic::NoAchievementSection),
        )
        .collect()
}

/// The two diagnostics that compare the file against the catalog. They assume the achievement
/// ids are contiguous 1..=N in the catalog, so that `in_catalog + 1` is the number of slots the
/// catalog expects. That holds on real files (2026-09-05: 1..=637, no gaps) and isn't verified
/// in code: a gap in the ids would cause an undercount, never an overcount.
///
/// `slots == 0` (section read but empty) is a degenerate save: `CatalogBeyondSlots` still
/// fires, and that's intentional.
fn catalog_against_slots(c: &Catalog, totals: UnlockTotals) -> Vec<UnlockDiagnostic> {
    let expected = c.achievements().count() as u32 + 1;
    [
        (totals.unknown > 0).then_some(UnlockDiagnostic::SlotsBeyondCatalog {
            count: totals.unknown,
        }),
        (expected > totals.slots).then(|| UnlockDiagnostic::CatalogBeyondSlots {
            count: expected - totals.slots,
        }),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// How to get an achievement, in one line. The game's `unlock_condition` first — it is the
/// game's own words about its own unlock — and the wiki's requirement only where the file
/// says nothing, which is 354 of 637 achievements on the reference profile.
///
/// The wiki's requirement is an inline tree; `wiki::plain` reads it the way a reader would,
/// so a reference becomes its label and an edition wrapper keeps its words. Whitespace-only
/// is no answer: a blank line under a headline reads as a condition nobody wrote.
fn condition_of(a: &catalog::Achievement, dataset: Option<&Dataset>) -> Option<String> {
    if let Some(from_file) = a.unlock_condition.clone() {
        return Some(from_file);
    }
    let entry = dataset?.entry(&wiki_target::achievement(a.id))?;
    let requirements = match &entry.infobox {
        wiki::Infobox::Achievement { requirements, .. } => requirements,
        // An achievement's page with another kind of infobox says no condition. Named one by
        // one, so a new kind of infobox has to be placed here rather than skipped.
        wiki::Infobox::Item { .. }
        | wiki::Infobox::Trinket { .. }
        | wiki::Infobox::Boss { .. }
        | wiki::Infobox::Challenge { .. }
        | wiki::Infobox::Transformation { .. }
        | wiki::Infobox::Character { .. } => return None,
    };
    let line = wiki::plain(requirements).trim().to_string();
    (!line.is_empty()).then_some(line)
}

fn first_item_origin(c: &Catalog, unlocks: &[Unlock]) -> Option<OriginView> {
    match unlocks.first()? {
        Unlock::Item { kind, id } => c.item(*kind, *id)?.origin,
        Unlock::Character { .. } | Unlock::Boss { .. } | Unlock::Challenge { .. } => None,
    }
}
