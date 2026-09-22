# Obiettivi e il Piano — one screen, two panes

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this
> plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Merge *Obiettivi* and *Piano* into one screen at `/progress/goals`, with recommendations
and the want on the left and the queue on the right, and replace the eleven-field queue row with
one that carries five things and opens the rest.

**Architecture:** Every decision the row makes moves into one pure module, `lib/plan/rowModel.ts`,
which both panes' rows read; the `.vue` files draw it and decide nothing. The queue's move rules,
the drag, the diagnostics and every store are untouched. `RouteName.Plan` leaves the table, a bare
path redirect keeps the URL alive, and a retired-name map in `sessionDocument.ts` carries a stored
tab across.

**Tech Stack:** Vue 3 + TypeScript, Vitest, Tailwind v4, shadcn-vue on Reka UI (`Collapsible` for
the expansion), `@lucide/vue`.

**Spec:** `docs/superpowers/specs/2026-09-22-goals-plan-merge-design.md`

## Global Constraints

Copied from `CLAUDE.md` and `docs/frontend-conventions.md`; every task's requirements include them.

- **No `<style>` in SFCs.** Dynamic values come from CSS variables bound by the template.
- **No hardcoded visual constants.** No `w-[48px]`, no `opacity-50`, no `:size="16"`: every value
  is a token in `@theme`. A new token goes in the right file under `ui/src/assets/theme/`.
- **No `invoke()` in components** — only the typed wrappers in `ui/src/lib/ipc/`.
- **No raw `<button>` / `<input>`** — use `ui/src/components/ui/`.
- **No string unions.** `const X = { … } as const`, never `type X = 'a' | 'b'`.
- **Exhaustiveness is mandatory**: no `_ =>`/`default:` arm that swallows a variant. Closed unions
  end in `assertNever`.
- **Test-first.** The expected value comes from the spec, never from the code's current output. A
  failing test is first a hypothesis of a bug in the code.
- **No Rust.** If a task seems to need a wire change, stop: the spec's §7 says everything is
  already there, and a contract change is a different cycle.
- **Every commit** is Conventional Commits, `type(scope): subject`, scope `ui`. **Never** a
  `Co-Authored-By` trailer or any reference to Claude.
- **Stage by explicit path.** Never `git add -A`: other sessions edit `docs/superpowers/` in
  parallel.
- Before declaring the whole thing done: `pnpm check`.

## File Structure

| File | Responsibility |
|---|---|
| `ui/src/lib/plan/rowModel.ts` **(new)** | Every decision a row makes: its text, art, page, fan-out, whether it is playable, its state, its condition. Replaces `lib/graph/goalCard.ts`. |
| `ui/src/lib/plan/rowModel.test.ts` **(new)** | The above, test-first. |
| `ui/src/lib/plan/queueExtras.ts` **(new)** | The three things only a *queue* row has: wanted, the wishes it serves, how many of its steps are outside the queue. |
| `ui/src/lib/plan/queueExtras.test.ts` **(new)** | The above. |
| `ui/src/components/plan/GoalRow.vue` **(new)** | The shared row. Five things in the row, the rest behind a `Collapsible`. Draws a model, decides nothing. |
| `ui/src/kit/sections/app/GoalRowSection.vue` **(new)** | The row's states on the Kit page — the only way its presentation gets looked at. |
| `ui/src/lib/plan/addPaneState.ts` **(new)** | Which of the left pane's four states is showing — decided on the want and the catalog, never on a list's length. |
| `ui/src/lib/plan/addPaneState.test.ts` **(new)** | The above. |
| `ui/src/screens/goals/AddPane.vue` **(new)** | The left pane: search bar, then the sections, the want's answer, the missing-game alert, or nothing to unlock. |
| `ui/src/screens/goals/GoalsHero.vue` **(new)** | The band the screen opens on — the title on a lit ground, and nothing countable (spec §4.2). |
| `ui/src/screens/GoalsScreen.vue` | Becomes the banded two-pane screen. |
| `docs/frontend-conventions.md` | Gains the third screen shape; Completion moves into it, where it has belonged since card #58. |
| `ui/src/screens/plan/QueueCard.vue` | Keeps the drag; draws `GoalRow` instead of `QueueRow`. |
| `ui/src/lib/window/sessionDocument.ts` | Gains the retired-name map. |
| `ui/src/router/routeTable.ts`, `routes.ts`, `components/shell/sectionNav.ts` | `Plan` leaves; `/progress/plan` redirects. |
| **Deleted** | `screens/PlanScreen.vue`, `screens/plan/ProposalAside.vue`, `screens/plan/QueueRow.vue`, `screens/goals/GoalCard.vue`, `lib/graph/goalCard.ts` (+ its test), `lib/plan/planNow.ts` (+ its test), and `queueSummary` from `lib/plan/queueRows.ts`. |

---

### Task 1: the row's decisions, in one pure module

The queue row and the recommendation card disagree today about what a row is *called*:
`QueueRow.vue` uses `knownText(node)` — the achievement's own sentence — while `GoalCard.vue` uses
the names of what it unlocks, because the file names a Tainted character by its base form (B28,
B32). One screen means one answer, and the reasoned one wins.

**Files:**
- Create: `ui/src/lib/plan/rowModel.ts`
- Test: `ui/src/lib/plan/rowModel.test.ts`
- Read first: `ui/src/lib/graph/goalCard.ts`, `ui/src/lib/graph/nodeState.ts`

**Interfaces:**
- Consumes: `UnlockNode` from `@/lib/ipc/types`; `nodeState`, `NodeState` from
  `@/lib/graph/nodeState`; `pageLocation` from `@/lib/wiki/category`; `targetName` from
  `@/lib/graph/characterName`.
- Produces: `RowModel` and `rowModel(node, t): RowModel`, used by Tasks 2, 3 and 4.

- [ ] **Step 1: Write the failing tests**

