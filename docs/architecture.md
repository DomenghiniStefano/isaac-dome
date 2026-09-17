# Architecture

Four diagrams, one question each: what the program does with the files it finds, which crates
that takes, what the screens are, and what it takes to build and check the thing.

> **This is the state, not the design.** `docs/PROJECT.md` is the design and freezes at M0 by
> its own header — it says so in its first paragraph — so a diagram of *today* could not live
> there without breaking that promise. Drawn on 2026-09-17 against `2ed4820`, the branch it
> lands on.
>
> **What keeps it true, and what does not.** Every path named here is checked by
> `scripts/check-doc-refs.mjs`, which is why the nodes carry real paths instead of pretty
> names. That catches a file renamed under the document and *nothing else*: an edge that
> stopped existing leaves its arrow drawn, and only a person ever notices.
>
> **Redraw it in the same commit as the change**, and `CLAUDE.md` names the five that trigger
> it: a crate added or removed, a route added or removed from `RouteName`, a command joining or
> leaving `generate_handler!` in `crates/app/src/lib.rs`, an event in
> `crates/app/src/events.rs`, a migration in `crates/store/src/migrations.rs`. The five counts
> just below are the tripwire — if one of them is wrong, so is the drawing.

Counted at that commit, and every number below is derived from the code, not from prose:
**16 crates**, **31 Tauri commands**, **4 events**, **15 routes**, **5 store migrations**.

---

## 1 — How the program works

From the files on a stranger's disk to the pixels, in one picture.

```mermaid
flowchart LR
  subgraph disk["The player's machine — opened read-only"]
    steam["Steam registry<br/>libraryfolders.vdf<br/>appmanifest_250900.acf"]
    dat["persistentgamedata1-3.dat"]
    packed[".a archives, ~1.3 GB"]
    logs["log.txt<br/>online_logs/"]
  end

  subgraph shipped["Shipped inside the binary"]
    wikijson["dataset/wiki.json"]
    rulesjson["crates/graph/rules/"]
  end

  subgraph io["Rust — the crates that touch the disk"]
    discovery["discovery"]
    coresave["core-save"]
    unpack["unpack"]
    catalog["catalog"]
    logwatch["log-watch"]
    store["store"]
  end

  subgraph purecrates["Rust — pure, no I/O of their own"]
    wiki["wiki"]
    graphc["graph"]
    plan["plan"]
    runc["run"]
    floor["floor"]
  end

  ipc["ipc<br/>view-models — the only contract"]
  app["crates/app<br/>31 commands, 4 events"]

  subgraph vue["Vue — never touches the disk"]
     app -->|"invoke — 31 commands"| wrappers["lib/ipc/*.ts<br/>typed wrappers, one call()"]
    stores["Pinia stores"]
    screens["14 screens"]
  end

  db[("isaacdome.db<br/>the only file written")]

  steam --> discovery
  discovery -->|"resolved paths"| coresave
  discovery --> unpack
  discovery --> logwatch
  dat --> coresave
  packed --> unpack
  unpack -->|"extracted XML and sprites"| catalog
  logs --> logwatch
  wikijson --> wiki
  rulesjson --> graphc
  catalog --> graphc
  wiki --> graphc
  graphc --> plan
  logwatch --> runc
  runc --> store
  plan --> store
  store <--> db

  coresave --> ipc
  discovery --> ipc
  unpack --> ipc
  catalog --> ipc
  graphc --> ipc
  plan --> ipc
  wiki --> ipc
  runc --> ipc
  floor --> ipc

  ipc --> app
  store --> app
  logwatch --> app

  app -->|"invoke — 30 commands"| wrappers
  app -.->|"4 events, no payload"| wrappers
  ipc -.->|"pnpm ipc:types, build time"| wrappers
  wrappers --> stores
  stores --> screens
```

**Everything that opens a file sits to the left of `ipc`.** That is the project's oldest rule
made visible: the frontend has no path, no offset and no log string to get wrong, because none
ever reaches it. `floor` is the one crate with no arrow coming in from the disk — its input is
the grid the user paints, which arrives from the right.

**One node is a cylinder, and that is the point.** `isaacdome.db` is the only file the app
writes. The `.dat` has an arrow in and none out, by construction: there is no write function in
`core-save` to draw one from.

**The frontend has one door, and it is not `invoke`.** Screens read Pinia stores, stores call
the typed wrappers in `ui/src/lib/ipc/`, and every wrapper goes through the single `call()` in
`transport.ts` — which answers from fixtures in a plain browser and from `invoke` inside Tauri.
`SearchScreen.vue` is today the only screen that reaches a wrapper directly; every other one
stops at a store. `pnpm scan` is what keeps a component from taking the shortcut.

**Pull, then a nudge.** The 31 commands are pull: a window asks, the backend answers. The 4
events (`profile-changed`, `settings-changed`, `plan-changed`, `runs-changed`) are the nudge,
and they carry **no payload** on purpose — a payload would be a copy of state the next command
could contradict. A second window only ever learns of a write it did not make this way.

