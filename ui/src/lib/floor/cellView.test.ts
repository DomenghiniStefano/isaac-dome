import { describe, expect, it } from 'vitest'
import { TargetView } from '@/lib/ipc/types'
import type { FloorSolutionView } from '@/lib/ipc/types'
import { START } from './painting'
import {
  Corner,
  RankStep,
  cellPosition,
  cornerOf,
  pipsFor,
  rankStep,
  toggled,
} from './cellView'

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

describe('cornerOf', () => {
  it('gives each target a corner of its own', () => {
    expect(cornerOf[TargetView.Secret]).toBe(Corner.TopLeft)
    expect(cornerOf[TargetView.SuperSecret]).toBe(Corner.TopRight)
    expect(cornerOf[TargetView.UltraSecret]).toBe(Corner.BottomLeft)
  })

  it('never puts two targets in the same corner, which is what makes the corner the answer', () => {
    const corners = all.map((target) => cornerOf[target])
    expect(new Set(corners).size).toBe(corners.length)
  })
})

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

describe('pipsFor', () => {
  const solutions = [
    solution(TargetView.UltraSecret, [[57, 0]]),
    solution(TargetView.Secret, [
      [57, 1],
      [58, 0],
    ]),
    solution(TargetView.SuperSecret, [[57, 2]]),
  ]

  it('says nothing about a cell no rule lit', () => {
    expect(pipsFor(9, solutions, all)).toEqual([])
  })

  it('carries one pip per target that lit the cell', () => {
    expect(pipsFor(57, solutions, all)).toHaveLength(3)
  })

  it('orders them by corner and not by the order the answers arrived in', () => {
    // The solutions above are deliberately out of order: ultra, secret, super.
    const targets = pipsFor(57, solutions, all).map((pip) => pip.target)
    expect(targets).toEqual([
      TargetView.Secret,
      TargetView.SuperSecret,
      TargetView.UltraSecret,
    ])
  })

  it('prints the rank one-based, the way the legend reads it', () => {
    const [secret] = pipsFor(58, solutions, all)
    expect(secret.rank).toBe(1)
    expect(secret.step).toBe(RankStep.First)
  })

  it('drops a target that is switched off, which is the whole point of the filter', () => {
    const shown = [TargetView.Secret, TargetView.UltraSecret]
    const targets = pipsFor(57, solutions, shown).map((pip) => pip.target)
    expect(targets).toEqual([TargetView.Secret, TargetView.UltraSecret])
  })

  it('says nothing at all when every target is switched off', () => {
    expect(pipsFor(57, solutions, [])).toEqual([])
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
