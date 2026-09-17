# 3.8 — the welcome flow: the app asks which save you are playing with, and shows it

B17 item 2, with B14 folded in. The profile screen stops being a settings page that happens to
hold a chooser: on launch, with no profile settled, the app asks one question and shows what each
answer holds. Decided in conversation on 2026-09-17; the three choices the owner made are §2, §3
and §5.

## 1. What is already built, read rather than assumed

- **A first-run behaviour already exists**, and it is not nothing: `ProgressGate.vue` draws the
  profile selection in place of Progress, on DESIGN-BRIEF §4.3 — *"Progress, while there is a
  choice to make, **is** the profile selection"*. This sub-project replaces that shape, it does
  not fill a hole.
- `CandidatesCard.vue` lists **facts about the file**: edition, slot, where it was found, modified,
  bytes. Nothing about what is inside it.
- `saveSummary()` **takes no argument**: it answers for the *active* profile. A preview per
  candidate is new on the wire, not new layout.
- `App.vue` calls `useWindowSession()` **in the setup, not in the template**, and the tabs store
  lives beside it. Replacing the template's body therefore unmounts neither: the session is not
  at risk from a screen that hides the shell. Had it been mounted inside the markup, §2 would
  have had to go the other way.
- `discovery::Options` already carries `steam_root`, `game_dir` and `save_dir`, `scan_override`
  exists, and `SaveSource::Override` reaches the wire as `CandidateSource::Manual`. **B14's
  backend is half built**; what is missing is a way for anyone to set them.
- `select_profile` re-checks the id against a fresh `discover()` before writing. The welcome
  inherits that: choosing a card is not "write what the frontend said".

Two things the entry got wrong, recorded because the next reader will otherwise trust them:

- B17 names *"the KPI strip already drawn by Progress: achievements, items, marks, last played"*.
  **It does not exist.** What exists is `CompletionKpis` on the Completion screen (normal, hard,
  complete characters, unreadable cells) and the indicator, which says edition, slot and date.
  This spec designs the strip rather than reusing one.
- A save is **11–12 KB**, and `discovery` accepts only the exact name
  `rep[+_]persistentgamedata<N>.dat` — the dated backups beside it are not candidates. So the
  count of candidates is a handful, and reading all of them costs nothing. "Eager or on demand"
  was not a real question and is not decided anywhere below.

## 2. Decision — the takeover is a state above the router, never a route

`App.vue` draws **either** the welcome **or** the shell. No route, no tab, nothing new in the
router table.

The alternative — a `/welcome` route with a guard — fails on something this repo already built:
here a route **is** a tab, and 3.7b's session document stores a window's tabs by location. A tab
pointing at the welcome would be saved and restored, which puts a *state* inside a document that
describes *contents*. The version and the migration that document carries are the measure of how
much that would cost later.

The title bar stays: the window controls and the drag region are needed whatever is drawn. The
tab strip does not. That is **a prop on `TitleBar`**, not a second title bar — frontend rule 4,
*"extend them with a prop instead of styling by hand"*.

## 3. Decision — the preview travels with the candidates, in one command

`CandidateView` gains `preview: Option<CandidatePreview>`; `setup_state` stays the only command
the choice needs.

The reason is N8's, learned in M4's Live screen and written into `docs/STATUS.md`: **the join is
one command**, because two commands cannot promise they describe the same thing. Here the two
would be the candidate list and the previews: a save that appears or disappears between the calls
leaves a card showing the numbers of a file that is no longer offered, and the join would sit in
the frontend where nothing can check it.

`ipc` stays pure. The parsed saves arrive as a **closure**, `read: impl Fn(&Path) -> Option<Save>`,
which is how `SaveCache` already takes its I/O — *"the I/O arrives as closures, so the policy is
testable without a disk"*. `app` holds no decision: it reads and parses. **Which** candidates get
a preview, and what happens when one cannot be read, is decided in `ipc`, where it is tested.

Nothing is cached. `SaveCache` has one slot on purpose — *"a second entry would be a second
profile nobody is looking at"* — and a failed read is never remembered, the rule this repo
already writes down for "the game isn't installed".

## 4. Decision — three counts, each able to say it could not be read

```rust
pub enum PreviewCount {
    Read { done: u32, of: u32 },
    Unread,
}
```

Tagged, with `rename_all_fields = "camelCase"`. A section that did not parse reports `Unread`,
**never `Read { done: 0 }`**: a zero is a profile at the start, and the two must not be one
sentence. It is the same choice as `Verdict::Partial` in `graph`, `Unmodelled` in `floor` and
`Generated::NotSaid` in `run`.

```rust
pub struct CandidatePreview {
    pub achievements: PreviewCount,
    pub items: PreviewCount,
    pub marks: PreviewCount,
    /// Cells the file does not let us read. Zero when the matrix is whole.
    pub unreadable_cells: u32,
    pub diagnostics: Vec<SaveDiagnostic>,
}
```

