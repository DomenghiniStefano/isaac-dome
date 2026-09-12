import { describe, expect, it } from 'vitest'
import { wikiEntryAnswer, wikiIndexAnswer } from './wiki'

describe('the wiki fixture', () => {
  const index = wikiIndexAnswer({ withArt: false, withWiki: true })
  const find = (kind: string, id: number) =>
    index.pages.find(
      (p) => p.target.kind === kind && 'id' in p.target && p.target.id === id,
    )

  it("lists the pack's pages with their titles, by kind", () => {
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

  it('has no icons without art, and a link per illustrated page with it', () => {
    expect(index.pages.every((p) => p.iconUrl === null)).toBe(true)
    const illustrated = wikiIndexAnswer({ withArt: true, withWiki: true })
    expect(
      illustrated.pages.find(
        (p) => p.target.kind === 'item' && p.target.id === 105,
      )?.iconUrl,
    ).not.toBeNull()
    expect(
      illustrated.pages.find((p) => p.target.kind === 'challenge')?.iconUrl,
    ).toBeNull()
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
    const none = wikiIndexAnswer({ withArt: true, withWiki: false })
    expect(none.info.kind).toBe('missing')
    expect(none.pages).toHaveLength(0)
  })
})
