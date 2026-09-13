# Background and tray Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Closing the last window stops the app today; after this it leaves the app running in
the notification area, one click brings a window back with the tabs it had, and launching the
executable twice never produces two processes.

**Architecture:** The Tauri event loop stops exiting (`RunEvent::ExitRequested` with
`code: None` is prevented while the setting is on). A tray icon, always present, and the
single-instance plugin both call one function, `open_or_focus`, which either brings an existing
window forward or rebuilds the main one from the entry already in `tauri.conf.json`
(`create: false` + `WebviewWindowBuilder::from_config`). Every decision that has a return value
worth checking — which window to focus, what the tray says, whether a session document is
acceptable — lives in the pure `ipc` crate; `app` keeps only the wiring. The tabs survive in a
versioned JSON document in `store` (migration 3), written by the main window as its tabs
change, read by the main window when it is born.

**Tech Stack:** Rust, Tauri 2.11.5 (`tray-icon` feature), `tauri-plugin-single-instance` 2.4.4,
`tauri-plugin-notification` 2.4.0, `sys-locale` 0.3.2, `rusqlite`, Vue 3 + TypeScript, Pinia,
Vitest.

**Spec:** `docs/superpowers/specs/2026-09-13-background-and-tray-design.md`

## Global Constraints

- **Tauri commands return `Result<T, IpcError>`**, never `Result<T, String>`.
- **Every struct crossing the IPC** carries `#[serde(rename_all = "camelCase")]`, and every
  enum with struct variants also `rename_all_fields = "camelCase"`.
- **A fieldless enum on the IPC is a bare camelCase string**, not a tagged union, and its
  TypeScript mirror is a union of values. Zero exceptions in the repo.
- **No `_ =>` arm on a closed enum.** The single exception this plan introduces is
  `tauri::RunEvent`, which is `#[non_exhaustive]` and not ours; it carries a comment saying so.
- **No `panic!` / `unwrap()`** outside tests on anything read from disk.
- **No `invoke()` in components** — only typed wrappers in `ui/src/lib/ipc/`. **Nothing outside
  `ui/src/lib/window/` imports `@tauri-apps/api`'s `window`, `webviewWindow` or `event`**, and
  every such module degrades outside Tauri instead of throwing.
- **No `<style>` in SFCs, no hardcoded visual constants, no raw `<button>`/`<input>`, no string
  unions** (`const X = { … } as const`). `pnpm scan` enforces all five plus the ban on visible
  strings in templates.
- **Both locales, always**: every new message key exists in `ui/src/i18n/messages/it.ts` **and**
  `en.ts`; the schema is `it.ts`, so a key missing from `en.ts` is a type error.
- **Commits**: Conventional Commits, `type(scope): subject`, English, atomic, scope = crate or
  package (`ipc`, `store`, `app`, `ui`). **Never** a `Co-Authored-By` trailer or any reference
  to Claude.
- **Before declaring anything done**: `pnpm check` (`cargo fmt --check`, `cargo clippy
  --all-targets -- -D warnings`, `cargo test --workspace`, `pnpm typecheck`, `pnpm ui:test`,
  `pnpm lint`, `pnpm format:check`, `pnpm scan`). There is no CI.
- **Do not `git add -A`**: other sessions edit `docs/superpowers/` in parallel. Stage by path.

---

### Task 1: `ipc` — which window a tray click means

