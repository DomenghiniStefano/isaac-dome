import { describe, expect, it } from 'vitest'
import { RouteName } from '@/router/routeTable'
import { readSession, writeSession } from './sessionDocument'

const tab = (name: RouteName) => ({
  entries: [{ location: { name } }],
  index: 0,
})

const box = { left: 100, top: 50, width: 1280, height: 800 }

// A version 1 document, as the app wrote it until 3.7b: one window's tabs at the top level.
const v1 = (tabs: unknown[], activeIndex = 0) =>
  JSON.stringify({ version: 1, tabs, activeIndex })

describe('the session document', () => {
  it('reads back exactly what it wrote', () => {
    const windows = [
      {
        tabs: [tab(RouteName.Goals), tab(RouteName.Wiki)],
        activeIndex: 1,
        box,
      },
    ]
    expect(readSession(writeSession({ windows }))?.windows).toEqual(windows)
  })

  it('keeps the query a tab was showing', () => {
    const windows = [
      {
        tabs: [
          {
            entries: [
              { location: { name: RouteName.Wiki, query: { q: 'brimstone' } } },
            ],
            index: 0,
          },
        ],
        activeIndex: 0,
      },
    ]
    expect(readSession(writeSession({ windows }))?.windows).toEqual(windows)
  })

  it('is nothing at all when there is nothing stored', () => {
    expect(readSession(null)).toBeNull()
  })

  it('is nothing at all when the document is not JSON', () => {
    expect(readSession('{ not json')).toBeNull()
  })

  it('is nothing at all when the version is one it does not know', () => {
    // A document from a newer app. Landing tab, not a guess at what the fields mean.
    expect(readSession('{"version":99,"windows":[]}')).toBeNull()
  })

  it('drops the one tab it cannot read and keeps the others', () => {
    const read = readSession(
      v1(
        [
          tab(RouteName.Goals),
          { entries: [{ name: 'a-screen-that-was-renamed' }], index: 0 },
          tab(RouteName.Wiki),
        ],
        2,
      ),
    )
    expect(read?.windows[0]?.tabs).toEqual([
      tab(RouteName.Goals),
      tab(RouteName.Wiki),
    ])
    // The active tab was the third; with one dropped before it, it is now the second.
    expect(read?.windows[0]?.activeIndex).toBe(1)
  })

  it('refuses a tab whose history index points outside its entries', () => {
    const raw = v1([{ entries: [{ name: RouteName.Goals }], index: 7 }])
    expect(readSession(raw)).toBeNull()
  })

  it('refuses a tab whose history holds one entry it cannot open', () => {
    // A tab is its history: one entry we cannot reach and the back button lies.
    const raw = v1([
      { entries: [{ name: RouteName.Goals }, { name: 'gone' }], index: 0 },
    ])
    expect(readSession(raw)).toBeNull()
  })
})

describe('the document is windows of tabs', () => {
  // Losing somebody's tabs on an update is not a thing the app can explain to them afterwards,
  // and this is the one test whose failure costs a real person theirs.
  it('reads a version 1 document as one window with no box', () => {
    const read = readSession(v1([tab(RouteName.Goals), tab(RouteName.Wiki)], 1))
    expect(read?.windows).toHaveLength(1)
    expect(read?.windows[0]?.tabs).toHaveLength(2)
    expect(read?.windows[0]?.activeIndex).toBe(1)
    expect(read?.windows[0]?.box).toBeUndefined()
  })

  it('writes version 2', () => {
    const stored = JSON.parse(
      writeSession({
        windows: [{ tabs: [tab(RouteName.Goals)], activeIndex: 0 }],
      }),
    )
    expect(stored.version).toBe(2)
    expect(stored.windows).toHaveLength(1)
  })

  it('reads the windows it holds, in the order it holds them', () => {
    const windows = [
      { tabs: [tab(RouteName.Goals)], activeIndex: 0, box },
      {
        tabs: [tab(RouteName.Wiki)],
        activeIndex: 0,
        box: { ...box, left: 20 },
      },
    ]
    const read = readSession(writeSession({ windows }))
    expect(read?.windows).toHaveLength(2)
    expect(read?.windows[0]?.box?.left).toBe(100)
    expect(read?.windows[1]?.box?.left).toBe(20)
  })

  it('drops a window whose every tab was unreadable, and keeps the others', () => {
    const raw = JSON.stringify({
      version: 2,
      windows: [
        { tabs: [{ entries: [{ name: 'gone' }], index: 0 }], activeIndex: 0 },
        { tabs: [tab(RouteName.Wiki)], activeIndex: 0 },
      ],
    })
    const read = readSession(raw)
    // Not one empty window and one full one: a window with nothing in it is a window the user
    // never had, and restoring it would open a landing page they did not leave.
    expect(read?.windows).toHaveLength(1)
    expect(read?.windows[0]?.tabs).toEqual([tab(RouteName.Wiki)])
  })

  it('is nothing at all when every window dropped', () => {
    const raw = JSON.stringify({
      version: 2,
      windows: [
        { tabs: [{ entries: [{ name: 'gone' }], index: 0 }], activeIndex: 0 },
      ],
    })
    expect(readSession(raw)).toBeNull()
  })

  // Half a box is not a position: a window placed at a left with no top is a window somewhere
  // nobody asked for.
  it('drops a box that is not four numbers, and keeps the window', () => {
    const raw = JSON.stringify({
      version: 2,
      windows: [
        {
          tabs: [tab(RouteName.Wiki)],
          activeIndex: 0,
          box: { left: 10, top: 20, width: 'wide', height: 800 },
        },
      ],
    })
    const read = readSession(raw)
    expect(read?.windows[0]?.tabs).toHaveLength(1)
    expect(read?.windows[0]?.box).toBeUndefined()
  })

  it('refuses a version 2 document whose windows are not a list', () => {
    expect(readSession('{"version":2,"windows":{}}')).toBeNull()
  })
})