Create `ui/src/lib/plan/rowModel.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { NodeState } from '@/lib/graph/nodeState'
import type { UnlockNode } from '@/lib/ipc/types'
import { rowModel } from './rowModel'

const t = ((key: string, params?: Record<string, unknown>) =>
  params ? `${key}:${String(params.name)}` : key) as never

const base: UnlockNode = {
  achievement: {
    kind: 'known',
    id: 484,
    text: 'You unlocked "The Lost"',
    condition: 'Arriva a Home e usa la Red Key',
    iconUrl: 'isaac://achievement/484',
  },
  done: false,
  unlocks: [],
  origin: null,
  missing: [],
  graph: {
    kind: 'computed',
    availableNow: true,
    blockedBy: 0,
    fanOut: 23,
    stepsMissing: 0,
  },
}

describe('what a row is called', () => {
  // B28: the file writes the base name for both forms. What you get comes first; the
  // achievement's own sentence is only the fallback. The queue used to use the sentence,
  // which is the disagreement this module ends.
  it('leads with what you get, not with what the file says', () => {
    const model = rowModel(
      {
        ...base,
        unlocks: [
          { kind: 'character', id: 31, name: 'The Lost', tainted: true, page: null },
        ],
      },
      t,
    )
    expect(model.text).not.toBe('You unlocked "The Lost"')
  })

  it('falls back to the achievement text when it unlocks nothing catalogued', () => {
    expect(rowModel(base, t).text).toBe('You unlocked "The Lost"')
  })
})

describe('what the row shows without being opened', () => {
  it('carries the fan-out as a number, never a sentence', () => {
    expect(rowModel(base, t).fanOut).toBe(23)
  })

  // §4.1: amber means "you can do this tonight". It is the graph's `availableNow`, never a
  // guess from `blockedBy` being zero.
  it('is playable when the graph says it is available now', () => {
    expect(rowModel(base, t).playable).toBe(true)
  })

  it('is not playable when something is in the way', () => {
    const blocked = {
      ...base,
      graph: { ...base.graph, availableNow: false, blockedBy: 2 },
    } as UnlockNode
    expect(rowModel(blocked, t).playable).toBe(false)
  })

  // The guard §4.1 asks for: the colour is never the only carrier, so the state has to be
  // here as a value the expansion can print as a word.
  it('carries the state as a value, not only as the playable flag', () => {
    expect(rowModel(base, t).state).toBe(NodeState.Now)
    const blocked = {
      ...base,
      graph: { ...base.graph, availableNow: false, blockedBy: 2 },
    } as UnlockNode
    expect(rowModel(blocked, t).state).toBe(NodeState.Blocked)
  })

  it('never invents a condition the file does not state', () => {
    const silent = {
      ...base,
      achievement: { ...base.achievement, condition: null },
    } as UnlockNode
    expect(rowModel(silent, t).condition).toBeNull()
  })

  it('has no page for a slot the catalog cannot name', () => {
    const unknown = {
      ...base,
      achievement: { kind: 'unknown', slot: 611 },
    } as UnlockNode
    expect(rowModel(unknown, t).location).toBeNull()
    expect(rowModel(unknown, t).art).toBeNull()
  })
})
```

- [ ] **Step 2: Run the tests and watch them fail**

Run: `cd ui && pnpm exec vitest run src/lib/plan/rowModel.test.ts`
Expected: FAIL — `Failed to resolve import "./rowModel"`.

- [ ] **Step 3: Write the module**

Create `ui/src/lib/plan/rowModel.ts`:

```ts
import { targetName } from '@/lib/graph/characterName'
import { NodeState, nodeState } from '@/lib/graph/nodeState'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import type { UnlockNode } from '@/lib/ipc/types'
import { pageLocation } from '@/lib/wiki/category'
import type { TabLocation } from '@/router/routeTable'

type Translate = (
  key: MessageKey<MessageSchema>,
  params?: Record<string, unknown>,
) => string

// More than one thing out of one achievement is rare and real — a challenge's rewards — so the
// names are listed, never counted.
const Separator = ' · '

// Everything a row decides, for both panes. The components draw this and decide nothing, which
// is what makes the decisions testable (DESIGN-BRIEF.md §7.1).
export interface RowModel {
  /** What you get. The achievement's own sentence only when it unlocks nothing catalogued. */
  text: string
  /** The game's own `unlock_condition`. `null` when the file states none. */
  condition: string | null
  art: string | null
  /** The raw count: the sentence around it belongs to the component and its messages. */
  fanOut: number
  /** Amber, and only that: whether it can be played tonight. */
  playable: boolean
  /** The same reading as a value, so the colour is never the only carrier (spec §4.1). */
  state: NodeState
  /** The achievement's page. `null` for a slot the catalog cannot name. */
  location: TabLocation | null
}

export const rowModel = (node: UnlockNode, t: Translate): RowModel => {
  const a = node.achievement
  const known = a.kind === 'known'
  const unlocked = node.unlocks.map((u) => targetName(t, u)).join(Separator)
  const fallback = known
    ? a.text
    : `${t('graph.unknownAchievement')} · ${t('graph.slot')} ${a.slot}`
  const state = nodeState(node)
  return {
    text: unlocked === '' ? fallback : unlocked,
    condition: known ? a.condition : null,
    art: known ? a.iconUrl : null,
    fanOut: node.graph.fanOut,
    playable: state === NodeState.Now,
    state,
    location: known ? pageLocation({ kind: 'achievement', id: a.id }) : null,
  }
}
```

- [ ] **Step 4: Run the tests and watch them pass**

Run: `cd ui && pnpm exec vitest run src/lib/plan/rowModel.test.ts`
Expected: PASS, 8 tests.

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/plan/rowModel.ts ui/src/lib/plan/rowModel.test.ts
git commit -m "feat(ui): one module decides what a row says"
```

---

### Task 2: the three things only a queue row has

**Files:**
- Create: `ui/src/lib/plan/queueExtras.ts`
- Test: `ui/src/lib/plan/queueExtras.test.ts`
- Read first: `ui/src/lib/plan/queueRows.ts` (`originRows`, `rowId`, `knownText`)

**Interfaces:**
- Consumes: `QueueRow` from `@/lib/ipc/types`; `originRows`, `rowId` from `./queueRows`;
  `rowModel` from `./rowModel` (Task 1).
- Produces: `QueueExtras` and `queueExtras(row, rows, t): QueueExtras`, used by Tasks 3 and 4.

- [ ] **Step 1: Write the failing tests**

Create `ui/src/lib/plan/queueExtras.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import type { QueueRow, UnlockNode } from '@/lib/ipc/types'
import { queueExtras } from './queueExtras'

const t = ((key: string) => key) as never

