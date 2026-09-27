import { describe, expect, it } from 'vitest'
import type { CategorySample } from '@/lib/ipc/types'
import { WikiCategory } from '@/router/routeTable'
import { mosaicSamples } from './landing'

const sample = (
  category: CategorySample['category'],
  iconUrl: string | null,
): CategorySample => ({ category, iconUrl, target: null })

describe('mosaicSamples', () => {
  it('keeps at most max samples', () => {
    const samples = Object.values(WikiCategory).map((c) => sample(c, null))
    expect(mosaicSamples(samples, 4)).toHaveLength(4)
  })

  it('never invents a sample beyond what it was given', () => {
    const samples = [sample(WikiCategory.Items, 'x')]
    expect(mosaicSamples(samples, 8)).toHaveLength(1)
  })

  // The owner asked for pictures, not icons ("le persone piacciono colori ed immagini"): a
  // category with no picture at all (transformations, stages, versions always answer
  // `null`, `category_sample` in `wiki_samples.rs`) is trimmed first, so the mosaic leads
  // with what it actually has a drawing for.
  it('prefers a real picture over one that would only draw the category icon', () => {
    const samples = [
      sample(WikiCategory.Stages, null),
      sample(WikiCategory.Items, 'items.png'),
      sample(WikiCategory.Versions, null),
      sample(WikiCategory.Characters, 'characters.png'),
    ]
    expect(mosaicSamples(samples, 2).map((s) => s.category)).toEqual([
      WikiCategory.Items,
      WikiCategory.Characters,
    ])
  })

  it('keeps the original order within the same picture-or-not group', () => {
    const samples = [
      sample(WikiCategory.Bosses, null),
      sample(WikiCategory.Monsters, null),
    ]
    expect(mosaicSamples(samples, 2).map((s) => s.category)).toEqual([
      WikiCategory.Bosses,
      WikiCategory.Monsters,
    ])
  })
})
