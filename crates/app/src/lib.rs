//! app — Tauri binary crate. Registers the IPC commands and does the one
//! I/O job that belongs to it (the settings file). The real logic lives in `ipc`.

mod error;
mod settings_file;

use std::sync::{Mutex, MutexGuard, OnceLock};

use catalog::Catalog;
use core_save::{Kind, OpenError, Save};
use discovery::{discover, Options};
use ipc::{ActiveProfile, GraphDeps, MarksMatrix, ProfileId, SaveSummary, Settings, SetupState};
use store::{GoalsRead, Store, StoreError};
use tauri::{AppHandle, Manager};
use unpack::ResourceSet;

pub use error::IpcError;

/// The catalog of the installed game, built on first use and then kept: reading it costs
/// milliseconds, but building it (`Catalog::build` reads every source) doesn't. It
/// doesn't open the `ResourceSet` itself: it receives it already open from the caller,
/// which opens it just once.
#[derive(Default)]
struct CatalogState(OnceLock<Catalog>);

impl CatalogState {
    fn get_or_build(&self, rs: &ResourceSet) -> Option<&Catalog> {
        Some(self.0.get_or_init(|| Catalog::build(|p| rs.read(p))))
    }
}

/// The unlock graph, built once from the catalog and the rules compiled into the binary.
/// Same shape as `CatalogState`: expensive to build, cheap to consult. Rules that don't
/// parse can only be our own broken file, and they degrade like everything else — the
/// commands answer without graph info rather than failing.
#[derive(Default)]
struct GraphState(OnceLock<graph::Graph>);

impl GraphState {
    fn get(&self, catalog: &catalog::Catalog) -> Option<&graph::Graph> {
        if let Some(g) = self.0.get() {
            return Some(g);
        }
        let rules = graph::rules::embedded().ok()?;
        Some(self.0.get_or_init(|| graph::Graph::build(catalog, rules)))
    }
}

/// The game's archives, opened **only once** for the whole life of the process.
///
/// Opening a `ResourceSet` now costs just the index, not the data (`unpack` reads entry by
/// entry), but it's still wasted work if repeated: `discover`, eight `File::open` calls,
/// eight tables re-read on every single command. This is the crate's one and only call to
/// `ResourceSet::open`.
///
/// The "game not installed" case is **not** cached: if it's missing, the next command
/// tries again. Someone who opens the app before installing the game shouldn't have to
/// restart it.
#[derive(Default)]
struct ResourcesState(OnceLock<ResourceSet>);

impl ResourcesState {
    fn get(&self) -> Option<&ResourceSet> {
        if let Some(rs) = self.0.get() {
            return Some(rs);
        }
        let d = discover(&Options::default());
        let dir = d.game.as_ref()?.dir.join("resources").join("packed");
        Some(self.0.get_or_init(|| ResourceSet::open(&dir)))
    }
}

/// The frames of the two anm2 files the completion marks are cut from, read once. Kept only
/// when both read: a failed read — the game absent, an archive missing — is tried again on
/// the next request, like `ResourcesState`.
#[derive(Default)]
struct MarkFramesState(OnceLock<ipc::MarkFrames>);

impl MarkFramesState {
    fn get(&self, rs: &ResourceSet) -> Option<&ipc::MarkFrames> {
        if let Some(f) = self.0.get() {
            return Some(f);
        }
        let widget = catalog::anm2_frames(&rs.read(ipc::WIDGET_ANM2)?)?;
        let lobby = catalog::anm2_frames(&rs.read(ipc::LOBBY_ANM2)?)?;
        Some(self.0.get_or_init(|| ipc::MarkFrames { widget, lobby }))
    }
}

#[tauri::command]
fn setup_state(app: AppHandle) -> Result<SetupState, IpcError> {
    let settings = settings_file::load(&app);
    let d = discover(&Options::default());
    Ok(ipc::setup_state(&d, settings.active_profile_id.as_ref()))
}

#[tauri::command]
fn select_profile(app: AppHandle, id: ProfileId) -> Result<SetupState, IpcError> {
    let d = discover(&Options::default());
    let views = ipc::candidates(&d.saves);
    if !views.iter().any(|c| c.id == id) {
        return Err(IpcError::UnknownProfile {
            id: id.as_str().to_string(),
        });
    }
    // Load the existing settings and update only the field that changed: since the
    // interface's size joined them, switching profile would otherwise put it back to 100.
    let settings = Settings {
        active_profile_id: Some(id),
        ..settings_file::load(&app)
    };
    settings_file::save(&app, &settings)?;
    Ok(ipc::setup_state(&d, settings.active_profile_id.as_ref()))
}

