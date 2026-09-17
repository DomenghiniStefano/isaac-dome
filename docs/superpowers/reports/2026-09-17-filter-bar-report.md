# 3.10 — one filter bar for three lists: report

**2026-09-17**, branch `feature/filter-bar`. B29 closed. Spec
`docs/superpowers/specs/2026-09-17-filter-bar-design.md`, plan
`docs/superpowers/plans/2026-09-17-filter-bar.md`.

## What is there now

One `components/facets/FilterBar.vue` on the Collection, Unlock and the Run diary: `N / M`, the
sort, the state row, the search, the one or two dropdowns that matter on that screen, a fold
holding the rest with **Azzera i filtri**, and the chips underneath. It is the `Card`'s header,
above the table it filters, where before the state control and the drawer sat outside the card
and only the toolbar sat inside it — one filter in three places, on three screens.

`FacetDrawer.vue` and `FilterToolbar.vue` are gone. `StateToggle.vue` is unchanged and is the
bar's state row.

New in the kit: `components/ui/multi-select/`, built on the kit's own `Command`. New in `lib`:
`facets/facetOptions.ts`, which holds every judgment the bar makes.

## What the work corrected, in its own documents

- **The entry said two screens; there are three.** B29 was written before N3 unified the faceting
  engine and the Run diary became the third list on it. Left alone, the app would have kept two
  ways of filtering with nothing recording why.
- **The spec said the multi-select would sit on Reka's `Listbox`.** The kit already had
  `Command` — `ListboxRoot` with the scored search, `CommandInput` on `ListboxFilter`,
  `CommandList` and `CommandEmpty`, all used by the search palette. Writing a second listbox
  beside it would have been two keyboard behaviours to keep agreeing forever, which is the exact
  shape of defect this sub-project removed from the filter. §4 was rewritten before any code.
- **`ListboxFilter` does not filter**, read in `reka-ui@2.10.4`'s own build rather than assumed:
  it keeps the keyboard wiring and highlights the first item. The narrowing is `command/filter.ts`
  — ours, and already written.
- **"Faccette" was already gone** before the work started: the 2026-09-12 fix had set the key's
  value to `Filtri` on all three screens. The card's fourth line was satisfied in advance; the
  keys went with the control that named them.

## The finding, and the decision that came out of it

The state row counted **every** row while the dropdowns counted **what the rest of the filter
leaves**. Both readings are defensible and the app had lived with them since 3.1 — but they were
in different places, and nobody could see the difference. Putting them in one bar made it a
sentence: with `Personaggio · Cain` picked, the Run diary read `2 / 5 run` under a row saying
*vinta 2 · morta 1 · abbandonata 1 · in corso 1*, five runs, three of them no longer in the table.

The owner's call, taken mid-work, was to make the whole bar say one kind of thing. `stateRowCounts`
counts the state row like a dropdown — zero-filled over the row's own order, so a value nobody has
still has a square, and never over its own picks, or picking one state would zero the others and a
second could never be reached. Measured after: `vinta 1 · morta 0 · abbandonata 0 · in corso 1`,
summing to the two rows shown.

Three functions went with the prop they fed — `itemStateCounts`, `stateCounts`, `outcomeCounts`.
The one test that was worth more than its function, the four counts pinned on the committed graph
payload (387 / 119 / 117 / 18), was rewritten through `stateRowCounts` rather than deleted: those
numbers are the fixture's shape, not that function's.

## Measured

| | before | after |
|---|---|---|
| components drawing a filter | 3 (`FacetDrawer`, `FilterToolbar`, `StateToggle`) | 2 (`FilterBar`, `StateToggle`) |
| filter words per screen, in each locale | 4 duplicated (`facets`, `activeFilters`, `noFilters`, a reset) | 0 — one shared `filters.*` block |
| judgment in a component's `computed` | the "a value with nothing behind it is not offered" rule | none: `lib/facets/facetOptions.ts`, 9 tests |
| ui tests | 615 | 626 |

The suite: `pnpm check` **all green**, 1048 Rust and 626 ui tests, 18 declared skips, doc
references with nothing new and nothing stale. The floor was raised to 626 in the same commit as
the tests that raised it.

`docs/architecture.md` was **not** redrawn, and that is a check rather than an omission: its five
pinned counts are crates, commands, events, routes and store migrations, and this sub-project
changed none of them. No IPC type moved; the Rust side was not touched at all.

## What a browser already answered

Driven with Playwright against `pnpm ui:dev`'s fixtures, while the code was being written:

- the threshold works both ways — Pool and Personaggio carry the search field, Qualità and
  Origine do not, and neither screen asked for it
- a menu narrowed to nothing shows its own sentence, not the palette's
- the trigger reads `Pool · angel, boss +1`, in the options' order and not the clicking order
- **a menu's count is a promise the list keeps**: `angel 69` picked, `69 / 721 oggetti`;
  `Apollyon 22` picked, `22 / 641 righe`
- a filter picked behind the fold and the fold then closed by hand: the chip still says what is
  filtering, which is §6's second answer working

One artefact worth recording because it looked like a bug for a minute: two clicks in the *same
tick* lose the first pick — the control is controlled, and Reka computes the next set from the
`modelValue` it currently has. A finger cannot produce it; a script can.

## What only a window can say

Four lines, in `docs/STATUS.md`'s gathered list. The shortest group so far, for the reason the
list's preamble now states: looking early does not empty it, it shortens it.
