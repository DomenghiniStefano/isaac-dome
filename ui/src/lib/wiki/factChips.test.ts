import { describe, expect, it } from 'vitest'
import type { PageFacts, WikiPageRef } from '@/lib/ipc/types'
import { WikiCategory } from '@/router/routeTable'
import { Tone } from './tone'
import { factChips, factColumns } from './factChips'

describe('factChips', () => {
  it('produces no chip at all for an empty item', () => {
    const facts: PageFacts = {
      kind: 'item',
      quality: null,
      activated: false,
      recharge: null,
      shopPrice: null,
      devilPrice: null,
      tags: [],
    }
    // `activated`/`passive` is a defining trait, always known, so it is the one chip left.
    expect(factChips(facts).map((c) => c.key)).toEqual(['template'])
  })

  it('colours the quality chip by the catalog tier, and skips it when unrated', () => {
    const rated: PageFacts = {
      kind: 'item',
      quality: 3,
      activated: true,
      recharge: null,
      shopPrice: null,
      devilPrice: null,
      tags: [],
    }
    const chips = factChips(rated)
    const quality = chips.find((c) => c.key === 'quality')
    expect(quality?.tone).toBe(Tone.QualityGold)
    expect(quality?.value).toBe(3)
    expect(chips.find((c) => c.key === 'template')?.label).toEqual({
      key: 'wiki.facts.activated',
    })
  })

  it('gives one chip per item tag, and none when there are no tags', () => {
    const facts: PageFacts = {
      kind: 'item',
      quality: null,
      activated: false,
      recharge: null,
      shopPrice: null,
      devilPrice: null,
      tags: ['guppy', 'syringe'],
    }
    const tags = factChips(facts).filter((c) => c.key.startsWith('tag:'))
    expect(tags.map((c) => c.value)).toEqual(['guppy', 'syringe'])
  })

  it('skips an achievement requirement chip when the text is empty', () => {
    expect(
      factChips({ kind: 'achievement', requirement: '', unlocks: null }),
    ).toEqual([])
    const chips = factChips({
      kind: 'achievement',
      requirement: 'Beat Mom',
      unlocks: null,
    })
    expect(chips).toEqual([
      {
        key: 'requirement',
        label: { key: 'wiki.facts.requirement', params: { value: 'Beat Mom' } },
        value: 'Beat Mom',
        tone: Tone.QualityNone,
      },
    ])
  })

  it('resolves a challenge character by id through the given titleOf, not by name', () => {
    const facts: PageFacts = {
      kind: 'challenge',
      character: { kind: 'character', id: 21 },
      goal: '',
      blindfolded: false,
      curse: '',
    }
    const titleOf = (key: string): string | null =>
      key === 'character:21' ? 'Tainted Isaac' : 'wrong'
    const chip = factChips(facts, titleOf).find((c) => c.key === 'character')
    expect(chip?.value).toBe('Tainted Isaac')
  })

  it('shows blindfolded only when true, the same reading the infobox restrictions give it', () => {
    const base: PageFacts = {
      kind: 'challenge',
      character: null,
      goal: '',
      blindfolded: false,
      curse: '',
    }
    expect(factChips(base).some((c) => c.key === 'blindfolded')).toBe(false)
    expect(
      factChips({ ...base, blindfolded: true }).some(
        (c) => c.key === 'blindfolded',
      ),
    ).toBe(true)
  })

  it('gives a character every non-empty stat chip and skips the empty ones', () => {
    const facts: PageFacts = {
      kind: 'character',
      health: '3 red hearts',
      damage: '',
      tears: '',
      range: '',
      speed: '',
      luck: '',
      shotSpeed: '',
      tainted: false,
    }
    expect(factChips(facts).map((c) => c.key)).toEqual(['health'])
  })

  it('shows a tainted chip only for a tainted character', () => {
    const facts: PageFacts = {
      kind: 'character',
      health: '',
      damage: '',
      tears: '',
      range: '',
      speed: '',
      luck: '',
      shotSpeed: '',
      tainted: true,
    }
    expect(factChips(facts).map((c) => c.key)).toEqual(['tainted'])
  })

  it('gives a boss its base HP and floors, and an entity the same shape', () => {
    const boss: PageFacts = { kind: 'boss', baseHp: 300, floors: 'Womb' }
    expect(factChips(boss).map((c) => c.key)).toEqual(['baseHp', 'floors'])
    const entity: PageFacts = { kind: 'entity', baseHp: null, floors: '' }
    expect(factChips(entity)).toEqual([])
  })

  it('reads an article category and, only for a version, its number and date', () => {
    const card: PageFacts = { kind: 'article', category: 'card', version: null }
    expect(factChips(card).map((c) => c.key)).toEqual(['category'])
    const version: PageFacts = {
      kind: 'article',
      category: 'version',
      version: { number: '1.7.9', date: '2021-04-27' },
    }
    expect(factChips(version).map((c) => c.key)).toEqual([
      'category',
      'versionNumber',
      'versionDate',
    ])
  })
})

describe('factColumns', () => {
  it('gives items their own scalar columns, and reads them from a page of another kind as null', () => {
    const columns = factColumns(WikiCategory.Items)
    const qualityColumn = columns.find((c) => c.key === 'quality')
    expect(qualityColumn).toBeDefined()
    const itemPage: WikiPageRef = {
      target: { kind: 'item', id: 1 },
      title: 'Sad Onion',
      iconUrl: null,
      category: WikiCategory.Items,
      dlc: [],
      facts: {
        kind: 'item',
        quality: 2,
        activated: false,
        recharge: null,
        shopPrice: null,
        devilPrice: null,
        tags: [],
      },
    }
    const otherPage: WikiPageRef = {
      ...itemPage,
      facts: { kind: 'achievement', requirement: '', unlocks: null },
    }
    expect(qualityColumn?.value(itemPage)).toBe(2)
    expect(qualityColumn?.value(otherPage)).toBeNull()
  })

  it('gives trinkets no scalar column, since a tag list is not one', () => {
    expect(factColumns(WikiCategory.Trinkets)).toEqual([])
  })
})