const node = (id: number, text: string): UnlockNode =>
  ({
    achievement: { kind: 'known', id, text, condition: null, iconUrl: null },
    done: false,
    unlocks: [],
    origin: null,
    missing: [],
    graph: {
      kind: 'computed',
      availableNow: true,
      blockedBy: 0,
      fanOut: 0,
      stepsMissing: 0,
    },
  }) as UnlockNode

const row = (
  id: number,
  text: string,
  over: Partial<QueueRow> = {},
): QueueRow =>
  ({
    node: node(id, text),
    wanted: false,
    origins: [],
    stepsNotQueued: 0,
    ...over,
  }) as QueueRow

describe('why a row is in the queue', () => {
  it('names the wish it serves, by that wish’s own text', () => {
    const polaroid = row(1, 'Polaroid', { wanted: true })
    const heart = row(2, 'Cuore di Isaac', { origins: [1] })
    expect(queueExtras(heart, [polaroid, heart], t).serves).toEqual([
      { id: 1, text: 'Polaroid' },
    ])
  })

  // A wish can have left the queue between the write and the read: the row still says it
  // serves something, and says it by id rather than by nothing.
  it('falls back to the id when the wish is not in the queue', () => {
    const orphan = row(2, 'Cuore di Isaac', { origins: [99] })
    expect(queueExtras(orphan, [orphan], t).serves).toEqual([
      { id: 99, text: null },
    ])
  })

  it('serves nothing when you asked for it yourself', () => {
    const own = row(1, 'Polaroid', { wanted: true })
    const extras = queueExtras(own, [own], t)
    expect(extras.wanted).toBe(true)
    expect(extras.serves).toEqual([])
  })

  it('carries how many of its steps are still outside the queue', () => {
    const r = row(3, 'Negativo', { stepsNotQueued: 2 })
    expect(queueExtras(r, [r], t).stepsNotQueued).toBe(2)
  })
})
```

- [ ] **Step 2: Run the tests and watch them fail**

Run: `cd ui && pnpm exec vitest run src/lib/plan/queueExtras.test.ts`
Expected: FAIL — `Failed to resolve import "./queueExtras"`.

- [ ] **Step 3: Write the module**

Create `ui/src/lib/plan/queueExtras.ts`:

```ts
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import type { QueueRow } from '@/lib/ipc/types'
import { knownText, originRows } from './queueRows'

type Translate = (
  key: MessageKey<MessageSchema>,
  params?: Record<string, unknown>,
) => string

/** A wish this row serves. `text` is `null` when that wish is no longer in the queue. */
export interface Serves {
  id: number
  text: string | null
}

// What a queue row has and a recommendation does not: you asked for it, it serves somebody
// else's wish, and some of what it still needs is outside the queue.
export interface QueueExtras {
  wanted: boolean
  serves: Serves[]
  stepsNotQueued: number
}

export const queueExtras = (
  row: QueueRow,
  rows: QueueRow[],
  _t: Translate,
): QueueExtras => ({
  wanted: row.wanted,
  serves: originRows(row, rows).map((origin) => ({
    id: origin.id,
    text: origin.row ? knownText(origin.row.node) : null,
  })),
  stepsNotQueued: row.stepsNotQueued,
})
```

> `_t` is kept in the signature because the component's call site already has it and the next
> task's copy would otherwise change shape. If it is still unused when Task 5 ends, delete it
> then — an unused parameter that survives a cycle is a lie about what the function needs.

- [ ] **Step 4: Run the tests and watch them pass**

Run: `cd ui && pnpm exec vitest run src/lib/plan/queueExtras.test.ts`
Expected: PASS, 4 tests.

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/plan/queueExtras.ts ui/src/lib/plan/queueExtras.test.ts
git commit -m "feat(ui): why a row is in the queue, decided in one place"
```

---

### Task 3: the shared row, drawn

Five things in the row, everything else behind the disclosure. Presentation is not unit-tested in
this repo — it is looked at on the Kit page — so this task's deliverable is the component **and**
its Kit section, and the gate is your own eyes plus `pnpm typecheck` and `pnpm scan`.

**Files:**
- Create: `ui/src/components/plan/GoalRow.vue`
- Create: `ui/src/kit/sections/app/GoalRowSection.vue`
- Modify: `ui/src/kit/KitPage.vue` (register the section)
- Modify: `ui/src/i18n/messages/it.ts` and `ui/src/i18n/messages/en.ts`
- Read first: `ui/src/screens/plan/QueueRow.vue`, `ui/src/components/ui/collapsible/index.ts`

**Interfaces:**
- Consumes: `RowModel` (Task 1), `QueueExtras` (Task 2), `NodeStateBadge`, `AchievementArt`,
  `Collapsible`/`CollapsibleTrigger`/`CollapsibleContent`, `Badge`, `Button`.
- Produces: `GoalRow.vue` with props
  `{ model: RowModel; node: UnlockNode; extras?: QueueExtras; position?: number; busy: boolean; dragging?: boolean }`
  — `node` is there for `NodeStateBadge`, which builds the *why* menu from the node's own
  requirements and cannot be fed a model — and emits `{ grab: [PointerEvent]; step: [KeyboardEvent]; remove: []; add: []; navigate: [TabLocation, boolean] }`.
  A row with `position` draws the grip and the number; one without draws `+` instead of the
  chevron's neighbour. Used by Tasks 4 and 5.

- [ ] **Step 1: Add the messages both panes need**

In `ui/src/i18n/messages/it.ts`, inside the existing `plan:` block, replace the `summary:` object
(which §5 of the spec deletes) with:

```ts
    opens: 'apre {count}',
    opensNothing: 'non apre altro',
    detail: 'Mostra il dettaglio',
    queueCount: 'La tua coda',
    addPane: 'Da aggiungere',
```

Mirror the same five keys in `ui/src/i18n/messages/en.ts`:

```ts
    opens: 'opens {count}',
    opensNothing: 'opens nothing',
    detail: 'Show the detail',
    queueCount: 'Your queue',
    addPane: 'To add',
```

- [ ] **Step 2: Write the row**

Create `ui/src/components/plan/GoalRow.vue`. The row is `grip · position · art · text · apre N ·
chevron`; `extras`, the condition and the state's word live in `CollapsibleContent`. `apre N` takes
`text-state-now-foreground` when `model.playable` and `text-subtle-foreground` otherwise — and the
state is **also** printed as a word inside, via `NodeStateBadge`, so the colour is never the only
carrier. The whole row is not a link: the text is, because a card that is itself a link and
contains a button is a trap for the keyboard.

