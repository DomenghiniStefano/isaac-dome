//! One pass from `dataset/wiki.json` to `requirements.json`. Runs offline, once per
//! snapshot, never from the app.

use std::collections::BTreeMap;

use wiki::{Dataset, Infobox, Inline, Target};

use crate::rules::{
    target_key, AchievementRefs, GeneratedFrom, RefRow, Requirements, TargetRow, SCHEMA_VERSION,
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

/// True when the target reduces to an achievement through the catalog's own `unlocked_by`
/// links, and therefore needs no hand verdict.
fn reduces_on_its_own(t: &Target) -> bool {
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

pub fn generate(d: &Dataset) -> Requirements {
    let mut achievements = BTreeMap::new();
    let mut uses: BTreeMap<String, (String, u32)> = BTreeMap::new();
    for (&id, entry) in &d.achievements {
        let Infobox::Achievement { requirements, .. } = &entry.infobox else {
            // An achievement page carrying another infobox is a wiki anomaly, not our
            // error: it contributes no requirement and no inventory row.
            continue;
        };
        let mut refs = Vec::new();
        collect_refs(requirements, &mut refs);
        for r in &refs {
            if reduces_on_its_own(&r.target) {
                continue;
            }
            let key = target_key(&r.target, &r.label);
            let e = uses.entry(key).or_insert((r.label.clone(), 0));
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
            .map(|(key, (label, uses))| TargetRow { key, label, uses })
            .collect(),
    }
}
