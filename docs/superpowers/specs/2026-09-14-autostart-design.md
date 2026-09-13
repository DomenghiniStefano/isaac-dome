# Starting with Windows

**Status:** design agreed in conversation on 2026-09-14. Three decisions were taken there —
the **registry is the only source of truth** and `settings.json` gains nothing; a login launch
is **silent**, tray and archive without a window; and the switch is **inert in development
builds**, because the alternative is a login entry pointing at `target\debug\app.exe`.
Everything below follows from those three plus the facts in §0.

**Scope.** One switch on the Background screen, one dependency, one argument, two commands, one
pure function. No migration, no new screen, no change to the shape of `settings.json`.

This is §11 of `2026-09-13-background-and-tray-design.md` coming due: *"Starting with Windows. A
different decision — it puts the app in someone's login — and it belongs to the same conversation
as M4's watcher, which is the only thing that would justify it."* The watcher landed on
`feature/log-watch`; this is that conversation.

---

## 0. What was verified for this design, and how

Read on 2026-09-14 from the vendored sources in `~/.cargo/registry`, not from memory.

1. **The plugin resolves.** `cargo add --dry-run --package app tauri-plugin-autostart` →
   **2.5.1**. Its one meaningful dependency is `auto-launch 0.5`, which on Windows pulls
   `winreg` — a crate `discovery` already depends on. Nothing talks to the network, nothing
   wants a key.
2. **What is written, and where.** `auto-launch-0.5.0/src/windows.rs` names two keys at `:6-11`:
   `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` and
   `…\Explorer\StartupApproved\Run`. `enable()` (`:37-57`) sets `Run\<app_name>` to
   `format!("{} {}", app_path, args.join(" "))` — **the path is not quoted** (`:42`) — and stamps
   the second key with twelve bytes beginning `02 00` (`:9-11`, `:46-54`).
3. **`disable()` fails when the value isn't there.** `:65-70` is `delete_value`, unguarded.
   Turning the switch off twice returns an error the second time, and that error means nothing.
4. **`is_enabled()` reads the Task Manager's own switch.** `:73-83` is
   `al_enabled && task_manager_enabled.unwrap_or(true)`, where the second half decodes the
   `StartupApproved` blob (`:85-101`). **This is the whole of §2**: the place a Windows user turns
   these things off is the Startup tab, and the plugin can see them do it.
5. **The plugin's error is a bare string, and it can carry the exe path.**
   `tauri-plugin-autostart-2.5.1/src/lib.rs:32-47` — `Error::Anyhow(String)` built from
   `e.to_string()` (`:52-71`), serialized with `serialize_str` (`:40-47`). Its own three commands
   (`enable`, `disable`, `is_enabled`, `:86-99`) therefore hand a string to the frontend, which is
   what this repo's IPC rules forbid twice over: a path crosses, and nothing is translatable.
   **We do not register their permissions and the frontend never calls them** (§3).
6. **The registry value's name is the app's name.** `builder.set_app_name(…)` defaults to
   `app.package_info().name` (`:178-182`), which is what Task Manager shows in its Startup list.
   So the plugin is built through `Builder` with an explicit `.app_name(…)`, not through `init()`.
7. **The arguments are fixed when the plugin is built, not when the switch is flipped.**
   `builder.set_args(&self.args)` (`:184`). The string that ends up in the Run value is decided in
   `crates/app/src/lib.rs`; §4 is about what reads it back.
8. **`app.autolaunch()` panics when the plugin isn't registered.** `ManagerExt::autolaunch`
   (`:79-84`) is `self.state::<AutoLaunchManager>()`, and `Manager::state` panics on an unmanaged
   type. In a development build §6 does not register the plugin, so **the commands use
   `try_state`**, never that helper. "Never `panic!` outside tests" is a repo rule, and this is the
   line that would have broken it.

---

## 1. What this buys, precisely

