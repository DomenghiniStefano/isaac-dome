//! The run archive as the UI sees it.
//!
//! No path crosses: a session is named by its folder — a date, and nothing about this machine —
//! and the live log has no name to give. Items carry a name only when the catalog is there; an
//! id with no name says "the game is not installed" rather than showing a blank.

use catalog::{Catalog, ItemId, ItemKind, Language};
use serde::Serialize;

/// Where a run came from. Tagged, because one variant carries a name and the other cannot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RunSource {
    /// The log the game is writing now.
    Live,
    /// One online session, by its folder's name: `09_12_2026__13_34_26`.
    Session { name: String },
}

/// How a run ended, as the UI draws it. `Open` is not a failure and must never be drawn as one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RunOutcomeView {
    Won { ending: String },
    Died { killer: String },
    Abandoned,
    Open,
}

/// An item in a run. `name` is `None` without a catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct RunItemRef {
    pub id: u32,
    pub name: Option<String>,
    /// The sprite from the user's own copy of the game, `None` without it. A run draws its
    /// items the way every other list does; the id stays, because a picture nobody can serve
    /// must not take the place of the one thing we know.
    pub icon_url: Option<String>,
}

/// One run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct RunView {
    pub source: RunSource,
    /// The position in its own source. A run is `(source, ordinal)`.
    pub ordinal: u32,
    /// `None` when no item line ever named the character: it is not in the seed line.
    pub character: Option<String>,
    /// The character the log states by id — `Initialized player with Variant 0 and Subtype N`.
    /// It tells a Tainted form from its base, which the name cannot: the game gives both the
    /// same one.
    pub character_id: Option<u32>,
    pub seed_words: String,
    /// The game called this run online. The only free discriminator we have for co-op.
    pub online: bool,
    pub outcome: RunOutcomeView,
    pub floors: u32,
    pub starting_items: Vec<RunItemRef>,
    pub collected: Vec<RunItemRef>,
    pub held_active: Option<RunItemRef>,
    pub achievements: Vec<u32>,
}

/// How many runs, and how they ended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS, Default)]
#[serde(rename_all = "camelCase")]
pub struct RunTotals {
    pub runs: u32,
    pub won: u32,
    pub died: u32,
    pub abandoned: u32,
    pub open: u32,
}

/// Every way the archive can be less than whole, said out loud.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RunsDiagnostic {
    /// Documents has no folder for the game: nothing to watch, and no archive to build.
    NoLogFolder,
    /// The database will not open, and which case it is.
    StoreUnavailable { reason: crate::StoreReason },
    /// Rows that did not parse as events. They are counted, never hidden.
    UnreadableEvents { count: u32 },
    /// No catalog, so items have ids and no names.
    NoCatalog,
    /// Session folders the archive could not read: the runs inside them are not in it.
    UnreadableSessions { count: u32 },
    /// The log the game is writing could not be read: the run being played is not in it.
    LiveLogUnreadable,
}

/// What the archive's own reading met, kept by the app between one reading and the next
/// (card #80, R4). It used to be dropped — `let _ =` on the live log, a backfill's errors
/// thrown away, `NoLogFolder` declared and never sent — and an archive that could not be read
/// then looked exactly like one with nothing in it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArchiveHealth {
    /// Discovery found no folder the game writes its logs to.
    pub no_log_folder: bool,
    /// Session folders the last backfill could not read.
    pub unreadable_sessions: u32,
    /// The last reading of the live log failed.
    pub live_log_unreadable: bool,
}

