import type { Infobox } from '@/lib/ipc/types'

type Transformation = Extract<Infobox, { kind: 'transformation' }>

/**
 * Whether a transformation's infobox has anything to put in a card.
 *
 * The three fields are independent and each one is missing on some page: `requires` is
 * `null` wherever the page states no count in a form the parser reads, `target` is empty on
 * fourteen of the sixteen, and Adult has none of the three. The row is drawn only where the
 * page filled it, because this box draws "nessuno" for an empty value — true of a character
 * with no starting items, a claim nobody measured about a transformation.
 */
export const hasRows = (infobox: Transformation): boolean =>
  infobox.requires !== null ||
  infobox.contributors.length > 0 ||
  infobox.target.length > 0
