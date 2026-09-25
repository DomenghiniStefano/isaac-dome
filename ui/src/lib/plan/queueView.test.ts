import { beforeEach, describe, expect, it } from 'vitest'
import { graphAnswers } from '@/lib/ipc/fixtures/graph'
import { QueueScenario, readQueue, resetQueue } from '@/lib/ipc/fixtures/queue'
import type { QueueView } from '@/lib/ipc/types'
import { membershipChanged, queueMembership, queueReadable } from './queueView'

const nodes = graphAnswers({ withCatalog: true }).unlock.nodes
const view = (scenario: QueueScenario, withCatalog = true): QueueView =>
  readQueue({ scenario, withCatalog, nodes })

beforeEach(resetQueue)

// What the queue holds, as the Goals screen watches it: the suggestions leave out what is
// queued, so they are asked again when that changes — and only then.
describe('queueMembership', () => {
  it('is nothing until the queue has been read', () => {
    expect(queueMembership(null)).toBeNull()
  })

  // The payload's queue holds 480, 55 and 69, in that order.
  it('names what the queue holds, whatever its order', () => {
    const rows = view(QueueScenario.Rows)
    const reordered = { ...rows, rows: [...rows.rows].reverse() }
    expect(queueMembership(rows)).toBe('55,69,480')
    expect(queueMembership(reordered)).toBe(queueMembership(rows))
  })
})

describe('membershipChanged', () => {
  it('is a change only between two reads that differ', () => {
    expect(membershipChanged('1,2', '1')).toBe(true)
    expect(membershipChanged('1,2', '1,2')).toBe(false)
  })

  // A queue arriving with the profile arrives with the graph beside it: nothing to ask again.
  it('is no change when either side was never read', () => {
    expect(membershipChanged('1,2', null)).toBe(false)
    expect(membershipChanged(null, '1,2')).toBe(false)
  })
})

// The queue card needs a queue that could be read; the other cases each say so in an alert.
describe('queueReadable', () => {
  it('reads a queue the store opened and the document parsed', () => {
    expect(queueReadable(view(QueueScenario.Rows))).toBe(true)
    expect(queueReadable(view(QueueScenario.Empty))).toBe(true)
  })

  it('refuses no queue, no database, an unreadable document and no catalog', () => {
    expect(queueReadable(null)).toBe(false)
    expect(queueReadable(view(QueueScenario.Unavailable))).toBe(false)
    expect(queueReadable(view(QueueScenario.Unreadable))).toBe(false)
    expect(queueReadable(view(QueueScenario.Rows, false))).toBe(false)
  })
})
