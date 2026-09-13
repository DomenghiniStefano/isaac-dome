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

// Where the Steam, game, saves chain broke down. No fields: a plain string.
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

// Mirrors crates/ipc/src/marks.rs. `tainted` groups the rows the way a player does, while
// `group` stays the file's three blocks. Every URL is null when the game's archives aren't
// open, and the screen draws the fallback outfit.
export interface CharacterRow {
  character: string
  group: string
  tainted: boolean
  cells: Cell[]
  headUrl: string | null
}

// One column's symbol URLs, one per tier.
export interface MarkArtView {
  normalUrl: string | null
  hardUrl: string | null
}

export interface MarksMatrix {
  characters: CharacterRow[]
  bosses: string[]
  // art[i] draws bosses[i].
  art: MarkArtView[]
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

// Mirrors crates/ipc/src/reasons.rs. Why a command could not answer, as variants: a
// reason built with `format!` in Rust is not translatable, and the system's own message
// can name a path. The numbers travel as numbers and the sentence is built here.
// No fields: a bare string, like `MissingReason`.
export const IoReason = {
  NotFound: 'notFound',
  PermissionDenied: 'permissionDenied',
  Other: 'other',
} as const
export type IoReason = (typeof IoReason)[keyof typeof IoReason]

export type SaveReason =
  { kind: 'tooShort' } | { kind: 'badMagic' } | { kind: 'io'; reason: IoReason }

export type SettingsReason =
  | { kind: 'configDirUnknown' }
  | { kind: 'io'; reason: IoReason }
  | { kind: 'encoding' }

export type StoreReason =
  | { kind: 'dataDirUnknown' }
  | { kind: 'dataDirNotCreatable' }
  | { kind: 'unreadable' }
  | { kind: 'newerSchema'; found: number; supported: number }
  | { kind: 'queueUnparseable' }

// Mirrors crates/app/src/error.rs. The tag is "kind" in camelCase; the fields
// of the struct variants are also camelCase (rename_all_fields).
export type IpcError =
  | { kind: 'noActiveProfile' }
  | { kind: 'unknownProfile'; id: string }
  | { kind: 'unreadableSave'; reason: SaveReason }
  | { kind: 'settingsNotWritable'; reason: SettingsReason }
  | { kind: 'unknownTarget' }
  | { kind: 'catalogUnavailable' }
  | { kind: 'storeUnavailable'; reason: StoreReason }
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
      // How to get it, in one line: the game's own `unlock_condition` where it states one,
      // and the wiki's requirement where it does not. It was called `hint` while it was only
      // the file's — 283 of 637 achievements, and 16 of the 119 unlockable now — and the name
      // changed with the meaning so that every reader had to be revisited.
      condition: string | null
      // A link the app serves, never an embedded image: `isaac://achievement/19` (on Windows
      // the same thing arrives rewritten as `http://isaac.localhost/achievement/19`). Put it
      // straight into an `<img src>` — the browser does the lazy loading, the caching and the
      // de-duplication, which is the reason the payload went from megabytes to kilobytes.
      // `null` means the catalog knows no picture for it: draw the placeholder, not a broken
      // image.
      iconUrl: string | null
    }
  | { kind: 'unknown'; slot: number }

// The twelve columns of the completion matrix. A value, not a discriminator — so on the
// wire it is a bare string, and here a union of values.
export const MarkColumnView = {
  MomsHeart: 'momsHeart',
  Isaac: 'isaac',
  Satan: 'satan',
  BossRush: 'bossRush',
  BlueBaby: 'blueBaby',
  TheLamb: 'theLamb',
  MegaSatan: 'megaSatan',
  Greed: 'greed',
  Hush: 'hush',
  Delirium: 'delirium',
  Mother: 'mother',
  TheBeast: 'theBeast',
} as const
export type MarkColumnView =
  (typeof MarkColumnView)[keyof typeof MarkColumnView]

// A level inside a cell, named for its bit. `second` is Ultra Greedier in the Greed
// column, measured; what it means in the other eleven is not, so it is not called `hard`.
export const MarkLevelView = {
  Base: 'base',
  Second: 'second',
} as const
export type MarkLevelView = (typeof MarkLevelView)[keyof typeof MarkLevelView]

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
      // The same kind of link as `AchievementRef.known.iconUrl` above: `isaac://item/passive/92`.
      iconUrl: string | null
      // Where to read about it. `null` means the dataset has no page: the name shows and
      // does not link, never a link that leads nowhere. The same field and the same rule as
      // `RequirementView.page` — what a node unlocks and what blocks it are two halves of
      // one row, and they answer alike (B35).
      page: Target | null
    }
  | {
      kind: 'character'
      id: number
      name: string
      tainted: boolean
      page: Target | null
    }
  | { kind: 'boss'; id: number; name: string; page: Target | null }
  // The challenge's reward: the ids of the achievements that completing it grants.
  // Each one's node (done or not, what it unlocks) already lives in UnlockView.nodes,
  // indexed by id.
  | {
      kind: 'challenge'
      id: number
      name: string
      rewards: number[]
      page: Target | null
    }

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