`log.txt` is rewritten at every launch of the game. `start_archive` (`crates/app/src/lib.rs`)
already does the right thing when the app starts late: it backfills `online_logs\sessions\`,
whose folders are permanent, then reads the **whole** current `log.txt` before watching it. So a
run played this afternoon is still there if the app opens before the game is launched again.

What is lost without this feature is narrow and real: **a game launch followed by another game
launch, with the app never having run in between.** Play, quit, play again, open IsaacDome — the
first session is gone, and nothing can bring it back. The switch closes exactly that hole, and
the prose under it says that rather than promising "no run is ever lost".

What it does not do: it does not start the game, it does not watch anything while nothing moves
(the watch is `notify` on a folder), and it does not survive **Quit** from the tray — which is
correct, because Quit is the user saying stop.

---

## 2. The truth is the registry

`ipc::Settings` gains **no field**, `settings.json` keeps its shape, and there is no migration.

A mirrored boolean would be a second source of truth that goes stale the first time someone uses
the Startup tab (§0.4), an antivirus strips the value, or a profile is copied to another machine
where the exe path is wrong. The switch would then show "on" for a login that never happens — the
failure this repo keeps writing rules against: a plausible wrong answer with nothing to
contradict it.

Consequences, each a decision and not an accident:

- The switch's position is **read when the Background screen mounts**, not once at startup: the
  app sits in the tray for days, and the Startup tab can have changed underneath it.
- A write is followed by a **read-back**, and the command answers what it read, never what it was
  asked (§3).
- §0.3's error — `disable()` on a value that isn't there — stops being a case anyone has to
  handle: the read-back says `false`, which is what was wanted.

---

## 3. Two commands, and a view that can say "not here"

```rust
// crates/ipc/src/autostart.rs
pub struct AutostartView { pub enabled: bool, pub available: bool }

pub enum AutostartReason { NotSupported, RegistryUnwritable, PathUnknown }
```

```rust
// crates/app/src/commands/profile.rs, beside the other settings commands
autostart(app)                -> Result<AutostartView, IpcError>
set_autostart(app, on: bool)  -> Result<AutostartView, IpcError>
```

- **`available` is a diagnostic, not an error.** "This build has no autostart" is an expected
  case — a development build (§6), and the platforms the repo compiles for but does not ship —
  and expected cases travel in the payload. `Err` is kept for the command that cannot answer at
  all: the registry read itself failed.
- **`set_autostart` writes, then reads, then answers the read.** A write a policy silently undid
  reports as off. The only `Err` it produces is
  `IpcError::AutostartNotWritable { reason: AutostartReason }`, raised when the read-back
  disagrees with what was asked — never when the plugin merely returned an error the registry
  contradicts.
- **The plugin's string never crosses.** `AutostartReason` is ours, tagged
  (`#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]`) like
  `SettingsReason` and `StoreReason` beside it in `crates/ipc/src/reasons.rs`. `PathUnknown` is
  `current_exe()` having failed at setup; `RegistryUnwritable` covers both a refused write and a
  read-back that disagrees; `NotSupported` is the manager not being there at all.
- Both commands reach the manager with `app.try_state::<AutoLaunchManager>()` (§0.8), and `None`
  is `available: false`.
- **`crates/app/capabilities/default.json` gains nothing.** Permissions gate the JavaScript API,
  and the frontend gains none: it calls our two commands, not the plugin's three. Same reasoning
  as §0.5 of the background design.
- `crates/ipc/src/contract.rs` gains `decl::<AutostartView>` and `decl::<AutostartReason>`, and
  `pnpm ipc:types` regenerates `ui/src/lib/ipc/types.ts`. **That file is never edited by hand.**

---

## 4. The silent launch: one argument, one pure function

The plugin is built with one argument:

```rust
tauri_plugin_autostart::Builder::new()
    .app_name(ipc::AUTOSTART_ENTRY)   // what Task Manager's Startup list shows
    .arg(ipc::SILENT_ARG)             // "--silent"
    .build()
```

**That string lives in the registry of everyone who ever turned the switch on.** Renaming the
constant does not rename what is already written, so those users would get a window at every login
and no error anywhere. The constant's doc comment says so; it is the reason it is a constant in
`ipc` and not a literal in two files.

The decision it feeds is a pure function, because it is worth checking and `app` is not tested:

```rust
pub enum LaunchIntent { Window, Silent }
pub fn launch_intent(args: &[String]) -> LaunchIntent
```

Rules, each one a test:

- No arguments → `Window`. The caller drops `argv[0]`; this function sees what follows.
- `--silent` among them → `Silent`.
- **An argument we don't know → `Window`**, alone or standing next to the one we do know. The only
  way to see this app is a window, and a launch that shows nothing because of a typo is a launch
  that looks like a crash.

`setup` becomes three lines with one condition:

```rust
tray::build(app.handle());
if matches!(ipc::launch_intent(&args_after_exe()), LaunchIntent::Window) {
    window::open_or_focus(app.handle());
}
start_archive(app.handle().clone());
```

The tray is built **always** — it is the way back in, and a silent launch with no tray is a
process nobody can reach. The archive starts **always** — it is the feature.