```vue
<script setup lang="ts">
import { ChevronDownIcon, GripVerticalIcon, ListPlusIcon, XIcon } from '@lucide/vue'
import { computed } from 'vue'
import AchievementArt from '@/components/graph/AchievementArt.vue'
import NodeStateBadge from '@/components/graph/NodeStateBadge.vue'
import { ArtSize } from '@/components/graph/artSize'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { UnlockNode } from '@/lib/ipc/types'
import type { QueueExtras } from '@/lib/plan/queueExtras'
import type { RowModel } from '@/lib/plan/rowModel'
import type { TabLocation } from '@/router/routeTable'

const props = defineProps<{
  model: RowModel
  node: UnlockNode
  /** Present only in the queue: it brings the grip and the number with it. */
  position?: number
  extras?: QueueExtras
  busy: boolean
  dragging?: boolean
}>()
const emit = defineEmits<{
  grab: [e: PointerEvent]
  step: [e: KeyboardEvent]
  remove: []
  add: []
  navigate: [location: TabLocation, newTab: boolean]
}>()
const { t } = useMessages()

const opensText = computed(() =>
  props.model.fanOut > 0
    ? t('plan.opens', { count: props.model.fanOut })
    : t('plan.opensNothing'),
)
const open = (newTab: boolean) => {
  if (props.model.location) emit('navigate', props.model.location, newTab)
}
</script>

<template>
  <Collapsible
    :class="cn('flex flex-col', dragging && 'opacity-disabled')"
    v-slot="{ open: shown }"
  >
    <div class="flex items-center gap-3 px-3 py-2.5">
      <Button
        v-if="position !== undefined"
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Icon"
        :aria-label="t('plan.row.move')"
        :disabled="busy"
        class="cursor-grab touch-none"
        @pointerdown="emit('grab', $event)"
        @keydown="emit('step', $event)"
      >
        <GripVerticalIcon />
      </Button>
      <span
        v-if="position !== undefined"
        class="w-4 shrink-0 text-right text-label text-subtle-foreground tabular-nums"
        >{{ position }}</span
      >
      <AchievementArt :url="model.art" :size="ArtSize.Thumb" />
      <Button
        v-if="model.location"
        :variant="ButtonVariant.Ref"
        :size="ButtonSize.Inline"
        class="min-w-0 flex-1 justify-start truncate text-row"
        @click="open($event.ctrlKey)"
        >{{ model.text }}</Button
      >
      <span v-else class="min-w-0 flex-1 truncate text-row text-foreground">{{
        model.text
      }}</span>
      <span
        :class="
          cn(
            'shrink-0 text-caption tabular-nums',
            model.playable
              ? 'text-state-now-foreground'
              : 'text-subtle-foreground',
          )
        "
        >{{ opensText }}</span
      >
      <Button
        v-if="extras === undefined"
        :variant="ButtonVariant.Outline"
        :size="ButtonSize.IconCompact"
        :aria-label="t('queue.add')"
        :disabled="busy"
        @click="emit('add')"
      >
        <ListPlusIcon />
      </Button>
      <CollapsibleTrigger as-child>
        <Button
          :variant="ButtonVariant.Ghost"
          :size="ButtonSize.IconCompact"
          :aria-label="t('plan.detail')"
        >
          <ChevronDownIcon :class="cn(shown && 'rotate-180')" />
        </Button>
      </CollapsibleTrigger>
    </div>
    <CollapsibleContent class="flex flex-col items-start gap-2 px-3 pb-3 pl-row-detail">
      <span v-if="model.condition" class="text-caption text-subtle-foreground">{{
        model.condition
      }}</span>
      <div class="flex flex-wrap items-center gap-1.5">
        <NodeStateBadge :node="node" />
        <Badge v-if="extras?.wanted" :variant="BadgeVariant.Wanted">{{
          t('plan.row.wanted')
        }}</Badge>
        <Badge
          v-for="s in extras?.serves ?? []"
          :key="s.id"
          :variant="BadgeVariant.Tag"
          >{{ t('plan.row.serves') }} «{{ s.text ?? `${t('plan.achievement')} ${s.id}` }}»</Badge
        >
        <Badge v-if="(extras?.stepsNotQueued ?? 0) > 0" :variant="BadgeVariant.Tag"
          >{{ t('plan.row.outsideQueue') }}: {{ extras?.stepsNotQueued }}</Badge
        >
      </div>
      <Button
        v-if="extras?.wanted"
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Compact"
        :disabled="busy"
        @click="emit('remove')"
        ><XIcon />{{ t('queue.removeShort') }}</Button
      >
    </CollapsibleContent>
  </Collapsible>
</template>
```

- [ ] **Step 3: Add the one token the row needs**

`pl-row-detail` above is not a Tailwind default and `pnpm scan` will refuse a bracket value. Add to
`ui/src/assets/theme/spacing.css`, beside the other row tokens:

```css
  /* Where a row's expansion starts: under the text, not under the grip. Measured off the row
     itself — grip (2.5rem) + position (1rem) + gap (0.75rem) + art (2rem) + gap (0.75rem). */
  --spacing-row-detail: 7rem;
```

- [ ] **Step 4: Write the Kit section**

Five states, because these are the five that can be got wrong and none of them has a unit test.
Create `ui/src/kit/sections/app/GoalRowSection.vue`:

