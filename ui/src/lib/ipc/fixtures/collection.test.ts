import { describe, expect, it } from 'vitest'
import type { CollectionItem } from '../types'
import {
  CollectionSource,
  collectionAnswer,
  collectionSource,
} from './collection'

interface IndexEntry {
  family: string
  id: number
  kind?: string
  name?: string
}
const indexes = import.meta.glob<IndexEntry[]>(
  '../../../../../design-export/isaacdome-design-pack/images/INDEX.json',
  { eager: true, import: 'default' },
)
const packCollectibles = (Object.values(indexes)[0] ?? []).filter(
  (e) => e.family === 'item' && e.kind !== 'trinket',
)
const byId = (items: CollectionItem[], id: number) =>
  items.find((i) => i.id === id)

describe('the Collection fixture', () => {
  const view = collectionAnswer({
    withArt: false,
    withCatalog: true,
    collectionRead: true,
  })

  it('declares its source: synthetic until the pack carries collection.json', () => {
    expect(collectionSource()).toBe(CollectionSource.Synthetic)
  })

  it("lists the pack's collectibles by id, with their names and kinds", () => {
    expect(view.items).toHaveLength(packCollectibles.length)
    expect(view.items.every((i) => i.kind !== 'trinket')).toBe(true)
    expect(byId(view.items, 1)).toMatchObject({
      name: 'The Sad Onion',
      kind: 'passive',
    })
    expect(view.items.map((i) => i.id)).toEqual(
      [...view.items.map((i) => i.id)].sort((a, b) => a - b),
    )
  })

  it('takes its locks from the unlock payload', () => {
    // contracts/payload/unlock.json: achievement 6 (done) unlocks Cube of Meat, familiar 73;
    // achievement 62 (not done) unlocks Epic Fetus, passive 168.
    expect(byId(view.items, 73)?.lock).toMatchObject({
      kind: 'unlocked',
      achievement: 6,
    })
    expect(byId(view.items, 168)?.lock).toMatchObject({
      kind: 'locked',
      achievement: 62,
    })
    expect(byId(view.items, 1)?.lock).toEqual({ kind: 'free' })
  })

  it('never puts a locked item in the collection', () => {
    expect(
      view.items
        .filter((i) => i.lock.kind === 'locked')
        .every((i) => i.inCollection === false),
    ).toBe(true)
  })

  it('takes the origin from the catalog id ranges', () => {
    expect(byId(view.items, 341)?.origin).toBe('rebirth')
    expect(byId(view.items, 342)?.origin).toBe('afterbirth')
    expect(byId(view.items, 553)?.origin).toBe('repentance')
  })

  it('answers an unread collection as null, and says so', () => {
    const unread = collectionAnswer({
      withArt: false,
      withCatalog: true,
      collectionRead: false,
    })
    expect(unread.items.every((i) => i.inCollection === null)).toBe(true)
    expect(unread.diagnostics).toEqual([{ kind: 'noCollectionSection' }])
  })

  it('answers without a catalog with totals only', () => {
    const none = collectionAnswer({
      withArt: false,
      withCatalog: false,
      collectionRead: true,
    })
    expect(none.items).toEqual([])
    expect(none.diagnostics).toEqual([{ kind: 'noCatalog' }])
  })
})
