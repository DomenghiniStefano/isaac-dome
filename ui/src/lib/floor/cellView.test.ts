import { describe, expect, it } from 'vitest'
import { TargetView } from '@/lib/ipc/types'
import type { FloorSolutionView } from '@/lib/ipc/types'
import { START } from './painting'
import {
  RankStep,
  TARGET_ORDER,
  candidateFor,
  cellPosition,
  rankStep,
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

describe('TARGET_ORDER', () => {
  it('holds the three targets once each: the switch is drawn from it', () => {
    expect([...TARGET_ORDER].sort()).toEqual(
      [...Object.values(TargetView)].sort(),
    )
  })

  it('opens on the one a player looks for on every floor', () => {
    expect(TARGET_ORDER[0]).toBe(TargetView.Secret)
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

describe('candidateFor', () => {
  const solutions = [
    solution(TargetView.UltraSecret, [[57, 0]]),
    solution(TargetView.Secret, [
      [57, 1],
      [58, 0],
    ]),
    solution(TargetView.SuperSecret, [[57, 2]]),
  ]

  it('says nothing about a cell no rule lit', () => {
    expect(candidateFor(9, solutions, TargetView.Secret)).toBeNull()
  })

  it('answers the target being shown and never another one', () => {
    // Cell 57 is a candidate for all three at three different ranks, which is exactly the
    // case the old design tried to draw at once and could not.
    expect(candidateFor(57, solutions, TargetView.Secret)?.rank).toBe(2)
    expect(candidateFor(57, solutions, TargetView.SuperSecret)?.rank).toBe(3)
    expect(candidateFor(57, solutions, TargetView.UltraSecret)?.rank).toBe(1)
  })

  it('says nothing when the target being shown has no answer at all', () => {
    const only = [solution(TargetView.Secret, [[58, 0]])]
    expect(candidateFor(58, only, TargetView.UltraSecret)).toBeNull()
  })

  it('fills the cell for the best place the rules allow', () => {
    expect(candidateFor(58, solutions, TargetView.Secret)?.step).toBe(
      RankStep.First,
    )
  })

  it('counts the place from one, the way the legend reads it', () => {
    expect(candidateFor(58, solutions, TargetView.Secret)?.rank).toBe(1)
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
