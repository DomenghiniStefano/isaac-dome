//! The run archive as the UI sees it.
//!
//! No path crosses: a session is named by its folder — a date, and nothing about this machine —
//! and a launch of `log.txt` has no name to give, only a row number and a date. Items carry a
//! name only when the catalog is there; an id with no name says "the game is not installed"
//! rather than showing a blank.

use catalog::{Catalog, ItemId, ItemKind};
use wiki::Dataset;

use crate::run_detail::{EntityRef, PickupView, Resolve, RunAchievementView, RunFloorView};
use serde::Serialize;

/// Where a run came from. Tagged, because each variant carries something different.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RunSource {
    /// The latest launch of `log.txt`: the one the game may be writing now. Its `id` is the same
    /// row number it keeps once it is an older launch, so a run's key does not change with it.
    Live {
        id: i64,
        /// When the launch's file was last written, in epoch seconds; `None` when it was read
        /// before the app kept dates.
        written_unix: Option<i64>,
    },
    /// An older launch. `id` only tells launches apart: it is a row number, nothing on disk.
    Launch {
        id: i64,
        /// When the launch's file was last written, in epoch seconds; `None` when it was read
        /// before the app kept dates.
        written_unix: Option<i64>,
    },
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
    Won {
        ending: String,
    },
    /// The killer and, when somebody spawned it, the spawner: a shot and the monster that fired
    /// it are both half of what killed you.
    Died {
        killer: EntityRef,
        spawner: Option<EntityRef>,
    },
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
    /// The co-op menu head of that character, `None` without a catalog or for a form the menu
    /// does not draw.
    pub character_head_url: Option<String>,
    pub seed_words: String,
    /// The game called this run online. The only free discriminator we have for co-op.
    pub online: bool,
    pub outcome: RunOutcomeView,
    /// How many floors, for the list; `floor_details` is each of them, for the page.
    pub floors: u32,
    pub floor_details: Vec<RunFloorView>,
    pub starting_items: Vec<RunItemRef>,
    /// Everything picked up after the starting window, in order, with its pool and floor.
    pub collected: Vec<PickupView>,
    pub passives: Vec<RunItemRef>,
    pub familiars: Vec<RunItemRef>,
    pub held_active: Option<RunItemRef>,
    pub achievements: Vec<RunAchievementView>,
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

/// What the archive's own reading met, kept by the app between one reading and the next.
/// Without it an archive that could not be read — the live log failing, a backfill's errors,
/// no folder to watch — looks exactly like one with nothing in it.
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
        [
            no_log_folder.then_some(RunsDiagnostic::NoLogFolder),
            (unreadable_sessions > 0).then_some(RunsDiagnostic::UnreadableSessions {
                count: unreadable_sessions,
            }),
            live_log_unreadable.then_some(RunsDiagnostic::LiveLogUnreadable),
        ]
        .into_iter()
        .flatten()
        .collect()
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
    /// they are shown in: that is `ui/src/lib/runs/runOrder.ts`, which orders by the date each
    /// source carries.
    pub sources: Vec<(RunSource, Vec<run::Run>)>,
    pub catalog: Option<&'a Catalog>,
    /// Which entities have a wiki page, so a killer links to one only when it exists.
    pub wiki: Option<&'a Dataset>,
    pub diagnostics: Vec<RunsDiagnostic>,
}

/// Where the fold gets item kinds when the game **is** installed. `run` must not depend on
/// `catalog`; this is the one place that joins them.
pub struct CatalogKinds<'a>(pub &'a Catalog);

impl run::ItemKinds for CatalogKinds<'_> {
    fn kind_of(&self, id: u32) -> run::ItemKind {
        match self.0.collectible(ItemId(id)).map(|i| i.kind) {
            Some(ItemKind::Active) => run::ItemKind::Active,
            Some(ItemKind::Familiar) => run::ItemKind::Familiar,
            Some(ItemKind::Passive) => run::ItemKind::Passive,
            // An item this catalog does not know accumulates rather than replacing: reading an
            // unknown id as an active would silently drop whatever the player was carrying.
            None => run::ItemKind::Passive,
            // `Catalog::collectible` looks up the three collectible kinds only, so a trinket cannot come
            // back here; named rather than folded into a wildcard, so a new kind of item has to
            // be decided here instead of counting as a passive in silence.
            Some(ItemKind::Trinket) => run::ItemKind::Passive,
        }
    }
}

