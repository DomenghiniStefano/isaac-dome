# 3.7 — a tab owns what it is showing, and a session is windows of tabs

**Status:** four decisions taken in conversation on 2026-09-15 — the scope, the shape, what the
tab bar does when it is full, and Floor's painted grid. Everything past §3 follows from them and
was not walked through one by one; it is the part most worth disagreeing with.

**Why now.** 3.7 is one of the two sub-projects keeping M1 open, and it is the one B39 has been
waiting for by name: *"a tab keeps its state means a tab owns its state, which is a different
shape from the one the shell has had since 3.1 — which is why this waits for that decision rather
than inventing a second one."* The scope was taken as all four pieces that point here: B39, the
two questions B6 left open, the windows of Decision 8 of
`2026-09-13-drag-and-windows-design.md`, and B27's remembered sizes.

---

## 1. What is already built, read rather than assumed

Read on 2026-09-15 before anything was designed, because half of 3.7 landed early with the tray
and a spec that re-decided it would be a spec re-deciding a shipped thing.

- **The session document exists.** `store` migration 3 — one row, one opaque JSON string, the
  crate storing a string it deliberately cannot parse. The frontend's half is
  `ui/src/lib/window/sessionDocument.ts`: `{ version: 1, tabs, activeIndex }`, a reader that
  drops an unreadable tab **alone** and lands the selection on its neighbour, and a writer.
- **The setting exists.** `resumeTabs` in `settings.json`, answered in one place:
  `window_session` returns `None` when it is off and `set_window_session` does nothing, so no
  caller checks twice.
- **The cap exists.** `MAX_SESSION_BYTES = 64 KiB`, refused rather than truncated, with a comment
  saying *"nothing should approach it — fifty tabs of route names and queries are a few
  kilobytes"*. §6 is that comment's bill coming due.
- **The seed handshake exists.** A window that creates another registers a debt *before*
  creating it (`lib/window/seeds.ts`); the newborn broadcasts `Ready` and is paid its `TabSeed`s.
  `windowPort.create(label, at, size)` opens a window hidden and places it after.
- **`TabSeed` is `Omit<Tab, 'id'>`**, written by subtraction on 2026-09-13 so that *"a tab gaining
  a field needs no line here"*. This spec is the case that was built for.

What does **not** exist: anything that survives about *how* a view was being read, and any window
but `main` in the document.

## 2. The four decisions, as taken

1. **Scope is all four pieces**, against the "one sub-project per merge" rule of 2026-09-11 —
   so: **one spec, three branches** (§11). The shape is a single decision and deciding it three
   times would be worse than one spec that outlives its first branch.
2. **The shape is per history entry** (§3), not per tab and not folded into the location's query.
3. **The tab bar shrinks, then scrolls.** No limit on the number of tabs (§7).
4. **Floor's painted grid stays a scratchpad** (§9) — declined out loud, with its cost.

## 3. Decision — a history entry is a view, not a location

```ts
interface Entry {
  location: TabLocation
  view?: unknown
}

interface Tab {
  id: string
  entries: Entry[] // never empty; `index` always points at one
  index: number
}
```

`TabSeed` stays `Omit<Tab, 'id'>` and does not change: **the tear-off carries the facets across
for free**, which is the property `tabSeed` was written by subtraction to have.

**Why per entry and not per tab.** The tab is already *"its own little browser"* — `entries` plus
an index, `HistoryDepth` of 50, back and forward. A reading state hung off the tab rather than
off the entry would mean going back and finding the facets of the screen you came *from*, which
takes from the history the one promise it makes. Per entry, back restores what you saw there,
and it costs nothing to persist because the document already stores entries.

**Why not the location's query.** It was considered and declined: one home and one mechanism are
worth something, but a facet filter is an array of picks and would make the query unreadable,
`sameView` would have to keep ignoring half of it, and a scroll offset inside a location is an
offset dressed up as a place you could link to. The query keeps what a *link* can carry — `q`,
`state`, `page`, `want` — and the view record keeps how the page is being read.