- **The denominators come from the file**, never from a constant: a 2024 save declares 638
  achievements and a 2025 one 641. Two cards side by side will show different denominators, and
  that is correct.
- **The marks' denominator is the readable cells**, matching what Completion already states —
  *"le celle non leggibili restano fuori dal denominatore"*. `unreadable_cells` is what the card's
  ⚠ line says, and it is the number B20 exists to drive to zero.
- **No `catalog`**: counting needs the file's own layout, which has lived in `core-save` since M2.
  Naming a character needs the catalog; counting does not. So the preview works on a machine with
  no game installed — which is the machine this was designed on.
- `preview` is `None` when the file could not be opened or parsed **at all**. Such a candidate is
  still choosable: "degrade, never fail" means the card says so and lets you through, not that the
  row disappears.

## 5. Decision — the same takeover is the way back

The picker is a state in the profile store, opened from the profile indicator. A `ProfileChanged`
event closes it, which is what makes more than one window behave: three windows with no profile
each draw the takeover, and the first choice frees all of them through the event `select_profile`
already announces.

`ProfileScreen` stays and **loses the candidate table**. What remains is the chain
(Steam → game → saves) and the sections the active save let us read: diagnostics, which is what a
settings page is for.

## 6. B14 inside, and the case B14 does not name

The welcome's "nothing found" branch is where choosing a folder by hand belongs, so B14 lands
here rather than after: its own entry says *"a button that does nothing is worse than none"*, and
shipping the branch without it would be exactly that button.

- The Tauri dialog plugin, with its capability.
- Two commands, `choose_game_folder` and `choose_saves_folder`, each answering a fresh
  `SetupState`. **The path travels inward only** and is never echoed back: what crosses outward is
  what already crosses, a redacted `pathHint`.
- The folders are persisted in the settings file, and `discover` is given them before its own
  search.
- **The case the entry does not name**: a folder chosen by hand that holds no save is *not*
  "we found nothing". "You pointed there, and there is nothing there" is a different sentence for
  whoever is reading, and a `Manual` candidate source that yields an empty list is how it is told
  apart.

## 7. What this does not decide

- The art behind a card. B19 and B33 want the game's own paper under a mark and an achievement;
  a welcome card is not that, and borrowing it here would decide their design by accident.
- B23's Completion KPIs, B27's table sizes, B29's filters: untouched.
- Whether an unreadable candidate should be hidden. It is shown, with what could not be read
  named. Hiding it would be the app deciding a stranger has no save.
- The copy. The wording of the question and of the ⚠ line is written against the design, not
  invented in the components.

## 8. Tests

Test-first, and the expected values come from the spec.

- `ipc`: `preview_of` over the **16 real saves in `samples/`**, through `test-support` so each one
  declares itself. Properties, not pinned values — `done` never exceeds `of`; the `rep_` series
  Jan–Jun 2024 never regresses, **with the vacuity guard** that the series actually contains a
  change, the way `the_three_located_columns_are_not_dead_cells` guards its neighbour; the two
  eras declare different totals.
- `ipc`: a truncated section reports `Unread`, not `Read { done: 0 }` — on a fixture, and the test
  is mutated to check that returning a zero turns it red.
- `ipc`: the JSON shape — `camelCase` on every struct, `kind` on every tagged variant, and the
  fields inside the variants renamed too. That failure is silent in TypeScript and this repo has
  paid for it.
- `ui`: which state draws the takeover, as a table over the three `ActiveProfile` kinds crossed
  with the picker flag — including the one that must **not** draw it, the state where the setup
  has not loaded yet, so the welcome never flashes before we know.
- `ui`: the card's formatting, and `?fixture=pick` extended so the flow can be seen in
  `pnpm ui:dev` with no game and no Tauri window.
- `pnpm check` green, and the test-count floor raised in the same commit.

**What cannot be verified here**: the folder dialog needs a real window. B14's half stays
`NEEDS WINDOW` and says so, and it is one line in *"What only a window can say"*.

## 9. Files

| where | what |
|---|---|
| `crates/ipc/src/preview.rs` | new: `CandidatePreview`, `PreviewCount`, `preview_of` |
| `crates/ipc/src/profile.rs` | `CandidateView.preview`; `setup_state` takes the reader closure |
| `crates/app/src/commands/profile.rs` | the reader; `choose_game_folder`, `choose_saves_folder` |
| `crates/app/src/settings_file.rs` | the two folders |
| `crates/app/capabilities/` | the dialog plugin's capability |
| `ui/src/App.vue` | welcome or shell |
| `ui/src/components/shell/TitleBar.vue` | the prop that drops the tab strip |
| `ui/src/screens/welcome/` | new: the takeover, the card, the "nothing found" branch |
| `ui/src/screens/ProfileScreen.vue` | loses the candidate table |
| `ui/src/stores/profile.ts` | the picker state |
| `ui/src/lib/ipc/fixtures/profile.ts` | previews in the `pick` scenario |
