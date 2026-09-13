import { describe, expect, it } from 'vitest'
import { RouteName } from '@/router/routeTable'
import { readSession, writeSession } from './sessionDocument'

const tab = (name: RouteName) => ({ entries: [{ name }], index: 0 })

describe('the session document', () => {
  it('reads back exactly what it wrote', () => {
    const session = {
      tabs: [tab(RouteName.Goals), tab(RouteName.Wiki)],
      activeIndex: 1,
    }
    expect(readSession(writeSession(session))).toEqual(session)
  })

  it('keeps the query a tab was showing', () => {
    const session = {
      tabs: [
        {
          entries: [{ name: RouteName.Wiki, query: { q: 'brimstone' } }],
          index: 0,
        },
      ],
      activeIndex: 0,
    }
    expect(readSession(writeSession(session))).toEqual(session)
  })

  it('is nothing at all when there is nothing stored', () => {
    expect(readSession(null)).toBeNull()
  })

  it('is nothing at all when the document is not JSON', () => {
    expect(readSession('{ not json')).toBeNull()
  })

  it('is nothing at all when the version is one it does not know', () => {
    // A document from a newer app. Landing tab, not a guess at what the fields mean.
    expect(readSession('{"version":99,"tabs":[],"activeIndex":0}')).toBeNull()
  })

  it('drops the one tab it cannot read and keeps the others', () => {
    const raw = JSON.stringify({
      version: 1,
      tabs: [
        tab(RouteName.Goals),
        { entries: [{ name: 'a-screen-that-was-renamed' }], index: 0 },
        tab(RouteName.Wiki),
      ],
      activeIndex: 2,
    })
    const read = readSession(raw)
    expect(read?.tabs).toEqual([tab(RouteName.Goals), tab(RouteName.Wiki)])
    // The active tab was the third; with one dropped before it, it is now the second.
    expect(read?.activeIndex).toBe(1)
  })

  it('answers an empty session when every tab dropped', () => {
    const raw = JSON.stringify({
      version: 1,
      tabs: [{ entries: [{ name: 'gone' }], index: 0 }],
      activeIndex: 0,
    })
    // Not null: the document was readable. Empty, which seedState turns into the landing tab.
    expect(readSession(raw)).toEqual({ tabs: [], activeIndex: 0 })
  })

  it('refuses a tab whose history index points outside its entries', () => {
    const raw = JSON.stringify({
      version: 1,
      tabs: [{ entries: [{ name: RouteName.Goals }], index: 7 }],
      activeIndex: 0,
    })
    expect(readSession(raw)).toEqual({ tabs: [], activeIndex: 0 })
  })

  it('refuses a tab whose history holds one entry it cannot open', () => {
    // A tab is its history: one entry we cannot reach and the back button lies.
    const raw = JSON.stringify({
      version: 1,
      tabs: [
        {
          entries: [{ name: RouteName.Goals }, { name: 'gone' }],
          index: 0,
        },
      ],
      activeIndex: 0,
    })
    expect(readSession(raw)).toEqual({ tabs: [], activeIndex: 0 })
  })
})
