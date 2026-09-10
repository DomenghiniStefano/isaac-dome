import { beforeEach, describe, expect, it } from 'vitest'
import { Command } from '../constants/commands'
import { FixtureScenario, answer, resetFixtures } from './fixtures'
import { call } from './transport'
import type { SetupState } from './types'

beforeEach(() => resetFixtures())

describe('call, outside Tauri in development', () => {
  it('answers a known command from the fixtures', async () => {
    const setup = await call<SetupState>(Command.SetupState)
    expect(setup.candidates.length).toBeGreaterThan(0)
  })

  it('rejects a command no fixture answers instead of resolving undefined', async () => {
    await expect(call(Command.Unlock)).rejects.toThrow(Command.Unlock)
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
