import type { RunView } from '@/lib/ipc/types'

// The order of the diary, and the reason it is not "by date".
//
// **There is no timestamp on the wire.** `log.txt` carries no clock, so a run knows only its
// source and its position in it. The one wall clock in the archive is the name of an online
// session's folder — `09_12_2026__13_34_26` — and the launch being watched has no folder at
// all. So `Live` is first *by decision*, because it is the run you are playing, and sessions
// follow newest first.

const SESSION_NAME = /^(\d{2})_(\d{2})_(\d{4})__(\d{2})_(\d{2})_(\d{2})$/

/**
 * The instant a session's folder name states, or `null` when the name is not one — a
 * `desyncs` folder, a name from another tool, a rename by hand. UTC because nothing here is
 * compared against a local time: it is used to order, not to display.
 */
export const sessionTime = (name: string): number | null => {
  const m = SESSION_NAME.exec(name)
  if (m === null) return null
  const [month, day, year, hour, minute, second] = m.slice(1).map(Number)
  const at = Date.UTC(year, month - 1, day, hour, minute, second)
  // `Date.UTC` accepts a 13th month by rolling over; a name that rolls over is not a name we
  // read, it is a name we guessed at.
  const rolled =
    new Date(at).getUTCMonth() !== month - 1 ||
    new Date(at).getUTCDate() !== day
  return rolled ? null : at
}

/** Every run of one source keeps its own place; a source is ordered once. */
const sourceKey = (run: RunView): string =>
  run.source.kind === 'live' ? 'live' : `session:${run.source.name}`

/**
 * The list as the screen draws it: the watched launch, then the sessions whose name is a
 * clock, newest first, then — **in the place the archive gave them** — the ones whose name is
 * not. Sorting an unreadable name to either end would say something about when it happened.
 *
 * The array it receives is not touched: the store hands out what it read.
 */
export const orderRuns = (runs: RunView[]): RunView[] => {
  // The sources in the order the archive gave them, each once.
  const sources: string[] = []
  const live: string[] = []
  for (const run of runs) {
    const key = sourceKey(run)
    if (sources.includes(key) || live.includes(key)) continue
    ;(run.source.kind === 'live' ? live : sources).push(key)
  }

  // Only the names that are a clock are sorted, and they are sorted **into the slots they
  // already occupy**: a name we cannot read keeps its position instead of being pushed to
  // one end, and the result is a total order rather than a comparator that can contradict
  // itself — with an unreadable session in the middle, "newer first" and "keeps its place"
  // are two rules that a single comparison function cannot both obey.
  const timed = sources
    .map((key, at) => ({
      key,
      at,
      time: sessionTime(key.slice('session:'.length)),
    }))
    .filter((s) => s.time !== null)
  const slots = timed.map((s) => s.at)
  const byTime = [...timed].sort((a, b) => (b.time ?? 0) - (a.time ?? 0))
  const ordered = [...sources]
  slots.forEach((at, i) => {
    ordered[at] = byTime[i].key
  })

  const rank = new Map([...live, ...ordered].map((key, at) => [key, at]))
  return [...runs].sort((a, b) => {
    const [ka, kb] = [sourceKey(a), sourceKey(b)]
    if (ka !== kb) return (rank.get(ka) ?? 0) - (rank.get(kb) ?? 0)
    return b.ordinal - a.ordinal
  })
}
