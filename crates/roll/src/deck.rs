use serde::{Deserialize, Serialize};

use crate::space::Space;
use crate::target::{Status, Target};

/// Every value of an axis, or the ones named. `All` is not "every id listed": a patch that
/// adds a character has to widen an `All` and leave an `Only` alone, and only two variants can
/// say that.
///
/// A struct variant rather than the spec's newtype: the IPC forbids newtype variants and
/// `SelectionView` is `only { ids }` there, so one shape serves both.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Selection {
    All,
    Only { ids: Vec<u8> },
}

impl Selection {
    pub fn has(&self, id: u8) -> bool {
        match self {
            Selection::All => true,
            Selection::Only { ids } => ids.contains(&id),
        }
    }
}

/// What may be drawn. All four axes are editable and every change is written immediately:
/// there is no save button, by decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub characters: Selection,
    pub columns: Selection,
    /// "anything", rather than "only what I'm missing".
    pub include_taken: bool,
    /// A character you have not unlocked is not a run. An option and not a fixed rule, by
    /// decision; it defaults on.
    pub only_playable: bool,
}

impl Default for Preset {
    fn default() -> Preset {
        Preset {
            characters: Selection::All,
            columns: Selection::All,
            include_taken: false,
            only_playable: true,
        }
    }
}

/// Why the rest is not in the deck. Every target of the space falls in exactly one bucket, so
/// `targets.len()` plus these four is `Space::target_count()` — a property, tested on every
/// preset rather than on one.
///
/// This is what lets an empty deck say *"137 already done, 40 I cannot read, 231 of characters
/// you do not have"* instead of "nothing to draw".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Excluded {
    pub taken: usize,
    pub unreadable: usize,
    pub locked: usize,
    pub filtered: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deck {
    pub targets: Vec<Target>,
    pub excluded: Excluded,
}

/// What a tick on each row would be worth, per axis. Parallel to the space's rows and columns,
/// so index *is* id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contributions {
    pub characters: Vec<usize>,
    pub columns: Vec<usize>,
}

/// One bucket per target, decided in this order: what you chose, then what you can play, then
/// what the file says. The order *is* the exclusivity — a target both filtered and taken is
/// reported as filtered, because your own choice is the one you can undo.
enum Bucket {
    Kept,
    Taken,
    Unreadable,
    Locked,
    Filtered,
}

fn bucket_of(space: &Space, preset: &Preset, target: &Target) -> Bucket {
    let character = character_of(target);
    let column = column_of(target, space);
    if !preset.characters.has(character) || !preset.columns.has(column) {
        return Bucket::Filtered;
    }
    // A row the space does not describe is not something to hide: unknown reads as playable.
    if preset.only_playable && !space.playable(character as usize).unwrap_or(true) {
        return Bucket::Locked;
    }
    // "Anything" asks no question of the file, so nothing can be excluded for its answer.
    if preset.include_taken {
        return Bucket::Kept;
    }
    match space.status(target) {
        Some(Status::Missing) => Bucket::Kept,
        Some(Status::Taken) => Bucket::Taken,
        // A target the space does not hold cannot be claimed missing either.
        Some(Status::Unreadable) | None => Bucket::Unreadable,
    }
}

fn all_targets(space: &Space) -> impl Iterator<Item = Target> + '_ {
    let marks = (0..space.rows()).flat_map(move |r| {
        (0..space.columns()).map(move |c| Target::Mark {
            character: r as u8,
            column: c as u8,
        })
    });
    let greedier = (0..space.rows()).map(|r| Target::Greedier { character: r as u8 });
    marks.chain(greedier)
}

/// The deck a preset leaves, and the name of everything it took out.
pub fn deck(space: &Space, preset: &Preset) -> Deck {
    all_targets(space).fold(
        Deck {
            targets: Vec::new(),
            excluded: Excluded::default(),
        },
        |mut acc, target| {
            match bucket_of(space, preset, &target) {
                Bucket::Kept => acc.targets.push(target),
                Bucket::Taken => acc.excluded.taken += 1,
                Bucket::Unreadable => acc.excluded.unreadable += 1,
                Bucket::Locked => acc.excluded.locked += 1,
                Bucket::Filtered => acc.excluded.filtered += 1,
            }
            acc
        },
    )
}

/// What each row of each axis is worth, computed with **that axis relaxed to `All`**.
///
/// Read off the deck as it stands, every unticked row would count zero — the one number that
/// cannot help you decide whether to tick it. This is the same judgment
/// `ui/src/lib/facets/facetOptions.ts` makes for a facet's own dropdown, and it is what lets
/// the panel say what each tick is worth.
pub fn contributions(space: &Space, preset: &Preset) -> Contributions {
    let by_character = deck(
        space,
        &Preset {
            characters: Selection::All,
            ..preset.clone()
        },
    );
    let by_column = deck(
        space,
        &Preset {
            columns: Selection::All,
            ..preset.clone()
        },
    );
    Contributions {
        characters: (0..space.rows())
            .map(|r| {
                by_character
                    .targets
                    .iter()
                    .filter(|t| character_of(t) as usize == r)
                    .count()
            })
            .collect(),
        columns: (0..space.columns())
            .map(|c| {
                by_column
                    .targets
                    .iter()
                    .filter(|t| column_of(t, space) as usize == c)
                    .count()
            })
            .collect(),
    }
}

fn character_of(target: &Target) -> u8 {
    match *target {
        Target::Mark { character, .. } => character,
        Target::Greedier { character } => character,
    }
}

fn column_of(target: &Target, space: &Space) -> u8 {
    match *target {
        Target::Mark { column, .. } => column,
        Target::Greedier { .. } => space.greed_column() as u8,
    }
}
