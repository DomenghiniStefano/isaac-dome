import { beforeEach, describe, expect, it } from 'vitest'
import { Command } from '../constants/commands'
import { FixtureScenario, answer, resetFixtures } from './fixtures'
import { call } from './transport'
import type { MarksMatrix, SetupState } from './types'

beforeEach(() => resetFixtures())

describe('call, outside Tauri in development', () => {
  it('answers a known command from the fixtures', async () => {
    const setup = await call<SetupState>(Command.SetupState)
    expect(setup.candidates.length).toBeGreaterThan(0)
  })

  it('rejects a command no fixture answers instead of resolving undefined', async () => {
    // Plan has no fixture until sub-project 3.3b; unlock gained one with 3.3a.
    await expect(call(Command.Plan)).rejects.toThrow(Command.Plan)
  })
})

describe('fixture scenarios', () => {
  it('finds no saves in the none scenario', async () => {
    const setup = await answer<SetupState>(
      Command.SetupState,
      undefined,
      FixtureScenario.None,
    )
    expect(setup.active.kind).toBe('none')
    expect(setup.candidates).toEqual([])
  })

  it('asks for a choice until one is made, then is active on it', async () => {
    const before = await answer<SetupState>(
      Command.SetupState,
      undefined,
      FixtureScenario.Pick,
    )
    expect(before.active.kind).toBe('needsChoice')
    const after = await answer<SetupState>(
      Command.SelectProfile,
      { id: 'rep-2' },
      FixtureScenario.Pick,
    )
    expect(after.active).toMatchObject({
      kind: 'active',
      profile: { id: 'rep-2' },
    })
  })

  it('refuses a summary without an active profile, as the backend does', async () => {
    await expect(
      answer(Command.SaveSummary, undefined, FixtureScenario.None),
    ).rejects.toEqual({ kind: 'noActiveProfile' })
  })
})

describe('the completion fixture', () => {
  it('answers the reference matrix with an active profile', async () => {
    const m = await answer<MarksMatrix>(
      Command.Completion,
      undefined,
      FixtureScenario.Active,
    )
    expect(m.characters).toHaveLength(34)
    expect(m.art).toHaveLength(m.bosses.length)
    // DESIGN-BRIEF.md §5.4: 166 cells with a level out of 368 readable, 40 unknown, 0
    // suspect. 152 of the 166 reached the second level — counted on the fixture's own digit
    // strings, not read back out of this code.
    expect(m.totals).toEqual({
      cells: 408,
      readable: 368,
      unknown: 40,
      unexpected: 0,
      normal: 166,
      hard: 152,
    })
  })

  it('refuses the matrix without an active profile, as the backend does', async () => {
    await expect(
      answer(Command.Completion, undefined, FixtureScenario.None),
    ).rejects.toEqual({ kind: 'noActiveProfile' })
  })
})