**The single-instance callback keeps ignoring its arguments**, and its comment changes rather than
disappearing. Today it reads *"this app has no command line, so the arguments are nothing to act
on"*; the first half stops being true here, and the second half becomes the point: a second launch
is a human asking for the app, so it opens a window whatever it was handed.

**`stay_in_background` off and this switch on is a legal combination, and it is worth saying out
loud.** At login you get a tray and no window; the first window you open and close ends the
process, and the watching stops with it. The two switches are **not coupled in code** — one switch
silently moving another is not something this app does — so the Background screen states it in a
line under the switch when the other one is off.

---

## 5. The switch

A third `Field` on `ui/src/screens/BackgroundScreen.vue`, **first** of the three: it is about when
the app starts, and the other two are about what happens afterwards.

Keys under `background.` in both locales: `startTitle`, `startHint`, `startDev`,
`startWithoutBackground`, `startFailedTitle`, `startFailed`. `startHint` says what §1 says — that
IsaacDome has to be running while the game runs, and that a session played before it opened is
recoverable only until the game is launched again.

`useSettingsStore` gains `autostart`, `autostartAvailable`, `refreshAutostart()` and
`setAutostart()`. The screen calls `refreshAutostart()` on mount (§2).

**This switch moves after the answer, not before** — the opposite of the three settings beside it,
and the comment in the store says why. Those change what the app is doing *now*, so the honest
thing is to do it and report a failed write; this one changes what happens at the next login,
there is nothing to already be doing, and the only honest position of the switch is the one the
registry reported back.

---

## 6. Development builds

The plugin is registered under `#[cfg(not(debug_assertions))]`. In a `pnpm dev` run there is no
`AutoLaunchManager`, `try_state` answers `None`, the commands answer `available: false`, and the
switch is disabled with `startDev` under it.

The reason is concrete: `current_exe()` in development is `target\debug\app.exe` (`:186,189`), and
a switch flipped once while testing leaves that path in someone's login — surviving `cargo clean`,
pointing at nothing, and failing silently at every boot. The developer of this app is also its
first user.

---

## 7. What is tested, and what is looked at

Pure, and therefore tested first:

- `ipc::launch_intent` — the three rules of §4, including an unknown argument beside `--silent`.
- `AutostartView`'s JSON shape (camelCase) and `AutostartReason`'s tagging, in
  `crates/ipc/tests/`, beside the contract tests already there.
- `ui`: the store takes the value the command answered — including **asked on, answered off** —
  and `available: false` disables the switch.

Wiring, and therefore **not** tested. Looked at by hand, once, on an **installed** build; the list
belongs in the plan:

1. Switch on, then
   `reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v IsaacDome`. **Read the
   value; do not reason about it.** The path is written unquoted (§0.2), so an install under
   `C:\Program Files\…` is the case that has to be *seen* starting, not argued about.
2. Log out and back in: no window, the icon in the tray, and a run played immediately after is in
   the archive.
3. Task Manager → Startup apps → disable IsaacDome. Open the Background screen: the switch is off.
4. Switch off, then off again. No error either time (§0.3, overruled by the read-back).
5. The same in a `pnpm dev` run: the switch is disabled and the registry is untouched.

---

## 8. Known gaps, and what is out of scope

- **Uninstalling leaves the Run value behind.** Nothing tells a running app it is being
  uninstalled, and the NSIS uninstaller knows nothing about the key. The fix is an uninstall hook
  (`bundle.windows.nsis.installerHooks`) deleting the value from **both** keys of §0.2 — and it is
  not in this task, because the repo has no installer configuration yet: `tauri.conf.json` says
  `"targets": "all"` and nothing more. It is carried as a closing criterion of B41 so it is not
  lost the day that configuration is written.
- **The unquoted path is upstream's.** If check 1 fails under `C:\Program Files\…`, the answer is
  not a patch here. It is either a per-user install under `%LOCALAPPDATA%` — which is what an app
  writing to `HKCU` should have anyway — or three lines of `winreg`, a crate already in the tree,
  in place of the plugin. Decide with the measurement in hand, not before it.
- **A Run entry, not a scheduled task.** It starts at login, as the user, which is the right scope
  for an app that reads that user's own save files. Nothing here asks for a service, or for
  anything that runs before someone logs in.
- **macOS and Linux compile and are not shipped.** `MacosLauncher::LaunchAgent` is the default and
  is named for the signature's sake, not because anyone has run it. §7's checks are Windows-only
  and the plan says so.
- **No notice at the first silent launch.** The one-time tray notice of the background design
  covers "the app is still running"; a second one at login would be the same sentence on a day the
  user did nothing. The tray icon is the notice.
