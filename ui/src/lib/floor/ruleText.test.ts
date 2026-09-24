import { camelCase } from 'lodash-es'
import { describe, expect, it as test } from 'vitest'
import { en } from '@/i18n/messages/en'
import { it } from '@/i18n/messages/it'
import placement from '../../../../crates/floor/rules/placement.json?raw'
import { ruleNoteKey, ruleTextKey } from './ruleText'

// The rules file the backend embeds, read as it is: the translations below are checked against
// the rules that exist, not against a list written a second time in this file.
interface PlacementRule {
  id: string
  quote: string
  constraint: { kind: string }
}
const rules: PlacementRule[] = JSON.parse(placement).rules

// The two constraints that reach the screen as "cannot judge": one the grid never models, and
// the distance rule while there is no start room to count from (`floor::solve`).
const JUDGED_LATER = ['unmodelled', 'deadEndDistanceRank']

const lookup = (tree: object, key: string): unknown =>
  key.split('.').reduce<unknown>((node, part) => {
    if (node === null || typeof node !== 'object') return undefined
    return (node as Record<string, unknown>)[part]
  }, tree)

describe('the rules in the language of the app', () => {
  test('the series is not empty', () => {
    expect(rules.length).toBeGreaterThan(0)
  })

  test('every rule has an Italian sentence', () => {
    const missing = rules.filter((rule) => ruleTextKey(rule.id) === null)
    expect(missing.map((rule) => rule.id)).toEqual([])
  })

  test('in English the sentence is the wiki quote itself, word for word', () => {
    for (const rule of rules) {
      const key = ruleTextKey(rule.id)
      expect(key).not.toBeNull()
      expect(lookup(en, key!)).toBe(rule.quote)
    }
  })

  test('every translated sentence names a rule that exists', () => {
    const ids = new Set(rules.map((rule) => camelCase(rule.id)))
    const orphans = Object.keys(it.floor.ruleText).filter(
      (name) => !ids.has(name),
    )
    expect(orphans).toEqual([])
  })

  test('every rule the grid cannot judge says why, in both languages', () => {
    const later = rules.filter((rule) =>
      JUDGED_LATER.includes(rule.constraint.kind),
    )
    expect(later.length).toBeGreaterThan(0)
    for (const rule of later) {
      const key = ruleNoteKey(rule.id)
      expect(key).not.toBeNull()
      expect(lookup(it, key!)).toEqual(expect.any(String))
      expect(lookup(en, key!)).toEqual(expect.any(String))
    }
  })

  test('a note belongs to a rule that can reach the screen unjudged', () => {
    const later = new Set(
      rules
        .filter((rule) => JUDGED_LATER.includes(rule.constraint.kind))
        .map((rule) => camelCase(rule.id)),
    )
    const orphans = Object.keys(it.floor.ruleNote).filter(
      (name) => !later.has(name),
    )
    expect(orphans).toEqual([])
  })

  test('a rule nobody translated has no key, so the caller can fall back to the quote', () => {
    expect(ruleTextKey('fixture-secret')).toBeNull()
    expect(ruleNoteKey('fixture-secret')).toBeNull()
  })
})
