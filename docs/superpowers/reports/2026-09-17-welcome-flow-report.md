# 3.8 — the welcome flow, and B14 with it

Branch `feature/welcome-flow`. Spec `docs/superpowers/specs/2026-09-17-welcome-flow-design.md`,
plan `docs/superpowers/plans/2026-09-17-welcome-flow.md`. Closes B17's item 2 and B14.

**What it is.** On launch, with no profile settled, the app draws a full-screen welcome instead of
the shell: one question, a card per save with what that save holds, and — when nothing was found —
which link of the chain broke and two buttons that point at a folder by hand. The same screen is
the way back, from the profile indicator.

## What landed

- **`ipc::preview_of`**, a pure reading of a `.dat` into three counts and the cells it could not
  read. No `catalog`, so it answers on a machine with no game installed.
- **`CandidateView.preview`**, filled by `setup_state`, which now takes the I/O as a closure —
  the shape `SaveCache` already uses. One command carries the list and the previews, so they
  cannot describe two different moments.
- **The takeover**, a state above the router in `App.vue`. No route, no tab, nothing in the
  session document; `TitleBar` gained a `bare` prop and keeps the window controls.
- **`welcomeState`**, which owns the three states the gate used to carry, and `gateState`
  narrowed to the two it can still see.
- **B14**: `tauri-plugin-dialog`, two `async` commands, the folders persisted outside
  `ipc::Settings`, and `discover` given them before its own search.

## Five things the work found that the spec and the plan did not have

1. **`ipc::Settings` crosses the IPC**, so the two chosen folders could not live in it: a
   `PathBuf` there would put the Windows username on the wire, through the `settings` command.
   They live in `settings_file::Stored`, and **`save` preserves them** rather than every caller
   knowing about them — otherwise choosing a folder and then moving the scale slider would
   silently forget the folder.
2. **`select_profile` searched with `Options::default()`**, so a save found in a hand-picked
   folder would have been refused as unknown the moment you chose it. Found by writing the
   command next to it, not by a test.
3. **The achievements denominator is `declared - 1`.** Slot 0 is no achievement — `unlock_view`
   skips it and `collection_view` says so — and this card is the **first place a user ever sees
   an achievements total**: `UnlockTotals.slots`, which counts the file's slots, is drawn only on
   the development-only verification page. Nothing contradicts anything; the number just had to
   be decided once.
4. **The gate lost a third state, not two.** The plan expected `selection` and `blocked` to go.
   With the welcome owning `failed` as well, `gateState` could no longer see that either: what is
   left is "the profile is there" and "we do not know yet". The three states' **tests moved** into
   `welcomeView.test.ts` rather than being deleted, which is the only way the property survives
   the move.
5. **`CandidatePreview` carries no diagnostics**, decided in the plan and worth repeating: a
   `SectionOverrun { section: 7 }` says nothing on a welcome card, and the three counts already
   say which part could not be read.

## The documentation the dialog needed, and what it could not say

Checked against `v2.tauri.app/plugin/dialog/` and docs.rs on 2026-09-17. `pick_folder` (callback)
is *"not a blocking operation, and should be used when running on the main thread"*;
`blocking_pick_folder` *"should NOT be used when running on the main thread"*. **Which thread a
synchronous Tauri command runs on is not documented on that page**, so the commands are `async`
and use the callback, bridged by `tauri::async_runtime::channel` — a re-export of
`tokio::sync::mpsc`, verified in the installed `tauri-2.11.5` source rather than assumed.
`try_send` and never `blocking_send`: the latter panics if the callback runs inside a runtime
thread, and this repo does not panic outside tests.

No `dialog:*` capability is declared: capabilities gate `invoke` from the webview, and this dialog
is opened in Rust. The npm package is not installed either, which is what frontend rule 3 wants.

## Tests

Rust: `crates/ipc/tests/preview.rs` (the counts, and the JSON shape), `preview_real.rs` (three
properties over the 16 real saves, each series walked on its own, with the vacuity guard that the
series actually moves), three new cases in `crates/ipc/tests/profile.rs` for the preview on the
wire — including **the preview following its own candidate through the sort**, which a match by
position would fail — and one for the two kinds of absence. `crates/discovery/tests/discover.rs`
gains the decision between them, as a pure function: asking `discover` would have made the test
depend on what is in the tester's Documents folder.

Frontend: `welcomeView.test.ts` (nine cases, including the three that moved from the gate) and
`previewView.test.ts` (six).

## What nobody has seen

Everything below needs the built app in front of a pair of eyes. It is gathered in
`docs/STATUS.md`, *"What only a window can say"*, which is the live list; these lines are the
record of the day.

- [ ] The takeover on a first launch: no sidebar, no tabs, and the window still moves and closes.
- [ ] The cards: four shapes exist and `?fixture=pick` draws all four — whole, a count that could
      not be read, unreadable cells, and a file that could not be parsed at all.
- [ ] The picker reopened from the indicator over a settled profile, and `Annulla` closing it.
- [ ] Two windows with no profile: choosing in one frees the other.
- [ ] The folder dialog actually opens, and the app stays responsive while it is up — the one
      check the documentation could not settle on paper.
- [ ] A folder chosen that holds no save says so, and does not read as "we found nothing".
