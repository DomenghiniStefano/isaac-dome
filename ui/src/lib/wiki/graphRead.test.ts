import { describe, expect, it } from 'vitest'
import { LoadStatus } from '@/stores/loadStatus'
import { wikiReadsGraph } from './graphRead'

describe('wikiReadsGraph', () => {
  it('reads a graph nobody has read in this window', () => {
    expect(wikiReadsGraph(LoadStatus.Idle)).toBe(true)
  })

  it('leaves a read in flight to finish', () => {
    expect(wikiReadsGraph(LoadStatus.Loading)).toBe(false)
  })

  it('never reads again a graph that is there: a read clears the view first', () => {
    expect(wikiReadsGraph(LoadStatus.Ready)).toBe(false)
  })

  it('does not retry a read that failed, which is how "no profile" answers', () => {
    expect(wikiReadsGraph(LoadStatus.Failed)).toBe(false)
  })
})
