import { Style } from '../types'
import type { ChallengeRow, ChallengesView, Inline } from '../types'

// Development only. The challenges are hand-written until the design pack carries a
// `challenges.json` (it needs a machine with the game and a save), and the point of the
// fixture is the one thing a real profile cannot show all at once: **every state on screen
// together**, including the two a screen gets wrong — a blocked row that has to name its
// gates, and a row the wiki knows nothing about.
let warned = false
const declare = () => {
  if (warned) return
  warned = true
  console.warn(
    'Challenges fixture: names, goals and gates are hand-written until the design pack carries challenges.json (pnpm design:export)',
  )
}

const row = (
  number: number,
  name: string,
  over: Partial<ChallengeRow> = {},
): ChallengeRow => ({
  number,
  name,
  state: { kind: 'available' },
  rewards: [],
  character: null,
  goal: null,
  blindfolded: null,
  page: { kind: 'challenge', number },
  ...over,
})

const text = (s: string): Inline[] => [
  { kind: 'text', text: s, style: Style.Plain },
]

const rows: ChallengeRow[] = [
  row(1, 'Pitch Black', {
    state: { kind: 'done' },
    rewards: [
      {
        achievement: 122,
        text: 'You unlocked "The Candle"',
        iconUrl: null,
        page: { kind: 'achievement', id: 122 },
        done: true,
      },
    ],
    character: { kind: 'character', id: 0 },
    goal: text('Mom'),
    blindfolded: false,
  }),
  row(6, 'Solar System', {
    character: { kind: 'character', id: 3 },
    goal: text('Mom'),
    blindfolded: false,
    rewards: [
      {
        achievement: 130,
        text: 'You unlocked "Distant Admiration"',
        iconUrl: null,
        page: { kind: 'achievement', id: 130 },
        done: false,
      },
    ],
  }),
  row(18, 'The Host', {
    state: { kind: 'blocked', missing: [61, 62] },
    character: { kind: 'character', id: 5 },
    goal: text('Mom’s Heart'),
    blindfolded: true,
    rewards: [
      {
        achievement: 145,
        text: 'You unlocked "Speed Ball"',
        iconUrl: null,
        page: { kind: 'achievement', id: 145 },
        done: false,
      },
    ],
  }),
  // The row the wiki knows nothing about: no character, no goal, no blindfolded — and it must
  // not draw as "any character, no goal, not blindfolded".
  row(45, 'Red Redemption', { page: null }),
]

export const challengesAnswer = (): ChallengesView => {
  declare()
  return {
    challenges: rows,
    totals: { slots: 46, challenges: rows.length, done: 1 },
    diagnostics: [],
  }
}

// Section 7 unread: every row is unknown, and nothing reads as "you have done none of them".
export const challengesUnread = (): ChallengesView => {
  declare()
  return {
    challenges: rows.map((r) => ({
      ...r,
      state: { kind: 'unknown' as const },
    })),
    totals: { slots: 0, challenges: rows.length, done: 0 },
    diagnostics: [{ kind: 'noChallengesSection' }],
  }
}
