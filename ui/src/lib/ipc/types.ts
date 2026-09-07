// Value, not discriminator: field-less enums arrive as a bare camelCase string.
// On the wire they stay strings; here they're constants because convention rule 5
// doesn't allow literal unions, and comparisons reference the symbol.
export const CandidateSource = {
  SteamCloud: 'steamCloud',
  Documents: 'documents',
  Manual: 'manual',
} as const
export type CandidateSource =
  (typeof CandidateSource)[keyof typeof CandidateSource]

// The two save-file name prefixes, `rep_` and `rep+`. They come from `SavePrefix` in
// Rust, which is a domain enum: it keeps its `snake_case`, and the values here mirror it.
export const SavePrefix = { Rep: 'rep', RepPlus: 'rep_plus' } as const
export type SavePrefix = (typeof SavePrefix)[keyof typeof SavePrefix]

export interface CandidateView {
  id: string
  prefix: SavePrefix
  slot: number
  source: CandidateSource
  modifiedUnix: number | null
  sizeBytes: number
  suggested: boolean
  pathHint: string
}

// Where the Steam → game → saves chain broke down. No fields: a plain string.
export const MissingReason = {
  SteamNotFound: 'steamNotFound',
  GameNotFound: 'gameNotFound',
  NoSaves: 'noSaves',
} as const
export type MissingReason = (typeof MissingReason)[keyof typeof MissingReason]

export type ActiveProfile =
  | { kind: 'none'; reason: MissingReason }
  | {
      kind: 'needsChoice'
      reason:
        { kind: 'neverChosen' } | { kind: 'savedProfileGone'; was: string }
      suggested: string | null
    }
  | { kind: 'active'; profile: CandidateView; autoSelected: boolean }

export interface SetupState {
  steam: { rootHint: string; libraries: number } | null
  game: { dirHint: string; edition: string; dlcs: string[] } | null
  candidates: CandidateView[]
  active: ActiveProfile
  diagnostics: SetupDiagnostic[]
}

export type SetupDiagnostic =
  | { kind: 'steamNotFound' }
  | { kind: 'gameNotFound' }
  | { kind: 'noSavesFound' }
  | { kind: 'unreadablePath'; name: string; reason: string }
  | { kind: 'malformedManifest'; name: string }

export type Cell =
  | { kind: 'known'; bits: number }
  | { kind: 'unknown' }
  | { kind: 'unexpected'; value: number }

export interface MarksMatrix {
  characters: { character: string; group: string; cells: Cell[] }[]
  bosses: string[]
  totals: {
    cells: number
    readable: number
    unknown: number
    unexpected: number
    started: number
  }
}

export interface SaveSummary {
  profile: string
  sections: { kind: string; count: number }[]
  diagnostics: SaveDiagnostic[]
}

export type SaveDiagnostic =
  | { kind: 'unexpectedKind'; expected: number; found: number }
  | { kind: 'sectionOverrun'; section: number }
  | { kind: 'trailingBytes' }

// Mirrors crates/app/src/error.rs. The tag is "kind" in camelCase; the fields
// of the struct variants are also camelCase (rename_all_fields).
export type IpcError =
  | { kind: 'noActiveProfile' }
  | { kind: 'unknownProfile'; id: string }
  | { kind: 'unreadableSave'; reason: string }
  | { kind: 'settingsNotWritable'; reason: string }
  | { kind: 'unknownTarget' }
  | { kind: 'catalogUnavailable' }
  | { kind: 'storeUnavailable'; reason: string }
  | { kind: 'wikiUnavailable' }

// Mirrors crates/ipc/src/resources.rs. The variant labels are pinned by a test
// on the JSON shape: `miniZ` is not a typo.
export type ArchiveMode =
  | { kind: 'bogocrypt1' }
  | { kind: 'lzw' }
  | { kind: 'miniZ' }
  | { kind: 'bogocrypt2' }
  | { kind: 'unknown'; value: number }

export interface ArchiveView {
  name: string
  mode: ArchiveMode
  entries: number
}

// Mirrors crates/ipc/src/catalog_view.rs.
export interface KindCounts {
  passives: number
  actives: number
  familiars: number
  trinkets: number
}

export interface CatalogView {
  counts: KindCounts
  total: number
  unresolvedNames: number
  languages: string[]
}

export interface SpriteView {
  id: number
  name: string
  dataUrl: string
}

export interface ExtractionReport {
  archives: ArchiveView[]
  totalEntries: number
  catalog: CatalogView | null
  sprites: SpriteView[]
  wiki: WikiInfo
}