**Files:**
- Create: `crates/ipc/src/tray.rs`
- Modify: `crates/ipc/src/lib.rs` (add `mod tray;` and the `pub use`)
- Test: `crates/ipc/src/tray.rs` (`#[cfg(test)] mod tests`, the crate's own convention)

**Interfaces:**
- Consumes: nothing.
- Produces: `ipc::TrayAction` (`enum { Focus { label: String }, CreateMain }`) and
  `ipc::tray_action(labels: &[String]) -> TrayAction`.

- [ ] **Step 1: Write the failing tests**

In `crates/ipc/src/tray.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn labels(of: &[&str]) -> Vec<String> {
        of.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn with_no_window_open_a_click_creates_the_main_one() {
        assert_eq!(tray_action(&[]), TrayAction::CreateMain);
    }

    #[test]
    fn the_main_window_is_the_one_a_click_means_whatever_else_is_open() {
        let action = tray_action(&labels(&["win-abc", "main", "tab-preview"]));
        assert_eq!(
            action,
            TrayAction::Focus {
                label: "main".to_string()
            }
        );
    }

    #[test]
    fn without_the_main_window_a_click_means_the_oldest_torn_off_one() {
        // `win-<ms in base 36>`: same width, so the smallest string is the oldest window.
        let action = tray_action(&labels(&["win-mfk2h9", "win-mfk1zz"]));
        assert_eq!(
            action,
            TrayAction::Focus {
                label: "win-mfk1zz".to_string()
            }
        );
    }

    #[test]
    fn the_drag_preview_is_never_the_window_a_click_means() {
        // It carries no tab: focusing it would hand the user a ghost.
        assert_eq!(tray_action(&labels(&["tab-preview"])), TrayAction::CreateMain);
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p ipc tray`
Expected: FAIL — `cannot find function tray_action in this scope`.

- [ ] **Step 3: Write the implementation**

```rust
//! What a click on the tray icon means. The tray itself is Tauri's, and lives in the `app`
//! crate; the decision is here, where it can be tested.

/// The label of the drag's preview window (`ui/src/lib/window/preview.ts`). It holds no tab
/// and must never be the window a click brings forward.
const PREVIEW_LABEL: &str = "tab-preview";

/// The first window's label, fixed by `tauri.conf.json`.
const MAIN_LABEL: &str = "main";

/// What the tray does with a click, given the windows that exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayAction {
    /// Bring this window forward: it may be behind, hidden or minimized.
    Focus { label: String },
    /// There is nothing to bring forward. Build the main window.
    CreateMain,
}

/// The main window if it is open; otherwise the oldest window that holds tabs; otherwise
/// "create one".
///
/// **The oldest is the smallest label.** A torn-off window is `win-<milliseconds in base 36>`
/// (`newWindowLabel`), which is six characters from 2005 to 2059 — same width, so lexicographic
/// order is chronological order. The day that stops being true, this comment is the one that
/// explains why the wrong window came forward.
pub fn tray_action(labels: &[String]) -> TrayAction {
    if labels.iter().any(|l| l == MAIN_LABEL) {
        return TrayAction::Focus {
            label: MAIN_LABEL.to_string(),
        };
    }
    match labels
        .iter()
        .filter(|l| l.as_str() != PREVIEW_LABEL)
        .min()
    {
        Some(label) => TrayAction::Focus {
            label: label.clone(),
        },
        None => TrayAction::CreateMain,
    }
}
```

In `crates/ipc/src/lib.rs`, beside the other modules (alphabetical): `mod tray;` and
`pub use tray::{tray_action, TrayAction};`.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p ipc tray`
Expected: PASS, 4 tests.

- [ ] **Step 5: Commit**

```bash
git add crates/ipc/src/tray.rs crates/ipc/src/lib.rs
git commit -m "feat(ipc): what a click on the tray icon means"
```

---

### Task 2: `ipc` — what the tray says, in the user's language

**Files:**
- Modify: `crates/ipc/src/tray.rs`
- Test: same file, same `mod tests`

**Interfaces:**
- Consumes: Task 1's module.
- Produces: `ipc::TrayLocale` (`enum { It, En }`), `ipc::tray_locale(tag: &str) -> TrayLocale`,
  `ipc::TrayText { open, quit, notice_title, notice_body, tooltip }` and
  `ipc::tray_text(locale: TrayLocale) -> TrayText`, all fields `&'static str`.

- [ ] **Step 1: Write the failing tests**

Append to `mod tests`:

```rust
    #[test]
    fn the_tray_speaks_the_systems_language_by_its_primary_subtag() {
        // The rule `ui/src/i18n/locale.ts` applies to navigator.languages, applied here to
        // the system tag: primary subtag, English when we don't have the language.
        assert_eq!(tray_locale("it-IT"), TrayLocale::It);
        assert_eq!(tray_locale("it"), TrayLocale::It);
        assert_eq!(tray_locale("IT"), TrayLocale::It);
        assert_eq!(tray_locale("en-GB"), TrayLocale::En);
        assert_eq!(tray_locale("fr"), TrayLocale::En);
        assert_eq!(tray_locale(""), TrayLocale::En);
    }

    #[test]
    fn every_string_the_tray_shows_exists_in_both_languages() {
        for locale in [TrayLocale::It, TrayLocale::En] {
            let t = tray_text(locale);
            for s in [t.open, t.quit, t.notice_title, t.notice_body, t.tooltip] {
                assert!(!s.trim().is_empty(), "empty string for {locale:?}");
            }
        }
        // Two languages, not one twice: a copy-paste would leave them equal.
        assert_ne!(tray_text(TrayLocale::It).quit, tray_text(TrayLocale::En).quit);
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p ipc tray`
Expected: FAIL — `cannot find function tray_locale in this scope`.

- [ ] **Step 3: Write the implementation**

Append to `crates/ipc/src/tray.rs`:

```rust
/// The languages the app speaks. **Not an IPC type**: the tray is built before any window
/// exists, and these strings never cross to the frontend, which has vue-i18n.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayLocale {
    It,
    En,
}

/// The system's language tag, read as one of ours. Same rule as `ui/src/i18n/locale.ts`:
/// primary subtag, English as the fallback. Two implementations of one rule, which is the
/// price of a menu that has to exist before the frontend does.
pub fn tray_locale(tag: &str) -> TrayLocale {
    match tag.split('-').next().unwrap_or("").to_ascii_lowercase().as_str() {
        "it" => TrayLocale::It,
        _ => TrayLocale::En,
    }
}

/// Everything the tray shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrayText {
    pub open: &'static str,
    pub quit: &'static str,
    pub notice_title: &'static str,
    pub notice_body: &'static str,
    pub tooltip: &'static str,
}

pub fn tray_text(locale: TrayLocale) -> TrayText {
    match locale {
        TrayLocale::It => TrayText {
            open: "Apri IsaacDome",
            quit: "Esci",
            notice_title: "IsaacDome resta aperta",
            notice_body: "Clicca l'icona qui accanto all'orologio per riaprire la finestra.",
            tooltip: "IsaacDome — clicca per aprire la finestra",
        },
        TrayLocale::En => TrayText {
            open: "Open IsaacDome",
            quit: "Quit",
            notice_title: "IsaacDome is still running",
            notice_body: "Click the icon next to the clock to bring the window back.",
            tooltip: "IsaacDome — click to open the window",
        },
    }
}
```

Extend the `pub use` in `lib.rs`:
`pub use tray::{tray_action, tray_locale, tray_text, TrayAction, TrayLocale, TrayText};`

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p ipc tray`
Expected: PASS, 6 tests.

- [ ] **Step 5: Commit**

```bash
git add crates/ipc/src/tray.rs crates/ipc/src/lib.rs
git commit -m "feat(ipc): the tray's own strings, in both languages"
```

---

### Task 3: `ipc` — the three new settings and the session cap

**Files:**
- Modify: `crates/ipc/src/settings.rs`
- Test: `crates/ipc/src/settings.rs` (`#[cfg(test)] mod tests` — add one if the file has none)

**Interfaces:**
- Consumes: nothing.
- Produces: `Settings { active_profile_id, scale, stay_in_background: bool, resume_tabs: bool,
  background_notice_shown: bool }`, `Settings::with_stay_in_background(bool) -> Settings`,
  `Settings::with_resume_tabs(bool) -> Settings`,
  `Settings::with_notice_shown(bool) -> Settings`, and
  `ipc::session_document_fits(&str) -> bool` with `ipc::MAX_SESSION_BYTES`.

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // A settings file written before this feature has none of the three keys. It must read,
    // and read as "background on": the default is the behaviour, not the opt-in.
    #[test]
    fn a_settings_file_from_before_this_feature_reads_with_the_background_on() {
        let old = r#"{"activeProfileId":null,"scale":110}"#;
        let s: Settings = serde_json::from_str(old).expect("old settings still read");
        assert_eq!(s.scale, 110);
        assert!(s.stay_in_background);
        assert!(s.resume_tabs);
        assert!(!s.background_notice_shown);
    }

    #[test]
    fn the_three_flags_serialize_as_camel_case() {
        let json = serde_json::to_string(&Settings::default()).expect("serializes");
        assert!(json.contains("\"stayInBackground\""), "{json}");
        assert!(json.contains("\"resumeTabs\""), "{json}");
        assert!(json.contains("\"backgroundNoticeShown\""), "{json}");
    }

    // The same reason `with_scale` exists: a write must never forget the other fields.
    #[test]
    fn changing_one_flag_keeps_every_other_field() {
        let s = Settings {
            scale: 125,
            resume_tabs: false,
            ..Settings::default()
        }
        .with_stay_in_background(false);
        assert_eq!(s.scale, 125);
        assert!(!s.resume_tabs);
        assert!(!s.stay_in_background);
    }

    #[test]
    fn a_session_document_is_refused_past_the_cap_and_accepted_at_it() {
        assert!(session_document_fits(&"a".repeat(MAX_SESSION_BYTES)));
        assert!(!session_document_fits(&"a".repeat(MAX_SESSION_BYTES + 1)));
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p ipc settings`
Expected: FAIL — `Settings` has no field `stay_in_background`.

- [ ] **Step 3: Write the implementation**

In `crates/ipc/src/settings.rs`, add to the struct (keeping `#[serde(rename_all =
"camelCase", default)]` as it is) and to `Default`:

```rust
    /// Closing the last window leaves the app in the notification area instead of ending it.
    /// **On by default**: it is the behaviour, and the switch exists for whoever doesn't want
    /// an app in their tray.
    pub stay_in_background: bool,
    /// A window born with nothing owed to it opens on the tabs of the last session.
    pub resume_tabs: bool,
    /// Whether the one-time "it's still running" notice has been shown. Written by the app,
    /// never shown to the user.
    pub background_notice_shown: bool,
```

`Default` gives `stay_in_background: true`, `resume_tabs: true`,
`background_notice_shown: false`.

Three constructors beside `with_scale`, each `Settings { field: value, ..self.clone() }`:

```rust
    pub fn with_stay_in_background(&self, stay: bool) -> Settings {
        Settings {
            stay_in_background: stay,
            ..self.clone()
        }
    }

    pub fn with_resume_tabs(&self, resume: bool) -> Settings {
        Settings {
            resume_tabs: resume,
            ..self.clone()
        }
    }

    pub fn with_notice_shown(&self, shown: bool) -> Settings {
        Settings {
            background_notice_shown: shown,
            ..self.clone()
        }
    }
```

And, at the end of the file:

```rust
/// The largest session document the app will store. Nothing should approach it — fifty tabs
/// of route names and queries are a few kilobytes — and a database that grows without a bound
/// is how you find out that something did.
pub const MAX_SESSION_BYTES: usize = 64 * 1024;

/// Whether a document offered by the frontend is small enough to keep. Refused, never
/// truncated: half a JSON document is not a session.
pub fn session_document_fits(document: &str) -> bool {
    document.len() <= MAX_SESSION_BYTES
}
```

Export from `lib.rs`: the `settings` line becomes
`pub use settings::{session_document_fits, snap_percent, Settings, DEFAULT_SCALE, MAX_SESSION_BYTES, SCALE_PERCENTS};`
(keep whatever the line already exports, adding the two new names).

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p ipc settings`
Expected: PASS, 4 tests. Then `cargo test --workspace` — `Settings` gained fields with
defaults, so nothing else should break; if a struct literal somewhere stops compiling, add
`..Settings::default()` rather than listing the new fields.

- [ ] **Step 5: Commit**

