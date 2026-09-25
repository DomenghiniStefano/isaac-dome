import type {
  CollectionItem,
  CollectionView,
  ItemKindView,
  LockView,
  OriginView,
} from '../types'
import { graphAnswers } from './graph'
import { warnOnce } from './warnOnce'

// Development only. There is no recorded `collection` payload to read — writing one needs a
// machine with the game and a save, and nothing in the repository produces it — so the items
// are built here: ids, kinds and names from the fixtures' index, real; their locks from
// unlock.json, real; their origin from the catalog's id ranges, real; and their quality,
// pools and collection flag from the id: synthetic, and said so on the console.
interface IndexEntry {
  family: string
  id: number
  kind?: string
  name?: string
}
const indexes = import.meta.glob<IndexEntry[]>(
  '../../../../fixtures/index.json',
  { eager: true, import: 'default' },
)

export interface CollectionAnswerOptions {
  withCatalog: boolean
  collectionRead: boolean
}

// The reference save's section 4 length (the pack's save_summary.json declares 733 items).
const referenceSlots = 733

const collectibleKinds: string[] = ['passive', 'active', 'familiar']
const isCollectible = (kind: string | undefined): kind is ItemKindView =>
  kind !== undefined && collectibleKinds.includes(kind)

// crates/catalog/src/origin.rs, COLLECTIBLES: the last collectible id of each DLC.
const originOf = (id: number): OriginView | null => {
  if (id <= 0) return null
  if (id <= 341) return 'rebirth'
  if (id <= 440) return 'afterbirth'
  if (id <= 552) return 'afterbirthPlus'
  if (id <= 732) return 'repentance'
  return null
}

// Synthetic, from the id: enough variety to look at facets and pips, and nothing more.
const syntheticPools = [
  'treasure',
  'boss',
  'shop',
  'devil',
  'angel',
  'secret',
  'library',
]
const poolsOf = (id: number): string[] => {
  if (id % 11 === 0) return []
  const first = syntheticPools[id % 7] ?? 'treasure'
  const second = syntheticPools[(id + 3) % 7] ?? 'boss'
  return id % 3 === 0 ? [first, second] : [first]
}
const qualityOf = (id: number): number | null => (id % 23 === 0 ? null : id % 5)

const free: LockView = { kind: 'free' }

// The locks the reference profile's unlock view implies: an item a node unlocks is unlocked or
// locked by that node's done; any other item is free.
const locksFrom = (): Map<string, LockView> => {
  const nodes = graphAnswers({ withCatalog: true }).unlock.nodes
  return new Map(
    nodes.flatMap((node) => {
      const a = node.achievement
      if (a.kind !== 'known') return []
      return node.unlocks.flatMap((target) => {
        if (target.kind !== 'item') return []
        // `page: null` is derived, not missing: this lock is built here out of the unlock
        // payload, which carries no wiki page, and inventing one would be a link the real
        // backend might not have.
        const lock: LockView = node.done
          ? { kind: 'unlocked', achievement: a.id, text: a.text, page: null }
          : { kind: 'locked', achievement: a.id, text: a.text, page: null }
        return [[`${target.itemKind}-${target.id}`, lock] as const]
      })
    }),
  )
}

const itemOf = (
  entry: IndexEntry,
  kind: ItemKindView,
  lock: LockView,
  collectionRead: boolean,
): CollectionItem => ({
  id: entry.id,
  kind,
  name: entry.name ?? '',
  // Null, as everywhere in the fixtures: the app cuts its sprites from the user's own copy
  // of the game at runtime, and the development server has no copy to cut from.
  iconUrl: null,
  quality: qualityOf(entry.id),
  pools: poolsOf(entry.id),
  origin: originOf(entry.id),
  // A locked item can't have been found: the synthetic flag never says otherwise.
  inCollection: collectionRead
    ? lock.kind !== 'locked' && entry.id % 3 !== 0
    : null,
  lock,
})

const synthetic = (collectionRead: boolean): CollectionView => {
  const locks = locksFrom()
  const entries = (Object.values(indexes)[0] ?? [])
    .filter((e) => e.family === 'item')
    .sort((a, b) => a.id - b.id)
  const items = entries.flatMap((e) =>
    isCollectible(e.kind)
      ? [
          itemOf(
            e,
            e.kind,
            locks.get(`${e.kind}-${e.id}`) ?? free,
            collectionRead,
          ),
        ]
      : [],
  )
  return {
    items,
    pools: syntheticPools.filter((p) => items.some((i) => i.pools.includes(p))),
    totals: {
      slots: collectionRead ? referenceSlots : 0,
      items: items.length,
      inCollection: items.filter((i) => i.inCollection === true).length,
    },
    diagnostics: collectionRead ? [] : [{ kind: 'noCollectionSection' }],
  }
}

const declareSynthetic = warnOnce(
  'Collection fixture: quality, pools and collection flags are synthetic, the fixtures carrying no recorded collection payload',
)

// Said once on the development server, where someone is looking at the screen.
const warnSynthetic = (): void => {
  if (typeof window !== 'undefined') declareSynthetic()
}

export const collectionAnswer = ({
  withCatalog,
  collectionRead,
}: CollectionAnswerOptions): CollectionView => {
  if (!withCatalog)
    return {
      items: [],
      pools: [],
      totals: {
        slots: collectionRead ? referenceSlots : 0,
        items: 0,
        inCollection: 0,
      },
      diagnostics: [{ kind: 'noCatalog' }],
    }
  warnSynthetic()
  return synthetic(collectionRead)
}
