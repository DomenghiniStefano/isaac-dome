//! The expensive things, opened once and kept: the catalog, the graph, the archives, the
//! mark frames, the search index, the database. Plus the active profile's save, which is
//! the one read that is deliberately **not** kept (N8).
//!
//! All wiring. Nothing here has a return value worth checking that is not already checked
//! in the pure crate it comes from.

use std::sync::{Mutex, MutexGuard, OnceLock};

use catalog::Catalog;
use core_save::{Kind, Save};
use discovery::{discover, Options};
use ipc::{ActiveProfile, IpcError, ProfileId};
use store::Store;
use tauri::{AppHandle, Manager};
use unpack::ResourceSet;

use crate::settings_file;

/// The catalog of the installed game, built on first use and then kept: reading it costs
/// milliseconds, but building it (`Catalog::build` reads every source) doesn't. It
/// doesn't open the `ResourceSet` itself: it receives it already open from the caller,
/// which opens it just once.
#[derive(Default)]
pub(crate) struct CatalogState(OnceLock<Catalog>);

impl CatalogState {
    pub(crate) fn get_or_build(&self, rs: &ResourceSet) -> Option<&Catalog> {
        Some(self.0.get_or_init(|| Catalog::build(|p| rs.read(p))))
    }
}

/// The unlock graph, built once from the catalog and the rules compiled into the binary.
/// Same shape as `CatalogState`: expensive to build, cheap to consult. Rules that don't
/// parse can only be our own broken file, and they degrade like everything else — the
/// commands answer without graph info rather than failing.
#[derive(Default)]
pub(crate) struct GraphState(OnceLock<graph::Graph>);

impl GraphState {
    pub(crate) fn get(&self, catalog: &catalog::Catalog) -> Option<&graph::Graph> {
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
pub(crate) struct ResourcesState(OnceLock<ResourceSet>);

impl ResourcesState {
    pub(crate) fn get(&self) -> Option<&ResourceSet> {
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
pub(crate) struct MarkFramesState(OnceLock<ipc::MarkFrames>);

impl MarkFramesState {
    pub(crate) fn get(&self, rs: &ResourceSet) -> Option<&ipc::MarkFrames> {
        if let Some(f) = self.0.get() {
            return Some(f);
        }
        let widget = catalog::anm2_frames(&rs.read(ipc::WIDGET_ANM2)?)?;
        let lobby = catalog::anm2_frames(&rs.read(ipc::LOBBY_ANM2)?)?;
        Some(self.0.get_or_init(|| ipc::MarkFrames { widget, lobby }))
    }
}
/// The wiki side of the search index, built once: the dataset is compiled into the binary and
/// never changes, so the flattening is paid for on the first query and never again. The
/// catalog side is *not* cached here — it is read per query, like everywhere else, because
/// "the game isn't installed" is never a cached answer.
#[derive(Default)]
pub(crate) struct SearchState(OnceLock<ipc::SearchIndex>);

impl SearchState {
    pub(crate) fn get(&self) -> &ipc::SearchIndex {
        self.0
            .get_or_init(|| ipc::SearchIndex::build(wiki::Dataset::embedded()))
    }
}
/// Opens the active profile's save. `NoActiveProfile` when there isn't one:
/// for the UI that means "go to selection", not an error to display.
pub(crate) fn active_save(app: &AppHandle) -> Result<(ProfileId, Save), IpcError> {
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
        reason: (&e).into(),
    })?;
    Ok((profile.id, save))
}
/// The app's database, opened once on first use. If it doesn't open (permissions, corrupt
/// file, newer schema), what's left is the reason as a variant: the commands return it
/// instead of retrying on every call. It's a `StoreReason` and not an `IpcError` because
/// the only variant that would make sense here is `StoreUnavailable` — the type pins that
/// down, and `plan` puts it straight into the plan without a `match` that would have to
/// discard impossible variants.
#[derive(Default)]
pub(crate) struct StoreState(OnceLock<Result<Mutex<Store>, ipc::StoreReason>>);

impl StoreState {
    pub(crate) fn get_or_open(&self, app: &AppHandle) -> Result<&Mutex<Store>, ipc::StoreReason> {
        self.0
            .get_or_init(|| {
                let dir = app
                    .path()
                    .app_data_dir()
                    .map_err(|_| ipc::StoreReason::DataDirUnknown)?;
                std::fs::create_dir_all(&dir).map_err(|_| ipc::StoreReason::DataDirNotCreatable)?;
                Store::open(&dir.join("isaacdome.db"))
                    .map(Mutex::new)
                    .map_err(|e| (&e).into())
            })
            .as_ref()
            .map_err(|r| *r)
    }

    /// The store, open and locked, or the reason there isn't one. Fails only if it
    /// couldn't be opened: a poisoned mutex is recovered from, because `Store` has no
    /// invariants that a panic partway through could break.
    pub(crate) fn lock(&self, app: &AppHandle) -> Result<MutexGuard<'_, Store>, ipc::StoreReason> {
        Ok(self
            .get_or_open(app)?
            .lock()
            .unwrap_or_else(|e| e.into_inner()))
    }
}

/// Section 1 of the active profile, or `None` if it can't be read. It's never flattened
/// into an empty vector: "section missing" and "zero achievements" are two different
/// things, and the view has to be able to say which one applies.
pub(crate) fn achievement_flags(app: &AppHandle) -> Result<Option<Vec<bool>>, IpcError> {
    let (_, save) = active_save(app)?;
    Ok(save.flags(Kind::Achievements))
}

/// Section 1 and section 2 together, from one open of the save: the graph needs both —
/// the flags for what is done, the counters for the marks and tallies it now asks about.
/// Either may be `None`, and the profile turns that into "I can't say" rather than a zero.
#[allow(clippy::type_complexity)]
pub(crate) fn progress_sections(
    app: &AppHandle,
) -> Result<(Option<Vec<bool>>, Option<Vec<u32>>), IpcError> {
    let (_, save) = active_save(app)?;
    Ok((save.flags(Kind::Achievements), save.u32s(Kind::Counters)))
}
