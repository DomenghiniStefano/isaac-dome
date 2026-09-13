# The app that outlives its windows

**Status:** design agreed in conversation on 2026-09-13. Five decisions were taken there —
the process stays alive for *fast reopening* and as the place a future reader of the game's
files will run (not for any work it does today); the X **really closes** the window rather
than hiding it; the tray icon is **always** present, not only when there are no windows; the
first close announces itself **once** through the tray; and the whole behaviour has an off
switch, **on by default**. Everything below follows from those five plus the facts in §0.

**Scope.** Closing the last window stops the process today. After this, it doesn't: the
window dies, the process stays, an icon in the notification area brings a window back, and
launching the executable a second time hands you the instance you already have instead of a
second one. Two settings, one migration, one new screen.

**Not** in scope: starting with Windows, and any background *work* — the `log-watch` module
of M4, the `.dat` watcher, dataset updates. This builds the place that work will run in; it
does not put anything in it. §11 lists them with the reason each is left out.

---

## 0. What was verified for this design, and how

Read on 2026-09-13 from the vendored sources in `~/.cargo/registry`, not from memory. The
repo builds against **tauri 2.11.5**.

1. **The event loop distinguishes "the user closed the last window" from "the code asked to
   exit".** `RunEvent::ExitRequested { code, api }` documents `code` as `None` "when the exit
   is requested by user interaction", `Some` "when requested programmatically via
   `AppHandle::exit`" (`tauri-2.11.5/src/app.rs:225-232`), and
   `ExitRequestApi::prevent_exit` is at `:90`. This is the whole of §2: no flag, no
   `AtomicBool` guarding a re-entrant quit — the variant already carries the distinction.
2. **A window declared in the config need not be created at startup.** `WindowConfig::create`
   (`tauri-utils-2.9.3/src/config.rs:1936`, default `true`) exists precisely to be paired
   with `WebviewWindowBuilder::from_config`
   (`tauri-2.11.5/src/webview/webview_window.rs:150`), and the field's own documentation
   shows that pairing. So the window's shape — 1280x800, `decorations: false`,
   `backgroundColor: #150e0d` — stays in `crates/app/tauri.conf.json`, where
   `ui/src/assets/background.test.ts` already reads it. **No constant moves into Rust**, and
   the third copy of the background colour that B18 is about is not created.
3. **The tray is a cargo feature of `tauri` itself, not a plugin.** `tray-icon = ["dep:tray-icon"]`
   (`tauri-2.11.5/Cargo.toml:129`); `tray-icon 0.24.2` is already in the local registry as a
   transitive dependency. The builder has what §4 needs: `show_menu_on_left_click`
   (`src/tray/mod.rs:319`), `on_tray_icon_event` (`:337`), `on_menu_event` (`:328`), `menu`
   (`:241`), `tooltip` (`:265`).
4. **The three new dependencies resolve.** `cargo add --dry-run`:
   `tauri-plugin-single-instance 2.4.4`, `tauri-plugin-notification 2.4.0`,
   `sys-locale 0.3.2`. None talks to the network, none wants a key.
5. **The capability file already names the windows this touches.**
   `crates/app/capabilities/default.json` lists `["main", "win-*", "tab-preview"]`, and the
   notice of §7 is sent from Rust, so no permission is added: permissions gate the
   JavaScript API, and the frontend gains none.

---

## 1. What runs when nothing is visible

The process, its event loop, the tray icon, and whatever the managed states are still
holding. **The webview does not**: closing the last window drops the WebView2 processes,
which are the expensive half of a running Tauri app. `CatalogState`, `ResourcesState` and
`StoreState` stay as they are — "opened once" is a rule of this repo, and dropping them at
the last close is exactly what would make reopening slow again. The honest figure for an app
sitting in the tray is therefore *the Rust side plus what the catalog and the archive index
hold*, not a number this document can pin; what it is not is a webview per window.

Nothing polls, nothing wakes on a timer, nothing touches the disk while no window is open.
The one thing the process does in that state is wait for a click.