```vue
<script setup lang="ts">
import KitSection from '@/kit/KitSection.vue'
import GoalRow from '@/components/plan/GoalRow.vue'
import { Card } from '@/components/ui/card'
import { NodeState } from '@/lib/graph/nodeState'
import type { UnlockNode } from '@/lib/ipc/types'
import type { QueueExtras } from '@/lib/plan/queueExtras'
import type { RowModel } from '@/lib/plan/rowModel'

// Fixtures, not data: the row draws a model, so the Kit hands it models directly and never
// needs a profile, a catalog or the graph.
const node = { achievement: { kind: 'unknown', slot: 1 }, done: false, unlocks: [],
  origin: null, missing: [],
  graph: { kind: 'computed', availableNow: true, blockedBy: 0, fanOut: 7, stepsMissing: 0 },
} as UnlockNode

const model = (over: Partial<RowModel> = {}): RowModel => ({
  text: 'Cuore di Isaac',
  condition: 'Sconfiggi Mom’s Heart 10 volte',
  art: null,
  fanOut: 7,
  playable: true,
  state: NodeState.Now,
  location: null,
  ...over,
})

const extras = (over: Partial<QueueExtras> = {}): QueueExtras => ({
  wanted: false,
  serves: [],
  stepsNotQueued: 0,
  ...over,
})

const rows: { label: string; model: RowModel; extras?: QueueExtras; position?: number }[] = [
  { label: 'coda · giocabile ora', model: model(), extras: extras(), position: 1 },
  {
    label: 'coda · bloccata',
    model: model({ playable: false, state: NodeState.Blocked }),
    extras: extras({ stepsNotQueued: 2 }),
    position: 2,
  },
  {
    label: 'coda · chiesta, serve a due',
    model: model({ text: 'Polaroid' }),
    extras: extras({
      wanted: true,
      serves: [
        { id: 1, text: 'Il Negativo' },
        { id: 2, text: null },
      ],
    }),
    position: 3,
  },
  {
    label: 'coda · il file non dice come si sblocca',
    model: model({ condition: null }),
    extras: extras(),
    position: 4,
  },
  { label: 'consiglio · nessun grip, il più al posto suo', model: model({ fanOut: 12 }) },
]
</script>

<template>
  <KitSection title="GoalRow">
    <div v-for="row in rows" :key="row.label" class="flex flex-col gap-1">
      <span class="text-label text-subtle-foreground">{{ row.label }}</span>
      <Card class="flex-col gap-0 p-0">
        <GoalRow
          :model="row.model"
          :node="node"
          :extras="row.extras"
          :position="row.position"
          :busy="false"
        />
      </Card>
    </div>
  </KitSection>
</template>
```

Check `ui/src/kit/KitSection.vue`'s actual prop name before committing — if it is not `title`, use
what it is rather than adding one.

- [ ] **Step 5: Register it and look at it**

Add `GoalRowSection` to `ui/src/kit/KitPage.vue` the way the other `app/` sections are registered.

Run: `pnpm ui:dev`, open `http://localhost:1420/#kit`, find the section.
**Do not skip this.** Check with your eyes: the five things are in the row and nothing else is;
`apre N` is amber on the playable row and grey on the blocked one; the expansion opens under the
text and not under the grip; the chevron turns.

- [ ] **Step 6: Typecheck, lint, scan**

Run: `pnpm typecheck && pnpm lint && pnpm scan`
Expected: clean. A `scan` violation here is almost always a hardcoded visual constant — fix it with
a token, never with an exemption.

- [ ] **Step 7: Commit**

```bash
git add ui/src/components/plan/GoalRow.vue ui/src/kit/sections/app/GoalRowSection.vue \
  ui/src/kit/KitPage.vue ui/src/assets/theme/spacing.css \
  ui/src/i18n/messages/it.ts ui/src/i18n/messages/en.ts
git commit -m "feat(ui): a row carries five things and opens the rest"
```

---

### Task 4: the queue draws the new row

**Files:**
- Modify: `ui/src/screens/plan/QueueCard.vue`
- Delete: `ui/src/screens/plan/QueueRow.vue`
- Read first: `ui/src/screens/plan/QueueCard.vue` — the drag, `useDragList`, `dropAnchor`

**Interfaces:**
- Consumes: `GoalRow` (Task 3), `rowModel` (Task 1), `queueExtras` (Task 2).
- Produces: nothing new. The queue behaves exactly as before.

- [ ] **Step 1: Swap the row**

In `QueueCard.vue`, replace the `QueueRow` import with `GoalRow`, and the element with:

```vue
        <GoalRow
          :model="rowModel(row.node, t)"
          :node="row.node"
          :extras="queueExtras(row, rows, t)"
          :position="index + 1"
          :dragging="drag.moving.value && drag.from.value === index"
          :busy="busy"
          @grab="drag.start(index, $event)"
          @step="onStep(index, $event)"
          @remove="emit('remove', rowId(row))"
        />
```

While you are in this file, give the card's heading the count §5 moved onto it — the number left
the top of the page and has to land here, or it lands nowhere:

```vue
      <CardTitle>{{ t('plan.queueCount') }} {{ rows.length }}</CardTitle>
```

The `DragGhost` copy at the bottom takes the same swap, keeping `:busy="true"` and no `@grab`.
**`data-queue-row` stays on the wrapping `div`** — it is how `rowElements()` finds the boxes, and
moving it onto the component would make the drag measure the wrong box.

- [ ] **Step 2: Delete the old row**

```bash
git rm ui/src/screens/plan/QueueRow.vue
```

- [ ] **Step 3: Run the whole suite**

Run: `pnpm ui:test`
Expected: PASS. **Watch the count**, not just the colour: it must be the previous total plus the
12 added in Tasks 1 and 2. A number lower than that means a test file left in the same breath as
this deletion, which is the exact shape `scripts/test-floor` exists to catch.

- [ ] **Step 4: Look at the queue**

Run: `pnpm dev` (not `ui:dev` — the queue needs a real profile), open the Plan, and drag a row.
The drag, the drop line and the prerequisite wall must behave exactly as before: this task changed
what a row looks like and nothing about how it moves.

- [ ] **Step 5: Commit**

```bash
git add ui/src/screens/plan/QueueCard.vue
git commit -m "refactor(ui): the queue draws the shared row"
```

---

### Task 5: the left pane

**Files:**
- Create: `ui/src/lib/plan/addPaneState.ts`, `ui/src/screens/goals/AddPane.vue`
- Modify: `ui/src/screens/goals/WantAnswer.vue` (draw `GoalRow`)
- Delete: `ui/src/screens/goals/GoalCard.vue`, `ui/src/lib/graph/goalCard.ts`,
  `ui/src/lib/graph/goalCard.test.ts`
- Test: `ui/src/lib/plan/addPaneState.test.ts`
- Read first: `ui/src/screens/GoalsScreen.vue` — the want, the sections and the `noCatalog` alert

**Interfaces:**
- Consumes: `GoalRow` (Task 3), `rowModel` (Task 1), `WantBar`, `WantAnswer`, `sectionTitle`,
  `isQueued` and `canQueue` from `@/lib/plan/queueRows`.