// Mirrors crates/ipc/src/graph.rs and goals.rs. One rule for enums: those whose
// variants carry different data are unions on `kind`, those with no fields are
// string unions. `UnlockTarget.item` carries `itemKind`, not `kind`: `kind` is the tag.
export type AchievementRef =
  | {
      kind: 'known'
      id: number
      text: string
      hint: string | null
      iconUrl: string | null
    }
  | { kind: 'unknown'; slot: number }

// Value, not discriminator: `ItemKindView` has no fields, so on the wire it's
// a bare string, like `OriginView`.
export const ItemKindView = {
  Passive: 'passive',
  Active: 'active',
  Familiar: 'familiar',
  Trinket: 'trinket',
} as const
export type ItemKindView = (typeof ItemKindView)[keyof typeof ItemKindView]

// The identity of a target, and the on-disk database format: name and icon aren't
// there, they're resolved from the catalog on every read. It's what `addGoal` sends back.
export type TargetKey =
  | { kind: 'item'; itemKind: ItemKindView; id: number }
  | { kind: 'character'; id: number }
  | { kind: 'boss'; id: number }
  | { kind: 'challenge'; id: number }

// The same identity as the UI shows it: name and icon resolved now.
export type UnlockTarget =
  | {
      kind: 'item'
      itemKind: ItemKindView
      id: number
      name: string
      iconUrl: string | null
    }
  | { kind: 'character'; id: number; name: string }
  | { kind: 'boss'; id: number; name: string }
  // The challenge's reward: the ids of the achievements that completing it grants.
  // Each one's node (done or not, what it unlocks) already lives in UnlockView.nodes,
  // indexed by id.
  | { kind: 'challenge'; id: number; name: string; rewards: number[] }

// Value, not discriminator: the origin DLC as `OriginView` serializes it in Rust.
// It's not the wiki's `Dlc`, which also has `repentancePlus`: here the variants are
// the ones `catalog::Origin` can infer from the items' id ranges.
export const OriginView = {
  Rebirth: 'rebirth',
  Afterbirth: 'afterbirth',
  AfterbirthPlus: 'afterbirthPlus',
  Repentance: 'repentance',
} as const
export type OriginView = (typeof OriginView)[keyof typeof OriginView]

export type GraphInfo =
  | { kind: 'stub' }
  | {
      kind: 'computed'
      availableNow: boolean
      blockedBy: number
      fanOut: number
      stepsMissing: number
    }

export interface UnlockNode {
  achievement: AchievementRef
  done: boolean
  unlocks: UnlockTarget[]
  origin: OriginView | null
  graph: GraphInfo
}

export interface UnlockTotals {
  slots: number
  done: number
  known: number
  unknown: number
}

// `noAchievementSection` means "section 1 of the save didn't get read": nodes and
// totals are zero, and that doesn't mean "zero achievements done".
export type UnlockDiagnostic =
  | { kind: 'slotsBeyondCatalog'; count: number }
  | { kind: 'catalogBeyondSlots'; count: number }
  | { kind: 'noCatalog' }
  | { kind: 'noAchievementSection' }

export interface UnlockView {
  nodes: UnlockNode[]
  totals: UnlockTotals
  diagnostics: UnlockDiagnostic[]
}

// What the steps list is ordered by. No fields: a string, like `OriginView`.
export const StepsBasis = { Stub: 'stub', FanOut: 'fanOut' } as const
export type StepsBasis = (typeof StepsBasis)[keyof typeof StepsBasis]

export interface NextSteps {
  steps: UnlockNode[]
  basis: StepsBasis
}

// `GoalId` is a transparent newtype in Rust: on the wire it's an opaque string,
// generated by the app, never constructed by the frontend.
export type GoalId = string

// A goal as the UI receives it: the saved key plus what the current catalog
// knows about it. `target: null` means "not resolvable right now" (game not
// installed, or an id a patch removed): the row stays, and is removed by `id`.
export interface GoalView {
  id: GoalId
  key: TargetKey
  target: UnlockTarget | null
  createdUnix: number
  note: string | null
}

export interface PlanStep {
  goal: GoalId
  node: UnlockNode
  done: boolean
}

export type PlanExpansion =
  { kind: 'stub' } | { kind: 'computed'; steps: PlanStep[] }

// The plan degrades and says why. `storeAvailable` is derived in Rust (= no
// `storeUnavailable` diagnostic): the UI uses it as a gate and reads the diagnostics
// for the text. `unreadableGoal` and `unresolvedGoal` carry the id: the UI can offer
// to remove them. `noCatalog` arrives once, not once per goal.
export type PlanDiagnostic =
  | { kind: 'storeUnavailable'; reason: string }
  | { kind: 'unreadableGoal'; id: GoalId }
  | { kind: 'noCatalog' }
  | { kind: 'unresolvedGoal'; id: GoalId }

