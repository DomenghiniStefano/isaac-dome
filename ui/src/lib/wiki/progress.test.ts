import { describe, expect, it } from 'vitest'
import type {
  PageFacts,
  PageProgress,
  Target,
  WikiPageRef,
} from '@/lib/ipc/types'
import { WikiCategory } from '@/router/routeTable'
import { categoryProgress, isComplete, overallProgress } from './progress'

const factsFor = (target: WikiPageRef['target']): PageFacts => {
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
    case 'achievement':
      return { kind: 'achievement', requirement: '', unlocks: null }
    case 'transformation':
    case 'trinket':
    case 'entity':
    case 'challenge':
    case 'stage':
    case 'room':
    case 'concept':
    case 'article':
      return { kind: 'article', category: null, version: null }
  }
}

const page = (
  target: Target,
  title: string,
  category: WikiPageRef['category'],
): WikiPageRef => ({
  target,
  title,
  iconUrl: null,
  category,
  dlc: [],
  facts: factsFor(target),
})

describe('isComplete', () => {
  it('reads an achievement by its own `done`', () => {
    expect(isComplete({ kind: 'achievement', done: true })).toBe(true)
    expect(isComplete({ kind: 'achievement', done: false })).toBe(false)
  })

  it('reads an item by `collected`, and answers null when the save never said', () => {
    expect(
      isComplete({
        kind: 'item',
        collected: true,
        unlocked: null,
        unlockedBy: null,
      }),
    ).toBe(true)
    expect(
      isComplete({
        kind: 'item',
        collected: false,
        unlocked: null,
        unlockedBy: null,
      }),
    ).toBe(false)
    expect(
      isComplete({
        kind: 'item',
        collected: null,
        unlocked: null,
        unlockedBy: null,
      }),
    ).toBeNull()
  })

  it('reads an unlockable by `unlocked`', () => {
    expect(
      isComplete({ kind: 'unlockable', unlocked: true, unlockedBy: 7 }),
    ).toBe(true)
    expect(
      isComplete({ kind: 'unlockable', unlocked: false, unlockedBy: 7 }),
    ).toBe(false)
  })

  it('reads a character by `unlocked`, and answers null when the save never said', () => {
    expect(
      isComplete({
        kind: 'character',
        unlocked: true,
        marksDone: 3,
        marksTotal: 12,
      }),
    ).toBe(true)
    expect(
      isComplete({
        kind: 'character',
        unlocked: null,
        marksDone: 0,
        marksTotal: 12,
      }),
    ).toBeNull()
  })

  it('reads a challenge as done only in the done state', () => {
    expect(isComplete({ kind: 'challenge', state: { kind: 'done' } })).toBe(
      true,
    )
    expect(
      isComplete({ kind: 'challenge', state: { kind: 'available' } }),
    ).toBe(false)
    expect(
      isComplete({
        kind: 'challenge',
        state: { kind: 'blocked', missing: [1] },
      }),
    ).toBe(false)
  })

  it('reads a bestiary entry as done once killed at least once', () => {
    expect(
      isComplete({ kind: 'bestiary', met: 1, killed: 0, killedYou: 0 }),
    ).toBe(false)
    expect(
      isComplete({ kind: 'bestiary', met: 1, killed: 2, killedYou: 0 }),
    ).toBe(true)
  })
})

describe('categoryProgress', () => {
  const isaac: Target = { kind: 'character', id: 0 }
  const taintedIsaac: Target = { kind: 'character', id: 21 }
  const achievement1: Target = { kind: 'achievement', id: 1 }
  const achievement2: Target = { kind: 'achievement', id: 2 }
  const transformation: Target = { kind: 'transformation', id: 1 }

  const pages: WikiPageRef[] = [
    page(isaac, 'Isaac', WikiCategory.Characters),
    page(taintedIsaac, 'Tainted Isaac', WikiCategory.Characters),
    page(achievement1, 'A1', WikiCategory.Achievements),
    page(achievement2, 'A2', WikiCategory.Achievements),
    page(transformation, 'Guppy', WikiCategory.Transformations),
  ]

  it('answers null without a save: every progressFor answers null', () => {
    expect(
      categoryProgress(pages, WikiCategory.Characters, () => null),
    ).toBeNull()
  })

  it('answers null for a category the save says nothing about', () => {
    const progressFor = (t: Target): PageProgress | null =>
      t.kind === 'transformation' ? null : { kind: 'achievement', done: true }
    expect(
      categoryProgress(pages, WikiCategory.Transformations, progressFor),
    ).toBeNull()
  })

  it('counts done and not done over a mixed category', () => {
    const progressFor = (t: Target): PageProgress | null => {
      if (t.kind !== 'achievement') return null
      return { kind: 'achievement', done: t.id === 1 }
    }
    expect(
      categoryProgress(pages, WikiCategory.Achievements, progressFor),
    ).toEqual({ done: 1, total: 2 })
  })

  it('looks Isaac and Tainted Isaac up as two distinct pages, by id and not by name', () => {
    const progressFor = (t: Target): PageProgress | null => {
      if (t.kind !== 'character') return null
      return {
        kind: 'character',
        unlocked: t.id === 21,
        marksDone: 0,
        marksTotal: 12,
      }
    }
    expect(
      categoryProgress(pages, WikiCategory.Characters, progressFor),
    ).toEqual({ done: 1, total: 2 })
  })
})

describe('overallProgress', () => {
  const item: Target = { kind: 'item', id: 1 }
  const achievement: Target = { kind: 'achievement', id: 1 }
  const transformation: Target = { kind: 'transformation', id: 1 }

  const pages: WikiPageRef[] = [
    page(item, 'Sad Onion', WikiCategory.Items),
    page(achievement, 'A1', WikiCategory.Achievements),
    page(transformation, 'Guppy', WikiCategory.Transformations),
  ]

  it('answers null without a save', () => {
    expect(overallProgress(pages, () => null)).toBeNull()
  })

  it('sums every category that has a progress of its own', () => {
    const progressFor = (t: Target): PageProgress | null => {
      if (t.kind === 'item') {
        return {
          kind: 'item',
          collected: true,
          unlocked: null,
          unlockedBy: null,
        }
      }
      if (t.kind === 'achievement') return { kind: 'achievement', done: false }
      return null
    }
    expect(overallProgress(pages, progressFor)).toEqual({ done: 1, total: 2 })
  })
})
