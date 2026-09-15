# M4 sub-project 2a — the Run screen: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this
> plan task by task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** the archive that fills itself is on screen — a diary of runs, filterable, with the
abandoned ones present and marked.

**Split, and why.** The spec covers two screens. This plan is **2a only**: `Run`, which needs
**nothing new on the wire** (`RunsView` already carries every column §2 of the spec lists).
`Live` is 2b and gets its own plan, because it is the half that adds a join with the graph and
therefore a contract decision. Splitting where the contract stops moving is the same cut
sub-project 1 made between 1a and 1b, for the same reason: a plan that changes the contract and
draws a screen at once cannot be reviewed as either.

**Architecture:** the frontend only. Two pure modules — the order of the list and its facets —
then the screen that draws them on the shared virtualized scroll box (`VirtualRows`, B43) and
the shared faceted engine (`faceting.ts`, N3). The route stops falling through to
`PlaceholderScreen`.

**Tech Stack:** Vue 3 + TypeScript, Vitest, Tailwind v4 tokens, vue-i18n, the components in
`ui/src/components/`. No new dependency.

**Spec:** `docs/superpowers/specs/2026-09-14-m4-run-screens-design.md` — §2, §5 and §6 are this
plan; §3 is 2b's.

## Global Constraints

- **No `<style>` in SFCs**, no hardcoded visual constants, no raw `<button>`/`<input>`, no string
  unions (`const X = { … } as const`), no visible string outside vue-i18n. `pnpm scan` checks all
  of it and has no exemption to add here.
- **No `invoke()` outside `ui/src/lib/ipc/`.** The wrapper already exists: `runs()`.
- **`types.ts` is generated.** If this plan ever needs a field, it stops and 2b's contract work
  starts — it does not hand-edit the mirror.
- **Test-first** on both pure modules. The expected value comes from the spec or from the
  archive's own data, never from what the code prints.
- **Nothing counted in the backend.** `RunsView` carries the rows; the facets are the
  frontend's, like Unlock's and the Collection's.
- **Degrade, never fail**: no catalog means items have ids and no names, and the row must still
  draw. An empty archive is one of four named diagnostics, never an invented sentence.
- **`pnpm check`** is the gate. Commits: `type(scope): subject`, English, atomic, **no
  `Co-Authored-By`**.

## What the wire already says, checked before this plan

`ipc::RunView`: `source` (`Live` | `Session { name }`), `ordinal`, `character: Option<String>`,
`seed_words`, `online: bool`, `outcome` (`Won { ending }` | `Died { killer }` | `Abandoned` |
`Open`), `floors: u32`, `starting_items`, `collected`, `held_active`, `achievements: Vec<u32>`.
`RunsView` adds `totals` and `diagnostics`. **There is no timestamp**, which is why Task 1 exists.

## File structure

| File | Responsibility |
|---|---|
| `ui/src/lib/runs/runOrder.ts` (create) | The order of the list: `Live` first, then sessions by the wall clock in their folder name, then by ordinal descending. Pure. |
| `ui/src/lib/runs/runOrder.test.ts` (create) | Including a folder name that does not parse: it keeps its place instead of moving to one end. |
| `ui/src/lib/runs/runFacets.ts` (create) | The facet spec — outcome, character, online, source — and the search text. Pure, on the shared engine. |
| `ui/src/lib/runs/runFacets.test.ts` (create) | Counts leave their own facet out (the engine's rule, pinned here on this screen's rows), an abandoned run is a value like any other, a run with no character is not a fake "unknown" pick. |
| `ui/src/lib/runs/runLabels.ts` (create) | Outcome and source to message keys. No sentence in a component. |
| `ui/src/screens/RunsScreen.vue` (create) | The screen: toolbar, facets, the list, the diagnostics. |
| `ui/src/screens/runs/RunsTable.vue` (create) | The columns, on `VirtualRows`. |
| `ui/src/screens/runs/RunRow.vue` (create) | One row, and the expanded half: starting items, collected, the held active. |
| `ui/src/router/routes.ts` (modify) | `RouteName.Runs` stops falling through to the placeholder. |
| `ui/src/i18n/messages/it.ts`, `en.ts` (modify) | The screen's strings, in key parity. |
| `ui/src/assets/theme/*.css` (modify, only if needed) | A token per new visual value, declared once. |