export interface PlanView {
  goals: GoalView[]
  expansion: PlanExpansion
  diagnostics: PlanDiagnostic[]
  storeAvailable: boolean
}

// --- wiki -----------------------------------------------------------------
export const SectionKind = {
  Effects: 'effects',
  Notes: 'notes',
  Synergies: 'synergies',
  Interactions: 'interactions',
  Bugs: 'bugs',
  Behavior: 'behavior',
  ChampionVersions: 'championVersions',
  DamageScaling: 'damageScaling',
  Strategies: 'strategies',
  Difficulty: 'difficulty',
  Reward: 'reward',
  Unlockable: 'unlockable',
} as const
export type SectionKind = (typeof SectionKind)[keyof typeof SectionKind]

export const Style = { Plain: 'plain', Bold: 'bold', Italic: 'italic' } as const
export type Style = (typeof Style)[keyof typeof Style]

export const Dlc = {
  Rebirth: 'rebirth',
  Afterbirth: 'afterbirth',
  AfterbirthPlus: 'afterbirthPlus',
  Repentance: 'repentance',
  RepentancePlus: 'repentancePlus',
} as const
export type Dlc = (typeof Dlc)[keyof typeof Dlc]

// The identity of a wiki element: what a `ref` points to, and what `loadWiki`
// accepts to load a page.
export type Target =
  | { kind: 'item'; id: number }
  | { kind: 'trinket'; id: number }
  | { kind: 'character'; id: number }
  | { kind: 'achievement'; id: number }
  | { kind: 'challenge'; number: number }
  | { kind: 'entity'; id: number; variant: number; subtype: number }
  | { kind: 'transformation'; id: number }
  | { kind: 'stage'; name: string }
  | { kind: 'room'; name: string }
  | { kind: 'pickup'; name: string }

export type Inline =
  | { kind: 'text'; text: string; style: Style }
  | { kind: 'ref'; target: Target; label: string }
  | { kind: 'concept'; page: string; label: string }
  | { kind: 'edition'; only: Dlc[]; inline: Inline[] }

export interface ListItem {
  inline: Inline[]
  children: Block[]
}
export type Block =
  | { kind: 'paragraph'; inline: Inline[] }
  | { kind: 'list'; ordered: boolean; items: ListItem[] }
  | { kind: 'table'; header: Inline[][]; rows: Inline[][][] }
  | { kind: 'heading'; level: number; inline: Inline[] }

export type Infobox =
  | { kind: 'item' }
  | { kind: 'trinket' }
  | {
      kind: 'achievement'
      description: string
      requirements: Inline[]
      unlocks: Target | null
    }
  | {
      kind: 'boss'
      baseHp: number | null
      environment: Inline[]
      pool: Inline[]
      unlockedBy: Target | null
    }
  | {
      kind: 'challenge'
      blindfolded: boolean
      hasShops: boolean
      hasTreasureRooms: boolean
      items: Inline[]
      trinkets: Inline[]
      pickups: Inline[]
      health: Inline[]
      curse: Inline[]
      goal: Inline[]
      unlocks: Target | null
      unlockedBy: Target | null
    }
  | {
      kind: 'character'
      health: Inline[]
      damage: string
      range: string
      speed: string
      luck: string
      shotSpeed: string
      pickups: Inline[]
      collectibles: Inline[]
      unlockedBy: Target | null
    }

export interface Section {
  kind: SectionKind
  blocks: Block[]
}
export interface Entry {
  title: string
  revid: number
  infobox: Infobox
  sections: Section[]
}

// No fields: a bare string, like `MissingReason`.
export const WikiMissingReason = {
  SchemaMismatch: 'schemaMismatch',
  Malformed: 'malformed',
} as const
export type WikiMissingReason =
  (typeof WikiMissingReason)[keyof typeof WikiMissingReason]

// The state of the embedded dataset: loaded (with counts and diagnostics) or not
// (with a reason). `unresolved` and `unknownTemplates` are totals of occurrences,
// not of pages: unresolved (occurrences) and unknown templates (occurrences).
export type WikiInfo =
  | {
      kind: 'loaded'
      snapshotAt: string
      lastKnownPatch: { number: string; date: string } | null
      counts: {
        items: number
        trinkets: number
        achievements: number
        bosses: number
        challenges: number
        characters: number
      }
      unresolved: number
      unknownTemplates: number
      gameNewerThanSnapshot: boolean | null
    }
  | { kind: 'missing'; reason: WikiMissingReason }