```bash
git add crates/ipc/src/settings.rs crates/ipc/src/lib.rs
git commit -m "feat(ipc): the background settings and the session document's cap"
```

---

### Task 4: `store` — migration 3, the session document

**Files:**
- Modify: `crates/store/src/migrations.rs`
- Modify: `crates/store/src/lib.rs`
- Test: `crates/store/tests/session.rs` (create — this crate's tests live in `tests/`, one file
  per subject: `goals.rs`, `queue.rs`, `degrade.rs`, `reason.rs`)

**Interfaces:**
- Consumes: nothing.
- Produces: `Store::session() -> Result<Option<String>, StoreError>` and
  `Store::set_session(Option<&str>) -> Result<(), StoreError>`; `SCHEMA_VERSION == 3`.

- [ ] **Step 1: Write the failing tests**

```rust
//! The window session on disk: one document, replaced whole, and the difference between
//! "there isn't one" and "there is an empty one".

use store::{Store, SCHEMA_VERSION};
use tempfile::{tempdir, TempDir};

// The same shape `queue.rs` uses: a real file in a temporary directory, because `Store::open`
// is what runs the migrations and that is half of what these tests are about.
fn store() -> (TempDir, Store) {
    let dir = tempdir().expect("temp dir");
    let s = Store::open(&dir.path().join("isaacdome.db")).expect("opens");
    (dir, s)
}

#[test]
fn the_schema_knows_the_session() {
    assert_eq!(SCHEMA_VERSION, 3);
}

#[test]
fn a_fresh_database_has_no_session() {
    let (_dir, s) = store();
    assert_eq!(s.session().expect("reads"), None);
}

#[test]
fn the_document_written_is_the_document_read() {
    let (_dir, s) = store();
    let document = r#"{"version":1,"tabs":[],"activeIndex":0}"#;
    s.set_session(Some(document)).expect("writes");
    assert_eq!(s.session().expect("reads").as_deref(), Some(document));
}

#[test]
fn a_session_survives_reopening() {
    // The point of a file rather than an in-memory database: a session is for the next launch.
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("isaacdome.db");
    {
        let s = Store::open(&path).expect("opens");
        s.set_session(Some("one")).expect("writes");
    }
    let s = Store::open(&path).expect("reopens");
    assert_eq!(s.session().expect("reads").as_deref(), Some("one"));
}

#[test]
fn a_second_write_replaces_the_first_rather_than_adding_a_second_answer() {
    let (_dir, s) = store();
    s.set_session(Some("one")).expect("writes");
    s.set_session(Some("two")).expect("writes again");
    assert_eq!(s.session().expect("reads").as_deref(), Some("two"));
}

#[test]
fn clearing_leaves_no_session_behind() {
    let (_dir, s) = store();
    s.set_session(Some("one")).expect("writes");
    s.set_session(None).expect("clears");
    assert_eq!(s.session().expect("reads"), None);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p store session`
Expected: FAIL — `SCHEMA_VERSION` is 2, and `session` is not a method.

- [ ] **Step 3: Write the implementation**

In `migrations.rs`: `SCHEMA_VERSION` becomes `3`, `MIGRATIONS` becomes `[&str; 3]`, and the
third entry is appended (never modify the first two):

```rust
    // 3: the window session, one JSON document written by the main window. An object with a
    // version, not a bare array of tabs: 3.7 adds the sidebar's width and each table's size
    // (B27) as named parts of the same document, and a named part costs no migration. What
    // the parts mean is the frontend's business — this crate stores the string.
    "CREATE TABLE window_session (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        document TEXT NOT NULL
    );",
```

In `lib.rs`, beside `queue`/`set_queue`:

```rust
    /// The session document, as it was written, or `None` when there isn't one.
    ///
    /// **No nested `Result` here, unlike `queue()`**: this crate cannot tell a good document
    /// from a bad one — the shape belongs to the frontend — so "it doesn't parse" is not a
    /// state it can report. The frontend answers that question, and answers it with the
    /// landing tab.
    pub fn session(&self) -> Result<Option<String>, StoreError> {
        self.conn
            .query_row("SELECT document FROM window_session WHERE id = 1", [], |r| {
                r.get(0)
            })
            .optional()
            .map_err(StoreError::from_sqlite)
    }

    /// Replaces the document, or removes it when there is nothing to keep.
    pub fn set_session(&self, document: Option<&str>) -> Result<(), StoreError> {
        match document {
            Some(raw) => self
                .conn
                .execute(
                    "INSERT INTO window_session (id, document) VALUES (1, ?1)
                     ON CONFLICT(id) DO UPDATE SET document = excluded.document",
                    params![raw],
                )
                .map(|_| ()),
            None => self
                .conn
                .execute("DELETE FROM window_session WHERE id = 1", [])
                .map(|_| ()),
        }
        .map_err(StoreError::from_sqlite)
    }
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p store`
Expected: PASS. The existing migration tests must pass unchanged — if one pins "version 2",
it is a fixture of the old era and its expected value moves to 3, but a test that pins *the
goals surviving a migration* must not be weakened.

- [ ] **Step 5: Commit**

```bash
git add crates/store/src/migrations.rs crates/store/src/lib.rs crates/store/tests/session.rs
git commit -m "feat(store): migration 3, the window session as one document"
```

---

### Task 5: `app` — the settings and session commands

**Files:**
- Modify: `crates/app/src/commands/profile.rs`
- Create: `crates/app/src/commands/session.rs`
- Modify: `crates/app/src/commands/mod.rs`, `crates/app/src/lib.rs` (register the commands)

**Interfaces:**
- Consumes: Task 3's `Settings` constructors, Task 4's `Store::session` / `Store::set_session`.
- Produces: commands `set_stay_in_background(stay: bool) -> Settings`,
  `set_resume_tabs(resume: bool) -> Settings`, `window_session() -> Option<String>`,
  `set_window_session(document: Option<String>) -> ()`.

**No tests**: this is wiring, and the `app` crate is not tested. Everything with a return value
worth checking was Tasks 1–4.

- [ ] **Step 1: Write the two settings commands**

In `crates/app/src/commands/profile.rs`, after `set_scale` and shaped like it — write, announce,
answer the settings as they now are:

```rust
/// Whether the app stays in the notification area when the last window closes.
#[tauri::command]
pub fn set_stay_in_background(app: AppHandle, stay: bool) -> Result<Settings, IpcError> {
    let settings = settings_file::load(&app).with_stay_in_background(stay);
    settings_file::save(&app, &settings)?;
    announce(&app, SETTINGS_CHANGED);
    Ok(settings)
}

/// Whether a window born with nothing owed to it opens on the last session's tabs.
/// Turning it off clears what was stored: the app should not keep a record the user has just
/// said they don't want.
#[tauri::command]
pub fn set_resume_tabs(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    resume: bool,
) -> Result<Settings, IpcError> {
    let settings = settings_file::load(&app).with_resume_tabs(resume);
    settings_file::save(&app, &settings)?;
    if !resume {
        // A store that won't open is not a reason to refuse the setting: the flag is off, so
        // nothing will be read back either way.
        if let Ok(guard) = store.lock(&app) {
            let _ = guard.set_session(None);
        }
    }
    announce(&app, SETTINGS_CHANGED);
    Ok(settings)
}
```

Add `use crate::state::StoreState;` to the file's imports.

- [ ] **Step 2: Write the session commands**

Create `crates/app/src/commands/session.rs`:

