import { describe, expect, it } from 'vitest'
import type { EntityRef } from '@/lib/ipc/types'
import { entityName, whoKilled } from './death'

const entity = (raw: string, name: string | null): EntityRef => ({
  raw,
  name,
  iconUrl: null,
  page: null,
})

describe('whoKilled', () => {
  // `Killed by (9.0) spawned by (20.0)`: a shot fired by Monstro. A row has room for one name,
  // and "killed by Monstro" is the answer a player means — the shot is the page's detail.
  it('names the spawner when there is one', () => {
    expect(
      whoKilled({
        killer: entity('9.0', 'Projectile'),
        spawner: entity('20.0', 'Monstro'),
      }),
    ).toEqual(entity('20.0', 'Monstro'))
  })

  it('names the killer when nobody spawned it', () => {
    expect(
      whoKilled({ killer: entity('20.0', 'Monstro'), spawner: null }),
    ).toEqual(entity('20.0', 'Monstro'))
  })
})

describe('entityName', () => {
  // A catalog that does not know the row — the game not installed — leaves the log's own words.
  it('is the name, or what the log wrote', () => {
    expect(entityName(entity('20.0', 'Monstro'))).toBe('Monstro')
    expect(entityName(entity('20.0', null))).toBe('20.0')
  })
})
