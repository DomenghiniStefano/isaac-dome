import { describe, expect, it } from 'vitest'
import rust from '../../../../crates/ipc/src/settings.rs?raw'
import {
  DefaultScale,
  nextPercent,
  previousPercent,
  scaleFactor,
  scalePercents,
  snapPercent,
  spriteMultiple,
} from './steps'

describe('the ladder', () => {
  it('is the eleven steps, in order', () => {
    expect(scalePercents).toEqual([
      50, 67, 75, 80, 90, 100, 110, 125, 150, 175, 200,
    ])
    expect(scalePercents).toContain(DefaultScale)
  })

  // Two ladders that drift are a slider that saves a value the backend refuses: the numbers
  // are read from the Rust source itself, the way the row height is read from the CSS.
  it('is the same ladder the backend has', () => {
    const declared = /SCALE_PERCENTS: \[u16; \d+\] = \[([^\]]*)\]/.exec(
      rust,
    )?.[1]
    expect(declared).toBeDefined()
    const numbers = (declared ?? '')
      .split(',')
      .map((part) => Number(part.trim()))
      .filter((n) => !Number.isNaN(n))
    expect(numbers).toEqual(scalePercents)
    expect(rust).toMatch(new RegExp(`DEFAULT_SCALE: u16 = ${DefaultScale};`))
  })
})

describe('snapPercent', () => {
  it('keeps a step and reads anything else as the default', () => {
    for (const step of scalePercents) expect(snapPercent(step)).toBe(step)
    // Not the nearest step: 137 is a file we didn't write, not "about 125".
    for (const off of [0, 1, 49, 51, 137, 199, 201, Number.NaN])
      expect(snapPercent(off)).toBe(DefaultScale)
  })
})

describe('walking the ladder', () => {
  it('stops at both ends', () => {
    expect(nextPercent(200)).toBe(200)
    expect(previousPercent(50)).toBe(50)
    expect(nextPercent(100)).toBe(110)
    expect(previousPercent(100)).toBe(90)
  })

  it('starts from the default when it is handed a value it never wrote', () => {
    expect(nextPercent(137)).toBe(110)
    expect(previousPercent(137)).toBe(90)
  })
})

describe('scaleFactor', () => {
  it('is the percentage as a factor', () => {
    expect(scaleFactor(100)).toBe(1)
    expect(scaleFactor(125)).toBe(1.25)
    expect(scaleFactor(67)).toBe(0.67)
  })
})

describe('spriteMultiple', () => {
  // A 32px sprite is drawn at a whole multiple of itself, never at a fraction: the game's
  // art is pixel art. The rule is round(2 x factor), floored at one.
  it('is a whole multiple at every step', () => {
    expect(scalePercents.map(spriteMultiple)).toEqual([
      1, 1, 2, 2, 2, 2, 2, 3, 3, 4, 4,
    ])
  })
})