```rust
//! The window session: one opaque document, written by the main window as its tabs change and
//! read by the main window when it is born. Its shape is the frontend's — a route name and a
//! query are not things this side has a type for — so the only judgements made here are the
//! setting and the cap.

use tauri::AppHandle;

use ipc::{session_document_fits, IpcError};

use crate::settings_file;
use crate::state::StoreState;

/// The stored session, or `None`. **`None` when the setting is off**, whatever the database
/// holds: "off" is answered in one place rather than in every caller.
///
/// An unreadable store is `None` too, not an error: a session that can't be read is a landing
/// tab, never a dialog.
#[tauri::command]
pub fn window_session(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
) -> Result<Option<String>, IpcError> {
    if !settings_file::load(&app).resume_tabs {
        return Ok(None);
    }
    Ok(store
        .lock(&app)
        .ok()
        .and_then(|guard| guard.session().ok())
        .flatten())
}

/// Replaces the session, or clears it with `None`. Silently does nothing when the setting is
/// off — the frontend shouldn't have to check twice — and refuses a document past the cap.
#[tauri::command]
pub fn set_window_session(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    document: Option<String>,
) -> Result<(), IpcError> {
    if !settings_file::load(&app).resume_tabs {
        return Ok(());
    }
    if document.as_deref().is_some_and(|d| !session_document_fits(d)) {
        return Err(IpcError::SessionTooLarge);
    }
    let guard = store.lock(&app).map_err(store_unavailable)?;
    guard
        .set_session(document.as_deref())
        .map_err(store_error)
}
```

`IpcError::SessionTooLarge` **does not exist yet** — it is part of this task. Add it to
`crates/ipc/src/error.rs`, fieldless like `UnknownTarget` and `WikiUnavailable`:

```rust
    /// The session document offered is past `MAX_SESSION_BYTES`. Nothing the user did: a
    /// frontend bug, reported rather than truncated, because half a document is not a session.
    SessionTooLarge,
```

and mirror it in `ui/src/lib/ipc/types.ts` in Task 10: `| { kind: 'sessionTooLarge' }`.

`store_error` takes a `StoreError` **by value** and `store_unavailable` a `StoreReason`
(`crates/store/src/degrade.rs`) — both are imported by `queue.rs` already, which is the command
file to read if any of this doesn't line up.

- [ ] **Step 3: Register everything**

`crates/app/src/commands/mod.rs`: add `pub mod session;`.
`crates/app/src/lib.rs`: add to `generate_handler!`, in the group where the other settings
commands are: `profile::set_stay_in_background`, `profile::set_resume_tabs`,
`session::window_session`, `session::set_window_session`.

- [ ] **Step 4: Verify it builds**

Run: `cargo clippy -p app --all-targets -- -D warnings`
Expected: no warnings, no errors.

- [ ] **Step 5: Commit**

```bash
git add crates/app/src/commands/profile.rs crates/app/src/commands/session.rs crates/app/src/commands/mod.rs crates/app/src/lib.rs
git commit -m "feat(app): the background settings and the session commands"
```

---

### Task 6: `app` — one window recipe, and the event loop that doesn't exit

**Files:**
- Modify: `crates/app/tauri.conf.json`
- Create: `crates/app/src/window.rs`
- Modify: `crates/app/src/lib.rs`
- Modify: `crates/app/Cargo.toml`
- Test: `ui/src/assets/background.test.ts` (one new assertion)

**Interfaces:**
- Consumes: `ipc::tray_action`, `ipc::TrayAction`, `Settings::stay_in_background`.
- Produces: `crate::window::open_or_focus(app: &AppHandle) -> ()` — infallible on the outside:
  a window that won't open is not a reason to kill the process.

- [ ] **Step 1: Write the failing test**

In `ui/src/assets/background.test.ts`, inside the same `describe`:

```ts
  // The window is declared here and built by Rust on demand (`crates/app/src/window.rs`), so
  // that the tray and the first launch open the same window. Were it created at startup too,
  // Tauri would build it *and* `setup` would try to build a second one under the same label.
  it('leaves the window for Rust to create', () => {
    const [window] = tauri.app.windows
    expect(window?.create).toBe(false)
    expect(window?.label).toBe('main')
  })
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `pnpm --filter ui test -- background`
Expected: FAIL — `expected undefined to be false`.

- [ ] **Step 3: Make the config declare it**

In `crates/app/tauri.conf.json`, the window entry gains two keys and keeps every other one
exactly as it is:

```json
      {
        "label": "main",
        "create": false,
        "title": "IsaacDome",
        "width": 1280,
        "height": 800,
        "decorations": false,
        "backgroundColor": "#150e0d"
      }
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `pnpm --filter ui test -- background`
Expected: PASS. (The app itself now opens no window at all — Step 5 is what gives it one
back. Don't run `pnpm dev` between these two steps and conclude anything.)

- [ ] **Step 5: Write the window module**

Create `crates/app/src/window.rs`:

```rust
//! The one place a window is opened. Three callers — the first launch, the tray, a second
//! launch of the executable — and one recipe, the entry in `tauri.conf.json`, so the window
//! the tray gives back is the window the app started with.

use ipc::{tray_action, TrayAction};
use tauri::{AppHandle, Manager, WebviewWindowBuilder};

/// Brings a window forward, or builds the main one if there is none.
///
/// Infallible on the outside: this is called from a tray click and from the event loop, and a
/// window that refuses to open is a click that did nothing, never a process that dies.
pub fn open_or_focus(app: &AppHandle) {
    let labels: Vec<String> = app.webview_windows().keys().cloned().collect();
    match tray_action(&labels) {
        TrayAction::Focus { label } => {
            if let Some(w) = app.get_webview_window(&label) {
                // Three states, three calls: a window can be minimized, hidden, or merely
                // behind another, and only the third is what `set_focus` alone fixes.
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
        TrayAction::CreateMain => {
            let Some(config) = app.config().app.windows.first().cloned() else {
                return;
            };
            match WebviewWindowBuilder::from_config(app, &config) {
                Ok(builder) => {
                    let _ = builder.build();
                }
                Err(_) => (),
            }
        }
    }
}
```

- [ ] **Step 6: Wire the event loop**

In `crates/app/src/lib.rs`: `mod window;`, and the tail of `run()` changes from `.run(...)` to

```rust
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
            tauri::RunEvent::ExitRequested { code: None, api, .. }
                if settings_file::load(app).stay_in_background =>
            {
                api.prevent_exit();
            }
            // `RunEvent` is `#[non_exhaustive]` and is not ours: this is the one catch-all the
            // repo's exhaustiveness rule cannot ask us to remove.
            _ => (),
        });
```

- [ ] **Step 7: Check it by hand**

Run: `pnpm dev`
Expected: the window opens as before. Close it: the window goes, the terminal does **not**
return — the process is alive with no window. `Ctrl+C` to stop it. (There is no way back in
yet; the tray is Task 7.)

- [ ] **Step 8: Commit**

```bash
git add crates/app/tauri.conf.json crates/app/src/window.rs crates/app/src/lib.rs ui/src/assets/background.test.ts
git commit -m "feat(app): the last window closes without ending the app"
```

---

### Task 7: `app` — the tray icon, and one instance

**Files:**
- Modify: `crates/app/Cargo.toml`
- Create: `crates/app/src/tray.rs`
- Modify: `crates/app/src/lib.rs`

**Interfaces:**
- Consumes: `window::open_or_focus`, `ipc::{tray_text, tray_locale, TrayText}`.
- Produces: `crate::tray::build(app: &AppHandle) -> ()`, called from `setup`.

**No tests**: wiring. The verification is Step 5's list, done by hand.

- [ ] **Step 1: Add the dependencies**

`crates/app/Cargo.toml`:

```toml
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-single-instance = "2"
tauri-plugin-notification = "2"
sys-locale = "0.3"
```

Run `cargo fetch` and check `Cargo.lock` picked up `tauri-plugin-single-instance 2.4.4`,
`tauri-plugin-notification 2.4.0`, `sys-locale 0.3.2` — the versions the spec's §0 resolved.

- [ ] **Step 2: Write the tray module**

Create `crates/app/src/tray.rs`:

```rust
//! The icon in the notification area. Always present, whether or not a window is open: the
//! app is running either way, and an icon that appears and disappears is an icon nobody
//! recognises.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use ipc::{tray_locale, tray_text, TrayText};

const OPEN_ID: &str = "open";
const QUIT_ID: &str = "quit";

