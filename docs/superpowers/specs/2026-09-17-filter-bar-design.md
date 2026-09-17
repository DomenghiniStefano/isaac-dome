# 3.10 — one filter bar, three lists: the fold, the dropdowns, and the end of the drawer

B29. The facet drawer stops being a panel of checkbox columns beside the list and becomes the
list's own filter bar: the state in view, the search, the two controls that matter on that
screen, and everything else behind a fold. Decided in conversation on 2026-09-17; the four
choices the owner made are §2, §3, §4 and §5.

## 1. What is already built, read rather than assumed

- **Three screens carry these controls, not two.** The entry says "both screens" and names
  Unlock and the Collection; `RunsScreen.vue` mounts `FacetDrawer` and `FilterToolbar` too. It
  was written before N3 unified the engine and the Run diary became the third list on it. The
  owner's call is that all three move together: two ways of filtering in one app is the thing
  this sub-project exists to end.
- **The two things wrong on sight are already fixed**, on 2026-09-12. `facets: 'Filtri'` in all
  three blocks of `ui/src/i18n/messages/it.ts`, so **the word "Faccette" is already absent from
  the app** — the card's fourth line is satisfied before the work starts, and the only thing this
  sub-project owes it is not to bring the word back. The empty values are gone too: the drawer
  filters on `count > 0 || picked`.
- **That rule lives in a `computed` inside `FacetDrawer.vue`**, where no test can see it. It is
  the one piece of judgment in the whole control, and it is the piece that is not covered.
- **The engine is not part of this.** `ui/src/lib/facets/faceting.ts` keeps matching, the counts
  that leave their own facet out, the options and the active count. Nothing below changes a
  number; what changes is which control shows it.
- **`StateToggle` is already screen-driven**: the order, the counts, the dot class and the text
  arrive as tables. A screen maps its own vocabulary onto shared tokens, which is why a third
  table for Runs is not a duplication.
- **The counts on the state control do not come from the engine.** The Collection reads
  `itemStateCounts(items)`, over every row; a dropdown's counts come from
  `faceting.counts(...)`, over the rows the *other* facets leave. Two different questions, and
  the bar must not quietly answer one with the other.
- **Reka `ListboxRoot` has what this needs**: `multiple`, `selectionBehavior: 'toggle'`,
  `highlightOnHover`, `by`.
- **`ListboxFilter` does not filter.** Read in `reka-ui@2.10.4`'s own build: it is an input that
  keeps the keyboard wiring and highlights the first item, and it narrows nothing. The narrowing
  is ours, over the options we hand the list. A design that assumed otherwise would have found
  out in the browser.
- `ListboxVirtualizer` exists and is **not** used: the longest option list in the app is 39
  characters, and the virtual list this repo already has is for 733 rows, not 39.
- **Runs' outcome already carries the four state tones**, in `ui/src/screens/runs/RunRow.vue`:
  won → `Done`, died → `Blocked`, abandoned → `Partial`, open → `Now`, each with the reason
  written beside it. The state control does not invent a palette for Runs, it reads the one the
  table already uses.
- **The same four words are written three times.** `facets`, `activeFilters`, `noFilters` and a
  reset live under `runs.`, `collection.` and `unlock.`, with identical values in both locales.

## 2. Decision — one bar, described by the screen and drawn by the bar

`ui/src/components/facets/FilterBar.vue` replaces `FacetDrawer.vue` and `FilterToolbar.vue`. A
screen hands it a **description** of its facets — which ones, in what order, titled how, in view
or behind the fold — and the bar draws the state row, the search, the dropdowns, the fold, the
chips, the reset and the sort.

The alternative was a slot-based shell, each screen composing its own controls. It was declined
because the three screens would then wire five controls each, and the chips row needs every pick
anyway — the freedom would be paid for and not used. The declarative shape is also the one these
components already have: `title`, `valueLabel` and `labels` are tables the screen passes, and
"in view or folded" is one more column of the same table.

`StateToggle.vue` survives unchanged, as the bar's state row. The sort group stays optional for
the reason it already is: the Run diary has nothing to choose between, and a single fake option
is a control that changes nothing.

## 3. Decision — what stays in view at rest

| screen | state row | in view | behind the fold |
|---|---|---|---|
| Collection | in collezione / da trovare / bloccato / non leggibile | Qualità | Pool, Tipo, DLC di origine |
| Unlock | fatti / ora / bloccati / parziali | Cosa sblocca | DLC di origine, Personaggio |
| Runs | **esito**, promoted (§5) | Personaggio | Con chi, Da dove |