/// The persisted settings, as the app will act on them: the scale comes back snapped to the
/// ladder, so a hand-edited file never puts the interface at a size nothing was drawn at.
#[tauri::command]
fn settings(app: AppHandle) -> Result<Settings, IpcError> {
    let stored = settings_file::load(&app);
    Ok(stored.with_scale(stored.scale()))
}

/// Changes the interface's size and answers the settings as they now are — the same shape as
/// `select_profile`, which also writes and answers. The value is snapped before it reaches
/// the file: what we write is always a size we drew.
#[tauri::command]
fn set_scale(app: AppHandle, percent: u16) -> Result<Settings, IpcError> {
    let settings = settings_file::load(&app).with_scale(percent);
    settings_file::save(&app, &settings)?;
    Ok(settings)
}

#[tauri::command]
fn save_summary(app: AppHandle) -> Result<SaveSummary, IpcError> {
    let (id, save) = active_save(&app)?;
    Ok(ipc::save_summary(&id, &save))
}

#[tauri::command]
fn completion(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<MarksMatrix, IpcError> {
    let (_, save) = active_save(&app)?;
    let counters = save.u32s(Kind::Counters).unwrap_or_default();
    // Game not installed is expected: the matrix goes out without art, and the screen draws
    // the fallback outfit.
    let catalog = resources.get().and_then(|rs| state.get_or_build(rs));
    Ok(ipc::marks_matrix(&counters, catalog, icon_url))
}

/// How many icons to extract for the verification screen: a sample, not the whole catalog.
const SAMPLE_ICONS: usize = 60;

/// What we managed to extract from the game's archives. The catalog (names, sprites)
/// is the real one, built once and kept in `CatalogState`.
#[tauri::command]
fn extraction_report(
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<ipc::ExtractionReport, IpcError> {
    let d = discover(&Options::default());
    // The wiki dataset's status is independent of the game's archives: it's computed
    // regardless, even when the game isn't installed.
    let game_updated_unix = d.game.as_ref().and_then(|g| g.updated_unix);
    let wiki = ipc::wiki_info(wiki::Dataset::embedded(), game_updated_unix);
    // Game not installed is an expected case, not an error: the command still answers
    // and the report says there's nothing to extract.
    let Some(resources) = resources.get() else {
        return Ok(ipc::extraction_report(Vec::new(), None, Vec::new(), wiki));
    };
    let catalog = state.get_or_build(resources);

    let view = catalog.map(ipc::catalog_view);
    // The icons are extracted here, where I/O is allowed, and go out already resolved.
    let sprites = catalog
        .map(|c| {
            ipc::item_views(c, |p| resources.read(p), SAMPLE_ICONS)
                .into_iter()
                .filter_map(|i| i.data_url.map(|u| (i.id, i.name, u)))
                .collect()
        })
        .unwrap_or_default();

    Ok(ipc::extraction_report(
        ipc::archive_views(resources.archives()),
        view,
        sprites,
        wiki,
    ))
}

/// The wiki page for a target, if the dataset knows it. The embedded dataset failing to
/// load is an expected case, diagnosed elsewhere (`ExtractionReport.wiki`): here it's
/// enough to say the command can't answer.
#[tauri::command]
fn wiki_entry(target: ipc::Target) -> Result<Option<ipc::Entry>, IpcError> {
    let ds = wiki::Dataset::embedded().map_err(|_| IpcError::WikiUnavailable)?;
    Ok(ds.entry(&target).cloned())
}

/// Every page the dataset has, once per window: the category lists, the tab labels and
/// the icon of every reference on a page read from it (spec 3.5, Decision 2). A dataset
/// that didn't load is an empty index that says so, not an `Err`: the landing shows it.
#[tauri::command]
fn wiki_index(
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<ipc::WikiIndex, IpcError> {
    let d = discover(&Options::default());
    let game_updated_unix = d.game.as_ref().and_then(|g| g.updated_unix);
    // No game is expected: the index goes out with no icon links, and the screen says so.
    let catalog = resources.get().and_then(|rs| state.get_or_build(rs));
    Ok(ipc::wiki_index(
        wiki::Dataset::embedded(),
        catalog,
        game_updated_unix,
        icon_url,
    ))
}

/// The wiki side of the search index, built once: the dataset is compiled into the binary and
/// never changes, so the flattening is paid for on the first query and never again. The
/// catalog side is *not* cached here — it is read per query, like everywhere else, because
/// "the game isn't installed" is never a cached answer.
#[derive(Default)]
struct SearchState(OnceLock<ipc::SearchIndex>);

impl SearchState {
    fn get(&self) -> &ipc::SearchIndex {
        self.0
            .get_or_init(|| ipc::SearchIndex::build(wiki::Dataset::embedded()))
    }
}

/// One query over the wiki's text and the catalog's names. No profile is a **diagnostic**, not
/// an error: search answers before a save is chosen, and says the marks are unknown.
#[tauri::command]
fn search(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    index: tauri::State<'_, SearchState>,
    query: String,
    limit: usize,
) -> Result<ipc::SearchView, IpcError> {
    // Game not installed is expected: the answer goes out with wiki titles alone.
    let catalog = resources.get().and_then(|rs| state.get_or_build(rs));
    let sections = active_save(&app)
        .ok()
        .map(|(_, s)| (s.flags(Kind::Achievements), s.flags(Kind::Items)));
    let flags = sections.as_ref().map(|(a, i)| ipc::SaveFlags {
        achievements: a.as_deref(),
        items: i.as_deref(),
    });
    Ok(ipc::search(
        index.get(),
        catalog,
        flags,
        &query,
        limit,
        icon_url,
    ))
}

/// Describes an `OpenError` without letting its `Debug` cross the IPC boundary: that
/// `Debug` is defined by `core-save`, not by us, and there's no guarantee its variants
/// will stay free of raw data in the future. The text here never contains a path:
/// `Io` wraps a system `std::io::Error` (permissions, missing file), not the path
/// that caused it.
fn describe_open_error(e: &OpenError) -> String {
    match e {
        OpenError::TooShort => "file too short to hold a save".to_string(),
        OpenError::BadMagic { .. } => "file signature not recognized".to_string(),
        OpenError::Io(io) => format!("error reading the file: {io}"),
    }
}

/// Opens the active profile's save. `NoActiveProfile` when there isn't one:
/// for the UI that means "go to selection", not an error to display.
fn active_save(app: &AppHandle) -> Result<(ProfileId, Save), IpcError> {
    let settings = settings_file::load(app);
    let d = discover(&Options::default());
    let views = ipc::candidates(&d.saves);
    let state = ipc::resolve_active(
        settings.active_profile_id.as_ref(),
        &views,
        d.steam.is_some(),
        d.game.is_some(),
    );
    let ActiveProfile::Active { profile, .. } = state else {
        return Err(IpcError::NoActiveProfile);
    };
    let candidate = d
        .saves
        .iter()
        .find(|s| ipc::profile_id(&s.path) == profile.id)
        .ok_or(IpcError::NoActiveProfile)?;
    let save = Save::open(&candidate.path).map_err(|e| IpcError::UnreadableSave {
        reason: describe_open_error(&e),
    })?;
    Ok((profile.id, save))
}

/// The app's database, opened once on first use. If it doesn't open (permissions,
/// corrupt file, newer schema), what's left is the reason, already in our own wording:
/// the commands return it instead of retrying on every call. It's a `String` and not an
/// `IpcError` because the only variant that would make sense here is `StoreUnavailable`:
/// the type pins that down, and `plan` puts it straight into the plan without a `match`
/// that would have to discard impossible variants.
#[derive(Default)]
struct StoreState(OnceLock<Result<Mutex<Store>, String>>);

impl StoreState {
    fn get_or_open(&self, app: &AppHandle) -> Result<&Mutex<Store>, String> {
        self.0
            .get_or_init(|| {
                let dir = app
                    .path()
                    .app_data_dir()
                    .map_err(|_| "app data folder unknown".to_string())?;
                std::fs::create_dir_all(&dir)
                    .map_err(|_| "app data folder cannot be created".to_string())?;
                Store::open(&dir.join("isaacdome.db"))
                    .map(Mutex::new)
                    .map_err(store_reason)
            })
            .as_ref()
            .map_err(Clone::clone)
    }

    /// The store, open and locked, or the reason there isn't one. Fails only if it
    /// couldn't be opened: a poisoned mutex is recovered from, because `Store` has no
    /// invariants that a panic partway through could break.
    fn lock(&self, app: &AppHandle) -> Result<MutexGuard<'_, Store>, String> {
        Ok(self
            .get_or_open(app)?
            .lock()
            .unwrap_or_else(|e| e.into_inner()))
    }
}

/// The reason behind a `store` error, in our own wording: `Unreadable` carries SQLite's
/// message, which can contain the file path, so it never crosses the IPC boundary.
fn store_reason(e: StoreError) -> String {
    match e {
        StoreError::Unreadable { .. } => "database unreadable".to_string(),
        StoreError::NewerSchema { found, supported } => {
            format!("database from a newer version ({found} > {supported})")
        }
    }
}

fn store_error(e: StoreError) -> IpcError {
    store_unavailable(store_reason(e))
}

fn store_unavailable(reason: String) -> IpcError {
    IpcError::StoreUnavailable { reason }
}

/// Section 1 of the active profile, or `None` if it can't be read. It's never flattened
/// into an empty vector: "section missing" and "zero achievements" are two different
/// things, and the view has to be able to say which one applies.
fn achievement_flags(app: &AppHandle) -> Result<Option<Vec<bool>>, IpcError> {
    let (_, save) = active_save(app)?;
    Ok(save.flags(Kind::Achievements))
}

/// Section 1 and section 2 together, from one open of the save: the graph needs both —
/// the flags for what is done, the counters for the marks and tallies it now asks about.
/// Either may be `None`, and the profile turns that into "I can't say" rather than a zero.
#[allow(clippy::type_complexity)]
fn progress_sections(app: &AppHandle) -> Result<(Option<Vec<bool>>, Option<Vec<u32>>), IpcError> {
    let (_, save) = active_save(app)?;
    Ok((save.flags(Kind::Achievements), save.u32s(Kind::Counters)))
}

#[tauri::command]
fn unlock(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::UnlockView, IpcError> {
    let (flags, counters) = progress_sections(&app)?;
    // Game not installed is expected: the view goes out without a catalog and says so.
    let resources = resources.get();
    let catalog = resources.and_then(|rs| state.get_or_build(rs));
    let g = catalog.and_then(|c| graph.get(c));
    let progress = ipc::SaveProgress::new(flags.as_deref(), counters.as_deref(), catalog);
    let eval = g.map(|g| g.evaluate(&progress));
    Ok(ipc::unlock_view(
        catalog,
        wiki::Dataset::embedded().ok(),
        flags.as_deref(),
        g,
        eval.as_ref(),
        Some(&progress),
        icon_url,
    ))
}

#[tauri::command]
fn next_steps(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::NextSteps, IpcError> {
    // The steps are a filter over the full view: same state, no extra work.
    let view = unlock(app, state, resources, graph)?;
    Ok(ipc::next_steps(&view))
}

#[tauri::command]
fn collection(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<ipc::CollectionView, IpcError> {
    let (_, save) = active_save(&app)?;
    // Game not installed is expected: the view goes out without a catalog and says so.
    let resources = resources.get();
    let catalog = resources.and_then(|rs| state.get_or_build(rs));
    let items = save.flags(Kind::Items);
    let achievements = save.flags(Kind::Achievements);
    Ok(ipc::collection_view(
        catalog,
        wiki::Dataset::embedded().ok(),
        items.as_deref(),
        achievements.as_deref(),
        icon_url,
    ))
}

/// What the plan receives from `store`, however things went: the goals that were read,
/// the ones that couldn't be, and the reason there are none. A database that won't open
/// and a query that fails are the same case for the user — "the goals can't be seen, and
/// here's why" — and neither one is an `Err`: the Plan degrades, it doesn't disappear.
/// Kept pure because in the Tauri crate everything else is wiring, and this mapping is
/// worth verifying.
fn plan_parts(
    read: Result<GoalsRead, StoreError>,
) -> (Vec<ipc::Goal>, Vec<ipc::GoalId>, Option<String>) {
    match read {
        Ok(r) => (r.goals, r.unreadable, None),
        Err(e) => (Vec::new(), Vec::new(), Some(store_reason(e))),
    }
}

/// What every queue command needs, gathered once so the five read the same way.
struct QueuePieces<'a> {
    catalog: Option<&'a Catalog>,
    graph: Option<&'a graph::Graph>,
    flags: Option<Vec<bool>>,
}

fn queue_pieces<'a>(
    app: &AppHandle,
    catalog: &'a CatalogState,
    resources: &'a ResourcesState,
    graph: &'a GraphState,
) -> Result<QueuePieces<'a>, IpcError> {
    let rs = resources.get();
    let c = rs.and_then(|rs| catalog.get_or_build(rs));
    Ok(QueuePieces {
        catalog: c,
        graph: c.and_then(|c| graph.get(c)),
        flags: achievement_flags(app)?,
    })
}

/// Reads the queue and turns it into the view. Never writes: the goals import is its own
/// command, precisely so that a read stays a read.
fn queue_view_now(
    app: &AppHandle,
    store: &StoreState,
    pieces: &QueuePieces<'_>,
) -> Result<ipc::QueueView, IpcError> {
    let (queue, goals_pending, reason) = match store.lock(app) {
        Ok(guard) => {
            let read = guard.queue();
            let queued: std::collections::BTreeSet<u32> = match &read {
                Ok(Ok(q)) => q.rows().iter().map(|r| r.achievement).collect(),
                _ => std::collections::BTreeSet::new(),
            };
            // A goal counts as pending while nothing in the queue stands for it. Without a
            // catalog we can't tell, and claiming zero would be a guess: none are reported.
            let pending = match (guard.goals(), pieces.catalog) {
                (Ok(goals), Some(c)) => goals
                    .goals
                    .iter()
                    .filter(|g| {
                        ipc::achievement_unlocking(c, &g.target)
                            .is_none_or(|a| !queued.contains(&a))
                    })
                    .count() as u32,
                _ => 0,
            };
            match read {
                Ok(inner) => (inner, pending, None),
                Err(e) => (Ok(plan::Queue::default()), pending, Some(store_reason(e))),
            }
        }
        Err(reason) => (Ok(plan::Queue::default()), 0, Some(reason)),
    };
    Ok(ipc::queue_view(
        ipc::QueueInputs {
            catalog: pieces.catalog,
            dataset: wiki::Dataset::embedded().ok(),
            flags: pieces.flags.as_deref(),
            graph: pieces.graph,
            eval: None,
            progress: None,
            queue: queue.as_ref(),
            goals_pending,
            store_reason: reason,
        },
        icon_url,
    ))
}

/// Reads the queue, hands it to the edit, writes it back. The only function here that
/// writes, so "a read never writes" has exactly one place to check.
fn queue_mutate(
    app: &AppHandle,
    store: &StoreState,
    pieces: &QueuePieces<'_>,
    edit: impl FnOnce(&mut plan::Queue, &graph::Graph, Option<&[bool]>),
) -> Result<(), IpcError> {
    let Some(g) = pieces.graph else {
        // No catalog, no graph, no way to keep the order honest: the queue is left exactly
        // as it is rather than reordered against nothing.
        return Err(IpcError::CatalogUnavailable);
    };
    let guard = store
        .lock(app)
        .map_err(|reason| IpcError::StoreUnavailable { reason })?;
    // A document that won't parse must not be silently replaced by an edited empty one:
    // editing would destroy a plan written by a version that knew more than this one.
    let mut q = match guard.queue() {
        Ok(Ok(q)) => q,
        Ok(Err(_)) => {
            return Err(IpcError::StoreUnavailable {
                reason: "coda del piano illeggibile".to_string(),
            })
        }
        Err(e) => {
            return Err(IpcError::StoreUnavailable {
                reason: store_reason(e),
            })
        }
    };
    edit(&mut q, g, pieces.flags.as_deref());
    guard.set_queue(&q).map_err(|e| IpcError::StoreUnavailable {
        reason: store_reason(e),
    })
}

/// The ids a move has to reason about: what is already queued, plus what is about to be.
fn ids_for(q: &plan::Queue, achievement: u32, chain: &[u32]) -> Vec<u32> {
    let mut ids: Vec<u32> = q.rows().iter().map(|r| r.achievement).collect();
    ids.push(achievement);
    ids.extend(chain.iter().copied());
    ids
}

#[tauri::command]
fn queue(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::QueueView, IpcError> {
    let pieces = queue_pieces(&app, &catalog, &resources, &graph)?;
    queue_view_now(&app, &store, &pieces)
}

#[tauri::command]
fn queue_add(
    app: AppHandle,
    achievement: u32,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::QueueView, IpcError> {
    let pieces = queue_pieces(&app, &catalog, &resources, &graph)?;
    queue_mutate(&app, &store, &pieces, |q, g, flags| {
        let chain = g.missing_chain(achievement, &graph::FlagsOnly(flags));
        let deps = GraphDeps::new(g, flags, &ids_for(q, achievement, &chain));
        q.enqueue(achievement, &chain, &deps);
    })?;
    queue_view_now(&app, &store, &pieces)
}

#[tauri::command]
fn queue_remove(
    app: AppHandle,
    achievement: u32,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::QueueView, IpcError> {
    let pieces = queue_pieces(&app, &catalog, &resources, &graph)?;
    queue_mutate(&app, &store, &pieces, |q, _g, _flags| q.remove(achievement))?;
    queue_view_now(&app, &store, &pieces)
}

#[tauri::command]
fn queue_move(
    app: AppHandle,
    achievement: u32,
    after: Option<u32>,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::QueueView, IpcError> {
    let pieces = queue_pieces(&app, &catalog, &resources, &graph)?;
    queue_mutate(&app, &store, &pieces, |q, g, flags| {
        let ids: Vec<u32> = q.rows().iter().map(|r| r.achievement).collect();
        let deps = GraphDeps::new(g, flags, &ids);
        // The row the drop landed under, not an index: the view the screen drew leaves
        // completed and unresolved rows out, so its positions are not the document's.
        q.move_after(achievement, after, &deps);
    })?;
    queue_view_now(&app, &store, &pieces)
}

/// The one-off move from the old goals table. A goal is a target; the queue holds
/// achievements, so each target is resolved to the achievement that unlocks it. A target
/// nothing unlocks is skipped rather than guessed at, and the goals table is left
/// untouched — so the step is repeatable and reversible.
#[tauri::command]
fn queue_import_goals(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::QueueView, IpcError> {
    let pieces = queue_pieces(&app, &catalog, &resources, &graph)?;
    let Some(c) = pieces.catalog else {
        return Err(IpcError::CatalogUnavailable);
    };
    let targets: Vec<ipc::TargetKey> = match store.lock(&app) {
        Ok(guard) => match guard.goals() {
            Ok(read) => read.goals.into_iter().map(|g| g.target).collect(),
            Err(_) => {
                return Err(IpcError::StoreUnavailable {
                    reason: "database illeggibile".to_string(),
                })
            }
        },
        Err(_) => {
            return Err(IpcError::StoreUnavailable {
                reason: "database illeggibile".to_string(),
            })
        }
    };
    queue_mutate(&app, &store, &pieces, |q, g, flags| {
        for target in &targets {
            let Some(achievement) = ipc::achievement_unlocking(c, target) else {
                continue;
            };
            let chain = g.missing_chain(achievement, &graph::FlagsOnly(flags));
            let deps = GraphDeps::new(g, flags, &ids_for(q, achievement, &chain));
            q.enqueue(achievement, &chain, &deps);
        }
    })?;
    queue_view_now(&app, &store, &pieces)
}

#[tauri::command]
fn plan(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<ipc::PlanView, IpcError> {
    // The catalog is needed to resolve saved keys into names and icons: without it,
    // the goals are still visible and the view says why they have no name.
    let resources = resources.get();
    let c = resources.and_then(|rs| catalog.get_or_build(rs));
    // Database that won't open, or a query that fails: expected cases, the plan comes
    // out empty and says why.
    let (goals, unreadable, unavailable) = match store.lock(&app) {
        Ok(guard) => plan_parts(guard.goals()),
        Err(reason) => (Vec::new(), Vec::new(), Some(reason)),
    };
    Ok(ipc::plan_view(c, goals, unreadable, unavailable, icon_url))
}

#[tauri::command]
fn add_goal(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    target: ipc::TargetKey,
) -> Result<ipc::PlanView, IpcError> {
    // The target must exist in the catalog: a goal for a made-up id doesn't get saved.
    // Without a catalog it can't be verified, and the UI needs to be able to say
    // "install the game" instead of "this item doesn't exist".
    let resources = resources.get();
    let c = resources.and_then(|rs| catalog.get_or_build(rs));
    match c {
        None => return Err(IpcError::CatalogUnavailable),
        Some(c) if !ipc::target_exists(c, &target) => return Err(IpcError::UnknownTarget),
        Some(_) => {}
    }
    let created_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let goal = ipc::Goal {
        id: ipc::GoalId::new(),
        target,
        created_unix,
        note: None,
    };
    let guard = store.lock(&app).map_err(store_unavailable)?;
    guard.add_goal(&goal).map_err(store_error)?;
    let read = guard.goals().map_err(store_error)?;
    Ok(ipc::plan_view(
        c,
        read.goals,
        read.unreadable,
        None,
        icon_url,
    ))
}

#[tauri::command]
fn remove_goal(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    id: ipc::GoalId,
) -> Result<ipc::PlanView, IpcError> {
    // Same as `plan`: the view returned carries the remaining goals, and naming them
    // needs the catalog.
    let resources = resources.get();
    let c = resources.and_then(|rs| catalog.get_or_build(rs));
    let guard = store.lock(&app).map_err(store_unavailable)?;
    // Idempotent: removing an id that's already gone isn't an error.
    let _removed = guard.remove_goal(&id).map_err(store_error)?;
    let read = guard.goals().map_err(store_error)?;
    Ok(ipc::plan_view(
        c,
        read.goals,
        read.unreadable,
        None,
        icon_url,
    ))
}

/// The URL the webview can actually fetch for an icon.
///
/// Rows carry one of these instead of a base64 image. On the real profile that took the
/// `unlock` payload from about 7 MB to under half of one, and it hands the lazy loading,
/// the caching and the de-duplication to the browser instead of to code we would have to
/// write and test in the UI.
///
/// **The platform difference lives here and nowhere else.** On Windows the webview never
/// sees a custom scheme: Tauri rewrites it to an `http://<scheme>.localhost/` origin.
/// Getting this wrong raises nothing at all — the picture simply never appears — which is
/// exactly the failure mode `unpack` already taught us to distrust.
fn icon_url(r: &ipc::IconRef) -> Option<String> {
    let path = r.to_path();
    Some(if cfg!(windows) {
        format!("http://{}.localhost/{path}", ipc::ICON_SCHEME)
    } else {
        format!("{}://localhost/{path}", ipc::ICON_SCHEME)
    })
}

/// A response with no body. Built without `?` or `unwrap`: this runs on data a webview
/// handed us, and a malformed request must degrade to "no image", never kill the process.
fn no_icon(status: u16) -> tauri::http::Response<Vec<u8>> {
    let mut r = tauri::http::Response::new(Vec::new());
    if let Ok(s) = tauri::http::StatusCode::from_u16(status) {
        *r.status_mut() = s;
    }
    r
}

/// Serves one icon: parse the reference, find which piece of which file it is, read it.
///
/// Every step answers 404 rather than guessing. Achievements and items are whole files;
/// the completion marks and the co-op menu heads are cells of a sheet, so their sprite
/// carries a `rect` and the answer is the crop.
fn icon_bytes(app: &AppHandle, path: &str) -> tauri::http::Response<Vec<u8>> {
    let Some(reference) = ipc::IconRef::parse(path) else {
        return no_icon(400);
    };
    let resources = app.state::<ResourcesState>();
    let Some(rs) = resources.get() else {
        // The game isn't installed: expected, not an error worth logging.
        return no_icon(404);
    };
    let sprite = match reference {
        ipc::IconRef::Mark { column, tier } => app
            .state::<MarkFramesState>()
            .get(rs)
            .and_then(|frames| ipc::mark_source(column, tier, frames)),
        ipc::IconRef::Achievement { .. }
        | ipc::IconRef::Item { .. }
        | ipc::IconRef::Head { .. }
        | ipc::IconRef::Page { .. } => app
            .state::<CatalogState>()
            .get_or_build(rs)
            .and_then(|c| ipc::icon_source(c, &reference).cloned()),
    };
    let Some(sprite) = sprite else {
        return no_icon(404);
    };
    let Some(png) = sprite_bytes(rs, &sprite) else {
        return no_icon(404);
    };
    let mut r = tauri::http::Response::new(png);
    r.headers_mut().insert(
        tauri::http::header::CONTENT_TYPE,
        tauri::http::HeaderValue::from_static("image/png"),
    );
    r
}

/// The file a sprite names, cropped when it names a piece of a sheet.
fn sprite_bytes(rs: &ResourceSet, sprite: &catalog::SpriteRef) -> Option<Vec<u8>> {
    let file = rs.read(&sprite.path)?;
    match sprite.rect {
        None => Some(file),
        Some(r) => ipc::crop_png(&file, r.x, r.y, r.w, r.h),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(CatalogState::default())
        .manage(GraphState::default())
        .manage(StoreState::default())
        .manage(ResourcesState::default())
        .manage(MarkFramesState::default())
        .manage(SearchState::default())
        // Icons don't travel inside the payloads any more: rows carry a link, and this
        // serves it. Asynchronous on purpose — a grid asks for a hundred at once, and each
        // one reads from an archive; on the main thread they would queue up behind the
        // commands.
        // **When a CSP is introduced** (it has to be, before the public release) it must
        // allow `img-src` from this scheme — `isaac:` and, on Windows,
        // `http://isaac.localhost`. Today `tauri.conf.json` says `"csp": null`, so nothing
        // blocks it; the day it doesn't, every icon in the app disappears with no error in
        // the console and no failing test.
        .register_asynchronous_uri_scheme_protocol(ipc::ICON_SCHEME, |ctx, request, responder| {
            let app = ctx.app_handle().clone();
            // Only the path matters, and it has to leave the request before the thread
            // takes it: `isaac://localhost/achievement/19` and the Windows rewrite
            // `http://isaac.localhost/achievement/19` both give the same path.
            let path = request.uri().path().trim_start_matches('/').to_string();
            std::thread::spawn(move || {
                responder.respond(icon_bytes(&app, &path));
            });
        })
        .invoke_handler(tauri::generate_handler![
            setup_state,
            select_profile,
            settings,
            set_scale,
            save_summary,
            completion,
            extraction_report,
            wiki_entry,
            wiki_index,
            search,
            unlock,
            next_steps,
            collection,
            queue,
            queue_add,
            queue_remove,
            queue_move,
            queue_import_goals,
            plan,
            add_goal,
            remove_goal
        ])
        .run(tauri::generate_context!())
        .expect("failed to start the application");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A recognizable input: if the outgoing `reason` contains it, the boundary leaks. The
    /// tests assert on the whole path **and** on the word inside it, because a `reason` that
    /// re-rendered or escaped the path would slip past the first check on its own.
    const SECRET_PATH: &str = r"C:\secret\isaacdome.db";

    fn reason_of(e: IpcError) -> String {
        match e {
            IpcError::StoreUnavailable { reason } => reason,
            other => panic!("expected StoreUnavailable, got {other:?}"),
        }
    }

    #[test]
    fn unreadable_does_not_leak_the_sqlite_message() {
        let reason = reason_of(store_error(StoreError::Unreadable {
            reason: SECRET_PATH.to_string(),
        }));
        assert!(!reason.contains(SECRET_PATH), "{reason}");
        assert!(!reason.contains("secret"), "{reason}");
        assert!(!reason.is_empty());
    }

    #[test]
    fn newer_schema_names_both_versions_and_nothing_else() {
        let reason = reason_of(store_error(StoreError::NewerSchema {
            found: 7,
            supported: 1,
        }));
        assert!(reason.contains('7') && reason.contains('1'), "{reason}");
        assert!(!reason.contains('\\'), "{reason}");
    }

    fn goal(id: &str) -> ipc::Goal {
        ipc::Goal {
            id: ipc::GoalId::from_str_unchecked(id),
            target: ipc::TargetKey::Boss { id: 1 },
            created_unix: 0,
            note: None,
        }
    }

    #[test]
    fn a_successful_read_passes_goals_and_unreadable_rows_through() {
        let read = Ok(GoalsRead {
            goals: vec![goal("a")],
            unreadable: vec![ipc::GoalId::from_str_unchecked("b")],
        });
        let (goals, unreadable, unavailable) = plan_parts(read);
        assert_eq!(goals, vec![goal("a")]);
        assert_eq!(unreadable, vec![ipc::GoalId::from_str_unchecked("b")]);
        assert_eq!(unavailable, None);
    }

    /// A failed query is not a plan that disappears: the Plan degrades and says why,
    /// just like when the database doesn't open at all.
    #[test]
    fn a_failed_query_degrades_into_the_reason_not_into_an_error() {
        let (goals, unreadable, unavailable) = plan_parts(Err(StoreError::NewerSchema {
            found: 7,
            supported: 1,
        }));
        assert!(goals.is_empty());
        assert!(unreadable.is_empty());
        let reason = unavailable.expect("the reason reaches the view");
        assert!(reason.contains('7') && reason.contains('1'), "{reason}");
    }

    /// And SQLite's message doesn't get through: `plan_parts` uses the same
    /// `store_reason` as the write commands.
    #[test]
    fn a_failed_query_does_not_leak_the_sqlite_message() {
        let (_, _, unavailable) = plan_parts(Err(StoreError::Unreadable {
            reason: SECRET_PATH.to_string(),
        }));
        let reason = unavailable.expect("the reason reaches the view");
        assert!(!reason.contains("secret"), "{reason}");
    }
}
