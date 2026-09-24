import { describe, expect, it } from 'vitest'
import { RestoreStep, restoreStep } from './restore'

// What a region does about the position it was left at, given how tall it is right now.
describe('restoring a position', () => {
  it('does nothing when there is no position to go back to', () => {
    expect(
      restoreStep(undefined, { scrollHeight: 900, clientHeight: 300 }),
    ).toEqual({
      kind: RestoreStep.Done,
    })
    expect(restoreStep(0, { scrollHeight: 900, clientHeight: 300 })).toEqual({
      kind: RestoreStep.Done,
    })
  })

  // The data arrives after the region is mounted. Set too early, the browser clamps the position
  // to the little there is, and the page is left somewhere it never was.
  it('waits while the content is too short to reach the position', () => {
    expect(restoreStep(700, { scrollHeight: 600, clientHeight: 300 })).toEqual({
      kind: RestoreStep.Wait,
    })
  })

  it('goes to the position once the content can reach it', () => {
    expect(restoreStep(700, { scrollHeight: 1400, clientHeight: 300 })).toEqual(
      {
        kind: RestoreStep.Go,
        top: 700,
      },
    )
    // Exactly the bottom is reachable.
    expect(
      restoreStep(700, { scrollHeight: 1000, clientHeight: 300 }).kind,
    ).toBe(RestoreStep.Go)
  })
})
