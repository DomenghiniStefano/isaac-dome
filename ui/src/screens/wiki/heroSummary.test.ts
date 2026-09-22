import { describe, expect, it } from 'vitest'
import type { Entry, Infobox, Inline } from '@/lib/ipc/types'
import { summaryOf } from './heroSummary'

type Achievement = Extract<Infobox, { kind: 'achievement' }>

const text = (s: string): Inline => ({ kind: 'text', text: s, style: 'plain' })

const achievement = (
  fields: Partial<Omit<Achievement, 'kind'>>,
): Achievement => ({
  kind: 'achievement',
  quote: [],
  requirements: [],
  notes: [],
  unlocks: null,
  ...fields,
})

const entry = (infobox: Infobox, description: Inline[] = []): Entry => ({
  title: 'x',
  revid: 1,
  description,
  dlc: [],
  unlockedBy: null,
  infobox,
  sections: [],
})

const titleOf = (key: string): string | null =>
  key === 'character:21' ? 'Tainted Isaac' : null

describe('summaryOf', () => {
  // The wiki leaves every achievement's description empty — what it filed there is the
  // unlock paper's line, now the quote — so the line under the title is composed from what
  // the achievement gives, with the verb in the reader's language.
  it('says what an achievement unlocks', () => {
    const e = entry(
      achievement({
        unlocks: { kind: 'character', id: 21 },
        requirements: [text('Open the closet as Isaac')],
      }),
    )
    expect(summaryOf(e, 'Unlocks', titleOf)).toEqual([
      text('Unlocks '),
      {
        kind: 'ref',
        target: { kind: 'character', id: 21 },
        label: 'Tainted Isaac',
      },
    ])
  })

  // 1000000% and Dead God unlock no page of their own: what they ask for is the line.
  it('falls back to the requirements when it unlocks nothing the wiki names', () => {
    const requirements = [text('Collect every item in the game')]
    expect(
      summaryOf(entry(achievement({ requirements })), 'Unlocks', titleOf),
    ).toEqual(requirements)
  })

  // A description written by hand in `corrections.json` wins on any kind, achievements
  // included: it is the reason the file has the section.
  it('keeps a description the entry already has', () => {
    const description = [text('You are the Dead God.')]
    expect(
      summaryOf(
        entry(achievement({ requirements: [text('r')] }), description),
        'Unlocks',
        titleOf,
      ),
    ).toEqual(description)
  })

  it('composes nothing for another kind', () => {
    const trinket: Infobox = { kind: 'trinket', quote: [], tags: [], pools: [] }
    expect(summaryOf(entry(trinket), 'Unlocks', titleOf)).toEqual([])
  })
})