/// The strings the tray shows, in the system's language.
pub fn text() -> TrayText {
    tray_text(tray_locale(
        sys_locale::get_locale().unwrap_or_default().as_str(),
    ))
}

/// Builds the icon and its menu. A failure here is an app without a tray, not an app that
/// won't start: the windows still work.
pub fn build(app: &AppHandle) {
    let t = text();
    let Ok(open) = MenuItem::with_id(app, OPEN_ID, t.open, true, None::<&str>) else {
        return;
    };
    let Ok(quit) = MenuItem::with_id(app, QUIT_ID, t.quit, true, None::<&str>) else {
        return;
    };
    let Ok(menu) = Menu::with_items(app, &[&open, &quit]) else {
        return;
    };
    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        // The left button is the shortcut, the right one is the menu. With the default, a
        // left click would open the menu and there would be no one-click way back.
        .show_menu_on_left_click(false)
        .tooltip(t.tooltip)
        .on_menu_event(|app, event| match event.id().as_ref() {
            OPEN_ID => crate::window::open_or_focus(app),
            QUIT_ID => app.exit(0),
            _ => (),
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::window::open_or_focus(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    let _ = builder.build(app);
}
```

- [ ] **Step 3: Wire the plugin and the tray**

In `crates/app/src/lib.rs`, `mod tray;`, and the builder starts with the plugin — the plugin's
own requirement is to be registered first:

```rust
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // A second launch is the same gesture as a click on the icon: this app has no
            // command line, so the arguments are nothing to act on.
            window::open_or_focus(app);
        }))
        .plugin(tauri_plugin_notification::init())
        .manage(CatalogState::default())
        // … the rest unchanged
```

and `setup` gains the tray, before the window so that a failure to build a window still leaves
a way back in:

```rust
        .setup(|app| {
            tray::build(app.handle());
            window::open_or_focus(app.handle());
            Ok(())
        })
```

- [ ] **Step 4: Build it**

Run: `cargo clippy -p app --all-targets -- -D warnings`
Expected: clean.

- [ ] **Step 5: Check it by hand** — this is the task's real test

Run: `pnpm dev`, then:

1. The icon is in the notification area while the window is open.
2. Close the window: the icon stays, the process stays.
3. Left-click the icon: a window comes back.
4. Right-click: **Open** and **Quit**, in the system's language.
5. **Quit**: the process ends, the terminal returns.
6. With the app in the tray, start it again (`pnpm dev` a second time, or the built exe twice):
   no second process, the existing window comes forward.
7. Tear a tab off into its own window, close the main window, click the icon: the torn-off
   window comes forward, no second window is born.

Write down which of the seven you actually ran — the report says so, and a step skipped is a
step nobody checked.

- [ ] **Step 6: Commit**

```bash
git add crates/app/Cargo.toml crates/app/src/tray.rs crates/app/src/lib.rs Cargo.lock
git commit -m "feat(app): an icon in the tray, and only ever one instance"
```

---

### Task 8: `app` — the notice, once

**Files:**
- Modify: `crates/app/src/lib.rs`
- Modify: `crates/app/src/tray.rs`

**Interfaces:**
- Consumes: `tray::text()`, `Settings::with_notice_shown`, `settings_file::{load, save}`.
- Produces: `crate::tray::notice_once(app: &AppHandle) -> ()`.

- [ ] **Step 1: Write it**

Append to `crates/app/src/tray.rs`:

```rust
use tauri_plugin_notification::NotificationExt;

/// Says, once ever, that the app is still running. **The result is ignored on purpose**: a
/// Windows toast wants a registered AppUserModelID, which an installed build has and a
/// development run may not, and an app that refused to stay in the tray because a toast
/// didn't appear would be worse than a silent one. The tooltip carries the same sentence
/// either way.
pub fn notice_once(app: &AppHandle) {
    let settings = crate::settings_file::load(app);
    if settings.background_notice_shown {
        return;
    }
    let t = text();
    let _ = app
        .notification()
        .builder()
        .title(t.notice_title)
        .body(t.notice_body)
        .show();
    // Written whether or not it appeared: a notice that failed twice is not worth a third try.
    let _ = crate::settings_file::save(app, &settings.with_notice_shown(true));
}
```

- [ ] **Step 2: Call it from the arm that prevents the exit**

In `lib.rs`, the `ExitRequested` arm becomes:

```rust
            tauri::RunEvent::ExitRequested { code: None, api, .. }
                if settings_file::load(app).stay_in_background =>
            {
                api.prevent_exit();
                tray::notice_once(app);
            }
```

- [ ] **Step 3: Check it by hand**

Run: `pnpm dev`, close the window. Expected: the notification appears **or it doesn't** — both
are acceptable outcomes of this step, and which one happened goes in the report. What must be
true either way: the process is still alive, and closing a window a second time shows nothing.
Delete `background_notice_shown` from the settings file (`%APPDATA%\dev.isaacdome.app\settings.json`,
or wherever `app_config_dir` resolves) to see it again.

- [ ] **Step 4: Commit**

```bash
git add crates/app/src/tray.rs crates/app/src/lib.rs
git commit -m "feat(app): say once that the app is still running"
```

---

### Task 9: `ui` — the session document, read and written

**Files:**
- Create: `ui/src/lib/window/sessionDocument.ts`
- Create: `ui/src/lib/window/sessionDocument.test.ts`

**Interfaces:**
- Consumes: `TabSeed` from `@/stores/tabModel`, `RouteName` and `TabLocation` from
  `@/router/routeTable`.
- Produces:
  `readSession(raw: string | null): { tabs: TabSeed[]; activeIndex: number } | null` and
  `writeSession(session: { tabs: TabSeed[]; activeIndex: number }): string`.

- [ ] **Step 1: Write the failing tests**

`ui/src/lib/window/sessionDocument.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { RouteName } from '@/router/routeTable'
import { readSession, writeSession } from './sessionDocument'

const tab = (name: RouteName) => ({ entries: [{ name }], index: 0 })

describe('the session document', () => {
  it('reads back exactly what it wrote', () => {
    const session = {
      tabs: [tab(RouteName.Goals), tab(RouteName.Wiki)],
      activeIndex: 1,
    }
    expect(readSession(writeSession(session))).toEqual(session)
  })

  it('is nothing at all when there is nothing stored', () => {
    expect(readSession(null)).toBeNull()
  })

  it('is nothing at all when the document is not JSON', () => {
    expect(readSession('{ not json')).toBeNull()
  })

  it('is nothing at all when the version is one it does not know', () => {
    // A document from a newer app. Landing tab, not a guess at what the fields mean.
    expect(readSession('{"version":99,"tabs":[],"activeIndex":0}')).toBeNull()
  })

  it('drops the one tab it cannot read and keeps the others', () => {
    const raw = JSON.stringify({
      version: 1,
      tabs: [
        tab(RouteName.Goals),
        { entries: [{ name: 'a-screen-that-was-renamed' }], index: 0 },
        tab(RouteName.Wiki),
      ],
      activeIndex: 2,
    })
    const read = readSession(raw)
    expect(read?.tabs).toEqual([tab(RouteName.Goals), tab(RouteName.Wiki)])
    // The active tab was the third; with one dropped before it, it is now the second.
    expect(read?.activeIndex).toBe(1)
  })

  it('answers an empty session when every tab dropped', () => {
    const raw = JSON.stringify({
      version: 1,
      tabs: [{ entries: [{ name: 'gone' }], index: 0 }],
      activeIndex: 0,
    })
    // Not null: the document was readable. Empty, which seedState turns into the landing tab.
    expect(readSession(raw)).toEqual({ tabs: [], activeIndex: 0 })
  })

  it('refuses a tab whose history index points outside its entries', () => {
    const raw = JSON.stringify({
      version: 1,
      tabs: [{ entries: [{ name: RouteName.Goals }], index: 7 }],
      activeIndex: 0,
    })
    expect(readSession(raw)).toEqual({ tabs: [], activeIndex: 0 })
  })
})
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `pnpm --filter ui test -- sessionDocument`
Expected: FAIL — cannot resolve `./sessionDocument`.

