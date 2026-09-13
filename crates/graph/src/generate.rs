//! One pass from `dataset/wiki.json` to `requirements.json`. Runs offline, once per
//! snapshot, never from the app.

use std::collections::BTreeMap;

use wiki::{Dataset, Infobox, Inline, Target};

use crate::rules::{
    target_key, AchievementRefs, GeneratedFrom, RefRow, Requirements, TargetRow, TransformationRow,
    SCHEMA_VERSION,
};

/// Walks the inline tree and keeps what points at something. `Inline::Text` carries no
/// target; `Inline::Edition` wraps content and is recursed into, or the refs inside a
/// DLC-only sentence would vanish silently.
pub fn collect_refs(inline: &[Inline], out: &mut Vec<RefRow>) {
    for i in inline {
        match i {
            Inline::Text { .. } => {}
            Inline::Ref { target, label } => out.push(RefRow {
                target: target.clone(),
                label: label.clone(),
            }),
            // A concept page is a named thing with no id in the game — exactly what
            // `Pickup` already models — so it travels as one instead of gaining a variant.
            Inline::Concept { page, label } => out.push(RefRow {
                target: Target::Pickup { name: page.clone() },
                label: label.clone(),
            }),
            Inline::Edition { only: _, inline } => collect_refs(inline, out),
        }
    }
}

/// True when the target is looked up in the catalog by id, and therefore never reaches
/// the verdict table.
fn reduces_by_id(t: &Target) -> bool {
    match t {
        Target::Item { .. }
        | Target::Trinket { .. }
        | Target::Character { .. }
        | Target::Achievement { .. }
        | Target::Challenge { .. } => true,
        Target::Entity { .. }
        | Target::Transformation { .. }
        | Target::Stage { .. }
        | Target::Room { .. }
        | Target::Pickup { .. } => false,
    }
}

/// Whether a verdict is always consulted, and so must exist.
///
/// True for every target in the inventory, entities included. An entity only escapes the
/// verdict table when it resolves to a boss the game itself gates by an achievement — 27
/// of 103 bosses — and the other 76 have to be judged, or a node behind Delirium would
/// read as "nothing in the way". Requiring a verdict for all of them costs a handful of
/// rows that are never read; not requiring them cost a silent hole, found on 2026-09-07.
fn verdict_required(_t: &Target) -> bool {
    true
}

pub fn generate(d: &Dataset) -> Requirements {
    let mut achievements = BTreeMap::new();
    let mut uses: BTreeMap<String, (String, u32, bool)> = BTreeMap::new();
    for (&id, entry) in &d.achievements {
        let Infobox::Achievement { requirements, .. } = &entry.infobox else {
            // An achievement page carrying another infobox is a wiki anomaly, not our
            // error: it contributes no requirement and no inventory row.
            continue;
        };
        let mut refs = Vec::new();
        collect_refs(requirements, &mut refs);
        for r in &refs {
            if reduces_by_id(&r.target) {
                continue;
            }
            let key = target_key(&r.target, &r.label);
            let e = uses
                .entry(key)
                .or_insert((r.label.clone(), 0, verdict_required(&r.target)));
            e.1 += 1;
        }
        achievements.insert(id, AchievementRefs { refs });
    }
    Requirements {
        schema_version: SCHEMA_VERSION,
        generated_from: GeneratedFrom {
            snapshot_at: d.meta.snapshot_at.clone(),
            max_revid: d.meta.max_revid,
        },
        achievements,
        // `BTreeMap` iterates sorted, so the file is stable across runs — which is what
        // makes the `derived` test mean anything.
        targets: uses
            .into_iter()
            .map(|(key, (label, uses, verdict_required))| TargetRow {
                key,
                label,
                uses,
                verdict_required,
            })
            .collect(),
        transformations: d
            .transformations
            .iter()
            .filter_map(|(&id, e)| match &e.infobox {
                Infobox::Transformation {
                    requires,
                    contributors,
                    ..
                } => Some((
                    id,
                    TransformationRow {
                        label: e.title.clone(),
                        at_least: *requires,
                        items: contributors.clone(),
                    },
                )),
                // A transformation page carrying another infobox is a wiki anomaly, not our
                // error — the same reading the achievement walk above takes.
                Infobox::Item { .. }
                | Infobox::Trinket { .. }
                | Infobox::Achievement { .. }
                | Infobox::Boss { .. }
                | Infobox::Challenge { .. }
                | Infobox::Character { .. } => None,
            })
            .collect(),
    }
}