---

## 2. Who prevents the exit

`crates/app/src/lib.rs` switches from `.run(generate_context!())` to `.build(...)` followed
by `.run(|app, event| …)`. One arm:

```rust
RunEvent::ExitRequested { code: None, api } if stay_in_background(app) => api.prevent_exit()
```

- `code: None` is the user closing the last window. `code: Some(_)` is `app.exit(0)` from the
  tray's **Quit**, and is never prevented — that is the escape hatch, and it works by not
  matching rather than by a flag someone has to remember to set.
- `stay_in_background(app)` reads the setting (§6). Off, the arm doesn't match and the app
  exits on the last close exactly as it does today, so the old behaviour stays reachable and
  can be checked by hand.

`RunEvent` is `#[non_exhaustive]`, so this match has a `_ => {}` arm. That is one of the few
places in the repo where a catch-all is not a violation of "exhaustiveness is mandatory": the
enum is not ours and cannot be closed. It is worth the comment saying so.

---

## 3. One window recipe, three callers

`crates/app/src/window.rs`, new, wiring only:

```rust
pub fn open_or_focus(app: &AppHandle) -> Result<(), tauri::Error>
```

It lists the existing windows, asks `ipc` what to do (§4), and either brings one forward
(`unminimize`, `show`, `set_focus` — a window can be any of hidden, minimized, or merely
behind) or creates the main one with
`WebviewWindowBuilder::from_config(app, &app.config().app.windows[0])`.

Three callers: `setup` at startup, the tray, and the second-instance callback. The config
entry gains `"label": "main"` (today it relies on the default) and `"create": false`.

**`ui/src/assets/background.test.ts` gains one assertion**: that `create` is `false`. If
someone puts it back, Tauri creates the window at startup *and* `setup` tries to create a
second one under the same label — a startup error rather than a silent second window, but the
test says why before anyone has to read the error.

---

## 4. The tray, and what a click means

Built in `setup`, always, whether or not there are windows. Icon:
`app.default_window_icon()` (`crates/app/icons/` already ships them). Tooltip: the product
name, and after the first close the sentence of §7.

- **Left click** (`TrayIconEvent::Click` with `MouseButton::Left` and
  `MouseButtonState::Up`) calls `open_or_focus`. `show_menu_on_left_click(false)`, so the
  left button is the shortcut and the right one is the menu.
- **Right click** opens a two-item menu: **Open**, which is the same call, and **Quit**,
  which is `app.exit(0)`.

**The choice of window is a pure function**, in `crates/ipc/src/tray.rs`, because it is worth
checking and the `app` crate is not tested:

```rust
pub enum TrayAction { Focus { label: String }, CreateMain }
pub fn tray_action(labels: &[String]) -> TrayAction
```

Rules, each a test:

- No windows → `CreateMain`.
- `main` present → `Focus { "main" }`, whatever else is open.
- Only torn-off windows → `Focus` the **oldest**, which is the smallest label: `win-<ms in
  base 36>` is fixed-width over any plausible span of dates, so lexicographic order is
  chronological order. The comment says that, because it stops being true in 2059.
- **`tab-preview` is never chosen.** It is the drag's preview window, it carries no tab, and
  focusing it would hand the user a ghost. It is filtered before anything else, and the test
  for "only the preview is open" expects `CreateMain`.

---

## 5. One instance

`tauri-plugin-single-instance`, registered **first** — the plugin's own requirement — with a
callback that calls `open_or_focus` and ignores the arguments: this app has no command line.
Double-clicking the executable while it sits in the tray is then the same gesture as clicking
the tray icon.

---

## 6. Two settings, and where they live

`ipc::Settings` gains three fields. The struct already carries `#[serde(default)]`, so a
settings file written before this change reads without a migration:

| field | default | what it does |
|---|---|---|
| `stay_in_background` | `true` | §2. Off: the last close exits, as today. |
| `resume_tabs` | `true` | §8. Off: a reborn window lands on its landing tab. |
| `background_notice_shown` | `false` | §7. Written by Rust, never shown. |

