import { describe, expect, it } from 'vitest'
import { TargetView } from '@/lib/ipc/types'
import { RankStep } from './cellView'
import { levelHeight, targetFill } from './targets'

const targets = Object.values(TargetView)
const steps = [RankStep.First, RankStep.Second, RankStep.Third]

describe('targetFill', () => {
  it('has a colour for every target: a target with none is an invisible answer', () => {
    for (const target of targets) expect(targetFill[target]).toBeTruthy()
  })

  it('gives the three targets three different colours', () => {
    // The defect the owner reported on 2026-09-20: Secret was blue and Super was teal, and on
    // the grid they read as the same colour. The hue is what tells you which of the three you
    // are looking at without going back to the switch, so two sharing one is the answer
    // being wrong.
    const fills = targets.map((target) => targetFill[target])
    expect(new Set(fills).size).toBe(fills.length)
  })
})

describe('levelHeight', () => {
  it('has a height for every step: a rank with none is a cell that fills itself', () => {
    for (const step of steps) expect(levelHeight[step]).toBeTruthy()
  })

  it('fills the cell to the brim for the best place the rules allow', () => {
    expect(levelHeight[RankStep.First]).toBe('h-floor-level-first')
  })

  it('never gives two steps the same height, which is the only thing telling them apart', () => {
    const heights = steps.map((step) => levelHeight[step])
    expect(new Set(heights).size).toBe(heights.length)
  })
})
