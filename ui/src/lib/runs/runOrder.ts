import { uniq } from 'lodash-es'
import { assertNever } from '@/lib/assertNever'
import type { RunView } from '@/lib/ipc/types'
import { sourcePart } from './runKey'

// The order of the diary: the launch being played first, by decision, then every other source
// newest first by the date it carries. A session is dated by its folder's name, a wall clock on
// the player's machine (`09_12_2026__13_34_26`); a launch by when its file was last written. A
// source with no date — a launch read before dates were kept, a name that is not a clock — keeps
// the place the archive gave it.

const SESSION_NAME = /^(\d{2})_(\d{2})_(\d{4})__(\d{2})_(\d{2})_(\d{2})$/

/**
 * The instant a session's folder name states, or `null` when the name is not one — a
 * `desyncs` folder, a name from another tool, a rename by hand. **Local time**: the name is
 * the player's wall clock, and the hour it is shown with must be the hour it says.
 */
export const sessionTime = (name: string): number | null => {
  const m = SESSION_NAME.exec(name)
  if (m === null) return null
  const [month, day, year, hour, minute, second] = m.slice(1).map(Number)
  const at = new Date(year, month - 1, day, hour, minute, second)
  // `Date` accepts a 13th month by rolling over; a name that rolls over is not a name we
  // read, it is a name we guessed at.
  const rolled = at.getMonth() !== month - 1 || at.getDate() !== day
  return rolled ? null : at.getTime()
}

/** When a run's source is dated, in epoch ms, or `null` when it carries no date. */
export const runTime = (run: RunView): number | null => {
  switch (run.source.kind) {
    case 'session':
      return sessionTime(run.source.name)
    case 'live':
    case 'launch':
      return run.source.writtenUnix === null
        ? null
        : run.source.writtenUnix * 1000
    default:
      return assertNever(run.source)
  }
}

/** Every run of one source keeps its own place; a source is ordered once. */
const sourceKey = (run: RunView): string => sourcePart(run.source)

/**
 * The list as the screen draws it: the watched launch, then the sources that carry a date,
 * newest first, then — **in the place the archive gave them** — the ones that do not. Sorting
 * an undated source to either end would say something about when it happened.
 *
 * The array it receives is not touched: the store hands out what it read.
 */
export const orderRuns = (runs: RunView[]): RunView[] => {
  // The sources in the order the archive gave them, each once. The archive is oldest first and
  // the diary newest first, so its order is read from the end: the place an undated source
  // keeps is then counted from the newest side, and it lands between its neighbours in time
  // instead of at their mirror image.
  // The live launch is told by its kind and not by its key: it is keyed like any other launch,
  // so that its runs keep their keys once a newer launch arrives.
  const keys = uniq(runs.map(sourceKey))
  const liveKeys = new Set(
    runs.filter((run) => run.source.kind === 'live').map(sourceKey),
  )
  const live = keys.filter((key) => liveKeys.has(key))
  const sources = keys.filter((key) => !liveKeys.has(key)).reverse()
  // A source's date, read off its first run: every run of a source carries the same one.
  // Reversed so that the first run of each source is the entry the map keeps.
  const firstOf = new Map(
    runs.map((run): [string, RunView] => [sourceKey(run), run]).reverse(),
  )

  // Only the dated sources are sorted, and they are sorted **into the slots they already
  // occupy**: an undated one keeps its position instead of being pushed to one end, and the
  // result is a total order rather than a comparator that can contradict itself — with an
  // undated source in the middle, "newer first" and "keeps its place" are two rules that a
  // single comparison function cannot both obey.
  const timed = sources
    .map((key, at) => {
      const first = firstOf.get(key)
      return { key, at, time: first === undefined ? null : runTime(first) }
    })
    .filter((s) => s.time !== null)
  const byTime = [...timed].sort((a, b) => (b.time ?? 0) - (a.time ?? 0))
  // The i-th slot a date occupies takes the i-th newest date.
  const slot = new Map(timed.map((s, i) => [s.at, i]))
  const ordered = sources.map((key, at) => {
    const i = slot.get(at)
    return i === undefined ? key : byTime[i].key
  })

  const rank = new Map([...live, ...ordered].map((key, at) => [key, at]))
  return [...runs].sort((a, b) => {
    const [ka, kb] = [sourceKey(a), sourceKey(b)]
    if (ka !== kb) return (rank.get(ka) ?? 0) - (rank.get(kb) ?? 0)
    return b.ordinal - a.ordinal
  })
}