Two commands shaped like the existing `set_scale`, and `with_*` constructors beside
`with_scale` so a write never forgets the other fields — the reason that method exists.

`settings.json` keeps **preferences**; it does not gain the session. That is §8.

**The screen**: a new route `background` at `/settings/background`, beside `profile` and
`appearance`, with the two switches (`components/ui/switch`, `components/ui/field`) and a
line of prose each. `RouteName`, `routePath`, `routeTitle`, `routeOrigin`, `routeIcon` and
`routeArrives` are exhaustive records, so adding the variant makes the compiler name every
place that has to answer — which is the point of that table.

---

## 7. Telling the user once

The first time §2 prevents an exit and `background_notice_shown` is `false`, the app sends
one notification through `tauri-plugin-notification` — *IsaacDome keeps running; click the
icon to bring it back* — and writes the flag.

**This is the part most likely not to work, and it degrades rather than blocking.** A Windows
toast wants a registered AppUserModelID, which an installed build has (the installer writes a
Start Menu shortcut) and `pnpm dev` may not. So: the notification is sent, its failure is
swallowed, and the tray icon's tooltip carries the same sentence from then on. The flag is
written whether or not the toast appeared — a notice that failed twice is not worth a third
attempt, and the icon is visible either way.

---

## 8. The tabs, across the death of their window

**A document, not a table** — the shape `store`'s migration 2 already uses for the plan
queue, for the same reason: the order is the position in the array.

```sql
-- migration 3
CREATE TABLE window_session (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  document TEXT NOT NULL
);
```

**This is part of 3.7, landing early.** B6 declares the fork and picks the same two sides
this does — the flag in `settings.json` "next to the other global preferences", the tabs in "a
`store` migration", because `isaacdome.db` "has a versioned schema for exactly this". So this
section does not invent a home; it opens the one 3.7 had already chosen, and 3.7 keeps the
rest: the sidebar's width and every table's dragged size (B27), which are deferred to "the
sub-project that saves the session". The document is therefore **an object with a version and
named parts**, not a bare array, so those parts join without a migration:

```json
{ "version": 1, "tabs": [ … ], "activeIndex": 0 }
```

**What the tabs part is.** The seed the tear-off already sends between windows —
`TabSeed[]` and an index, the payload of `WindowMessageKind.Seed` — serialized. Its shape is
the frontend's (`TabLocation` is a route name and a query; mirroring it into Rust would be a
second router to keep in step), so **Rust carries the whole document as an opaque string** and
the frontend is the only thing that parses it. The boundary rule it has to answer to is "no
paths, no offsets, no raw bytes", and a JSON document the frontend wrote is none of those.

Three things keep that honest:

- **A cap.** `ipc::MAX_SESSION_BYTES = 64 * 1024`, checked by a pure function on the way in; a
  longer document is refused, not truncated. Nothing should ever approach it, and a database
  that grows without a bound is how you find out that something did.
- **A parse that can fail, one tab at a time.** The frontend's `readSession(document)` answers
  `null` for what it cannot read at all — malformed JSON, a `version` it doesn't know, a
  missing `tabs` — and `null` means the landing tab. A **single** tab it can't read, because
  its route name no longer exists, is dropped **alone**: the other tabs are not punished for
  it, and a session of eight tabs does not vanish because one screen was renamed. A document
  whose tabs all drop ends up empty, which `seedState` already turns into the landing tab.
  Note the limit of that leniency, which B6 names: a tab pointing at an item or a page that no
  longer exists is *not* this case and is not dropped — it opens and its screen states the
  gap, as every screen already does for a target it can't resolve.
- **One writer.** Only the window labelled `main` writes, and only when `resume_tabs` is on. A
  torn-off window's tabs are not the session.

