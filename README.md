# IsaacDome

Desktop app that reads the local saves of *The Binding of Isaac: Repentance+* and answers
one question: **what am I missing, and what's worth playing tonight.**

Fan-made project, not affiliated with Nicalis or Edmund McMillen. Works offline, no
account, no telemetry, no configuration: install it and it reads whatever Steam copy it
finds on the machine. It's not a personal tool — the constraint "has to work at a
stranger's house" drives half the project's decisions.

## What it does

Seven screens, all powered by the save file and the game files already on disk.

| Screen | Answers | Status |
|---|---|---|
| **Profile selection** | Which save I'm looking at, and how to switch it | data complete |
| **Next steps** | The 5 things worth doing right now, and the queue I put them in | contract fixed, ordering with M2 |
| **Unlock** | What's missing, filterable on every facet | contract fixed, graph with M2 |
| **Completion** | Character × mark matrix, and how much of it is actually readable | data complete |
| **Collection** | Items never touched, by pool and quality | data complete (909 items) |
| **Runs** | Win rate, nemesis, streaks | M4 |
| **Live** | What I've collected in this run | M4 |
| **Search** | Where this thing lives: items, achievements, challenges, wiki, trees, screens | catalog and wiki complete, graph nodes with M2 |

**Next steps** and **Plan** were two screens until 2026-09-22 and are one now: what is worth doing
and what you decided to do are halves of the same question, and a text button at the bottom of a
section was the only thing joining them. The numbering this table used to carry went with the
merge — it was `DESIGN-BRIEF.md`'s and re-deriving it here only made two documents to keep
agreeing. **Sfide, Wiki, Floor and Stasera are not in this table**: they were built after it was
written, and the list is the shape the project was planned in rather than what ships today.

And a shell that behaves like a browser: **several tabs open together** — a tab is a
location, so it can also be an item's detail or a wiki page, not just a section — with a
global *keep tabs on app close* option. The active profile stays a single one for the
whole window: every number shown depends on which save is being read, and seeing that is
more important than being able to compare two side by side.

Behind the screens:

- **`discovery`** finds Steam, the game, and the saves — even on a second library, even
  with Steam Cloud (where the `.dat` is *not* in Documents), with manual fallback at every
  step.
- **`unpack`** extracts XML and sprites from the game's `.a` archives, on the user's own
  machine: no asset travels in the package. All three compression modes this install uses
  are implemented and verified (14,751 resources extracted, 0 failed).
- **`core-save`** opens the `.dat` **read-only**, by construction. There is no write
  function anywhere in the module, and the checksum is never recomputed.
- **`catalog`** normalizes the game's XML files: 909 items with name, sprite, quality,
  tags and pool, achievements with their unlock condition, challenges with their reward,
  characters with their portrait.
