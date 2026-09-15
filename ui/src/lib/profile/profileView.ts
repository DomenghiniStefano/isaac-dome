import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import type { Locale } from '@/i18n/locale'
import { assertNever } from '@/lib/assertNever'
import { SavePrefix } from '@/lib/ipc/types'
import type { ActiveProfile, CandidateView, SetupState } from '@/lib/ipc/types'

type Message = MessageKey<MessageSchema>

// The chain the profile screen draws (Schermate.dc.html): each link can be missing alone.
export const ChainLink = {
  Steam: 'steam',
  Game: 'game',
  Saves: 'saves',
} as const
export type ChainLink = (typeof ChainLink)[keyof typeof ChainLink]

export const LinkState = {
  Found: 'found',
  Missing: 'missing',
  YourChoice: 'yourChoice',
  Several: 'several',
  Chosen: 'chosen',
} as const
export type LinkState = (typeof LinkState)[keyof typeof LinkState]

export type LinkDetail =
  | { kind: 'hint'; hint: string }
  | { kind: 'count'; n: number }
  | { kind: 'profile'; candidate: CandidateView }
  | { kind: 'none' }

export interface ChainRow {
  link: ChainLink
  state: LinkState
  detail: LinkDetail
}

const none: LinkDetail = { kind: 'none' }

const savesRow = (setup: SetupState): ChainRow => {
  const active = setup.active
  switch (active.kind) {
    case 'active':
      return {
        link: ChainLink.Saves,
        state: LinkState.Chosen,
        detail: { kind: 'profile', candidate: active.profile },
      }
    case 'needsChoice':
      return {
        link: ChainLink.Saves,
        state: LinkState.Several,
        detail: { kind: 'count', n: setup.candidates.length },
      }
    case 'none':
      return { link: ChainLink.Saves, state: LinkState.Missing, detail: none }
    default:
      return assertNever(active)
  }
}

export const chainLinks = (setup: SetupState): ChainRow[] => [
  setup.steam
    ? {
        link: ChainLink.Steam,
        state: LinkState.Found,
        detail: { kind: 'hint', hint: setup.steam.rootHint },
      }
    : { link: ChainLink.Steam, state: LinkState.Missing, detail: none },
  setup.game
    ? {
        link: ChainLink.Game,
        state: LinkState.Found,
        detail: { kind: 'hint', hint: setup.game.dirHint },
      }
    : {
        link: ChainLink.Game,
        // Without Steam there is no library to look in: the game is for the user to point at.
        state: setup.steam ? LinkState.Missing : LinkState.YourChoice,
        detail: none,
      },
  savesRow(setup),
]

const DayMs = 86_400_000

const startOfDay = (date: Date): number =>
  new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime()

// "oggi", "ieri", "l’altro ieri", "3 giorni fa": calendar days in the machine's time zone,
// in words where the language has them.
export const formatRelativeDay = (
  unix: number | null,
  now: Date,
  locale: Locale,
): string | null => {
  if (unix === null) return null
  const days = Math.round(
    (startOfDay(now) - startOfDay(new Date(unix * 1000))) / DayMs,
  )
  return new Intl.RelativeTimeFormat(locale, { numeric: 'auto' }).format(
    -days,
    'day',
  )
}

export const formatModified = (
  unix: number | null,
  now: Date,
  locale: Locale,
): string | null => {
  const relative = formatRelativeDay(unix, now, locale)
  if (unix === null || relative === null) return null
  const absolute = new Intl.DateTimeFormat(locale, {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  }).format(new Date(unix * 1000))
  return `${relative} · ${absolute}`
}

export const formatCount = (n: number, locale: Locale): string =>
  new Intl.NumberFormat(locale).format(n)

// Game names are data, not messages: they read the same in both languages.
const editionShortName: Record<SavePrefix, string> = {
  [SavePrefix.Rep]: 'Rep',
  [SavePrefix.RepPlus]: 'Rep+',
}
const editionLongName: Record<SavePrefix, string> = {
  [SavePrefix.Rep]: 'Repentance',
  [SavePrefix.RepPlus]: 'Repentance+',
}

export const editionShort = (prefix: SavePrefix): string =>
  editionShortName[prefix]
export const editionLong = (prefix: SavePrefix): string =>
  editionLongName[prefix]

// discovery's Edition and Dlc travel as snake_case strings; an unknown one shows as it came.
const gameNames: Record<string, string> = {
  rebirth: 'Rebirth',
  afterbirth: 'Afterbirth',
  afterbirth_plus: 'Afterbirth+',
  repentance: 'Repentance',
  repentance_plus: 'Repentance+',
}
export const gameName = (value: string): string => gameNames[value] ?? value

export type IndicatorView =
  | { kind: 'active'; edition: string; slot: number; modified: string | null }
  | { kind: 'noProfile' }
  | { kind: 'notFound' }

export const indicator = (
  active: ActiveProfile,
  now: Date,
  locale: Locale,
): IndicatorView => {
  switch (active.kind) {
    case 'active':
      return {
        kind: 'active',
        edition: editionShort(active.profile.prefix),
        slot: active.profile.slot,
        modified: formatRelativeDay(active.profile.modifiedUnix, now, locale),
      }
    case 'needsChoice':
      return { kind: 'noProfile' }
    case 'none':
      return { kind: 'notFound' }
    default:
      return assertNever(active)
  }
}

// The save's sections by their wire names (core_save::Kind, snake_case). The three the game
// names but no measurement has confirmed stay "to identify".
const sectionLabels: Record<string, Message> = {
  achievements: 'profile.sections.achievements',
  counters: 'profile.sections.counters',
  level_counters: 'profile.sections.levelCounters',
  items: 'profile.sections.items',
  unknown5: 'profile.sections.unknown',
  bosses: 'profile.sections.bosses',
  challenges: 'profile.sections.challenges',
  cutscene_counters: 'profile.sections.unknown',
  unknown9: 'profile.sections.unknown',
  bestiary: 'profile.sections.bestiary',
}
export const sectionLabel = (kind: string): Message | null =>
  sectionLabels[kind] ?? null