**How a screen touches it: it doesn't.** One composable, `useTabView(spec)`, hands back a
reactive record. It reads the current entry at mount, writes back as the record changes, on the
same debounce the session already uses. No screen reaches into the tab store, exactly as no
screen calls `invoke()`.

**The write lands in the entry, not on top of it.** `refineTab` already exists and already does
this: it replaces the current entry when the location is the same view, and it refuses a write
whose location is no longer the one the tab is showing — the guard built for the search's
debounced keystroke arriving after the user has gone back or switched tab. A facet click
therefore changes the entry it belongs to and never stacks a new one, so the history stays a
history of places and not of clicks.

## 4. Decision — the shell holds the record, the screen reads it

The shell types `view` as `unknown` and knows nothing about any screen. Each screen declares:

```ts
interface TabViewSpec<T> {
  key: string
  empty: () => T
  read: (value: unknown) => T | null
}
```

`read` is the same shape as `readLocation` in `sessionDocument.ts`, and for the same reason: this
is data written by an **older version of this app**, and the only honest way to receive it is a
validator that can answer "no".

**A record that cannot be read is dropped alone, and the location stays.** This is one notch
finer than today's rule and it is the answer to B6's open question — *how a restored tab states a
gap when its target no longer exists*. A tab whose **route** is gone still falls whole, because
there is nothing left to open. A tab whose **view record** is gone, or whose remembered item no
longer exists, **opens on its screen with the screen's own empty state**, which every screen
already has and which B48 spent 2026-09-15 making say the right sentence. A tab that opens saying
the gap beats a tab that vanishes; a tab that vanishes is the app deciding it knows better than
the thing the user left open.

**What each screen declares** (the survey, read on 2026-09-15):

| screen | what it owns today | in the record |
|---|---|---|
| Unlock | `filter: UnlockFilter`, `sort: UnlockSort` | both, plus the scroll offset |
| Collection | `filter: CollectionFilter`, `sort: CollectionSort` | both, plus the scroll offset |
| Runs | `filter: FacetFilter<RunFacet>`, `selected: RunView \| null` | the filter, and the selected run **by its key**, never the view-model |
| Search | `picked: RowGroup[]` | the groups; the query is already in the location |

Nothing else declares one. A screen with no reading state passes no spec and the entry carries no
`view` — the field is optional so that most entries stay exactly as small as they are today.

**The scroll offset is the awkward one and is treated as B39 wrote it**: it is restored *after*
the data is there, and it means nothing against a list of a different length. The record carries
the offset **and the row count it was taken at**; a list that changed underneath keeps the top
rather than guessing. One component owns every virtualized scroll box in the app
(`components/ui/virtual/VirtualRows.vue`, unified by B43), so this is one place, not four.

## 5. Decision — the session is windows of tabs, and who writes it is elected

Decision 8 of the drag spec already fixed the shape and delegated it here:

> A session is **windows of tabs**: an ordered list of windows, each with its tabs, its active
> one, and its position and size. `main` is the first and is restored first.

