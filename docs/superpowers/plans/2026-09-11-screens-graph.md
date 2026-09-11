# Cycle 3.3a — the node, Next steps and Unlock: Implementation Plan

> **For agentic workers:** executed inline with superpowers:executing-plans, on the owner's
> delegation ("procedi come credi"). Steps use checkbox (`- [ ]`) syntax. Every commit is pushed
> right away (the owner's rule: the remote stays aligned).

**Goal:** Next steps and Unlock on the real unlock graph — one way to draw a node's state and
its "why", four facets that have data, a search, three sorts and a virtualized table.

**Architecture:** no IPC change. Every decision about a node lives in pure functions in
`ui/src/lib/graph/`, tested first against the design pack's committed `unlock.json`, which the
development fixtures also answer. A Pinia store reads `unlock()` and `nextSteps()` once per
profile. Components are presentational; the Unlock body is virtualized with TanStack Virtual.

**Tech Stack:** Vue 3.5, TypeScript, Pinia 4.0.3, Tailwind v4.3, Reka UI 2.10.4, vue-i18n 11.4,
`@tanstack/vue-virtual` 3.13.37, Vitest.

**Spec:** `docs/superpowers/specs/2026-09-11-screens-graph-design.md`

## Global Constraints

- No IPC change: `unlock()` and `nextSteps()` as they are.
- A `partial` node is never gold, never a step, never counted as unlockable.
- Only the facets with data: state, what it unlocks, origin DLC, required character.
- Degrade: `noCatalog`, `noAchievementSection`, `slotsBeyondCatalog`, `catalogBeyondSlots` each shown.
- Frontend: the five rules, `assertNever` on closed switches, `as const` objects, every visible string through `useMessages()`, game texts are data.
- Fixtures and pack images only under `import.meta.env.DEV`; payloads read through `import.meta.glob`, never a JSON import outside `src/` (it would leave `vue-tsc --build`'s file list).
- Commits: Conventional Commits, English, no `Co-Authored-By` or Claude reference, **pushed after each task**.

**Deviations decided while planning** (recorded in the spec): TanStack Table is not introduced
here — its faceted unique values count an array cell as one value, while "what it unlocks" needs
one count per kind and every count must exclude its own facet; the pure functions already do
filtering, counting and sorting, and a second engine would be a second source of truth. It is
reconsidered for Collection (3.4), whose columns are scalar. The filter state lives in the
Unlock screen, so it resets when the tab is left (tab state is 3.7).

---

### Task 1: The Badge learns the partial state

**Files:** Modify `ui/src/components/ui/badge/variants.ts`, `icons.ts`, `ui/src/kit/sections/BadgeSection.vue`

- [ ] **Step 1**: `BadgeVariant.Partial: 'partial'`; its classes `${statePill} border-dashed border-state-blocked bg-state-blocked-surface text-state-blocked-foreground`, beside a comment: not unlockable (the blocked colours), not fully computed (the dashed edge), never the unknown hatch.
- [ ] **Step 2**: `badgeIcons[BadgeVariant.Partial] = LockIcon` — the record's type makes a variant without a mark fail to compile.
- [ ] **Step 3**: the Kit's Badge section shows `Grafo parziale` beside `Da 2`.
- [ ] **Step 4**: `pnpm typecheck && pnpm scan` → green. Commit `feat(ui): the Badge learns the partial state` and push.

### Task 2: The graph's fixtures

**Files:** Create `ui/src/lib/ipc/fixtures/graph.ts`, `graph.test.ts`; Modify `fixtures/art.ts` (achievement and item globs), `fixtures/index.ts`

**Interfaces:** Produces `graphAnswers({ withArt, withCatalog }): { unlock: UnlockView; steps: NextSteps }`; `packIconUrl(url: string | null): string | null`; handlers for `Command.Unlock` and `Command.NextSteps`; `?catalog=none`.

- [ ] **Step 1: Failing test** — `graph.test.ts`:
  - `graphAnswers({ withArt: false, withCatalog: true })`: 641 nodes, `totals` `{ slots: 642, done: 387, known: 637, unknown: 4 }`, diagnostics `[{ kind: 'slotsBeyondCatalog', count: 4 }]`, steps' ids `[484, 488, 489, 479, 480]`, every `iconUrl` null;
  - `withCatalog: false`: no nodes, no steps, a `noCatalog` diagnostic, totals as `ipc::unlock_view` gives them without a catalog (read `crates/ipc/src/graph.rs` for that branch and mirror it, citing the function in a comment);
  - `packIconUrl('isaac://achievement/1')` ends with `/0001_you_unlocked_magdalene.png`; `packIconUrl('isaac://item/familiar/73')` ends with `/familiar_0073_` + the pack's slug; an unknown link and `null` give `null`.
- [ ] **Step 2**: run it → FAIL (no module).
- [ ] **Step 3**: `art.ts` gains globs over `images/achievement/*.png` and `images/items/*.png` and `packIconUrl` (a regex over `isaac://achievement/N` and `isaac://item/<kind>/<id>`, the id padded to four digits); `graph.ts` reads the two payloads with `import.meta.glob(…/contracts/payload/{unlock,next_steps}.json, { eager: true, import: 'default' })`, maps every `achievement.iconUrl` and item target's `iconUrl` through `packIconUrl` or `null`, and builds the no-catalog answer; `index.ts` answers `unlock` and `next_steps` with an active profile and refuses `noActiveProfile` otherwise, reading `?art=none` and `?catalog=none`.
- [ ] **Step 4**: `pnpm --filter ui exec vitest run src/lib/ipc` → green; typecheck, scan. Commit `feat(ui): the graph's fixtures, the design pack's real payloads` and push.

### Task 3: A node's state and its why

**Files:** Create `ui/src/lib/graph/nodeState.ts`, `nodeState.test.ts`

**Interfaces:** Produces `NodeState { Done, Now, Blocked, Partial }`; `nodeState(node): NodeState`; `stateCounts(nodes): Record<NodeState, number>`; `RequirementKind` order `[character, boss, challenge, item, gate, unknown]`; `missingGroups(node): { kind: RequirementKind; names: string[] }[]`.

- [ ] **Step 1: Failing test**, expectations from the spec:
  - synthetic nodes: done + computed → `Done`; done + partial → `Done`; not done + computed + `availableNow` → `Now`; not done + computed + not available → `Blocked`; not done + partial → `Partial`; an `unknown` achievement follows the same rules;
  - `stateCounts` on `graphAnswers(…).unlock.nodes` → `{ done: 387, now: 119, blocked: 117, partial: 18 }`;
  - `missingGroups` of a node missing `[unknown "Collect", character "The Lost", unknown "ending"]` → `[{ character, ["The Lost"] }, { unknown, ["Collect", "ending"] }]`; empty `missing` → `[]`.
- [ ] **Step 2**: run → FAIL. **Step 3**: implement with an exhaustive `switch` on `graph.kind`, `groupBy` from lodash-es for the groups, ordered by the kind list, empty groups dropped. **Step 4**: run → green; typecheck; commit `feat(ui): a node's state and why, one answer for every screen` and push.

### Task 4: Facets, search and sort

**Files:** Create `ui/src/lib/graph/unlockFilter.ts`, `unlockFilter.test.ts`

**Interfaces:** Produces `FacetId { State, Unlocks, Origin, Character }`; `UnlockKind { Passive, Active, Familiar, Trinket, Character, Boss, Challenge, Nothing }`; `OriginValue { Rebirth, Afterbirth, AfterbirthPlus, Repentance, None }`; `UnlockSort { FanOut, Steps, Name }`; `UnlockFilter { query: string; picks: Record<FacetId, string[]> }`; `emptyFilter()`; `facetValues(node, facet): string[]`; `matchesFilter(node, filter): boolean`; `facetCounts(nodes, filter, facet): Map<string, number>`; `facetOptions(nodes, facet): string[]`; `sortNodes(nodes, sort): UnlockNode[]`; `activeFilterCount(filter): number`.

- [ ] **Step 1: Failing test**:
  - `facetValues` — a node unlocking a passive and a character → `['passive', 'character']` for Unlocks; unlocking nothing → `['nothing']`; `origin: null` → `['none']`; required characters are the names of `missing` entries of kind `character`;
  - `matchesFilter` — two values in one facet match either (any); values in two facets must both hold (all); the query matches the text, the hint, an unlocked target's name, case-insensitive; an empty filter matches everything;
  - `facetCounts` — ignores its own facet's picks and applies the others, counts a node once per value; on the payload with an empty filter, Unlocks `nothing` → **231**, Origin `none` → **274**; with State picked `now`, the Origin counts sum to **119** (each node has one origin value);
  - `facetOptions` — State and Unlocks and Origin in their fixed orders, Character sorted by name, **10** characters on the payload;
  - `sortNodes` — fan-out highest first; steps lowest `blockedBy` first with `Partial` and `Done` after; name alphabetical, unknown achievements last; ties keep slot order.
- [ ] **Step 2**: run → FAIL. **Step 3**: implement with lodash-es (`sortBy`, `countBy`, `uniq`), the orders as `as const` arrays, the state through `nodeState`. **Step 4**: green; commit `feat(ui): Unlock's facets, search and sort, as pure functions` and push.

### Task 5: The store and Next steps

**Files:** Create `ui/src/stores/graph.ts`, `ui/src/components/graph/AchievementArt.vue`, `NodeStateBadge.vue`, `TargetLabel.vue`, `ui/src/screens/NextStepsScreen.vue`, `ui/src/screens/nextSteps/StepCard.vue`, `ui/src/assets/theme/aspect.css`; Modify `main.css` (import), `spacing.css`, `StoreId`, `routes.ts`, `routeTable.ts`, `it.ts`, `en.ts`

- [ ] **Step 1: tokens** — `--aspect-achievement: 263 / 176` in `theme/aspect.css` (imported from `main.css`), `--spacing-achievement-thumb: 48px`.
- [ ] **Step 2: store** — `useGraphStore` (`StoreId.Graph`): `unlock`, `steps`, `status`, `error`; `load()` clears both, then `Promise.all([unlock(), nextSteps()])`, keeping `isIpcError(e) ? e : null` on failure.
- [ ] **Step 3: components** — `AchievementArt` (props `url`, `size: ArtSize { Thumb, Card }`): the mark paper, the drawing `object-contain` at `w-achievement-thumb` or `w-achievement` with `aspect-achievement`, a `hatch-placeholder` box when there is no URL or it fails; `NodeStateBadge` (prop `node`): the badge variant and label of Decision 1 ("bloccato da N" composed from the number and a message), inside a `Tooltip` listing `missingGroups` under their headings, nothing when there is nothing missing; `TargetLabel` (prop `target`): an item's sprite through `PixelSprite` at `size-8`, then the name and the kind's message.
- [ ] **Step 4: screen** — `NextStepsScreen`: loads on the active profile's id; `ScreenHeader` and the intro; one `StepCard` per step (rank, `AchievementArt` card size, text, a `Tag` badge per target, the fan-out in `text-state-now-foreground` over "sblocca", `NodeStateBadge`); empty with `noCatalog` → `Alert`, empty otherwise → `EmptyCategory`; `ProfileError` on failure; `Skeleton`s while loading.
- [ ] **Step 5: i18n and route** — `graph.*` and `nextSteps.*` in both languages; `RouteName.NextSteps` maps to the screen and loses its `routeArrives` entry.
- [ ] **Step 6: verify** — typecheck, lint, format, scan, `ui:test`; commit `feat(ui): Next steps, the five things worth doing now` and push.

### Task 6: Unlock

**Files:** Create `ui/src/screens/UnlockScreen.vue`, `ui/src/screens/unlock/StateCards.vue`, `FacetDrawer.vue`, `UnlockToolbar.vue`, `UnlockTable.vue`, `UnlockDiagnostics.vue`, `ui/src/screens/unlock/unlockLayout.ts`, `unlockLayout.test.ts`; Modify `ui/package.json`, `pnpm-lock.yaml`, `spacing.css`, `utilities.css`, `routes.ts`, `routeTable.ts`, `it.ts`, `en.ts`

- [ ] **Step 1: install** — `pnpm --filter ui add @tanstack/vue-virtual@3.13.37`.
- [ ] **Step 2: the row height, pinned** — `unlockLayout.ts` exports `unlockRowHeight = 40`; its test reads `theme/spacing.css` raw and expects `--spacing-row-wide: 40px`, so the virtualizer's arithmetic and the row's class can't drift apart. Run → FAIL, write, → green.
- [ ] **Step 3: tokens** — `--spacing-unlock-art: 64px`, `--spacing-unlock-state: 136px`, `--spacing-unlock-fan: 56px`, `--spacing-unlock-body: 560px`; `@utility grid-cols-unlock` composing art, three `minmax(0, fr)` columns (1.5, 1.1, 1), state and fan.
- [ ] **Step 4: parts** — `StateCards` (four cards, count and label, each toggling its state in the filter, pressed when picked); `FacetDrawer` (`CardCollapsible` closed by default, title with `activeFilterCount`, "Azzera", one column per facet: a `Checkbox` per `facetOptions` value with its label and `facetCounts` count, a value with count 0 and unpicked is disabled); `UnlockToolbar` (the rows count, a search `Input`, a `ToggleGroup` single for the sort, the active picks as `Tag` badges with a close button); `UnlockTable` (header row; a scroll container `max-h-unlock-body overflow-auto`; `useVirtualizer(computed(() => ({ count, getScrollElement, estimateSize: () => unlockRowHeight, overscan: 8 })))`; a spacer `h-(--unlock-total)` with rows positioned by `translate-y-(--row-start)` bound from the template; each row `grid-cols-unlock h-row-wide`: art, text and slot, first target and "+N", hint or `EmptyValue`, `NodeStateBadge`, fan-out or "—"); `UnlockDiagnostics` (`noCatalog`, `noAchievementSection` as `Alert`s; the other two as one line).
- [ ] **Step 5: screen** — `UnlockScreen`: loads on the profile id; the filter and sort in `ref`s; `filtered = sortNodes(nodes.filter(matchesFilter), sort)`; header, cards, drawer, toolbar, table or the no-results `EmptyCategory` with "Azzera i filtri"; `ProfileError`, `Skeleton`s.
- [ ] **Step 6: i18n and route** — `unlock.*` in both languages; `RouteName.Unlock` maps to the screen, its placeholder entry goes; `placeholder.graph` stays for Plan.
- [ ] **Step 7: verify** — typecheck, lint, format, scan, `ui:test`; commit `feat(ui): Unlock, every node, filterable and virtualized` and push.

### Task 7: Looked at, handed on, checked

- [ ] **Step 1: visual check** — `pnpm ui:dev` and a throwaway headless-Chrome script outside the repository (DevTools protocol, as in 3.2): Next steps with and without `?catalog=none`; Unlock's four counts, a partial badge beside a blocked one, the why tooltip, a facet count changing when another facet is picked, the search, no results, scrolling to the last row (the last slot's text is in the DOM only after scrolling), `?art=none`.
- [ ] **Step 2: documents** — `docs/STATUS.md` (3.3 split, 3.3a ticked, session log), `docs/frontend-conventions.md` (`lib/graph/`, `components/graph/`, `aspect.css`, the Badge extension, TanStack Virtual), `CLAUDE.md` (stack: Virtual now, Table deferred; State paragraph), the spec's deviations, `DESIGN-BRIEF.md` question 4 answered.
- [ ] **Step 3: production build** — no fixtures, no pack payloads or images in `ui/dist`.
- [ ] **Step 4: `sh scripts/check`** → all green. Commit `docs: cycle 3.3a lands, Next steps and Unlock` and push.
