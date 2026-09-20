# Updating the app itself

**Status:** design agreed in conversation on 2026-09-20. Four decisions were taken there — the
**official plugin with its signature**, never a hand-rolled download; **download in the
background and install on the user's word**, never an install nobody asked for; the releases
live on **`isaac-dome` made public**, because a private repository's assets need a token and the
project forbids keys; and the whole thing has **a screen of its own**, `/settings/updates`.
Everything below follows from those four plus the facts in §0.

**Scope.** One dependency, one plugin registered in release builds only, four commands, one
event, one field in `settings.json`, one route, one screen, one release script, one document.
No new crate, no store migration, no change to any existing command's shape.

This closes the half of `M5 — Public release` that nobody had written down: the app can be
packaged since 2026-09-16, and a packaged app with no way to update itself is a package that has
to be re-downloaded by hand for every fix.

---

## 0. What was verified for this design, and how

Read on 2026-09-20 from the crate's own source — `tauri-plugin-updater 2.12.0`, downloaded from
`static.crates.io` and extracted — and from `tauri-docs@v2`, not from memory. Line numbers are
this version's.

1. **The plugin resolves.** `cargo add --dry-run --package app tauri-plugin-updater` →
   **2.12.0**, default features `rustls-tls`, `system-proxy`, `zip`. It brings `reqwest`,
   `minisign-verify`, `semver` and `zip`; nothing wants a key, and the only host it ever talks to
   is the one in `endpoints`.
2. **`download` and `install` are separate calls.** `src/updater.rs:680` is
   `Update::download(on_chunk, on_download_finish) -> Result<Vec<u8>>`, which verifies the
   signature before returning the bytes; `:757` is `Update::install(bytes)`. The combined
   `download_and_install` at `:767` is exactly those two in a row. **This is what makes the agreed
   behaviour possible at all**: the bytes can sit verified in memory while the user keeps working,
   and the install happens when they say so.
3. **On Windows, `install` ends the process with `std::process::exit(0)`** (`:882`), documented at
   `:753-756` as *"This function exits the app after launching the updater installer
   successfully"*. No `RunEvent::Exit` is delivered and no destructor runs. **This is why
   `stay_in_background` cannot interfere**: `lib.rs:145` prevents an exit by intercepting
   `ExitRequested`, and this exit never raises one. Anything that must happen first goes in the
   `on_before_exit` hook (`:335`, called at `:849`).
4. **The installer restarts the app by itself**, and that is the default: `restart_after_install`
   is `true` at `:197`, which adds NSIS's `/R` (`config.rs:53-58`). So "Restart and install" is one
   click and the app comes back on its own.
5. **`installMode` has three values and `passive` is the default** (`config.rs:11-23`): `passive`
   → NSIS `/P`, a progress bar and no questions; `basicUi` → a wizard the user has to finish;
   `quiet` → `/S`, no feedback and no way to ask for elevation. We take the default and say so.
6. **`requireSignedVersion` exists and the documentation page does not mention it**
   (`config.rs:119-137`). Without it, the endpoint's JSON — fetched over TLS but **not itself
   signed** — can pair an inflated `version` with the `url` and `signature` of an older release
   and force a downgrade to a genuine, validly signed, outdated build. It is off by default
   because it rejects releases signed before the CLI started recording the version in the
   signature's trusted comment. **This project has never published a release**, so every release it
   will ever have carries one: we turn it on now, at a cost of exactly zero, and it can never be
   turned on this cheaply again.
7. **The plugin's `Error` serializes as a bare string** (`error.rs:110-118`,
   `serialize_str(self.to_string())`), and several variants carry a URL or a path —
   `Network(String)`, `Io`, `UrlParse`, `TargetNotFound`. It is `#[non_exhaustive]` (`:10`).
   **It must never cross the IPC**: §3 maps it onto an enum of ours, and the catch-all arm that
   costs is owed to `#[non_exhaustive]`, the same exemption `lib.rs:151` already takes for
   `RunEvent`.
8. **The endpoint must be `https`** (`config.rs:196-210`): a plain-`http` endpoint warns in
   development and is a hard `InsecureTransportProtocol` error in a release build.
9. **A static GitHub endpoint is the documented shape.** `plugin/updater.mdx` names
   `https://github.com/user/repo/releases/latest/download/latest.json` as an endpoint, and gives
   the manifest as `{version, notes, pub_date, platforms: {"windows-x86_64": {signature, url}}}`.
   `/releases/latest/` resolves to the newest **non-prerelease** release, which is what makes a
   draft or a pre-release safe to publish.
