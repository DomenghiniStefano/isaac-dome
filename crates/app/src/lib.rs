//! app — the Tauri binary crate: it registers the commands and the icon protocol, keeps the
//! managed state, and does the I/O only a running app can — the settings file, the archive's
//! watcher, the updater. Anything with a return value worth checking lives in a pure crate.

mod clock;
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
    catalog_now, AllPassive, ArchiveState, CatalogState, GraphState, LiveUnlockState,
    MarkFramesState, ResourcesState, SaveState, SearchState, StoreState,
};

use tauri::{Builder, Manager, Wry};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    release_plugins(icon_protocol(managed_state(plugins(Builder::default()))))
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
            floor::room_icons,
            update::update_status,
            update::check_update,
            update::install_update
        ])
        // The first window is built here, not by the config: one recipe, and the same call
        // the tray and a second launch make.
        .setup(setup)
        .build(tauri::generate_context!())
        .expect("failed to start the application")
        .run(on_run_event);
}

/// The plugins every build registers.
fn plugins(builder: Builder<Wry>) -> Builder<Wry> {
    builder
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
}

/// The expensive things the commands share, each opened on first use (`state.rs`).
fn managed_state(builder: Builder<Wry>) -> Builder<Wry> {
    builder
        .manage(CatalogState::default())
        .manage(GraphState::default())
        .manage(LiveUnlockState::default())
        .manage(StoreState::default())
        .manage(ResourcesState::default())
        .manage(MarkFramesState::default())
        .manage(SearchState::default())
        .manage(SaveState::default())
        .manage(ArchiveState::default())
        .manage(update::UpdaterState::default())
}

/// Icons don't travel inside the payloads any more: rows carry a link, and this serves it.
/// Asynchronous on purpose — a grid asks for a hundred at once, and each one reads from an
/// archive; on the main thread they would queue up behind the commands.
///
/// **The CSP in `tauri.conf.json` names this scheme in `img-src`** — `isaac:` and, on Windows,
/// `http://isaac.localhost` (card #80, R1). Renaming `ICON_SCHEME` without changing both
/// policies there (`csp` and `devCsp`) makes every icon in the app disappear with no error in
/// the console and no failing test.
fn icon_protocol(builder: Builder<Wry>) -> Builder<Wry> {
    builder.register_asynchronous_uri_scheme_protocol(
        ipc::ICON_SCHEME,
        |ctx, request, responder| {
            let app = ctx.app_handle().clone();
            // Only the path matters, and it has to leave the request before the thread
            // takes it: `isaac://localhost/achievement/19` and the Windows rewrite
            // `http://isaac.localhost/achievement/19` both give the same path.
            let path = request.uri().path().trim_start_matches('/').to_string();
            std::thread::spawn(move || {
                responder.respond(icon_bytes(&app, &path));
            });
        },
    )
}

/// The plugins a development build leaves out, each for a reason of its own.
fn release_plugins(builder: Builder<Wry>) -> Builder<Wry> {
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
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
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
}

