import { assertNever } from '@/lib/assertNever'
import type { Block, Inline, ListItem, Target } from '@/lib/ipc/types'

// What a name's icon is: an achievement's painted drawing, dark strokes on transparency that
// need the mark paper under them, or the game's pixel art, drawn at a whole multiple.
export const RefArt = {
  Drawing: 'drawing',
  Sprite: 'sprite',
} as const
export type RefArt = (typeof RefArt)[keyof typeof RefArt]

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

// The reference an item of a list of names is — what `{{achievement text}}` and the
// collectible tables become — or null for an item of prose. What sits under the item (an
// achievement's unlock condition) does not change what the item is.
export const nameOf = (
  item: ListItem,
): Extract<Inline, { kind: 'ref' }> | null => {
  const node = soleNode(item.inline)
  return node?.kind === 'ref' ? node : null
}

export const isNameItem = (item: ListItem): boolean => nameOf(item) !== null

// A list drawn as names beside their pictures instead of as bullets: every item is a name.
// One item of prose among them and the list is prose, drawn as the wiki's bullets — a row of
// pictures with a sentence in the middle reads as a mistake.
export const isNameList = (block: Block): boolean =>
  block.kind === 'list' &&
  !block.ordered &&
  block.items.length > 0 &&
  block.items.every(isNameItem)
