import { describe, expect, it } from 'vitest'
import { en } from '@/i18n/messages/en'
import { it as itMessages } from '@/i18n/messages/it'
import { collectionEntries } from './collection'
import { planEntries } from './plan'
import { searchEntries } from './search'
import { Severity } from './spec'
import { unlockEntries } from './unlock'
import type { DiagnosticEntry } from './spec'
import type {
  CollectionDiagnostic,
  QueueDiagnostic,
  SearchDiagnostic,
  UnlockDiagnostic,
} from '@/lib/ipc/types'

const has = (messages: Record<string, unknown>, key: string): boolean =>
  key
    .split('.')
    .reduce<unknown>(
      (node, part) =>
        node && typeof node === 'object'
          ? (node as Record<string, unknown>)[part]
          : undefined,
      messages,
    ) !== undefined

const keysExist = (entries: DiagnosticEntry[]): void => {
  for (const e of entries) {
    for (const part of [...e.title, ...e.body]) {
      expect(has(en, part.key), `en is missing ${part.key}`).toBe(true)
      expect(has(itMessages, part.key), `it is missing ${part.key}`).toBe(true)
    }
  }
}

const EVERY_UNLOCK: UnlockDiagnostic[] = [
  { kind: 'noCatalog' },
  { kind: 'noAchievementSection' },
  { kind: 'slotsBeyondCatalog', count: 3 },
  { kind: 'catalogBeyondSlots', count: 2 },
]

const EVERY_COLLECTION: CollectionDiagnostic[] = [
  { kind: 'noCatalog' },
  { kind: 'noCollectionSection' },
  { kind: 'noAchievementSection' },
  { kind: 'itemsBeyondSlots', count: 4 },
]

const EVERY_SEARCH: SearchDiagnostic[] = [
  'noProfile',
  'noCatalog',
  'noWiki',
  'noAchievementSection',
  'noCollectionSection',
]

const EVERY_QUEUE: QueueDiagnostic[] = [
  { kind: 'storeUnavailable', reason: { kind: 'unreadable' } },
  { kind: 'unreadable' },
  { kind: 'completed', count: 2, wanted: [41] },
  { kind: 'unresolved', achievement: 9 },
  { kind: 'goalsPending', count: 3 },
  { kind: 'noCatalog' },
]

describe('one table per screen, one list that draws them', () => {
  it('keeps a count-carrying diagnostic a note, not an alarm', () => {
    const [entry] = unlockEntries([{ kind: 'slotsBeyondCatalog', count: 3 }])
    expect(entry?.severity).toBe(Severity.Note)
    expect(entry?.body[0]?.params).toEqual({ count: 3 })
  })

  it('raises an unread section to a warning on every screen that can see one', () => {
    expect(unlockEntries([{ kind: 'noAchievementSection' }])[0]?.severity).toBe(
      Severity.Warning,
    )
    expect(
      collectionEntries([{ kind: 'noCollectionSection' }])[0]?.severity,
    ).toBe(Severity.Warning)
    expect(searchEntries(['noAchievementSection'])[0]?.severity).toBe(
      Severity.Warning,
    )
  })

  // `noProfile` is the expected state before a save is chosen, not a failure.
  it('leaves "no profile" and "no catalog" as information', () => {
    expect(searchEntries(['noProfile'])[0]?.severity).toBe(Severity.Info)
    expect(unlockEntries([{ kind: 'noCatalog' }])[0]?.severity).toBe(
      Severity.Info,
    )
  })

  // The one alert that asks for something. It stays a row in a table; the button is the
  // screen's, handed in through a slot.
  it('marks the goals-pending row as the one with an action', () => {
    const [entry] = planEntries([{ kind: 'goalsPending', count: 3 }])
    expect(entry?.action).toBe('importGoals')
    expect(entry?.title[0]?.params).toEqual({ count: 3 })
  })

  // N2 made the store reason a variant; the row carries both halves as keys.
  it('builds the store reason from keys, never from a sentence', () => {
    const [entry] = planEntries([
      {
        kind: 'storeUnavailable',
        reason: { kind: 'newerSchema', found: 7, supported: 1 },
      },
    ])
    expect(entry?.body.map((p) => p.key)).toEqual([
      'ipcErrors.storeUnavailable',
      'ipcReasons.storeNewerSchema',
    ])
    expect(entry?.body[1]?.params).toEqual({ found: 7, supported: 1 })
  })

  // The rows the screen draws under the table itself are not diagnostics to show here.
  it('leaves completed and unresolved rows to the table that owns them', () => {
    expect(
      planEntries([{ kind: 'completed', count: 2, wanted: [41] }]),
    ).toEqual([])
    expect(planEntries([{ kind: 'unresolved', achievement: 9 }])).toEqual([])
  })

  it('has a text in both locales for every kind every screen can produce', () => {
    keysExist(unlockEntries(EVERY_UNLOCK))
    keysExist(collectionEntries(EVERY_COLLECTION))
    keysExist(searchEntries(EVERY_SEARCH))
    keysExist(planEntries(EVERY_QUEUE))
  })
})

// The gap the first version of this test had: `keysExist` proves a key resolves, not that
// it uses what it was handed. A string written for the old `${count} ${t(...)}` shape has no
// `{count}` in it, so the number disappears and nothing fails.
//
// The invariant is per entry, not per part: the builder hands a diagnostic's values to the
// title and the body alike, and it is enough that one of them places each. What must never
// happen is a value handed over and rendered nowhere.
const text = (messages: Record<string, unknown>, key: string): string =>
  key
    .split('.')
    .reduce<unknown>(
      (node, part) =>
        node && typeof node === 'object'
          ? (node as Record<string, unknown>)[part]
          : undefined,
      messages,
    ) as string

const paramsAreUsed = (entries: DiagnosticEntry[]): void => {
  for (const e of entries) {
    const parts = [...e.title, ...e.body]
    const names = new Set(parts.flatMap((p) => Object.keys(p.params ?? {})))
    for (const name of names) {
      for (const [locale, messages] of [
        ['en', en],
        ['it', itMessages],
      ] as const) {
        const placeholder = '{' + name + '}'
        const used = parts.some((p) =>
          text(messages, p.key).includes(placeholder),
        )
        expect(
          used,
          locale + ' never spends ' + placeholder + ' in ' + e.key,
        ).toBe(true)
      }
    }
  }
}

describe('a value handed to a translation is a value the translation uses', () => {
  it('spends every param it is given, in both locales', () => {
    paramsAreUsed(unlockEntries(EVERY_UNLOCK))
    paramsAreUsed(collectionEntries(EVERY_COLLECTION))
    paramsAreUsed(searchEntries(EVERY_SEARCH))
    paramsAreUsed(planEntries(EVERY_QUEUE))
  })
})
