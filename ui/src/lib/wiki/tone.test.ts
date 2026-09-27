import { describe, expect, it } from 'vitest'
import { Dlc } from '@/lib/ipc/types'
import { WikiCategory } from '@/router/routeTable'
import { Tone, toneOfCategory, toneOfEdition, toneOfQuality } from './tone'

const allTones = new Set(Object.values(Tone))

describe('toneOfEdition', () => {
  it('has a tone for every Dlc', () => {
    for (const dlc of Object.values(Dlc)) {
      expect(allTones.has(toneOfEdition(dlc))).toBe(true)
    }
  })

  it('gives each edition its own tone', () => {
    const tones = Object.values(Dlc).map(toneOfEdition)
    expect(new Set(tones).size).toBe(tones.length)
  })
})

describe('toneOfCategory', () => {
  it('has a tone for every WikiCategory', () => {
    for (const category of Object.values(WikiCategory)) {
      expect(allTones.has(toneOfCategory(category))).toBe(true)
    }
  })

  it('gives each category its own tone', () => {
    const tones = Object.values(WikiCategory).map(toneOfCategory)
    expect(new Set(tones).size).toBe(tones.length)
  })
})

describe('toneOfQuality', () => {
  it.each([-1, 0, 1, 2, 3, 4])('has a tone for quality %i', (quality) => {
    expect(allTones.has(toneOfQuality(quality))).toBe(true)
  })

  it('gives each rated tier its own tone', () => {
    const tones = [-1, 0, 1, 2, 3, 4].map(toneOfQuality)
    expect(new Set(tones).size).toBe(tones.length)
  })

  it('falls back to QualityNone outside the catalog range, the same reading -1 gets', () => {
    expect(toneOfQuality(99)).toBe(Tone.QualityNone)
  })
})