- **`wiki`** builds a dataset from [wiki.gg](https://bindingofisaacrebirth.wiki.gg)
  (CC BY-SA 4.0) pages for effects, notes and synergies: the things the game files don't
  contain.
- **`store`** persists the Plan's goals in a single SQLite file (`isaacdome.db`). It's the
  only file the app writes.

**Rust does everything that touches disk; Vue only ever receives resolved JSON** — the
frontend knows nothing about offsets, paths, or log strings. The boundary is the `ipc`
crate, hand-mirrored in `ui/src/lib/ipc/types.ts`.

## Non-negotiable constraints

1. **Read-only on saves.** A bug here destroys a stranger's profile.
2. **No API keys**, neither asked of the user nor embedded in the binary.
3. **No game assets in the package.** Extracted from the user's own copy at runtime.
4. **No accounts, no backend, no telemetry.** The network is only for optional dataset
   updates.
5. **Degrade, never fail.** If a section of the save can't be read, the app still starts,
   shows what it knows, and states what's missing.

## State

The up-to-date state, as checkboxes, lives in [`docs/STATUS.md`](docs/STATUS.md).

- [x] **M0** — `.dat` format decoded and verified against 28 real saves
- [ ] **M1** — Rust parser, discovery, unpack, Completion screen ← in progress
- [x] **M2** — unlock graph (2026-09-07). The Unlock *section* is frontend work and
      waits for the design system; the graph behind it is done.
- [ ] **M3** — derived plan ← in progress. The plan queue closed on 2026-09-08; what
      remains is the screen.
- [ ] **M4** — log watcher and run archive
- [ ] **M5** — public release

M1 is closed on the Rust side: `core-save`, `discovery`, `unpack`, `catalog`, `ipc`,
`store` and the Tauri skeleton run end-to-end on a real profile. M2, the unlock graph,
closed on 2026-09-07, and M3's plan queue on 2026-09-08 — so the graph screens are no
longer described against placeholders: a node is `computed` or, where a requirement can't
be interpreted, **`partial`**, which is a declared "we can't say" and never reads as
"nothing in the way". The design package went over on 2026-09-09 and the design system is
being built; the frontend code starts with it.

## Ideas and future features

Registered, not promised. Details, with closing criteria, live in
[`docs/BACKLOG.md`](docs/BACKLOG.md) and [`docs/completed/quality-review.md`](docs/completed/quality-review.md).

**The missing heart**

- **"Unlockable now"** (M2): the unlock graph computes what's within reach, how many steps
  are missing, and how much opens up behind a single goal. This is where the app stops
  being a viewer.
- **The plan that updates itself** (M3): pick a goal, the app expands it into steps,
  groups the ones that fit in the same run, and ticks them off by itself by comparing
  saves.
- **Live runs and a historical archive** (M4): `log.txt` gets rewritten on every game
  launch, and unobserved runs are lost forever — the app has to be running while you play.
  From there: what you've collected in this run, then win rate, nemesis, and streaks over
  time.

**On the data**

- **Item detail** with effects, notes and synergies from the wiki dataset, distributed
  with its own license and attribution; optional update from GitHub, never a backend.
- **Lists and search** over challenges and items, after the design system.
- **Free historical series**: `save_backups\` holds dated backups the game creates on its
  own, and `online_logs\` a profile snapshot before and after every co-op session.
  Importing them gives a diff nobody had to record by hand.
- **The online co-op profile** is a separate, shared progression that grows with the
  group, and that the game shows nowhere. Worth surfacing.
- **The save's still-anonymous sections** (3, 5, 8, 9), the counter tail, and the
  bestiary: these close by collecting more saves where those values change.
- **REPENTOGON**, when present, gives better labels to the counters. It's an optional
  bonus, never a promised feature.

**On distribution**

- Tauri installer and updater backed by GitHub Releases, no server. The SmartScreen
  warning for an unsigned installer needs explaining here before the first user sees it.
- **IT/EN i18n** from day one of the frontend, and a real dark theme as the default.
- **Linux and macOS**: the game is on both and Tauri compiles for both — only `discovery`
  would change. Not in v1, but nothing in the code should make it impossible.

**What will never be done**

Modifying or "repairing" a save, not even as a future idea; accounts, backend, or
telemetry; distributing game assets; opening the app on a grid of items, i.e. rebuilding
the menu the user already has.

## Development

Needs Node 22 LTS and pnpm. The pnpm version is declared in `package.json`
(`packageManager`), so `corepack enable` once and it aligns itself.

```
pnpm install
pnpm dev              # Tauri app in development (uses crates/app/tauri.conf.json)
pnpm build            # build the app
```

The wiki dataset is regenerated with two commands. `fetch` is the only point in the repo
that talks to the network; `build` is offline:

```
pnpm wiki:fetch       # downloads pages and Cargo tables into dataset/raw/
pnpm wiki:build       # builds dataset/wiki.json
```

## Verifying

**There's no CI**, by choice: the checks run here, and their list lives in one place,
`scripts/check`. One command runs all of them — `cargo fmt`, `clippy`, the workspace's
tests, `typecheck`, `ui:test`, `lint`, `format:check` and the conventions scanner —
without stopping at the first red, and it summarizes what failed at the end:

```
pnpm check            # or: sh scripts/check
```

The fast ones can be made to run via git, before every commit. Install it once, in a
fresh clone:

```
git config core.hooksPath scripts/git-hooks
```

The hook only runs `cargo fmt --check` and `pnpm scan`: the tests stay in `pnpm check`,
because a slow hook ends up getting bypassed.

**Watch out for skips.** Tests on real data skip *with a note* when the sample is
missing, and `cargo test` alone **doesn't show** those lines: the harness hides output
from passing tests. You need `cargo test --workspace -- --nocapture`, which is what
`scripts/check` does before counting how many tests skipped and which real files the
others ran on.

### Python reference

The format's reference implementation, the one to translate into Rust from:

```
python3 reference/isaac_save.py samples/<save>.dat                # structure and counts
python3 reference/isaac_save.py samples/<a>.dat samples/<b>.dat   # diff between two saves
python3 reference/isaac_counters.py samples/<save>.dat            # mark matrix
```

### Test data

Put a real save in `samples/` (git-ignored), ideally two different dates so the diff gets
tested too. Samples are named with the date —
`YYYYMMDD.rep+persistentgamedata1.dat` — because the pinned numbers in the tests are a
fixture of a known era, and a file named "live" invites overwriting it.

## Documents

- [`CLAUDE.md`](CLAUDE.md) — operating context: constraints, save format, conventions.
- [`docs/PROJECT.md`](docs/PROJECT.md) — the full project document.
- [`docs/architecture.md`](docs/architecture.md) — four diagrams: data flow, the crates, the
  screens, the build.
- [`docs/STATUS.md`](docs/STATUS.md) — checkbox state, open blockers, session log.
- [`DESIGN-BRIEF.md`](DESIGN-BRIEF.md) — the design system's contract, with the TypeScript
  types.
- [`docs/frontend-conventions.md`](docs/frontend-conventions.md) — the frontend rules.
- [`docs/BACKLOG.md`](docs/BACKLOG.md), [`docs/completed/quality-review.md`](docs/completed/quality-review.md) —
  registered tasks.
- `reference/` — reference implementation in Python.

## License and attribution

Three different things travel in this repository, under three different terms.

**The code is [GPL-3.0-only](LICENSE).** A fork stays open, which is the point: this is a
tool built on top of a community's own wiki, and it shouldn't be possible to close it and
sell it back. The identifier is declared once in `[workspace.package]` and inherited by
every crate, so there is one answer and not thirteen.

**The wiki dataset is not covered by that.** `dataset/` derives from the
[Binding of Isaac: Rebirth Wiki](https://bindingofisaacrebirth.wiki.gg) and keeps its
source's **CC BY-SA 4.0**, with attribution and share-alike; the terms and the snapshot's
provenance are in [`dataset/ATTRIBUTION.md`](dataset/ATTRIBUTION.md), which ships in the
package. Including it in a GPL program is a collection, not a derivative work of it: the
dataset's own license rides along with the dataset, and neither license swallows the
other.

**The game's assets are nobody's to license here.** *The Binding of Isaac* and its assets
belong to Nicalis, Inc. and Edmund McMillen: none of it ships in the package, and images
are extracted from the user's own copy at runtime.
