import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { effectScope } from 'vue'
import { graphAnswers } from '@/lib/ipc/fixtures/graph'
import { QueueScenario, readQueue, resetQueue } from '@/lib/ipc/fixtures/queue'
import { useQueueStore } from '@/stores/queue'
import { useQueueOffer } from './useQueueOffer'

const nodes = graphAnswers({ withCatalog: true }).unlock.nodes
const view = (scenario: QueueScenario) =>
  readQueue({ scenario, withCatalog: true, nodes })

const inScope = () => {
  const offer = effectScope().run(() => useQueueOffer())
  if (!offer) throw new Error('the scope ran nothing')
  return offer
}

beforeEach(() => {
  setActivePinia(createPinia())
  resetQueue()
})

// What a list next to the queue may offer: which rows are already in it, and whether a `+` can
// be drawn at all.
describe('what the queue lets a screen offer', () => {
  it('offers nothing before the queue has been read', () => {
    const { queued, canWrite } = inScope()
    expect([...queued.value]).toEqual([])
    expect(canWrite.value).toBe(false)
  })

  it('names the queued achievements and lets a readable queue be written', () => {
    const offer = inScope()
    useQueueStore().view = view(QueueScenario.Rows)
    expect([...offer.queued.value].sort((a, b) => a - b)).toEqual([55, 69, 480])
    expect(offer.canWrite.value).toBe(true)
  })

  it('refuses the button when the store will not open', () => {
    const offer = inScope()
    useQueueStore().view = view(QueueScenario.Unavailable)
    expect(offer.canWrite.value).toBe(false)
  })
})