impl ArchiveHealth {
    pub fn diagnostics(&self) -> Vec<RunsDiagnostic> {
        let ArchiveHealth {
            no_log_folder,
            unreadable_sessions,
            live_log_unreadable,
        } = *self;
        let mut out = Vec::new();
        if no_log_folder {
            out.push(RunsDiagnostic::NoLogFolder);
        }
        if unreadable_sessions > 0 {
            out.push(RunsDiagnostic::UnreadableSessions {
                count: unreadable_sessions,
            });
        }
        if live_log_unreadable {
            out.push(RunsDiagnostic::LiveLogUnreadable);
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct RunsView {
    pub runs: Vec<RunView>,
    pub totals: RunTotals,
    pub diagnostics: Vec<RunsDiagnostic>,
}

/// What the command gathers before this crate can answer.
pub struct RunsInputs<'a> {
    /// Each source with its folded runs, in the order the archive took them in — not the order
    /// they are shown in: that is `ui/src/lib/runs/runOrder.ts`, which reads the one clock the
    /// archive has, a session folder's name (card #80, P8).
    pub sources: Vec<(RunSource, Vec<run::Run>)>,
    pub catalog: Option<&'a Catalog>,
    pub diagnostics: Vec<RunsDiagnostic>,
}

/// Where the fold gets item kinds when the game **is** installed. `run` must not depend on
/// `catalog`; this is the one place that joins them.
pub struct CatalogKinds<'a>(pub &'a Catalog);

impl run::ItemKinds for CatalogKinds<'_> {
    fn kind_of(&self, id: u32) -> run::ItemKind {
        match collectible(self.0, id).map(|i| i.kind) {
            Some(ItemKind::Active) => run::ItemKind::Active,
            Some(ItemKind::Familiar) => run::ItemKind::Familiar,
            Some(ItemKind::Passive) => run::ItemKind::Passive,
            // An item this catalog does not know accumulates rather than replacing: reading an
            // unknown id as an active would silently drop whatever the player was carrying.
            None => run::ItemKind::Passive,
            // `collectible` looks up the three collectible kinds only, so a trinket cannot come
            // back here; named rather than folded into a wildcard (card #80, item 12), so a new
            // kind of item has to be decided here instead of counting as a passive in silence.
            Some(ItemKind::Trinket) => run::ItemKind::Passive,
        }
    }
}

/// The item a log line means by an id.
///
/// The catalog is keyed by `(kind, id)` and a trinket can carry the same number as a
/// collectible — but the line the fold reads is `Adding collectible N`, so the three
/// collectible kinds are the only ones that can be meant. Looking a trinket up here would put
/// the wrong name on a run.
fn collectible(catalog: &Catalog, id: u32) -> Option<&catalog::Item> {
    collectible_of(catalog, id).map(|(_, item)| item)
}

/// The kind as well as the item: an icon is addressed by both, and trying the three kinds is
/// how this crate has always found one — the run only carries the number.
fn collectible_of(catalog: &Catalog, id: u32) -> Option<(ItemKind, &catalog::Item)> {
    [ItemKind::Passive, ItemKind::Active, ItemKind::Familiar]
        .into_iter()
        .find_map(|kind| catalog.item(kind, ItemId(id)).map(|item| (kind, item)))
}

pub fn runs_view(
    inputs: RunsInputs<'_>,
    mut icon: impl FnMut(&crate::icon::IconRef) -> Option<String>,
) -> RunsView {
    let RunsInputs {
        sources,
        catalog,
        mut diagnostics,
    } = inputs;
    if catalog.is_none() {
        diagnostics.push(RunsDiagnostic::NoCatalog);
    }
    let mut named = |id: u32| -> RunItemRef {
        let found = catalog.and_then(|c| collectible_of(c, id));
        RunItemRef {
            id,
            name: found
                .zip(catalog)
                .map(|((_, item), c)| c.text(&item.name, Language::English).to_string()),
            icon_url: found.and_then(|(kind, _)| {
                icon(&crate::icon::IconRef::Item {
                    kind: crate::catalog_view::kind_view(kind),
                    id,
                })
            }),
        }
    };

    let mut runs = Vec::new();
    let mut totals = RunTotals::default();
    for (source, folded) in sources {
        for (ordinal, r) in folded.into_iter().enumerate() {
            let outcome = match r.outcome {
                run::Outcome::Won { ending } => {
                    totals.won += 1;
                    RunOutcomeView::Won { ending }
                }
                run::Outcome::Died { killer } => {
                    totals.died += 1;
                    RunOutcomeView::Died { killer }
                }
                run::Outcome::Abandoned => {
                    totals.abandoned += 1;
                    RunOutcomeView::Abandoned
                }
                run::Outcome::Open => {
                    totals.open += 1;
                    RunOutcomeView::Open
                }
            };
            totals.runs += 1;
            runs.push(RunView {
                source: source.clone(),
                ordinal: ordinal as u32,
                character: r.character,
                character_id: r.character_id,
                seed_words: r.seed_words,
                online: r.seed_kind == run::SeedKind::Net,
                outcome,
                floors: r.floors.len() as u32,
                starting_items: r.starting_items.iter().copied().map(&mut named).collect(),
                collected: r.collected.iter().copied().map(&mut named).collect(),
                held_active: r.held_active.map(&mut named),
                achievements: r.achievements,
            });
        }
    }
    RunsView {
        runs,
        totals,
        diagnostics,
    }
}
