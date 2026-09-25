//! The expensive things, opened once and kept: the catalog, the graph, the archives, the
//! mark frames, the search index, the database. Plus the active profile's save, which since
//! N8 is kept too — and is the only one of them that has to be **given up** again, because
//! the game rewrites it while the app is open. The rule for that lives in `ipc::SaveCache`.
//!
//! All wiring. Nothing here has a return value worth checking that is not already checked
//! in the pure crate it comes from.

use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use catalog::Catalog;
use core_save::{Kind, Save};
use discovery::{discover, Discovery};
use ipc::{ActiveProfile, IpcError, ProfileId};
use store::Store;
use tauri::{AppHandle, Manager};
use unpack::ResourceSet;

use crate::settings_file;

/// The catalog of the installed game, built on first use and then kept: reading it costs
/// milliseconds, but building it (`Catalog::build` reads every source) doesn't. It
/// doesn't open the `ResourceSet` itself: it receives it already open from the caller,
/// which opens it just once.
///
/// Its boss keys are kept beside it (card #82, S3): settled once, from this catalog and the
/// dataset compiled into the binary, and handed to every lookup that names a boss.
#[derive(Default)]
pub(crate) struct CatalogState {
    catalog: OnceLock<Catalog>,
    bosses: OnceLock<ipc::BossKeys>,
}

impl CatalogState {
    pub(crate) fn get_or_build(&self, rs: &ResourceSet) -> &Catalog {
        self.catalog.get_or_init(|| Catalog::build(|p| rs.read(p)))
    }

    /// The boss keys of the catalog this state holds, or none when there is no catalog to
    /// key. `catalog` is the one [`catalog_now`] returned from this same state.
    pub(crate) fn bosses(&self, catalog: Option<&Catalog>) -> &ipc::BossKeys {
        match catalog {
            Some(c) => self
                .bosses
                .get_or_init(|| ipc::boss_keys(c, wiki::Dataset::embedded().ok())),
            None => ipc::BossKeys::NONE,
        }
    }
}

/// The catalog as it stands now: `None` when the game isn't installed, which every caller
/// treats as an expected case and never as an error. **The one way to ask for it** in this
/// crate — the chain from the archives to the catalog used to be written out at every call
/// site. Nothing is cached on the way out: "absent" comes from `ResourcesState`, which never
/// keeps it.
pub(crate) fn catalog_now<'a>(
    app: &AppHandle,
    resources: &'a ResourcesState,
    catalog: &'a CatalogState,
) -> Option<&'a Catalog> {
    resources.get(app).map(|rs| catalog.get_or_build(rs))
}

/// The Unlock view Live reads, kept until the save it was evaluated on is read again (card #80,
/// R10): the watcher reports a line every two seconds during a run, and the profile moves only
/// when the game writes the `.dat`. When to build is `ipc::PerSave`'s rule, tested there.
#[derive(Default)]
pub(crate) struct LiveUnlockState(pub(crate) ipc::PerSave<Save, ipc::UnlockView>);

/// The unlock graph, built once from the catalog and the rules compiled into the binary.
/// Same shape as `CatalogState`: expensive to build, cheap to consult. Rules that don't
/// parse can only be our own broken file, and they degrade like everything else — the
/// commands answer without graph info rather than failing.
#[derive(Default)]
pub(crate) struct GraphState(OnceLock<graph::build::Graph>);

