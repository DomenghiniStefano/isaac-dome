# Design system, cycle 3.1 — the shell and profile selection (design)

**Date:** 2026-09-11
**Milestone:** design system (`docs/STATUS.md`), cycle 3 of 3, sub-project 1 of 7
**Depends on:** cycles 1 and 2 (`2026-09-10-design-system-foundations-design.md`,
`2026-09-10-design-system-components-design.md`; cycle 2 on
`feature/design-system-components`, not yet merged), `Schermate.dc.html` of the Claude
Design export, `DESIGN-BRIEF.md` §4, §4.1, §4.2, §10, `docs/BACKLOG.md` B6
**Status:** every decision below was taken by the author on the owner's delegation ("do as
much as you can, we look at it on first launch"; "push and keep developing meanwhile").
Each is marked **(delegated)** so the first look can overturn it cheaply.

## Cycle 3, decomposed

Cycle 3 is "the screens", and it is seven sub-projects, each with its own spec, plan and
branch history. The order puts what every screen needs first, then the screens with real
data, then what only polishes the whole.

| # | sub-project | what it brings | needs |
|---|---|---|---|
| **1** | **Shell and profile selection** (this document) | the window as the app: real tabs, navbar, sidebar, router, the profile indicator; the profile selection screen; placeholders for the rest | cycles 1–2 |
| 2 | Completion | the marks matrix screen; `docs/BACKLOG.md` B13 (the marks map in `ipc`, the icon protocol for mark sprites) | 1 |
| 3 | Next steps, Unlock, Plan | TanStack Table and Virtual, the graph's facets, the plan queue with drag | 1 |
| 4 | Collection | items by pool and quality | 1, 3 (the table) |
| 5 | Wiki in tabs, search | wiki pages as tabs, the `Ctrl+K` palette and the Search screen (B5) | 1 |
| 6 | Settings and About | the provenance page (§4.3), credits and the three promises (§14) | 1 |
| 7 | Tabs that survive a restart | B6: the store migration, the settings flag, restored tabs that point at nothing | 1, 6 |

## What this sub-project is

The window becomes the application. `App.vue` stops being the verification page and becomes
the shell: the title bar with real tabs and working window controls, the navbar with the two
sections and a persistent profile indicator, the section sidebar, and a router that shows
the active tab's screen. One screen is real — **profile selection**, the one the brief says
to design first and the one every Progress screen depends on. Every other screen already has
its route, its sidebar entry and its tab label, and shows a placeholder that says which
sub-project brings it.

## Applicable constraints

1. **No IPC change and no new command.** Rust changes are limited to window configuration:
   `decorations: false` in `crates/app/tauri.conf.json` and a capability file.
2. **Read-only, offline, no telemetry** — unchanged.
3. **Degrade, never fail.** Steam missing, no saves, a candidate list to choose from, an
   unreadable section: each is a screen state, not an error page.
4. **The frontend rules** hold, with one addition to the scanner (Decision 3).
5. **The active profile is window-global** (§4.1, §4.2): one indicator, outside the tab bar;
   changing profile updates every tab.

## Decision 1 — tabs own locations, the router renders the active one (delegated)

- **Vue Router with memory history.** A tab is a position, and in a Tauri window no one reads
  the URL; the hash is already taken by the development pages (`#kit`, `#verify`). The router
  holds exactly one location: the active tab's.
- **A Pinia store owns the tabs** (`stores/tabs.ts`): each tab is `{ id, location }`, where a
  location is a route name plus query. Selecting a tab replaces the router's location with
  the tab's; navigating inside a tab (a sidebar item, a link) replaces the active tab's
  location. The tab's label and origin icon are derived from its route, never stored.
- **The rules are pure functions** in `stores/tabModel.ts`, over `{ tabs, activeId }`, tested
  first:
  - opening appends a tab after the active one and activates it;
  - closing the active tab activates its right neighbour, or the left one when it was last;
  - closing the only tab replaces it with a fresh tab on the default route — the bar is never
    empty;
  - closing an inactive tab leaves the active one alone;
  - moving uses cycle 2's `moveIndex` result as the final index;
  - navigating replaces the active tab's location and nothing else.
- **Browser gestures:** `+` opens a tab on the default route; a middle click on a tab closes
  it (a `TabItem` change, emitted as `close`); `Ctrl`+click on a sidebar item opens it in a new
  tab.
