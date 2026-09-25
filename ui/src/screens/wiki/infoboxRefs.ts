import type { Inline, Target } from '@/lib/ipc/types'
import { pageKey } from '@/lib/wiki/pageKey'

// A list of targets as bare references, resolved through the index and kept in the page's
// own order — `WikiInline` already draws consecutive refs as a row (the transformation
// card's `contributors`, B40), so the only work here is naming each one: the page's title
// from the index when it is known, its key while the index isn't, `''` for the handful of
// kinds that carry no key at all (`pageKey`).
export const refsOf = (
  targets: Array<Target>,
  titleOf: (key: string) => string | null,
): Array<Inline> =>
  targets.map((target) => {
    const key = pageKey(target)
    const label = (key === null ? null : titleOf(key)) ?? key ?? ''
    return { kind: 'ref', target, label }
  })

// A single `Target` field becomes a one-reference inline, so it links like any other; no field
// is no reference, and the row then says "none".
export const refOf = (
  target: Target | null,
  titleOf: (key: string) => string | null,
): Array<Inline> => (target === null ? [] : refsOf([target], titleOf))
