import { assertNever } from '../../assertNever'
import type {
  IpcError,
  QueueDiagnostic,
  QueueRow,
  QueueView,
  UnlockNode,
} from '../types'
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
      .map((r) => ({
        kind: 'unresolved' as const,
        achievement: r.achievement,
      })),
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
  const shown = rows.flatMap((r): QueueRow[] => {
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
  })
  return { rows: shown, diagnostics, storeAvailable: true }
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
        : {
            rows: [],
            diagnostics: [{ kind: 'noCatalog' }],
            storeAvailable: true,
          }
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
export const addToQueue = (
  options: QueueOptions,
  achievement: number,
): Promise<QueueView> =>
  mutate(options, (rows) =>
    rows.some((r) => r.achievement === achievement)
      ? rows.map((r) =>
          r.achievement === achievement ? { ...r, wanted: true } : r,
        )
      : [
          ...rows,
          { achievement, wanted: true, origins: [], stepsNotQueued: 0 },
        ],
  )

// crates/plan/src/edit.rs `remove`: the row goes, it leaves every origin list, and a step
// nothing keeps any more goes with it.
export const removeFromQueue = (
  options: QueueOptions,
  achievement: number,
): Promise<QueueView> =>
  mutate(options, (rows) =>
    rows
      .filter((r) => r.achievement !== achievement)
      .map((r) => ({
        ...r,
        origins: r.origins.filter((id) => id !== achievement),
      }))
      .filter((r) => r.wanted || r.origins.length > 0),
  )

export const moveInQueue = (
  options: QueueOptions,
  achievement: number,
  after: number | null,
): Promise<QueueView> =>
  mutate(options, (rows, requires) =>
    moveAfter(
      rows.map((r) => r.achievement),
      achievement,
      after,
      requires,
    ).flatMap((id) => rows.filter((r) => r.achievement === id)),
  )

export const importGoals = (options: QueueOptions): Promise<QueueView> =>
  mutate(options, (rows) => {
    imported = true
    return rows.length > 0 ? rows : seed()
  })
