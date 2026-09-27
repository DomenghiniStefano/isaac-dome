import type {
  CategorySample,
  Entry,
  ExtractionReport,
  PageFacts,
  PageProgress,
  PageProgressEntry,
  PoolMembershipView,
  Target,
  UnlockView,
  WikiIndex,
  WikiInfo,
  WikiPageRef,
  WikiProgress,
} from '../types'
import { WikiPageCategory } from '../types'
import { categoryOf } from '@/lib/wiki/category'
import { pageKey, parsePageKey } from '@/lib/wiki/pageKey'
import { warnOnce } from './warnOnce'

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
  const challenges = new Map<number, string>(
    (Object.values(unlocks)[0]?.nodes ?? []).flatMap((node) =>
      node.unlocks.flatMap((t) =>
        t.kind === 'challenge' ? [[t.id, t.name] as const] : [],
      ),
    ),
  )
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
//
// The recorded payload predates the resolved condition (graph.ts's `withCondition` reads the
// same fact): it still carries the game file's `hint`, not the `condition` the real type
// declares, so it is read with the same cast.
export const wikiConditions = (): Map<number, string> =>
  new Map(
    (Object.values(unlocks)[0]?.nodes ?? []).flatMap((node) => {
      if (node.achievement.kind !== 'known') return []
      const { hint } = node.achievement as unknown as {
        hint: string | null | undefined
      }
      return hint ? [[node.achievement.id, hint] as const] : []
    }),
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
    // There are no transformation, entity or article pages here: a fixture that invented
    // a number would show the verification screen a count nothing produced.
    transformations: count('transformation'),
    monsters: 0,
    cardsAndRunes: 0,
    pickups: 0,
    stages: 0,
    versions: 0,
    articles: 0,
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

// One sample per landing tile, null like every picture this fixture draws (the development
// server has no copy of the game to cut a sprite from — see `refs` below for the same
// reasoning). `target` is `null` too: this fixture never runs the real `category_sample`
// choice, so it has none to report, the same "not modelled" reading every other field here
// gets. What the Kit page checks here is the fallback the tile draws without one, the state
// every category is actually in on a machine with no game installed.
const categorySamples: CategorySample[] = Object.values(WikiPageCategory).map(
  (category) => ({ category, iconUrl: null, target: null }),
)

// This fixture's `Page` carries only an identity and a title, never an infobox: there is no
// wikitext to derive a real `PageFacts` from. Every field reads as "not stated" — `null`,
// `false`, `''`, `[]` — the same shape a page with an empty infobox gets from the real
// `facts()`, never an invented value. Only the kinds `wikiPages()` ever produces are named;
// the rest fall to the `article` shape with no category, since this fixture carries no
// article at all.
const factsOf = (target: Target): PageFacts => {
  switch (target.kind) {
    case 'item':
      return {
        kind: 'item',
        quality: null,
        activated: false,
        recharge: null,
        shopPrice: null,
        devilPrice: null,
        tags: [],
      }
    case 'trinket':
      return { kind: 'trinket', tags: [] }
    case 'achievement':
      return { kind: 'achievement', requirement: '', unlocks: null }
    case 'entity':
      return { kind: 'boss', baseHp: null, floors: '' }
    case 'challenge':
      return {
        kind: 'challenge',
        character: null,
        goal: '',
        blindfolded: false,
        curse: '',
      }
    case 'character':
      return {
        kind: 'character',
        health: '',
        damage: '',
        tears: '',
        range: '',
        speed: '',
        luck: '',
        shotSpeed: '',
        tainted: false,
      }
    case 'transformation':
    case 'stage':
    case 'room':
    case 'concept':
    case 'article':
      return { kind: 'article', category: null, version: null }
  }
}

export const wikiIndexAnswer = ({ withWiki }: WikiAnswerOptions): WikiIndex => {
  if (!withWiki) return { info: missing, pages: [], samples: [] }
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
      // `categoryOf` is exact here (never approximate the way its own doc comment warns
      // about): this fixture's only `entity` pages are real bosses and it carries no
      // article at all, so the ambiguity that function can't resolve never arises.
      category: categoryOf(p.target),
      // No fixture ever recorded a page's editions: the same "not stated" `facts` gets.
      dlc: [],
      facts: factsOf(p.target),
    }
  })
  return { info: infoOf(list), pages: refs, samples: categorySamples }
}