impl GraphState {
    pub(crate) fn get(&self, catalog: &catalog::Catalog) -> Option<&graph::build::Graph> {
        if let Some(g) = self.0.get() {
            return Some(g);
        }
        let rules = graph::rules::embedded().ok()?;
        Some(
            self.0
                .get_or_init(|| graph::build::Graph::build(catalog, rules)),
        )
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
    pub(crate) fn get(&self, app: &AppHandle) -> Option<&ResourceSet> {
        if let Some(rs) = self.0.get() {
            return Some(rs);
        }
        let d = discovery_now(app);
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
/// The active profile's save, from the one read a screen pays for (N8). `NoActiveProfile`
/// when there isn't one: for the UI that means "go to selection", not an error to display.
///
/// Every rule about *when* the last read may stand lives in `ipc::SaveCache`, with its
/// tests; what is here is the I/O it asks for — the settings file, the walk of the Steam
/// libraries, `Save::open`, and the file's modified time. Both of the expensive ones run
/// only on a miss.
pub(crate) fn active_save(app: &AppHandle) -> Result<(ProfileId, Arc<Save>), IpcError> {
    let settings = settings_file::load(app);
    let cache = app.state::<SaveState>();
    cache.0.get(
        settings.active_profile_id.as_ref(),
        |path| std::fs::metadata(path).ok().and_then(|m| m.modified().ok()),
        || {
            let d = discovery_now(app);
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
            Ok((profile.id, candidate.path.clone()))
        },
        |path| {
            Save::open(path).map_err(|e| IpcError::UnreadableSave {
                reason: (&e).into(),
            })
        },
    )
}

/// The active profile's save, kept between commands. One screen is several commands and
/// they all ask for the same file in the same second; the game rewrites that file while the
/// app is open, which is why this is a cache with a rule and not a `OnceLock`.
#[derive(Default)]
pub(crate) struct SaveState(pub(crate) ipc::SaveCache<Save>);
/// The app's database, opened on first use and kept once it opens. **A failed open is not
/// kept** (card #80, R2): permissions, a full disk, a file another program holds — "an
/// expected failure is never cached", as `ResourcesState` already does, or the only way back
/// from a moment's failure would be a restart. The reason travels as a `StoreReason` and not
/// an `IpcError` because the only variant that would make sense here is `StoreUnavailable` —
/// the type pins that down, and `plan` puts it straight into the plan without a `match` that
/// would have to discard impossible variants.
#[derive(Default)]
pub(crate) struct StoreState {
    store: OnceLock<Mutex<Store>>,
    /// Held while opening: two commands arriving together would otherwise both run the
    /// migrations on the same file, and the second would meet the first's lock.
    opening: Mutex<()>,
}

impl StoreState {
    pub(crate) fn get_or_open(&self, app: &AppHandle) -> Result<&Mutex<Store>, ipc::StoreReason> {
        if let Some(store) = self.store.get() {
            return Ok(store);
        }
        let _opening = self.opening.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(store) = self.store.get() {
            return Ok(store);
        }
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|_| ipc::StoreReason::DataDirUnknown)?;
        std::fs::create_dir_all(&dir).map_err(|_| ipc::StoreReason::DataDirNotCreatable)?;
        let store =
            Store::open(&dir.join("isaacdome.db")).map_err(|e| ipc::StoreReason::from(&e))?;
        Ok(self.store.get_or_init(|| Mutex::new(store)))
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

/// The rules that turn a log line into an event, parsed once; the watcher's handle — kept only
/// because dropping it would end the watch; and what the archive's reading last met, which
/// the runs command sends out (card #80, R4).
#[derive(Default)]
pub(crate) struct ArchiveState {
    rules: OnceLock<run::Rules>,
    watcher: Mutex<Option<log_watch::LogWatcher>>,
    health: Mutex<ipc::ArchiveHealth>,
}

impl ArchiveState {
    pub(crate) fn rules(&self) -> &run::Rules {
        self.rules.get_or_init(run::Rules::embedded)
    }

    pub(crate) fn keep(&self, watcher: log_watch::LogWatcher) {
        if let Ok(mut guard) = self.watcher.lock() {
            *guard = Some(watcher);
        }
    }

    pub(crate) fn health(&self) -> ipc::ArchiveHealth {
        self.health
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub(crate) fn record(&self, change: impl FnOnce(&mut ipc::ArchiveHealth)) {
        change(&mut self.health.lock().unwrap_or_else(|e| e.into_inner()));
    }
}

/// When the game is not installed there is no catalog, and the fold still needs item kinds.
/// Everything accumulates: reading an unknown item as an active would silently drop whatever the
/// player was carrying, which is the same reading `ipc::CatalogKinds` gives an id it cannot find.
pub(crate) struct AllPassive;

impl run::ItemKinds for AllPassive {
    fn kind_of(&self, _id: u32) -> run::ItemKind {
        run::ItemKind::Passive
    }
}

/// Discovery as the user configured it: the folders chosen by hand (B14) tried before the
/// automatic search. **The one way to discover** in this crate (card #80, item 01) — five
/// places used to call `discover(&Options::default())`, so a save found in a chosen folder
/// was accepted by `select_profile` and then answered as `NoActiveProfile` everywhere else.
pub(crate) fn discovery_now(app: &AppHandle) -> Discovery {
    discover(&settings_file::options(app))
}
