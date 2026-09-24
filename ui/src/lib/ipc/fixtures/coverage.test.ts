import { describe, expect, it } from 'vitest'
import { Command } from '../../constants/commands'
import type { CommandName } from '../transport'
import { hasFixture } from './index'

// Every wrapper under `lib/ipc/`, as source text. Read rather than listed, so a command added to
// a wrapper is held to a fixture the day it is written.
const wrappers = import.meta.glob<string>(
  ['../*.ts', '!../*.test.ts', '!../types.ts'],
  {
    query: '?raw',
    import: 'default',
    eager: true,
  },
)

const sent = (): CommandName[] => {
  const names = new Set<CommandName>()
  for (const source of Object.values(wrappers)) {
    for (const [, key] of source.matchAll(/\bCommand\.(\w+)\b/g)) {
      const value = (Command as Record<string, CommandName>)[key ?? '']
      if (value) names.add(value)
    }
  }
  return [...names]
}

// Card #80, item 10: `pnpm ui:dev` answered only what the screens read on the day each fixture
// was written, so Roll and the auto-update switch threw "no fixture answers" in the browser.
describe('the development fixtures', () => {
  it('read the wrappers they are held to', () => {
    expect(sent().length).toBeGreaterThan(20)
  })

  it('answer every command a wrapper can send', () => {
    expect(sent().filter((c) => !hasFixture(c))).toEqual([])
  })
})