---

## 2 — The crates

Runtime dependencies only. Dev-dependencies are left out: nine crates depend on `test-support`,
and drawing that would say nothing except that the tests run on real data.

```mermaid
flowchart TD
  subgraph wiring["Wiring — not tested"]
    app["app<br/>Tauri commands, state, tray"]
  end

  subgraph io["Touch the disk"]
    discovery["discovery"]
    coresave["core-save"]
    unpack["unpack"]
    catalog["catalog"]
    logwatch["log-watch"]
    store["store"]
  end

  subgraph purecrates["Pure — no I/O of their own"]
    ipc["ipc"]
    graphc["graph"]
    plan["plan"]
    runc["run"]
    floor["floor"]
    wiki["wiki"]
  end

  subgraph aside["Tools and dev-only"]
    wikisnap["wiki-snapshot<br/>the only crate that talks to the network"]
    designexport["design-export<br/>retired 2026-09-15"]
    testsupport["test-support<br/>dev-dependency only"]
  end

  app --> ipc
  app --> store
  app --> logwatch

  ipc --> discovery
  ipc --> coresave
  ipc --> unpack
  ipc --> catalog
  ipc --> runc
  ipc --> graphc
  ipc --> plan
  ipc --> floor
  ipc --> wiki

  store --> ipc
  store --> runc
  store --> plan

  logwatch --> runc
  logwatch --> store

  graphc --> catalog
  graphc --> wiki
  plan --> graphc

  wikisnap --> wiki
```

**"Pure" means no I/O of its own, not no dependency on I/O.** `ipc` is pure and depends on
`discovery`, `core-save` and `unpack`: it reads their *types* and shapes them, it opens nothing.
That distinction is what lets `ipc` be a crate whose return values are all worth checking.

**`app` depends on eleven crates, and only three edges are drawn.** `ipc`, `store` and
`log-watch` are the ones it orchestrates; the other eight it names to pass their types through.
The full list is `crates/app/Cargo.toml`, and it is the one place where reading the manifest
beats reading a diagram.

**`store` depends on `ipc`, which is the edge worth pausing on** — persistence pointing at the
boundary crate, not the other way round. The plan queue is stored as one JSON document of the
same view-models the frontend receives, so the shape has exactly one definition. `ipc` does not
depend on `store` in return: nothing in the contract knows there is a database.

**Seven crates depend on nothing of ours**: `discovery`, `core-save`, `unpack`, `catalog`,
`run`, `wiki`, `floor`. Each is a leaf that can be read, tested and replaced on its own — which
is the whole reason a patch to the game's format is a patch here and not a rewrite.

---

## 3 — The screens

The shell behaves like a browser: several tabs at once, and a tab is a **location** — a screen,
a wiki page, an item — that saves its identity and never its content.

```mermaid
flowchart TD
  shell["Shell — tab strip, sidebar, profile indicator<br/>one active profile for the whole window"]

  shell --> searchRoute["/search<br/>Search"]
  shell --> progressGroup
  shell --> toolGroup
  shell --> wikiRoute["/wiki<br/>Wiki"]
  shell --> settingsGroup

  subgraph progressGroup["progress — behind the profile gate"]
    goals["/progress/goals"]
    completion["/progress/completion"]
    unlock["/progress/unlock"]
    planr["/progress/plan"]
    collection["/progress/collection"]
    challengesr["/progress/challenges"]
  end

  subgraph toolGroup["tool — answers without a save"]
    runsr["/tool/runs"]
    liver["/tool/live"]
    floorr["/tool/floor"]
  end

  subgraph settingsGroup["settings"]
    profiler["/settings/profile"]
    appearance["/settings/appearance"]
    background["/settings/background"]
    tabsr["/settings/tabs"]
  end
```

The grouping is not decoration: `routes.ts` derives `needsProfile` from the origin being
`progress`, so the five screens in that box are exactly the ones the gate covers. The three
tools answer from the log, the archive and the user's own drawing, which is why they sit
outside it.

| Screen | Path | Origin | Reads through | Commands behind it |
|---|---|---|---|---|
| Search | `/search` | search | `lib/ipc/search` directly, `tabs` | `search` |
| Goals | `/progress/goals` | progress | `views`, `queue`, `tabs` | `graph_views`, `want`, `plan`, `add_goal`, `remove_goal`, the five `queue_*` |
| Completion | `/progress/completion` | progress | `views` | `save_summary`, `completion` |
| Unlock | `/progress/unlock` | progress | `views`, `queue` | `graph_views`, `want`, the five `queue_*` |
| Plan | `/progress/plan` | progress | `views`, `queue` | `plan`, `add_goal`, `remove_goal`, the five `queue_*` |
| Collection | `/progress/collection` | progress | `views` | `collection` |
| Challenges | `/progress/challenges` | progress | `views`, `queue`, `tabs` | `challenges`, the five `queue_*` |
| Runs | `/tool/runs` | tool | `views` | `runs` |
| Live | `/tool/live` | tool | `views` | `live` |
| Floor | `/tool/floor` | tool | `floor` | `floor_candidates` |
| Wiki | `/wiki` | wiki | `wiki` | `wiki_entry`, `wiki_index` |
| Profile | `/settings/profile` | settings | `profile` | `setup_state`, `select_profile`, `save_summary`, `completion` |
| Appearance | `/settings/appearance` | settings | `settings` | the six below |
| Background | `/settings/background` | settings | `settings` | the six below |
| Tabs | `/settings/tabs` | settings | `settings` | the six below |

