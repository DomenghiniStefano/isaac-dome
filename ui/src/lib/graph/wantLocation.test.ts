import { describe, expect, it } from 'vitest'
import { RouteName } from '@/router/routeTable'
import type { Target } from '@/lib/ipc/types'
import { wantLocation, wantOf, wantable } from './wantLocation'

describe('wantLocation', () => {
  it('names the Goals screen and carries the page key', () => {
    expect(wantLocation({ kind: 'character', id: 41 })).toEqual({
      name: RouteName.Goals,
      query: { want: 'character:41' },
    })
  })

  // The four kinds with no page key are the four that are not things you unlock: a link to
  // one would be a question the app cannot ask.
  it('has no location for a thing you do not unlock', () => {
    expect(wantLocation({ kind: 'stage', name: 'Basement' })).toBeNull()
  })

  it('reads the want back out of a location', () => {
    expect(wantOf({ want: 'item:105' })).toEqual({ kind: 'item', id: 105 })
  })

  // A hand-typed or stale URL is not an error: the screen shows the recommendations.
  it('reads nothing from a key it never wrote', () => {
    expect(wantOf({ want: 'mode:greed' })).toBeNull()
    expect(wantOf({})).toBeNull()
    expect(wantOf(undefined)).toBeNull()
  })
})

describe('wantable', () => {
  const hit = (target: Target) => ({ target })

  it('keeps what something can grant and drops what nothing does', () => {
    const hits = [
      hit({ kind: 'item', id: 105 }),
      hit({ kind: 'achievement', id: 509 }),
      hit({ kind: 'stage', name: 'Basement' }),
      hit({ kind: 'transformation', id: 1 }),
    ]
    expect(wantable(hits).map((h) => h.target.kind)).toEqual([
      'item',
      'achievement',
    ])
  })
})
