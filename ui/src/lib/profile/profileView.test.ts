import { describe, expect, it } from 'vitest'
import { Locale } from '@/i18n/locale'
import { candidates, noneSetup, setupWith } from '@/lib/ipc/fixtures/profile'
import { MissingReason, SavePrefix } from '@/lib/ipc/types'
import type { CandidateView, SetupState } from '@/lib/ipc/types'
import {
  LinkState,
  chainLinks,
  editionShort,
  formatCount,
  formatModified,
  indicator,
  sectionLabel,
} from './profileView'

const now = new Date(2026, 8, 2, 12)
const unixAt = (day: Date) => Math.floor(day.getTime() / 1000)
const states = (setup: SetupState) => chainLinks(setup).map((r) => r.state)
const first = candidates[0] as CandidateView

describe('chainLinks', () => {
  it('breaks at Steam: the game is your choice, the saves are missing', () => {
    expect(states(noneSetup)).toEqual([
      LinkState.Missing,
      LinkState.YourChoice,
      LinkState.Missing,
    ])
  })

  it('breaks at the game when Steam was found', () => {
    const setup: SetupState = {
      ...noneSetup,
      steam: { rootHint: 'C:\\Steam', libraries: 1 },
      active: { kind: 'none', reason: MissingReason.GameNotFound },
    }
    expect(states(setup)).toEqual([
      LinkState.Found,
      LinkState.Missing,
      LinkState.Missing,
    ])
  })

  it('breaks at the saves when Steam and the game were found', () => {
    const setup = {
      ...setupWith({ kind: 'none', reason: MissingReason.NoSaves }),
      candidates: [],
    }
    expect(states(setup)).toEqual([
      LinkState.Found,
      LinkState.Found,
      LinkState.Missing,
    ])
  })

  it('has several saves to choose from', () => {
    const setup = setupWith({
      kind: 'needsChoice',
      reason: { kind: 'neverChosen' },
      suggested: null,
    })
    expect(chainLinks(setup)[2]).toEqual({
      link: 'saves',
      state: LinkState.Several,
      detail: { kind: 'count', n: 4 },
    })
  })

  it('has one chosen save once active', () => {
    const setup = setupWith({
      kind: 'active',
      profile: first,
      autoSelected: false,
    })
    expect(chainLinks(setup)[2]).toEqual({
      link: 'saves',
      state: LinkState.Chosen,
      detail: { kind: 'profile', candidate: first },
    })
  })
})

describe('formatModified', () => {
  it('says today, yesterday and the day before in words', () => {
    expect(
      formatModified(unixAt(new Date(2026, 8, 2, 8)), now, Locale.It),
    ).toBe('oggi · 2 set 2026')
    expect(
      formatModified(unixAt(new Date(2026, 8, 1, 8)), now, Locale.It),
    ).toBe('ieri · 1 set 2026')
    expect(
      formatModified(unixAt(new Date(2026, 7, 31, 10)), now, Locale.It),
    ).toBe('l’altro ieri · 31 ago 2026')
  })

  it('counts days beyond that', () => {
    expect(
      formatModified(unixAt(new Date(2026, 7, 30, 10)), now, Locale.It),
    ).toBe('3 giorni fa · 30 ago 2026')
  })

  it('speaks English too', () => {
    expect(
      formatModified(unixAt(new Date(2026, 7, 31, 10)), now, Locale.En),
    ).toBe('2 days ago · Aug 31, 2026')
  })

  it('has nothing to say without a date', () => {
    expect(formatModified(null, now, Locale.It)).toBeNull()
  })
})

describe('formatCount', () => {
  it('groups digits as each language does', () => {
    expect(formatCount(14948, Locale.It)).toBe('14.948')
    expect(formatCount(4068, Locale.It)).toBe('4068')
    expect(formatCount(14948, Locale.En)).toBe('14,948')
    expect(formatCount(4068, Locale.En)).toBe('4,068')
  })
})

describe('editionShort', () => {
  it('names both prefixes', () => {
    expect(editionShort(SavePrefix.Rep)).toBe('Rep')
    expect(editionShort(SavePrefix.RepPlus)).toBe('Rep+')
  })
})

describe('indicator', () => {
  it('names the active profile by edition, slot and age', () => {
    const profile = {
      ...first,
      modifiedUnix: unixAt(new Date(2026, 7, 31, 10)),
    }
    expect(
      indicator(
        { kind: 'active', profile, autoSelected: false },
        now,
        Locale.It,
      ),
    ).toEqual({
      kind: 'active',
      edition: 'Rep+',
      slot: 1,
      modified: 'l’altro ieri',
    })
  })

  it('has no profile while a choice is pending', () => {
    expect(
      indicator(
        {
          kind: 'needsChoice',
          reason: { kind: 'neverChosen' },
          suggested: null,
        },
        now,
        Locale.It,
      ),
    ).toEqual({ kind: 'noProfile' })
  })

  it('has found nothing when there are no saves', () => {
    expect(indicator(noneSetup.active, now, Locale.It)).toEqual({
      kind: 'notFound',
    })
  })
})

describe('sectionLabel', () => {
  it('has a message for a known section and none for an unknown one', () => {
    expect(sectionLabel('achievements')).toBe('profile.sections.achievements')
    expect(sectionLabel('unknown5')).toBe('profile.sections.unknown')
    expect(sectionLabel('something_new')).toBeNull()
  })
})