/// The runs of every source, in the order given, with the totals counted off them and the
/// catalog's absence said after whatever the archive's reading already met.
pub fn runs_view(
    inputs: RunsInputs<'_>,
    icon: impl FnMut(&crate::icon::IconRef) -> Option<String>,
) -> RunsView {
    let RunsInputs {
        sources,
        catalog,
        wiki,
        diagnostics,
    } = inputs;
    let mut resolve = Resolve {
        catalog,
        wiki,
        icon,
    };
    let runs: Vec<RunView> = sources
        .into_iter()
        .flat_map(|(source, folded)| {
            folded
                .into_iter()
                .enumerate()
                .map(move |(ordinal, r)| (source.clone(), ordinal as u32, r))
        })
        .map(|(source, ordinal, r)| run_view(source, ordinal, r, &mut resolve))
        .collect();
    RunsView {
        totals: RunTotals::of(&runs),
        runs,
        diagnostics: diagnostics
            .into_iter()
            .chain(catalog.is_none().then_some(RunsDiagnostic::NoCatalog))
            .collect(),
    }
}

fn run_view<F: FnMut(&crate::icon::IconRef) -> Option<String>>(
    source: RunSource,
    ordinal: u32,
    r: run::Run,
    resolve: &mut Resolve<'_, F>,
) -> RunView {
    let items = |ids: &[u32], resolve: &mut Resolve<'_, F>| -> Vec<RunItemRef> {
        ids.iter().map(|id| resolve.item(*id)).collect()
    };
    RunView {
        source,
        ordinal,
        character_head_url: resolve.head(r.character_id, r.character.as_deref()),
        character: r.character,
        character_id: r.character_id,
        seed_words: r.seed_words,
        online: r.seed_kind == run::SeedKind::Net,
        outcome: outcome_view(r.outcome, resolve),
        floors: r.floors.len() as u32,
        floor_details: r.floors.iter().map(|f| resolve.floor(f, r.greed)).collect(),
        starting_items: items(&r.starting_items, resolve),
        collected: r.collected.iter().map(|p| resolve.pickup(p)).collect(),
        passives: items(&r.passives, resolve),
        familiars: items(&r.familiars, resolve),
        held_active: r.held_active.map(|id| resolve.item(id)),
        achievements: r
            .achievements
            .iter()
            .map(|id| resolve.achievement(*id))
            .collect(),
    }
}

fn outcome_view<F: FnMut(&crate::icon::IconRef) -> Option<String>>(
    o: run::Outcome,
    resolve: &mut Resolve<'_, F>,
) -> RunOutcomeView {
    match o {
        run::Outcome::Won { ending } => RunOutcomeView::Won { ending },
        run::Outcome::Died { killer, spawner } => RunOutcomeView::Died {
            killer: resolve.entity(&killer),
            spawner: resolve.spawner(&spawner, &killer),
        },
        run::Outcome::Abandoned => RunOutcomeView::Abandoned,
        run::Outcome::Open => RunOutcomeView::Open,
    }
}

impl RunTotals {
    /// Counted off the finished runs, so the totals can never disagree with the list beside them.
    fn of(runs: &[RunView]) -> RunTotals {
        runs.iter()
            .fold(RunTotals::default(), |t, r| t.counting(&r.outcome))
    }

    fn counting(self, outcome: &RunOutcomeView) -> RunTotals {
        let t = RunTotals {
            runs: self.runs + 1,
            ..self
        };
        match outcome {
            RunOutcomeView::Won { .. } => RunTotals {
                won: t.won + 1,
                ..t
            },
            RunOutcomeView::Died { .. } => RunTotals {
                died: t.died + 1,
                ..t
            },
            RunOutcomeView::Abandoned => RunTotals {
                abandoned: t.abandoned + 1,
                ..t
            },
            RunOutcomeView::Open => RunTotals {
                open: t.open + 1,
                ..t
            },
        }
    }
}
