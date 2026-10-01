# The Data page — where the app writes, and what is in it

Card #19 (B30), scope reduced by the owner on 2026-09-30.

## What it is for

A stranger who installs IsaacDome has three questions the app never answers today: *where does
it keep things*, *what do I copy to back it up*, *what do I delete to start over*. About says
"it only writes to its own folders" — a promise. The Data page shows it: which folder, which
files, how large, what they hold, and a button that opens the folder with the file selected.

## What it is not

**Moving the folder is out.** The card asked for it; the owner dropped it. `StoreState` is a
`OnceLock<Mutex<Store>>` that cannot be closed, the log watcher writes runs into the database
during a game, and the one realistic reason to move it — a OneDrive/Dropbox folder shared by
two PCs — is the way SQLite databases get corrupted. The reasons are on the card, to reopen it
with a real request.

Also out: a size that refreshes live while a game is played (the page reads when it opens),
and any change to About's wording (it stays true).

## What the app writes — measured 2026-09-30

Two files and nothing else: `settings.json` in `app_config_dir()` (`settings_file.rs`) and
`isaacdome.db` in `app_data_dir()` (`StoreState`). No image cache — game images are served
from memory over `isaac:`, the archives read in place. On Windows both directories resolve
to `%APPDATA%\dev.isaacdome.app`, but the page does not assume it: each file carries its own
folder.

## The view

```rust
#[serde(rename_all = "camelCase")]
pub struct DataView { pub files: Vec<DataFileView> }

#[serde(rename_all = "camelCase")]
pub struct DataFileView {
    pub file: DataFile,          // which one — also the argument of `reveal_data_file`
    pub state: DataFileState,
}

#[serde(rename_all = "camelCase")]
pub enum DataFile { Database, Settings }            // fieldless: a bare string

#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum DataFileState {
    Present { folder_hint: String, size_bytes: u64, contents: Option<StoreContents> },
    NotCreated { folder_hint: String },             // a fresh install, nothing written yet
    Unreadable { folder_hint: String, reason: StoreReason }, // the database only
    FolderUnknown,                                  // the OS could not name the directory
}

#[serde(rename_all = "camelCase")]
pub struct StoreContents {
    pub goals: u32,
    pub queue_rows: Option<u32>,   // None: the queue document does not parse
    pub sessions: u32,             // rows of `sources`
    pub runs: u32,                 // rows of `runs`, as last folded
    pub roll_saved: bool,
}
```

- `contents` is `Some` only for the database, and only when the store opened.
- `folder_hint` sits inside the states that have a folder — `FolderUnknown` has none, and an
  empty string would be a value pretending to be one. It is the folder's display string through `mask_user_dir`, the function
  `discovery`'s save hints already use. It moves out of `ipc::profile` into a module both
  call (`ipc::hint`), so the masking is written once.
- `DataFile` crosses **inward** as the argument of the reveal command. No path crosses in
  either direction.

## Rust

**`store`** — `Store::contents() -> Result<ipc::StoreContents, StoreError>` (`store` already
depends on `ipc`, so there is no second type): `COUNT(*)` on `goals`,
`sources`, `runs`; the queue's length through the existing `queue()` read (a document that
does not parse is `None`, not a failure); whether the `roll` row exists. Test-first, on a
temporary database: an empty store counts zero everywhere, each write moves exactly its count.

**`ipc`** — `data_view(facts) -> DataView`, pure. `app` gathers the facts (the two
directories as `Option<PathBuf>`, each file's `metadata` length or absence, the store's
contents or its reason) and `ipc` shapes them. The store is not asked when its file is
absent: opening it creates the file, and the page would never say "not created yet". Tests: the username is masked, an absent file
reads `NotCreated`, an unreadable store reads `Unreadable` with its reason and still carries
the settings row, an unknown directory reads `FolderUnknown`; plus a JSON-shape test pinning
`sizeBytes`, `folderHint`, `queueRows` (`rename_all_fields`).

**`app`** — two commands:

- `data_location() -> Result<DataView, IpcError>` — `metadata` on both files, never a read of
  their contents; the summary through `StoreState`. A store that will not open is a state in
  the payload, not an `Err`. Opening the store from this page is what every other screen
  already does at startup, so a fresh install creating its database here is no new behaviour.
- `reveal_data_file(file: DataFile) -> Result<(), IpcError>` — resolves the path in Rust and
  opens Explorer with the file selected, through `tauri-plugin-opener` called from Rust only
  (the dialog plugin's pattern: no npm package, no JS capability). When the file does not
  exist yet it opens the folder instead (`open_path`), since `reveal_item_in_dir` wants an
  item that exists. A failure is a new `IpcError::FolderNotOpenable`.
  Plugin: `tauri-plugin-opener` 2.7, `OpenerExt::opener().reveal_item_in_dir(path)`, returning
  `tauri_plugin_opener::Error`, mapped here and never printed. The docs do not say whether a
  Rust-side call passes through the capability ACL; Tauri's model says it does not, and the
  first window settles it — that is this card's `NEEDS WINDOW`.

**Measured after the build: the database is created at startup, not by this page.** The window
session (`window_session`, read by `App.vue` whenever tabs are resumed, on by default) and the
log backfill (whenever the game folder is found) both open the store before any screen. So on a
real install the database row reads `Present` with every count at zero, which is true; its
`NotCreated` is reachable only with tab resume off, no game folder and no store-backed screen
visited. `settings.json`'s `NotCreated` is the common case — it is written on the first save.
The page still never creates the file itself, and that is what its order of operations is for.

## UI

- Route `Data`, path `/settings/data`, in the Settings section of the sidebar after Updates,
  icon `DatabaseIcon`.
- `ui/src/screens/DataScreen.vue`, reading through `ui/src/lib/ipc/data.ts`; dev-server
  fixtures for the four states.
- A row per file: the file name, the folder hint, the size, the state line, and for the
  database the contents as a short list; "Show in folder" on each row.
- Sizes through a pure `formatSize(bytes, locale)` (`Intl.NumberFormat`, unit
  `byte`/`kilobyte`/`megabyte`), with Vitest cases at the unit boundaries.
- i18n in `it` and `en`.

## Documents that move in the same work

- `docs/architecture.md`: commands 43 → 45, routes 16 → 17, the diagrams redrawn.
- `CLAUDE.md`: `opener` joins the Tauri plugins in the stack line.
- `docs/BACKLOG.md`: B30 rewritten to this scope.
- The card's checklist: the "moving it" item leaves; the rest stays.

## Done when

The Data page names each file's folder in words, says how large it is and what the database
holds; a file not created yet and a database that will not open are said, not crashed on;
"Show in folder" opens Explorer on the file; and no full path has crossed the IPC boundary.