10. **The capability is not needed.** The `updater:default` permission the documentation asks for
    gates the plugin's **own** commands, called from JavaScript. Nothing here calls them: the
    frontend calls four commands of ours, exactly as `tauri-plugin-dialog` is already used from
    Rust only (`lib.rs:33-36`). `crates/app/capabilities/default.json` does not change.

---

## 1. What the owner does, once

Not code, and not something a session can do on the owner's behalf. It is listed here because the
feature does not exist until it is done, and because step 2 is irreversible.

1. **Generate the key pair**: `pnpm tauri signer generate -w $HOME/.tauri/isaacdome.key`, with a
   password. The private half **never enters the repository** and is not recoverable: lose it and
   every installation in the world stops being able to update, with no message, until somebody
   downloads an installer by hand. It needs a copy somewhere that is not this disk.
2. **Make `github.com/DomenghiniStefano/isaac-dome` public.** Today it is private, and a private
   release's assets need an `Authorization` header — a credential, which constraint 2 forbids in
   the binary and to the user alike. Making it public exposes the whole history, `docs/` included.
   Worth a look before the click: the repository has never held a key, `samples/` has always been
   ignored, and the signing key of step 1 must not be committed afterwards either.

## 2. What the owner does, per release

Four steps, no GitHub Action, by decision — the same decision that keeps `scripts/check` instead
of CI. They live in **`docs/release.md`**, written by this task.

1. Bump `version` in `crates/app/tauri.conf.json`. That file is the single place the version is
   written; `crates/app/Cargo.toml` keeps its own and Tauri ignores it when the config names one.
