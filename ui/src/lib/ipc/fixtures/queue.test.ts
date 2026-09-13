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
const options = (
  scenario: QueueScenario = QueueScenario.Rows,
  withCatalog = true,
) => ({ scenario, withCatalog, nodes })
const shown = (view: QueueView) =>
  view.rows.map((r) =>
    r.node.achievement.kind === 'known' ? r.node.achievement.id : null,
  )

beforeEach(resetQueue)

// contracts/payload/queue.with_rows.json, with achievement 1 done above its rows.
describe('the seeded queue', () => {
  it("answers the payload's rows and the completed one", () => {
    const view = readQueue(options())
    expect(shown(view)).toEqual([480, 55, 69])
    expect(
      view.rows.map((r) => [r.wanted, r.origins, r.stepsNotQueued]),
    ).toEqual([
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
    const view = await moveInQueue(options(), 480, 69)
    expect(shown(view)).toEqual([69, 480, 55])
  })

  it('stops 55 under 480 when it is sent to the top', async () => {
    const view = await moveInQueue(options(), 55, null)
    expect(shown(view)).toEqual([480, 55, 69])
  })

  it('keeps an order between reads', async () => {
    await moveInQueue(options(), 69, null)
    expect(shown(readQueue(options()))).toEqual([69, 480, 55])
  })

  it('adds a wish at the end', async () => {
    const view = await addToQueue(options(), 484)
    expect(shown(view)).toEqual([480, 55, 69, 484])
    expect(view.rows[3]?.wanted).toBe(true)
  })

  it('takes a step away with the last wish that kept it', async () => {
    const view = await removeFromQueue(options(), 55)
    expect(shown(view)).toEqual([69])
  })
})

describe("the queue's other states", () => {
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
    const reason = { kind: 'newerSchema', found: 3, supported: 2 }
    expect(readQueue(options(QueueScenario.Unavailable))).toEqual({
      rows: [],
      diagnostics: [{ kind: 'storeUnavailable', reason }],
      storeAvailable: false,
    })
    await expect(
      addToQueue(options(QueueScenario.Unavailable), 484),
    ).rejects.toEqual({ kind: 'storeUnavailable', reason })
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
    await expect(
      moveInQueue(options(QueueScenario.Rows, false), 480, 69),
    ).rejects.toEqual({ kind: 'catalogUnavailable' })
  })
})
