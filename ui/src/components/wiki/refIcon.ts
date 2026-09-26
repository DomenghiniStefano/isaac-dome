import { assertNever } from '@/lib/assertNever'
import type { Inline, ListItem, Target } from '@/lib/ipc/types'

// What a reference's icon is: an achievement's painted drawing, dark strokes on transparency
// that need the mark paper under them, or the game's pixel art, drawn at a whole multiple.
export const RefArt = {
  Drawing: 'drawing',
  Sprite: 'sprite',
} as const
export type RefArt = (typeof RefArt)[keyof typeof RefArt]

// How large the icon is: beside a sentence's words, or leading an item of a list of names.
export const RefIconSize = {
  Inline: 'inline',
  Name: 'name',
} as const
export type RefIconSize = (typeof RefIconSize)[keyof typeof RefIconSize]

export const refArt = (target: Target): RefArt => {
  switch (target.kind) {
    case 'achievement':
      return RefArt.Drawing
    case 'item':
    case 'trinket':
    case 'character':
    case 'challenge':
    case 'entity':
    case 'transformation':
    case 'stage':
    case 'room':
    case 'concept':
      return RefArt.Sprite
    default:
      return assertNever(target)
  }
}

// The one node an item holds, seen through the edition a `dlc =` list wraps it in.
const soleNode = (inline: Inline[]): Inline | null => {
  const [only, ...rest] = inline
  if (only === undefined || rest.length > 0) return null
  return only.kind === 'edition' ? soleNode(only.inline) : only
}

// An item of a list of names — what `{{achievement text}}` and the collectible tables become
// — is one reference and nothing else. An item whose reference opens a sentence is prose, and
// a large icon there would push every line of an Effects list apart.
export const isNameItem = (item: ListItem): boolean =>
  soleNode(item.inline)?.kind === 'ref'
