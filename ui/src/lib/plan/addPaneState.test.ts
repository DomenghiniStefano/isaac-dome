import { describe, expect, it } from 'vitest'
import { StepsBasis } from '@/lib/ipc/types'
import type { StepsSection } from '@/lib/ipc/types'
import { AddPaneState, addPaneState } from './addPaneState'

const full: StepsSection[] = [{ basis: StepsBasis.FanOut, steps: [] }]

describe('what the left pane is showing', () => {
  it('answers the want, whatever the sections hold', () => {
    expect(addPaneState(true, full, false)).toBe(AddPaneState.Want)
    expect(addPaneState(true, [], false)).toBe(AddPaneState.Want)
  })

  it('suggests when nothing was asked and there is something to suggest', () => {
    expect(addPaneState(false, full, false)).toBe(AddPaneState.Sections)
  })

  // The case the component must not reach by counting: no want and no sections is the only
  // page that says "there is nothing to unlock right now".
  it('says there is nothing only when nothing was asked either', () => {
    expect(addPaneState(false, [], false)).toBe(AddPaneState.Nothing)
  })

  // An empty page has two reasons and the sections do not say which (DESIGN-BRIEF §7.3):
  // with no game installed the graph knows nothing, and "there is nothing to unlock" would
  // be a wrong answer rather than a short one.
  it('says the game is missing rather than that there is nothing', () => {
    expect(addPaneState(false, [], true)).toBe(AddPaneState.NoCatalog)
  })

  // A want still wins: you asked a question, and the answer names what it cannot resolve.
  it('still answers the want with no catalog', () => {
    expect(addPaneState(true, [], true)).toBe(AddPaneState.Want)
  })
})