**Restoring needs no Rust.** `windowPort.create(label, at, size)` and `oweSeed` are the tear-off's
own machinery: `main` is born, seeds itself from the document, then creates one window per stored
window and owes each of them its seed. A restored window is a torn-off window that nobody
dragged. The only new call into `@tauri-apps/api` is reading the monitors, and it lives in
`lib/window/` like the rest (the scanner's rule covers all three namespaces since 2026-09-13).

**A stored box is clamped onto a monitor that exists.** A window remembered on a second screen
that is no longer plugged in must not open where nobody can reach it. The clamp is a pure
function over the stored box and the available monitors — the geometry never opens off-screen,
and a box with no intersection lands on the primary monitor at the size it had.

**Who writes the document — the finding this spec turns up.** Today only `main` writes, and
`session.rs`'s own doc comment says so. **But `main` can be closed while other windows live**:
nothing prevents it, and `open_or_focus` re-creates `main` only when there is no window at all.
So "only main writes" means the session stops being written the moment the user closes the first
window, silently, and the app is alive in the tray to prove it.

So: **every window broadcasts its tabs and its box as they change; every window keeps the same
ledger of windows; the one that writes is elected by a pure function over the open labels.**
`main` when it is there, the oldest surviving label otherwise. This is the shape `tray_action`
already has — a pure function choosing one window from a list of labels — and it is pure, so the
election is a Vitest test and not a thing you find out about in a window.

The debounce stays what it is: a burst of changes is one write. A failed write is still swallowed
— **except the one in §6**.

## 6. Decision — what the document carries, and the cap it now approaches

```
{
  version: 2,
  windows: [ { tabs: TabSeed[], activeIndex: number, box: StoredBox } ],
  sidebarWidth?: number,
  tables?: Record<string, TableSize>
}
```

**Version 2, and the reader still reads 1.** A `{ version: 1, tabs, activeIndex }` document is
read as one window with no box. This is not politeness: the alternative is that everyone who
updates the app loses the tabs they had open, and the app has no way to tell them why. The bump
is right because version 1's top level said `tabs` and version 2's says `windows` — an old app
reading a new document finds no `tabs` and answers the landing page, which is the "could read it
and be wrong" the version number exists for.

**The cap becomes reachable, and its failure is silent today.** `MAX_SESSION_BYTES` is 64 KiB and
its comment budgets *"fifty tabs of route names and queries"*. A view record per entry, fifty
entries a tab, several tabs a window, several windows: the same arithmetic no longer lands in the
same place. And the failure mode is the worst kind — `set_window_session` answers
`IpcError::SessionTooLarge`, the frontend swallows it because *"a dialog because a session didn't
save would be worse than the session not saving"*, and the user finds out at the next restart,
with nothing having gone wrong on screen.

So the document is bounded **by construction, not by a number**: it keeps the `view` of each
tab's **current entry only**. Restoring puts you back on what you were looking at; going back in
a restored tab gets the screen's empty state, which is the same thing that happens to a tab
restored today. In memory and across a tear-off nothing is pruned — those have no cap and back
must work there.

And the swallowed write gains **one** exception: `SessionTooLarge` is not swallowed, because it
is the only error that means *the session has silently stopped being saved*. It is a diagnostic
in the existing list, not a dialog.

## 7. Decision — the strip shrinks, then scrolls

Tabs shrink to a minimum width — a token in `@theme`, never a pixel in a template — and below it
the strip scrolls horizontally, with the active tab always brought into view. **No limit on the
number of tabs.** The shell says it behaves like a browser, and a browser does not refuse to open
what you ask it to; a hard limit would also need a number that no measurement here could produce.

Two things follow: the tear-off's hit test reads the strip's geometry, so a scrolled strip must
report the same coordinates it draws (`stripUnderPoint`, `toClient`), and the drag-to-reorder
must still work when the strip scrolls under the pointer.

## 8. Decision — the measured sizes (B27)

`sidebarWidth` and `tables` are **named keys beside `windows`**, not tab state and not window
state: the same table read in two tabs has one size, which is what "remembered per table" means.
This is exactly what migration 3's comment promised — *"an object with a version, not a bare
array of tabs, so the sidebar width and per-table sizes join as named parts of the same document,
and a named part costs no migration"* — and it costs none.

A table keys itself by a name it declares, not by its route: two tables on one screen are two
sizes.

## 9. Out of scope, named so nobody has to guess

- **Floor's painted grid.** `stores/floor.ts` declares it *"a scratchpad, not a document: it
  lives here and nowhere else, and it is gone when the app closes. Persisting it would outlive
  the floor it describes."* That still holds: restoring a painted floor tomorrow morning is
  showing the map of a game that ended. **The cost is real and is stated rather than discovered**
  — a Floor tab dragged into another window arrives empty, and this spec chose that over a grid
  that outlives its floor.
- **The profile stays global**, per B6's second constraint and §4.1 of `DESIGN-BRIEF.md`: tabs
  do not each read their own save.
- **The view stores** (`stores/views.ts`) stay per window and keyed by nothing. Two tabs on the
  same screen still share one read of the profile; what they stop sharing is how it is *filtered*.
  Making a view store per tab would be a second read of the same profile for no new answer.
- **No new IPC command and no migration.** The document is opaque both ways and stays one string.

## 10. Tests

Pure, test-first, in Vitest:

- `tabModel`: a view written into the current entry; back and forward carrying their own records;
  a record surviving `tabSeed` (the tear-off) by construction.
- `sessionDocument`: a version 1 document read as one window; an unreadable `view` dropped with
  its location kept; an unreadable route still dropping its tab alone; the current-entry pruning
  of §6 — asserted as *"no stored entry but the current one carries a view"*, which is a property
  and not a byte count.
- the writer election: `main` when present, the oldest label otherwise, and the hand-over when
  the elected window closes.
- the monitor clamp: a box wholly off every monitor lands on the primary at its own size; a box
  that intersects one is left alone.
- `VirtualRows`: an offset taken at a row count is not applied to a list of a different length.

**And the half no test in this repo does.** B39's tag is `nothing, then a window` and B6's is the
same. What only a window says: a filtered tab dragged into another window arrives filtered and
scrolled where it was; two windows closed and the app reopened come back as two windows in their
places; a strip full of tabs scrolls and still tears off correctly. The report writes down what
was seen, and says so when something was not.

## 11. The three sub-projects

One spec, three branches, each merged into `develop` on its own with `scripts/check` green.

1. **3.7a — a tab owns its state.** `Entry`, `useTabView`, the four screens' specs, the scroll
   offset in `VirtualRows`, the strip that shrinks and scrolls, and B6's "a restored tab states
   the gap". The session document keeps version 1's top level and gains the pruned `view`;
   nothing about windows changes. **Landable alone**: at the end of it a tab keeps its reading
   across a tear-off and a restart, which is the whole of B39.
2. **3.7b — windows of tabs.** Version 2, the ledger, the elected writer, the monitor clamp, the
   restore at launch, and the `SessionTooLarge` diagnostic.
3. **3.7c — the measured sizes.** B27: the sidebar's width and each table's size.

## 12. Files

**New.** `ui/src/composables/useTabView.ts` (+ test), `ui/src/lib/window/sessionWriter.ts`
(+ test, the election), `ui/src/lib/window/monitorClamp.ts` (+ test), one `tabView.ts` per screen
that declares a spec.

**Changed.** `ui/src/stores/tabModel.ts` (+ test) and `ui/src/stores/tabs.ts` — `Entry`;
`ui/src/lib/window/sessionDocument.ts` (+ test) — version 2 and the pruning;
`ui/src/lib/window/session.ts` — the ledger and the election; `ui/src/lib/window/windowPort.ts` —
the monitors; `ui/src/lib/window/messages.ts` — the broadcast of a window's tabs and box;
`ui/src/components/shell/TabStrip.vue` — shrink and scroll;
`ui/src/components/ui/virtual/VirtualRows.vue` — the offset;
`ui/src/screens/{UnlockScreen,CollectionScreen,RunsScreen,SearchScreen}.vue`;
`ui/src/components/shell/SectionSidebar.vue` and the tables (3.7c); `@theme` gains the minimum
tab width and nothing else gains a pixel.

**Unchanged, and worth saying.** `crates/store` (migration 3 already holds this), `crates/ipc`'s
session settings, and every Tauri command: 3.7 adds no command and no migration.