- **The default route is Next steps**, as B6 says for the first launch. No persistence here:
  sub-project 7 saves tabs.

## Decision 2 — routes (delegated)

| route name | path | section | real in |
|---|---|---|---|
| `NextSteps` | `/progress/next-steps` | Progress | 3 |
| `Completion` | `/progress/completion` | Progress | 2 |
| `Unlock` | `/progress/unlock` | Progress | 3 |
| `Plan` | `/progress/plan` | Progress | 3 |
| `Collection` | `/progress/collection` | Progress | 4 |
| `Runs` | `/progress/runs` | Progress | M4 |
| `Live` | `/progress/live` | Progress | M4 |
| `Wiki` | `/wiki` (`?category=` items, trinkets, achievements, bosses, challenges, characters) | Wiki | 5 |
| `Profile` | `/settings/profile` | Settings | **1** |
| `TabsSettings` | `/settings/tabs` | Settings | 7 |
| `About` | `/about` | About | 6 |

Route names are an `as const` object (`RouteName`); each record carries typed meta: its
section (`TabOrigin`), its title (a `MessageKey`), its sidebar icon, whether it **needs a
profile**, and — for placeholders — which sub-project brings it.

**The profile screen lives under Settings** (delegated, as `Schermate.dc.html` places it:
"the choice of save isn't progress, it's the configuration every progress depends on"). And
**a Progress route without an active profile renders profile selection in its place** (§4:
"Progress, until a choice is made, *is* the profile selection screen"): a `ProgressGate`
wraps every route that needs a profile. The tab keeps its own label — "Unlock" stays "Unlock"
— and the screen says above the selection why it is showing it.

## Decision 3 — the window's chrome goes live

- `crates/app/tauri.conf.json`: `"decorations": false`. The title bar is ours.
- `crates/app/capabilities/default.json`: `core:default` plus
  `core:window:allow-minimize`, `allow-toggle-maximize`, `allow-close`,
  `allow-start-dragging` (the last is what `data-tauri-drag-region` needs). Identifiers
  checked against `crates/app/gen/schemas/desktop-schema.json`.
- **Components never call the window API**, as they never call `invoke()`:
  `lib/window/appWindow.ts` wraps `getCurrentWindow()` — `minimize`, `toggleMaximize`,
  `close`, `onFocusChanged`. Outside Tauri (the development server in a browser) it degrades:
  the controls do nothing and the window counts as focused.
- **The scanner learns it**: an import of `@tauri-apps/api/window` outside `src/lib/window/`
  is a violation, beside the existing `invoke()` rule.

## Decision 4 — the profile indicator (delegated)

`Schermate.dc.html` computes an indicator (`pidLabel`) and never draws it; the brief requires
it in plain view. So:

- **`NavBar` gains a `status` slot**, between the spacer and the search trigger.
- **`shell/ProfileIndicator.vue`** fills it: a `Button` (`Compact`, `Field`) with a 6px square
  and a label, opening the profile screen in the active tab.

| profile state | square | label |
|---|---|---|
| active | `state-done` | `Rep+ · slot 1 · 3 giorni fa` (edition short name, slot, relative modification) |
| needs a choice | `state-unexpected` | "Nessun profilo attivo" |
| none | `state-unexpected` | "Salvataggi non trovati" |
| loading | — | a `Skeleton` the label's width |

Edition short names are data (`rep` → `Rep`, `rep_plus` → `Rep+`), like the DLC names of
cycle 2.

## Decision 5 — sections and the sidebar (delegated)

- **The navbar's section is the sidebar's section.** Clicking Wiki or Progress shows that
  section's items; it doesn't navigate. The cog shows the Settings items; Info opens About in
  the active tab. When the active tab changes, the sidebar follows its section.
  `Schermate.dc.html` draws two states at once (where you are, what you browse); one state is
  simpler and loses nothing a single window needs — revisit on first launch.
- **Items per section** (`shell/sectionNav.ts`, a record over `NavSection` plus Settings):
  Progress lists its seven routes; Wiki its six categories (each a `Wiki` location with its
  `category`); Settings lists Profile and Tabs.
- **Titles and hints** from `Schermate.dc.html`: "Progressi · ogni voce si legge sul profilo
  attivo", "Wiki · funziona senza gioco né salvataggio", "Impostazioni · da qui l'app trova
  gioco e salvataggi".