- [ ] **Step 3: Write the implementation**

`ui/src/lib/window/sessionDocument.ts`:

```ts
import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import type { TabSeed } from '@/stores/tabModel'

// The document's version. It is bumped when an older app could read the new shape and be
// wrong about it — never for a part it can simply ignore, which is why 3.7's sidebar width
// and table sizes will join as named keys without touching this number.
const Version = 1

export interface Session {
  tabs: TabSeed[]
  activeIndex: number
}

const routeNames: readonly string[] = Object.values(RouteName)

const isRouteName = (value: unknown): value is RouteName =>
  typeof value === 'string' && routeNames.includes(value)

// A location we can still open. The query is carried as it was written: a filter or a page
// that no longer resolves is the screen's business, and every screen already says so. What is
// checked here is only the one thing that decides whether the tab can exist at all.
const readLocation = (value: unknown): TabLocation | null => {
  if (typeof value !== 'object' || value === null) return null
  const { name, query } = value as { name?: unknown; query?: unknown }
  if (!isRouteName(name)) return null
  return query === undefined || query === null
    ? { name }
    : { name, query: query as TabLocation['query'] }
}

const readTab = (value: unknown): TabSeed | null => {
  if (typeof value !== 'object' || value === null) return null
  const { entries, index } = value as { entries?: unknown; index?: unknown }
  if (!Array.isArray(entries) || entries.length === 0) return null
  if (typeof index !== 'number' || index < 0 || index >= entries.length)
    return null
  const read = entries.map(readLocation)
  // A tab is its history: one entry we cannot open and the back button lies. All or nothing.
  if (read.some((entry) => entry === null)) return null
  return { entries: read as TabLocation[], index }
}

// What was stored, as far as it can be read. `null` means "nothing usable", which the caller
// turns into the landing tab. A single unreadable tab is dropped alone — eight tabs do not
// vanish because one screen was renamed — and the active index follows what is left.
export const readSession = (raw: string | null): Session | null => {
  if (raw === null) return null
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch {
    return null
  }
  if (typeof parsed !== 'object' || parsed === null) return null
  const { version, tabs, activeIndex } = parsed as {
    version?: unknown
    tabs?: unknown
    activeIndex?: unknown
  }
  if (version !== Version || !Array.isArray(tabs)) return null
  const wanted = typeof activeIndex === 'number' ? activeIndex : 0
  const kept: TabSeed[] = []
  let active = 0
  tabs.forEach((value, at) => {
    const tab = readTab(value)
    if (!tab) return
    if (at <= wanted) active = kept.length
    kept.push(tab)
  })
  return { tabs: kept, activeIndex: kept.length === 0 ? 0 : active }
}

export const writeSession = (session: Session): string =>
  JSON.stringify({
    version: Version,
    tabs: session.tabs,
    activeIndex: session.activeIndex,
  })
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `pnpm --filter ui test -- sessionDocument`
Expected: PASS, 7 tests.

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/window/sessionDocument.ts ui/src/lib/window/sessionDocument.test.ts
git commit -m "feat(ui): the session document, read one tab at a time"
```

---

### Task 10: `ui` — the wrappers, the fixtures, and the types

**Files:**
- Modify: `ui/src/lib/constants/commands.ts`
- Modify: `ui/src/lib/ipc/types.ts`
- Create: `ui/src/lib/ipc/session.ts`
- Modify: `ui/src/lib/ipc/settings.ts`
- Modify: `ui/src/lib/ipc/fixtures/settings.ts`, `ui/src/lib/ipc/fixtures/index.ts`

**Interfaces:**
- Consumes: Task 5's four commands.
- Produces: `windowSession(): Promise<string | null>`,
  `setWindowSession(document: string | null): Promise<void>`,
  `setStayInBackground(stay: boolean): Promise<Settings>`,
  `setResumeTabs(resume: boolean): Promise<Settings>`, and `Settings` gaining the three fields.

- [ ] **Step 1: Mirror the wire types**

`ui/src/lib/ipc/types.ts`, the `Settings` interface gains — in the same order as the Rust
struct, which is how this file is read against it:

```ts
  stayInBackground: boolean
  resumeTabs: boolean
  backgroundNoticeShown: boolean
```

`ui/src/lib/constants/commands.ts` gains:

```ts
  SetStayInBackground: 'set_stay_in_background',
  SetResumeTabs: 'set_resume_tabs',
  WindowSession: 'window_session',
  SetWindowSession: 'set_window_session',
```

- [ ] **Step 2: Write the wrappers**

Append to `ui/src/lib/ipc/settings.ts`:

```ts
export const setStayInBackground = (stay: boolean): Promise<Settings> =>
  call(Command.SetStayInBackground, { stay })

export const setResumeTabs = (resume: boolean): Promise<Settings> =>
  call(Command.SetResumeTabs, { resume })
```

Create `ui/src/lib/ipc/session.ts`:

```ts
import { Command } from '../constants/commands'
import { call } from './transport'

// The session document, opaque on the way there and back: its shape lives in
// `lib/window/sessionDocument.ts`, which is the only thing that parses it.
export const windowSession = (): Promise<string | null> =>
  call(Command.WindowSession)

export const setWindowSession = (document: string | null): Promise<void> =>
  call(Command.SetWindowSession, { document })
```

- [ ] **Step 3: Teach the fixtures to answer**

`ui/src/lib/ipc/fixtures/settings.ts` — the module-level state gains the flags, so that
`pnpm ui:dev` can toggle the switches and see them hold for the page's life:

```ts
let stayInBackground = true
let resumeTabs = true
let session: string | null = null

export const settingsAnswer = (): Settings => ({
  activeProfileId: null,
  scale,
  stayInBackground,
  resumeTabs,
  backgroundNoticeShown: false,
})

export const setStayInBackgroundAnswer = (stay: boolean): Settings => {
  stayInBackground = stay
  return settingsAnswer()
}

// Off clears what was stored, as the backend does: one rule, not two.
export const setResumeTabsAnswer = (resume: boolean): Settings => {
  resumeTabs = resume
  if (!resume) session = null
  return settingsAnswer()
}

export const sessionAnswer = (): string | null => (resumeTabs ? session : null)

export const setSessionAnswer = (document: string | null): void => {
  if (resumeTabs) session = document
}
```

`resetSettingsFixture` sets all four back (`scale`, `stayInBackground`, `resumeTabs`,
`session`).

`ui/src/lib/ipc/fixtures/index.ts` — four handlers beside `Command.SetScale`:

```ts
  [Command.SetStayInBackground]: (args) =>
    setStayInBackgroundAnswer(Boolean(args?.stay)),
  [Command.SetResumeTabs]: (args) => setResumeTabsAnswer(Boolean(args?.resume)),
  [Command.WindowSession]: () => sessionAnswer(),
  [Command.SetWindowSession]: (args) => {
    setSessionAnswer((args?.document as string | null) ?? null)
    return undefined
  },
```

with the imports extended.

- [ ] **Step 4: Verify**

Run: `pnpm typecheck && pnpm --filter ui test`
Expected: green. A missing field on the `Settings` fixture is a type error, which is the point
of mirroring the struct by hand.

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/constants/commands.ts ui/src/lib/ipc/types.ts ui/src/lib/ipc/session.ts ui/src/lib/ipc/settings.ts ui/src/lib/ipc/fixtures/settings.ts ui/src/lib/ipc/fixtures/index.ts
git commit -m "feat(ui): wrappers and fixtures for the background settings and the session"
```

---

### Task 11: `ui` — the main window is born from its session

**Files:**
- Modify: `ui/src/stores/tabs.ts`
- Modify: `ui/src/lib/window/session.ts`
- Test: `ui/src/stores/tabs.test.ts` (if the file doesn't exist, create it; check first with
  `ls ui/src/stores`)

**Interfaces:**
- Consumes: `readSession` / `writeSession` (Task 9), `windowSession` / `setWindowSession`
  (Task 10), `tabSeed` and `seedState` from `@/stores/tabModel`.
- Produces: `useTabsStore().session` — a computed `{ tabs: TabSeed[]; activeIndex: number }`
  of the window's current tabs.

- [ ] **Step 1: Write the failing test**

In `ui/src/stores/tabs.test.ts`:

```ts
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { RouteName } from '@/router/routeTable'
import { useTabsStore } from './tabs'

