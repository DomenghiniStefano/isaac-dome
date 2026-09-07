//! app — Tauri binary crate. Registers the IPC commands and does the one
//! I/O job that belongs to it (the settings file). The real logic lives in `ipc`.

mod error;
mod settings_file;

use std::sync::{Mutex, MutexGuard, OnceLock};

use catalog::Catalog;
use core_save::{Kind, OpenError, Save};
use discovery::{discover, Options};
use ipc::{ActiveProfile, MarksMatrix, ProfileId, SaveSummary, Settings, SetupState};
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

/// The game's archives, opened **only once** for the whole life of the process.
///
/// Opening a `ResourceSet` now costs just the index, not the data (`unpack` reads
/// entry by entry), but it's still wasted work if repeated: `discover`, eight
/// `File::open` calls, eight tables re-read on every single command. This is the
/// crate's one and only call to `ResourceSet::open`.
///
/// The "game not installed" case is **not** cached: if it's missing, the next command
/// tries again. Someone who opens the app before installing the game shouldn't have to
/// restart it.
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
    // Load the existing settings and update only the field that changed: `Settings`
    // has a single field today, so clippy flags the spread as "needless" — but it has
    // to be written this way, because the day `Settings` grows, switching profiles
    // must not silently wipe out the other preferences.
    #[allow(clippy::needless_update)]
    let settings = Settings {
        active_profile_id: Some(id),
        ..settings_file::load(&app)
    };
    settings_file::save(&app, &settings)?;
    Ok(ipc::setup_state(&d, settings.active_profile_id.as_ref()))
}

#[tauri::command]
fn save_summary(app: AppHandle) -> Result<SaveSummary, IpcError> {
    let (id, save) = active_save(&app)?;
    Ok(ipc::save_summary(&id, &save))
}

#[tauri::command]
fn completion(app: AppHandle) -> Result<MarksMatrix, IpcError> {
    let (_, save) = active_save(&app)?;
    let counters = save.u32s(Kind::Counters).unwrap_or_default();
    Ok(ipc::marks_matrix(&counters))
}

/// How many icons to extract for the verification screen: a sample, not the whole catalog.
const ICONE_DI_ESEMPIO: usize = 60;

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
            ipc::item_views(c, |p| resources.read(p), ICONE_DI_ESEMPIO)
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

/// Describes an `OpenError` without letting its `Debug` cross the IPC boundary: that
/// `Debug` is defined by `core-save`, not by us, and there's no guarantee its variants
/// will stay free of raw data in the future. The text here never contains a path:
/// `Io` wraps a system `std::io::Error` (permissions, missing file), not the path
/// that caused it.
fn describe_open_error(e: &OpenError) -> String {
    match e {
        OpenError::TooShort => "file troppo corto per contenere un salvataggio".to_string(),
        OpenError::BadMagic { .. } => "firma del file non riconosciuta".to_string(),
        OpenError::Io(io) => format!("errore di lettura del file: {io}"),
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
                    .map_err(|_| "cartella dati dell'app non nota".to_string())?;
                std::fs::create_dir_all(&dir)
                    .map_err(|_| "cartella dati dell'app non creabile".to_string())?;
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
        StoreError::Unreadable { .. } => "database illeggibile".to_string(),
        StoreError::NewerSchema { found, supported } => {
            format!("database di una versione più nuova ({found} > {supported})")
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

#[tauri::command]
fn unlock(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::UnlockView, IpcError> {
    let flags = achievement_flags(&app)?;
    // Game not installed is expected: the view goes out without a catalog and says so.
    let resources = resources.get();
    let catalog = resources.and_then(|rs| state.get_or_build(rs));
    let g = catalog.and_then(|c| graph.get(c));
    let eval = g.map(|g| g.evaluate(flags.as_deref()));
    Ok(ipc::unlock_view(
        catalog,
        flags.as_deref(),
        g,
        eval.as_ref(),
        |p| resources.and_then(|rs| rs.read(p)),
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
    Ok(ipc::plan_view(c, goals, unreadable, unavailable, |p| {
        resources.and_then(|rs| rs.read(p))
    }))
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
    Ok(ipc::plan_view(c, read.goals, read.unreadable, None, |p| {
        resources.and_then(|rs| rs.read(p))
    }))
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
    Ok(ipc::plan_view(c, read.goals, read.unreadable, None, |p| {
        resources.and_then(|rs| rs.read(p))
    }))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(CatalogState::default())
        .manage(GraphState::default())
        .manage(StoreState::default())
        .manage(ResourcesState::default())
        .invoke_handler(tauri::generate_handler![
            setup_state,
            select_profile,
            save_summary,
            completion,
            extraction_report,
            wiki_entry,
            unlock,
            next_steps,
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

    /// A recognizable input: if the outgoing `reason` contains it, the boundary leaks.
    const SEGRETO: &str = r"C:\segreto\isaacdome.db";

    fn reason_of(e: IpcError) -> String {
        match e {
            IpcError::StoreUnavailable { reason } => reason,
            other => panic!("expected StoreUnavailable, got {other:?}"),
        }
    }

    #[test]
    fn unreadable_does_not_leak_the_sqlite_message() {
        let reason = reason_of(store_error(StoreError::Unreadable {
            reason: SEGRETO.to_string(),
        }));
        assert!(!reason.contains(SEGRETO), "{reason}");
        assert!(!reason.contains("segreto"), "{reason}");
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
            reason: SEGRETO.to_string(),
        }));
        let reason = unavailable.expect("the reason reaches the view");
        assert!(!reason.contains("segreto"), "{reason}");
    }
}