2. `$env:TAURI_SIGNING_PRIVATE_KEY` and `$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, then
   `pnpm build`. With `createUpdaterArtifacts: true` the NSIS setup comes out with a `.sig` beside
   it.
3. `pnpm release:manifest` — `scripts/release-manifest.mjs`, written by this task. It reads the
   version from `tauri.conf.json`, finds the setup and its `.sig` under
   `target/release/bundle/nsis/`, and writes `target/release/latest.json` with the download URL
   pointing at the tag it is about to be published under. It **refuses to write** when the version
   and the file names disagree, because a manifest that names an asset the release does not carry
   is an update that fails for everyone at once. Release notes come from a file passed as an
   argument; without one, `notes` is left out and the screen shows none.
4. Tag on `master`, create the release, upload the NSIS setup, the MSI and `latest.json`.

**The artifact that updates is the NSIS setup, not the MSI**: NSIS installs per-user and needs no
elevation, the MSI does. Both are still built and both are still published — one is what people
download the first time, the other is what the app fetches.

---

## 3. The phases, in `ipc`

No new crate. `ipc` is pure, has no I/O and is already the home of `Settings`, which is not a view
onto any domain crate and carries logic of its own (`snap_percent`). The phase machine lives
beside it in `crates/ipc/src/update.rs`; `app` holds it in state and never decides anything.

```rust
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum UpdatePhase {
    /// Nothing asked yet since this launch.
    Idle,
    Checking,
    /// A check answered, and this build is the newest there is.
    UpToDate,
    /// `percent` is `None` when the server sent no `Content-Length`: an indeterminate bar is
    /// the truth, and a bar inventing a number is not.
    Downloading { version: String, percent: Option<u8> },
    /// Verified, in memory, waiting for the word.
    Ready { version: String, notes: Option<String> },
    Failed { reason: UpdateFailure },
}
```

**There is no `Available` phase**, and that is a decision rather than an omission: a found update
is downloaded immediately on both paths — automatic and the button — so the state would never rest
anywhere a screen could draw it.

`rename_all_fields` is not decoration. `rename_all` on an enum renames the **variants**; without
the second attribute a two-word field would cross as `snake_case` and TypeScript would read
`undefined` with no error anywhere. The JSON shape is pinned by a test, as `CLAUDE.md` requires.

```rust
#[serde(rename_all = "camelCase")]
pub enum UpdateFailure {
    /// The endpoint could not be reached: no network, DNS, a proxy, GitHub down.
    Offline,
    /// Reached, and there is nothing to install: no manifest at that URL, or it carries no
    /// entry for this platform. The normal answer for a repository with no release yet.
    NotPublished,
    /// **Reached, answered, and refused.** The signature did not verify, or it was signed for a
    /// version other than the one announced. This is the one failure that is not bad luck, and
    /// it gets its own sentence on screen.
    Rejected,
    /// The bytes were good and the installer would not run.
    InstallFailed,
    /// Anything else. Owed to `#[non_exhaustive]` (§0.7), not to laziness.
    Unknown,
}
```

Fieldless, so a **bare camelCase string** on the wire with a TypeScript union mirroring it — the
repo rule with zero exceptions, and the same shape `AutostartReason` already has.

```rust
pub struct UpdateView {
    /// What is running, from `package_info().version`. The screen has to be able to answer
    /// "which one do I have" with no network at all.
    pub current_version: String,
    pub phase: UpdatePhase,
    /// `None` when updating can be offered. `Some` in a development build, and the switch and
    /// the button are both inert — the shape `AutostartView.unavailable` already uses, for the
    /// same reason: "off" and "impossible" are not the same answer.
    pub unavailable: Option<UpdateReason>,
}
```

`UpdateReason` has one variant today, `NotSupported`. It is an enum and not a `bool` for the
reason the autostart design recorded: a boolean makes a development build and a genuine failure
give the same answer with two different causes.

**What is pure and tested here**: the classifier from a plugin error class to `UpdateFailure`, the
transitions (which phase may follow which, and that a check during a download is a no-op), and the
serialized shape. **What is not**: everything that touches the network, which is `app`.

## 4. The commands, the state and the event

Four commands. Three new in `crates/app/src/commands/update.rs`, one beside the other settings
setters in `profile.rs`. The handler goes from 36 to 40.

| command | answers | notes |
|---|---|---|
| `update_status` | `UpdateView` | Pure read of the held state. No network, ever. |
| `check_update` | `UpdateView` | Checks, and downloads if there is something. The button. |
| `install_update` | `Result<(), IpcError>` | Only from `Ready`. On success it never returns: §0.3. |
| `set_auto_update` | `Settings` | Same shape as `set_stay_in_background`. |

`UpdaterState` in `tauri::State`: a `Mutex` over the phase, the `Update`, and the downloaded
bytes. **The bytes are held whole in memory** — some tens of megabytes — which is a deviation from
"no file is ever loaded whole" worth naming rather than hiding: the plugin's `download` returns a
`Vec<u8>` and there is no streaming form, the size is our own installer and not a stranger's file,
and writing it to a temporary file to read it straight back would add a path and a cleanup for
nothing. The real number goes in `docs/release.md` the first time a release is built.

**`UPDATE_CHANGED`**, a sixth event in `events.rs`, **with no payload**, like the five before it:
every window answers by calling `update_status`. Download progress is the one place where that
convention costs something, so the emit is **throttled to whole percentage points** — at most a
hundred over a download, each one a cheap read of a mutex, against one per chunk which would be
thousands.

**The startup check** runs in `setup()`, after `start_archive`, on the async runtime, and **only
if `settings.auto_update` is on**. With the switch off, the app makes **no request at all** — that
is the whole point of the switch, and the property is worth a test on the function that decides it
rather than trust in an `if`.

**Failures never become `Err`.** `Offline`, `NotPublished` and `Rejected` are phases in the
payload, because they are things the user reads, not things that break a command. `IpcError` gains
nothing for the check and the download. `install_update` is the exception and gains one variant,
`UpdateNotReady`, for the only case that is a frontend defect: install asked while the bytes are
not there.

## 5. The screen

`/settings/updates`, `RouteName.Updates`, fifth and last in the Settings section of the sidebar,
after Profile, Appearance, Background and Tabs. Seventeenth route. Icon: `RefreshCwIcon`.

Top to bottom, and it reads like Windows Update because that is what was asked for:

1. **The installed version**, always, whatever the network did.
2. **The switch**, "Aggiorna automaticamente", with a `HelpTip` saying plainly what it does: with
   it on, the app asks GitHub at every start whether there is a new version; with it off, nothing
   leaves the machine unless the button below is pressed.
3. **"Controlla ora"**, disabled during `Checking` and `Downloading` — a second check on top of a
   running one is not a second answer.
4. **The phase**, one line: nothing since this launch / checking / up to date / downloading with a
   progress bar / ready / the failure's own sentence. `Rejected` gets a sentence of its own that
   does not read like bad luck.
5. **The notes**, when there are any, above the button.
6. **"Riavvia e installa"**, only in `Ready`, and the copy says the app will close and come back.
7. **In a development build**, the switch and the button are disabled with the reason written
   underneath, exactly as the Background screen already does for autostart.

Frontend rules apply as everywhere: no `<style>`, no hardcoded sizes, no `invoke` outside
`ui/src/lib/ipc/update.ts`, primitives from `components/ui`, strings through `t()` in both `it.ts`
and `en.ts`. The store is `ui/src/stores/update.ts`, listening for `UPDATE_CHANGED` through
`lib/window/appEvents.ts`.

## 6. The setting

`auto_update: bool` on `ipc::Settings`, **default on**. It is a field in `settings.json` and not a
registry entry: unlike autostart, nothing outside the app has an opinion about it, so there is no
second source of truth to go stale. `#[serde(default)]` is already on the struct, so an existing
`settings.json` reads as "on" without a migration.

