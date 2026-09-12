import { assertNever } from '@/lib/assertNever'
import { pageKey } from '@/lib/wiki/pageKey'
import type {
  Block,
  Inline,
  SearchDiagnostic,
  SearchHit,
  SearchMatch,
  SearchView,
  Section,
  SectionKind,
  Target,
} from '../types'
import { iconOf, packConditions, packPages, samplePages } from './wiki'

// Development only, and **synthetic**: the ranking that counts is Rust's
// (`crates/ipc/src/search.rs`). This exists so the palette and the Search screen can be looked
// at without the backend, on the pack's real names and its eleven sample pages.
export interface FixtureSection {
  section: SectionKind
  text: string
}

export interface FixtureDoc {
  target: Target
  title: string
  condition: string | null
  sections: FixtureSection[]
  hasPage: boolean
  iconUrl: string | null
}

const fold = (s: string): string => s.toLowerCase()

const words = (query: string): string[] =>
  fold(query).trim().split(/\s+/).filter(Boolean)

const containsAll = (folded: string, all: string[]): boolean =>
  all.length > 0 && all.every((w) => folded.includes(w))

// The same window the backend cuts: enough of the sentence to read the match in place.
const Before = 60
const After = 90

const fragment = (
  section: SectionKind,
  text: string,
  all: string[],
): SearchMatch | null => {
  const folded = fold(text)
  if (!containsAll(folded, all)) return null
  const at = Math.min(...all.map((w) => folded.indexOf(w)))
  const word = all.find((w) => folded.indexOf(w) === at) ?? ''
  return {
    kind: 'section',
    section,
    before: text.slice(Math.max(0, at - Before), at),
    matched: text.slice(at, at + word.length),
    after: text.slice(at + word.length, at + word.length + After),
  }
}

const titleTier = (folded: string, query: string): number => {
  if (folded === query) return 0
  if (folded.startsWith(query)) return 1
  if (folded.split(/\s+/).some((w) => w.startsWith(query))) return 2
  return 3
}

// Exported for its test: the ranking is the only part of the fixture worth checking.
export const rankFixture = (
  docs: FixtureDoc[],
  query: string,
  limit: number,
  diagnostics: SearchDiagnostic[] = [],
): SearchView => {
  const all = words(query)
  if (all.length === 0) return { query, hits: [], total: 0, diagnostics: [] }
  const folded = fold(query.trim())
  const ranked = docs.flatMap((doc) => {
    const title = fold(doc.title)
    if (containsAll(title, all))
      return [
        {
          tier: titleTier(title, folded),
          doc,
          match: { kind: 'title' } as SearchMatch,
        },
      ]
    if (doc.condition !== null && containsAll(fold(doc.condition), all))
      return [
        {
          tier: 4,
          doc,
          match: { kind: 'condition', text: doc.condition } as SearchMatch,
        },
      ]
    for (const s of doc.sections) {
      const match = fragment(s.section, s.text, all)
      if (match) return [{ tier: 5, doc, match }]
    }
    return []
  })
  ranked.sort(
    (a, b) => a.tier - b.tier || a.doc.title.localeCompare(b.doc.title),
  )
  const hits: SearchHit[] = ranked.slice(0, limit).map(({ doc, match }) => ({
    target: doc.target,
    title: doc.title,
    iconUrl: doc.iconUrl,
    hasPage: doc.hasPage,
    match,
    // The fixture has no save behind it: every mark is unknown, as `noProfile` says.
    progress: 'unknown',
  }))
  return { query, hits, total: ranked.length, diagnostics }
}

const inlineText = (inline: Inline[]): string[] =>
  inline.flatMap((i) => {
    switch (i.kind) {
      case 'text':
        return [i.text]
      case 'ref':
      case 'concept':
        return [i.label]
      case 'edition':
        return inlineText(i.inline)
      default:
        return assertNever(i)
    }
  })

const blockText = (block: Block): string[] => {
  switch (block.kind) {
    case 'paragraph':
    case 'heading':
      return inlineText(block.inline)
    case 'list':
      return block.items.flatMap((item) => [
        ...inlineText(item.inline),
        ...item.children.flatMap(blockText),
      ])
    case 'table':
      return [
        ...block.header.flatMap(inlineText),
        ...block.rows.flatMap((row) => row.flatMap(inlineText)),
      ]
    default:
      return assertNever(block)
  }
}

// One string per section, as `crates/ipc` flattens it: pieces joined by a single space.
const sectionText = (section: Section): string =>
  section.blocks
    .flatMap(blockText)
    .map((piece) => piece.trim())
    .filter(Boolean)
    .join(' ')

export interface SearchAnswerOptions {
  withArt: boolean
  withCatalog: boolean
  withWiki: boolean
}

let warned = false

export const searchAnswer = (
  { withArt, withCatalog, withWiki }: SearchAnswerOptions,
  query: string,
  limit: number,
): SearchView => {
  if (!warned) {
    warned = true
    console.warn(
      'search fixture: the ranking is synthetic; the real one lives in crates/ipc',
    )
  }
  const conditions = packConditions()
  const diagnostics: SearchDiagnostic[] = [
    ...(withCatalog ? [] : (['noCatalog'] as const)),
    ...(withWiki ? [] : (['noWiki'] as const)),
    // The fixture never has a save behind it, whatever profile the scenario shows.
    'noProfile' as const,
  ]
  // The pack's names are the **catalog's**, so they answer even with no dataset: what a
  // missing dataset takes away is the text of the sections and the page to open, exactly as
  // the backend degrades (spec 3.5, Decision 9).
  const docs: FixtureDoc[] = packPages().map((page) => {
    const key = pageKey(page.target)
    const sample = withWiki && key !== null ? samplePages.get(key) : undefined
    return {
      target: page.target,
      title: sample?.title ?? page.title,
      condition:
        withCatalog && page.target.kind === 'achievement'
          ? (conditions.get(page.target.id) ?? null)
          : null,
      sections: (sample?.sections ?? []).map((s) => ({
        section: s.kind,
        text: sectionText(s),
      })),
      hasPage: withWiki,
      iconUrl: withArt && withCatalog ? iconOf(page.target, page.bossId) : null,
    }
  })
  return rankFixture(docs, query, limit, diagnostics)
}
