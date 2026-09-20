import { describe, expect, it } from 'vitest'
import { TargetView } from '@/lib/ipc/types'
import { RankStep } from './cellView'
import { bandFill } from './bands'

const targets = Object.values(TargetView)
const steps = Object.values(RankStep)

const classes = (target: TargetView, step: RankStep): string[] =>
  bandFill[target][step].split(' ')

const fill = (target: TargetView, step: RankStep): string =>
  classes(target, step).find((c) => c.startsWith('bg-')) ?? ''

const ink = (target: TargetView, step: RankStep): string =>
  classes(target, step).find((c) => c.startsWith('text-')) ?? ''

/** `secret` out of `bg-floor-band-secret-second`: the hue the three steps walk down. */
const hue = (target: TargetView, step: RankStep): string =>
  fill(target, step)
    .replace(/^bg-floor-band-/, '')
    .split('-')[0]

describe('bandFill', () => {
  it('has a fill for every target at every step: a rank with no colour is an invisible answer', () => {
    for (const target of targets) {
      for (const step of steps) expect(fill(target, step)).toBeTruthy()
    }
  })

  it('never repeats a fill, so colour alone still tells the nine apart', () => {
    const fills = targets.flatMap((target) =>
      steps.map((step) => fill(target, step)),
    )
    expect(new Set(fills).size).toBe(fills.length)
  })

  it('keeps one hue per target, because the three steps are one ramp and not three answers', () => {
    for (const target of targets) {
      expect(new Set(steps.map((step) => hue(target, step))).size).toBe(1)
    }
  })

  it('gives the three targets three different hues', () => {
    // The defect the owner reported on 2026-09-20: Secret was blue and Super was teal, and on
    // the grid they read as the same colour. Nothing but a hue tells two bands apart now that
    // the corner is gone, so two targets sharing one is the whole answer being wrong.
    for (const step of steps) {
      const hues = targets.map((target) => hue(target, step))
      expect(new Set(hues).size).toBe(hues.length)
    }
  })

  it('carries an ink beside every fill, never a fill on its own', () => {
    for (const target of targets) {
      for (const step of steps) expect(ink(target, step)).toBeTruthy()
    }
  })

  it('does not use one ink for all three steps of a hue', () => {
    // A band prints its rank. The first step is the brightest of its ramp and the third the
    // darkest, so one foreground across the three is a digit that disappears at one end.
    for (const target of targets) {
      expect(
        new Set(steps.map((step) => ink(target, step))).size,
      ).toBeGreaterThan(1)
    }
  })
})