describe('the view a stored entry carries', () => {
  // The document on disk today has entries that are bare locations. Whoever updates the app
  // must not lose the tabs they had open.
  it('reads an entry written before entries had a view', () => {
    const raw = v1([{ entries: [{ name: RouteName.Unlock }], index: 0 }])
    expect(readSession(raw)?.windows[0]?.tabs[0]?.entries[0]).toEqual({
      location: { name: RouteName.Unlock },
    })
  })

  it('reads an entry that carries one', () => {
    const raw = v1([
      {
        entries: [
          { location: { name: RouteName.Unlock }, view: { sort: 'name' } },
        ],
        index: 0,
      },
    ])
    expect(readSession(raw)?.windows[0]?.tabs[0]?.entries[0]?.view).toEqual({
      sort: 'name',
    })
  })

  // B6's open question, answered one notch finer than the rule above it: a tab whose *route* is
  // gone still falls whole, because there is nothing left to open. A tab whose *record* is
  // unreadable opens on its screen and lets the screen say what it has.
  it('drops an unreadable view and keeps the tab', () => {
    const raw = v1([
      {
        entries: [
          { location: { name: RouteName.Unlock }, view: 'not-an-object' },
        ],
        index: 0,
      },
    ])
    const entry = readSession(raw)?.windows[0]?.tabs[0]?.entries[0]
    expect(entry?.location).toEqual({ name: RouteName.Unlock })
    expect(entry?.view).toBeUndefined()
  })

  it('still drops the whole tab when a route is gone', () => {
    const raw = v1(
      [
        { entries: [{ location: { name: 'a-screen-that-left' } }], index: 0 },
        { entries: [{ location: { name: RouteName.Unlock } }], index: 0 },
      ],
      1,
    )
    expect(readSession(raw)?.windows[0]?.tabs).toHaveLength(1)
  })

  // The document is bounded by construction rather than by a number: the cap is 64 KiB and its
  // budget was written for locations alone. Everything in memory keeps its views; only what is
  // stored is pruned — and now in every window, not only in the one main happened to hold.
  it('stores the view of the entry each tab is showing and of no other', () => {
    const one = {
      entries: [
        { location: { name: RouteName.Unlock }, view: { sort: 'name' } },
        { location: { name: RouteName.Collection }, view: { sort: 'id' } },
      ],
      index: 1,
    }
    const stored = JSON.parse(
      writeSession({
        windows: [
          { tabs: [one], activeIndex: 0 },
          { tabs: [one], activeIndex: 0 },
        ],
      }),
    )
    for (const window of stored.windows) {
      expect(window.tabs[0].entries[0].view).toBeUndefined()
      expect(window.tabs[0].entries[1].view).toEqual({ sort: 'id' })
    }
  })

  it('round-trips what it kept', () => {
    const one = {
      entries: [
        { location: { name: RouteName.Unlock }, view: { sort: 'name' } },
      ],
      index: 0,
    }
    const back = readSession(
      writeSession({ windows: [{ tabs: [one], activeIndex: 0 }] }),
    )
    expect(back?.windows[0]?.tabs[0]?.entries[0]?.view).toEqual({
      sort: 'name',
    })
  })
})

describe('the sizes the document remembers', () => {
  const one = [{ tabs: [tab(RouteName.Goals)], activeIndex: 0 }]

  it('round-trips the sidebar width', () => {
    const back = readSession(writeSession({ windows: one, sidebarWidth: 260 }))
    expect(back?.sidebarWidth).toBe(260)
  })

  it('has none when none was ever set', () => {
    const stored = JSON.parse(writeSession({ windows: one }))
    // Absent, not `null`: a key that is there and means nothing is a key somebody has to read.
    expect('sidebarWidth' in stored).toBe(false)
    expect(
      readSession(writeSession({ windows: one }))?.sidebarWidth,
    ).toBeUndefined()
  })

  it('drops a width that is not a finite number, and keeps the windows', () => {
    const raw = JSON.stringify({
      version: 2,
      windows: [{ tabs: [tab(RouteName.Goals)], activeIndex: 0 }],
      sidebarWidth: 'wide',
    })
    const read = readSession(raw)
    expect(read?.windows).toHaveLength(1)
    expect(read?.sidebarWidth).toBeUndefined()
  })

  // Beside `windows`, not inside it and not above it: an app that does not know this key ignores
  // it and is wrong about nothing, which is the only thing the version number is for.
  it('does not move the version', () => {
    const stored = JSON.parse(writeSession({ windows: one, sidebarWidth: 260 }))
    expect(stored.version).toBe(2)
  })

  it('reads a version 1 document, which never had one', () => {
    expect(
      readSession(v1([tab(RouteName.Goals)]))?.sidebarWidth,
    ).toBeUndefined()
  })

  // The bounds are the sidebar's and stay there: a parser that knew 168 and 420 would be a
  // parser holding the design's pixels.
  it('keeps a width outside the sidebar bounds rather than judging it', () => {
    expect(
      readSession(writeSession({ windows: one, sidebarWidth: 9000 }))
        ?.sidebarWidth,
    ).toBe(9000)
  })
})