**When it is written**: on every change of the main window's tabs, debounced. **Not on close**
— the webview is being torn down at that moment, and a write that races the teardown is a
write that sometimes doesn't happen. Turning `resume_tabs` off clears the row once and stops
writing: the app should not keep a record the user has just said they don't want.

**When it is read**: by a newborn `main`, which nobody owes a seed to. Today `main` is the one
window that starts *not* pending — it is born holding its landing tab — so this is the one
place the mechanism has to change: `main` starts pending too, and `useWindowSession` seeds it
from the session instead of from a sibling window. Nothing else moves. `seedState` already
turns an empty seed into the landing tab, so "no session", "the setting is off" and "the
document was unreadable" all arrive at today's behaviour through code that already exists, and
the `SeedTimeout` deadline covers a read that never answers. Windows born from a tear-off are
untouched: they are owed a seed and never reach this path.

The cost of that change is a bar that is empty for the length of one SQLite read instead of
painting the landing tab immediately. The alternative — paint the landing tab, then swap it for
the session — shows the user a tab appearing and being replaced, which is worse than a few
milliseconds of nothing.

Two commands: `window_session() -> Option<String>` and `set_window_session(Option<String>)`,
where `None` means "clear". Both `Result<_, IpcError>`; an unreadable store is already a
diagnostic in this repo, and a session that can't be read is a landing tab, never an error
dialog.

---

## 9. The language of a menu that exists before any window

The tray menu is built in `setup`, when no webview exists and vue-i18n is not running. Two
strings, plus the sentence in §7.

`crates/ipc/src/tray.rs` carries them for the two locales the app speaks, and `sys-locale`
reads the system's tag. The mapping is the frontend's rule, in Rust, tested on the same cases
(`it-IT` → It, `it` → It, `en-GB` → En, `fr` → En, `""` → En): primary subtag, English as the
fallback. `ui/src/i18n/locale.ts` and this function are two implementations of one rule, which
is a duplication worth naming here — the alternative was a tray that speaks English until the
first window opens.

---

## 10. What is tested, and what is looked at

Pure, and therefore tested first:

- `ipc::tray_action` — the four rules of §4, `tab-preview` included.
- `ipc`'s tray labels and locale mapping — the five tags of §9.
- `ipc`'s session cap — at the limit, over it.
- `store` migration 3 — `SCHEMA_VERSION` becomes 3, a round trip, the single-row `CHECK`, and
  a file at version 2 that migrates without losing its goals or its queue.
- `ui`: `readSession` on a good document, on malformed JSON, on an unknown `version`, on one
  tab with an unknown route name among three good ones, and on a document whose tabs all drop;
  `writeSession` round-tripping what `readSession` reads.
- `ui/src/assets/background.test.ts` — plus `create === false`.

Wiring, and therefore **not** tested: `open_or_focus`, the tray builder, the `RunEvent` arm,
the single-instance callback. They are looked at instead, by hand, once — and the list is
short enough to live in the plan rather than in someone's head:

1. Close the last window: the process is still in Task Manager, the icon is in the tray.
2. Click the icon: a window comes back with the tabs it had.
3. Launch the executable again: no second process, the window you had comes forward.
4. Tray → Quit: the process is gone.
5. Setting off, close the last window: the process is gone.
6. Tear a tab off, close the main window, click the icon: the torn-off window comes forward,
   no new window is born.

---

## 11. Out of scope, and why

- **Starting with Windows.** A different decision — it puts the app in someone's login — and
  it belongs to the same conversation as M4's watcher, which is the only thing that would
  justify it.
- **Any background work.** `log-watch` (M4) needs the app running while the game runs, which
  is what this builds; it does not follow that it ships with it. A watcher added here would be
  a watcher nobody asked to be running in §1's "nothing polls".
- **Restoring several windows.** One window comes back, with the session of the main one.
  Position and size per window, and the whole arrangement, were considered and dropped: the
  cost is in the part that has to be right on a machine with two screens, one of them
  unplugged since.
- **Hiding instead of closing.** Decided against in conversation: instant reopening is not
  worth a webview kept in memory all day.
