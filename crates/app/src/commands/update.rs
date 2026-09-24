//! Updating the app itself: the plugin, the bytes, and the three commands.
//!
//! Wiring only, which is why it is here and not in `ipc`. Every decision about what may
//! follow what — whether a check is allowed, whether the windows are worth telling — is
//! `ipc::UpdateState`, where it can be read and tested. What is left is the network, the
//! mutex, and one table that groups the plugin's errors into the five answers a user gets.
//!
//! Spec: `docs/superpowers/specs/2026-09-20-app-update-design.md`.

use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use ipc::{IpcError, UpdateFailure, UpdateReason, UpdateState, UpdateView};

use crate::events::{announce, UPDATE_CHANGED};
use crate::settings_file;

/// Whether this build can update itself at all.
///
/// **A development build registers no plugin**, for the same reason it registers no autostart:
/// an installer run over `target\debug` replaces the build you are working on with a release,
/// and nothing says so. The autostart commands ask `try_state` whether the plugin is there;
/// that door is shut here, because `tauri_plugin_updater::UpdaterState` is private, so the
/// build kind is read directly — **and it must stay the same condition `lib.rs` registers the
/// plugin under**, or `app.updater()` panics on a state nobody managed.
pub const UPDATER_BUILD: bool = !cfg!(debug_assertions);

/// The phase, the update it belongs to, and the bytes that were verified.
///
/// The bytes are held whole in memory, which is a deviation from "no file is ever loaded whole"
/// and is written down rather than hidden: the plugin's `download` returns a `Vec<u8>` and has
/// no streaming form, and this is our own installer, not a file whose size is a stranger's
/// choice. They live from the end of the download to the moment the user says install.
#[derive(Default)]
pub struct UpdaterState {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    state: UpdateState,
    ready: Option<(Arc<Update>, Vec<u8>)>,
}

impl UpdaterState {
    /// The view as it stands. **Never takes the lock across an await**: everything here is a
    /// read or a field assignment, and a poisoned mutex answers the default rather than
    /// bringing down a command — a screen that says "nothing asked yet" is recoverable, a
    /// panic in a Tauri command is not.
    fn view(&self, current_version: &str) -> UpdateView {
        let unavailable = (!UPDATER_BUILD).then_some(UpdateReason::NotSupported);
        match self.inner.lock() {
            Ok(inner) => inner.state.view(current_version, unavailable),
            Err(_) => UpdateState::default().view(current_version, unavailable),
        }
    }

    fn with<T>(&self, job: impl FnOnce(&mut Inner) -> T) -> Option<T> {
        self.inner.lock().ok().map(|mut inner| job(&mut inner))
    }
}

fn version(app: &AppHandle) -> String {
    app.package_info().version.to_string()
}

/// Tells every window to read again, exactly as the five events before this one do: no
/// payload, and each window answers by calling `update_status`.
fn changed(app: &AppHandle) {
    announce(app, UPDATE_CHANGED);
}

/// Which of the five answers an error from the plugin is.
///
/// The grouping is the judgement, and it is written as a table so it can be argued with. The
/// plugin's `Error` is `#[non_exhaustive]` and its own variants carry URLs and paths, so it
/// never crosses the boundary — only the answer does. The catch-all arm is owed to that
/// attribute, the same exemption `lib.rs` already takes for `RunEvent`.
#[allow(clippy::wildcard_enum_match_arm)] // a foreign enum; the reason is at the wildcard arm
fn failure_of(error: &tauri_plugin_updater::Error) -> UpdateFailure {
    use tauri_plugin_updater::Error as E;
    match error {
        // Nothing answered. Bad luck, and worth trying again later.
        E::Network(_) | E::Reqwest(_) | E::Io(_) | E::UrlParse(_) | E::Http(_) => {
            UpdateFailure::Offline
        }
        // Something answered and there is nothing here to install: no manifest at that URL, a
        // manifest that is not one, or one carrying no entry for this platform. It is the
        // ordinary answer for as long as no release exists.
        E::ReleaseNotFound
        | E::TargetNotFound(_)
        | E::TargetsNotFound(_)
        | E::Serialization(_)
        | E::Semver(_)
        | E::EmptyEndpoints => UpdateFailure::NotPublished,
        // **Answered, and refused.** What was served is not what it claims to be: the
        // signature did not verify, or it was signed for a version other than the one
        // announced — which is the downgrade `requireSignedVersion` exists to stop. The one
        // failure here that is not bad luck.
        E::Minisign(_)
        | E::SignatureUtf8(_)
        | E::Base64(_)
        | E::SignedVersionMismatch { .. }
        | E::MissingSignedVersion => UpdateFailure::Rejected,
        // The bytes were good and the installer would not run.
        E::PackageInstallFailed
        | E::DebInstallFailed
        | E::InvalidUpdaterFormat
        | E::BinaryNotFoundInArchive
        | E::AuthenticationFailed
        | E::FailedToDetermineExtractPath
        | E::TempDirNotFound
        | E::TempDirNotOnSameMountPoint => UpdateFailure::InstallFailed,
        // `tauri_plugin_updater::Error` is not ours: a variant the plugin adds is a failure we have
        // no sentence for, which is what `Unknown` says.
        _ => UpdateFailure::Unknown,
    }
}

