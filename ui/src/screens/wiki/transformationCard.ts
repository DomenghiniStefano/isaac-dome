import type { Entry, Infobox } from '@/lib/ipc/types'

type Transformation = Extract<Infobox, { kind: 'transformation' }>

/**
 * Whether a transformation's infobox has anything of its own to put in the card.
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

/**
 * Whether the card is drawn at all.
 *
 * Not the same question, and reading it as the same one was a defect of a few hours:
 * `hasRows` suppressed the **whole** card for Adult, and the card also carries the two facts
 * every kind declares — the description and what unlocks it. Adult had a description and it
 * went dark with the rows it does not have. The card exists when anything in it does.
 */
export const hasCard = (infobox: Transformation, entry: Entry): boolean =>
  hasRows(infobox) || entry.description.length > 0 || entry.unlockedBy !== null
