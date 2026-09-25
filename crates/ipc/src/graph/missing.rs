//! What a node is still missing, resolved to names.

use catalog::{AchievementId, BossId, Catalog, ChallengeId, CharacterId, ItemId, ItemKind};
use graph::model::{Requirement, ThresholdItem};
use graph::rules::{CounterName, MarkColumn, MarkLevel};
use wiki::{Dataset, Target};

use super::types::{MarkLevelView, RequirementView, ThresholdItemView};
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
    let profile = Standing {
        c,
        bosses,
        dataset,
        flags,
        progress,
    };
    node.requirements
        .iter()
        .filter_map(|r| profile.missing(r))
        .collect()
}

/// What one requirement is read against: the catalog that names it, the wiki that links it,
/// and the profile that says whether it is still in the way.
struct Standing<'a> {
    c: &'a Catalog,
    bosses: &'a BossKeys,
    dataset: Option<&'a Dataset>,
    flags: &'a [bool],
    progress: Option<&'a dyn graph::evaluate::Profile>,
}

const EN: catalog::Language = catalog::Language::English;

impl Standing<'_> {
    /// The requirement as the screen names it, or `None` when it is not in the way — met, or
    /// naming something this catalog does not have.
    fn missing(&self, r: &Requirement) -> Option<RequirementView> {
        match r {
            Requirement::None => None,
            Requirement::Mark {
                character,
                column,
                level,
            } => self.mark(*character, *column, *level),
            Requirement::Counter { name, at_least } => self.counter(*name, *at_least),
            Requirement::Character { id } => self.character(*id),
            Requirement::Boss { id } => self.boss(*id),
            Requirement::Challenge { id } => self.challenge(*id),
            Requirement::Item { kind, id } => self.item(*kind, *id),
            Requirement::Threshold {
                transformation,
                label,
                at_least,
                of,
                unresolved,
            } => self.threshold(*transformation, label, *at_least, of, *unresolved),
            Requirement::Gate { gate } => Some(gate_view(gate)),
            Requirement::Unknown { label } => Some(RequirementView::Unknown {
                label: label.clone(),
            }),
        }
    }

    fn done(&self, a: Option<AchievementId>) -> bool {
        a.is_some_and(|a| crate::flags::recorded_done(self.flags, a.0))
    }

    /// Reached already: not missing. Cannot say: not this list's job to report it either — the
    /// node is `Partial` and that is where it says so.
    fn mark(
        &self,
        character: CharacterId,
        column: MarkColumn,
        level: MarkLevel,
    ) -> Option<RequirementView> {
        let reached = self.progress?.mark(character, column)?;
        let ch = self.c.character(character)?;
        reached
            .is_none_or(|r| r < level)
            .then(|| RequirementView::Mark {
                character: character.0,
                character_name: self.c.text(&ch.name, EN).to_string(),
                column,
                level: level_view(level),
            })
    }

    fn counter(&self, name: CounterName, at_least: u32) -> Option<RequirementView> {
        let current = self.progress?.counter(name)?;
        (current < at_least).then(|| RequirementView::Counter {
            label: counter_label(name).to_string(),
            current,
            at_least,
        })
    }

    fn character(&self, id: CharacterId) -> Option<RequirementView> {
        let ch = self.c.character(id)?;
        (!self.done(ch.unlocked_by)).then(|| RequirementView::Character {
            id: id.0,
            name: self.c.text(&ch.name, EN).to_string(),
            tainted: ch.tainted,
            page: page_of(self.dataset, wiki_target::character(ch)),
        })
    }

    fn boss(&self, id: BossId) -> Option<RequirementView> {
        let b = self.c.boss(id)?;
        (!self.done(b.unlocked_by)).then(|| RequirementView::Boss {
            id: id.0,
            name: b.name.clone(),
            page: wiki_target::boss(self.bosses, b).and_then(|t| page_of(self.dataset, t)),
        })
    }

    /// A challenge nothing gates is not in the way; one is when none of its ways in is done.
    fn challenge(&self, id: ChallengeId) -> Option<RequirementView> {
        let ch = self.c.challenge(id)?;
        let unlocked = ch.unlocked_by.iter().any(|a| self.done(Some(*a)));
        (!ch.unlocked_by.is_empty() && !unlocked).then(|| RequirementView::Challenge {
            id: id.0,
            name: ch.name.clone(),
            page: page_of(self.dataset, wiki_target::challenge(ch)),
        })
    }

    fn item(&self, kind: ItemKind, id: ItemId) -> Option<RequirementView> {
        let i = self.c.item(kind, id)?;
        (!self.done(i.unlocked_by)).then(|| RequirementView::Item {
            item_kind: kind,
            id: id.0,
            name: self.c.text(&i.name, EN).to_string(),
            page: page_of(self.dataset, wiki_target::item(i)),
        })
    }

    /// Like `Counter`: it appears only while it is not met. A threshold already reached is not
    /// in the way, and the list here is what is in the way.
    fn threshold(
        &self,
        transformation: u32,
        label: &str,
        at_least: u32,
        of: &[ThresholdItem],
        unresolved: u32,
    ) -> Option<RequirementView> {
        let items: Vec<ThresholdItemView> =
            of.iter().filter_map(|t| self.threshold_item(t)).collect();
        let current = items.iter().filter(|i| i.unlocked).count() as u32;
        (current < at_least).then(|| RequirementView::Threshold {
            transformation,
            label: label.to_string(),
            current,
            at_least,
            of: items,
            unresolved,
            page: page_of(self.dataset, Target::Transformation { id: transformation }),
        })
    }

    fn threshold_item(&self, t: &ThresholdItem) -> Option<ThresholdItemView> {
        let i = self.c.item(t.kind, t.id)?;
        Some(ThresholdItemView {
            item_kind: t.kind,
            id: t.id.0,
            name: self.c.text(&i.name, EN).to_string(),
            unlocked: self.done(i.unlocked_by),
            page: page_of(self.dataset, wiki_target::item(i)),
        })
    }
}

/// A gate is always in the way, named by the label after its `kind:` prefix.
fn gate_view(gate: &str) -> RequirementView {
    RequirementView::Gate {
        label: gate
            .split_once(':')
            .map(|(_, l)| l)
            .unwrap_or(gate)
            .to_string(),
    }
}

fn level_view(l: MarkLevel) -> MarkLevelView {
    match l {
        MarkLevel::Base => MarkLevelView::Base,
        MarkLevel::Second => MarkLevelView::Second,
    }
}

/// What the screen calls the tally: the boss's English name, because that is what the
/// player is being asked to go and beat. Not the counter's identifier, which is ours. The
/// name is the column's, the one the matrix header draws: a tally counts one column's boss.
fn counter_label(n: CounterName) -> &'static str {
    crate::marks::boss_name(n.column())
}
