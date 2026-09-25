//! The Unlock view: one node per slot of section 1, and the pieces a node is made of.

use catalog::{AchievementId, Catalog, Origin, Unlock};
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

/// The Unlock view: one node per slot 1..=N of section 1 of the save. `flags[i]` is
/// slot i; slot 0 is unused (the `slot[id]` mapping, verified on 2026-09-05: 169 items
/// out of 171 seen with the achievement done).
///
/// `flags: None` means "section 1 wasn't read", and it is not the same thing as a save
/// with no achievements: flattening the two cases would make the view claim the catalog
/// has 638 more achievements than the file, which is false. `Some(&[])` stays the
/// degenerate save, with its own diagnostic.
/// `graph` and `eval` travel together or not at all: without a catalog there is no graph,
/// and a node then carries `Partial` with one unknown — never `Computed`, which would read
/// as "nothing is in the way".
///
/// Eight parameters, one past `clippy::too_many_arguments`: the eighth is the catalog's boss
/// keys (card #82, S3). Gathering them into an inputs struct, as `QueueInputs` does, is the
/// restructuring of this module that card #82 leaves to its own item.
#[allow(clippy::too_many_arguments)]
pub fn unlock_view(
    catalog: Option<&Catalog>,
    bosses: &BossKeys,
    dataset: Option<&Dataset>,
    flags: Option<&[bool]>,
    graph: Option<&graph::build::Graph>,
    eval: Option<&graph::evaluate::Eval>,
    progress: Option<&dyn graph::evaluate::Profile>,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> UnlockView {
    let read = flags.unwrap_or(&[]);
    let slots = read.len() as u32;
    let mut nodes = Vec::with_capacity(read.len().saturating_sub(1));
    let (mut done, mut known, mut unknown) = (0u32, 0u32, 0u32);
    for (slot, &flag) in read.iter().enumerate().skip(1) {
        let slot = slot as u32;
        let achievement = catalog.and_then(|c| c.achievement(AchievementId(slot)));
        let (achievement_ref, unlocks, origin) = match (catalog, achievement) {
            (Some(c), Some(a)) => {
                known += 1;
                let unlocks: Vec<UnlockTarget> = c
                    .unlocks(a.id)
                    .iter()
                    .map(|u| target_of(c, bosses, u, dataset, &mut icon))
                    .collect();
                let origin = first_item_origin(c, c.unlocks(a.id));
                (
                    AchievementRef::Known {
                        id: a.id.0,
                        text: a.text.clone(),
                        condition: condition_of(a, dataset),
                        icon_url: icon(&IconRef::Achievement { id: a.id.0 }),
                    },
                    unlocks,
                    origin,
                )
            }
            _ => {
                unknown += 1;
                (AchievementRef::Unknown { slot }, Vec::new(), None)
            }
        };
        if flag {
            done += 1;
        }
        let info = match eval.and_then(|e| e.node(AchievementId(slot))) {
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
        };
        let missing = match (catalog, graph.and_then(|g| g.node(AchievementId(slot)))) {
            (Some(c), Some(n)) => missing_view(c, bosses, dataset, n, read, progress),
            _ => Vec::new(),
        };
        nodes.push(UnlockNode {
            achievement: achievement_ref,
            done: flag,
            unlocks,
            origin,
            missing,
            graph: info,
        });
    }

    let mut diagnostics = Vec::new();
    // The two diagnostics that compare the file against the catalog assume the
    // achievement ids are contiguous 1..=N in the catalog, so that `in_catalog + 1` is
    // the number of slots the catalog expects. That holds on real files (2026-09-05:
    // 1..=637, no gaps) and isn't verified in code: a gap in the ids would cause an
    // undercount, never an overcount.
    match (catalog, flags) {
        (Some(c), Some(_)) => {
            let in_catalog = c.achievements().count() as u32;
            if unknown > 0 {
                diagnostics.push(UnlockDiagnostic::SlotsBeyondCatalog { count: unknown });
            }
            // `slots == 0` (section read but empty) is a degenerate save: the
            // diagnostic still fires, and that's intentional.
            if in_catalog + 1 > slots {
                diagnostics.push(UnlockDiagnostic::CatalogBeyondSlots {
                    count: in_catalog + 1 - slots,
                });
            }
        }
        // Without a catalog every slot is `unknown`: `SlotsBeyondCatalog` would just
        // repeat `NoCatalog` with a number attached. Without section 1 there's nothing
        // to compare.
        (None, _) => diagnostics.push(UnlockDiagnostic::NoCatalog),
        (Some(_), None) => {}
    }
    if flags.is_none() {
        diagnostics.push(UnlockDiagnostic::NoAchievementSection);
    }
    UnlockView {
        nodes,
        totals: UnlockTotals {
            slots,
            done,
            known,
            unknown,
        },
        diagnostics,
    }
}

fn first_item_origin(c: &Catalog, unlocks: &[Unlock]) -> Option<OriginView> {
    match unlocks.first()? {
        Unlock::Item { kind, id } => c.item(*kind, *id)?.origin.map(origin_view),
        Unlock::Character { .. } | Unlock::Boss { .. } | Unlock::Challenge { .. } => None,
    }
}

pub(crate) fn origin_view(o: Origin) -> OriginView {
    match o {
        Origin::Rebirth => OriginView::Rebirth,
        Origin::Afterbirth => OriginView::Afterbirth,
        Origin::AfterbirthPlus => OriginView::AfterbirthPlus,
        Origin::Repentance => OriginView::Repentance,
    }
}
