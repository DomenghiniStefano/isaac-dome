# Design system, sub-project 3.5d — "Bloccato" says where to go (design)

**Date:** 2026-09-12
**Milestone:** design system (`docs/STATUS.md`), cycle 3 of 3, pulled ahead of 3.6 the way 3.5c
was: it changes the live IPC contract and every screen that draws a node, so it is cheaper
before 3.6 than after
**Branch:** `feature/blocked-menu`, cut from `develop` (one branch per sub-project, decided
2026-09-11)
**Depends on:** 3.3a (`NodeStateBadge`, `missingGroups`, a node's state and its why), 3.4 (the
Collection's lock badge), 3.5a (a wiki page is a tab location: `pageLocation`, `pageKey`),
3.5b (the palette's open gesture, and `SearchIndex`'s catalog → wiki `Target` mapping);
`DESIGN-BRIEF.md` §7.1 (a node's state and its why), §4.2 (the shell: tabs)
**Owner's request (2026-09-12):** "dove c'è scritto bloccato mi serve sempre un link che mi
spiega come sbloccarlo, non basta il nome", then "cosa ne dici di una lista di link tipo menu
windows?"
**Status:** four choices below are marked **(owner)** — they were answered in the brainstorm.
Everything else is the author's and marked **(delegated)**, so the first look can overturn it
cheaply.

## What this sub-project is

Today the app says *bloccato* and then names what stands in the way — and stops there. The name
is a dead end: to find out how to unlock *Tainted Lost* you have to go looking for it yourself,
in a wiki the app already ships.

Two places say it:

- **`NodeStateBadge.vue`** — Unlock, Next steps and the Plan. The badge reads "bloccato da 3";
  its tooltip lists what is missing, grouped by kind. Plain text.
- **`CollectionRow.vue`** — the Collection. The badge reads "bloccato"; its tooltip reads
  "si sblocca con «Dad's Note»". Plain text.

Both become a **menu**: the badge is the trigger, each blocker is an entry, and an entry opens
that thing's wiki page — the app's own explanation of how it is unlocked, offline, already in
the binary.

The wiki itself needs nothing: `WikiInfobox` already renders "Sbloccato da" as a reference.

## Applicable constraints

1. **Only resolved view-models cross the IPC.** The frontend must not derive a wiki identity
   from a game id: it holds `{ kind, id }` and a page is keyed `entity:20.0.0`. Rust resolves
   the page, as it already resolves names and icons.
2. **Never a link that leads nowhere.** The dataset has no page for everything, and two of the
   six requirement kinds are labels we deliberately did not interpret. Those entries stay in
   the menu, disabled: "we can't take you there" is information; hiding them is not.
3. **Degrade, never fail.** No dataset, no catalog, no page: the menu still lists what is
   missing, with every entry inert. The why never depends on the link working.
4. **One gesture for opening things.** Click navigates the active tab, Ctrl+click opens beside
   it — the two lines in `SearchScreen.vue` and `App.vue`. No third gesture.
5. **The frontend rules hold.** The menu is a primitive in `ui/src/components/ui/`, tokens in
   `@theme`, no `<style>`, no visual constants, strings from the messages.
6. **The IPC contract is live.** The new field is handed on in `DESIGN-BRIEF.md`, not merely
   committed.

## Decision 1 — a requirement carries its page, resolved in Rust (delegated)

`RequirementView` gains `page` on the four variants that are entities:

```rust
pub enum RequirementView {
    Character { id: u32, name: String, tainted: bool, page: Option<Target> },
    Boss { id: u32, name: String, page: Option<Target> },
    Challenge { id: u32, name: String, page: Option<Target> },
    Item { item_kind: ItemKindView, id: u32, name: String, page: Option<Target> },
    Gate { label: String },
    Unknown { label: String },
}
```

`Gate` and `Unknown` do **not** gain the field. They carry a curated label and nothing else; a
variant that cannot have a page must not carry an empty one, and the TypeScript `switch` then
cannot ask for it by mistake.

`LockView` gains the same field on `Unlocked`, `Locked` and `Unknown` — the achievement's page.
`Free` stays as it is: nothing unlocks the item.

`Target` already crosses the boundary (3.5b) and `ui/src/lib/wiki/category.ts` already turns one
into a tab location. Nothing new is invented on the wire.

## Decision 2 — one mapping, shared, not a second copy (delegated)

The catalog → wiki `Target` mapping already exists inside `documents()` in
`crates/ipc/src/search.rs`. It is **not** obvious, and must not be written twice:

| requirement | target |
|---|---|
| item | `Target::Item { id }`, or `Target::Trinket { id }` when `ItemKind::Trinket` |
| character | `Target::Character { id }` — the catalog's id already tells the two forms apart (B28) |
| challenge | `Target::Challenge { number: id }` — the wiki names the field `number` |
| boss | `Target::Entity { id, variant, subtype: 0 }`, read from the **portrait's file name** with `entity_key`, never from `BossId` |

It moves into one function — `crates/ipc/src/wiki_target.rs` — and `search.rs` starts calling
it. A boss whose portrait declares no key names no target, exactly as it names no search
document today: `page: None`.

`page` is `Some` **only when the dataset actually has that page**: `Dataset::entry(&t)`. This is
the same distinction `SearchHit.has_page` draws, so it is drawn the same way and needs no second
field.

## Decision 3 — the dataset reaches the two views the way the catalog does (delegated)

`unlock_view` and `collection_view` gain `dataset: Option<&Dataset>`, the same shape as the
`catalog: Option<&Catalog>` they already take. `crates/app` has it (`wiki::Dataset::embedded()`,
already held for search). No dataset → every `page` is `None` → a menu that still names what is
missing, with every entry disabled. That is constraint 3, not a special case.

## Decision 4 — the badge is the trigger, and the tooltip goes (owner)

Click, or Enter on the focused badge, opens the menu. The tooltip is removed, not kept
alongside: two overlays on one target fight each other, and the menu shows everything the
tooltip showed plus the way out.

A badge with nothing to say — a node with an empty `missing` — is **not** a trigger: it stays
the inert badge it is today. That is exactly the rule already in the file (`groups.length > 0`),
kept.

```
  [bloccato da 3] ▾
  ┌───────────────────────┐
  │ Personaggi            │   ← a kind is a label, from graph.why.*
  │   Tainted Lost        │   ← a blocker is an item: opens its page
  │ Boss                  │
  │   Delirium            │
  │ Condizioni            │
  │   greedmode           │   ← no page: present, disabled
  └───────────────────────┘
```

The groups and their order are `missingGroups()` and `requirementOrder` — the pure function
already tested in `ui/src/lib/graph/nodeState.test.ts`, which now carries the destination beside
the name.

## Decision 5 — "grafo parziale" keeps its why, in the same menu (delegated)

A `partial` node's why lives in the same tooltip today, so removing the tooltip would silently
take it away. It gets the same menu, and there most entries will be disabled labels — which is
the honest reading: *we can't translate this requirement*, not *there is nothing in the way*.

## Decision 6 — the Collection uses the same component, with one group (owner)

One blocker, one group titled with `collection.lockedBy` ("Si sblocca con"), one entry: the
achievement. A menu of one is slightly ceremonious, and is chosen anyway, because the badge then
does the same thing on every screen and you know what a click will do before you make it.

The menu appears exactly where the tooltip appears today — the three lock kinds that name an
achievement (`locked`, `unlocked`, `unknown`) — and not on `free`. An item already in the
collection keeps its way back to the achievement that opened it; that is not a "blocked" screen,
it is the same fact read later.

## Decision 7 — opening is the app's one gesture (delegated)

`tabs.navigate(location)` on click, `tabs.open(location)` with Ctrl — the same two lines as
`SearchScreen.vue` and the sidebar, from the same store. The location comes from
`pageLocation(target)`, which already answers `null` for a target with no category; such an
entry is disabled like any other pageless one.

## Decision 8 — a new primitive: `dropdown-menu` (delegated)

`ui/src/components/ui/dropdown-menu/`, Reka UI's `DropdownMenu` in the shadcn-vue shape, in the
repo like every other primitive — not a component library (the `Don't` list). It brings arrow
keys, type-ahead, Esc and focus return for free, which is what "tipo menu Windows" means. Parts
used: root, trigger, content, label, item, separator. Tokens in `@theme`, a section on the Kit
page (`#kit`) like every other primitive.

## What this sub-project does not do

- **It does not touch the wiki screens.** `WikiInfobox` already links "Sbloccato da".
- **It does not add a second destination.** "Go to the Unlock row of the blocker" was offered
  and turned down: the wiki page is the answer to "how do I unlock this".
- **It does not link Unlock's "sblocca" column** (`UnlockTarget`), nor the Plan's queue rows.
  Same idea, different surface: it belongs in its own pass, logged in `docs/BACKLOG.md` rather
  than smuggled in here.
- **It does not put icons in the menu entries.** The blockers read as names in the tooltip
  today; sprites in a menu are a visual decision for the first look.

## Risks, and the one thing the plan must actually check

**The Plan's rows are draggable.** A badge that opens a menu inside a drag-and-drop row can
steal the `pointerdown` from the drag, or have its own trigger swallowed by it. This is not a
footnote: it is a step in the plan, checked on the real screen, and if the two cannot coexist
the answer is a change to the drag handle's surface, not a badge that behaves differently on one
screen.

Second, smaller: the badge lives inside a virtualized table. The menu must close when its row is
recycled out of view — Reka handles unmount, but the plan verifies it rather than assuming.

## Tests

**Rust**

- A property: for every catalog entity the dataset has a page for, `wiki_target` returns the
  same `Target` that `search`'s `documents()` keys that entity by. The two must never drift —
  that is the whole reason the mapping moved into one function.
- `page` is `Some` if and only if `Dataset::entry` answers: a requirement naming an entity with
  no page carries `None`.
- The JSON shape, pinned like `summary_shape.rs`: `page` is camelCase and present only on the
  four variants; `gate` and `unknown` have no such key.
- Without a dataset, every `page` is `None` and the rest of the view is unchanged.

**Vitest**

- `missingGroups` returns destinations beside names, in `requirementOrder`.
- An entry with no page is rendered disabled, and is still rendered.
- Click navigates, Ctrl+click opens beside — the same two assertions the search rows have.
- A `partial` node has a menu; a node with an empty `missing` has no trigger.

**By eye**

- The Kit page gains the primitive's section; the menu is looked at on Unlock, on the Plan (the
  drag question) and on the Collection before the sub-project closes.

## Handing on the contract

`DESIGN-BRIEF.md` §7.1 gains `page` on the requirement type and on the lock, with the rule that
`None` means *no page*, never *no requirement*. `ui/src/lib/ipc/types.ts` mirrors it by hand, as
always.
