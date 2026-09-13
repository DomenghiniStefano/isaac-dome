# 3.5d — "Bloccato" says where to go (report)

**Date:** 2026-09-12
**Branch:** `feature/blocked-menu`, cut from `develop` at `4320516`
**Spec:** `docs/superpowers/specs/2026-09-12-blocked-menu-design.md`
**Plan:** `docs/superpowers/plans/archive/2026-09-12-blocked-menu.md`

## What the owner asked for

> "dove c'è scritto bloccato mi serve sempre un link che mi spiega come sbloccarlo, non basta
> il nome" — then, on the shape: "una lista di link tipo menu windows?"

## What landed

Seven commits, one per task.

- **`refactor(ipc)`** — the catalog → wiki page mapping came out of `documents()` in `search.rs`
  into `crates/ipc/src/wiki_target.rs`. It is not an obvious mapping: a boss is identified by
  the file name of its portrait, a trinket is a different page kind from a collectible, and the
  wiki's `number` is the catalog's `id`. A test pins the keys search produces, so the two
  readers can't drift.
- **`feat(ipc)`** — `RequirementView`'s four entity kinds carry `page: Option<Target>`, `Some`
  only when `Dataset::entry` answers. `unlock_view` and `queue_view` take the dataset the way
  they already take the catalog; `crates/app` and `design-export` hand over
  `wiki::Dataset::embedded().ok()`.
- **`feat(ipc)`** — the same field on `LockView`'s three variants that name an achievement.
  `free` has none: nothing unlocks the item.
- **`feat(ui)`** — the TypeScript mirror, and the design pack's fixtures filled with `null` plus
  a one-shot console warning, exactly as they already do for the character's `tainted` flag.
- **`feat(ui)`** — `missingGroups` now returns entries (key, name, location) instead of bare
  names, and `whyMenu.ts` turns a node or a lock into menu groups. Pure functions, which is
  where this repo's frontend tests live.
- **`feat(ui)`** — a `dropdown-menu` primitive on Reka, in the repo like every other one, with
  its Kit section.
- **`feat(ui)`** — `WhyMenu.vue`; `NodeStateBadge` and `CollectionRow` lost their tooltips and
  became triggers.

## The two risks the spec named, and what actually happened

**The Plan's drag: no conflict.** The queue's drag starts on the grip button's own
`pointerdown` (`QueueRow.vue`), not on the row, so the badge is a different element and neither
steals the other's gesture. Checked on the real screen: the menu opens on a queued row, and
dragging row 2 onto row 1 still works — the queue answered *"si è fermata sotto «You unlocked
"Samson"»: è un prerequisito"*, which is the repair rule doing its job.

**The virtualized table: no floating menu.** With a menu open on Unlock, scrolling 4000px
recycles the row; the trigger unmounts and the menu goes with it. Measured in the page: both
`[data-slot="dropdown-menu-content"]` and the open trigger are gone after the scroll.

## What could not be checked from here

The click-through **in the real Tauri window**. `pnpm dev` builds and starts
(`target\debug\isaac-dome.exe` runs), but the window can't be driven from this session. What
stands in for it:

- a real-data test — `a_blocked_node_links_to_the_pages_the_dataset_has` in
  `crates/ipc/tests/graph_real.rs` — asserts against the real catalog, the real graph and the
  embedded dataset that every page a requirement carries exists in the dataset, and that the
  real profile has at least one blocker the wiki documents;
- `pageLocation()` is unit-tested through `whyMenu.test.ts`;
- the opening gesture is the same two lines as `SearchScreen.vue` and the sidebar.

So the chain is proven end to end except for the last click. **On fixtures every entry is
disabled**, because the committed design pack predates the field and the fixture layer fills
`page: null` and says so in the console. Regenerating the pack (`pnpm design:export` on this
machine) would fix that and silence both warnings — deliberately not done here: the pack is one
committed artefact and its regeneration would also carry the mark and counter work, which is
not this branch's diff to own.

## Decisions taken during execution

- **A mark and a counter carry no page.** They arrived on `develop` between the spec and the
  execution. They are conditions, not entities — two of the twelve mark columns (Boss Rush,
  Greed) are not entities at all — so a column → entity table would be a curation nobody
  measured. Logged as **B36**; what a node *unlocks* is **B35**.
- **`DESIGN-BRIEF.md` §7.1 also gained the two variants it was missing.** The contract document
  showed six requirement kinds when the wire had eight; adding `page` on top of a stale enum
  would have left it lying in a second way.
- **The gesture has no unit test.** `ui/` mounts no component anywhere, so what a click does is
  checked by eye, as the Kit page is. Said out loud because the spec promised "the same two
  assertions the search rows have" — those assertions don't exist there either.

## One question for the first look

A menu entry with nowhere to go is greyed and says nothing about *why*. Three different reasons
end up looking identical: the dataset has no page, the requirement is a condition (a gate), or
it is a mark the app can't resolve yet. A line of text under the group, or a different
treatment per reason, is a design decision — left open rather than guessed at.
