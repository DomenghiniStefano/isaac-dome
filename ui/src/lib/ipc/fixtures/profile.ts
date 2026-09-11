import { CandidateSource, MissingReason, SavePrefix } from '../types'
import type {
  ActiveProfile,
  CandidateView,
  SaveSummary,
  SetupState,
} from '../types'

// DESIGN-BRIEF.md §5.4, the reference machine: Steam on C:, the game on a second library,
// four candidates whose sizes tell a played profile from a new one. Hints, not paths.
const unix = (year: number, month: number, day: number): number =>
  Math.floor(new Date(year, month - 1, day, 18).getTime() / 1000)

const remote = 'Steam\\userdata\\…\\250900\\remote'

export const candidates: CandidateView[] = [
  {
    id: 'rep_plus-1',
    prefix: SavePrefix.RepPlus,
    slot: 1,
    source: CandidateSource.SteamCloud,
    modifiedUnix: unix(2026, 9, 9),
    sizeBytes: 14948,
    suggested: true,
    pathHint: remote,
  },
  {
    id: 'rep_plus-2',
    prefix: SavePrefix.RepPlus,
    slot: 2,
    source: CandidateSource.SteamCloud,
    modifiedUnix: unix(2026, 7, 12),
    sizeBytes: 4068,
    suggested: false,
    pathHint: remote,
  },
  {
    id: 'rep-1',
    prefix: SavePrefix.Rep,
    slot: 1,
    source: CandidateSource.SteamCloud,
    modifiedUnix: unix(2025, 6, 20),
    sizeBytes: 14172,
    suggested: false,
    pathHint: remote,
  },
  {
    id: 'rep-2',
    prefix: SavePrefix.Rep,
    slot: 2,
    source: CandidateSource.SteamCloud,
    modifiedUnix: unix(2025, 3, 2),
    sizeBytes: 3956,
    suggested: false,
    pathHint: remote,
  },
]

export const setupWith = (active: ActiveProfile): SetupState => ({
  steam: { rootHint: 'C:\\Program Files (x86)\\Steam', libraries: 2 },
  game: {
    dirHint:
      'D:\\SteamLibrary\\steamapps\\common\\The Binding of Isaac Rebirth',
    edition: 'repentance_plus',
    dlcs: ['afterbirth', 'afterbirth_plus', 'repentance', 'repentance_plus'],
  },
  candidates,
  active,
  diagnostics: [],
})

export const noneSetup: SetupState = {
  steam: null,
  game: null,
  candidates: [],
  active: { kind: 'none', reason: MissingReason.SteamNotFound },
  diagnostics: [{ kind: 'steamNotFound' }],
}

// The section counts §5.4 read from the reference profile.
export const summary: SaveSummary = {
  profile: 'rep_plus-1',
  sections: [
    { kind: 'achievements', count: 642 },
    { kind: 'counters', count: 523 },
    { kind: 'level_counters', count: 14 },
    { kind: 'items', count: 733 },
    { kind: 'unknown5', count: 7 },
    { kind: 'bosses', count: 104 },
    { kind: 'challenges', count: 46 },
    { kind: 'unknown8', count: 27 },
    { kind: 'unknown9', count: 2 },
    { kind: 'bestiary', count: 80 },
  ],
  diagnostics: [],
}