- Produces: `AddPane.vue` with props
  `{ sections: StepsSection[]; queued: Set<number>; canWrite: boolean; busy: boolean; wantActive: boolean; noCatalog: boolean; blocks: WantBlock[]; banner: WantBanner | null }`
  and emits `{ add: [slot: number]; pick: [Target]; clear: []; navigate: [TabLocation, boolean] }`.

- [ ] **Step 1: Write the failing test for the pane's four states**

Spec §10 asks for this one by name: the states are driven by the want, **not** by
`sections.length`. An empty section list and an active want are different pages, and a component
that switched on `length` would draw "niente da sbloccare" over a question you just asked. That
judgment goes in `lib/`, where it can be tested; the component switches on the answer.

Create `ui/src/lib/plan/addPaneState.ts` — declare the enum first so the test can import it —
and `ui/src/lib/plan/addPaneState.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { StepsBasis } from '@/lib/ipc/types'
import type { StepsSection } from '@/lib/ipc/types'
import { AddPaneState, addPaneState } from './addPaneState'

const full: StepsSection[] = [{ basis: StepsBasis.FanOut, steps: [] as never[] }]

describe('what the left pane is showing', () => {
  it('answers the want, whatever the sections hold', () => {
    expect(addPaneState(true, full, false)).toBe(AddPaneState.Want)
    expect(addPaneState(true, [], false)).toBe(AddPaneState.Want)
  })

  it('suggests when nothing was asked and there is something to suggest', () => {
    expect(addPaneState(false, full, false)).toBe(AddPaneState.Sections)
  })

  // The case the component must not reach by counting: no want and no sections is the only
  // page that says "there is nothing to unlock right now".
  it('says there is nothing only when nothing was asked either', () => {
    expect(addPaneState(false, [], false)).toBe(AddPaneState.Nothing)
  })

  // An empty page has two reasons and the sections do not say which (DESIGN-BRIEF §7.3): with
  // no game installed the graph knows nothing, and "there is nothing to unlock" would be a
  // wrong answer rather than a short one.
  it('says the game is missing rather than that there is nothing', () => {
    expect(addPaneState(false, [], true)).toBe(AddPaneState.NoCatalog)
  })

  // A want still wins: you asked a question, and the answer names what it cannot resolve.
  it('still answers the want with no catalog', () => {
    expect(addPaneState(true, [], true)).toBe(AddPaneState.Want)
  })
})
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cd ui && pnpm exec vitest run src/lib/plan/addPaneState.test.ts`
Expected: FAIL — `Failed to resolve import "./addPaneState"`.

- [ ] **Step 3: Write the module**

```ts
import type { StepsSection } from '@/lib/ipc/types'

// What the left pane is showing. A closed set, so the template switches on a value instead of
// on a length — an empty section list and an active want are different pages.
export const AddPaneState = {
  Want: 'want',
  Sections: 'sections',
  NoCatalog: 'noCatalog',
  Nothing: 'nothing',
} as const
export type AddPaneState = (typeof AddPaneState)[keyof typeof AddPaneState]

export const addPaneState = (
  wantActive: boolean,
  sections: StepsSection[],
  noCatalog: boolean,
): AddPaneState => {
  if (wantActive) return AddPaneState.Want
  if (sections.length > 0) return AddPaneState.Sections
  return noCatalog ? AddPaneState.NoCatalog : AddPaneState.Nothing
}
```

`noCatalog` is what `GoalsScreen.vue` already computes —
`graph.view?.unlock.diagnostics.some((d) => d.kind === 'noCatalog')` — passed down rather than
recomputed: two answers to "is the game installed" is two chances to disagree.

- [ ] **Step 4: Run it and watch it pass**

Run: `cd ui && pnpm exec vitest run src/lib/plan/addPaneState.test.ts`
Expected: PASS, 6 tests.

- [ ] **Step 5: Write the pane**

Create `ui/src/screens/goals/AddPane.vue`. `WantBar` is always on top; below it the template
switches on `addPaneState`, with a branch per member and no fallthrough:

```vue
<template>
  <div class="flex w-full min-w-0 flex-col gap-3">
    <span class="text-label text-subtle-foreground">{{ t('plan.addPane') }}</span>
    <WantBar @pick="emit('pick', $event)" @clear="emit('clear')" />

    <WantAnswer
      v-if="state === AddPaneState.Want"
      :blocks="blocks"
      :banner="banner"
      :can-write="canWrite"
      :busy="busy"
      @queue="emit('add', $event)"
      @navigate="(l, n) => emit('navigate', l, n)"
    />

    <template v-else-if="state === AddPaneState.Sections">
      <section
        v-for="section in sections"
        :key="section.basis"
        class="flex flex-col gap-1"
      >
        <h2 class="text-label text-subtle-foreground">
          {{ t(sectionTitle[section.basis]) }}
        </h2>
        <Card class="flex-col gap-0 p-0">
          <div
            v-for="step in section.steps"
            :key="nodeSlot(step)"
            class="border-b border-hairline last:border-b-0"
            :class="isQueued(step, queued) && 'opacity-disabled'"
          >
            <GoalRow
              :model="rowModel(step, t)"
              :node="step"
              :busy="busy || !canWrite || !canQueue(step, queued)"
              @add="emit('add', nodeSlot(step))"
              @navigate="(l, n) => emit('navigate', l, n)"
            />
          </div>
        </Card>
      </section>
      <Button
        :variant="ButtonVariant.Ref"
        :size="ButtonSize.Inline"
        class="self-start"
        @click="emit('navigate', seeAll, $event.ctrlKey)"
        >{{ t('goals.seeAll') }}</Button
      >
    </template>

    <Alert v-else-if="state === AddPaneState.NoCatalog">
      <InfoIcon />
      <AlertTitle>{{ t('goals.noCatalogTitle') }}</AlertTitle>
      <AlertDescription>{{ t('goals.noCatalog') }}</AlertDescription>
    </Alert>

    <EmptyCategory v-else>{{ t('goals.nothingNow') }}</EmptyCategory>
  </div>
</template>
```

Four branches for four members, in the enum's own order. `v-else` on the last is the exhaustive
arm here — a template cannot call `assertNever`, so the guard is that every member above it is
named: adding a fifth without a branch makes it fall into `Nothing`, which is why the enum and
this template are read together.

A row already in the queue stays visible, dimmed, with its `+` disabled — spec §4: the list must
not shuffle under the finger while you add to it. `seeAll` is the same
`{ name: RouteName.Unlock, query: { state: NodeState.Now } }` `GoalsScreen.vue` builds today; move
the constant with it.