// `partial` carries no stepsMissing on purpose: with a requirement uninterpreted, or a
// node caught in a cycle, the transitive count isn't knowable and a zero would read as
// "nothing in the way". A node that is `partial` must never be drawn as unlockable.
export type GraphInfo =
  | {
      kind: 'computed'
      availableNow: boolean
      blockedBy: number
      fanOut: number
      stepsMissing: number
    }
  | { kind: 'partial'; blockedBy: number; fanOut: number; unknown: number }

// What a node is still missing, typed by the nature of the target: this is what the
// screen groups by, so it can say "1 character and 2 bosses" instead of "blocked by 3".
export type RequirementView =
  // `tainted` is part of the identity, not decoration: the two forms of a character share
  // the game's name, so the name alone names both (`docs/BACKLOG.md` B28).
  // `page` is the wiki page that says how *this* is unlocked: `null` means the dataset has
  // no page for it — the name shows and does not link. Never "no requirement".
  | {
      kind: 'character'
      id: number
      name: string
      tainted: boolean
      page: Target | null
    }
  | { kind: 'boss'; id: number; name: string; page: Target | null }
  | { kind: 'challenge'; id: number; name: string; page: Target | null }
  | {
      kind: 'item'
      itemKind: ItemKindView
      id: number
      name: string
      page: Target | null
    }
  // A gate carries no page by construction: it is a condition we chose not to resolve to an
  // entity. Same for a mark and a counter below.
  | { kind: 'gate'; label: string }
  // One cell of the completion matrix. No progress: for a cell the state is binary, and a
  // percentage here would be a number nobody measured.
  | {
      kind: 'mark'
      character: number
      characterName: string
      column: MarkColumnView
      level: MarkLevelView
    }
  // A tally and its threshold, with where the profile stands. The one requirement that is
  // not a wall: the content is already reachable, it only has to be played.
  | { kind: 'counter'; label: string; current: number; atLeast: number }
  // A transformation: N of a set of items, in any combination. Like a counter it is not a
  // wall and appears only while `current < atLeast`; unlike everything else here it lists
  // what would satisfy it rather than the one thing blocking it. `unresolved` is how many of
  // the wiki's items this catalog does not have — they can only add to `current`, so a
  // non-zero value means the count shown is a floor.
  | {
      kind: 'threshold'
      transformation: number
      label: string
      current: number
      atLeast: number
      of: ThresholdItemView[]
      unresolved: number
      page: Target | null
    }
  | { kind: 'unknown'; label: string }

export interface ThresholdItemView {
  itemKind: ItemKindView
  id: number
  name: string
  /** Whether the profile can already find it: its achievement is done, or nothing gates it. */
  unlocked: boolean
  page: Target | null
}

