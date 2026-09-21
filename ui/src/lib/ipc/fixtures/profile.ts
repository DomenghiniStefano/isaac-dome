import { CandidateSource, MissingReason, SavePrefix } from '../types'
import type {
  ActiveProfile,
  CandidateView,
  SaveSummary,
  SetupState,
} from '../types'

// DESIGN-BRIEF.md §5.4, the reference machine: Steam on C:, the game on a second library,
// four candidates whose sizes tell a played profile from a new one. Hints, not paths.
//
// The four previews cover the four shapes a welcome card can take: whole, with a count the
// file did not let us read, with unreadable cells, and a file that could not be parsed at
// all. **Illustrative, not measured** — the real numbers are read from the `.dat`, and the
// denominators differ per era on purpose: 640 is the 641-achievement Repentance+ file minus
// its slot 0, 637 the 638 of a Repentance one.
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
    preview: {
      achievements: { kind: 'read', done: 379, of: 640 },
      items: { kind: 'read', done: 612, of: 732 },
      marks: { kind: 'read', done: 148, of: 368 },
      unreadableCells: 40,
    },
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
    preview: {
      achievements: { kind: 'read', done: 12, of: 640 },
      items: { kind: 'read', done: 74, of: 732 },
      marks: { kind: 'read', done: 3, of: 368 },
      unreadableCells: 40,
    },
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
    preview: {
      achievements: { kind: 'read', done: 210, of: 637 },
      items: { kind: 'unread' },
      marks: { kind: 'read', done: 96, of: 368 },
      unreadableCells: 40,
    },
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
    preview: null,
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
  // The Kit runs outside Tauri, where no protocol answers: a URL here would draw a broken
  // picture in every empty square instead of the empty square itself.
  unknownIconUrl: null,
})

export const noneSetup: SetupState = {
  steam: null,
  game: null,
  unknownIconUrl: null,
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
    { kind: 'cutscene_counters', count: 27 },
    { kind: 'unknown9', count: 2 },
    { kind: 'bestiary', count: 80 },
  ],
  diagnostics: [],
}
