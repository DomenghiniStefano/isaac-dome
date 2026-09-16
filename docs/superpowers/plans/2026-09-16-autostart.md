# Plan — starting with Windows (B41)

Spec: `docs/superpowers/specs/2026-09-14-autostart-design.md`, agreed 2026-09-14. Branch
`feature/autostart`. The spec verified the plugin's sources line by line; this plan is the order
of the work, the tests that come first, and the three places it departs from the spec.

## Where this plan departs from the spec, and why

1. **`AutostartReason` is a bare camelCase string, not a tagged enum.** §3 says to tag it *"like
   `SettingsReason` and `StoreReason` beside it"* — but those two are tagged because a variant of
   each carries data (`Io { reason }`, `NewerSchema { found, supported }`). `AutostartReason` has
   none, and `CLAUDE.md` is explicit: a fieldless enum on the wire is a value, not a
   discriminator, with **zero exceptions in the repo** and *"a tagged unit enum showing up again
   is a bug, not an alternative style"*. The analogy in §3 is to where the type lives, not to why
   the tag is there.
2. **`available: bool` becomes `unavailable: Option<AutostartReason>`.** With a boolean, a
   development build and a registry that would not answer are the same answer with two different
   causes — and §3 wanted the reason to exist, it just gave it nowhere to travel. `available` is
   `unavailable === null` and the frontend reads it that way.
3. **`PathUnknown` does not exist**: nothing in our code calls `current_exe()` — the plugin does,
   internally, and its error is the bare string §0.5 forbids from crossing.
4. **The error carries a reason after all, and it is a second enum.** This was written as a
   fieldless `IpcError::AutostartNotWritable` — one write failure, nothing for a reason to add —
   and the owner said it looked like an enum to make. It is, and the case that settles it came
   from the plugin's own reading: `is_enabled()` is `value && approved` (§0.4), so an entry a
   user switched off in Task Manager's Startup tab reads as off **however well the value was
   written**. A write can therefore be *refused* or *accepted and ignored*, the app can tell them
   apart — the plugin says whether the write errored, the registry says whether it survived — and
   they send the user to two different places. `AutostartFailure { WriteRefused, WriteIgnored }`,
   a type of its own and not more variants on `AutostartReason`: the two sets never meet, one
   says why the switch cannot be *offered* and the other why it would not *move*, and a `switch`
   over either should carry no branch that cannot happen.
   **`WriteIgnored` is named for what was observed**, not for the Startup tab: the plugin does
   not expose the two halves separately, so naming the cause would be naming a cell from a guess
   — the message points at the Startup tab, the variant does not claim it.

## Tasks

1. **`ipc::launch_intent`** — `LaunchIntent { Window, Silent }` and the three rules of §4, tests
   first in `crates/ipc/tests/autostart.rs`. It does not cross the wire: `app` is its only caller.
2. **The two constants**, `AUTOSTART_ENTRY` and `SILENT_ARG`, with the doc comment §4 asks for:
   the argument lives in the registry of everyone who ever turned the switch on, so renaming the
   constant does not rename what is written.
3. **`AutostartView` and `AutostartReason`**, plus `IpcError::AutostartNotWritable`; `contract.rs`
   gains both declarations and `pnpm ipc:types` regenerates. A test on the JSON shape beside
   `crates/ipc/tests/contract_shapes.rs`: camelCase, and the reason a bare string.
4. **The dependency and the plugin**, `tauri-plugin-autostart 2.5.1`, registered under
   `#[cfg(not(debug_assertions))]` through `Builder` with `.app_name()` and `.arg()` (§0.6, §6).
5. **The two commands** in `crates/app/src/commands/profile.rs`, both through
   `try_state::<AutoLaunchManager>()` and never through `app.autolaunch()`, which panics when the
   plugin is not registered (§0.8). `set_autostart` writes, reads back, answers the read.
6. **`setup` and the single-instance comment** (§4): the tray always, the archive always, a window
   only when `launch_intent` says so. The callback's comment changes rather than disappearing.
7. **The frontend**: `ui/src/lib/ipc/autostart.ts`, the store's `autostart`,
   `autostartAvailable`, `refreshAutostart()` and `setAutostart()` — **this switch moves after the
   answer**, unlike the three beside it — the third `Field` on `BackgroundScreen.vue` placed
   **first**, and the six keys in both locales. Vitest on the store: asked on, answered off.

## What is looked at by hand, once, on an installed build

Straight from §7, and it goes to *"What only a window can say"* in `docs/STATUS.md` when the
branch lands. None of it can be a test: the registry, a login and a logout.

1. Switch on, then `reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v IsaacDome`.
   **Read the value, do not reason about it**: the path is written unquoted (§0.2), so an install
   under `C:\Program Files\…` is the case that has to be *seen* starting.
2. Log out and back in: no window, the icon in the tray, and a run played immediately after is in
   the archive.
3. Task Manager → Startup apps → disable IsaacDome. Open the Background screen: the switch is off.
4. Switch off, then off again. No error either time.
5. The same in a `pnpm dev` run: the switch is disabled, and the registry is untouched.