- [ ] **Step 6: Delete the card and its model**

`goalCard.ts` is superseded by `rowModel.ts`, which Task 1's tests cover — including the B28 case
that `goalCard.test.ts` guarded, so **that guard is moved, not dropped**.

```bash
git rm ui/src/screens/goals/GoalCard.vue ui/src/lib/graph/goalCard.ts \
  ui/src/lib/graph/goalCard.test.ts
```

- [ ] **Step 7: Run the suite and read the count out loud**

Run: `pnpm ui:test`
Expected: PASS. `goalCard.test.ts` took N tests with it; Task 1 added 8 covering the same
decisions. Write the arithmetic in the commit body — this is the one deletion in the plan that
removes tests, and a floor that is lowered silently is what B63 is about.

- [ ] **Step 8: Commit**

```bash
git add ui/src/screens/goals/AddPane.vue ui/src/screens/goals/WantAnswer.vue
git commit -m "feat(ui): one pane for everything that can enter the queue"
```

---

### Task 6: one screen, one route, and the tab that survives it

**Files:**
- Modify: `ui/src/screens/GoalsScreen.vue`, `ui/src/router/routeTable.ts`,
  `ui/src/router/routes.ts`, `ui/src/components/shell/sectionNav.ts`,
  `ui/src/lib/window/sessionDocument.ts`, `docs/architecture.md`
- Test: `ui/src/lib/window/sessionDocument.test.ts`
- Delete: `ui/src/screens/PlanScreen.vue`, `ui/src/screens/plan/ProposalAside.vue`,
  `ui/src/lib/plan/planNow.ts` (+ its test)

**Interfaces:**
- Consumes: `AddPane` (Task 5), `QueueCard` (Task 4).
- Produces: `RETIRED_ROUTE_NAMES` in `sessionDocument.ts`.

> **Order matters, and it is the reason the map is in this task and not its own.** A test that
> restores a tab named `plan` while `RouteName.Plan` still exists **passes without the map** —
> it is the vacuity trap `CLAUDE.md` names. The route has to leave in the same commit that
> teaches the reader to carry it, or the test proves nothing.

- [ ] **Step 1: Assemble the screen, on the band**

Spec §4.2: the same grammar as `CompletionHero.vue` and `WikiHero.vue`. Read `CompletionScreen.vue`
and `CompletionHero.vue` first and copy the **shape**, not the content — the band here holds the
`ScreenHeader` and nothing else, no headline number and no progress bar.

Create `ui/src/screens/goals/GoalsHero.vue`:

```vue
<template>
  <!-- The band the screen opens on, the same grammar as Completion's and a wiki page's: the
       light comes from the corner and falls back into the page, and the grain stands in for
       the shadow the skin does not have. It carries the title and nothing countable — spec
       §5 declined a count at the top of this screen, and a band is not a reason to find one. -->
  <header class="relative border-b border-hairline hero-wash px-5.5 py-5">
    <span class="pointer-events-none absolute inset-0 hero-grain" />
    <div class="relative">
      <ScreenHeader :icon="ListChecksIcon" :title="t('routes.goals')">{{
        t('goals.intro')
      }}</ScreenHeader>
    </div>
  </header>
</template>
```

`GoalsScreen.vue` becomes the banded shape — the band stays, the two panes take the height that is
left. It is **not** the `flowing` root it has today:

```vue
  <div class="-mx-5.5 flex h-full min-h-0 flex-col overflow-hidden">
    <GoalsHero />
    <div class="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-5.5 pt-4 pb-5
                @wide/page:flex-row @wide/page:items-start @wide/page:overflow-hidden">
      <AddPane class="w-full @wide/page:w-add-pane @wide/page:shrink-0" … />
      <QueueCard class="w-full min-w-0 flex-1 @wide/page:min-h-0" … />
    </div>
  </div>
```

> **Why the overflow swaps sides at `wide`.** Stacked, the two panes are one column and the
> column scrolls. Side by side they are two columns of different lengths, and one scrollbar for
> both would scroll the queue out of sight to reach the bottom of the recommendations — which is
> the one thing §2 says must never happen. So above `wide` the row holds the height and each pane
> scrolls inside itself. `min-h-0` on every link of that chain is not decoration: the contract in
> `docs/frontend-conventions.md` says what one missing link costs.

It loads both stores (`graph.load()`, `queue.load()`) as `PlanScreen` did, keeps `QueueError`, the
`DiagnosticsList` with the import button, and drops the three-count `<p>` and the *"nel tuo piano"*
section. Rename `--spacing-plan-aside` to `--spacing-add-pane` in `spacing.css` and update the
comment: *aside* was the name of a secondary column, and it is now the only way in.

- [ ] **Step 2: Write the failing test for the retired name**

Append to `ui/src/lib/window/sessionDocument.test.ts`:

```ts
describe('a tab stored on a screen that has since merged', () => {
  // The route left the table in the same commit as this test: before that, `plan` was a
  // RouteName and the assertion would have passed without a line of the map being written.
  it('restores a Piano tab on Obiettivi', () => {
    const stored = JSON.stringify({
      version: 2,
      windows: [
        { tabs: [{ entries: [{ location: { name: 'plan' } }], index: 0 }], activeIndex: 0 },
      ],
    })
    expect(readSession(stored)?.windows[0]?.tabs[0]?.entries[0]?.location).toEqual({
      name: RouteName.Goals,
    })
  })

  it('carries the query across with it', () => {
    const stored = JSON.stringify({
      version: 2,
      windows: [
        {
          tabs: [
            { entries: [{ location: { name: 'plan', query: { want: 'item:5' } } }], index: 0 },
          ],
          activeIndex: 0,
        },
      ],
    })
    expect(readSession(stored)?.windows[0]?.tabs[0]?.entries[0]?.location).toEqual({
      name: RouteName.Goals,
      query: { want: 'item:5' },
    })
  })

  it('still drops a name that is neither known nor retired, and keeps the window', () => {
    const stored = JSON.stringify({
      version: 2,
      windows: [
        {
          tabs: [
            { entries: [{ location: { name: 'nowhere' } }], index: 0 },
            tab(RouteName.Wiki),
          ],
          activeIndex: 0,
        },
      ],
    })
    expect(readSession(stored)?.windows[0]?.tabs).toHaveLength(1)
  })
})
```