const warnSamples = warnOnce(
  `wiki fixture: ${samplePages.size} sample pages are recorded; every other page reads as unknown`,
)

// `obtainedFrom` (design decision 3, card #90) was added to `Infobox::Item`/`Infobox::Trinket`
// after every sample here was last recorded: the field the recording predates is filled in by
// the reader, with the value that recording would have carried (`[]`, "the wiki states no
// guaranteed source" — this file's own README, "a field a payload predates").
const withObtainedFrom = (entry: Entry): Entry => {
  const box = entry.infobox
  return box.kind === 'item' || box.kind === 'trinket'
    ? { ...entry, infobox: { ...box, obtainedFrom: box.obtainedFrom ?? [] } }
    : entry
}

// The sample pages answer with their real text; every other page is one the fixtures don't
// carry, answered the way the app answers a page the dataset lacks.
export const wikiEntryAnswer = (target: Target): Entry | null => {
  warnSamples()
  const key = pageKey(target)
  if (key === null) return null
  // Round-tripping the key guards the sample names against a target the app never writes.
  const sample = parsePageKey(key) === null ? undefined : samplePages.get(key)
  return sample ? withObtainedFrom(sample) : null
}

// The recorded extraction report, as the development-only verification page asks for it. A
// machine that recorded none answers the empty report it would have.
export const extractionReportAnswer = (): ExtractionReport | undefined =>
  Object.values(reports)[0]

// No pools payload was ever recorded (nothing here produces one any more — see this
// directory's `README.md`), and this fixture never invents numbers `infoOf` doesn't already
// have a source for. So the pools row reads as it does on a machine with no game installed,
// with or without the game shown elsewhere: `null`, absent rather than wrong.
export const wikiItemPoolsAnswer = (): PoolMembershipView[] | null => null

// One page's progress, from what this fixture actually has: an achievement's `done` is the
// recorded unlock payload's own (the same fact `wikiConditions` reads), everything else this
// fixture has no recorded save for reads as a plain, uncommitted state — never invented as
// "0 of N" the way the real command never would either. `null` for a kind this fixture
// carries no gate for at all (trinkets) or the kind states nothing about (decision 6's table).
const progressOf = (
  target: Target,
  doneById: Map<number, boolean>,
): PageProgress | null => {
  switch (target.kind) {
    case 'achievement':
      return { kind: 'achievement', done: doneById.get(target.id) ?? false }
    case 'item':
      return {
        kind: 'item',
        collected: null,
        unlocked: null,
        unlockedBy: null,
      }
    case 'entity':
      return { kind: 'bestiary', met: 0, killed: 0, killedYou: 0 }
    case 'challenge':
      return { kind: 'challenge', state: { kind: 'available' } }
    case 'character':
      return { kind: 'character', unlocked: true, marksDone: 0, marksTotal: 12 }
    case 'trinket':
    case 'transformation':
    case 'stage':
    case 'room':
    case 'concept':
    case 'article':
      return null
  }
}

// The save's state for every page this fixture's `wikiPages()` lists. No profile chosen
// answers `null`, the same "no save" the real command answers — never the rejection the
// commands that read a save's own bytes reject with.
export const wikiProgressAnswer = (
  hasProfile: boolean,
): WikiProgress | null => {
  if (!hasProfile) return null
  const doneById = new Map<number, boolean>(
    (Object.values(unlocks)[0]?.nodes ?? []).flatMap((node) =>
      node.achievement.kind === 'known'
        ? [[node.achievement.id, node.done] as const]
        : [],
    ),
  )
  const pages: PageProgressEntry[] = wikiPages().flatMap((p) => {
    const progress = progressOf(p.target, doneById)
    return progress ? [{ target: p.target, progress }] : []
  })
  return { pages }
}
