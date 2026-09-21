import { describe, expect, it } from 'vitest'
import { SectionKind } from '@/lib/ipc/types'
import type { Section } from '@/lib/ipc/types'
import { sectionAnchor, outlineOf } from './wikiOutline'

const section = (kind: SectionKind): Section => ({ kind, blocks: [] })

// The column beside a page's text exists to let a long page be navigated. What it must not
// do is appear on a page that is one glance long: an index of one entry is furniture.
describe('the page outline', () => {
  it('is empty on a page with a single section', () => {
    expect(outlineOf([section(SectionKind.Effects)])).toEqual([])
  })

  it('is empty on a page with no sections at all', () => {
    expect(outlineOf([])).toEqual([])
  })

  it('lists every section in the order the dataset gives them', () => {
    const outline = outlineOf([
      section(SectionKind.Effects),
      section(SectionKind.Synergies),
      section(SectionKind.Notes),
    ])
    expect(outline.map((item) => item.kind)).toEqual([
      SectionKind.Effects,
      SectionKind.Synergies,
      SectionKind.Notes,
    ])
  })

  it('anchors each entry on its own section, by position', () => {
    const outline = outlineOf([
      section(SectionKind.Effects),
      section(SectionKind.Notes),
    ])
    expect(outline.map((item) => item.id)).toEqual([
      sectionAnchor(0),
      sectionAnchor(1),
    ])
  })

  // Two sections of the same kind would otherwise share an id, and the link would always
  // land on the first of them: the anchor is the position, never the kind.
  it('gives two sections of the same kind two different anchors', () => {
    const outline = outlineOf([
      section(SectionKind.Notes),
      section(SectionKind.Notes),
    ])
    expect(outline[0]?.id).not.toBe(outline[1]?.id)
  })
})