/// What the screen reads. **No network, ever** — this is the held phase and the running
/// version, so it answers the same on a machine that has never been online.
#[tauri::command]
pub fn update_status(app: AppHandle) -> Result<UpdateView, IpcError> {
    let state: tauri::State<'_, UpdaterState> = app.state();
    Ok(state.view(&version(&app)))
}

/// The button, and the startup check: looks, and downloads whatever it finds.
///
/// Answers the view as it stands when it returns — which is `Ready`, `UpToDate` or `Failed`,
/// never the download in progress, because the whole thing is awaited here. The windows follow
/// it through the event while it runs.
#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<UpdateView, IpcError> {
    check_now(&app).await;
    let state: tauri::State<'_, UpdaterState> = app.state();
    Ok(state.view(&version(&app)))
}

/// The whole check, shared by the command and the launch.
///
/// Returns nothing: both callers read the state afterwards, and a failure here is a phase and
/// not an error — an endpoint that will not answer is something the user reads, not something
/// that breaks a command.
pub async fn check_now(app: &AppHandle) {
    if !UPDATER_BUILD {
        return;
    }
    {
        let state: tauri::State<'_, UpdaterState> = app.state();
        // `false` means one is already running, or a download is: doing nothing is the answer,
        // and the state is left exactly as it was.
        if state.with(|inner| inner.state.begin_check()) != Some(true) {
            return;
        }
    }
    changed(app);

    let found = match app.updater() {
        Ok(updater) => updater.check().await,
        Err(e) => Err(e),
    };
    let update = match found {
        Ok(Some(update)) => Arc::new(update),
        Ok(None) => {
            let state: tauri::State<'_, UpdaterState> = app.state();
            state.with(|inner| inner.state.up_to_date());
            changed(app);
            return;
        }
        Err(e) => return fail(app, &e),
    };

    let announced = update.version.clone();
    {
        let state: tauri::State<'_, UpdaterState> = app.state();
        state.with(|inner| {
            inner.ready = None;
            inner.state.begin_download(announced.clone());
        });
    }
    changed(app);

    // The callback is called once per chunk and does two cheap things: counts the bytes, and
    // asks whether the whole percentage point moved. Only then is every window told — at most
    // a hundred times over a download instead of thousands.
    let downloaded = update
        .download(
            |chunk, total| {
                let state: tauri::State<'_, UpdaterState> = app.state();
                if state.with(|inner| inner.state.advance(chunk, total)) == Some(true) {
                    changed(app);
                }
            },
            || {},
        )
        .await;

    match downloaded {
        Ok(bytes) => {
            let state: tauri::State<'_, UpdaterState> = app.state();
            let notes = update.body.clone();
            state.with(|inner| {
                inner.state.ready(announced.clone(), notes.clone());
                inner.ready = Some((update.clone(), bytes));
            });
            changed(app);
        }
        Err(e) => fail(app, &e),
    }
}

fn fail(app: &AppHandle, error: &tauri_plugin_updater::Error) {
    let state: tauri::State<'_, UpdaterState> = app.state();
    state.with(|inner| {
        inner.ready = None;
        inner.state.fail(failure_of(error));
    });
    changed(app);
}

/// Installs what was downloaded. **On Windows this does not return**: the plugin launches the
/// installer and ends the process with `std::process::exit(0)`, which is also why the tray's
/// "stay in the background" cannot hold it back — that works by intercepting an exit request,
/// and this raises none.
///
/// **An installer that would not run is not an `Err`**: it is `InstallFailed`, a phase the
/// screen already has a sentence for. The only `Err` is being asked to install nothing, which
/// can only be a window acting on a phase it no longer had.
///
/// The bytes go back where they were on a failure rather than being dropped: a retry is right
/// when the cause was a file still locked, useless when it was the installer itself, and costs
/// nothing either way.
#[tauri::command]
pub fn install_update(app: AppHandle) -> Result<(), IpcError> {
    let state: tauri::State<'_, UpdaterState> = app.state();
    // Taken out of the lock rather than installed under it: the guard would otherwise be held
    // across a call that, when all goes well, never returns.
    let Some(Some((update, bytes))) = state.with(|inner| inner.ready.take()) else {
        return Err(IpcError::UpdateNotReady);
    };
    match update.install(&bytes) {
        Ok(()) => Ok(()),
        Err(e) => {
            let reason = failure_of(&e);
            state.with(|inner| {
                inner.state.fail(reason);
                inner.ready = Some((update, bytes));
            });
            changed(&app);
            Ok(())
        }
    }
}

/// The launch's own check, which is the whole of what the switch decides.
///
/// **With the switch off nothing is spawned at all** — not a check that stays quiet, no
/// request. It is read here, once, from the file, and the function says so by name.
pub fn check_at_launch(app: &AppHandle) {
    if !UPDATER_BUILD || !settings_file::load(app).auto_update {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        check_now(&app).await;
    });
}
