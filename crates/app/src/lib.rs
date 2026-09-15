//! app — Tauri binary crate. Registers the IPC commands and does the one
//! I/O job that belongs to it (the settings file). The real logic lives in `ipc`.

mod commands;
mod events;
mod icons;
mod settings_file;
mod state;
mod tray;
mod window;
use crate::commands::{completion, floor, graph, plan, profile, queue, runs, session, wiki};
use crate::icons::icon_bytes;
use crate::state::{
    AllPassive, ArchiveState, CatalogState, GraphState, MarkFramesState, ResourcesState, SaveState,
    SearchState, StoreState,
};

use tauri::Manager;

pub use ipc::IpcError;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // **First, by the plugin's own requirement.** A second launch is the same gesture as a
        // click on the icon: this app has no command line, so the arguments are nothing to act
        // on.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            window::open_or_focus(app);
        }))
        .plugin(tauri_plugin_notification::init())
        .manage(CatalogState::default())
        .manage(GraphState::default())
        .manage(StoreState::default())
        .manage(ResourcesState::default())
        .manage(MarkFramesState::default())
        .manage(SearchState::default())
        .manage(SaveState::default())
        .manage(ArchiveState::default())
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
            profile::setup_state,
            profile::select_profile,
            profile::settings,
            profile::set_scale,
            profile::set_stay_in_background,
            profile::set_resume_tabs,
            session::window_session,
            session::set_window_session,
            completion::save_summary,
            completion::completion,
            completion::extraction_report,
            wiki::wiki_entry,
            wiki::wiki_index,
            wiki::search,
            graph::graph_views,
            graph::collection,
            graph::want,
            queue::queue,
            queue::queue_add,
            queue::queue_remove,
            queue::queue_move,
            queue::queue_import_goals,
            plan::plan,
            plan::add_goal,
            plan::remove_goal,
            runs::runs,
            runs::live,
            floor::floor_candidates
        ])
        // The first window is built here, not by the config: one recipe, and the same call
        // the tray and a second launch make.
        .setup(|app| {
            // The tray before the window: if a window fails to open, the way back in still
            // exists.
            tray::build(app.handle());
            window::open_or_focus(app.handle());
            start_archive(app.handle().clone());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to start the application")
        .run(|app, event| match event {
            // `code` is `None` when the user closed the last window and `Some` when the code
            // asked to exit (`AppHandle::exit`, the tray's Quit). Preventing only the first is
            // what makes Quit work without a flag anyone has to remember to set.
            tauri::RunEvent::ExitRequested {
                code: None, api, ..
            } if settings_file::load(app).stay_in_background => {
                api.prevent_exit();
                tray::notice_once(app);
            }
            // `RunEvent` is `#[non_exhaustive]` and is not ours: this is the one catch-all the
            // repo's exhaustiveness rule cannot ask us to remove.
            _ => (),
        });
}

/// Fills the run archive and then follows the log, off the main thread.
///
/// A backfill of twenty-eight sessions must not hold the window shut, and **every failure here
/// is a quiet archive, never an app that will not start**: no game folder, no database, no
/// watch — each of them leaves the rest of the app exactly as it was.
fn start_archive(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let discovery = discovery::discover(&discovery::Options::default());
        let Some(data) = discovery.game_data else {
            return;
        };
        if let Some(online) = &data.online_logs {
            ingest_with(&app, |i| {
                i.backfill(online);
            });
        }
        if let Some(log) = &data.log {
            ingest_with(&app, |i| {
                let _ = i.live_log(log);
            });
        }
        events::announce(&app, events::RUNS_CHANGED);

        let Some(log) = data.log.clone() else {
            return;
        };
        let handle = app.clone();
        let watched = log.clone();
        if let Ok(watcher) = log_watch::watch(&log, move || {
            ingest_with(&handle, |i| {
                let _ = i.live_log(&watched);
            });
            events::announce(&handle, events::RUNS_CHANGED);
        }) {
            let archive: tauri::State<'_, ArchiveState> = app.state();
            archive.keep(watcher);
        }
    });
}

/// Runs one job with an `Ingest` built from the app's state, holding the database only for as
/// long as the job takes.
///
/// The catalog answers what an item is when the game is installed; without it everything
/// accumulates, which changes which active a run is carrying and never changes an outcome. So
/// the archive is built either way rather than waiting for a game that may not be there.
fn ingest_with(app: &tauri::AppHandle, job: impl FnOnce(&log_watch::Ingest<'_>)) {
    let archive: tauri::State<'_, ArchiveState> = app.state();
    let store_state: tauri::State<'_, StoreState> = app.state();
    let catalog_state: tauri::State<'_, CatalogState> = app.state();
    let resources: tauri::State<'_, ResourcesState> = app.state();
    let Ok(store) = store_state.lock(app) else {
        return;
    };
    let kinds = resources
        .get()
        .and_then(|rs| catalog_state.get_or_build(rs))
        .map(ipc::CatalogKinds);
    let all_passive = AllPassive;
    let ingest = log_watch::Ingest {
        store: &store,
        rules: archive.rules(),
        kinds: match &kinds {
            Some(k) => k,
            None => &all_passive,
        },
    };
    job(&ingest);
}
