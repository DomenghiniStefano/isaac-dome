import { describe, expect, it } from 'vitest'
import { TargetView } from '@/lib/ipc/types'
import type { FloorSolutionView } from '@/lib/ipc/types'
import { START } from './painting'
import { RankStep, bandsFor, cellPosition, rankStep, toggled } from './cellView'

const solution = (
  target: TargetView,
  cells: readonly [number, number][],
): FloorSolutionView => ({
  target,
  candidates: cells.map(([cell, rank]) => ({
    cell,
    neighbours: 1,
    rank,
    applied: [],
  })),
  unresolved: [],
})

const all = [TargetView.Secret, TargetView.SuperSecret, TargetView.UltraSecret]

describe('rankStep', () => {
  it('walks the first three ranks down its own step', () => {
    expect(rankStep(0)).toBe(RankStep.First)
    expect(rankStep(1)).toBe(RankStep.Second)
    expect(rankStep(2)).toBe(RankStep.Third)
  })

  it('stops counting past the third, because no rule here claims how much less likely', () => {
    expect(rankStep(3)).toBe(RankStep.Third)
    expect(rankStep(40)).toBe(RankStep.Third)
  })
})

describe('bandsFor', () => {
  const solutions = [
    solution(TargetView.UltraSecret, [[57, 0]]),
    solution(TargetView.Secret, [
      [57, 1],
      [58, 0],
    ]),
    solution(TargetView.SuperSecret, [[57, 2]]),
  ]

  it('says nothing about a cell no rule lit', () => {
    expect(bandsFor(9, solutions, all)).toEqual([])
  })

  it('carries one band per target that lit the cell', () => {
    expect(bandsFor(57, solutions, all)).toHaveLength(3)
  })

  it('gives a cell one target lit a single band, which is the cell filled edge to edge', () => {
    // The whole reason the corner went: a cell only the Secret Room can be in is a Secret
    // Room's colour, not a small square in the top-left of an otherwise empty square.
    const bands = bandsFor(58, solutions, all)
    expect(bands).toHaveLength(1)
    expect(bands[0].target).toBe(TargetView.Secret)
  })

  it('orders them by the reading order and not by the order the answers arrived in', () => {
    // The solutions above are deliberately out of order: ultra, secret, super.
    const targets = bandsFor(57, solutions, all).map((band) => band.target)
    expect(targets).toEqual([
      TargetView.Secret,
      TargetView.SuperSecret,
      TargetView.UltraSecret,
    ])
  })

  it('prints the rank one-based, the way the legend reads it', () => {
    const [secret] = bandsFor(58, solutions, all)
    expect(secret.rank).toBe(1)
    expect(secret.step).toBe(RankStep.First)
  })

  it('drops a target that is switched off, which is the whole point of the filter', () => {
    const shown = [TargetView.Secret, TargetView.UltraSecret]
    const targets = bandsFor(57, solutions, shown).map((band) => band.target)
    expect(targets).toEqual([TargetView.Secret, TargetView.UltraSecret])
  })

  it('says nothing at all when every target is switched off', () => {
    expect(bandsFor(57, solutions, [])).toEqual([])
  })
})

describe('cellPosition', () => {
  it('counts from one, because nobody reads a grid from zero', () => {
    expect(cellPosition(0)).toEqual({ row: 1, column: 1 })
  })

  it('puts the start room in the middle of a 13 by 13', () => {
    expect(cellPosition(START)).toEqual({ row: 7, column: 7 })
  })

  it('reaches the far corner', () => {
    expect(cellPosition(168)).toEqual({ row: 13, column: 13 })
  })

  it('walks along a row before dropping to the next', () => {
    expect(cellPosition(12)).toEqual({ row: 1, column: 13 })
    expect(cellPosition(13)).toEqual({ row: 2, column: 1 })
  })
})

describe('toggled', () => {
  it('takes a target out', () => {
    expect(toggled(all, TargetView.SuperSecret)).toEqual([
      TargetView.Secret,
      TargetView.UltraSecret,
    ])
  })

  it('puts one back where it belongs, not at the end', () => {
    // The filters are drawn from this list. A target that came back at the end would move
    // the row under the hand that is using it.
    const without = [TargetView.Secret, TargetView.UltraSecret]
    expect(toggled(without, TargetView.SuperSecret)).toEqual(all)
  })

  it('answers a new list, never the one it was given', () => {
    const before = [...all]
    expect(toggled(all, TargetView.Secret)).not.toBe(all)
    expect(all).toEqual(before)
  })

  it('can end up showing nothing, which is a filter and not a broken state', () => {
    expect(
      all.reduce<TargetView[]>((shown, target) => toggled(shown, target), all),
    ).toEqual([])
  })
})
