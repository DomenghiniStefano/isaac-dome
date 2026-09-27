import type { Inline, Target } from '@/lib/ipc/types'
import { pageKey } from '@/lib/wiki/pageKey'

// A target's own title from the index when it is known, its key while the index isn't, `''`
// for the handful of kinds that carry no key at all (`pageKey`) — the one resolution every
// reference on a page goes through, whether it is a word in a sentence (`refsOf`, below) or
// the whole fact (`InfoboxRefRow`).
export const titleOrKey = (
  target: Target,
  titleOf: (key: string) => string | null,
): string => {
  const key = pageKey(target)
  return (key === null ? null : titleOf(key)) ?? key ?? ''
}

// A list of targets as bare references, resolved through the index and kept in the page's
// own order — `WikiInline` already draws consecutive refs as a row (the transformation
// card's `contributors`, B40), so the only work here is naming each one.
export const refsOf = (
  targets: Array<Target>,
  titleOf: (key: string) => string | null,
): Array<Inline> =>
  targets.map((target) => ({
    kind: 'ref',
    target,
    label: titleOrKey(target, titleOf),
  }))

// A single `Target` field becomes a one-reference inline, so it links like any other; no field
// is no reference, and the row then says "none".
export const refOf = (
  target: Target | null,
  titleOf: (key: string) => string | null,
): Array<Inline> => (target === null ? [] : refsOf([target], titleOf))
