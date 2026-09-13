import type {
  Entry,
  ExtractionReport,
  Target,
  UnlockView,
  WikiIndex,
  WikiInfo,
  WikiPageRef,
} from '../types'
import { assertNever } from '@/lib/assertNever'
import { pageKey, parsePageKey } from '@/lib/wiki/pageKey'
import { packIconUrl } from './graphArt'

// Development only. The dataset as the pack can stand in for it: every page's identity and
// title from the image index (items, trinkets, achievements, bosses, characters — real names,
// real ids) and from the unlock payload (challenges), the eleven sample pages with their real
// text, and the dataset's own provenance from the extraction report. Any other page is one
// the fixture doesn't carry, answered as the app answers a page the dataset lacks.
interface IndexEntry {
  file: string
  family: string
  id: number
  kind?: string
  name?: string
  source?: string
}
const indexes = import.meta.glob<IndexEntry[]>(
  '../../../../../design-export/isaacdome-design-pack/images/INDEX.json',
  { eager: true, import: 'default' },
)
const samples = import.meta.glob<Entry>(
  '../../../../../design-export/isaacdome-design-pack/wiki/samples/*.json',
  { eager: true, import: 'default' },
)
const reports = import.meta.glob<ExtractionReport>(
  '../../../../../design-export/isaacdome-design-pack/contracts/payload/extraction_report.json',
  { eager: true, import: 'default' },
)
const unlocks = import.meta.glob<UnlockView>(
  '../../../../../design-export/isaacdome-design-pack/contracts/payload/unlock.json',
  { eager: true, import: 'default' },
)
const bossFiles = import.meta.glob<string>(
  '../../../../../design-export/isaacdome-design-pack/images/boss/*.png',
  { eager: true, query: '?url', import: 'default' },
)
const characterFiles = import.meta.glob<string>(
  '../../../../../design-export/isaacdome-design-pack/images/characters/character_*.png',
  { eager: true, query: '?url', import: 'default' },
)

const entries = (): IndexEntry[] => Object.values(indexes)[0] ?? []

// A sample file is named after its page key with `_` for `:` and `.`: `item_105`,
// `entity_20_0_0`. The pack's INDEX.json sits in the same folder and is not a page.
const SampleFile = /\/([a-z]+)_(\d+(?:_\d+)*)\.json$/
const sampleKey = (path: string): string | null => {
  const m = SampleFile.exec(path)
  if (!m?.[1] || !m[2]) return null
  const [kind, ids] = [m[1], m[2]]
  return `${kind}:${ids.replaceAll('_', '.')}`
}
export const samplePages = new Map<string, Entry>(
  Object.entries(samples).flatMap(([path, entry]) => {
    const key = sampleKey(path)
    return key === null ? [] : [[key, entry] as const]
  }),
)

// The boss portrait's file name carries the entity key the wiki indexes bosses by
// (crates/ipc/src/target_sprite.rs): `Portrait_20.0_Monstro.png` is entity 20, variant 0.
const PortraitKey = /\/Portrait_(\d+)\.(\d+)_/
const bossTarget = (entry: IndexEntry): Target | null => {
  const m = PortraitKey.exec(entry.source ?? '')
  return m?.[1] && m[2]
    ? { kind: 'entity', id: Number(m[1]), variant: Number(m[2]), subtype: 0 }
    : null
}

// The wiki's page title is the achievement's name, not the game's "You unlocked" line.
const Unlocked = /^You unlocked "?(.*?)"?$/
const achievementTitle = (name: string): string =>
  Unlocked.exec(name)?.[1] ?? name

const fourDigits = (id: number): string => String(id).padStart(4, '0')
const twoDigits = (id: number): string => String(id).padStart(2, '0')
const fileWith = (files: Record<string, string>, stem: string): string | null =>
  Object.entries(files).find(([path]) => path.includes(`/${stem}_`))?.[1] ??
  null

export const iconOf = (
  target: Target,
  bossId: number | null,
): string | null => {
  switch (target.kind) {
    case 'item':
    case 'trinket': {
      const kind =
        target.kind === 'trinket'
          ? 'trinket'
          : (entries().find(
              (e) =>
                e.family === 'item' &&
                e.id === target.id &&
                e.kind !== 'trinket',
            )?.kind ?? 'passive')
      return packIconUrl(`isaac://item/${kind}/${target.id}`)
    }
    case 'achievement':
      return packIconUrl(`isaac://achievement/${target.id}`)
    case 'entity':
      return bossId === null ? null : fileWith(bossFiles, fourDigits(bossId))
    case 'character':
      return fileWith(characterFiles, `character_${twoDigits(target.id)}`)
    // The game has no picture for a challenge (DESIGN-BRIEF.md §5.7), and the pack none.
    case 'challenge':
    case 'stage':
    case 'room':
    case 'pickup':
    case 'transformation':
      return null
    default:
      return assertNever(target)
  }
}

