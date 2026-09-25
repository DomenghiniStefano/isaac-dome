//! The forty-five challenges as the UI reads them: section 7 joined with `challenges.xml`, and
//! the wiki's page for the conditions that decide whether one is playable tonight.
//!
//! **Challenge `n` is cell `n`, and cell 0 is unused** — measured 2026-09-17 and recorded in
//! `docs/save-format.md`. The count comes from the section itself, never from the constant 46.

use catalog::Catalog;
use serde::Serialize;
use wiki::{Dataset, Infobox, Inline, Target};

use crate::flags::{recorded, recorded_done};
use crate::icon::IconRef;
use crate::wiki_target::page_of;

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ChallengesView {
    pub challenges: Vec<ChallengeRow>,
    pub totals: ChallengeTotals,
    pub diagnostics: Vec<ChallengesDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeTotals {
    /// Section 7's own length; 0 when it wasn't read. Never the constant 46.
    pub slots: u32,
    pub challenges: u32,
    pub done: u32,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeRow {
    pub number: u32,
    pub name: String,
    pub state: ChallengeStateView,
    /// What finishing it grants. Empty for the challenges that grant nothing.
    pub rewards: Vec<RewardView>,
    /// The character the wiki says it forces. `None` is "the dataset has no page for this
    /// challenge, or the page names no character" — never "any character".
    pub character: Option<Target>,
    /// That character's name, read from **its own** wiki page's title. The reference the
    /// infobox carries is an id, and a screen cannot name an id: without this the row would
    /// have to invent a word. `None` when the dataset has no page for that character.
    pub character_name: Option<String>,
    pub goal: Option<Vec<Inline>>,
    /// `None` when there is no page. It must not draw as "not blindfolded".
    pub blindfolded: Option<bool>,
    pub page: Option<Target>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct RewardView {
    pub achievement: u32,
    pub text: Option<String>,
    pub icon_url: Option<String>,
    pub page: Option<Target>,
    /// Whether the save says it is earned. `None` when section 1 wasn't read: unread is never
    /// "not earned".
    pub done: Option<bool>,
}

/// Tagged, because `Blocked` carries the gates it waits for — the repo's rule, and the shape
/// `LockView` already has.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ChallengeStateView {
    Done,
    Available,
    /// The achievements that are not done yet, by id. **Named rather than counted**: the
    /// reading of `unlocked_by` as "all of these" is not settled, so a reader has to be able
    /// to see the claim and disbelieve it.
    Blocked {
        missing: Vec<u32>,
    },
    /// Section 7 wasn't read. Never "not done".
    Unknown,
}

/// What the list could not take into account. Fieldless: a bare string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum ChallengesDiagnostic {
    NoCatalog,
    NoChallengesSection,
    NoAchievementSection,
    NoWiki,
}

/// Section 7, `challenges.xml` and the wiki, joined. Every absence travels in the payload:
/// nothing here is an error, and nothing unread is reported as a fact.
pub fn challenges_view(
    catalog: Option<&Catalog>,
    dataset: Option<&Dataset>,
    challenges: Option<&[bool]>,
    achievements: Option<&[bool]>,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> ChallengesView {
    let slots = challenges.map_or(0, |f| f.len() as u32);
    let diagnostics = diagnostics_of(catalog, dataset, challenges, achievements);
    let Some(c) = catalog else {
        return uncatalogued(slots, challenges, diagnostics);
    };
    let profile = Profile {
        challenges,
        achievements,
    };
    let rows = challenge_rows(c, dataset, profile, &mut icon);
    let done = rows
        .iter()
        .filter(|r| r.state == ChallengeStateView::Done)
        .count() as u32;
    ChallengesView {
        totals: ChallengeTotals {
            slots,
            challenges: rows.len() as u32,
            done,
        },
        challenges: rows,
        diagnostics,
    }
}

/// One diagnostic per input that was not there, in the order the screen lists them.
fn diagnostics_of(
    catalog: Option<&Catalog>,
    dataset: Option<&Dataset>,
    challenges: Option<&[bool]>,
    achievements: Option<&[bool]>,
) -> Vec<ChallengesDiagnostic> {
    [
        challenges
            .is_none()
            .then_some(ChallengesDiagnostic::NoChallengesSection),
        achievements
            .is_none()
            .then_some(ChallengesDiagnostic::NoAchievementSection),
        dataset.is_none().then_some(ChallengesDiagnostic::NoWiki),
        catalog.is_none().then_some(ChallengesDiagnostic::NoCatalog),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// The view without a catalog: no names and no ids to list, but the cells are still countable,
/// and saying so is the difference between "we could not read the game" and "you have done
/// none". Cell 0 is no challenge, exactly as slot 0 is no item.
fn uncatalogued(
    slots: u32,
    challenges: Option<&[bool]>,
    diagnostics: Vec<ChallengesDiagnostic>,
) -> ChallengesView {
    let set_cells = challenges.map_or(0, |f| f.iter().skip(1).filter(|b| **b).count() as u32);
    ChallengesView {
        challenges: Vec::new(),
        totals: ChallengeTotals {
            slots,
            challenges: 0,
            done: set_cells,
        },
        diagnostics,
    }
}

/// Every challenge of the catalog, by number, as a row.
fn challenge_rows(
    c: &Catalog,
    dataset: Option<&Dataset>,
    profile: Profile<'_>,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> Vec<ChallengeRow> {
    let mut listed: Vec<_> = c.challenges().collect();
    listed.sort_by_key(|ch| ch.id.0);
    listed
        .iter()
        .map(|ch| challenge_row(c, dataset, profile, ch, icon))
        .collect()
}

/// The two sections a row is read against, either of which may not have been read.
#[derive(Clone, Copy)]
struct Profile<'a> {
    challenges: Option<&'a [bool]>,
    achievements: Option<&'a [bool]>,
}

fn challenge_row(
    c: &Catalog,
    dataset: Option<&Dataset>,
    profile: Profile<'_>,
    ch: &catalog::Challenge,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> ChallengeRow {
    let number = ch.id.0;
    let facts = challenge_facts(dataset, number);
    let character = facts.as_ref().and_then(|f| f.character.clone());
    let character_name = character
        .as_ref()
        .and_then(|t| dataset.and_then(|d| d.entry(t)))
        .map(|e| e.title.clone());
    ChallengeRow {
        number,
        name: ch.name.clone(),
        state: state_of(ch, profile),
        rewards: ch
            .rewards
            .iter()
            .map(|a| reward_view(c, dataset, profile.achievements, *a, icon))
            .collect(),
        character,
        character_name,
        goal: facts.as_ref().map(|f| f.goal.clone()),
        blindfolded: facts.as_ref().map(|f| f.blindfolded),
        page: page_of(dataset, Target::Challenge { number }),
    }
}

/// Unread is `Unknown`, never "not done"; a challenge not done is available when every
/// achievement gating it is done, and blocked by the ones that are not.
fn state_of(ch: &catalog::Challenge, profile: Profile<'_>) -> ChallengeStateView {
    let missing: Vec<u32> = ch
        .unlocked_by
        .iter()
        .filter(|a| !profile.achievements.is_some_and(|f| recorded_done(f, a.0)))
        .map(|a| a.0)
        .collect();
    match recorded(profile.challenges, ch.id.0) {
        None => ChallengeStateView::Unknown,
        Some(true) => ChallengeStateView::Done,
        Some(false) if missing.is_empty() => ChallengeStateView::Available,
        Some(false) => ChallengeStateView::Blocked { missing },
    }
}

fn reward_view(
    c: &Catalog,
    dataset: Option<&Dataset>,
    achievements: Option<&[bool]>,
    a: catalog::AchievementId,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> RewardView {
    RewardView {
        achievement: a.0,
        text: c.achievement(a).map(|x| x.text.clone()),
        icon_url: icon(&IconRef::Achievement { id: a.0 }),
        page: page_of(dataset, Target::Achievement { id: a.0 }),
        done: recorded(achievements, a.0),
    }
}

/// What the wiki's challenge infobox says about one challenge.
struct ChallengeFacts {
    character: Option<Target>,
    goal: Vec<Inline>,
    blindfolded: bool,
}

/// The challenge's own infobox, when the dataset has the page and the page has one.
///
/// `let … else` and not a `match` with a `_` arm: this reads one variant of an open
/// catalogue of infoboxes, it does not claim to enumerate them.
fn challenge_facts(dataset: Option<&Dataset>, number: u32) -> Option<ChallengeFacts> {
    let entry = dataset?.entry(&Target::Challenge { number })?;
    let Infobox::Challenge {
        character,
        goal,
        blindfolded,
        ..
    } = &entry.infobox
    else {
        return None;
    };
    Some(ChallengeFacts {
        character: character.clone(),
        goal: goal.clone(),
        blindfolded: *blindfolded,
    })
}
