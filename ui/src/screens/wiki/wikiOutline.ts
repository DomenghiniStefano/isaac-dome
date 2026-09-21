import type { Section, SectionKind } from '@/lib/ipc/types'

// The index of a page's sections, for the column that stays beside the text (card #57).
// It exists so a long page can be moved through without scrolling it blind, which means a
// page that is already one glance long gets none: an index of a single entry is furniture,
// and it would appear on the many pages that carry one section and nothing else.
export const OutlineFloor = 2

export interface OutlineItem {
  kind: SectionKind
  /** The id its section's heading carries, and what the link scrolls to. */
  id: string
}

// The anchor is the section's **position**, never its kind: a page with two sections of one
// kind would otherwise give them one id, and every link would land on the first of the two.
export const sectionAnchor = (index: number): string => `wiki-section-${index}`

export const outlineOf = (sections: Section[]): OutlineItem[] =>
  sections.length < OutlineFloor
    ? []
    : sections.map((section, index) => ({
        kind: section.kind,
        id: sectionAnchor(index),
      }))
