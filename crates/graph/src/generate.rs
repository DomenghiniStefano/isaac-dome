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
            // A concept page is a named thing the game gives no id, and `Target::Concept`
            // is that and nothing else — the same word `Inline::Concept` and
            // `Resolution::Concept` already use, so the chain reads one way through.
            // It was `Target::Pickup` until 2026-09-14, and the name did real damage:
            // 45 of the 49 targets it holds are not in the wiki's pickup table at all
            // (`Hard mode` 38 uses, `Completion Mark` 19, `Donation Machine`, `Chapter 2`,
            // `bed`), so "pickups that are not pickups" read as the bug, and the filter
            // drafted against it would have dropped every one of them — including the
            // thirteen that are real requirements this model cannot express.
            Inline::Concept { page, label } => out.push(RefRow {
                target: Target::Concept { name: page.clone() },
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
        | Target::Concept { .. } => false,
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
        let requirements = match &entry.infobox {
            Infobox::Achievement { requirements, .. } => requirements,
            // An achievement page carrying another infobox is a wiki anomaly, not our
            // error: it contributes no requirement and no inventory row. Named one by one,
            // so an eighth kind of infobox has to be placed here rather than skipped.
            Infobox::Item { .. }
            | Infobox::Trinket { .. }
            | Infobox::Boss { .. }
            | Infobox::Challenge { .. }
            | Infobox::Transformation { .. }
            | Infobox::Character { .. } => continue,
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
