import { describe, expect, it } from 'vitest'
import type { UnlockNode, UnlockTarget } from '@/lib/ipc/types'
import {
  characterForms,
  characterLabel,
  characterValue,
  targetName,
} from './characterName'

const t = (key: string, params?: Record<string, unknown>) =>
  params ? `${key}:${JSON.stringify(params)}` : key

const node = (
  missing: UnlockNode['missing'],
  unlocks: UnlockTarget[] = [],
): UnlockNode => ({
  achievement: { kind: 'unknown', slot: 1 },
  done: false,
  unlocks,
  origin: null,
  missing,
  graph: { kind: 'partial', blockedBy: 0, fanOut: 0, unknown: 1 },
})

describe('characterLabel', () => {
  it("is the game's name for a base character", () => {
    expect(characterLabel(t, { name: 'The Lost', tainted: false })).toBe(
      'The Lost',
    )
  })

  it('says which form a tainted character is, through a message', () => {
    expect(characterLabel(t, { name: 'The Lost', tainted: true })).toBe(
      'graph.taintedName:{"name":"The Lost"}',
    )
  })
})

describe('characterValue', () => {
  it("is the character's id: the two forms share the name, not the id", () => {
    expect(characterValue({ id: 10 })).toBe('10')
    expect(characterValue({ id: 31 })).not.toBe(characterValue({ id: 10 }))
  })
})

describe('characterForms', () => {
  const nodes = [
    node([
      {
        kind: 'character',
        id: 31,
        name: 'The Lost',
        tainted: true,
        page: null,
      },
    ]),
    node([
      {
        kind: 'character',
        id: 10,
        name: 'The Lost',
        tainted: false,
        page: null,
      },
      { kind: 'boss', id: 1, name: 'Monstro', page: null },
    ]),
    node([
      {
        kind: 'character',
        id: 10,
        name: 'The Lost',
        tainted: false,
        page: null,
      },
    ]),
  ]

  it('reads every character a node is missing, once per id', () => {
    const forms = characterForms(nodes)
    expect([...forms.keys()].sort()).toEqual(['10', '31'])
    expect(forms.get('31')).toEqual({ name: 'The Lost', tainted: true })
    expect(forms.get('10')).toEqual({ name: 'The Lost', tainted: false })
  })
})

describe('targetName', () => {
  it("is the game's name for everything but a tainted character", () => {
    expect(
      targetName(t, { kind: 'boss', id: 1, name: 'Monstro', page: null }),
    ).toBe('Monstro')
    expect(
      targetName(t, {
        kind: 'character',
        id: 10,
        name: 'The Lost',
        tainted: false,
        page: null,
      }),
    ).toBe('The Lost')
    expect(
      targetName(t, {
        kind: 'character',
        id: 31,
        name: 'The Lost',
        tainted: true,
        page: null,
      }),
    ).toBe('graph.taintedName:{"name":"The Lost"}')
  })
})