export interface Page {
  target: Target
  title: string
  bossId: number | null
}

const collectibleKinds = ['passive', 'active', 'familiar']

export const packPages = (): Page[] => {
  const all = entries()
  const items: Page[] = all
    .filter(
      (e) => e.family === 'item' && collectibleKinds.includes(e.kind ?? ''),
    )
    .map((e) => ({
      target: { kind: 'item', id: e.id },
      title: e.name ?? '',
      bossId: null,
    }))
  const trinkets: Page[] = all
    .filter((e) => e.family === 'item' && e.kind === 'trinket')
    .map((e) => ({
      target: { kind: 'trinket', id: e.id },
      title: e.name ?? '',
      bossId: null,
    }))
  const achievements: Page[] = all
    .filter((e) => e.family === 'achievement')
    .map((e) => ({
      target: { kind: 'achievement', id: e.id },
      title: achievementTitle(e.name ?? ''),
      bossId: null,
    }))
  const bosses: Page[] = all
    .filter((e) => e.family === 'boss')
    .flatMap((e) => {
      const target = bossTarget(e)
      return target ? [{ target, title: e.name ?? '', bossId: e.id }] : []
    })
  const challenges = new Map<number, string>()
  for (const node of Object.values(unlocks)[0]?.nodes ?? []) {
    for (const t of node.unlocks) {
      if (t.kind === 'challenge') challenges.set(t.id, t.name)
    }
  }
  const challengePages: Page[] = [...challenges]
    .sort(([a], [b]) => a - b)
    .map(([id, name]) => ({
      target: { kind: 'challenge', number: id },
      title: name,
      bossId: null,
    }))
  const characters: Page[] = all
    .filter((e) => e.family === 'character' && e.file.includes('/character_'))
    .map((e) => ({
      target: { kind: 'character', id: e.id },
      title: e.name ?? '',
      bossId: null,
    }))
  return [
    ...items,
    ...trinkets,
    ...achievements,
    ...bosses,
    ...challengePages,
    ...characters,
  ]
}

// What an achievement asks of you, as the pack's unlock payload words it: the search fixture
// matches on it the way the backend matches on `unlock_condition`.
export const packConditions = (): Map<number, string> =>
  new Map(
    (Object.values(unlocks)[0]?.nodes ?? []).flatMap((node) =>
      node.achievement.kind === 'known' && node.achievement.condition !== null
        ? [[node.achievement.id, node.achievement.condition] as const]
        : [],
    ),
  )

const missing: WikiInfo = { kind: 'missing', reason: 'malformed' }

// The pack's extraction report carries the real dataset's info; the counts are recomputed
// from the pages this fixture actually lists, so the landing's cards and the lists agree.
const infoOf = (list: Page[]): WikiInfo => {
  const real = Object.values(reports)[0]?.wiki
  const count = (kind: Target['kind']) =>
    list.filter((p) => p.target.kind === kind).length
  const counts = {
    items: count('item'),
    trinkets: count('trinket'),
    achievements: count('achievement'),
    bosses: count('entity'),
    challenges: count('challenge'),
    characters: count('character'),
  }
  return real?.kind === 'loaded'
    ? { ...real, counts }
    : {
        kind: 'loaded',
        snapshotAt: '2026-09-05T14:20:41Z',
        lastKnownPatch: null,
        counts,
        unresolved: 0,
        unknownTemplates: 0,
        gameNewerThanSnapshot: null,
      }
}

export interface WikiAnswerOptions {
  withArt: boolean
  withWiki: boolean
}

export const wikiIndexAnswer = ({
  withArt,
  withWiki,
}: WikiAnswerOptions): WikiIndex => {
  if (!withWiki) return { info: missing, pages: [] }
  const list = packPages()
  const refs: WikiPageRef[] = list.map((p) => {
    // A sample page's own title wins over the image index's name.
    const key = pageKey(p.target)
    const sample = key === null ? undefined : samplePages.get(key)
    return {
      target: p.target,
      title: sample?.title ?? p.title,
      iconUrl: withArt ? iconOf(p.target, p.bossId) : null,
    }
  })
  return { info: infoOf(list), pages: refs }
}

let warned = false

// The eleven sample pages answer with their real text; every other page is one the pack
// doesn't carry, answered the way the app answers a page the dataset lacks.
export const wikiEntryAnswer = (target: Target): Entry | null => {
  if (!warned) {
    warned = true
    console.warn(
      `wiki fixture: the design pack carries ${samplePages.size} sample pages; every other page reads as unknown`,
    )
  }
  const key = pageKey(target)
  if (key === null) return null
  // Round-tripping the key guards the sample names against a target the app never writes.
  return parsePageKey(key) === null ? null : (samplePages.get(key) ?? null)
}