**Default on is a decision with a cost, and the cost is stated here**: one HTTPS GET to
`github.com` at every start, carrying what any HTTPS request carries — an IP address and a user
agent. No identifier of ours, nothing about the save, nothing about the machine. `CLAUDE.md`'s
constraint 4 says today that *"the network is only for optional dataset updates"*; this task
amends that sentence to name the second use, because a constraint that quietly stops being true is
worse than one that was never written.

## 7. What is checked

Test-first, expected values from this document and never from the code's output.

**Rust (`crates/ipc`)**: every plugin error class maps to the intended `UpdateFailure`, the
catch-all included; the transitions, in particular that a check during a download changes nothing;
the serialized JSON of `UpdateView` in all six phases, keys pinned in `camelCase`, which is what
catches a missing `rename_all_fields`; `UpdateFailure` and `UpdateReason` serialize as bare strings
and not as tagged objects.

**Frontend (Vitest)**: the store's reducer over a sequence of statuses; which control is enabled in
each phase; that a development build's view disables both; the failure sentences, one per variant,
so a new variant cannot land without its copy.

**Not tested, by the repo's own rule**: `crates/app`, which is the wiring — the plugin call, the
throttle, the startup task.

**The floor moves in the same commit.** `scripts/test-floor` is at `RUST_TESTS=1130`,
`UI_TESTS=688`; `scripts/check` prints the line to paste.

**What no test here can reach** — and it is the part that decides whether the feature works — is
the chain end to end: a signed build, a real release, an older build that finds it, downloads it,
verifies it and installs it. That is a `NEEDS WINDOW` item on the card and it needs two versions,
so it cannot be done before the first release exists. It is the reason the card does not go to
`Done` when the code is green.

## 8. Documents that change in the same commits

- **`docs/architecture.md`** — its header pins five counts and this task makes three of them
  wrong: commands 36 → 40, events 5 → 6, routes 16 → 17. Crates stay 17 and migrations stay 6. The
  route diagram gains `/settings/updates` and the flow diagram gains the one arrow that leaves the
  machine.
- **`CLAUDE.md`** — constraint 4's sentence about the network (§6), and `tauri-plugin-updater` in
  the crate list.
- **`docs/release.md`** — new, and the only place §1 and §2 live in full.
- **`docs/paths.md`, `docs/save-format.md`, `docs/log-format.md`** — untouched. This feature knows
  nothing about the game.

## 9. Known gaps, and what is out of scope

- **Nothing announces a ready update outside the screen.** `tauri-plugin-notification` is already
  registered and a notification when the download finishes would be four lines, but a person who
  never opens the Updates screen is also a person who did not ask to be interrupted; the next start
  installs it anyway. If it turns out to be wanted, it is a follow-up, not a redesign.
- **No rollback, and no "skip this version".** One release is ever offered, the newest, and the
  only way back is to install an older release by hand. A version to skip is a second piece of
  persisted state for a case nobody has hit yet.
- **macOS and Linux compile and are not shipped.** The manifest names one platform key because the
  build produces one. `UnsupportedOs` and `UnsupportedArch` land in `Unknown`, which is honest on a
  machine that cannot exist yet.
- **The uninstaller still leaves the autostart entry**, as the autostart design recorded. An update
  is not an uninstall and does not touch it; the NSIS hook is still that task's.
- **A failed install leaves the downloaded bytes in memory** and the phase in `InstallFailed`.
  Pressing the button again retries with the same verified bytes, which is right when the cause was
  a lock and useless when it was the installer itself; it costs nothing either way.
- **Nothing runs `pnpm build` in any gate**, which was already true and now matters more: the
  signing configuration is only exercised by a real release build, so a typo in `tauri.conf.json`
  is found at release time. `docs/release.md` is where that risk is written down.
