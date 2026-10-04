# A run's page — everything the log said about one run

Cards #95 and #97. Design agreed with the owner on 2026-10-04.

## What it is for

Looking at Runs in UAT of #92, the owner asked for the single run to be *"more informative, not
only the items — a page of its own"*, and said what matters: **how it ended, what killed me, every
piece of information there is**. The items are fine as they are, **with further ways to look at
them**. Card #97 adds two things beside: a **Copy** button by the seed, and **the character's
face** by the name.

## What it is not

- Not a clock per run, nor a duration: the log has neither (B55, #92). The date is the source's.
- Not a new route: the page is the Runs screen with a `run` query, the way a wiki page is the
  Wiki screen with a `page` query. The route count in `docs/architecture.md` does not move.
- Not a fix of #98 (a local run abandoned when an online one starts): that is the fold's rule and
  has its own card. This page shows whatever the fold says.

## How you get there

A click on a run in the list opens its page **in the same tab**; Ctrl-click opens it in a new
one — what every link in the app already does (`tabs.go(location, ctrlKey)`). Back returns to the
list, at the place the page box kept (`v-scroll-memory`).

The location is `{ name: RouteName.Runs, query: { run: <key> } }`. `RunsScreen.vue` picks its body
from the query, as `WikiScreen.vue` does: `RunPage` when there is a key, `RunsList` (today's list)
otherwise. Both bodies are `PageScroll` roots. `RunsScreen.vue` has no root of its own any more, so
its exemption in `scan-conventions.mjs` changes reason to WikiScreen's (*it picks one of two
bodies, and each carries the shape*), and `RunsList.vue` / `RunPage.vue` are checked as screens.

**A key that matches no run** — the archive was rebuilt, the run belonged to a source since gone —
is a page that says so, with a link back to the list. Never an empty page, never the list silently.

`RunDetail.vue`, the card under the table, is deleted with its last caller.

## A run's key becomes stable

Today the latest launch is `live:#3`, and after a relaunch the same key names run 3 of the *new*
launch (the minor left open on #92). `RunSource::Live` gains the source's row id —
`Live { id, written_unix }` — and the key of any launch, latest or past, becomes `log:<id>#<n>`.
A session stays `session:<name>#<n>`. `sourcePart` in `ui/src/lib/runs/runKey.ts` remains the one
place that writes it. A stored selection or link with an old `live:` key matches nothing, and is
the "no such run" page.

## What the fold keeps that it reads and drops today

`run::Run` changes shape:

- `collected: Vec<u32>` becomes `collected: Vec<Pickup>`, where
  `Pickup { id: u32, pool: String, floor: Option<u32> }` — `pool` is the word the log writes on the
  item line (`from pool treasure`), kept as written; `floor` is the index into `floors` of the
  floor the item was taken on, `None` before the first `FloorEntered`.
- `starting_items` keeps `Vec<u32>`: they come with the character, not from a pool or a floor.
- `Outcome::Died { killer }` becomes `Outcome::Died { killer, spawner }`: the fold reads
  `Event::Died { killer, .. }` and drops the spawner today. One outcome carrying both, not a
  second field beside it that could disagree with it.

The run's cached JSON changes shape, so **the fold's rules version goes up**: every cached fold is
stale, and `Ingest::refold_stale` rebuilds the archive from the events it already holds at the next
launch. Nothing is read from disk again and nothing is lost.

## What crosses the wire

`RunView` gains, all `camelCase`:

- `characterHeadUrl: Option<String>` — the co-op menu head, from the same function that fills
  `CharacterRow::head_url` on the Completion matrix, by `character_id` first and by name only as a
  fallback (`CLAUDE.md`: a reference that carries an id is resolved by the id). `None` without a
  catalog; the row keeps its width either way.
- `floorDetails: Vec<FloorView>` — `FloorView { stage, stageType, name: Option<String>, rooms:
  Option<u32> }`. `floors: u32` stays for the list.
- `collected` becomes `Vec<PickupView>`: the existing `RunItemRef` plus `pool: String` and
  `floor: Option<u32>`.
- `passives` and `familiars`: `Vec<RunItemRef>`, which the fold already separates.
- `RunOutcomeView::Died` gains `spawner: String` and `killerPage: Option<Target>`, the wiki
  entity the killer's name resolves to, when it does.

The list carries these too: one view for list and page, no second command. An archive of a few
hundred runs is not a payload worth a second contract.

### A floor's name — measured before it is written

The log names a floor by numbers (`m_Stage 2, m_StageType 1`). **Where the name comes from is
measured first** — the game's own archives are the expected source (a `stages.xml`, read by
`catalog` like `players.xml`), and the wiki's floor pages the second. The plan's first task is that
measurement, done through `test-support`, writing nothing under `samples/` (the extraction example
writes into `samples/sprites/`, and must not be used for this). Until there is a source:
`name: None`, and the page shows the numbers, never a guessed name — `CLAUDE.md`'s rule about
naming from a guess applies to floors as it does to sections.

## The page

`PageScroll`, top to bottom:

1. **Header** — the face and the character; the outcome in full: the ending reached, or the
   killer, its spawner when it says more, and the killer's picture and link when the wiki knows it;
   the date and time with what the time is (#92's tooltip, written out here); the seed with a
   **Copy** button (a `copyText` helper in `ui/src/lib/`, the first in the codebase, with a
   "copied" confirmation); solo or online; the number of floors.
2. **Achievements unlocked** in the run, as chips linking to their pages.
3. **The route** — one row per floor: its name (or its numbers), the rooms it had, the items taken
   on it.
4. **The items** — as today by default (starting, collected, active held), and a selector with
   three more views: **by type** (starting, passives, familiars, active held), **by origin** (one
   group per pool, the known pools translated, an unknown one shown as written), **by floor**. The
   selected view is the tab's (`TabViewSpec`), so it survives a tab switch and a restart.

## The list (#97)

The character's face beside the name in each row (`characterHeadUrl`, the sprite tokens), and
nothing that is a button inside the row — the row is one. Copy is on the page.

## Documents

- `docs/frontend-conventions.md`: Runs leaves "the one screen still filling".
- `CLAUDE.md`'s `run` row: the fold keeps each pickup's pool and floor.
- `docs/log-format.md`: what `pool` holds on the item line, if the measurement adds to it.

## Tests (first)

- **run**: the fold keeps pool and floor on each pickup, `None` before the first floor; the spawner
  is kept; the rules version moved.
- **ipc**: the JSON shape of `FloorView`, `PickupView`, the `died` outcome, `characterHeadUrl` (camelCase,
  pinned); a face resolved by id for a Tainted character whose name is the base form's.
- **store**: `Live` carries its id.
- **ui**: keys `log:<id>#n` for live and past launches alike; the page's choice of body from the
  query; a key that matches nothing is the "no such run" state; each item view groups as described;
  an unknown pool shown as written; `copyText` resolves and rejects.

## What only a window can say

The page on a real archive, the faces, the killer's picture, the Copy confirmation, and the route
of a run with a Greed floor (no rooms line) and a Mines II (two passes). NEEDS WINDOW.
