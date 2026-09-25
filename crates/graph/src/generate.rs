//! One pass from `dataset/wiki.json` to `requirements.json`. Runs offline, once per
//! snapshot, never from the app.

use std::collections::BTreeMap;

use wiki::{Dataset, Entry, Infobox, Inline, Target};

use crate::rules::{
    target_key, AchievementRefs, GeneratedFrom, RefRow, Requirements, TargetRow, TransformationRow,
    SCHEMA_VERSION,
};

/// Walks the inline tree and keeps what points at something, in reading order.
/// `Inline::Text` carries no target; `Inline::Edition` wraps content and is recursed into,
/// or the refs inside a DLC-only sentence would vanish silently.
pub fn collect_refs(inline: &[Inline]) -> Vec<RefRow> {
    inline.iter().flat_map(refs_of).collect()
}

fn refs_of(i: &Inline) -> Vec<RefRow> {
    match i {
        Inline::Text { .. } => Vec::new(),
        Inline::Ref { target, label } => vec![RefRow {
            target: target.clone(),
            label: label.clone(),
        }],
        // A concept page is a named thing the game gives no id, and `Target::Concept`
        // is that and nothing else — the same word `Inline::Concept` and
        // `Resolution::Concept` already use, so the chain reads one way through.
        // It was `Target::Pickup` until 2026-09-14, and the name did real damage:
        // 45 of the 49 targets it holds are not in the wiki's pickup table at all
        // (`Hard mode` 38 uses, `Completion Mark` 19, `Donation Machine`, `Chapter 2`,
        // `bed`), so "pickups that are not pickups" read as the bug, and the filter
        // drafted against it would have dropped every one of them — including the
        // thirteen that are real requirements this model cannot express.
        Inline::Concept { page, label } => vec![RefRow {
            target: Target::Concept { name: page.clone() },
            label: label.clone(),
        }],
        Inline::Edition { only: _, inline } => collect_refs(inline),
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

pub fn generate(d: &Dataset) -> Requirements {
    let achievements: BTreeMap<u32, AchievementRefs> = d
        .achievements
        .iter()
        .filter_map(|(&id, entry)| {
            let refs = collect_refs(requirements_of(entry)?);
            Some((id, AchievementRefs { refs }))
        })
        .collect();
    Requirements {
        schema_version: SCHEMA_VERSION,
        generated_from: GeneratedFrom {
            snapshot_at: d.meta.snapshot_at.clone(),
            max_revid: d.meta.max_revid,
        },
        targets: inventory(&achievements),
        achievements,
        transformations: d
            .transformations
            .iter()
            .filter_map(|(&id, e)| transformation_row(e).map(|row| (id, row)))
            .collect(),
    }
}

/// An achievement page's requirement sentence. `None` for a page carrying another infobox:
/// a wiki anomaly, not our error, and it contributes no requirement and no inventory row.
/// Named one by one, so an eighth kind of infobox has to be placed here rather than skipped.
fn requirements_of(entry: &Entry) -> Option<&[Inline]> {
    match &entry.infobox {
        Infobox::Achievement { requirements, .. } => Some(requirements),
        Infobox::Item { .. }
        | Infobox::Trinket { .. }
        | Infobox::Boss { .. }
        | Infobox::Challenge { .. }
        | Infobox::Transformation { .. }
        | Infobox::Character { .. } => None,
    }
}

/// Every target that doesn't reduce by id, once, with how many refs use it and the label of
/// the first. Keyed in a `BTreeMap`, which iterates sorted, so the file is stable across runs
/// — which is what makes the `derived` test mean anything.
fn inventory(achievements: &BTreeMap<u32, AchievementRefs>) -> Vec<TargetRow> {
    achievements
        .values()
        .flat_map(|a| a.refs.iter())
        .filter(|r| !reduces_by_id(&r.target))
        .fold(BTreeMap::<String, TargetRow>::new(), |mut rows, r| {
            rows.entry(target_key(&r.target, &r.label))
                .or_insert_with_key(|key| TargetRow {
                    key: key.clone(),
                    label: r.label.clone(),
                    uses: 0,
                })
                .uses += 1;
            rows
        })
        .into_values()
        .collect()
}

/// A transformation page carrying another infobox is a wiki anomaly, not our error — the
/// same reading `requirements_of` takes.
fn transformation_row(e: &Entry) -> Option<TransformationRow> {
    match &e.infobox {
        Infobox::Transformation {
            requires,
            contributors,
            ..
        } => Some(TransformationRow {
            label: e.title.clone(),
            at_least: *requires,
            items: contributors.clone(),
        }),
        Infobox::Item { .. }
        | Infobox::Trinket { .. }
        | Infobox::Achievement { .. }
        | Infobox::Boss { .. }
        | Infobox::Challenge { .. }
        | Infobox::Character { .. } => None,
    }
}
