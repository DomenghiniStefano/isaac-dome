import { describe, expect, it } from 'vitest'
import type { Inline } from '@/lib/ipc/types'
import { inlinePlain } from './inlinePlain'

const text = (t: string): Inline => ({ kind: 'text', text: t, style: 'plain' })

describe('inlinePlain', () => {
  it('joins text nodes with nothing between them, the same as the tree lays them out', () => {
    expect(inlinePlain([text('Bag of '), text('Crafting')])).toBe(
      'Bag of Crafting',
    )
  })

  it('takes a ref by its label, not its target', () => {
    const ref: Inline = {
      kind: 'ref',
      target: { kind: 'item', id: 5 },
      label: 'Bomb',
    }
    expect(inlinePlain([text('Interactions with '), ref])).toBe(
      'Interactions with Bomb',
    )
  })

  it('takes a concept by its label', () => {
    const concept: Inline = { kind: 'concept', page: 'Damage', label: 'damage' }
    expect(inlinePlain([concept])).toBe('damage')
  })

  it('reads through an edition wrapper to its own words', () => {
    const edition: Inline = {
      kind: 'edition',
      only: ['repentance'],
      inline: [text('Mausoleum/Gehenna')],
    }
    expect(inlinePlain([text('Behavior in '), edition])).toBe(
      'Behavior in Mausoleum/Gehenna',
    )
  })

  it('is empty on an empty heading, the way an anchor-only title renders', () => {
    expect(inlinePlain([])).toBe('')
  })
})