describe('what a window would save of itself', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('is its tabs and which one is active', () => {
    const tabs = useTabsStore()
    tabs.seed(
      [
        { entries: [{ name: RouteName.Goals }], index: 0 },
        { entries: [{ name: RouteName.Wiki }], index: 0 },
      ],
      1,
    )
    expect(tabs.session).toEqual({
      tabs: [
        { entries: [{ name: RouteName.Goals }], index: 0 },
        { entries: [{ name: RouteName.Wiki }], index: 0 },
      ],
      activeIndex: 1,
    })
  })

  it('carries no tab id: an id is this window's, not the session's', () => {
    const tabs = useTabsStore()
    tabs.seed([{ entries: [{ name: RouteName.Goals }], index: 0 }], 0)
    expect(JSON.stringify(tabs.session)).not.toContain('"id"')
  })
})
```

- [ ] **Step 2: Run it to verify it fails**

Run: `pnpm --filter ui test -- tabs`
Expected: FAIL — `tabs.session` is undefined.

- [ ] **Step 3: Add the computed, and make `main` start pending**

In `ui/src/stores/tabs.ts`:

```ts
  // What this window would be restored from: the tabs without their ids, and which one is
  // active. The same shape a tear-off sends, which is why there is one seed mechanism and not
  // two.
  const session = computed(() => ({
    tabs: state.value.tabs.map(tabSeed),
    activeIndex: state.value.tabs.findIndex(
      (tab) => tab.id === state.value.activeId,
    ),
  }))
```

returned alongside the rest. `tabSeed` is already imported by this file.

**The `activeIndex` of a window with no tabs is `-1`** — `findIndex` on an empty array. That is
never written (Task 12 writes only what `readSession` could read back) and `seedState` clamps
on the way in, but say so in a comment rather than leaving the reader to find out.

Then the two lines that decide how `main` is born:

```ts
  // Every window now starts pending, `main` included: what it holds comes from its session,
  // the way a torn-off window's comes from the window that created it (`lib/window/session.ts`).
  // A bar empty for the length of one read beats a landing tab that appears and is replaced.
  const state = ref<TabsState>(empty)
  const pending = ref(true)
```

and the comment above them, which currently explains that `main` starts with its landing tab,
is rewritten to say this instead. `born` and its `firstState(...)` call go away; check with
`grep -n born ui/src/stores/tabs.ts` that nothing else used them.

- [ ] **Step 4: Run it to verify it passes**

Run: `pnpm --filter ui test -- tabs`
Expected: PASS.

- [ ] **Step 5: Seed the main window from its session**

In `ui/src/lib/window/session.ts`, the `onMounted` body's tail changes. Today every pending
window broadcasts `Ready` and falls back after `SeedTimeout`; now only a window that someone
may owe a seed to does that, and `main` reads instead:

```ts
    if (!tabs.pending) return
    // Nobody owes the first window a seed: what it holds is its own last session. Everything
    // that can go wrong — no session, the setting off, a document we can't read, a read that
    // never answers — ends at the same place, an empty seed, which `seedState` turns into the
    // landing tab.
    if (windowPort.isMain()) {
      timer = window.setTimeout(() => tabs.seed([], 0), SeedTimeout)
      try {
        const restored = readSession(await windowSession())
        forget()
        tabs.seed(restored?.tabs ?? [], restored?.activeIndex ?? 0)
      } catch {
        forget()
        tabs.seed([], 0)
      }
      return
    }
    timer = window.setTimeout(() => tabs.seed([], 0), SeedTimeout)
    await windowPort.broadcast({
      kind: WindowMessageKind.Ready,
      label: windowPort.label(),
    })
```

**`tabs.seed` twice is harmless but sloppy** — the timeout may have fired first, in which case
`pending` is already false and the second call would replace the bar under the user. Guard the
timeout's callback with `if (!tabs.pending) return` inside `seed`'s caller, or check
`tabs.pending` before the second `seed`. Pick one and say which in a comment.

- [ ] **Step 6: Check it by hand on the development server**

Run: `pnpm ui:dev`, open two or three tabs, reload the page.
Expected: the same tabs come back (the fixture holds the document for the page's life — a
reload of the *page* keeps it, a restart of the dev server does not).

- [ ] **Step 7: Commit**

```bash
git add ui/src/stores/tabs.ts ui/src/stores/tabs.test.ts ui/src/lib/window/session.ts
git commit -m "feat(ui): the first window opens on the tabs it had"
```

---

### Task 12: `ui` — and it writes them as they change

**Files:**
- Modify: `ui/src/lib/window/session.ts`

**Interfaces:**
- Consumes: `useTabsStore().session`, `writeSession`, `setWindowSession`.
- Produces: nothing new.

- [ ] **Step 1: Write it**

In `useWindowSession`, beside the listener it already mounts:

```ts
  // **Only `main`, and only as the tabs change.** Not on close: the webview is being torn down
  // at that moment and a write that races the teardown is a write that sometimes doesn't
  // happen. A torn-off window's tabs are not the session.
  //
  // The debounce is for the burst — opening a tab moves the active index twice — not for the
  // cost: one small row in SQLite.
  let saving: number | null = null
  const remember = (): void => {
    if (!windowPort.isMain()) return
    if (saving !== null) window.clearTimeout(saving)
    saving = window.setTimeout(() => {
      const { tabs: seeds, activeIndex } = tabs.session
      if (seeds.length === 0) return
      void setWindowSession(
        writeSession({ tabs: seeds, activeIndex: Math.max(0, activeIndex) }),
      ).catch(() => undefined)
    }, SaveDelay)
  }
```

with `const SaveDelay = 400` beside `SeedTimeout` and its own comment, a `watch(() =>
tabs.session, remember, { deep: true })` in `onMounted`, and the timer cleared in
`onBeforeUnmount` alongside `forget()`.

**A failed write is swallowed**: the tabs are on screen either way, and a dialog because a
session didn't save would be worse than the session not saving. The settings screen already has
the place where a failed write is *said* — that is the scale's `saveFailed`, and it is about a
setting the user just asked for, which this is not.

- [ ] **Step 2: Check it by hand**

Run: `pnpm ui:dev`. Open three tabs, switch to the second, reload.
Expected: three tabs, the second active.

- [ ] **Step 3: Verify nothing else moved**

Run: `pnpm --filter ui test && pnpm typecheck && pnpm scan`
Expected: green.

- [ ] **Step 4: Commit**

```bash
git add ui/src/lib/window/session.ts
git commit -m "feat(ui): the main window writes its session as its tabs change"
```

---

### Task 13: `ui` — the Background screen

**Files:**
- Modify: `ui/src/router/routeTable.ts`, `ui/src/router/routes.ts`
- Create: `ui/src/screens/BackgroundScreen.vue`
- Modify: `ui/src/stores/settings.ts`
- Modify: `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts`

**Interfaces:**
- Consumes: `setStayInBackground`, `setResumeTabs` (Task 10).
- Produces: `RouteName.Background` at `/settings/background`; the settings store gains
  `stayInBackground`, `resumeTabs`, `setStayInBackground`, `setResumeTabs`.

- [ ] **Step 1: Add the route**

`ui/src/router/routeTable.ts`: `Background: 'background'` in `RouteName`, then the compiler
names every record that must answer — `routePath` (`'/settings/background'`), `routeTitle`
(`'routes.background'`), `routeOrigin` (`TabOrigin.Settings`), `routeIcon`. Import
`MonitorDotIcon` from `@lucide/vue` for it, beside the other icons. `routeArrives` is
`Partial`, and this screen is real, so it gets no entry.

`ui/src/router/routes.ts`: `[RouteName.Background]: BackgroundScreen` in `screens`, with the
import.

- [ ] **Step 2: Add the messages**

`ui/src/i18n/messages/it.ts`:

```ts
    background: 'Background',
