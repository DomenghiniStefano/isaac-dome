import { describe, expect, it } from 'vitest'
import { TargetView } from '@/lib/ipc/types'
import { Corner, RankStep } from './cellView'
import { cornerAt, pipFill } from './pips'

const targets = Object.values(TargetView)
const steps = Object.values(RankStep)

describe('pipFill', () => {
  it('has a fill for every target at every step: a rank with no colour is an invisible answer', () => {
    for (const target of targets) {
      for (const step of steps) expect(pipFill[target][step]).toBeTruthy()
    }
  })

  it('never repeats a fill, so colour alone still tells the nine apart', () => {
    const fills = targets.flatMap((target) =>
      steps.map((step) => pipFill[target][step]),
    )
    expect(new Set(fills).size).toBe(fills.length)
  })

  it('keeps one hue per target, because the colour is the corner said twice', () => {
    for (const target of targets) {
      const hues = steps.map((step) => pipFill[target][step].split('-').at(-2))
      expect(new Set(hues).size).toBe(1)
    }
  })
})

describe('cornerAt', () => {
  it('places every corner a target can own', () => {
    for (const corner of Object.values(Corner))
      expect(cornerAt[corner]).toBeTruthy()
  })

  it('never puts two of them in the same place', () => {
    const places = Object.values(Corner).map((corner) => cornerAt[corner])
    expect(new Set(places).size).toBe(places.length)
  })
})