- The export's sidebar legend ("dati veri / parziale / serve il grafo") describes the design
  pack's progress, not the product: not drawn.
- The sidebar's width is shell state, not persisted (sub-project 7).

## Decision 6 — the profile screen

State comes from a Pinia store, `stores/profile.ts`: `setup` (`SetupState`), `summary`
(`SaveSummary`, only with an active profile), a load status and the last `IpcError`.
`load()` reads `setupState()` and, when active, `saveSummary()`; `choose(id)` calls
`selectProfile(id)` and reloads; `reload()` re-reads both. `noActiveProfile` from
`saveSummary` means "no summary", not an error.

What the screen shows is decided by pure functions in `screens/profile/profileView.ts`,
tested first:

- `chainLinks(setup)` — three links, Steam → game → saves, each `{ link, state, detail }`
  with `state` one of `Found`, `Missing`, `YourChoice`, `Several`, `Chosen`;
- `formatModified(unix | null, now, locale)` — relative and absolute ("l’altro ieri · 31 ago
  2026", with `numeric: 'auto'`: oggi, ieri, l’altro ieri, then "3 giorni fa"), or "sconosciuto" for `null`;
- `formatBytes(n, locale)` — "14.948 byte" / "14,948 bytes", through `Intl.NumberFormat`;
- `editionShort(prefix)`, `indicator(active, now, locale)`;
- `sectionLabel(kind)` — a message for the known section kinds, the raw kind otherwise.

**Layout**, from `Schermate.dc.html` lines 172–379, in cycle 1 and 2 components:

- **Header:** eyebrow, title with the save icon, the paragraph "Non è un passaggio da
  attraversare una volta: è lo stato che decide ogni numero dell'app."
- **Chain** (`CardCollapsible`, open): one row per link, icon, name, detail, a `Badge` —
  `Done` for found or chosen, `Unexpected` for missing, `Tag` for "choose yourself" and
  "more than one". The export's cyan for "more than one" is not allowed (cycle 1).
- **None:** an `Alert` (`Destructive`) "Nessun salvataggio trovato" with the reason spelled
  out per `MissingReason`, a "Riprova la ricerca" button (`reload`), and the diagnostics, one
  line each, from an exhaustive switch over `SetupDiagnostic`.
- **Needs a choice:** `CardCollapsible` "Serve scegliere · N candidati"; when the saved
  profile is gone, an `Alert` saying so without picking another; a `Table` of candidates with
  a `RadioGroup` (edition, slot, where found with a "più recente" `Tag` on the suggestion,
  modified, size); nothing is preselected, the suggestion stays a suggestion; "Usa questo
  profilo" enabled only once a row is chosen.
- **Active:** `CardCollapsible` "Profilo attivo" with a "scelto da noi · era l'unico" `Tag`
  when `autoSelected`; edition and slot as the title; modified, size, DLCs; where found;
  "Cambia profilo" (the candidate table, the active one preselected) and "Rileggi il file".
- **What we could read:** `CardCollapsible` with a grid of the summary's sections (label,
  count) and the save's diagnostics as an `Alert`, plus the export's note that four sections
  are still unnamed and no count is hardcoded.
- **Loading:** `Skeleton` blocks in the cards' shapes. **Error:** an `Alert` with the error
  kind's message and a retry.

The export's manual-folder buttons ("Scegli la cartella del gioco / dei salvataggi") are
**not drawn**: no command accepts a path, and a button that does nothing is worse than none.
Logged as `docs/BACKLOG.md` B14.

## Decision 7 — placeholders and the gate

- `screens/PlaceholderScreen.vue`: the route's title and an `EmptyCategory` saying the screen
  arrives with sub-project N (or M4). Honest, and removed screen by screen.
- `screens/ProgressGate.vue`: for a route that needs a profile, the profile screen with a
  line above it ("Progressi dipende dal profilo attivo: scegline uno per vedere questa
  schermata") until one is active, then the route's own screen.

## Decision 8 — development without Tauri (delegated)

The shell must be verifiable in a browser, where `invoke()` doesn't exist. So:

- the IPC wrappers call `lib/ipc/transport.ts`'s `call(command, args)` instead of `invoke`
  directly: in Tauri it is `invoke`; in a development build outside Tauri it answers from
  **fixtures**; in a production build the fixture module is never imported;
- fixtures (`lib/ipc/fixtures/`) hold the brief's real values (§5.4: Steam on `C:`, the game
  on a second library, four candidates with their sizes, the ten section counts) in three
  scenarios chosen with `?fixture=none|pick|active` on the development server;
