import { assertNever } from '@/lib/assertNever'
import type { Target } from '@/lib/ipc/types'

// A page's identity as one string, the way the design pack names its sample files and the
// dataset keys its bosses: `item:105`, `entity:20.0.0`. It's what a tab location carries
// (spec 3.5, Decision 1) — the identity, never the page.
const PageKind = {
  Item: 'item',
  Trinket: 'trinket',
  Achievement: 'achievement',
  Challenge: 'challenge',
  Character: 'character',
  Entity: 'entity',
} as const
type PageKind = (typeof PageKind)[keyof typeof PageKind]

const Separator = ':'
const EntitySeparator = '.'

// The four kinds the dataset has no page for get no key: `Dataset::entry` answers nothing
// for them by construction, so a key would name a page that can't exist.
export const pageKey = (target: Target): string | null => {
  switch (target.kind) {
    case 'item':
      return `${PageKind.Item}${Separator}${target.id}`
    case 'trinket':
      return `${PageKind.Trinket}${Separator}${target.id}`
    case 'achievement':
      return `${PageKind.Achievement}${Separator}${target.id}`
    case 'challenge':
      return `${PageKind.Challenge}${Separator}${target.number}`
    case 'character':
      return `${PageKind.Character}${Separator}${target.id}`
    case 'entity':
      return `${PageKind.Entity}${Separator}${[target.id, target.variant, target.subtype].join(EntitySeparator)}`
    case 'stage':
    case 'room':
    case 'pickup':
    case 'transformation':
      return null
    default:
      return assertNever(target)
  }
}

// Decimal digits only: no sign, no fraction, no whitespace. `Number` alone would accept all
// three, and a key we never wrote must read as a page the dataset doesn't know.
const Digits = /^\d+$/
const number = (s: string | undefined): number | null =>
  s !== undefined && Digits.test(s) ? Number(s) : null

const isPageKind = (s: string): s is PageKind =>
  Object.values(PageKind).some((kind) => kind === s)

export const parsePageKey = (key: string): Target | null => {
  const at = key.indexOf(Separator)
  if (at < 0) return null
  const kind = key.slice(0, at)
  const rest = key.slice(at + 1)
  if (!isPageKind(kind)) return null
  if (kind === PageKind.Entity) {
    const parts = rest.split(EntitySeparator)
    if (parts.length !== 3) return null
    const [id, variant, subtype] = parts.map(number)
    return id === null ||
      variant === null ||
      subtype === null ||
      id === undefined ||
      variant === undefined ||
      subtype === undefined
      ? null
      : { kind: 'entity', id, variant, subtype }
  }
  const id = number(rest)
  if (id === null) return null
  switch (kind) {
    case PageKind.Item:
      return { kind: 'item', id }
    case PageKind.Trinket:
      return { kind: 'trinket', id }
    case PageKind.Achievement:
      return { kind: 'achievement', id }
    case PageKind.Challenge:
      return { kind: 'challenge', number: id }
    case PageKind.Character:
      return { kind: 'character', id }
    default:
      return assertNever(kind)
  }
}