The state keeps the shape it has — coloured squares, every value, its count — and does not become
a dropdown. It is the filter that matters most (DESIGN-BRIEF §6), and putting it in a menu would
mean opening a menu to find out how many items are still missing.

## 4. Decision — one multi-select, and the search appears by itself

`ui/src/components/ui/multi-select/`, a kit primitive on Reka `Listbox` inside `Popover`, in the
shape the other primitives have (`index.ts`, the component, `variants.ts` if it earns one). It
takes `{ value, label, count }` options, the picked values and a label; the trigger reads
`Qualità · 3, 4`. Its own row on the Kit page.

**The search field inside it appears from a named threshold on the number of options, not from a
per-facet flag.** How many pools exist is a property of the user's install — on a machine with no
game the list arrives empty — so a `searchable: true` written by hand on the Pool facet would be
a guess about somebody else's data. The control reads the list it was actually given.

The number itself is the plan's to pick, and it is a **named constant in the primitive's own
module**, not a literal at a call site: it is a behaviour, not a visual value, so it belongs
neither in `@theme` nor in three screens that would drift apart.

The field is Reka's `ListboxFilter`, for the keyboard behaviour (arrow down from the input into
the list, first match highlighted), over options **we** have already narrowed (§1).

## 5. Decision — Runs' outcome is the state control

The esito becomes Runs' state row: squares, counts, all four values in view, on the tones
`RunRow.vue` already gives them. An `outcomeDot` table joins `outcomeText` in
`ui/src/lib/runs/runLabels.ts`, the way the other two screens carry theirs.

The KPI tiles above the table stay, and they are not the same statement: they count the archive,
the state row counts what the search and the other facets have left. The two numbers differ the
moment anything else is picked, which is exactly why the row is not a repetition.

## 6. The fold remembers nothing, and can never hide an active filter

The fold opens by itself when a facet behind it holds a pick, and is otherwise closed; after that
it is the component's own state.

So there is **no change to the session document**: no migration, no version bump, nothing added
to a tab's view. And the failure the fold could produce — a list filtered by something the reader
cannot see — cannot happen: either the control is in view, or the fold opened to show it. The
chips row is the second answer to the same risk, and is kept for it.

## 7. The bar is the card's header

Today the state control and the drawer sit **outside** the `Card` and the toolbar sits inside it:
one filter in three places, on all three screens. The bar is the card's header, above the table
it filters.

## 8. The pure half, and it is tested first

`ui/src/lib/facets/facetOptions.ts`, Vitest, written before the components:

- **the options of a facet**: value, label, count, picked — dropping a value with no count unless
  it is picked, which is the rule that has never had a test
- **which facets are in view and which are folded**, from the screen's description
- **whether the fold starts open**: true when any folded facet holds a pick

The components stay drawing. Anything worth checking lives in that module, which is the frontend
reading of the rule the Rust side states as *"if a return value is worth checking, it lives in a
pure crate"*.

## 9. Words

One shared `filters.*` block — "altri filtri", "meno filtri", "filtri attivi", "azzera" — read by
all three screens. What stays per screen is what is genuinely its own: the noun for a row, the
search placeholder, the facet titles, the state names. The three `*.facets` keys go, with the
control they named.

## 10. What this does not do

- It does not touch `lib/facets/faceting.ts`, nor any count.
- It does not make a table fill the page or be sized by the mouse: that is B27, and it stays
  open with two of its three parts untouched.
- It does not unify the three `dot` tables into one palette. Each maps a screen's own vocabulary
  onto tokens declared once in CSS, which is the component's contract and not a duplication.
- It adds no dependency: `reka-ui` is already here and already has `Listbox`.

## 11. What only a window can say

Written here so the report can copy it into `docs/STATUS.md`'s gathered list:

- the three bars at rest, and the fold opened, at a normal window width
- the bar in a narrow window: the dropdowns wrap and the chips row stays readable
- the search inside the Pool and Personaggio menus, and its absence from Qualità and Origine
- a menu with one option left after typing, and one with none
- a filter picked behind the fold, the tab left and reopened: the fold is open on arrival
- the Runs state row beside the KPI tiles: two numbers that differ do not read as a contradiction