// A function of its own so the lint's allow covers this one match and nothing else in `run`.
#[allow(clippy::wildcard_enum_match_arm)] // a foreign enum; the reason is at the wildcard arm
fn on_run_event(app: &tauri::AppHandle, event: tauri::RunEvent) {
    match event {
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
    }
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
/// is an archive that says what it is missing, never an app that will not start**. No game
/// folder, no database, a session or the live log that would not read: each leaves the rest of
/// the app exactly as it was and reaches the Runs screen as a diagnostic (card #80, R4). A
/// watch that will not start is the one that stays quiet — the archive read at launch is
/// still whole, it only stops following.
fn start_archive(app: tauri::AppHandle) {
    std::thread::spawn(move || fill_and_follow(&app));
}

fn fill_and_follow(app: &tauri::AppHandle) {
    let archive: tauri::State<'_, ArchiveState> = app.state();
    let Some(data) = crate::state::discovery_now(app).game_data else {
        // Said, not left to look like an archive with nothing in it (card #80, R4).
        archive.record(|h| h.no_log_folder = true);
        events::announce(app, events::RUNS_CHANGED);
        return;
    };
    // A release that edits the rules finds an archive folded by the ones before: fold it
    // again from the events it already holds, or every session but the current launch
    // drops out of Runs. A source that will not fold is a database that will not answer,
    // which the runs command reports on its own.
    ingest_with(app, |i| i.refold_stale());
    if let Some(online) = &data.online_logs {
        let unreadable = import_sessions(app, online);
        archive.record(|h| h.unreadable_sessions = unreadable);
    }
    // Derived even when the file is not there yet (card #80, R5): discovery names only a
    // log that exists, and a fresh install has none until the game's first launch — which
    // is exactly the launch worth watching. The watch is on the folder, so a file that
    // appears later is seen.
    let log = data.log.unwrap_or_else(|| data.dir.join("log.txt"));
    read_live_log(app, &log);
    events::announce(app, events::RUNS_CHANGED);
    follow_live_log(app, log);
}

/// Imports every session not in the archive yet, answering how many would not read.
///
/// One lock per session and not one for them all (card #80, R3): a first launch reads
/// twenty-eight folders, and the queue, the plan, the runs and the window session wait on this
/// same database meanwhile.
fn import_sessions(app: &tauri::AppHandle, online: &std::path::Path) -> u32 {
    log_watch::sessions(online)
        .iter()
        .filter(|folder| matches!(ingest_with(app, |i| i.session(folder)), Some(Err(_))))
        .count() as u32
}

/// Keeps reading the live log as the game writes it. A watch that will not start is kept
/// quiet: what was read at launch is still whole, the archive only stops following.
fn follow_live_log(app: &tauri::AppHandle, log: std::path::PathBuf) {
    let handle = app.clone();
    let watched = log.clone();
    if let Ok(watcher) = log_watch::watch(&log, move || {
        read_live_log(&handle, &watched);
        events::announce(&handle, events::RUNS_CHANGED);
    }) {
        app.state::<ArchiveState>().keep(watcher);
    }
}

/// Reads the live log into the archive and keeps whether it could (card #80, R4). A log that
/// is not there is not unreadable: it is a game that has not been launched yet.
fn read_live_log(app: &tauri::AppHandle, log: &std::path::Path) {
    let Some(read) = ingest_with(app, |i| i.live_log(log)) else {
        // No database: the runs command says so itself.
        return;
    };
    let unreadable = match read {
        Ok(_) => false,
        Err(log_watch::WatchError::Io { kind }) => kind != std::io::ErrorKind::NotFound,
        Err(log_watch::WatchError::Store(_)) => true,
    };
    let archive: tauri::State<'_, ArchiveState> = app.state();
    archive.record(|h| h.live_log_unreadable = unreadable);
}

/// Runs one job with an `Ingest` built from the app's state, holding the database only for as
/// long as the job takes. `None` when the database is not there to hold.
///
/// The catalog answers what an item is when the game is installed; without it everything
/// accumulates, which changes which active a run is carrying and never changes an outcome. So
/// the archive is built either way rather than waiting for a game that may not be there.
fn ingest_with<R>(
    app: &tauri::AppHandle,
    job: impl FnOnce(&log_watch::Ingest<'_>) -> R,
) -> Option<R> {
    let archive: tauri::State<'_, ArchiveState> = app.state();
    let store_state: tauri::State<'_, StoreState> = app.state();
    let catalog_state: tauri::State<'_, CatalogState> = app.state();
    let resources: tauri::State<'_, ResourcesState> = app.state();
    let store = store_state.lock(app).ok()?;
    let kinds = catalog_now(app, &resources, &catalog_state).map(ipc::CatalogKinds);
    let all_passive = AllPassive;
    let ingest = log_watch::Ingest {
        store: &store,
        rules: archive.rules(),
        kinds: match &kinds {
            Some(k) => k,
            None => &all_passive,
        },
    };
    Some(job(&ingest))
}
