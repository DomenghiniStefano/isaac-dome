//! What a node is still missing, resolved to names.

use catalog::{AchievementId, Catalog};
use wiki::{Dataset, Target};

use super::types::{MarkLevelView, RequirementView, ThresholdItemView};
use crate::catalog_view::kind_view;
use crate::target_sprite::BossKeys;
use crate::wiki_target::{self, page_of};

/// The requirements still in the way, resolved to names. What is already satisfied is left
/// out: a node blocked by nothing shows an empty list, and that agrees with `blocked_by`.
/// `Requirement::None` never reaches here — it was judged as gating nothing.
pub(super) fn missing_view(
    c: &Catalog,
    bosses: &BossKeys,
    dataset: Option<&Dataset>,
    node: &graph::build::Node,
    flags: &[bool],
    progress: Option<&dyn graph::evaluate::Profile>,
) -> Vec<RequirementView> {
    let en = catalog::Language::English;
    let done =
        |a: Option<AchievementId>| a.is_some_and(|a| crate::flags::recorded_done(flags, a.0));
    let mut out = Vec::new();
    for r in &node.requirements {
        match r {
            graph::model::Requirement::None => {}
            graph::model::Requirement::Mark {
                character,
                column,
                level,
            } => {
                let Some(p) = progress else { continue };
                let Some(ch) = c.character(*character) else {
                    continue;
                };
                // Reached already: not missing. Cannot say: not this list's job to report
                // it either — the node is `Partial` and that is where it says so.
                match p.mark(*character, *column) {
                    None => {}
                    Some(reached) if reached.is_some_and(|r| r >= *level) => {}
                    Some(_) => out.push(RequirementView::Mark {
                        character: character.0,
                        character_name: c.text(&ch.name, en).to_string(),
                        column: *column,
                        level: level_view(*level),
                    }),
                }
            }
            graph::model::Requirement::Counter { name, at_least } => {
                let Some(p) = progress else { continue };
                let Some(current) = p.counter(*name) else {
                    continue;
                };
                if current < *at_least {
                    out.push(RequirementView::Counter {
                        label: counter_label(*name).to_string(),
                        current,
                        at_least: *at_least,
                    });
                }
            }
            graph::model::Requirement::Character { id } => {
                let Some(ch) = c.character(*id) else { continue };
                if !done(ch.unlocked_by) {
                    out.push(RequirementView::Character {
                        id: id.0,
                        name: c.text(&ch.name, en).to_string(),
                        tainted: ch.tainted,
                        page: page_of(dataset, wiki_target::character(ch)),
                    });
                }
            }
            graph::model::Requirement::Boss { id } => {
                let Some(b) = c.boss(*id) else { continue };
                if !done(b.unlocked_by) {
                    out.push(RequirementView::Boss {
                        id: id.0,
                        name: b.name.clone(),
                        page: wiki_target::boss(bosses, b).and_then(|t| page_of(dataset, t)),
                    });
                }
            }
            graph::model::Requirement::Challenge { id } => {
                let Some(ch) = c.challenge(*id) else { continue };
                let unlocked = ch.unlocked_by.iter().any(|a| done(Some(*a)));
                if !ch.unlocked_by.is_empty() && !unlocked {
                    out.push(RequirementView::Challenge {
                        id: id.0,
                        name: ch.name.clone(),
                        page: page_of(dataset, wiki_target::challenge(ch)),
                    });
                }
            }
            graph::model::Requirement::Item { kind, id } => {
                let Some(i) = c.item(*kind, *id) else {
                    continue;
                };
                if !done(i.unlocked_by) {
                    out.push(RequirementView::Item {
                        item_kind: kind_view(*kind),
                        id: id.0,
                        name: c.text(&i.name, en).to_string(),
                        page: page_of(dataset, wiki_target::item(i)),
                    });
                }
            }
            graph::model::Requirement::Threshold {
                transformation,
                label,
                at_least,
                of,
                unresolved,
            } => {
                let items: Vec<ThresholdItemView> = of
                    .iter()
                    .filter_map(|t| {
                        let i = c.item(t.kind, t.id)?;
                        Some(ThresholdItemView {
                            item_kind: kind_view(t.kind),
                            id: t.id.0,
                            name: c.text(&i.name, en).to_string(),
                            unlocked: done(i.unlocked_by),
                            page: page_of(dataset, wiki_target::item(i)),
                        })
                    })
                    .collect();
                let current = items.iter().filter(|i| i.unlocked).count() as u32;
                // Like `Counter`: it appears only while it is not met. A threshold already
                // reached is not in the way, and the list here is what is in the way.
                if current < *at_least {
                    out.push(RequirementView::Threshold {
                        transformation: *transformation,
                        label: label.clone(),
                        current,
                        at_least: *at_least,
                        of: items,
                        unresolved: *unresolved,
                        page: page_of(
                            dataset,
                            Target::Transformation {
                                id: *transformation,
                            },
                        ),
                    });
                }
            }
            graph::model::Requirement::Gate { gate } => out.push(RequirementView::Gate {
                label: gate
                    .split_once(':')
                    .map(|(_, l)| l)
                    .unwrap_or(gate)
                    .to_string(),
            }),
            graph::model::Requirement::Unknown { label } => out.push(RequirementView::Unknown {
                label: label.clone(),
            }),
        }
    }
    out
}

fn level_view(l: graph::rules::MarkLevel) -> MarkLevelView {
    match l {
        graph::rules::MarkLevel::Base => MarkLevelView::Base,
        graph::rules::MarkLevel::Second => MarkLevelView::Second,
    }
}

/// What the screen calls the tally: the boss's English name, because that is what the
/// player is being asked to go and beat. Not the counter's identifier, which is ours. The
/// name is the column's, the one the matrix header draws: a tally counts one column's boss.
fn counter_label(n: graph::rules::CounterName) -> &'static str {
    crate::marks::boss_name(n.column())
}
