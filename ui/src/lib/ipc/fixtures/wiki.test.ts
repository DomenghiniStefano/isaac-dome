import { describe, expect, it } from 'vitest'
import {
  extractionReportAnswer,
  samplePages,
  wikiConditions,
  wikiEntryAnswer,
  wikiIndexAnswer,
} from './wiki'

describe('the wiki fixture', () => {
  const index = wikiIndexAnswer({ withWiki: true })
  const find = (kind: string, id: number) =>
    index.pages.find(
      (p) => p.target.kind === kind && 'id' in p.target && p.target.id === id,
    )

  it('lists the pages with their titles, by kind', () => {
    expect(find('item', 105)).toMatchObject({ title: 'The D6' })
    expect(find('trinket', 1)).toMatchObject({ title: 'Swallowed Penny' })
    expect(find('achievement', 1)).toMatchObject({ title: 'Magdalene' })
    expect(find('entity', 20)).toMatchObject({
      target: { kind: 'entity', id: 20, variant: 0, subtype: 0 },
      title: 'Monstro',
    })
    expect(find('character', 0)).toMatchObject({ title: 'Isaac' })
    expect(index.pages.some((p) => p.target.kind === 'challenge')).toBe(true)
    const kinds = index.pages.map((p) => p.target.kind)
    expect(kinds.indexOf('trinket')).toBeGreaterThan(kinds.lastIndexOf('item'))
    expect(kinds.indexOf('entity')).toBeGreaterThan(
      kinds.lastIndexOf('achievement'),
    )
  })

  it('counts what it lists', () => {
    expect(index.info.kind).toBe('loaded')
    if (index.info.kind !== 'loaded') return
    expect(index.info.counts.items).toBe(
      index.pages.filter((p) => p.target.kind === 'item').length,
    )
    expect(index.info.counts.bosses).toBeGreaterThan(70)
  })

  // It used to check the other half too: with art, item 105 had a link and a challenge did
  // not. Nothing here carries a drawing any more — the app cuts its sprites from the user's
  // own copy of the game at runtime, and the development server has no copy to cut from.
  it('has no icons at all', () => {
    expect(index.pages.every((p) => p.iconUrl === null)).toBe(true)
  })

  it('answers the eleven sample pages and no other', () => {
    expect(wikiEntryAnswer({ kind: 'item', id: 105 })?.title).toBe('The D6')
    expect(
      wikiEntryAnswer({ kind: 'entity', id: 20, variant: 0, subtype: 0 })
        ?.infobox.kind,
    ).toBe('boss')
    expect(wikiEntryAnswer({ kind: 'item', id: 1 })).toBeNull()
    expect(wikiEntryAnswer({ kind: 'stage', name: 'Basement' })).toBeNull()
  })

  it('is a missing dataset on request', () => {
    const none = wikiIndexAnswer({ withWiki: false })
    expect(none.info.kind).toBe('missing')
    expect(none.pages).toHaveLength(0)
  })
})

describe('wikiConditions', () => {
  it('names the game file condition for the achievements it recorded one for', () => {
    const conditions = wikiConditions()
    expect(conditions.size).toBe(283)
    expect(conditions.get(1)).toBe('have 7 or more max red hearts at one time')
  })
})

// The kinds `Target` (ui/src/lib/ipc/types.ts) knows. A ref target outside this set is what
// `assertNever` in `lib/wiki/pageKey.ts` throws on, which blanks the page it is read from.
const targetKinds = new Set([
  'item',
  'trinket',
  'character',
  'achievement',
  'challenge',
  'entity',
  'transformation',
  'stage',
  'room',
  'concept',
])

// A `ref` inline's `target` is the only shape carrying a `Target`; walking every object that
// has one, anywhere in the recorded page, needs no knowledge of where in the tree it sits.
const targetKindsIn = (value: unknown, found: string[]): void => {
  if (Array.isArray(value)) {
    value.forEach((v) => targetKindsIn(v, found))
    return
  }
  if (value === null || typeof value !== 'object') return
  const obj = value as Record<string, unknown>
  const target = obj.target
  if (
    target !== null &&
    typeof target === 'object' &&
    typeof (target as Record<string, unknown>).kind === 'string'
  ) {
    found.push((target as Record<string, unknown>).kind as string)
  }
  Object.values(obj).forEach((v) => targetKindsIn(v, found))
}

describe('every recorded sample page', () => {
  it('names a ref target the Target union still knows', () => {
    const kinds: string[] = []
    for (const entry of samplePages.values()) targetKindsIn(entry, kinds)
    expect(kinds.length).toBeGreaterThan(0)
    expect(kinds.filter((k) => !targetKinds.has(k))).toEqual([])
  })
})

describe('the recorded extraction report', () => {
  it('lists no broken archive, a field it was recorded before', () => {
    // Card #80, R6: the report carries the archives that did not open. The machine that
    // recorded it had none, and an absent key would make the verification page throw.
    expect(extractionReportAnswer()?.broken).toEqual([])
  })
})
