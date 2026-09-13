//! app — Tauri binary crate. Registers the IPC commands and does the one
//! I/O job that belongs to it (the settings file). The real logic lives in `ipc`.

mod commands;
mod events;
mod icons;
mod settings_file;
mod state;
mod window;
use crate::commands::{completion, graph, plan, profile, queue, session, wiki};
use crate::icons::icon_bytes;
use crate::state::{
    CatalogState, GraphState, MarkFramesState, ResourcesState, SearchState, StoreState,
};

pub use ipc::IpcError;

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
            graph::unlock,
            graph::next_steps,
            graph::collection,
            graph::want,
            queue::queue,
            queue::queue_add,
            queue::queue_remove,
            queue::queue_move,
            queue::queue_import_goals,
            plan::plan,
            plan::add_goal,
            plan::remove_goal
        ])
        // The first window is built here, not by the config: one recipe, and the same call
        // the tray and a second launch make.
        .setup(|app| {
            window::open_or_focus(app.handle());
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
            }
            // `RunEvent` is `#[non_exhaustive]` and is not ours: this is the one catch-all the
            // repo's exhaustiveness rule cannot ask us to remove.
            _ => (),
        });
}
