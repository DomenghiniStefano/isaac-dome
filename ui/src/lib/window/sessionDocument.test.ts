import { describe, expect, it } from 'vitest'
import { RouteName } from '@/router/routeTable'
import { readSession, writeSession } from './sessionDocument'

const tab = (name: RouteName) => ({
  entries: [{ location: { name } }],
  index: 0,
})

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
          entries: [
            { location: { name: RouteName.Wiki, query: { q: 'brimstone' } } },
          ],
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

describe('the view a stored entry carries', () => {
  // The document on disk today has entries that are bare locations. Whoever updates the app
  // must not lose the tabs they had open.
  it('reads an entry written before entries had a view', () => {
    const raw = JSON.stringify({
      version: 1,
      tabs: [{ entries: [{ name: RouteName.Unlock }], index: 0 }],
      activeIndex: 0,
    })
    expect(readSession(raw)?.tabs[0]?.entries[0]).toEqual({
      location: { name: RouteName.Unlock },
    })
  })

  it('reads an entry that carries one', () => {
    const raw = JSON.stringify({
      version: 1,
      tabs: [
        {
          entries: [
            { location: { name: RouteName.Unlock }, view: { sort: 'name' } },
          ],
          index: 0,
        },
      ],
      activeIndex: 0,
    })
    expect(readSession(raw)?.tabs[0]?.entries[0]?.view).toEqual({
      sort: 'name',
    })
  })

  // B6's open question, answered one notch finer than the rule above it: a tab whose *route* is
  // gone still falls whole, because there is nothing left to open. A tab whose *record* is
  // unreadable opens on its screen and lets the screen say what it has.
  it('drops an unreadable view and keeps the tab', () => {
    const raw = JSON.stringify({
      version: 1,
      tabs: [
        {
          entries: [
            { location: { name: RouteName.Unlock }, view: 'not-an-object' },
          ],
          index: 0,
        },
      ],
      activeIndex: 0,
    })
    const entry = readSession(raw)?.tabs[0]?.entries[0]
    expect(entry?.location).toEqual({ name: RouteName.Unlock })
    expect(entry?.view).toBeUndefined()
  })

  it('still drops the whole tab when a route is gone', () => {
    const raw = JSON.stringify({
      version: 1,
      tabs: [
        { entries: [{ location: { name: 'a-screen-that-left' } }], index: 0 },
        { entries: [{ location: { name: RouteName.Unlock } }], index: 0 },
      ],
      activeIndex: 1,
    })
    expect(readSession(raw)?.tabs).toHaveLength(1)
  })

  // The document is bounded by construction rather than by a number: the cap is 64 KiB and its
  // budget was written for locations alone. Everything in memory keeps its views; only what is
  // stored is pruned.
  it('stores the view of the entry each tab is showing and of no other', () => {
    const one = {
      entries: [
        { location: { name: RouteName.Unlock }, view: { sort: 'name' } },
        { location: { name: RouteName.Collection }, view: { sort: 'id' } },
      ],
      index: 1,
    }
    const stored = JSON.parse(writeSession({ tabs: [one], activeIndex: 0 }))
    expect(stored.tabs[0].entries[0].view).toBeUndefined()
    expect(stored.tabs[0].entries[1].view).toEqual({ sort: 'id' })
  })

  it('round-trips what it kept', () => {
    const one = {
      entries: [
        { location: { name: RouteName.Unlock }, view: { sort: 'name' } },
      ],
      index: 0,
    }
    const back = readSession(writeSession({ tabs: [one], activeIndex: 0 }))
    expect(back?.tabs[0]?.entries[0]?.view).toEqual({ sort: 'name' })
  })
})
