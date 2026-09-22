import { describe, expect, it } from 'vitest'
import containers from '@/assets/theme/containers.css?raw'
import { Threshold, ThresholdPx } from './thresholds'

// The pattern of `lib/scale/rows.test.ts`: a number lives in the CSS, where the layout reads it,
// and in TypeScript, where anything that has to name it reads it. If the two drift, a threshold
// moves in one place while the other goes on describing the old layout — and nothing else says so.
describe('the page thresholds', () => {
  it('are declared in containers.css at the px the constants say', () => {
    for (const name of Object.values(Threshold))
      expect(containers).toMatch(
        new RegExp(String.raw`--container-${name}:\s*${ThresholdPx[name]}px;`),
      )
  })

  it('ascend: compact is narrower than regular, regular than wide', () => {
    expect(ThresholdPx[Threshold.Compact]).toBeLessThan(
      ThresholdPx[Threshold.Regular],
    )
    expect(ThresholdPx[Threshold.Regular]).toBeLessThan(
      ThresholdPx[Threshold.Wide],
    )
  })

  it('leave the window floor inside compact', () => {
    // 640 window (spec §8) − 168 sidebar at its minimum: the page box pads nothing since card
    // #63, so that is its whole width. If compact ever drops below that, the narrowest window the
    // app allows draws a layout that was never designed for it.
    expect(ThresholdPx[Threshold.Compact]).toBeGreaterThan(640 - 168)
  })
})