- [ ] **Step 3: Run it and watch it fail**

Run: `cd ui && pnpm exec vitest run src/lib/window/sessionDocument.test.ts`
Expected: FAIL on the first two — the tab is dropped, so `windows[0].tabs` is empty. The third
passes already, and that is correct: it is the guard that the map did not widen what gets accepted.

- [ ] **Step 4: Retire the route and write the map**

In `routeTable.ts` remove `Plan: 'plan'` from `RouteName` and its four record entries. In
`routes.ts` remove `[RouteName.Plan]: PlanScreen` and add, beside the `/` redirect:

```ts
  // The URL a link or a bookmark may still carry. No `name`, so it is not a location and
  // cannot become a tab: a stored tab is carried by `sessionDocument`'s retired-name map,
  // which is a different mechanism for a different thing.
  { path: '/progress/plan', redirect: routePath[RouteName.Goals] },
```

In `sectionNav.ts` drop `RouteName.Plan` from the Progress list. In `sessionDocument.ts`, above
`readLocation`:

```ts
// Screens that merged into another. A stored tab on one of these is **carried**, not dropped:
// the reader's own comment below says eight tabs do not vanish because one screen was renamed,
// and losing the ninth quietly is the same failure at a smaller size.
const RETIRED_ROUTE_NAMES: Readonly<Record<string, RouteName>> = {
  plan: RouteName.Goals,
}
```

and, inside `readLocation`, before the `isRouteName` refusal:

```ts
  const retired = typeof name === 'string' ? RETIRED_ROUTE_NAMES[name] : undefined
  const resolved = retired ?? name
  if (!isRouteName(resolved)) return null
```

then build the result from `resolved`.

- [ ] **Step 5: Delete what the merge replaced**

```bash
git rm ui/src/screens/PlanScreen.vue ui/src/screens/plan/ProposalAside.vue \
  ui/src/lib/plan/planNow.ts ui/src/lib/plan/planNow.test.ts
```

Remove `queueSummary` and `QueueSummary` from `ui/src/lib/plan/queueRows.ts` and their tests: §5
deleted the only thing that called them.

- [ ] **Step 6: Run the suite**

Run: `pnpm ui:test`
Expected: PASS, with the three new session tests green. If `sectionNav.test.ts` fails, read it
before editing it — it asserts that every Progress route is in the sidebar, and it should stay
green on its own now that `Plan` is in neither.

- [ ] **Step 7: Add the third screen shape to the contract**

`docs/frontend-conventions.md` §"A screen is one of two shapes" lists *flowing* and *filling*, and
files **Completion** under flowing. It has not been flowing since card #58. Add a third row and
move Completion into it beside Goals:

| shape | root classes | who |
|---|---|---|
| **banded** | `-mx-5.5 flex h-full min-h-0 flex-col overflow-hidden`, a `hero-wash` header, and a body carrying `min-h-0 flex-1` | a screen that opens on a band: Completion, Goals |

Rename the heading to "A screen is one of three shapes", and remove Goals and Plan from the
flowing row. Say in a line that Completion was listed wrongly since card #58 and that this is the
correction — a contract that describes two of three shapes is read as forbidding the third.

- [ ] **Step 8: Redraw `docs/architecture.md`**

The header pins **17 routes**; it becomes **16**. Update the count, remove the `planr` node from
the `progressGroup` subgraph, remove the *Plan* row from the route table, and fold its commands
into the *Goals* row. Add a line to the header's drawn-on paragraph saying what moved and when,
the way the existing entries do.

- [ ] **Step 9: Commit**

```bash
git add ui/src/screens/GoalsScreen.vue ui/src/router/routeTable.ts ui/src/router/routes.ts \
  ui/src/components/shell/sectionNav.ts ui/src/lib/window/sessionDocument.ts \
  ui/src/lib/window/sessionDocument.test.ts ui/src/lib/plan/queueRows.ts \
  ui/src/assets/theme/spacing.css ui/src/screens/goals/GoalsHero.vue \
  docs/frontend-conventions.md docs/architecture.md
git commit -m "feat(ui): Obiettivi holds the queue, and the Piano route retires into it"
```

---

### Task 7: the documents, the floor, and the window

**Files:**
- Modify: `README.md`, `docs/PROJECT.md`, `scripts/test-floor`

- [ ] **Step 1: Update the screen lists**

`README.md`'s table and `docs/PROJECT.md` §11 both list *Next steps* and *Plan* as two screens.
They become one row. Do not touch `docs/completed/` or anything under `docs/superpowers/` other
than this plan: those are history, and a sentence recording that there were two screens is a dated
fact.

- [ ] **Step 2: Run the full gate**

Run: `pnpm check`
Expected: all green, with a line printing the new test totals.

- [ ] **Step 3: Raise the floor with a written delta**

Paste the printed `RUST_TESTS=` / `UI_TESTS=` into `scripts/test-floor`, and **above the numbers
write the entry as a delta, never as an arrow between two totals** — the file says why, and it has
been paid for four times. The entry states: how many tests arrived (Tasks 1, 2 and 6), how many
left with `goalCard.test.ts`, `planNow.test.ts` and `queueSummary`'s, and that the B28 guard moved
into `rowModel.test.ts` rather than being dropped.

- [ ] **Step 4: Look at it in a real window**

Run: `pnpm dev`. Three things no assertion in this plan covers, and they are the reason the card
carries `NEEDS WINDOW`:

1. The two panes at the window floor, **640 × 480** — they must stack, not clip.
2. A queue of thirty rows: is it actually faster to scan than the eleven-field row it replaced?
   That is the whole point of the cycle, and only your eyes answer it.
3. An open row while dragging another: the expansion must not move the drop line.

- [ ] **Step 5: Commit**

```bash
git add README.md docs/PROJECT.md scripts/test-floor
git commit -m "docs: Obiettivi and the Plan are one screen in the lists too"
```

---

## What is deliberately not here

- **Dragging a row from the left pane into the queue** — spec §8. `useDragList` is what every list
  in the app drags with, tabs included, and a cross-list drop is new logic inside it.
- **Any change to the queue's move rules**, the prerequisite wall, or `dropAnchor`. Task 4 changes
  what a row looks like and nothing about how it moves.
- **Any Rust.** If a step seems to need a field that is not on the wire, stop and re-read spec §7
  before writing a line of it.
