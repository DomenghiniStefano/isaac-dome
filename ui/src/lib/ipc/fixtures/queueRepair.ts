import { clamp, findLastIndex } from 'lodash-es'

// Development only: a port of crates/plan/src/order.rs `move_after` and `move_row`, so the
// development server shows a queue that repairs. The app's order is the backend's; this exists
// to look at the screen, and its tests are the Rust ones.
export type Requires = (a: number, b: number) => boolean

// `to` is a position once the row is taken out. Dependents are dragged right below it,
// prerequisites are a wall it stops under, everything else keeps its order.
export const moveRow = (
  ids: number[],
  achievement: number,
  to: number,
  requires: Requires,
): number[] => {
  if (!ids.includes(achievement)) return ids
  const others = ids.filter((id) => id !== achievement)
  const dragged = others.filter((id) => requires(id, achievement))
  const rest = others.filter((id) => !requires(id, achievement))
  const target =
    to - others.slice(0, to).filter((id) => requires(id, achievement)).length
  const floor = findLastIndex(rest, (id) => requires(achievement, id)) + 1
  const landed = clamp(target, floor, rest.length)
  return [
    ...rest.slice(0, landed),
    achievement,
    ...dragged,
    ...rest.slice(landed),
  ]
}

// Right below `after`, or the top. A stale anchor, or the row itself, changes nothing.
export const moveAfter = (
  ids: number[],
  achievement: number,
  after: number | null,
  requires: Requires,
): number[] => {
  const from = ids.indexOf(achievement)
  if (from < 0 || after === achievement) return ids
  if (after === null) return moveRow(ids, achievement, 0, requires)
  const at = ids.indexOf(after)
  if (at < 0) return ids
  return moveRow(ids, achievement, at < from ? at + 1 : at, requires)
}
