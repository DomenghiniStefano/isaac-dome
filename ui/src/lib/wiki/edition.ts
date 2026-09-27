import { Dlc } from '@/lib/ipc/types'

// `Entry.dlc` in release order (`Object.values(Dlc)`, the same reading `editionLabel` uses):
// the Rust enum's declaration order **is** the release order, so this needs no order of its
// own — a sixth edition added to `Dlc` slots in here with no change.
const releaseOrder = Object.values(Dlc)

// The edition that added the entry, for the page header's "Added in …". `null` covers both
// an unrestricted entry (`dlc` empty, the wiki's "no restriction") and one that has existed
// since the original release: `Dlc.Rebirth` is the default assumption, not a restriction
// worth flagging.
export const editionAdded = (dlc: Dlc[]): Dlc | null => {
  const earliest = releaseOrder.find((edition) => dlc.includes(edition))
  return earliest && earliest !== Dlc.Rebirth ? earliest : null
}

// The edition that removed the entry, for the header's "Removed in …": the release right
// after the last one `dlc` names. `null` when `dlc` is empty (no restriction stated) or its
// range still reaches the newest edition (never removed).
export const editionRemoved = (dlc: Dlc[]): Dlc | null => {
  // No `findLastIndex`: the reversed copy keeps the same "first match wins" reading the
  // rest of this file uses, without raising the project's target just for one call.
  const latest = [...releaseOrder]
    .reverse()
    .find((edition) => dlc.includes(edition))
  if (!latest) return null
  return releaseOrder[releaseOrder.indexOf(latest) + 1] ?? null
}
