//! app — Tauri binary crate. Registers the IPC commands and does the one
//! I/O job that belongs to it (the settings file). The real logic lives in `ipc`.

mod commands;
mod events;
mod icons;
mod settings_file;
mod state;
mod tray;
mod window;
use crate::commands::{
    completion, floor, graph, plan, profile, queue, roll, runs, session, update, wiki,
};
use crate::icons::icon_bytes;
use crate::state::{
    AllPassive, ArchiveState, CatalogState, GraphState, MarkFramesState, ResourcesState, SaveState,
    SearchState, StoreState,
};

use tauri::Manager;

pub use ipc::IpcError;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        // **First, by the plugin's own requirement.** A second launch is the same gesture as a
        // click on the icon, and its arguments are ignored **because** of what they can now
        // contain: a login launch carries `--silent`, and a person double-clicking the icon
        // while one is already running is asking for the app, not for the tray.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            window::open_or_focus(app);
        }))
        .plugin(tauri_plugin_notification::init())
        // Opened from Rust only, so no `dialog:*` capability is needed and the npm package
        // is not installed: capabilities gate `invoke` from the webview, and a command that
        // calls the plugin in Rust never crosses that boundary. It also keeps frontend rule 3
        // — the path stays inside the backend and never reaches JavaScript.
        .plugin(tauri_plugin_dialog::init())
        .manage(CatalogState::default())
        .manage(GraphState::default())
        .manage(StoreState::default())
        .manage(ResourcesState::default())
        .manage(MarkFramesState::default())
        .manage(SearchState::default())
        .manage(SaveState::default())
        .manage(ArchiveState::default())
        .manage(update::UpdaterState::default())
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
        });

    // **Not in a development build**, and the reason is concrete: `current_exe()` there is
    // `target\debug\app.exe`, so a switch flipped once while testing leaves that path in the
    // developer's own login — surviving `cargo clean`, pointing at nothing, and failing
    // silently at every boot. With no plugin the two commands answer `notSupported` and the
    // switch is disabled, which is the truth about this build.
    //
    // `Builder` and not `init()`: the entry's name is what Task Manager shows in its Startup
    // list, and the argument is what tells a login launch from a double-click.
    #[cfg(not(debug_assertions))]
    let builder = builder.plugin(
        tauri_plugin_autostart::Builder::new()
            .app_name(ipc::AUTOSTART_ENTRY)
            .arg(ipc::SILENT_ARG)
            .build(),
    );

    // **Not in a development build either**, and for a sharper reason than autostart's: an
    // installer run from here replaces the build you are working on with a release, silently.
    // The condition is mirrored by `update::UPDATER_BUILD`, which is what the commands read —
    // the plugin's own state type is private, so there is no `try_state` to ask, and the two
    // have to say the same thing or `app.updater()` panics on a state nobody managed.
    #[cfg(not(debug_assertions))]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());

    builder
        .invoke_handler(tauri::generate_handler![
            profile::setup_state,
            profile::select_profile,
            profile::choose_game_folder,
            profile::choose_saves_folder,
            profile::settings,
            profile::set_scale,
            profile::set_stay_in_background,
            profile::set_resume_tabs,
            profile::set_auto_update,
            profile::autostart,
            profile::set_autostart,
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
            graph::challenges,
            graph::want,
            queue::queue,
            queue::queue_add,
            queue::queue_remove,
            queue::queue_move,
            queue::queue_import_goals,
            roll::roll,
            roll::roll_draw,
            roll::set_roll_preset,
            plan::plan,
            plan::add_goal,
            plan::remove_goal,
            runs::runs,
            runs::live,
            floor::floor_candidates,
            update::update_status,
            update::check_update,
            update::install_update
        ])
        // The first window is built here, not by the config: one recipe, and the same call
        // the tray and a second launch make.
        .setup(|app| {
            // The tray before the window: if a window fails to open, the way back in still
            // exists. **Always**, and a silent launch is why it matters: a process with no
            // window and no icon is one nobody can reach.
            tray::build(app.handle());
            if matches!(
                ipc::launch_intent(&args_after_exe()),
                ipc::LaunchIntent::Window
            ) {
                window::open_or_focus(app.handle());
            }
            // **Always**, whichever way the app was started. It is the feature: the login entry
            // exists so the archive is following the log before the game is.
            start_archive(app.handle().clone());
            // After the archive, and only if the switch is on. With it off this spawns nothing
            // at all: the setting is about the request, not about a notice.
            update::check_at_launch(app.handle());
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

/// What this process was handed, without the executable's own argument.
///
/// `launch_intent` is given what follows `argv[0]` because that is the only part a login entry
/// controls: the plugin writes `"<path> --silent"` into the registry value.
fn args_after_exe() -> Vec<String> {
    std::env::args().skip(1).collect()
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