The `settings` store is shared by its three screens and holds all six between them: `settings`,
`set_scale`, `set_stay_in_background`, `set_resume_tabs`, `autostart`, `set_autostart`. A
column splitting those per screen would be a guess, and the store is the honest granularity.

**Twenty-seven of the thirty commands are reachable from a screen.** The other three are not
loose ends: `window_session` and `set_window_session` belong to the shell and travel through
`ui/src/lib/window/session.ts`, and `extraction_report` is called only by the development-only
verification page, `ui/src/verify/VerifyPage.vue`. 27 + 2 + 1 = 30, which is the kind of sum
worth recomputing whenever this table is edited.

---

## 4 — The infrastructure

Two workspaces in one repository, no CI, and a single script that holds the list of checks.

```mermaid
flowchart TD
  subgraph repo["One repository, two workspaces"]
    cargo["Cargo workspace — crates/"]
    pnpmws["pnpm workspace — ui/"]
  end

  subgraph generated["Generated, never edited by hand"]
    types["ui/src/lib/ipc/types.ts<br/>pnpm ipc:types"]
    reqs["crates/graph/rules/requirements.json<br/>pnpm graph:rules"]
    ds["dataset/wiki.json<br/>pnpm wiki:build"]
  end

  net(["wiki.gg — the only network call in the repo"])
  raw["dataset/raw/<br/>pnpm wiki:fetch"]

  subgraph dev["pnpm dev"]
    predev["predev<br/>scripts/dev-reset.mjs<br/>scripts/prune-incremental.mjs"]
    tauri["tauri dev --config crates/app/tauri.conf.json"]
    vite["vite on :1420, strictPort"]
  end

  subgraph gate["pnpm check — scripts/check, and nowhere else"]
    fmt["cargo fmt --check"]
    clippy["cargo clippy --all-targets -D warnings"]
    ctest["cargo test --workspace -- --nocapture"]
    fe["typecheck · ui:test · lint · format:check · scan"]
    contract["types.ts regenerated and diffed"]
    refs["check-doc-refs.mjs — a report, never a gate"]
    floorc["test-count floor — scripts/test-floor"]
  end

  hook["pre-commit — scripts/git-hooks<br/>cargo fmt and pnpm scan only"]

  subgraph flow["Git"]
    feature["feature/*"]
    develop["develop"]
    master["master — the public face"]
    tag["a release is a tag"]
  end

  cargo --> reqs
  cargo --> types
  net --> raw
  raw --> ds
  pnpmws --> types

  predev --> tauri
  tauri --> vite

  cargo --> gate
  pnpmws --> gate
  gate --> hook

  feature -->|"--no-ff, check green"| develop
  develop -->|"--ff-only"| master
  master --> tag
```

**There is no CI, by decision.** The consequence is that `scripts/check` is not *a* list of the
checks, it is *the* list: the README, the pre-commit hook and `CLAUDE.md` all point at that file
instead of repeating it, so they cannot drift apart. The hook deliberately runs only the two
fast ones — a slow hook is a bypassed hook.

**Three of the boxes in the gate check things a test cannot.** The contract is regenerated into
a scratch copy and diffed, never written, because a check that repairs what it is checking
cannot report on it. The reference report prints and never fails, because it cannot tell a path
written as history from one written as a promise. The test-count floor fails on a suite that
**shrank** and only prints on one that grew — a deleted test fails nothing otherwise.

**`pnpm wiki:fetch` is the only line in this repository that reaches the network**, and it is a
one-pass-per-release tool, never something the app does. `wiki::Dataset::embedded()` reads the
derivative compiled into the binary.

---

## What this document cannot tell you

- **Whether an edge is still there.** A renamed file is caught; a dependency that was deleted
  leaves its arrow drawn. The manifests are the truth: `crates/*/Cargo.toml`.
- **What the app looks like.** That is `DESIGN-BRIEF.md` and the Kit page (`pnpm ui:dev`, `#kit`).
- **Why any of it is like this.** `docs/PROJECT.md` for the design, `docs/STATUS.md` for where
  the project is, `CLAUDE.md` for the rules and the three format documents it points at.