export interface UnlockNode {
  achievement: AchievementRef
  done: boolean
  unlocks: UnlockTarget[]
  origin: OriginView | null
  missing: RequirementView[]
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

// What a section is ordered by. No fields: a string, like `OriginView`. `closeness` is the
// one a counter makes possible — it is the only requirement that carries a distance.
export const StepsBasis = {
  FanOut: 'fanOut',
  Closeness: 'closeness',
} as const
export type StepsBasis = (typeof StepsBasis)[keyof typeof StepsBasis]

// One reason and the steps it produced. A section is never emitted empty — the rule lives in
// Rust — so the screen never has to draw a heading over nothing.
export interface StepsSection {
  basis: StepsBasis
  steps: UnlockNode[]
}

export interface NextSteps {
  sections: StepsSection[]
}

// B37 — a want, read from the other end of the graph: you name a thing, and these are the
// ways to it. `routes` is a list because a challenge can be named by two achievements (14 of
// 45 are, measured 2026-09-13); an empty list always travels with the diagnostic that says
// which empty it is.
export type WantState =
  | { kind: 'done' }
  | { kind: 'availableNow' }
  // The prerequisites in the order the Plan would play them, the wanted node excluded.
  // `unknown` counts the steps — the final node included — whose requirements the graph only
  // partly interprets: an empty `steps` is never read as "nothing in the way".
  | { kind: 'chain'; steps: UnlockNode[]; unknown: number }
  // Section 1 was not read: the route is named, where you stand is not claimed.
  | { kind: 'noProfile' }

export interface WantRoute {
  node: UnlockNode
  state: WantState
}

// What you named, drawn back. `UnlockTarget` has four variants and none of them is an
// achievement, so a want named as an achievement — the only way to ask for what the catalog
// models no target for, a mode or an event — is its own case.
export type WantedView =
  | { kind: 'target'; target: UnlockTarget }
  | { kind: 'achievement'; achievement: AchievementRef }
  | { kind: 'unresolved' }

export type WantDiagnostic =
  | { kind: 'noCatalog' }
  | { kind: 'noProfile' }
  | { kind: 'nothingUnlocks' }
  | { kind: 'notUnlockable' }

export interface WantView {
  wanted: WantedView
  routes: WantRoute[]
  diagnostics: WantDiagnostic[]
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
  | { kind: 'storeUnavailable'; reason: StoreReason }
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

// No fields: a bare camelCase string, like `SectionKind`. Which of the wiki's two
// collectible templates the page used — not the game's three-way item kind, because the
// wiki has no familiar template and writes familiars with the passive one.
export const CollectibleTemplate = {
  Passive: 'passive',
  Activated: 'activated',
} as const
export type CollectibleTemplate =
  (typeof CollectibleTemplate)[keyof typeof CollectibleTemplate]

export type Infobox =
  | {
      kind: 'item'
      /// The pickup quote — the same string as the game's own item description.
      quote: Inline[]
      template: CollectibleTemplate
      quality: number | null
      tags: string[]
      /// Not a number: the real values include `unlimited`, `one time`, `4s`, and forms
      /// that differ per edition.
      recharge: Inline[]
      devilPrice: Inline[]
      shopPrice: Inline[]
      /// Only what the wiki states — 45 of 720 pages. The game's own pools are the
      /// complete source, and they arrive through the catalog, not here.
      pools: Inline[]
    }
  | { kind: 'trinket'; quote: Inline[]; tags: string[]; pools: Inline[] }
  | {
      kind: 'achievement'
      requirements: Inline[]
      /// Caveats on the requirement. An achievement has no sections of its own, so this is
      /// the only prose it carries beyond the description and the requirement.
      notes: Inline[]
      unlocks: Target | null
    }
  | {
      kind: 'boss'
      baseHp: number | null
      /// Inline, not a number: the real values are per-stage notes.
      stageHp: Inline[]
      variant: number | null
      environment: Inline[]
      pool: Inline[]
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
      /// The character the challenge forces, when it forces one.
      character: Target | null
      unlocks: Target | null
    }
  | {
      kind: 'character'
      health: Inline[]
      damage: string
      tears: string
      range: string
      speed: string
      luck: string
      shotSpeed: string
      pickups: Inline[]
      collectibles: Inline[]
      /// The character this one is a variant of.
      parent: Target | null
    }

export interface Section {
  kind: SectionKind
  blocks: Block[]
}
export interface Entry {
  title: string
  revid: number
  /// The infobox's summary line. Plain text for achievements, wikitext elsewhere: one
  /// shape for every kind, so reading it needs no switch.
  description: Inline[]
  /// The edition codes the infobox declares. Empty when it declares none — which is not
  /// the same as "it exists everywhere": the wiki does not say which of the two it means.
  dlc: Dlc[]
  /// What the wiki states has to be unlocked first. `null` means the wiki does not state
  /// one, never "it is free": that answer comes from the catalog, not from here.
  unlockedBy: Target | null
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
        transformations: number
      }
      unresolved: number
      unknownTemplates: number
      gameNewerThanSnapshot: boolean | null
    }
  | { kind: 'missing'; reason: WikiMissingReason }

// Mirrors crates/ipc/src/wiki.rs `WikiPageRef` and `WikiIndex`: every page the dataset has,
// once per window. `iconUrl` is null when there is no catalog or the game draws nothing for
// the page: a placeholder, never a broken image.
export interface WikiPageRef {
  target: Target
  title: string
  iconUrl: string | null
}
export interface WikiIndex {
  info: WikiInfo
  pages: WikiPageRef[]
}

// --- search -----------------------------------------------------------------
// Mirrors crates/ipc/src/search.rs. `ProgressMark` and `SearchDiagnostic` are fieldless in
// Rust, so they travel as bare strings: values, not discriminators.
export const ProgressMark = {
  Done: 'done',
  Pending: 'pending',
  Unknown: 'unknown',
  None: 'none',
} as const
export type ProgressMark = (typeof ProgressMark)[keyof typeof ProgressMark]

export const SearchDiagnostic = {
  NoProfile: 'noProfile',
  NoCatalog: 'noCatalog',
  NoWiki: 'noWiki',
  NoAchievementSection: 'noAchievementSection',
  NoCollectionSection: 'noCollectionSection',
} as const
export type SearchDiagnostic =
  (typeof SearchDiagnostic)[keyof typeof SearchDiagnostic]

// Why the hit matched, in the words its row shows.
export type SearchMatch =
  | { kind: 'title' }
  | { kind: 'condition'; text: string }
  | {
      kind: 'section'
      section: SectionKind
      before: string
      matched: string
      after: string
    }

export interface SearchHit {
  target: Target
  title: string
  iconUrl: string | null
  // The dataset has this page: a Wiki destination exists for the hit.
  hasPage: boolean
  match: SearchMatch
  progress: ProgressMark
}

export interface SearchView {
  query: string
  hits: SearchHit[]
  // How many matched before the limit: what "mostrati 300 di N" is read from.
  total: number
  diagnostics: SearchDiagnostic[]
}

// --- The plan queue ---

// A queue row is an **achievement**, not a target: wanting Tainted Lost and wanting the
// achievement that unlocks it are the same wish seen from two sides.
export interface QueueRow {
  // The same node the Unlock screen draws, so the two can never disagree.
  node: UnlockNode
  // You asked for this one, for itself.
  wanted: boolean
  // The wanted achievements whose chain passes through this row. Both `wanted` and a
  // non-empty `origins` can be true at once: you asked for it *and* it serves another wish.
  origins: number[]
  // Prerequisites this row still needs that are not in the queue.
  stepsNotQueued: number
}

// Every way a row can be absent, said out loud. `unreadable` and an empty queue are
// different things, and so are `completed` and a row that just vanished.
export type QueueDiagnostic =
  | { kind: 'storeUnavailable'; reason: StoreReason }
  | { kind: 'unreadable' }
  | { kind: 'completed'; count: number; wanted: number[] }
  | { kind: 'unresolved'; achievement: number }
  | { kind: 'goalsPending'; count: number }
  | { kind: 'noCatalog' }

export interface QueueView {
  rows: QueueRow[]
  diagnostics: QueueDiagnostic[]
  // `false` when the database won't open: the queue can't be seen or changed, and the UI
  // says so instead of showing an empty list.
  storeAvailable: boolean
}

// --- The Collection ---

// What stands between an item and a run. Tagged: three variants carry data. `locked` means its
// achievement isn't done, so the item can't appear yet; `unknown` means section 1 wasn't read.
// `page` is the achievement's wiki page, `null` when the dataset has none. `free` has no such
// key: nothing unlocks the item, so there is nothing to open.
export type LockView =
  | { kind: 'free' }
  | {
      kind: 'unlocked'
      achievement: number
      text: string | null
      page: Target | null
    }
  | {
      kind: 'locked'
      achievement: number
      text: string | null
      page: Target | null
    }
  | {
      kind: 'unknown'
      achievement: number
      text: string | null
      page: Target | null
    }

// A collectible as the Collection shows it — never a trinket, which has no slot in section 4.
// `inCollection: null` is unread (section 4 missing, or no slot for this id), never "not in the
// collection".
export interface CollectionItem {
  id: number
  kind: ItemKindView
  name: string
  iconUrl: string | null
  quality: number | null
  pools: string[]
  origin: OriginView | null
  inCollection: boolean | null
  lock: LockView
}

export interface CollectionTotals {
  slots: number
  items: number
  inCollection: number
}

export type CollectionDiagnostic =
  | { kind: 'noCatalog' }
  | { kind: 'noCollectionSection' }
  | { kind: 'noAchievementSection' }
  | { kind: 'itemsBeyondSlots'; count: number }

export interface CollectionView {
  items: CollectionItem[]
  // The pools any listed item belongs to, each once, in the catalog's order.
  pools: string[]
  totals: CollectionTotals
  diagnostics: CollectionDiagnostic[]
}

// Mirrors crates/ipc/src/settings.rs. What the app persists: the chosen profile's opaque id
// and the interface's size as a percentage, which the backend always answers snapped to the
// eleven steps of `lib/scale/steps.ts`.
export interface Settings {
  activeProfileId: string | null
  scale: number
}
