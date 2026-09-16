import { describe, expect, it } from 'vitest'
import { RouteName } from '@/router/routeTable'
import {
  SessionAction,
  decideSessionWrite,
  electWriter,
  mintedAt,
  windowOrder,
} from './sessionWriter'
import type { StoredWindow } from './sessionDocument'

// The shape `newWindowLabel()` mints: `win-` and `Date.now()` in base 36. Written as literals
// rather than minted here, so the expected order comes from the spec and not from the clock.
const first = 'win-m0d3f1' // 1757900000001
const second = 'win-m0d3f2'
const third = 'win-m0d40a'

describe('when a window was minted', () => {
  it('reads the time out of the label', () => {
    expect(mintedAt(first)).toBe(parseInt('m0d3f1', 36))
  })

  it('has no time for main, which was never minted', () => {
    expect(mintedAt('main')).toBeNull()
  })

  it('has no time for a label of another shape', () => {
    expect(mintedAt('win-')).toBeNull()
    expect(mintedAt('window-1')).toBeNull()
    expect(mintedAt('win-not base 36')).toBeNull()
  })
})

describe('the order windows are kept in', () => {
  it('puts main first however the labels arrive', () => {
    expect(windowOrder([second, 'main', first])).toEqual([
      'main',
      first,
      second,
    ])
  })

  it('orders the rest by when they were minted, oldest first', () => {
    expect(windowOrder([third, first, second])).toEqual([first, second, third])
  })

  it('puts a label whose time it cannot read last, and keeps those by name', () => {
    expect(windowOrder(['zz', 'aa', second, 'main'])).toEqual([
      'main',
      second,
      'aa',
      'zz',
    ])
  })
})

describe('who writes the session', () => {
  it('is main when main is open', () => {
    expect(electWriter([first, 'main', second])).toBe('main')
  })

  // The finding this sub-project exists for: nothing prevents main from being closed while
  // other windows live, and under "only main writes" the session stops being written there.
  it('is the oldest surviving window when main has gone', () => {
    expect(electWriter([third, second])).toBe(second)
  })

  it('hands over when the elected window closes', () => {
    expect(electWriter([first, second])).toBe(first)
    expect(electWriter([second])).toBe(second)
  })

  it('is nobody when there are no windows', () => {
    expect(electWriter([])).toBeNull()
  })

  // Every window elects for itself, from the roster it just read. Two windows reading the same
  // set in a different order must reach the same writer, or two of them write at once.
  it('does not depend on the order the labels are given in', () => {
    const one = electWriter(['main', first, second])
    const other = electWriter([second, first, 'main'])
    expect(other).toBe(one)
  })
})

const holding = (name: RouteName): StoredWindow => ({
  tabs: [{ entries: [{ location: { name } }], index: 0 }],
  activeIndex: 0,
})

const ledgerOf = (
  entries: readonly [string, StoredWindow][],
): Map<string, StoredWindow> => new Map(entries)

describe('what a window does when something changed', () => {
  it('writes every window it knows of, main first', () => {
    const ledger = ledgerOf([
      [second, holding(RouteName.Wiki)],
      ['main', holding(RouteName.Goals)],
    ])
    const decision = decideSessionWrite('main', ['main', second], ledger, true)
    expect(decision.kind).toBe(SessionAction.Write)
    if (decision.kind !== SessionAction.Write) return
    expect(decision.windows).toHaveLength(2)
    expect(decision.windows[0]?.tabs[0]?.entries[0]?.location.name).toBe(
      RouteName.Goals,
    )
  })

  it('does nothing in a window that is not the elected one', () => {
    const ledger = ledgerOf([
      ['main', holding(RouteName.Goals)],
      [second, holding(RouteName.Wiki)],
    ])
    expect(decideSessionWrite(second, ['main', second], ledger, true)).toEqual({
      kind: SessionAction.Nothing,
    })
  })

  // The whole of 3.7b in one assertion: main is gone, and the session is still written down —
  // by the survivor, which under the old rule wrote nothing ever again.
  it('keeps writing once main has been closed', () => {
    const ledger = ledgerOf([[second, holding(RouteName.Wiki)]])
    const decision = decideSessionWrite(second, [second], ledger, true)
    expect(decision.kind).toBe(SessionAction.Write)
    if (decision.kind !== SessionAction.Write) return
    expect(decision.windows).toHaveLength(1)
  })

  it('waits for a window that has not said what it holds yet', () => {
    const ledger = ledgerOf([['main', holding(RouteName.Goals)]])
    expect(decideSessionWrite('main', ['main', second], ledger, true)).toEqual({
      kind: SessionAction.Postpone,
    })
  })

  // The roster keeps naming a webview for a while after it closes, so patience has to run out:
  // waiting for ever is this sub-project's own bug wearing a different coat.
  it('goes ahead without it when it has waited long enough', () => {
    const ledger = ledgerOf([['main', holding(RouteName.Goals)]])
    const decision = decideSessionWrite('main', ['main', second], ledger, false)
    expect(decision.kind).toBe(SessionAction.Write)
    if (decision.kind !== SessionAction.Write) return
    expect(decision.windows).toHaveLength(1)
  })

  it('leaves out a window holding nothing, mid-tear-off', () => {
    const ledger = ledgerOf([
      ['main', holding(RouteName.Goals)],
      [second, { tabs: [], activeIndex: 0 }],
    ])
    const decision = decideSessionWrite('main', ['main', second], ledger, true)
    expect(decision.kind).toBe(SessionAction.Write)
    if (decision.kind !== SessionAction.Write) return
    expect(decision.windows).toHaveLength(1)
  })

  it('writes nothing at all when no window holds anything', () => {
    const ledger = ledgerOf([['main', { tabs: [], activeIndex: 0 }]])
    expect(decideSessionWrite('main', ['main'], ledger, true)).toEqual({
      kind: SessionAction.Nothing,
    })
  })
})