```

in `routes`, and a block beside `appearance`:

```ts
  background: {
    intro:
      "Cosa fa l'app quando chiudi l'ultima finestra, e cosa ritrovi quando la riapri.",
    stayTitle: 'Resta aperta in background',
    stayHint:
      "Chiudendo l'ultima finestra l'app resta nell'area di notifica, accanto all'orologio: un clic sull'icona la riapre. Spenta, chiudere l'ultima finestra chiude l'app.",
    resumeTitle: 'Riapri le tab dell'ultima sessione',
    resumeHint:
      'La prima finestra si riapre sulle tab che aveva. Spenta, riparte dalla schermata iniziale e quello che era salvato viene cancellato.',
    saveFailedTitle: "L'impostazione non è stata salvata",
    saveFailed:
      "L'app si comporta già così, ma non siamo riusciti a scriverlo: al prossimo avvio torna com'era.",
  },
```

`en.ts` gets the same keys — `it.ts` is the schema, so a missing one is a type error:

```ts
  background: {
    intro:
      'What the app does when you close the last window, and what you find when you open it again.',
    stayTitle: 'Keep running in the background',
    stayHint:
      'Closing the last window leaves the app in the notification area, next to the clock: one click on the icon brings it back. Off, closing the last window closes the app.',
    resumeTitle: 'Reopen the last session's tabs',
    resumeHint:
      'The first window reopens on the tabs it had. Off, it starts on the landing screen and what was saved is deleted.',
    saveFailedTitle: 'The setting was not saved',
    saveFailed:
      'The app already behaves this way, but we could not write it down: it will be back as it was next time you start it.',
  },
```

(Watch the apostrophes in the TypeScript strings: `'Riapri le tab dell'ultima sessione'` does
not compile. Use `"…"` for those, as the files already do.)

- [ ] **Step 3: Extend the settings store**

`ui/src/stores/settings.ts` gains two refs, filled by `load()` from the same `settings()` call
it already makes, and two actions shaped like `setScale` — **applied optimistically, saved
after, and a failed write said rather than undone**, which is the store's existing rule:

```ts
  const stayInBackground = ref(true)
  const resumeTabs = ref(true)

  const setStayInBackground = async (stay: boolean): Promise<void> => {
    stayInBackground.value = stay
    saveFailed.value = false
    saveError.value = null
    try {
      stayInBackground.value = (await saveStayInBackground(stay)).stayInBackground
    } catch (e) {
      saveFailed.value = true
      saveError.value = isIpcError(e) ? e : null
    }
  }
```

and the same for `resumeTabs`. `load()` sets both from the answer; the `catch` in `load` leaves
them at their defaults, which are the backend's defaults.

- [ ] **Step 4: Write the screen**

`ui/src/screens/BackgroundScreen.vue`, modelled on `AppearanceScreen.vue` — same
`ScreenHeader`, same `Alert` on `settings.saveFailed`, and two `Field`s each holding a `Switch`
from `@/components/ui/switch` with its title and hint. No `<style>`, no hardcoded sizes, every
visible string through `t(…)`.

- [ ] **Step 5: Verify**

Run: `pnpm typecheck && pnpm lint && pnpm scan && pnpm --filter ui test`
Expected: green. `pnpm scan` is the one that catches a visible string that isn't a message key.

Run `pnpm ui:dev`, go to `/settings/background`: both switches, both hints, toggling holds
across a page reload (the fixture), and turning "reopen tabs" off and reloading lands on the
landing tab.

- [ ] **Step 6: Commit**

```bash
git add ui/src/router/routeTable.ts ui/src/router/routes.ts ui/src/screens/BackgroundScreen.vue ui/src/stores/settings.ts ui/src/i18n/messages/it.ts ui/src/i18n/messages/en.ts
git commit -m "feat(ui): the Background screen, two switches"
```

---

### Task 14: The whole thing, on the machine

**Files:**
- Modify: `docs/STATUS.md`
- Modify: `docs/BACKLOG.md` (B6's entry, and 3.7's mention of the session document)

- [ ] **Step 1: Run the full check**

Run: `pnpm check`
Expected: every command green. Read the skip lines — `cargo test --workspace -- --nocapture`
is what shows them, and `scripts/check` counts them: a suite that passes with dozens of skips
has not told you what it ran on.

- [ ] **Step 2: Run the seven checks of Task 7, on a built app**

Run: `pnpm build`, then start the built executable (`target/release/isaac-dome.exe`).
This is the only configuration where the notification of Task 8 has a registered
AppUserModelID, so it is the one that answers whether the toast appears at all.

Go through all seven, plus:

8. With `stayInBackground` off (toggle it in the screen), close the last window: the process
   ends.
9. Turn it back on, close the last window, click the icon, check the tabs came back.

- [ ] **Step 3: Write down what happened**

`docs/STATUS.md`: a session-log entry for 2026-09-13 saying what landed, and the 3.7 line
noting that the session document and its two settings landed early, with this feature, leaving
3.7 the sidebar's width and the tables' sizes (B27). A checkbox is ticked **only for work that
is committed** — if one of the nine checks failed and its fix isn't in, it is not ticked.

`docs/BACKLOG.md`: B6 gains a note saying which half is closed (the flag, the document, the
restore of tabs) and which is not (tab limits, how a restored tab states a missing target).

- [ ] **Step 4: Commit**

```bash
git add docs/STATUS.md docs/BACKLOG.md
git commit -m "docs: the app that outlives its windows, and what it leaves to 3.7"
```

---

## Self-Review

**Spec coverage.** §1 nothing polls — Tasks 6–8 add no timer, and the plan adds no watcher.
§2 the exit — Task 6, with the notice in Task 8. §3 one recipe — Task 6, plus the
`create: false` assertion. §4 the tray — Tasks 1, 2, 7. §5 one instance — Task 7. §6 the
settings and the screen — Tasks 3, 5, 10, 13. §7 the notice — Task 8. §8 the session — Tasks 3
(cap), 4 (migration), 5 (commands), 9 (document), 11 (read), 12 (write). §9 the language —
Tasks 2 and 7. §10's test list — each item is a step above; its hand-checked list is Task 7
Step 5 and Task 14 Step 2. §11 out of scope — nothing in this plan starts with Windows or
watches a file.

**Type consistency.** `tray_action` / `TrayAction` (Tasks 1, 6, 7); `tray_text` / `TrayText` /
`tray_locale` (Tasks 2, 7); `with_stay_in_background` / `with_resume_tabs` /
`with_notice_shown` (Tasks 3, 5, 8); `session()` / `set_session()` (Tasks 4, 5);
`readSession` / `writeSession` / `Session` (Tasks 9, 11, 12); `windowSession` /
`setWindowSession` (Tasks 10, 11, 12); `useTabsStore().session` (Tasks 11, 12). The wire names
`stayInBackground`, `resumeTabs`, `backgroundNoticeShown` appear in Tasks 3, 10, 13 with the
same spelling.

**Closed while reviewing.** Three things this plan first guessed at and then read:
`IpcError` has no variant for a refused write past a cap, so Task 5 adds `SessionTooLarge` and
Task 10 mirrors it; `store_error` takes its `StoreError` by value; `store::for_tests` has no
`in_memory`, so Task 4's tests open a real file in a `tempdir`, the shape `queue.rs` uses.

**Left deliberately loose.** Task 13 Step 4 describes the screen instead of giving its SFC:
`AppearanceScreen.vue` is the model, the primitives are in `ui/src/components/ui/`, and a
hand-written template here would be a fourth copy of conventions `pnpm scan` already enforces.
Everything with a return value worth checking is spelled out above.
