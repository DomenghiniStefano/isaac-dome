# Cycle 3.3b — Plan and the queue: Implementation Plan

> **For agentic workers:** executed inline with superpowers:executing-plans, on the owner's
> delegation ("procedi come credi"). Steps use checkbox (`- [ ]`) syntax. Every commit is pushed
> right away (the owner's rule: the remote stays aligned).

**Goal:** the Plan screen on the real queue — rows you drag, a repair that never fails and says
where a row stopped — and "in the queue" / "add to the queue" on Next steps and Unlock.

**Architecture:** one contract fix first: `queue_move` names the row a drop lands under instead of
an index, because the view's indices are not the file's (Decision 1). Everything the screen
decides lives in pure functions in `ui/src/lib/plan/`, tested first; a Pinia store holds the
queue and replaces it with every mutation's answer; the development fixtures keep a queue in
memory and repair it with a port of the Rust rule. Components are presentational.

**Tech Stack:** Rust (`crates/plan`, `crates/app`), Vue 3.5, TypeScript, Pinia 4.0.3, Tailwind
v4.3, Reka UI 2.10.4, vue-i18n 11.4, lodash-es, Vitest.

**Spec:** `docs/superpowers/specs/2026-09-11-screens-plan-design.md`

## Global Constraints

- A move repairs, never fails; the screen draws the order the backend answers with, never an optimistic one.
- Every mutation answers with the whole `QueueView`; a refused mutation leaves the view as it was and says why.
- `QueueView` and the saved document do not change: no migration, no `store` change.
- Degrade: `storeUnavailable`, `unreadable`, `noCatalog`, `goalsPending`, `completed`, `unresolved` each shown.
- Frontend: the five rules, `assertNever` on closed switches, `as const` objects, every visible string through `useMessages()`, game texts are data.
- Fixtures only under `import.meta.env.DEV`; payloads through `import.meta.glob`.
- Rust: no `unwrap`/`panic` outside tests, exhaustive matches, `cargo fmt` and `clippy -D warnings` clean.
- Commits: Conventional Commits, English, no `Co-Authored-By` or Claude reference, **pushed after each task**.
- Checks are judged by **exit code**, never by grepping coloured output; never two `vue-tsc --build` at once.

**Deviation decided while planning:** the fixture's `storeUnavailable` reason mirrors the app's
own text, `store_reason` in `crates/app/src/lib.rs` — "database from a newer version (3 > 2)" —
not the Italian wording the spec quoted.

---

### Task 1: A move names the row it lands under

**Files:** Modify `crates/plan/src/order.rs`, `crates/plan/tests/order.rs`, `crates/app/src/lib.rs` (`queue_move`), `ui/src/lib/ipc/queue.ts`

**Interfaces:** Produces `Queue::move_after(&mut self, achievement: u32, after: Option<u32>, deps: &impl Dependencies)`; Tauri `queue_move(achievement: u32, after: Option<u32>)`; TS `queueMove(achievement: number, after: number | null): Promise<QueueView>`.

- [x] **Step 1: Failing tests** — append to `crates/plan/tests/order.rs`:

```rust
#[test]
fn an_index_means_the_same_place_when_dependents_sit_before_it() {
    // 2 requires 1. Once 1 is out the queue reads [2, 3, 4]: index 2 is right after 3.
    let deps = Deps(&[(2, 1)]);
    let mut q = queue(&[1, 2, 3, 4]);
    q.move_row(1, 2, &deps);
    assert_eq!(
        ids(&q),
        vec![3, 1, 2, 4],
        "1 lands below 3 with 2 in tow, not below 4"
    );
}

#[test]
fn move_after_lands_right_below_the_row_named() {
    let deps = Deps(&[]);
    let mut q = queue(&[1, 2, 3, 4]);
    q.move_after(1, Some(3), &deps);
    assert_eq!(ids(&q), vec![2, 3, 1, 4], "downwards");
    q.move_after(4, Some(2), &deps);
    assert_eq!(ids(&q), vec![2, 4, 3, 1], "upwards");
}

#[test]
fn move_after_nothing_is_the_top() {
    let deps = Deps(&[]);
    let mut q = queue(&[1, 2, 3]);
    q.move_after(3, None, &deps);
    assert_eq!(ids(&q), vec![3, 1, 2]);
}

#[test]
fn moving_under_one_of_its_own_dependents_goes_as_low_as_it_can() {
    // 2 requires 1. Below 2 is not a place 1 can be: it goes right above 2, and 2 follows.
    let deps = Deps(&[(2, 1)]);
    let mut q = queue(&[1, 3, 2, 4]);
    q.move_after(1, Some(2), &deps);
    assert_eq!(ids(&q), vec![3, 1, 2, 4]);
}

#[test]
fn rising_past_a_prerequisite_stops_right_below_it() {
    // 3 requires 2. Asked to sit below 1, 3 rises past 4 and stops under 2.
    let deps = Deps(&[(3, 2)]);
    let mut q = queue(&[1, 2, 4, 3]);
    q.move_after(3, Some(1), &deps);
    assert_eq!(ids(&q), vec![1, 2, 3, 4]);
}

#[test]
fn an_anchor_not_queued_or_the_row_itself_changes_nothing() {
    let deps = Deps(&[]);
    let mut q = queue(&[1, 2, 3]);
    q.move_after(1, Some(99), &deps);
    q.move_after(2, Some(2), &deps);
    q.move_after(99, None, &deps);
    assert_eq!(ids(&q), vec![1, 2, 3], "a stale picture is answered with the truth");
}
```

and turn the property into a helper run by two tests:

```rust
/// 1 <- 2 <- 3, transitive closure written out; 4 and 5 unconstrained.
const PAIRS: &[(u32, u32)] = &[(2, 1), (3, 2), (3, 1)];

fn assert_consistent(order: &[u32], round: usize) {
    for (a, b) in PAIRS {
        let ia = order.iter().position(|x| x == a).expect("present");
        let ib = order.iter().position(|x| x == b).expect("present");
        assert!(
            ib < ia,
            "round {round}: {a} requires {b}, and {b} ended up below it: {order:?}"
        );
    }
    assert_eq!(order.len(), 5, "round {round}: a move lost or duplicated a row");
}

/// Deterministic pseudo-random: a failure has to be reproducible from the round printed.
fn lcg(mut seed: u64) -> impl FnMut() -> usize {
    move || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as usize
    }
}

#[test]
fn after_any_move_the_queue_never_contradicts_the_graph() {
    let deps = Deps(PAIRS);
    let mut next = lcg(12345);
    for round in 0..500 {
        let mut q = queue(&[1, 2, 3, 4, 5]);
        for _ in 0..4 {
            let who = [1u32, 2, 3, 4, 5][next() % 5];
            q.move_row(who, next() % 5, &deps);
        }
        assert_consistent(&ids(&q), round);
    }
}

#[test]
fn after_any_move_after_the_queue_never_contradicts_the_graph() {
    let deps = Deps(PAIRS);
    let mut next = lcg(54321);
    let anchors = [None, Some(1u32), Some(2), Some(3), Some(4), Some(5), Some(99)];
    for round in 0..500 {
        let mut q = queue(&[1, 2, 3, 4, 5]);
        for _ in 0..4 {
            let who = [1u32, 2, 3, 4, 5][next() % 5];
            q.move_after(who, anchors[next() % anchors.len()], &deps);
        }
        assert_consistent(&ids(&q), round);
    }
}
```

(the old `after_any_move_the_queue_never_contradicts_the_graph` body is replaced by the version above).

- [x] **Step 2: Run** `cargo test -p plan --test order` → FAIL: `move_after` doesn't exist (compile error). Temporarily comment the `move_after` tests out and run again to see `an_index_means_the_same_place_when_dependents_sit_before_it` fail with `[3, 4, 1, 2]`; restore them.

- [x] **Step 3: Implement** in `crates/plan/src/order.rs` — replace the `move_row` doc comment and the partition loop, add `move_after`:

```rust
impl Queue {
    /// Moves a row right below another one — `None` for the top.
    ///
    /// The way a screen names a drop: by the row it lands under. An index would not do, since
    /// the view leaves completed and unresolved rows out and its positions are not the file's.
    /// An `after` that isn't queued, or is the moved row itself, leaves the queue as it is: the
    /// caller's picture was stale, and the view it gets back is the truth.
    pub fn move_after(&mut self, achievement: u32, after: Option<u32>, deps: &impl Dependencies) {
        let Some(from) = self.position(achievement) else {
            return;
        };
        let to = match after {
            None => 0,
            Some(a) if a == achievement => return,
            Some(a) => match self.position(a) {
                // A position once the moved row is out: one past the anchor.
                Some(i) if i < from => i + 1,
                Some(i) => i,
                None => return,
            },
        };
        self.move_row(achievement, to, deps);
    }

    /// Moves a row to `to` — a position in the queue once the row is taken out — and returns
    /// the index it landed at among the rows that didn't move with it.
    ///
    /// Rows that depend on it are dragged along, right below it; rows it depends on are a wall
    /// it stops under, and never move; everything else keeps its relative order. The landing
    /// is clamped between one past the last prerequisite and the end of the list.
    pub fn move_row(&mut self, achievement: u32, to: usize, deps: &impl Dependencies) -> usize {
        // … unchanged up to the partition …
        let mut dragged = Vec::new();
        let mut rest = Vec::new();
        // `to` counts the dragged rows too, and each one above it leaves with the moved row:
        // the target among the rows that stay is that much higher.
        let mut target = to;
        for (i, r) in rows.into_iter().enumerate() {
            if deps.requires(r.achievement, achievement) {
                if i < to {
                    target -= 1;
                }
                dragged.push(r);
            } else {
                rest.push(r);
            }
        }
        // … floor unchanged …
        let landed = target.clamp(floor, rest.len());
        // … assembly unchanged …
    }
}
```

The module doc keeps its first paragraph; the long comment on the two relations inside `move_row` stays as it is.

- [x] **Step 4: Run** `cargo test -p plan` → all green (the nine old cases still hold: each was checked against the new target arithmetic in the spec's session).

- [x] **Step 5: The command and the wrapper** — in `crates/app/src/lib.rs`, `queue_move` takes `after: Option<u32>` instead of `to: usize` and calls `q.move_after(achievement, after, &deps)`. In `ui/src/lib/ipc/queue.ts`:

```ts
// `after` is the row it was dropped under, `null` for the top: a row and not an index, because
// the view leaves completed and unresolved rows out and its positions are not the file's. The
// row lands there when the graph allows it and as close as it allows otherwise: read the
// returned order, never assume it.
export const queueMove = (
  achievement: number,
  after: number | null,
): Promise<QueueView> => call(Command.QueueMove, { achievement, after })
```

- [x] **Step 6: Verify** `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test -p plan && cargo build -p app` and `pnpm typecheck`, each by exit code. Commit `fix(plan): a move names the row it lands under, not an index` and push.

### Task 2: The Badge learns the wanted row

**Files:** Modify `ui/src/components/ui/badge/variants.ts`, `icons.ts`, `ui/src/kit/sections/BadgeSection.vue`

- [x] **Step 1**: `BadgeVariant.Wanted: 'wanted'`; classes `${squareTag} border-primary-edge bg-primary text-primary-foreground`, with a comment: a row you asked for, told apart from a step a wish dragged in (the export's red "chiesta").
- [x] **Step 2**: `badgeIcons[BadgeVariant.Wanted] = null` — a tag, not a state.
- [x] **Step 3**: the Kit's Badge section shows `<Badge :variant="BadgeVariant.Wanted">Chiesta</Badge>` after `Oggetto`.
- [x] **Step 4**: `pnpm typecheck` and `pnpm scan` → exit 0. Commit `feat(ui): the Badge learns the wanted row` and push.

### Task 3: The queue's pure logic

**Files:** Create `ui/src/lib/plan/queueRows.ts`, `queueRows.test.ts`, `queueDrop.ts`, `queueDrop.test.ts`

**Interfaces:** Produces, from `queueRows.ts`: `rowId(row: QueueRow): number`; `knownText(node: UnlockNode): string | null`; `achievementText(nodes: UnlockNode[], id: number): string | null`; `queuedIds(view: QueueView | null): Set<number>`; `isQueued(node: UnlockNode, queued: Set<number>): boolean`; `canQueue(node: UnlockNode, queued: Set<number>): boolean`; `QueueSummary { rows; wanted; pulledIn }`, `queueSummary(rows): QueueSummary`; `OriginRow { id: number; row: QueueRow | null }`, `originRows(row, rows): OriginRow[]`; `stoppedUnder(rows, achievement: number, after: number | null): QueueRow | null`. From `queueDrop.ts`: `Anchor { after: number | null }`; `DropEdge { Above, Below }`; `StepDirection { Up, Down }`; `dropEdge(pointerY, top, height): DropEdge`; `dropAnchor(ids: number[], from: number, target: number, edge: DropEdge): Anchor | null`; `stepAnchor(ids: number[], index: number, direction: StepDirection): Anchor | null`.

- [x] **Step 1: Failing tests** — `queueDrop.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import {
  DropEdge,
  StepDirection,
  dropAnchor,
  dropEdge,
  stepAnchor,
} from './queueDrop'

// The payload's queue (contracts/payload/queue.with_rows.json): 480, 55, 69.
const ids = [480, 55, 69]

describe('dropEdge', () => {
  it('is the half of the row under the pointer, the middle counting as below', () => {
    expect(dropEdge(109, 100, 20)).toBe(DropEdge.Above)
    expect(dropEdge(110, 100, 20)).toBe(DropEdge.Below)
  })
})

describe('dropAnchor', () => {
  it('names the row a drop lands under, or the top', () => {
    expect(dropAnchor(ids, 2, 0, DropEdge.Above)).toEqual({ after: null })
    expect(dropAnchor(ids, 2, 0, DropEdge.Below)).toEqual({ after: 480 })
    expect(dropAnchor(ids, 0, 1, DropEdge.Below)).toEqual({ after: 55 })
    expect(dropAnchor(ids, 0, 2, DropEdge.Below)).toEqual({ after: 69 })
  })

  it('sends nothing for a drop on the gap the row already fills', () => {
    expect(dropAnchor(ids, 1, 1, DropEdge.Above)).toBeNull()
    expect(dropAnchor(ids, 1, 1, DropEdge.Below)).toBeNull()
    expect(dropAnchor(ids, 0, 1, DropEdge.Above)).toBeNull()
    expect(dropAnchor(ids, 1, 0, DropEdge.Below)).toBeNull()
  })

  it('sends nothing for a row or a target outside the queue', () => {
    expect(dropAnchor(ids, 3, 0, DropEdge.Above)).toBeNull()
    expect(dropAnchor(ids, 0, 3, DropEdge.Above)).toBeNull()
    expect(dropAnchor(ids, 0, -1, DropEdge.Below)).toBeNull()
  })
})

describe('stepAnchor', () => {
  it('moves a row one place up or down', () => {
    expect(stepAnchor(ids, 2, StepDirection.Up)).toEqual({ after: 480 })
    expect(stepAnchor(ids, 1, StepDirection.Up)).toEqual({ after: null })
    expect(stepAnchor(ids, 0, StepDirection.Down)).toEqual({ after: 55 })
    expect(stepAnchor(ids, 1, StepDirection.Down)).toEqual({ after: 69 })
  })

  it('does nothing past either end', () => {
    expect(stepAnchor(ids, 0, StepDirection.Up)).toBeNull()
    expect(stepAnchor(ids, 2, StepDirection.Down)).toBeNull()
  })
})
```

`queueRows.test.ts` — rows built from the Unlock fixture's nodes, with the payload's `wanted`, `origins` and `stepsNotQueued`:

```ts
import { describe, expect, it } from 'vitest'
import { graphAnswers } from '@/lib/ipc/fixtures/graph'
import type { QueueRow, UnlockNode } from '@/lib/ipc/types'
import {
  achievementText,
  canQueue,
  isQueued,
  originRows,
  queueSummary,
  queuedIds,
  rowId,
  stoppedUnder,
} from './queueRows'

const nodes = graphAnswers({ withArt: false, withCatalog: true }).unlock.nodes
const node = (id: number): UnlockNode => {
  const found = nodes.find(
    (n) => n.achievement.kind === 'known' && n.achievement.id === id,
  )
  if (!found) throw new Error(`no node ${id}`)
  return found
}
const row = (id: number, wanted: boolean, origins: number[] = []): QueueRow => ({
  node: node(id),
  wanted,
  origins,
  stepsNotQueued: id === 69 ? 1 : 0,
})
// queue.with_rows.json: 480 pulled in by 55, 55 wanted, 69 wanted.
const rows = [row(480, false, [55]), row(55, true), row(69, true)]
const order = (...ids: number[]) =>
  ids.map((id) => rows.find((r) => rowId(r) === id) as QueueRow)

describe('the queue, read', () => {
  it('knows which achievements are queued', () => {
    const view = { rows, diagnostics: [], storeAvailable: true }
    expect([...queuedIds(view)]).toEqual([480, 55, 69])
    expect(queuedIds(null).size).toBe(0)
    expect(isQueued(node(55), queuedIds(view))).toBe(true)
  })

  it('counts rows asked for and rows pulled in', () => {
    expect(queueSummary(rows)).toEqual({ rows: 3, wanted: 2, pulledIn: 1 })
  })

  it("names the wish a step serves, and says when it isn't shown", () => {
    expect(originRows(rows[0] as QueueRow, rows)).toEqual([
      { id: 55, row: rows[1] },
    ])
    expect(originRows(row(480, false, [1]), rows)).toEqual([
      { id: 1, row: null },
    ])
  })

  it('finds an achievement’s text among the nodes', () => {
    expect(achievementText(nodes, 1)).toBe('You unlocked "Magdalene"')
    expect(achievementText(nodes, 9999)).toBeNull()
  })
})

describe('canQueue', () => {
  const queued = new Set([480, 55, 69])
  it('offers a known achievement not done and not queued', () => {
    expect(canQueue(node(484), queued)).toBe(true)
  })
  it('never a done one, an unknown one or one already queued', () => {
    expect(canQueue(node(1), queued)).toBe(false)
    expect(canQueue(node(55), queued)).toBe(false)
    const unknown = nodes.find((n) => n.achievement.kind === 'unknown')
    expect(unknown && canQueue(unknown, queued)).toBe(false)
  })
})

describe('stoppedUnder', () => {
  it('is the row above, when a row rising to the top stopped short', () => {
    expect(stoppedUnder(order(480, 55, 69), 55, null)).toBe(rows[0])
  })
  it('is nothing when the row reached the gap it was dropped in', () => {
    expect(stoppedUnder(order(69, 480, 55), 480, 69)).toBeNull()
    expect(stoppedUnder(order(55, 480, 69), 55, null)).toBeNull()
  })
  it('is nothing when the row was dropped under one of its own dependents', () => {
    // 480 dropped under 55, which needs it: 480 sits right above 55, not below it.
    expect(stoppedUnder(order(69, 480, 55), 480, 55)).toBeNull()
  })
})
```

Check the expected text of achievement 1 against the payload before running (`"You unlocked \"Magdalene\""` in `unlock.json`); if the payload spells it differently, the expectation follows the payload.

- [x] **Step 2: Run** `pnpm --filter ui exec vitest run src/lib/plan` → FAIL (no modules).

- [x] **Step 3: Implement** `queueDrop.ts`:

```ts
// Where a dragged row goes, named the way the backend takes it: right below a row, or the top.
export interface Anchor {
  after: number | null
}

export const DropEdge = { Above: 'above', Below: 'below' } as const
export type DropEdge = (typeof DropEdge)[keyof typeof DropEdge]

export const StepDirection = { Up: 'up', Down: 'down' } as const
export type StepDirection = (typeof StepDirection)[keyof typeof StepDirection]

// The half of the row under the pointer decides; the exact middle counts as below.
export const dropEdge = (
  pointerY: number,
  top: number,
  height: number,
): DropEdge => (pointerY < top + height / 2 ? DropEdge.Above : DropEdge.Below)

// A drop on one edge of the row at `target` fills the gap above or below it. The gap right
// above or right below the dragged row is where it already is: nothing to send.
export const dropAnchor = (
  ids: number[],
  from: number,
  target: number,
  edge: DropEdge,
): Anchor | null => {
  if (from < 0 || from >= ids.length || target < 0 || target >= ids.length)
    return null
  const gap = edge === DropEdge.Above ? target : target + 1
  if (gap === from || gap === from + 1) return null
  return { after: gap === 0 ? null : (ids[gap - 1] ?? null) }
}

// Alt+arrow on a row: the same anchor a drop on the neighbour's far edge would give.
export const stepAnchor = (
  ids: number[],
  index: number,
  direction: StepDirection,
): Anchor | null => {
  switch (direction) {
    case StepDirection.Up:
      return dropAnchor(ids, index, index - 1, DropEdge.Above)
    case StepDirection.Down:
      return dropAnchor(ids, index, index + 1, DropEdge.Below)
    default:
      return assertNever(direction)
  }
}
```

(with `import { assertNever } from '@/lib/assertNever'` at the top) and `queueRows.ts`:

```ts
import { nodeSlot } from '@/lib/graph/unlockFilter'
import type { QueueRow, QueueView, UnlockNode } from '@/lib/ipc/types'

// A queue row is always a known achievement (the view leaves unresolved ones out), so its slot
// is its id.
export const rowId = (row: QueueRow): number => nodeSlot(row.node)

export const knownText = (node: UnlockNode): string | null =>
  node.achievement.kind === 'known' ? node.achievement.text : null

export const achievementText = (
  nodes: UnlockNode[],
  id: number,
): string | null => {
  const found = nodes.find(
    (n) => n.achievement.kind === 'known' && n.achievement.id === id,
  )
  return found ? knownText(found) : null
}

export const queuedIds = (view: QueueView | null): Set<number> =>
  new Set(view?.rows.map(rowId) ?? [])

export const isQueued = (node: UnlockNode, queued: Set<number>): boolean =>
  node.achievement.kind === 'known' && queued.has(node.achievement.id)

// What can be put in the queue: a known achievement, not done, not already there.
export const canQueue = (node: UnlockNode, queued: Set<number>): boolean =>
  node.achievement.kind === 'known' && !node.done && !isQueued(node, queued)

export interface QueueSummary {
  rows: number
  wanted: number
  pulledIn: number
}

export const queueSummary = (rows: QueueRow[]): QueueSummary => {
  const wanted = rows.filter((r) => r.wanted).length
  return { rows: rows.length, wanted, pulledIn: rows.length - wanted }
}

export interface OriginRow {
  id: number
  row: QueueRow | null
}

export const originRows = (row: QueueRow, rows: QueueRow[]): OriginRow[] =>
  row.origins.map((id) => ({
    id,
    row: rows.find((r) => rowId(r) === id) ?? null,
  }))

// The row a moved one stopped under. A move stops short only on the way up — dependents are
// dragged, never jumped — so it stopped short when it sits below the gap it was dropped in; the
// wall is then the row right above it.
export const stoppedUnder = (
  rows: QueueRow[],
  achievement: number,
  after: number | null,
): QueueRow | null => {
  const index = rows.findIndex((r) => rowId(r) === achievement)
  const anchor = rows.findIndex((r) => rowId(r) === after)
  if (index < 0 || (after !== null && anchor < 0)) return null
  const asked = after === null ? 0 : anchor + 1
  return index > asked ? (rows[index - 1] ?? null) : null
}
```

- [x] **Step 4: Run** the tests → green; `pnpm typecheck`, `pnpm lint`, `pnpm scan` → exit 0. Commit `feat(ui): the queue's rows, drops and anchors, as pure functions` and push.

### Task 4: The queue's fixtures

**Files:** Create `ui/src/lib/ipc/fixtures/queueRepair.ts`, `queueRepair.test.ts`, `queue.ts`, `queue.test.ts`; Modify `ui/src/lib/ipc/fixtures/index.ts`

**Interfaces:** Consumes `graphAnswers`. Produces `Requires = (a: number, b: number) => boolean`; `moveRow(ids, achievement, to, requires): number[]`; `moveAfter(ids, achievement, after, requires): number[]`; `QueueScenario { Rows, Empty, Unavailable, Unreadable }`; `QueueOptions { scenario: QueueScenario; withCatalog: boolean; nodes: UnlockNode[] }`; `resetQueue()`; `readQueue(o): QueueView`; `addToQueue(o, id)`, `removeFromQueue(o, id)`, `moveInQueue(o, id, after)`, `importGoals(o)`, each `Promise<QueueView>`; handlers for the five queue commands; `?queue=`.

- [x] **Step 1: Failing tests** — `queueRepair.test.ts`, the Rust cases on plain arrays:

```ts
import { describe, expect, it } from 'vitest'
import type { Requires } from './queueRepair'
import { moveAfter, moveRow } from './queueRepair'

// The same readable cases as crates/plan/tests/order.rs: this port exists only for the
// development server, and it has to repair the way the app does.
const pairs =
  (...p: [number, number][]): Requires =>
  (a, b) =>
    p.some(([x, y]) => x === a && y === b)

describe('moveRow', () => {
  it('lands where dropped with no dependencies', () => {
    expect(moveRow([1, 2, 3, 4], 4, 1, pairs())).toEqual([1, 4, 2, 3])
  })
  it('drags what needs the row', () => {
    expect(moveRow([1, 2, 3], 1, 2, pairs([3, 1]))).toEqual([2, 1, 3])
  })
  it('stops under prerequisites, which never move', () => {
    expect(moveRow([1, 2, 3, 4], 4, 0, pairs([4, 1], [4, 2]))).toEqual([1, 2, 4, 3])
  })
  it('means the same place when dependents sit before the index', () => {
    expect(moveRow([1, 2, 3, 4], 1, 2, pairs([2, 1]))).toEqual([3, 1, 2, 4])
  })
})

describe('moveAfter', () => {
  it('lands right below the row named, or at the top', () => {
    expect(moveAfter([1, 2, 3, 4], 1, 3, pairs())).toEqual([2, 3, 1, 4])
    expect(moveAfter([1, 2, 3], 3, null, pairs())).toEqual([3, 1, 2])
  })
  it('goes as low as it can under one of its own dependents', () => {
    expect(moveAfter([1, 3, 2, 4], 1, 2, pairs([2, 1]))).toEqual([3, 1, 2, 4])
  })
  it('changes nothing for a stale anchor or the row itself', () => {
    expect(moveAfter([1, 2, 3], 1, 99, pairs())).toEqual([1, 2, 3])
    expect(moveAfter([1, 2, 3], 2, 2, pairs())).toEqual([1, 2, 3])
  })
})
```

`queue.test.ts`:

```ts
import { beforeEach, describe, expect, it } from 'vitest'
import type { QueueView } from '../types'
import { graphAnswers } from './graph'
import {
  QueueScenario,
  addToQueue,
  importGoals,
  moveInQueue,
  readQueue,
  removeFromQueue,
  resetQueue,
} from './queue'

const nodes = graphAnswers({ withArt: false, withCatalog: true }).unlock.nodes
const options = (scenario: QueueScenario = QueueScenario.Rows, withCatalog = true) => ({
  scenario,
  withCatalog,
  nodes,
})
const shown = (view: QueueView) =>
  view.rows.map((r) =>
    r.node.achievement.kind === 'known' ? r.node.achievement.id : null,
  )

beforeEach(resetQueue)

// contracts/payload/queue.with_rows.json, with achievement 1 done above its rows.
describe('the seeded queue', () => {
  it('answers the payload’s rows and the completed one', () => {
    const view = readQueue(options())
    expect(shown(view)).toEqual([480, 55, 69])
    expect(view.rows.map((r) => [r.wanted, r.origins, r.stepsNotQueued])).toEqual([
      [false, [55], 0],
      [true, [], 0],
      [true, [], 1],
    ])
    expect(view.diagnostics).toEqual([
      { kind: 'completed', count: 1, wanted: [1] },
    ])
    expect(view.storeAvailable).toBe(true)
  })

  it('drags 55 along when 480 goes below 69', async () => {
    expect(shown(await moveInQueue(options(), 480, 69))).toEqual([69, 480, 55])
  })

  it('stops 55 under 480 when it is sent to the top', async () => {
    expect(shown(await moveInQueue(options(), 55, null))).toEqual([480, 55, 69])
  })

  it('adds a wish at the end', async () => {
    const view = await addToQueue(options(), 484)
    expect(shown(view)).toEqual([480, 55, 69, 484])
    expect(view.rows[3]?.wanted).toBe(true)
  })

  it('takes a step away with the last wish that kept it', async () => {
    expect(shown(await removeFromQueue(options(), 55))).toEqual([69])
  })
})

describe('the queue’s other states', () => {
  it('holds goals to import, and imports them', async () => {
    expect(readQueue(options(QueueScenario.Empty))).toEqual({
      rows: [],
      diagnostics: [{ kind: 'goalsPending', count: 3 }],
      storeAvailable: true,
    })
    const view = await importGoals(options(QueueScenario.Empty))
    expect(shown(view)).toEqual([480, 55, 69])
    expect(view.diagnostics.some((d) => d.kind === 'goalsPending')).toBe(false)
  })

  it('says the database is unavailable, and refuses to write', async () => {
    const reason = 'database from a newer version (3 > 2)'
    expect(readQueue(options(QueueScenario.Unavailable))).toEqual({
      rows: [],
      diagnostics: [{ kind: 'storeUnavailable', reason }],
      storeAvailable: false,
    })
    await expect(addToQueue(options(QueueScenario.Unavailable), 484)).rejects.toEqual({
      kind: 'storeUnavailable',
      reason,
    })
  })

  it('says an unreadable queue is unreadable', () => {
    expect(readQueue(options(QueueScenario.Unreadable)).diagnostics).toEqual([
      { kind: 'unreadable' },
    ])
  })

  it('shows nothing without a catalog, and refuses to reorder against nothing', async () => {
    expect(readQueue(options(QueueScenario.Rows, false))).toEqual({
      rows: [],
      diagnostics: [{ kind: 'noCatalog' }],
      storeAvailable: true,
    })
    await expect(moveInQueue(options(QueueScenario.Rows, false), 480, 69)).rejects.toEqual({
      kind: 'catalogUnavailable',
    })
  })
})
```

- [x] **Step 2: Run** `pnpm --filter ui exec vitest run src/lib/ipc/fixtures` → FAIL (no modules).

- [x] **Step 3: Implement** `queueRepair.ts`:

```ts
import { clamp, findLastIndex } from 'lodash-es'

// Development only: a port of crates/plan/src/order.rs `move_after` and `move_row`, so the
// development server shows a queue that repairs. The app's order is the backend's; this exists
// to look at the screen, and its tests are the Rust ones.
export type Requires = (a: number, b: number) => boolean

export const moveRow = (
  ids: number[],
  achievement: number,
  to: number,
  requires: Requires,
): number[] => {
  if (!ids.includes(achievement)) return ids
  const others = ids.filter((id) => id !== achievement)
  const dragged = others.filter((id) => requires(id, achievement))
  const rest = others.filter((id) => !requires(id, achievement))
  const target =
    to - others.slice(0, to).filter((id) => requires(id, achievement)).length
  const floor = findLastIndex(rest, (id) => requires(achievement, id)) + 1
  const landed = clamp(target, floor, rest.length)
  return [
    ...rest.slice(0, landed),
    achievement,
    ...dragged,
    ...rest.slice(landed),
  ]
}

export const moveAfter = (
  ids: number[],
  achievement: number,
  after: number | null,
  requires: Requires,
): number[] => {
  const from = ids.indexOf(achievement)
  if (from < 0 || after === achievement) return ids
  if (after === null) return moveRow(ids, achievement, 0, requires)
  const at = ids.indexOf(after)
  if (at < 0) return ids
  return moveRow(ids, achievement, at < from ? at + 1 : at, requires)
}
```

`queue.ts`:

```ts
import { assertNever } from '../../assertNever'
import type { IpcError, QueueDiagnostic, QueueView, UnlockNode } from '../types'
import type { Requires } from './queueRepair'
import { moveAfter } from './queueRepair'

// `?queue=` on the development server; the seeded rows when absent.
export const QueueScenario = {
  Rows: 'rows',
  Empty: 'empty',
  Unavailable: 'unavailable',
  Unreadable: 'unreadable',
} as const
export type QueueScenario = (typeof QueueScenario)[keyof typeof QueueScenario]

export interface QueueOptions {
  scenario: QueueScenario
  withCatalog: boolean
  nodes: UnlockNode[]
}

interface StoredRow {
  achievement: number
  wanted: boolean
  origins: number[]
  stepsNotQueued: number
}

// The app's own texts (crates/app/src/lib.rs, `store_reason` and `queue_mutate`).
const newerDatabase = 'database from a newer version (3 > 2)'
const unreadableQueue = 'coda del piano illeggibile'
const pendingGoals = 3

// contracts/payload/queue.with_rows.json's rows, with achievement 1 — done — above them: the
// completed row the view leaves out, which is why a move names a row and not an index.
const seed = (): StoredRow[] => [
  { achievement: 1, wanted: true, origins: [], stepsNotQueued: 0 },
  { achievement: 480, wanted: false, origins: [55], stepsNotQueued: 0 },
  { achievement: 55, wanted: true, origins: [], stepsNotQueued: 0 },
  { achievement: 69, wanted: true, origins: [], stepsNotQueued: 1 },
]

// The stored queue lives for the page's life, like the chosen profile.
let stored: StoredRow[] | null = null
let imported = false

export const resetQueue = (): void => {
  stored = null
  imported = false
}

const rowsFor = (scenario: QueueScenario): StoredRow[] => {
  stored ??= scenario === QueueScenario.Rows ? seed() : []
  return stored
}

const pending = (scenario: QueueScenario): number =>
  scenario === QueueScenario.Empty && !imported ? pendingGoals : 0

const byId = (nodes: UnlockNode[]): Map<number, UnlockNode> =>
  new Map(
    nodes.flatMap((n) =>
      n.achievement.kind === 'known' ? [[n.achievement.id, n] as const] : [],
    ),
  )

// A row requires another when a character it's missing is one the other unlocks: 55 (beat
// Chest with Samson) requires 480 (which unlocks Samson). The graph's relation is richer; this
// is enough to watch the repair.
const requiresFrom =
  (index: Map<number, UnlockNode>): Requires =>
  (a, b) =>
    (index.get(a)?.missing ?? []).some(
      (m) =>
        m.kind === 'character' &&
        (index.get(b)?.unlocks ?? []).some(
          (u) => u.kind === 'character' && u.id === m.id,
        ),
    )

// What crates/ipc/src/queue.rs `queue_view` builds: done rows and rows the catalog doesn't know
// are left out, and each is said.
const viewOf = (
  rows: StoredRow[],
  nodes: UnlockNode[],
  goals: number,
): QueueView => {
  const index = byId(nodes)
  const done = rows.filter((r) => index.get(r.achievement)?.done === true)
  const diagnostics: QueueDiagnostic[] = [
    ...(goals > 0 ? [{ kind: 'goalsPending' as const, count: goals }] : []),
    ...rows
      .filter((r) => !index.has(r.achievement))
      .map((r) => ({ kind: 'unresolved' as const, achievement: r.achievement })),
    ...(done.length > 0
      ? [
          {
            kind: 'completed' as const,
            count: done.length,
            wanted: done.filter((r) => r.wanted).map((r) => r.achievement),
          },
        ]
      : []),
  ]
  return {
    rows: rows.flatMap((r) => {
      const node = index.get(r.achievement)
      return node && !node.done
        ? [
            {
              node,
              wanted: r.wanted,
              origins: r.origins,
              stepsNotQueued: r.stepsNotQueued,
            },
          ]
        : []
    }),
    diagnostics,
    storeAvailable: true,
  }
}

export const readQueue = ({
  scenario,
  withCatalog,
  nodes,
}: QueueOptions): QueueView => {
  switch (scenario) {
    case QueueScenario.Unavailable:
      return {
        rows: [],
        diagnostics: [{ kind: 'storeUnavailable', reason: newerDatabase }],
        storeAvailable: false,
      }
    case QueueScenario.Unreadable:
      return {
        rows: [],
        diagnostics: [{ kind: 'unreadable' }],
        storeAvailable: true,
      }
    case QueueScenario.Rows:
    case QueueScenario.Empty:
      // Without a catalog the app can't tell which goals are pending, and reports none.
      return withCatalog
        ? viewOf(rowsFor(scenario), nodes, pending(scenario))
        : { rows: [], diagnostics: [{ kind: 'noCatalog' }], storeAvailable: true }
    default:
      return assertNever(scenario)
  }
}

// The order `queue_mutate` refuses in: no graph first, then a database it can't use.
const refusal = ({ scenario, withCatalog }: QueueOptions): IpcError | null => {
  if (!withCatalog) return { kind: 'catalogUnavailable' }
  if (scenario === QueueScenario.Unavailable)
    return { kind: 'storeUnavailable', reason: newerDatabase }
  if (scenario === QueueScenario.Unreadable)
    return { kind: 'storeUnavailable', reason: unreadableQueue }
  return null
}

const mutate = (
  options: QueueOptions,
  edit: (rows: StoredRow[], requires: Requires) => StoredRow[],
): Promise<QueueView> => {
  const refused = refusal(options)
  if (refused) return Promise.reject(refused)
  stored = edit(rowsFor(options.scenario), requiresFrom(byId(options.nodes)))
  return Promise.resolve(readQueue(options))
}

// A wish with no chain: the fixture has no graph to read one from.
export const addToQueue = (o: QueueOptions, achievement: number) =>
  mutate(o, (rows) =>
    rows.some((r) => r.achievement === achievement)
      ? rows.map((r) =>
          r.achievement === achievement ? { ...r, wanted: true } : r,
        )
      : [...rows, { achievement, wanted: true, origins: [], stepsNotQueued: 0 }],
  )

// crates/plan/src/edit.rs `remove`: the row goes, it leaves every origin list, and a step
// nothing keeps any more goes with it.
export const removeFromQueue = (o: QueueOptions, achievement: number) =>
  mutate(o, (rows) =>
    rows
      .filter((r) => r.achievement !== achievement)
      .map((r) => ({
        ...r,
        origins: r.origins.filter((id) => id !== achievement),
      }))
      .filter((r) => r.wanted || r.origins.length > 0),
  )

export const moveInQueue = (
  o: QueueOptions,
  achievement: number,
  after: number | null,
) =>
  mutate(o, (rows, requires) =>
    moveAfter(
      rows.map((r) => r.achievement),
      achievement,
      after,
      requires,
    ).flatMap((id) => rows.filter((r) => r.achievement === id)),
  )

export const importGoals = (o: QueueOptions) =>
  mutate(o, (rows) => {
    imported = true
    return rows.length > 0 ? rows : seed()
  })
```

In `index.ts`: `const QueueParam = 'queue'`; `currentQueueScenario()` like `currentScenario()` (default `QueueScenario.Rows`); `resetFixtures` also calls `resetQueue()`; and

```ts
// The queue's nodes are the Unlock view's, as in the app: the same node on both screens.
const queueOptions = async (): Promise<QueueOptions> => ({
  scenario: currentQueueScenario(),
  withCatalog: catalogShown(),
  nodes: (await graph()).unlock.nodes,
})
const achievementArg = (args: CommandArgs | undefined): number =>
  Number(args?.achievement)
const afterArg = (args: CommandArgs | undefined): number | null =>
  args?.after === null || args?.after === undefined ? null : Number(args.after)
```

with handlers

```ts
  [Command.Queue]: (_args, scenario) =>
    whenActive(scenario, async () => readQueue(await queueOptions())),
  [Command.QueueAdd]: (args, scenario) =>
    whenActive(scenario, async () =>
      addToQueue(await queueOptions(), achievementArg(args)),
    ),
  [Command.QueueRemove]: (args, scenario) =>
    whenActive(scenario, async () =>
      removeFromQueue(await queueOptions(), achievementArg(args)),
    ),
  [Command.QueueMove]: (args, scenario) =>
    whenActive(scenario, async () =>
      moveInQueue(await queueOptions(), achievementArg(args), afterArg(args)),
    ),
  [Command.QueueImportGoals]: (_args, scenario) =>
    whenActive(scenario, async () => importGoals(await queueOptions())),
```

- [x] **Step 4: Run** `pnpm --filter ui exec vitest run src/lib/ipc` → green (the transport test still expects `Command.Plan` to have no fixture); typecheck, lint, scan by exit code. Commit `feat(ui): the queue's fixtures, a queue that repairs in memory` and push.

### Task 5: The queue store, and one text for every IPC error

**Files:** Create `ui/src/stores/queue.ts`, `ui/src/stores/queue.test.ts`, `ui/src/composables/useIpcErrorText.ts`, `ui/src/components/plan/QueueError.vue`; Modify `ui/src/lib/constants/stores.ts`, `ui/src/screens/profile/ProfileError.vue`, `it.ts`, `en.ts`

**Interfaces:** Produces `useQueueStore()` with `view: QueueView | null`, `status: LoadStatus`, `error: IpcError | null`, `busy: boolean`, `mutationFailed: boolean`, `mutationError: IpcError | null`, `lastMove: { achievement: number; after: number | null } | null`, `load()`, `add(id)`, `remove(id)`, `move(id, after)`, `importGoals()`; `useIpcErrorText(): { errorText(e: IpcError | null): string }`; `<QueueError :error>`.

- [x] **Step 1: Messages** — move `profile.errors.{noBackend, noActiveProfile, unknownProfile, unreadableSave, settingsNotWritable, unknownTarget, catalogUnavailable, storeUnavailable, wikiUnavailable}` to a top-level `ipcErrors` in both languages (`profile.errors` keeps `title` and `retry`); add `queue: { inQueue: 'in coda', inPlan: 'già nella coda del Piano', add: 'Aggiungi alla coda', addShort: 'Aggiungi', remove: 'Togli dalla coda', removeShort: 'Togli', errorTitle: 'La coda non è cambiata' }` and the English `queue: { inQueue: 'queued', inPlan: 'already in the Plan’s queue', add: 'Add to the queue', addShort: 'Add', remove: 'Remove from the queue', removeShort: 'Remove', errorTitle: 'The queue didn’t change' }`.

- [x] **Step 2: The composable** — `useIpcErrorText.ts` holds ProfileError's switch unchanged, reading `ipcErrors.*`:

```ts
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import type { IpcError } from '@/lib/ipc/types'

// One sentence per IpcError, for every screen that has to say why a command failed. `null` is
// a failure that isn't an IpcError at all: the backend never answered.
export const useIpcErrorText = () => {
  const { t } = useMessages()
  const errorText = (e: IpcError | null): string => {
    if (e === null) return t('ipcErrors.noBackend')
    switch (e.kind) {
      case 'noActiveProfile':
        return t('ipcErrors.noActiveProfile')
      case 'unknownProfile':
        return t('ipcErrors.unknownProfile')
      case 'unreadableSave':
        return `${t('ipcErrors.unreadableSave')} ${e.reason}`
      case 'settingsNotWritable':
        return `${t('ipcErrors.settingsNotWritable')} ${e.reason}`
      case 'unknownTarget':
        return t('ipcErrors.unknownTarget')
      case 'catalogUnavailable':
        return t('ipcErrors.catalogUnavailable')
      case 'storeUnavailable':
        return `${t('ipcErrors.storeUnavailable')} ${e.reason}`
      case 'wikiUnavailable':
        return t('ipcErrors.wikiUnavailable')
      default:
        return assertNever(e)
    }
  }
  return { errorText }
}
```

`ProfileError.vue` keeps its template and computes `message` as `errorText(props.error)`. `QueueError.vue`: a destructive `Alert` with `TriangleAlertIcon`, title `queue.errorTitle`, description `errorText(error)`; prop `error: IpcError | null`.

- [x] **Step 3: Failing store test** — `stores/queue.test.ts`:

```ts
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { resetFixtures } from '@/lib/ipc/fixtures'
import { LoadStatus } from './profile'
import { useQueueStore } from './queue'

// Through the development fixtures, as the browser runs it: Vitest is a development build with
// no Tauri, so every command answers from lib/ipc/fixtures.
const ids = (store: ReturnType<typeof useQueueStore>) =>
  store.view?.rows.map((r) =>
    r.node.achievement.kind === 'known' ? r.node.achievement.id : null,
  )

beforeEach(() => {
  setActivePinia(createPinia())
  resetFixtures()
})
afterEach(() => {
  vi.unstubAllGlobals()
})

describe('useQueueStore', () => {
  it('loads the queue', async () => {
    const store = useQueueStore()
    await store.load()
    expect(store.status).toBe(LoadStatus.Ready)
    expect(ids(store)).toEqual([480, 55, 69])
  })

  it('replaces the view with a move’s answer and remembers the move', async () => {
    const store = useQueueStore()
    await store.load()
    await store.move(480, 69)
    expect(ids(store)).toEqual([69, 480, 55])
    expect(store.lastMove).toEqual({ achievement: 480, after: 69 })
    expect(store.busy).toBe(false)
  })

  it('keeps the view and says why when a write is refused', async () => {
    const store = useQueueStore()
    await store.load()
    vi.stubGlobal('location', { search: '?catalog=none' })
    await store.add(484)
    expect(ids(store)).toEqual([480, 55, 69])
    expect(store.mutationFailed).toBe(true)
    expect(store.mutationError).toEqual({ kind: 'catalogUnavailable' })
    expect(store.lastMove).toBeNull()
  })
})
```

Run `pnpm --filter ui exec vitest run src/stores/queue.test.ts` → FAIL (no store).

- [x] **Step 4: The store** — `StoreId.Queue: 'queue'`; `stores/queue.ts`:

```ts
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { isIpcError } from '@/lib/ipc/errors'
import {
  queue as readQueue,
  queueAdd,
  queueImportGoals,
  queueMove,
  queueRemove,
} from '@/lib/ipc/queue'
import type { IpcError, QueueView } from '@/lib/ipc/types'
import { LoadStatus } from './profile'

export interface QueueMove {
  achievement: number
  after: number | null
}

// The plan queue of the active profile, read by the Plan, Next steps and Unlock. Every write
// answers with the whole view, which replaces what is shown: a move can reorder much of the
// queue, and the backend's order is the truth. A refused write leaves the view as it was.
export const useQueueStore = defineStore(StoreId.Queue, () => {
  const view = ref<QueueView | null>(null)
  const status = ref<LoadStatus>(LoadStatus.Idle)
  const error = ref<IpcError | null>(null)
  const busy = ref(false)
  const mutationFailed = ref(false)
  const mutationError = ref<IpcError | null>(null)
  // The last move that succeeded, so the Plan can say where the row stopped.
  const lastMove = ref<QueueMove | null>(null)

  const load = async (): Promise<void> => {
    view.value = null
    status.value = LoadStatus.Loading
    error.value = null
    mutationFailed.value = false
    mutationError.value = null
    lastMove.value = null
    try {
      view.value = await readQueue()
      status.value = LoadStatus.Ready
    } catch (e) {
      error.value = isIpcError(e) ? e : null
      status.value = LoadStatus.Failed
    }
  }

  const write = async (run: () => Promise<QueueView>): Promise<boolean> => {
    busy.value = true
    lastMove.value = null
    try {
      view.value = await run()
      mutationFailed.value = false
      mutationError.value = null
      return true
    } catch (e) {
      mutationFailed.value = true
      mutationError.value = isIpcError(e) ? e : null
      return false
    } finally {
      busy.value = false
    }
  }

  const add = async (achievement: number): Promise<void> => {
    await write(() => queueAdd(achievement))
  }
  const remove = async (achievement: number): Promise<void> => {
    await write(() => queueRemove(achievement))
  }
  const importGoals = async (): Promise<void> => {
    await write(() => queueImportGoals())
  }
  const move = async (
    achievement: number,
    after: number | null,
  ): Promise<void> => {
    const moved = await write(() => queueMove(achievement, after))
    if (moved) lastMove.value = { achievement, after }
  }

  return {
    view,
    status,
    error,
    busy,
    mutationFailed,
    mutationError,
    lastMove,
    load,
    add,
    remove,
    move,
    importGoals,
  }
})
```

- [x] **Step 5: Run** the store test → green; `pnpm ui:test`, typecheck, lint, scan by exit code. Commit `feat(ui): the queue store, and one text for every IPC error` and push.

### Task 6: The Plan screen

**Files:** Create `ui/src/screens/PlanScreen.vue`, `ui/src/screens/plan/{PlanAlerts.vue, QueueCard.vue, QueueRow.vue, QueueFootnotes.vue, ProposalAside.vue}`; Modify `spacing.css`, `ui/src/lib/constants/eventKeys.ts`, `ui/src/router/routes.ts`, `routeTable.ts`, `it.ts`, `en.ts`

**Interfaces:** Consumes Tasks 2–5.

- [x] **Step 1: tokens and keys** — `--spacing-plan-aside: 248px;` under a comment "the Plan's proposal column (Schermate.dc.html, 246px on the 4px grid)"; `EventKey.ArrowUp: 'ArrowUp'`, `EventKey.ArrowDown: 'ArrowDown'`.

- [x] **Step 2: messages** — `plan.*` in both languages:

```ts
  plan: {
    intro:
      "Gli obiettivi sono l'insieme di ciò che vuoi, la coda è l'ordine in cui intendi farlo: le righe che hai chiesto, più i prerequisiti che si sono tirate dietro. Trascini una riga e la coda si ripara attorno al vincolo: i prerequisiti sono un muro contro cui si ferma, non un rifiuto.",
    summary: { rows: 'righe', wanted: 'chieste', pulledIn: 'tirate dentro' },
    queueTitle: 'La coda',
    hint: {
      idle: 'trascina per riordinare — una mossa ripara, non fallisce',
      dragging: 'rilascia dove vuoi: la coda si ripara',
      stoppedUnder: 'si è fermata sotto',
      prerequisite: 'è un prerequisito',
    },
    row: {
      move: 'Sposta la riga (Alt+↑, Alt+↓)',
      wanted: 'chiesta',
      serves: 'serve',
      unlocks: 'sblocca',
      fanOut: 'sblocca',
      outsideQueue: 'passi fuori dalla coda',
      hint: 'indizio del gioco:',
    },
    achievement: 'achievement',
    empty: 'La coda è vuota.',
    emptyHint: 'Aggiungi una riga dalla proposta qui accanto, o da Unlock.',
    completed: { closed: 'righe chiuse giocando', wanted: 'fra quelle chieste' },
    unresolved: 'non è più nel catalogo',
    alerts: {
      storeUnavailableTitle: 'Il piano non è disponibile',
      unreadableTitle: 'La coda salvata non si legge con questa versione',
      unreadable:
        'Resta com’è nel file e non viene sovrascritta: una versione più recente dell’app potrebbe saperla leggere.',
      noCatalogTitle: 'Serve il gioco installato',
      noCatalog:
        'Senza catalogo non si sa quale achievement sia ogni riga né cosa le manchi: la coda resta salvata e torna appena il gioco c’è.',
      goalsPendingTitle: 'Obiettivi salvati da importare',
      goalsPending:
        'Obiettivi salvati prima che esistesse la coda: nessuno li sposta da solo.',
      import: 'Importa nella coda',
    },
    aside: {
      title: 'Prossimi passi',
      intro:
        'Righe sbloccabili adesso, ordinate per quante cose aprono. Non sono la tua coda: sono la proposta.',
      empty: 'Nessuna proposta adesso.',
    },
  },
```

English: the same keys, translated in the tone of `en.ts`.

- [x] **Step 3: `QueueRow.vue`** — props `row: QueueRow`, `rows: QueueRow[]`, `position: number`, `dragging: boolean`, `busy: boolean`; emits `grab: [e: PointerEvent]`, `step: [e: KeyboardEvent]`, `remove: []`. Layout, one flex row `items-start gap-3 px-3 py-2.5`, `opacity-disabled` when dragging:
  - handle: `<Button :variant="ButtonVariant.Ghost" :size="ButtonSize.Icon" :aria-label="t('plan.row.move')" :disabled="busy" class="cursor-grab touch-none" @pointerdown="emit('grab', $event)" @keydown="emit('step', $event)"><GripVerticalIcon /></Button>`;
  - position in `w-4 text-label text-subtle-foreground tabular-nums text-right`;
  - `<AchievementArt :url="…iconUrl" :size="ArtSize.Thumb" />`;
  - a column: text (`text-row text-foreground`); the hint as `{{ t('plan.row.hint') }} {{ hint }}` in `text-caption text-subtle-foreground` when there is one; a wrap of badges — `Wanted` "chiesta" when `row.wanted`; one `Tag` per `originRows(row, rows)`: `serve «text»`, text from `knownText(origin.row.node)` or `${t('plan.achievement')} ${origin.id}`; a `Tag` for the first unlocked target `sblocca name · kind` (`unlockKindText[targetKind(target)]`);
  - a right column `items-end gap-1`: `<NodeStateBadge :node="row.node" />`; `sblocca: N` in `text-label text-subtle-foreground` when `graph.fanOut > 0`; `passi fuori dalla coda: N` in `text-label text-foreground-soft` when `stepsNotQueued > 0`;
  - remove: `<Button v-if="row.wanted" :variant="ButtonVariant.Ghost" :size="ButtonSize.Icon" :aria-label="t('queue.remove')" :disabled="busy" @click="emit('remove')"><XIcon /></Button>`.
  The texts with `«»` and `:` are composed in `computed`s in the script, as `StepCard` composes its text.

- [x] **Step 4: `QueueCard.vue`** — props `rows`, `diagnostics: QueueDiagnostic[]`, `nodes: UnlockNode[]`, `busy`, `lastMove: QueueMove | null`; emits `move: [achievement: number, after: number | null]`, `remove: [achievement: number]`. A `Card`; a `CardHeader` with `CardTitle` "La coda" and the hint in `text-caption`; the list; `QueueFootnotes`; when `rows` is empty, an `EmptyCategory` with `plan.empty` and `plan.emptyHint` under it. The drag, following `TabStrip`:

```ts
interface Drag {
  from: number
  startY: number
  moving: boolean
}
interface Drop {
  index: number
  edge: DropEdge
}

// Pixels the pointer travels before a press on the grip becomes a drag: below it, a click.
const dragThreshold = 4
const rowSelector = '[data-queue-row]'

const list = ref<HTMLElement | null>(null)
const drag = ref<Drag | null>(null)
const drop = ref<Drop | null>(null)
// Read once when a press becomes a drag: nothing moves until the drop.
let rowRects: DOMRect[] = []

const ids = computed(() => props.rows.map(rowId))
const rowElements = (): HTMLElement[] =>
  list.value ? [...list.value.querySelectorAll<HTMLElement>(rowSelector)] : []

const dropAt = (y: number): Drop | null => {
  for (const [index, r] of rowRects.entries()) {
    if (y >= r.top && y < r.bottom)
      return { index, edge: dropEdge(y, r.top, r.height) }
  }
  return null
}

const anchorOf = (d: Drag, target: Drop | null): Anchor | null =>
  target ? dropAnchor(ids.value, d.from, target.index, target.edge) : null

// The gap the line is drawn in: only for a drop that would move something.
const gap = computed((): number | null => {
  const d = drag.value
  const target = drop.value
  if (!d?.moving || !target || !anchorOf(d, target)) return null
  return target.edge === DropEdge.Above ? target.index : target.index + 1
})

const onGrab = (index: number, e: PointerEvent) => {
  if (e.button !== 0 || props.busy) return
  drag.value = { from: index, startY: e.clientY, moving: false }
}

const onPointerMove = (e: PointerEvent) => {
  const d = drag.value
  if (!d) return
  if (!d.moving) {
    if (Math.abs(e.clientY - d.startY) < dragThreshold) return
    d.moving = true
    rowRects = rowElements().map((el) => el.getBoundingClientRect())
    list.value?.setPointerCapture(e.pointerId)
  }
  drop.value = dropAt(e.clientY)
}

const onPointerUp = (e: PointerEvent) => {
  const d = drag.value
  const target = drop.value
  drag.value = null
  drop.value = null
  rowRects = []
  if (list.value?.hasPointerCapture(e.pointerId))
    list.value.releasePointerCapture(e.pointerId)
  if (!d?.moving) return
  const anchor = anchorOf(d, target)
  const moved = ids.value[d.from]
  if (anchor && moved !== undefined) emit('move', moved, anchor.after)
}

const directionOf = (key: string): StepDirection | null => {
  if (key === EventKey.ArrowUp) return StepDirection.Up
  if (key === EventKey.ArrowDown) return StepDirection.Down
  return null
}

const onStep = (index: number, e: KeyboardEvent) => {
  const direction = e.altKey ? directionOf(e.key) : null
  if (!direction || props.busy) return
  e.preventDefault()
  const anchor = stepAnchor(ids.value, index, direction)
  const moved = ids.value[index]
  if (anchor && moved !== undefined) emit('move', moved, anchor.after)
}

const hint = computed((): string => {
  if (drag.value?.moving) return t('plan.hint.dragging')
  const m = props.lastMove
  const wall = m ? stoppedUnder(props.rows, m.achievement, m.after) : null
  if (!wall) return t('plan.hint.idle')
  const text = knownText(wall.node) ?? `${t('plan.achievement')} ${rowId(wall)}`
  return `${t('plan.hint.stoppedUnder')} «${text}»: ${t('plan.hint.prerequisite')}`
})
```

  Template: `<div ref="list" class="flex flex-col py-1" @pointermove="onPointerMove" @pointerup="onPointerUp" @pointercancel="onPointerUp">`; each row a `<div data-queue-row class="relative border-b border-hairline last:border-b-0">` holding `<span v-if="gap === index" class="absolute inset-x-0 -top-px h-0.5 bg-primary" />`, the `QueueRow` (`:dragging="drag?.moving === true && drag.from === index"`, `@grab="onGrab(index, $event)"`, `@step="onStep(index, $event)"`, `@remove="emit('remove', rowId(row))"`), and for the last row `<span v-if="gap === rows.length" class="absolute inset-x-0 -bottom-px h-0.5 bg-primary" />`.

- [x] **Step 5: `QueueFootnotes.vue`** — props `diagnostics`, `nodes`, `busy`; emits `remove: [achievement: number]`. Under the rows, `border-t border-hairline px-3 py-2 text-caption text-foreground-soft`: for `completed`, a `CheckIcon` in `text-state-done-foreground` and `righe chiuse giocando: N`, followed when `wanted` isn't empty by ` · fra quelle chieste: ` and the texts (`achievementText(nodes, id) ?? achievement N`) joined by `, `; for each `unresolved`, `achievement N non è più nel catalogo` and a `Button` ghost compact `queue.removeShort` emitting `remove`. Nothing when neither is present. The switch over `diagnostics` filters by `kind` with type guards; the other kinds belong to `PlanAlerts`.

- [x] **Step 6: `PlanAlerts.vue`** — props `view: QueueView`, `busy`; emits `import: []`. One `Alert` per diagnostic of the four kinds, in the order the view lists them: `storeUnavailable` destructive with the title and `reason`; `unreadable` and `noCatalog` default with title and text; `goalsPending` default with `Obiettivi salvati da importare: N`, its text, and an outline `Button` `plan.alerts.import` (`:disabled="busy"`). A `computed` narrows `view.diagnostics` to these kinds with an exhaustive `switch` returning `true`/`false`, `assertNever` in the default.

- [x] **Step 7: `ProposalAside.vue`** — props `steps: UnlockNode[]`, `queued: Set<number>`, `canWrite: boolean`, `busy: boolean`; emits `add: [achievement: number]`. A `Card` with `CardHeader`/`CardTitle` "Prossimi passi"; in a `CardContent`, the intro in `text-caption text-foreground-soft`; one row per step (`flex items-center gap-2`): `AchievementArt` thumb, the text truncated, the fan-out in `text-label text-state-now-foreground tabular-nums`, then `queue.inQueue` in `text-label text-state-done-foreground` when `isQueued`, or an outline compact `Button` `queue.addShort` when `canWrite && canQueue(step, queued)`. No steps: `EmptyValue` `plan.aside.empty`.

- [x] **Step 8: `PlanScreen.vue`**:

```ts
const queue = useQueueStore()
const graph = useGraphStore()
const { t } = useMessages()

useOnActiveProfile(async () => {
  await Promise.all([queue.load(), graph.load()])
})

// The queue card needs a queue that could be read: no database, an unreadable document and no
// catalog each say so in an alert instead of an empty list.
const readable = computed((): boolean => {
  const v = queue.view
  return (
    v !== null &&
    v.storeAvailable &&
    !v.diagnostics.some((d) => d.kind === 'unreadable' || d.kind === 'noCatalog')
  )
})
const summary = computed(() => queueSummary(queue.view?.rows ?? []))
const queued = computed(() => queuedIds(queue.view))
const nodes = computed(() => graph.unlock?.nodes ?? [])
```

  Template: `max-w-300 flex-col gap-4`; `ScreenHeader` (`MapIcon`, `routes.plan`, `plan.intro`); `ProfileError` on `queue.status === Failed` with retry `queue.load()`; with a view: the summary line `righe: N · chieste: M · tirate dentro: K` (`text-caption text-foreground-soft tabular-nums`, only when `readable`), `PlanAlerts` (`@import="queue.importGoals()"`), `QueueError` when `queue.mutationFailed`, and when `readable` a `flex flex-col items-start gap-4 lg:flex-row` holding `QueueCard` (`class="w-full min-w-0 flex-1"`, `@move="queue.move"`, `@remove="queue.remove"`) and `ProposalAside` (`class="w-full lg:w-plan-aside lg:shrink-0"`, `:steps="graph.steps?.steps ?? []"`, `:can-write="queue.view.storeAvailable"`, `@add="queue.add"`); otherwise three `Skeleton`s.

- [x] **Step 9: route** — `routes.ts` maps `RouteName.Plan` to `PlanScreen`; `routeTable.ts` drops `[RouteName.Plan]` from `routeArrives`; if `placeholder.graph` has no user left, remove it from both message files.

- [x] **Step 10: verify** — typecheck, lint, format, scan, `ui:test`, each by exit code. Commit `feat(ui): the Plan, a queue you drag and a repair that says where it stopped` and push.

### Task 7: In the queue, on Next steps and Unlock

**Files:** Modify `ui/src/screens/NextStepsScreen.vue`, `screens/nextSteps/StepCard.vue`, `UnlockScreen.vue`, `screens/unlock/UnlockTable.vue`, `UnlockRow.vue`, `spacing.css`, `utilities.css`

- [x] **Step 1: tokens** — `--spacing-unlock-queue: 40px;` beside the other Unlock columns; `grid-cols-unlock` gains `var(--spacing-unlock-queue)` as its last column.
- [x] **Step 2: Next steps** — the screen reads `useQueueStore()` too and loads both in `useOnActiveProfile(async () => { await Promise.all([graph.load(), queue.load()]) })`; `queued = computed(() => queuedIds(queue.view))`, `canWrite = computed(() => queue.view?.storeAvailable === true)`; `QueueError` under the header when `queue.mutationFailed`. `StepCard` gains props `queued: boolean`, `canAdd: boolean`, `busy: boolean` and emits `add`; under the state badge: `queue.inPlan` in `text-caption text-state-done-foreground` when `queued`, else an outline compact `Button` with `ListPlusIcon` and `queue.add` when `canAdd` (`:disabled="busy"`). The screen passes `:queued="isQueued(step, queued)"`, `:can-add="canWrite && canQueue(step, queued)"`, `@add="queue.add(nodeSlot(step))"`.
- [x] **Step 3: Unlock** — the screen loads and reads the queue the same way and shows `QueueError`; `UnlockTable` gains props `queued: Set<number>`, `canWrite: boolean`, `busy: boolean` and emits `add: [achievement: number]`, adds an empty `<span />` to the header, and passes each row `:queued="isQueued(node, queued)"`, `:can-add="canWrite && canQueue(node, queued)"`, `:busy`, `@add="emit('add', nodeSlot(node))"`. `UnlockRow` gains those props and `add`; the slot line becomes `{{ t('graph.slot') }} {{ nodeSlot(node) }}<template v-if="queued"> · {{ t('queue.inQueue') }}</template>`; a last cell `<span class="flex justify-center"><Button v-if="canAdd" :variant="ButtonVariant.Ghost" :size="ButtonSize.IconCompact" :aria-label="t('queue.add')" :disabled="busy" @click="emit('add')"><ListPlusIcon /></Button></span>`.
- [x] **Step 4: verify** — typecheck, lint, format, scan, `ui:test` by exit code (the `unlockLayout` test still pins the row height). Commit `feat(ui): in the queue, and one click to put it there, on Next steps and Unlock` and push.

### Task 8: Looked at, handed on, checked

- [x] **Step 1: visual check** — `pnpm ui:dev` and a throwaway headless-Chrome script outside the repository (DevTools protocol, as in 3.2 and 3.3a), in the job's temp folder: the Plan's three rows with "chiesta", "serve «…»", "passi fuori dalla coda: 1" and the completed line; a pointer drag of 480 below 69 (55 follows); a drag of 55 to the top (it stays under 480 and the hint names it); `Alt+↑` on 69; removing 55 (480 goes too); adding from the aside; "in coda" and the add buttons on Next steps and Unlock, and the add reaching the Plan; `?queue=empty` and the import; `?queue=unavailable`; `?queue=unreadable`; `?catalog=none`. Each finding fixed with a test where it is logic, recorded in the spec's deviations.
- [x] **Step 2: documents** — `docs/STATUS.md` (3.3b ticked, 3.3 closed, the session log: the move's contract fix and why); `DESIGN-BRIEF.md` §7.6 (a drop names the row it lands under: `queueMove(achievement, after)`, and why an index can't be); `docs/frontend-conventions.md` (`lib/plan/`, `components/plan/`, `stores/queue.ts`, `useIpcErrorText`, `ipcErrors.*`, the `Wanted` badge); `CLAUDE.md` (the `plan` module line: a move names the row it lands under; the State paragraph); the spec's deviations while executing; this plan's checkboxes.
- [x] **Step 3: production build** — `pnpm --filter ui build`: no fixture, payload or pack image in `ui/dist`.
- [ ] **Step 4: `sh scripts/check`** → "all green", exit 0. Commit `docs: cycle 3.3b lands, the Plan and the queue` and push; merge `feature/design-system-screens` into `develop` with `--no-ff` ("merge: screens 3.3b, the Plan and the queue, into develop") through a temporary worktree, push `develop`, fast-forward the feature branch and push it.
