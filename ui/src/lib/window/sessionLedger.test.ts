import { describe, expect, it } from 'vitest'
import { RouteName } from '@/router/routeTable'
import type { StoredWindow } from './sessionDocument'
import { createSessionLedger } from './sessionLedger'

const holding = (name: RouteName): StoredWindow => ({
  tabs: [{ entries: [{ location: { name } }], index: 0 }],
  activeIndex: 0,
})

describe('the session ledger', () => {
  it('keeps what each window last said it holds', () => {
    const ledger = createSessionLedger()
    ledger.hold('main', holding(RouteName.Floor))
    ledger.hold('main', holding(RouteName.Unlock))
    ledger.hold('win-a', holding(RouteName.Goals))
    expect([...ledger.held.keys()]).toEqual(['main', 'win-a'])
    expect(ledger.held.get('main')).toEqual(holding(RouteName.Unlock))
  })

  it('answers the roster as it is, and forgets a window the roster no longer names', () => {
    const ledger = createSessionLedger()
    ledger.hold('main', holding(RouteName.Floor))
    ledger.hold('win-a', holding(RouteName.Goals))
    expect(ledger.alive(['main', 'win-b'])).toEqual(['main', 'win-b'])
    expect([...ledger.held.keys()]).toEqual(['main'])
  })

  // The roster keeps naming a webview for a while after it closes.
  it('a window that said it was going is out, even while the roster still names it', () => {
    const ledger = createSessionLedger()
    ledger.hold('main', holding(RouteName.Floor))
    ledger.hold('win-a', holding(RouteName.Goals))
    ledger.leave('win-a')
    expect([...ledger.held.keys()]).toEqual(['main'])
    expect(ledger.alive(['main', 'win-a'])).toEqual(['main'])
  })
})
