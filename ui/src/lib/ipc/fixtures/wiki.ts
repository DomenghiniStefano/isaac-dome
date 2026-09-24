import { CollectibleTemplate } from '../types'
import { assertNever } from '@/lib/assertNever'
import type {
  Entry,
  Infobox,
  ExtractionReport,
  Target,
  UnlockView,
  WikiIndex,
  WikiInfo,
  WikiPageRef,
} from '../types'
import { pageKey, parsePageKey } from '@/lib/wiki/pageKey'

// Development only. The dataset as the fixtures can stand in for it: every page's identity
// and title from the index (items, trinkets, achievements, bosses, characters — real names,
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
  '../../../../fixtures/index.json',
  { eager: true, import: 'default' },
)
const samples = import.meta.glob<Entry>('../../../../fixtures/wiki/*.json', {
  eager: true,
  import: 'default',
})
const reports = import.meta.glob<ExtractionReport>(
  '../../../../fixtures/payload/extraction_report.json',
  { eager: true, import: 'default' },
)
const unlocks = import.meta.glob<UnlockView>(
  '../../../../fixtures/payload/unlock.json',
  { eager: true, import: 'default' },
)

const entries = (): IndexEntry[] => Object.values(indexes)[0] ?? []

// A sample file is named after its page key with `_` for `:` and `.`: `item_105`,
// `entity_20_0_0`.
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

export interface Page {
  target: Target
  title: string
}

const collectibleKinds = ['passive', 'active', 'familiar']