---

## Task 1 — the order of the list

- [ ] **Test first** (`runOrder.test.ts`): `Live` sorts before every session; two sessions sort by
      the wall clock in `MM_DD_YYYY__HH_MM_SS`, newest first; inside one source, ordinal
      descending; a name that does not parse keeps the position it had rather than sorting to
      one end, because "unreadable" is not "old".
- [ ] `runOrder.ts`: `sessionTime(name): number | null` and `orderRuns(runs): RunView[]`.
- [ ] The doc comment says what the spec says: **there is no timestamp on the wire**, the folder
      name is the only clock, and `Live` has none — which is why it is first by decision and not
      by comparison.

## Task 2 — the facets

- [ ] **Test first** (`runFacets.test.ts`): the four facets and their values; search matches the
      seed and the character; a run with `character: null` contributes **no** value to the
      character facet instead of a value that reads like a character; picking `abandoned` shows
      exactly the abandoned runs.
- [ ] `runFacets.ts` on `createFaceting`, with the same shape as `unlockFacets.ts`.
- [ ] `runLabels.ts`: outcome → key, source → key. `Open` must not read as a failure (§4 of
      sub-project 1's spec), and the label says so.

## Task 3 — the row and the table

- [ ] `RunRow.vue`: character, outcome, floors, seed, and the online mark. The expanded half
      lists `starting_items`, `collected` and `held_active` — **each item degrades to its id when
      the catalog is absent**, which the row renders as the id rather than as an empty cell.
- [ ] `RunsTable.vue`: the columns on `VirtualRows` (B43), `--spacing-virtual-rows-body` for the
      scroll box. 28 sessions on the owner's machine today, so the list is worth virtualizing.
- [ ] No sentence in either file: every string is a key.

## Task 4 — the screen

- [ ] `RunsScreen.vue`: the header, `FilterToolbar` + `FacetDrawer` wired to Task 2, the table,
      and the totals (`runs`, `won`, `died`, `abandoned`, `open`) as the strip above it.
- [ ] **The four diagnostics are the empty states**: `NoLogFolder`, `StoreUnavailable { reason }`,
      `UnreadableEvents { count }`, `NoCatalog`. Each says what is missing; none of them says
      "no runs" when the truth is "we could not read them". An empty archive with no diagnostic
      is its own sentence: nothing has been played since the app was installed.
- [ ] The store reads through the existing `runs()` wrapper and listens to `runs-changed`, which
      `app` already emits.

## Task 5 — the route and the strings

- [ ] `routes.ts`: `RunsScreen` for `RouteName.Runs`.
- [ ] `it.ts` / `en.ts`: every key added to both, in parity. Italian is the app's language and
      the English is the mirror.
- [ ] `pnpm scan` green, including the ban on visible strings.

## Task 6 — the gate, and the window

- [ ] `pnpm check` green.
- [ ] **Seen in a window**: `pnpm ui:dev` draws the screen on the fixtures, and the real app on
      this machine draws the 28 sessions the archive has already ingested. The list scrolling,
      the facets narrowing it, an abandoned run marked, and a row expanding are what no test in
      this repo can say.
- [ ] The report goes to `docs/superpowers/reports/`, and this plan moves to `plans/archive/`
      when the sub-project merges into `develop`.

## What this plan will not do

- **No sort by date**, because there is none. Task 1 is the whole answer to that question.
- **No statistics over the series** — the "bilancio" the design conversation declined.
- **No Live**, no join with the graph, no new field on the wire. That is 2b, and 2b's plan opens
  with the decision this one does not take: whether the marks a character still misses are
  answered by the backend inside the runs view or by a second command. **N8's lesson points at
  the backend** — one screen, one read — and 2b has to say so with its own reason.