- the verification page moves to `src/verify/VerifyPage.vue` behind `#verify`, like the Kit,
  and the scanner's development-only directories become `src/kit/` and `src/verify/`.

## i18n

New messages under `routes.*` (titles), `shell.*` (section hints, indicator labels,
placeholders, the gate line) and `profile.*` (headings, chain link names and states, missing
reasons, diagnostics, candidate columns, actions, section labels). Game names (editions,
DLCs) stay data.

## Testing

Vitest, test-first, expectations from this document:

- **`tabModel`** — every rule of Decision 1, one test each, plus "the bar is never empty".
- **`profileView`** — `chainLinks` for `none` with each `MissingReason`, for `needsChoice`
  (saves `Several`), for `active` (saves `Chosen`); `formatModified` with a fixed `now`: today,
  yesterday ("ieri"), two days ("l’altro ieri · 31 ago 2026"), three days ("3 giorni fa"), English ("2 days ago · Aug 31, 2026"), `null`; `formatBytes` in `it`
  (14948 → "14.948 byte", 4068 → "4068 byte": Italian groups from five digits) and `en`
  ("14,948 bytes", "4,068 bytes"); `editionShort` for both prefixes; `indicator` for the
  three states; `sectionLabel` for a known and an unknown kind.
- **`transport`** — outside Tauri in development it answers from the scenario's fixture; an
  unknown command is an error, not `undefined`.

Visual checks on the development server with each fixture scenario: the three profile
states, the gate on a Progress route, the indicator in each state, tabs opening, closing and
moving, the sidebar following the active tab. The production build is checked for the absence
of the fixtures, the Kit and the verification page.

## Files

```
crates/app/tauri.conf.json            decorations: false
crates/app/capabilities/default.json  the window permissions
ui/package.json                       vue-router, pinia
ui/src/
  main.ts                             dev pages first; the app with pinia, router, i18n
  App.vue                             the shell
  router/index.ts routes.ts           RouteName, route meta
  stores/tabs.ts tabModel.ts (+test) profile.ts
  screens/PlaceholderScreen.vue ProgressGate.vue ProfileScreen.vue
  screens/profile/                    ChainCard NoSavesCard CandidatesCard ActiveProfileCard
                                      SectionsCard profileView.ts (+test)
  components/shell/                   ProfileIndicator.vue sectionNav.ts; NavBar (status
                                      slot); TabItem (middle click)
  lib/window/appWindow.ts
  lib/ipc/transport.ts (+test) fixtures/  and every wrapper calling transport
  verify/VerifyPage.vue               the former App.vue
  i18n/messages/it.ts en.ts
ui/scripts/scan-conventions.mjs       window API confinement; dev-only dirs
```

## Out of scope for this sub-project

- Every screen but profile selection (sub-projects 2–6).
- Tab persistence, the tabs settings page, restored tabs that point at nothing (7).
- Choosing a folder by hand (B14: a dialog and a command that accepts a path).
- The search palette's content (5): the trigger opens nothing yet.

## Deviations recorded while planning and executing

- **The pure profile logic lives in `ui/src/lib/profile/`** (`profileView.ts`,
  `profileLabels.ts`), not under `screens/profile/`: the indicator in `components/shell/`
  reads it too, and a component folder importing from a screen folder would run backwards.
- **`router/routeTable.ts` carries no components**: names, paths, titles, icons, origins.
  `routes.ts` adds the screens, so the tab model and its tests import the table without
  pulling `.vue` files into Vitest.
- **`formatCount`, not `formatBytes`**: the unit is a message ("byte"/"bytes"), the number
  is `Intl.NumberFormat`, and the same function formats the section counts.
- **`formatModified(null)` returns `null`**, and the caller shows its own "unknown date"
  message: the pure function has no messages to translate.
- **Two days ago reads "l’altro ieri"**, not "2 giorni fa": that is what
  `Intl.RelativeTimeFormat` with `numeric: 'auto'` says in Italian, and the tests follow it.
- **vue-router 4.6.4 and pinia 4.0.3**, the versions current on 2026-09-11.
- **Tasks 2, 6 and 7 landed in one commit**: moving `App.vue` to `verify/` leaves `main.ts`
  without an app until the shell exists, and no intermediate commit typechecks.