export const wikiPages = (): Page[] => {
  const all = entries()
  const items: Page[] = all
    .filter(
      (e) => e.family === 'item' && collectibleKinds.includes(e.kind ?? ''),
    )
    .map((e) => ({
      target: { kind: 'item', id: e.id },
      title: e.name ?? '',
    }))
  const trinkets: Page[] = all
    .filter((e) => e.family === 'item' && e.kind === 'trinket')
    .map((e) => ({
      target: { kind: 'trinket', id: e.id },
      title: e.name ?? '',
    }))
  const achievements: Page[] = all
    .filter((e) => e.family === 'achievement')
    .map((e) => ({
      target: { kind: 'achievement', id: e.id },
      title: achievementTitle(e.name ?? ''),
    }))
  const bosses: Page[] = all
    .filter((e) => e.family === 'boss')
    .flatMap((e) => {
      const target = bossTarget(e)
      return target ? [{ target, title: e.name ?? '' }] : []
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
    }))
  const characters: Page[] = all
    .filter((e) => e.family === 'character' && e.file.includes('/character_'))
    .map((e) => ({
      target: { kind: 'character', id: e.id },
      title: e.name ?? '',
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

// What an achievement asks of you, as the unlock payload words it: the search fixture
// matches on it the way the backend matches on `unlock_condition`.
export const wikiConditions = (): Map<number, string> =>
  new Map(
    (Object.values(unlocks)[0]?.nodes ?? []).flatMap((node) =>
      node.achievement.kind === 'known' && node.achievement.condition !== null
        ? [[node.achievement.id, node.achievement.condition] as const]
        : [],
    ),
  )

const missing: WikiInfo = { kind: 'missing', reason: 'malformed' }

// The recorded extraction report carries the real dataset's info; the counts are recomputed
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
    // There are no transformation pages here: a fixture that invented a number would
    // show the verification screen a count nothing produced.
    transformations: count('transformation'),
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
  withWiki: boolean
}

export const wikiIndexAnswer = ({ withWiki }: WikiAnswerOptions): WikiIndex => {
  if (!withWiki) return { info: missing, pages: [] }
  const list = wikiPages()
  const refs: WikiPageRef[] = list.map((p) => {
    // A sample page's own title wins over the index's name.
    const key = pageKey(p.target)
    const sample = key === null ? undefined : samplePages.get(key)
    return {
      target: p.target,
      title: sample?.title ?? p.title,
      // Null, like every drawing here: the app cuts its sprites from the user's own copy of
      // the game at runtime, and the development server has no copy to cut from.
      iconUrl: null,
    }
  })
  return { info: infoOf(list), pages: refs }
}

// The recorded pages predate most of the fields `Entry` has gained since (`fixtures/README.md`:
// "a field a payload predates is filled in by the fixture that reads it"). An item's infobox in
// there is literally `{ kind: 'item' }` — no quote, no quality, no prices — and a page has no
// `description`, no `dlc` and no `unlockedBy` at all. Completed here to today's shape, with
// **empty** values and never invented ones: empty is the state the app already draws as "the
// dataset didn't fill this", so what is missing reads as missing instead of as a fact.
//
// The one value that is a choice rather than an absence is a collectible's `template`: there is
// no empty for it, and `passive` is the reading that hides the recharge row rather than showing
// an empty one. The warning below says so, because on this server every item reads as passive.
// A recorded value read as what it is: a file written before the type grew. TypeScript is told
// the truth here — every field may be absent — so the fallbacks below are answers and not dead
// code the spread would overwrite.
type Recorded<T> = { [K in keyof T]?: T[K] }
type Box<K extends Infobox['kind']> = Recorded<Extract<Infobox, { kind: K }>>

const filledInfobox = (infobox: Infobox): Infobox => {
  switch (infobox.kind) {
    case 'item': {
      const box = infobox as Box<'item'>
      return {
        kind: 'item',
        quote: box.quote ?? [],
        template: box.template ?? CollectibleTemplate.Passive,
        quality: box.quality ?? null,
        tags: box.tags ?? [],
        recharge: box.recharge ?? [],
        devilPrice: box.devilPrice ?? [],
        shopPrice: box.shopPrice ?? [],
        pools: box.pools ?? [],
      }
    }
    case 'trinket': {
      const box = infobox as Box<'trinket'>
      return {
        kind: 'trinket',
        quote: box.quote ?? [],
        tags: box.tags ?? [],
        pools: box.pools ?? [],
      }
    }
    case 'achievement': {
      const box = infobox as Box<'achievement'>
      return {
        kind: 'achievement',
        quote: box.quote ?? [],
        requirements: box.requirements ?? [],
        notes: box.notes ?? [],
        unlocks: box.unlocks ?? null,
      }
    }
    case 'boss': {
      const box = infobox as Box<'boss'>
      return {
        kind: 'boss',
        baseHp: box.baseHp ?? null,
        stageHp: box.stageHp ?? [],
        variant: box.variant ?? null,
        environment: box.environment ?? [],
        pool: box.pool ?? [],
      }
    }
    case 'challenge': {
      const box = infobox as Box<'challenge'>
      return {
        kind: 'challenge',
        blindfolded: box.blindfolded ?? false,
        hasShops: box.hasShops ?? true,
        hasTreasureRooms: box.hasTreasureRooms ?? true,
        items: box.items ?? [],
        trinkets: box.trinkets ?? [],
        pickups: box.pickups ?? [],
        health: box.health ?? [],
        curse: box.curse ?? [],
        goal: box.goal ?? [],
        character: box.character ?? null,
        unlocks: box.unlocks ?? null,
      }
    }
    case 'transformation': {
      const box = infobox as Box<'transformation'>
      return {
        kind: 'transformation',
        requires: box.requires ?? null,
        contributors: box.contributors ?? [],
        target: box.target ?? [],
      }
    }
    case 'character': {
      const box = infobox as Box<'character'>
      return {
        kind: 'character',
        health: box.health ?? [],
        damage: box.damage ?? '',
        tears: box.tears ?? '',
        range: box.range ?? '',
        speed: box.speed ?? '',
        luck: box.luck ?? '',
        shotSpeed: box.shotSpeed ?? '',
        pickups: box.pickups ?? [],
        collectibles: box.collectibles ?? [],
        parent: box.parent ?? null,
      }
    }
    default:
      return assertNever(infobox)
  }
}

const filled = (recorded: Entry): Entry => {
  const entry = recorded as Recorded<Entry>
  return {
    title: entry.title ?? '',
    revid: entry.revid ?? 0,
    description: entry.description ?? [],
    dlc: entry.dlc ?? [],
    unlockedBy: entry.unlockedBy ?? null,
    infobox: filledInfobox(recorded.infobox),
    sections: entry.sections ?? [],
  }
}

let warned = false

// The eleven sample pages answer with their real text; every other page is one the fixtures
// don't carry, answered the way the app answers a page the dataset lacks.
export const wikiEntryAnswer = (target: Target): Entry | null => {
  if (!warned) {
    warned = true
    console.warn(
      `wiki fixture: ${samplePages.size} sample pages are recorded; every other page reads as unknown`,
    )
    console.warn(
      'wiki fixture: the pages predate the summary, the editions and every field of an item’s card; those read as empty here, and every collectible reads as passive',
    )
  }
  const key = pageKey(target)
  if (key === null) return null
  // Round-tripping the key guards the sample names against a target the app never writes.
  const sample = parsePageKey(key) === null ? undefined : samplePages.get(key)
  return sample === undefined ? null : filled(sample)
}

// The recorded extraction report, as the development-only verification page asks for it
// (card #80, item 10). A machine that recorded none answers the empty report it would have.
export const extractionReportAnswer = (): ExtractionReport | undefined =>
  Object.values(reports)[0]
