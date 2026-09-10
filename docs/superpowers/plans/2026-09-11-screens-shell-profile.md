# Cycle 3.1 — the shell and profile selection: Implementation Plan

> **For agentic workers:** executed inline with superpowers:executing-plans (the owner
> declined the one-subagent-per-task loop). Steps use checkbox (`- [ ]`) syntax.

**Goal:** the window becomes the app — tabs that own router locations, live window chrome,
navbar with the profile indicator, section sidebar — with profile selection as the first real
screen and placeholders for the rest, verifiable in a browser through fixtures.

**Architecture:** a Pinia store owns the tabs; the router (memory history) renders the active
tab's location. Pure rules live beside their stores and in `lib/profile/`, tested first.
Components stay presentational where they are shared (`components/shell/`); screens read the
stores. IPC wrappers go through one `transport` that answers from fixtures in a development
browser.

**Tech Stack:** Vue 3.5, TypeScript 6, vue-router 4.6.4, pinia 4.0.3, Tailwind v4.3, Reka UI
2.10.4, `@tauri-apps/api` 2.11.1, vue-i18n 11.4, Vitest 5.

**Spec:** `docs/superpowers/specs/2026-09-11-screens-shell-profile-design.md`

## Global Constraints

- No IPC change and no new Tauri command; Rust changes are `tauri.conf.json` and a capability file.
- Components never call `invoke()` or the window API: `lib/ipc/` and `lib/window/` only.
- The five frontend rules; exhaustive switches with `assertNever`; `as const` objects, no string unions.
- Every visible string through `useMessages()`; game names (editions, DLCs) are data.
- Fixtures, the Kit page and the verification page never reach the production build.
- Commits: Conventional Commits, English, no `Co-Authored-By`, at logical boundaries.
- Deviations from the spec, decided while planning and recorded in Task 9: pure profile logic
  in `lib/profile/` (the shell's indicator uses it too), a component-free `router/routeTable.ts`
  (tab labels and the sidebar are testable without screens), `formatCount` instead of
  `formatBytes` (the unit word is a message), `formatModified(null)` returns `null`, vue-router
  4.6.4 rather than 5.x (5 ships file-based routing tooling as runtime dependencies).

---

### Task 1: One transport for every command, and fixtures

**Files:**
- Modify: `ui/package.json`, `pnpm-lock.yaml` (vue-router 4.6.4, pinia 4.0.3)
- Create: `ui/src/lib/ipc/transport.ts`, `ui/src/lib/ipc/fixtures/index.ts`, `ui/src/lib/ipc/fixtures/profile.ts`
- Test: `ui/src/lib/ipc/transport.test.ts`
- Modify: `ui/src/lib/ipc/setup.ts`, `save.ts`, `graph.ts`, `queue.ts`, `resources.ts`, `wiki.ts`

**Interfaces:**
- Produces: `call<T>(command: CommandName, args?: CommandArgs): Promise<T>`; `CommandName`, `CommandArgs`; fixtures `answer`, `resetFixtures`, `FixtureScenario { None, Pick, Active }`.

- [ ] **Step 1: Install** — `pnpm --filter ui add vue-router@4.6.4 pinia@4.0.3`

- [ ] **Step 2: Write the failing test** — `ui/src/lib/ipc/transport.test.ts`

```ts
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
```

- [ ] **Step 3: Run it to see it fail** — `pnpm --filter ui exec vitest run src/lib/ipc` → FAIL, `Cannot find module './fixtures'`.

- [ ] **Step 4: `ui/src/lib/ipc/transport.ts`**

```ts
import { invoke, isTauri } from '@tauri-apps/api/core'
import type { Command } from '../constants/commands'

export type CommandName = (typeof Command)[keyof typeof Command]
export type CommandArgs = Record<string, unknown>

// Every wrapper goes through here. Inside Tauri it is invoke(). On the development server in
// a plain browser there is no backend, so a development build answers from the fixtures; in
// a production build the branch is dead and the fixtures module is never bundled.
export const call = async <T>(
  command: CommandName,
  args?: CommandArgs,
): Promise<T> => {
  if (!isTauri() && import.meta.env.DEV) {
    const { answer } = await import('./fixtures')
    return answer<T>(command, args)
  }
  return invoke<T>(command, args)
}
```

- [ ] **Step 5: `ui/src/lib/ipc/fixtures/profile.ts`**

```ts
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
    dirHint: 'D:\\SteamLibrary\\steamapps\\common\\The Binding of Isaac Rebirth',
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
```

- [ ] **Step 6: `ui/src/lib/ipc/fixtures/index.ts`**

```ts
import { Command } from '../../constants/commands'
import { assertNever } from '../../assertNever'
import type { CommandArgs, CommandName } from '../transport'
import type { IpcError, SetupState } from '../types'
import { candidates, noneSetup, setupWith, summary } from './profile'

export const FixtureScenario = {
  None: 'none',
  Pick: 'pick',
  Active: 'active',
} as const
export type FixtureScenario =
  (typeof FixtureScenario)[keyof typeof FixtureScenario]

// `?fixture=none|pick|active` on the development server; active when absent.
const ScenarioParam = 'fixture'

const currentScenario = (): FixtureScenario => {
  const requested = new URLSearchParams(globalThis.location?.search ?? '').get(
    ScenarioParam,
  )
  return (
    Object.values(FixtureScenario).find((s) => s === requested) ??
    FixtureScenario.Active
  )
}

// A profile chosen through select_profile stays chosen for the page's life, as in the app.
let chosenId: string | null = null

export const resetFixtures = (): void => {
  chosenId = null
}

const activeOn = (id: string): SetupState => {
  const profile = candidates.find((c) => c.id === id)
  return profile
    ? setupWith({ kind: 'active', profile, autoSelected: false })
    : setupWith({
        kind: 'needsChoice',
        reason: { kind: 'savedProfileGone', was: id },
        suggested: candidates[0]?.id ?? null,
      })
}

const setupFor = (scenario: FixtureScenario): SetupState => {
  switch (scenario) {
    case FixtureScenario.None:
      return noneSetup
    case FixtureScenario.Pick:
      return chosenId === null
        ? setupWith({
            kind: 'needsChoice',
            reason: { kind: 'neverChosen' },
            suggested: candidates[0]?.id ?? null,
          })
        : activeOn(chosenId)
    case FixtureScenario.Active:
      return activeOn(chosenId ?? candidates[0]?.id ?? '')
    default:
      return assertNever(scenario)
  }
}

const noActiveProfile: IpcError = { kind: 'noActiveProfile' }

type Handler = (
  args: CommandArgs | undefined,
  scenario: FixtureScenario,
) => unknown

// Only what the shell reads today. Any other command is refused loudly: a screen built on a
// command with no fixture should fail on the development server, not render undefined.
const handlers: Partial<Record<CommandName, Handler>> = {
  [Command.SetupState]: (_args, scenario) => setupFor(scenario),
  [Command.SelectProfile]: (args, scenario) => {
    chosenId = String(args?.id ?? '')
    return setupFor(scenario)
  },
  [Command.SaveSummary]: (_args, scenario) =>
    setupFor(scenario).active.kind === 'active'
      ? summary
      : Promise.reject(noActiveProfile),
}

export const answer = async <T>(
  command: CommandName,
  args?: CommandArgs,
  scenario: FixtureScenario = currentScenario(),
): Promise<T> => {
  const handler = handlers[command]
  if (!handler) throw new Error(`no fixture answers ${command}`)
  return (await handler(args, scenario)) as T
}
```

- [ ] **Step 7: Route every wrapper through `call`** — in each of `setup.ts`, `save.ts`, `graph.ts`, `queue.ts`, `resources.ts`, `wiki.ts`: replace `import { invoke } from '@tauri-apps/api/core'` with `import { call } from './transport'`, and every `invoke(` with `call(`. Nothing else changes. For example `setup.ts` becomes:

```ts
import { Command } from '../constants/commands'
import { call } from './transport'
import type { SetupState } from './types'

export const setupState = (): Promise<SetupState> => call(Command.SetupState)

export const selectProfile = (id: string): Promise<SetupState> =>
  call(Command.SelectProfile, { id })
```

- [ ] **Step 8: Run the tests to see them pass** — `pnpm --filter ui exec vitest run src/lib/ipc` → PASS (5 tests).

- [ ] **Step 9: Verify and commit** — `pnpm typecheck && pnpm lint && pnpm scan`, then

```bash
git add ui/package.json pnpm-lock.yaml ui/src/lib/ipc
git commit -m "feat(ui): one transport for every command, with fixtures for a browser"
```

---

### Task 2: Development pages, the window wrapper and the chrome going live

**Files:**
- Move: `ui/src/App.vue` → `ui/src/verify/VerifyPage.vue` (imports rewritten to `@/`)
- Modify: `ui/src/lib/constants/devRoutes.ts`, `ui/scripts/scan-conventions.mjs`
- Create: `ui/src/lib/window/appWindow.ts`, `crates/app/capabilities/default.json`
- Modify: `crates/app/tauri.conf.json`

**Interfaces:**
- Produces: `DevRoute.Verify = '#verify'`; `minimizeWindow()`, `toggleMaximizeWindow()`, `closeWindow()`, `watchWindowFocus(onChange): Promise<() => void>`.

- [ ] **Step 1: Move the verification page** — `git mv ui/src/App.vue ui/src/verify/VerifyPage.vue`, then replace its import block (the lines from `import { onMounted, ref } from 'vue'` to `import { Label } from '@/components/ui/label'`) with:

```ts
import { onMounted, ref } from 'vue'
import { selectProfile, setupState } from '@/lib/ipc/setup'
import { completion, saveSummary } from '@/lib/ipc/save'
import { extractionReport } from '@/lib/ipc/resources'
import { nextSteps, unlock } from '@/lib/ipc/graph'
import { wikiEntry } from '@/lib/ipc/wiki'
import type {
  ArchiveMode,
  Cell,
  Entry,
  ExtractionReport,
  IpcError,
  MarksMatrix,
  NextSteps,
  SaveSummary,
  SetupState,
  Target,
  UnlockView,
  WikiInfo,
} from '@/lib/ipc/types'
import { StepsBasis } from '@/lib/ipc/types'
import { assertNever } from '@/lib/assertNever'
import WikiBlocks from '@/components/wiki/WikiBlocks.vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
```

- [ ] **Step 2: `ui/src/lib/constants/devRoutes.ts`**

```ts
// Hash routes that only exist under `pnpm ui:dev`: main.ts checks import.meta.env.DEV
// before honouring them, and the production build drops what they import.
export const DevRoute = { Kit: '#kit', Verify: '#verify' } as const
export type DevRoute = (typeof DevRoute)[keyof typeof DevRoute]
```

- [ ] **Step 3: Scanner** — in `ui/scripts/scan-conventions.mjs`:
  replace
  ```js
  const DEV_ONLY_DIR = join('src', 'kit')
  ```
  with
  ```js
  const DEV_ONLY_DIRS = [join('src', 'kit'), join('src', 'verify')]
  const WINDOW_DIR = join('src', 'lib', 'window')
  ```
  (and the comment above it: "Development-only pages: main.ts imports them behind `import.meta.env.DEV`…" stays, now in the plural);
  replace `!isUnder(file, DEV_ONLY_DIR) &&` with `!DEV_ONLY_DIRS.some((dir) => isUnder(file, dir)) &&`;
  delete the `src/App.vue` object from `EXEMPTIONS` (the page is now under a development-only directory), leaving `const EXEMPTIONS = []`;
  after the `invoke() outside src/lib/ipc/` check add
  ```js
  {
    name: 'window API outside src/lib/window/',
    test: (file, body) =>
      /@tauri-apps\/api\/window/.test(body) && !isUnder(file, WINDOW_DIR),
  },
  ```

- [ ] **Step 4: `ui/src/lib/window/appWindow.ts`**

```ts
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'

// The only module that talks to the window, as lib/ipc is for commands. Outside Tauri (the
// development server in a browser) the controls do nothing and the window counts as focused.
export const minimizeWindow = async (): Promise<void> => {
  if (isTauri()) await getCurrentWindow().minimize()
}

export const toggleMaximizeWindow = async (): Promise<void> => {
  if (isTauri()) await getCurrentWindow().toggleMaximize()
}

export const closeWindow = async (): Promise<void> => {
  if (isTauri()) await getCurrentWindow().close()
}

// Calls `onChange` whenever the window gains or loses focus; resolves to the unsubscribe.
export const watchWindowFocus = async (
  onChange: (focused: boolean) => void,
): Promise<() => void> => {
  if (!isTauri()) return () => undefined
  return getCurrentWindow().onFocusChanged(({ payload }) => onChange(payload))
}
```

- [ ] **Step 5: `crates/app/capabilities/default.json`**

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "The main window draws its own title bar: it needs the window controls and dragging.",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-minimize",
    "core:window:allow-toggle-maximize",
    "core:window:allow-close",
    "core:window:allow-start-dragging"
  ]
}
```

- [ ] **Step 6: `crates/app/tauri.conf.json`** — the window becomes
  `"windows": [{ "title": "IsaacDome", "width": 1280, "height": 800, "decorations": false }]`.

- [ ] **Step 7: Verify** — `cargo clippy -p app -- -D warnings` (tauri-build validates the capability), `pnpm typecheck && pnpm lint && pnpm scan`. `main.ts` still mounts `App.vue`, which no longer exists: Task 7 restores it; until then typecheck is expected to fail on `main.ts` only, so this task commits together with Task 7's `main.ts` (no commit here).

---

### Task 3: The tab model

**Files:**
- Create: `ui/src/router/routeTable.ts`, `ui/src/stores/tabModel.ts`, `ui/src/stores/tabs.ts`, `ui/src/lib/constants/stores.ts`
- Test: `ui/src/stores/tabModel.test.ts`, `ui/src/router/routeTable.test.ts`

**Interfaces:**
- Produces: `RouteName`, `WikiCategory`, `TabLocation`, `routeTitle`, `routeIcon`, `routeOrigin`, `routePath`, `routeArrives`, `wikiCategoryTitle`, `wikiCategoryIcon`, `locationTitle(location)`, `defaultLocation`; `Tab`, `TabsState`, `firstState`, `openTab`, `selectTab`, `closeTab`, `moveTab`, `navigateTab`; `useTabsStore` with `tabs`, `activeId`, `active`, `open(location?)`, `select(id)`, `close(id)`, `move(from, to)`, `navigate(location)`.

- [ ] **Step 1: Write the failing tests**

`ui/src/stores/tabModel.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import {
  closeTab,
  firstState,
  moveTab,
  navigateTab,
  openTab,
  selectTab,
} from './tabModel'
import type { Tab, TabsState } from './tabModel'

const at = (name: TabLocation['name']): TabLocation => ({ name })
const three = (): TabsState => ({
  tabs: [
    { id: 'a', location: at(RouteName.NextSteps) },
    { id: 'b', location: at(RouteName.Unlock) },
    { id: 'c', location: at(RouteName.Plan) },
  ],
  activeId: 'b',
})
const ids = (s: TabsState) => s.tabs.map((t) => t.id)
const fresh = (): Tab => ({ id: 'new', location: at(RouteName.NextSteps) })

describe('tabModel', () => {
  it('starts with one active tab', () => {
    expect(firstState('a', at(RouteName.NextSteps))).toEqual({
      tabs: [{ id: 'a', location: at(RouteName.NextSteps) }],
      activeId: 'a',
    })
  })

  it('opens a tab right after the active one and activates it', () => {
    const s = openTab(three(), 'd', at(RouteName.Wiki))
    expect(ids(s)).toEqual(['a', 'b', 'd', 'c'])
    expect(s.activeId).toBe('d')
  })

  it('selects an existing tab and ignores an unknown id', () => {
    expect(selectTab(three(), 'c').activeId).toBe('c')
    expect(selectTab(three(), 'zz').activeId).toBe('b')
  })

  it('closing the active tab activates its right neighbour', () => {
    const s = closeTab(three(), 'b', fresh)
    expect(ids(s)).toEqual(['a', 'c'])
    expect(s.activeId).toBe('c')
  })

  it('closing the active last tab activates its left neighbour', () => {
    const s = closeTab({ ...three(), activeId: 'c' }, 'c', fresh)
    expect(s.activeId).toBe('b')
  })

  it('closing an inactive tab leaves the active one alone', () => {
    const s = closeTab(three(), 'a', fresh)
    expect(ids(s)).toEqual(['b', 'c'])
    expect(s.activeId).toBe('b')
  })

  it('never leaves the bar empty: closing the only tab opens a fresh one', () => {
    const s = closeTab(firstState('a', at(RouteName.Plan)), 'a', fresh)
    expect(s).toEqual({ tabs: [fresh()], activeId: 'new' })
  })

  it('moves a tab to its final index', () => {
    expect(ids(moveTab(three(), 0, 2))).toEqual(['b', 'c', 'a'])
    expect(ids(moveTab(three(), 2, 0))).toEqual(['c', 'a', 'b'])
  })

  it('navigating replaces the active tab location and nothing else', () => {
    const s = navigateTab(three(), at(RouteName.Profile))
    expect(s.tabs.map((t) => t.location.name)).toEqual([
      RouteName.NextSteps,
      RouteName.Profile,
      RouteName.Plan,
    ])
    expect(s.activeId).toBe('b')
  })
})
```

`ui/src/router/routeTable.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { RouteName, WikiCategory, locationTitle } from './routeTable'

describe('locationTitle', () => {
  it("is the route's title", () => {
    expect(locationTitle({ name: RouteName.Profile })).toBe('routes.profile')
  })

  it("is the category's title for a wiki category", () => {
    expect(
      locationTitle({
        name: RouteName.Wiki,
        query: { category: WikiCategory.Bosses },
      }),
    ).toBe('wikiCategories.bosses')
  })

  it('is the wiki title for the wiki without a category', () => {
    expect(locationTitle({ name: RouteName.Wiki })).toBe('routes.wiki')
  })
})
```

- [ ] **Step 2: Run them to see them fail** — `pnpm --filter ui exec vitest run src/stores src/router` → FAIL, modules not found.

- [ ] **Step 3: `ui/src/router/routeTable.ts`**

```ts
import type { Component } from 'vue'
import {
  ActivityIcon,
  AppWindowIcon,
  FlagIcon,
  GemIcon,
  Grid2x2Icon,
  LayersIcon,
  ListChecksIcon,
  LockOpenIcon,
  MapIcon,
  PackageIcon,
  PlayIcon,
  SaveIcon,
  SkullIcon,
  TrophyIcon,
  UserIcon,
} from '@lucide/vue'
import { TabOrigin } from '@/components/shell/tabs'
import { tabOriginIcon } from '@/components/shell/tabOriginIcon'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'

// Everything a location needs to be named, drawn and placed, with no screen component in
// sight: tab labels and the sidebar are built and tested from this alone.
export const RouteName = {
  NextSteps: 'nextSteps',
  Completion: 'completion',
  Unlock: 'unlock',
  Plan: 'plan',
  Collection: 'collection',
  Runs: 'runs',
  Live: 'live',
  Wiki: 'wiki',
  Profile: 'profile',
  TabsSettings: 'tabsSettings',
  About: 'about',
} as const
export type RouteName = (typeof RouteName)[keyof typeof RouteName]

export const WikiCategory = {
  Items: 'items',
  Trinkets: 'trinkets',
  Achievements: 'achievements',
  Bosses: 'bosses',
  Challenges: 'challenges',
  Characters: 'characters',
} as const
export type WikiCategory = (typeof WikiCategory)[keyof typeof WikiCategory]

// A tab's identity: a route and its query, never the view's content (B6).
export interface TabLocation {
  name: RouteName
  query?: { category?: WikiCategory }
}

type Message = MessageKey<MessageSchema>

export const defaultLocation: TabLocation = { name: RouteName.NextSteps }

export const routePath: Record<RouteName, string> = {
  [RouteName.NextSteps]: '/progress/next-steps',
  [RouteName.Completion]: '/progress/completion',
  [RouteName.Unlock]: '/progress/unlock',
  [RouteName.Plan]: '/progress/plan',
  [RouteName.Collection]: '/progress/collection',
  [RouteName.Runs]: '/progress/runs',
  [RouteName.Live]: '/progress/live',
  [RouteName.Wiki]: '/wiki',
  [RouteName.Profile]: '/settings/profile',
  [RouteName.TabsSettings]: '/settings/tabs',
  [RouteName.About]: '/about',
}

export const routeTitle: Record<RouteName, Message> = {
  [RouteName.NextSteps]: 'routes.nextSteps',
  [RouteName.Completion]: 'routes.completion',
  [RouteName.Unlock]: 'routes.unlock',
  [RouteName.Plan]: 'routes.plan',
  [RouteName.Collection]: 'routes.collection',
  [RouteName.Runs]: 'routes.runs',
  [RouteName.Live]: 'routes.live',
  [RouteName.Wiki]: 'routes.wiki',
  [RouteName.Profile]: 'routes.profile',
  [RouteName.TabsSettings]: 'routes.tabsSettings',
  [RouteName.About]: 'routes.about',
}

export const routeOrigin: Record<RouteName, TabOrigin> = {
  [RouteName.NextSteps]: TabOrigin.Progress,
  [RouteName.Completion]: TabOrigin.Progress,
  [RouteName.Unlock]: TabOrigin.Progress,
  [RouteName.Plan]: TabOrigin.Progress,
  [RouteName.Collection]: TabOrigin.Progress,
  [RouteName.Runs]: TabOrigin.Progress,
  [RouteName.Live]: TabOrigin.Progress,
  [RouteName.Wiki]: TabOrigin.Wiki,
  [RouteName.Profile]: TabOrigin.Settings,
  [RouteName.TabsSettings]: TabOrigin.Settings,
  [RouteName.About]: TabOrigin.About,
}

export const routeIcon: Record<RouteName, Component> = {
  [RouteName.NextSteps]: ListChecksIcon,
  [RouteName.Completion]: Grid2x2Icon,
  [RouteName.Unlock]: LockOpenIcon,
  [RouteName.Plan]: MapIcon,
  [RouteName.Collection]: LayersIcon,
  [RouteName.Runs]: PlayIcon,
  [RouteName.Live]: ActivityIcon,
  [RouteName.Wiki]: tabOriginIcon[TabOrigin.Wiki],
  [RouteName.Profile]: SaveIcon,
  [RouteName.TabsSettings]: AppWindowIcon,
  [RouteName.About]: tabOriginIcon[TabOrigin.About],
}

// Which sub-project brings a screen that is still a placeholder; absent once it's real.
export const routeArrives: Partial<Record<RouteName, Message>> = {
  [RouteName.NextSteps]: 'placeholder.graph',
  [RouteName.Completion]: 'placeholder.completion',
  [RouteName.Unlock]: 'placeholder.graph',
  [RouteName.Plan]: 'placeholder.graph',
  [RouteName.Collection]: 'placeholder.collection',
  [RouteName.Runs]: 'placeholder.runArchive',
  [RouteName.Live]: 'placeholder.runArchive',
  [RouteName.Wiki]: 'placeholder.wiki',
  [RouteName.TabsSettings]: 'placeholder.tabs',
  [RouteName.About]: 'placeholder.settings',
}

export const wikiCategoryTitle: Record<WikiCategory, Message> = {
  [WikiCategory.Items]: 'wikiCategories.items',
  [WikiCategory.Trinkets]: 'wikiCategories.trinkets',
  [WikiCategory.Achievements]: 'wikiCategories.achievements',
  [WikiCategory.Bosses]: 'wikiCategories.bosses',
  [WikiCategory.Challenges]: 'wikiCategories.challenges',
  [WikiCategory.Characters]: 'wikiCategories.characters',
}

export const wikiCategoryIcon: Record<WikiCategory, Component> = {
  [WikiCategory.Items]: PackageIcon,
  [WikiCategory.Trinkets]: GemIcon,
  [WikiCategory.Achievements]: TrophyIcon,
  [WikiCategory.Bosses]: SkullIcon,
  [WikiCategory.Challenges]: FlagIcon,
  [WikiCategory.Characters]: UserIcon,
}

// A tab's label: a wiki category names itself, every other location is its route.
export const locationTitle = (location: TabLocation): Message => {
  const category = location.query?.category
  return location.name === RouteName.Wiki && category
    ? wikiCategoryTitle[category]
    : routeTitle[location.name]
}
```

- [ ] **Step 4: `ui/src/stores/tabModel.ts`**

```ts
import type { TabLocation } from '@/router/routeTable'

export interface Tab {
  id: string
  location: TabLocation
}

export interface TabsState {
  tabs: Tab[]
  activeId: string
}

// The tab bar's rules, pure: the store only holds the result.
export const firstState = (id: string, location: TabLocation): TabsState => ({
  tabs: [{ id, location }],
  activeId: id,
})

const indexOf = (state: TabsState, id: string): number =>
  state.tabs.findIndex((tab) => tab.id === id)

export const openTab = (
  state: TabsState,
  id: string,
  location: TabLocation,
): TabsState => {
  const at = indexOf(state, state.activeId) + 1
  return {
    tabs: [
      ...state.tabs.slice(0, at),
      { id, location },
      ...state.tabs.slice(at),
    ],
    activeId: id,
  }
}

export const selectTab = (state: TabsState, id: string): TabsState =>
  indexOf(state, id) < 0 ? state : { ...state, activeId: id }

// Closing the active tab moves to its right neighbour, or the left one when it was last;
// the bar is never empty, so closing the only tab puts a fresh one in its place.
export const closeTab = (
  state: TabsState,
  id: string,
  fresh: () => Tab,
): TabsState => {
  const index = indexOf(state, id)
  if (index < 0) return state
  const tabs = state.tabs.filter((tab) => tab.id !== id)
  if (tabs.length === 0) {
    const tab = fresh()
    return { tabs: [tab], activeId: tab.id }
  }
  if (id !== state.activeId) return { tabs, activeId: state.activeId }
  const next = tabs[Math.min(index, tabs.length - 1)]
  return next ? { tabs, activeId: next.id } : state
}

// `to` is the tab's final index, as TabStrip computes it with moveIndex.
export const moveTab = (
  state: TabsState,
  from: number,
  to: number,
): TabsState => {
  const tabs = [...state.tabs]
  const [tab] = tabs.splice(from, 1)
  if (!tab) return state
  tabs.splice(to, 0, tab)
  return { ...state, tabs }
}

export const navigateTab = (
  state: TabsState,
  location: TabLocation,
): TabsState => ({
  ...state,
  tabs: state.tabs.map((tab) =>
    tab.id === state.activeId ? { ...tab, location } : tab,
  ),
})
```

- [ ] **Step 5: `ui/src/lib/constants/stores.ts`**

```ts
// Pinia store ids.
export const StoreId = { Tabs: 'tabs', Profile: 'profile' } as const
export type StoreId = (typeof StoreId)[keyof typeof StoreId]
```

- [ ] **Step 6: `ui/src/stores/tabs.ts`**

```ts
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { defaultLocation } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import {
  closeTab,
  firstState,
  moveTab,
  navigateTab,
  openTab,
  selectTab,
} from './tabModel'
import type { Tab } from './tabModel'

// The open tabs, window-wide. The rules are tabModel's; this holds the result. Nothing is
// saved yet: tabs surviving a restart is sub-project 7.
export const useTabsStore = defineStore(StoreId.Tabs, () => {
  let counter = 0
  const nextId = (): string => `tab-${++counter}`
  const fresh = (): Tab => ({ id: nextId(), location: defaultLocation })

  const state = ref(firstState(nextId(), defaultLocation))

  const tabs = computed(() => state.value.tabs)
  const activeId = computed(() => state.value.activeId)
  const active = computed(() =>
    state.value.tabs.find((tab) => tab.id === state.value.activeId),
  )

  const open = (location: TabLocation = defaultLocation): void => {
    state.value = openTab(state.value, nextId(), location)
  }
  const select = (id: string): void => {
    state.value = selectTab(state.value, id)
  }
  const close = (id: string): void => {
    state.value = closeTab(state.value, id, fresh)
  }
  const move = (from: number, to: number): void => {
    state.value = moveTab(state.value, from, to)
  }
  const navigate = (location: TabLocation): void => {
    state.value = navigateTab(state.value, location)
  }

  return { tabs, activeId, active, open, select, close, move, navigate }
})
```

- [ ] **Step 7: Run the tests to see them pass** — `pnpm --filter ui exec vitest run src/stores src/router` → PASS (12 tests).

---

### Task 4: Sections and the sidebar entries

**Files:**
- Create: `ui/src/components/shell/sectionNav.ts`
- Test: `ui/src/components/shell/sectionNav.test.ts`

**Interfaces:**
- Consumes: Task 3's route table; cycle 2's `NavSection`, `TabOrigin`, `tabOriginIcon`.
- Produces: `SidebarSection`, `SidebarEntry`, `sidebarEntries`, `sidebarHeaders`, `sectionOfOrigin`, `navSectionOf`, `sidebarSectionOf`, `isEntryActive`.

- [ ] **Step 1: Write the failing test** — `ui/src/components/shell/sectionNav.test.ts`

```ts
import { describe, expect, it } from 'vitest'
import { RouteName, WikiCategory } from '@/router/routeTable'
import { NavSection } from './navSection'
import {
  SidebarSection,
  isEntryActive,
  navSectionOf,
  sectionOfOrigin,
  sidebarEntries,
  sidebarSectionOf,
} from './sectionNav'
import { TabOrigin } from './tabs'

describe('sectionNav', () => {
  it('puts About under Settings', () => {
    expect(sectionOfOrigin(TabOrigin.About)).toBe(SidebarSection.Settings)
  })

  it('marks no navbar section while browsing Settings', () => {
    expect(navSectionOf(SidebarSection.Settings)).toBeNull()
    expect(navSectionOf(SidebarSection.Wiki)).toBe(NavSection.Wiki)
    expect(sidebarSectionOf(NavSection.Progress)).toBe(SidebarSection.Progress)
  })

  it('lists the seven Progress screens', () => {
    expect(sidebarEntries[SidebarSection.Progress]).toHaveLength(7)
  })

  it('matches an entry by route', () => {
    const [nextSteps] = sidebarEntries[SidebarSection.Progress]
    expect(nextSteps && isEntryActive(nextSteps, { name: RouteName.NextSteps })).toBe(true)
    expect(nextSteps && isEntryActive(nextSteps, { name: RouteName.Plan })).toBe(false)
    expect(nextSteps && isEntryActive(nextSteps, undefined)).toBe(false)
  })

  it('matches a wiki entry by category', () => {
    const bosses = sidebarEntries[SidebarSection.Wiki].find(
      (e) => e.location.query?.category === WikiCategory.Bosses,
    )
    const at = (category: WikiCategory) => ({
      name: RouteName.Wiki,
      query: { category },
    })
    expect(bosses && isEntryActive(bosses, at(WikiCategory.Bosses))).toBe(true)
    expect(bosses && isEntryActive(bosses, at(WikiCategory.Items))).toBe(false)
  })
})
```

- [ ] **Step 2: Run it to see it fail** — `pnpm --filter ui exec vitest run src/components/shell/sectionNav` → FAIL.

- [ ] **Step 3: `ui/src/components/shell/sectionNav.ts`**

```ts
import type { Component } from 'vue'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { assertNever } from '@/lib/assertNever'
import {
  RouteName,
  WikiCategory,
  routeIcon,
  routeTitle,
  wikiCategoryIcon,
  wikiCategoryTitle,
} from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import { NavSection } from './navSection'
import { TabOrigin } from './tabs'
import { tabOriginIcon } from './tabOriginIcon'

type Message = MessageKey<MessageSchema>

// The sidebar shows one section at a time (Schermate.dc.html): the two navbar sections,
// plus Settings, reached from the cog.
export const SidebarSection = {
  Wiki: 'wiki',
  Progress: 'progress',
  Settings: 'settings',
} as const
export type SidebarSection = (typeof SidebarSection)[keyof typeof SidebarSection]

export interface SidebarEntry {
  key: string
  location: TabLocation
  label: Message
  icon: Component
}

export interface SidebarHeader {
  title: Message
  hint: Message
  icon: Component
}

const routeEntry = (name: RouteName): SidebarEntry => ({
  key: name,
  location: { name },
  label: routeTitle[name],
  icon: routeIcon[name],
})

const wikiEntry = (category: WikiCategory): SidebarEntry => ({
  key: `${RouteName.Wiki}-${category}`,
  location: { name: RouteName.Wiki, query: { category } },
  label: wikiCategoryTitle[category],
  icon: wikiCategoryIcon[category],
})

export const sidebarEntries: Record<SidebarSection, SidebarEntry[]> = {
  [SidebarSection.Progress]: [
    RouteName.NextSteps,
    RouteName.Completion,
    RouteName.Unlock,
    RouteName.Plan,
    RouteName.Collection,
    RouteName.Runs,
    RouteName.Live,
  ].map(routeEntry),
  [SidebarSection.Wiki]: Object.values(WikiCategory).map(wikiEntry),
  [SidebarSection.Settings]: [RouteName.Profile, RouteName.TabsSettings].map(
    routeEntry,
  ),
}

export const sidebarHeaders: Record<SidebarSection, SidebarHeader> = {
  [SidebarSection.Progress]: {
    title: 'sidebar.progressTitle',
    hint: 'sidebar.progressHint',
    icon: tabOriginIcon[TabOrigin.Progress],
  },
  [SidebarSection.Wiki]: {
    title: 'sidebar.wikiTitle',
    hint: 'sidebar.wikiHint',
    icon: tabOriginIcon[TabOrigin.Wiki],
  },
  [SidebarSection.Settings]: {
    title: 'sidebar.settingsTitle',
    hint: 'sidebar.settingsHint',
    icon: tabOriginIcon[TabOrigin.Settings],
  },
}

export const sectionOfOrigin = (origin: TabOrigin): SidebarSection => {
  switch (origin) {
    case TabOrigin.Wiki:
      return SidebarSection.Wiki
    case TabOrigin.Progress:
      return SidebarSection.Progress
    case TabOrigin.Settings:
    case TabOrigin.About:
      return SidebarSection.Settings
    default:
      return assertNever(origin)
  }
}

// The navbar marks Wiki or Progress; while Settings is shown it marks neither.
export const navSectionOf = (section: SidebarSection): NavSection | null => {
  switch (section) {
    case SidebarSection.Wiki:
      return NavSection.Wiki
    case SidebarSection.Progress:
      return NavSection.Progress
    case SidebarSection.Settings:
      return null
    default:
      return assertNever(section)
  }
}

export const sidebarSectionOf = (section: NavSection): SidebarSection => {
  switch (section) {
    case NavSection.Wiki:
      return SidebarSection.Wiki
    case NavSection.Progress:
      return SidebarSection.Progress
    default:
      return assertNever(section)
  }
}

export const isEntryActive = (
  entry: SidebarEntry,
  location: TabLocation | undefined,
): boolean =>
  location !== undefined &&
  entry.location.name === location.name &&
  (entry.location.query?.category ?? null) ===
    (location.query?.category ?? null)
```

- [ ] **Step 4: Run it to see it pass** — PASS (5 tests).

---

### Task 5: What the profile screen shows, and the profile store

**Files:**
- Create: `ui/src/lib/profile/profileView.ts`, `ui/src/lib/profile/profileLabels.ts`, `ui/src/stores/profile.ts`
- Test: `ui/src/lib/profile/profileView.test.ts`

**Interfaces:**
- Produces: `ChainLink`, `LinkState`, `ChainRow`, `chainLinks(setup)`, `formatRelativeDay(unix, now, locale)`, `formatModified(unix, now, locale)`, `formatCount(n, locale)`, `editionShort(prefix)`, `editionLong(prefix)`, `gameName(value)`, `indicator(active, now, locale)`, `IndicatorView`, `sectionLabel(kind)`; `candidateSourceLabel`; `useProfileStore` with `setup`, `summary`, `status`, `error`, `isActive`, `load()`, `choose(id)`; `LoadStatus`.

- [ ] **Step 1: Write the failing test** — `ui/src/lib/profile/profileView.test.ts`

```ts
import { describe, expect, it } from 'vitest'
import { Locale } from '@/i18n/locale'
import { candidates, noneSetup, setupWith } from '@/lib/ipc/fixtures/profile'
import { MissingReason, SavePrefix } from '@/lib/ipc/types'
import type { SetupState } from '@/lib/ipc/types'
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
const first = candidates[0]!

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
    const setup = setupWith({ kind: 'active', profile: first, autoSelected: false })
    expect(chainLinks(setup)[2]).toEqual({
      link: 'saves',
      state: LinkState.Chosen,
      detail: { kind: 'profile', candidate: first },
    })
  })
})

describe('formatModified', () => {
  it('says today, yesterday and the day before in words', () => {
    expect(formatModified(unixAt(new Date(2026, 8, 2, 8)), now, Locale.It)).toBe(
      'oggi · 2 set 2026',
    )
    expect(formatModified(unixAt(new Date(2026, 8, 1, 8)), now, Locale.It)).toBe(
      'ieri · 1 set 2026',
    )
    expect(formatModified(unixAt(new Date(2026, 7, 31, 10)), now, Locale.It)).toBe(
      'l’altro ieri · 31 ago 2026',
    )
  })

  it('counts days beyond that', () => {
    expect(formatModified(unixAt(new Date(2026, 7, 30, 10)), now, Locale.It)).toBe(
      '3 giorni fa · 30 ago 2026',
    )
  })

  it('speaks English too', () => {
    expect(formatModified(unixAt(new Date(2026, 7, 31, 10)), now, Locale.En)).toBe(
      '2 days ago · Aug 31, 2026',
    )
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
    const profile = { ...first, modifiedUnix: unixAt(new Date(2026, 7, 31, 10)) }
    expect(
      indicator({ kind: 'active', profile, autoSelected: false }, now, Locale.It),
    ).toEqual({ kind: 'active', edition: 'Rep+', slot: 1, modified: 'l’altro ieri' })
  })

  it('has no profile while a choice is pending', () => {
    expect(
      indicator(
        { kind: 'needsChoice', reason: { kind: 'neverChosen' }, suggested: null },
        now,
        Locale.It,
      ),
    ).toEqual({ kind: 'noProfile' })
  })

  it('has found nothing when there are no saves', () => {
    expect(indicator(noneSetup.active, now, Locale.It)).toEqual({ kind: 'notFound' })
  })
})

describe('sectionLabel', () => {
  it('has a message for a known section and none for an unknown one', () => {
    expect(sectionLabel('achievements')).toBe('profile.sections.achievements')
    expect(sectionLabel('unknown5')).toBe('profile.sections.unknown')
    expect(sectionLabel('something_new')).toBeNull()
  })
})
```

- [ ] **Step 2: Run it to see it fail** — `pnpm --filter ui exec vitest run src/lib/profile` → FAIL.

- [ ] **Step 3: `ui/src/lib/profile/profileView.ts`**

```ts
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import type { Locale } from '@/i18n/locale'
import { assertNever } from '@/lib/assertNever'
import { SavePrefix } from '@/lib/ipc/types'
import type {
  ActiveProfile,
  CandidateView,
  SetupState,
} from '@/lib/ipc/types'

type Message = MessageKey<MessageSchema>

// The chain the profile screen draws (Schermate.dc.html): each link can be missing alone.
export const ChainLink = { Steam: 'steam', Game: 'game', Saves: 'saves' } as const
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
  unknown8: 'profile.sections.unknown',
  unknown9: 'profile.sections.unknown',
  bestiary: 'profile.sections.bestiary',
}
export const sectionLabel = (kind: string): Message | null =>
  sectionLabels[kind] ?? null
```

- [ ] **Step 4: `ui/src/lib/profile/profileLabels.ts`**

```ts
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { CandidateSource, MissingReason } from '@/lib/ipc/types'
import { ChainLink, LinkState } from './profileView'

type Message = MessageKey<MessageSchema>

// Records over closed sets: a new value with no message fails to compile.
export const candidateSourceLabel: Record<CandidateSource, Message> = {
  [CandidateSource.SteamCloud]: 'profile.sources.steamCloud',
  [CandidateSource.Documents]: 'profile.sources.documents',
  [CandidateSource.Manual]: 'profile.sources.manual',
}

export const missingReasonLabel: Record<MissingReason, Message> = {
  [MissingReason.SteamNotFound]: 'profile.none.steamNotFound',
  [MissingReason.GameNotFound]: 'profile.none.gameNotFound',
  [MissingReason.NoSaves]: 'profile.none.noSaves',
}

export const chainLinkLabel: Record<ChainLink, Message> = {
  [ChainLink.Steam]: 'profile.chain.steam',
  [ChainLink.Game]: 'profile.chain.game',
  [ChainLink.Saves]: 'profile.chain.saves',
}

export const linkStateLabel: Record<LinkState, Message> = {
  [LinkState.Found]: 'profile.chain.found',
  [LinkState.Missing]: 'profile.chain.missing',
  [LinkState.YourChoice]: 'profile.chain.yourChoice',
  [LinkState.Several]: 'profile.chain.several',
  [LinkState.Chosen]: 'profile.chain.chosen',
}
```

- [ ] **Step 5: `ui/src/stores/profile.ts`**

```ts
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { saveSummary } from '@/lib/ipc/save'
import { selectProfile, setupState } from '@/lib/ipc/setup'
import type { IpcError, SaveSummary, SetupState } from '@/lib/ipc/types'

export const LoadStatus = {
  Idle: 'idle',
  Loading: 'loading',
  Ready: 'ready',
  Failed: 'failed',
} as const
export type LoadStatus = (typeof LoadStatus)[keyof typeof LoadStatus]

const isIpcError = (e: unknown): e is IpcError =>
  typeof e === 'object' && e !== null && 'kind' in e

// The active profile is the window's, never a tab's (DESIGN-BRIEF.md §4.1, §4.2): one store,
// read by the indicator, the gate and the profile screen alike.
export const useProfileStore = defineStore(StoreId.Profile, () => {
  const setup = ref<SetupState | null>(null)
  const summary = ref<SaveSummary | null>(null)
  const status = ref<LoadStatus>(LoadStatus.Idle)
  const error = ref<IpcError | null>(null)

  const isActive = computed(() => setup.value?.active.kind === 'active')

  // No active profile means no summary, not an error.
  const readSummary = async (): Promise<void> => {
    if (!isActive.value) {
      summary.value = null
      return
    }
    try {
      summary.value = await saveSummary()
    } catch (e) {
      if (isIpcError(e) && e.kind === 'noActiveProfile') summary.value = null
      else throw e
    }
  }

  const run = async (read: () => Promise<SetupState>): Promise<void> => {
    status.value = LoadStatus.Loading
    error.value = null
    try {
      setup.value = await read()
      await readSummary()
      status.value = LoadStatus.Ready
    } catch (e) {
      error.value = isIpcError(e) ? e : null
      status.value = LoadStatus.Failed
    }
  }

  const load = (): Promise<void> => run(setupState)
  const choose = (id: string): Promise<void> => run(() => selectProfile(id))

  return { setup, summary, status, error, isActive, load, choose }
})
```

- [ ] **Step 6: Run the tests to see them pass** — `pnpm --filter ui exec vitest run src/lib/profile` → PASS (17 tests).

- [ ] **Step 7: Commit Tasks 3–5** (typecheck still waits on Task 7's `main.ts`, so only Vitest here)

```bash
pnpm --filter ui exec vitest run
git add ui/src/router/routeTable.ts ui/src/router/routeTable.test.ts ui/src/stores ui/src/lib/constants/stores.ts ui/src/components/shell/sectionNav.ts ui/src/components/shell/sectionNav.test.ts ui/src/lib/profile
git commit -m "feat(ui): the tab model, the sidebar's sections and what the profile screen shows"
```

---

### Task 6: Messages, and the profile screen

**Files:**
- Modify: `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts`
- Create: `ui/src/screens/ScreenHeader.vue`, `ui/src/screens/PlaceholderScreen.vue`, `ui/src/screens/ProgressGate.vue`, `ui/src/screens/ProfileScreen.vue`, `ui/src/screens/profile/ChainCard.vue`, `NoSavesCard.vue`, `CandidatesCard.vue`, `ActiveProfileCard.vue`, `ProfileFact.vue`, `SectionsCard.vue`, `ProfileError.vue`

- [ ] **Step 1: Messages** — add to `it` (after `marks`) and the matching English in `en`:

```ts
  routes: {
    nextSteps: 'Prossimi passi',
    completion: 'Completamento',
    unlock: 'Unlock',
    plan: 'Piano',
    collection: 'Collezione',
    runs: 'Run',
    live: 'Live',
    wiki: 'Wiki',
    profile: 'Profilo di gioco',
    tabsSettings: 'Tab',
    about: 'Informazioni',
  },
  wikiCategories: {
    items: 'Oggetti',
    trinkets: 'Trinket',
    achievements: 'Achievement',
    bosses: 'Boss',
    challenges: 'Sfide',
    characters: 'Personaggi',
  },
  sidebar: {
    progressTitle: 'Progressi',
    progressHint: 'Ogni voce si legge sul profilo attivo.',
    wikiTitle: 'Wiki',
    wikiHint: 'La Wiki funziona senza gioco né salvataggio.',
    settingsTitle: 'Impostazioni',
    settingsHint: "Da qui l'app trova gioco e salvataggi.",
  },
  placeholder: {
    graph: 'Arriva con Prossimi passi, Unlock e Piano.',
    completion: 'Arriva con la schermata Completamento.',
    collection: 'Arriva con la schermata Collezione.',
    runArchive: "Arriva con l'archivio delle run (M4).",
    wiki: 'Arriva con le pagine wiki nelle tab e la ricerca.',
    settings: 'Arriva con Impostazioni e Informazioni.',
    tabs: 'Arriva con le tab che sopravvivono alla chiusura.',
  },
  gate: {
    needsProfile:
      'Progressi dipende dal profilo attivo: scegline uno per vedere questa schermata.',
  },
  indicator: {
    noProfile: 'Nessun profilo attivo',
    notFound: 'Salvataggi non trovati',
    slot: 'slot',
  },
  profile: {
    eyebrow: 'Schermata 0',
    title: 'Profilo di gioco',
    intro:
      "Non è un passaggio da attraversare una volta: è lo stato che decide ogni numero dell'app. Resta consultabile e modificabile per sempre.",
    chain: {
      title: 'La catena dei tre requisiti',
      summary: 'ogni livello può mancare da solo',
      steam: 'Steam',
      game: 'Gioco',
      saves: 'Salvataggi',
      found: 'trovato',
      missing: 'non trovato',
      yourChoice: 'scegli tu',
      several: 'più di uno',
      chosen: 'scelto',
      candidates: 'file candidati',
    },
    none: {
      title: 'Nessun salvataggio trovato',
      steamNotFound:
        'Steam non è nel registro di sistema: senza Steam non troviamo la cartella del gioco, e senza quella non troviamo i salvataggi.',
      gameNotFound:
        'Steam c’è, ma il gioco non è in nessuna delle sue librerie: senza la cartella del gioco non troviamo i salvataggi.',
      noSaves:
        'Il gioco c’è, ma nelle posizioni note non c’è nessun file di salvataggio.',
      retry: 'Riprova la ricerca',
      diagnostics: 'Diagnostica — cosa abbiamo provato',
    },
    diagnostics: {
      steamNotFound: 'Steam: nessuna installazione trovata',
      gameNotFound: 'Gioco: non presente nelle librerie di Steam',
      noSavesFound: 'Salvataggi: nessun file nelle posizioni note',
      unreadablePath: 'Percorso non leggibile',
      malformedManifest: 'Manifest di Steam non leggibile',
    },
    pick: {
      title: 'Serve scegliere',
      summary: 'finché non scegli, nessun profilo è attivo',
      savedGone:
        'Il profilo che usavi non esiste più dove stava. Non ne abbiamo scelto un altro al suo posto: i numeri di un profilo diverso, mostrati senza dirlo, sono l’errore che non ti accorgi di avere.',
      edition: 'Edizione',
      slot: 'Slot',
      foundIn: 'Trovato qui',
      modified: 'Modificato',
      size: 'Dimensione',
      suggested: 'più recente',
      hint: 'Il più recente è solo un suggerimento: scegli tu quale profilo leggere.',
      use: 'Usa questo profilo',
      cancel: 'Annulla',
    },
    sources: {
      steamCloud: 'Steam Cloud',
      documents: 'Documenti',
      manual: 'Scelto a mano',
    },
    active: {
      title: 'Profilo attivo',
      autoSelected: 'scelto da noi · era l’unico',
      modified: 'Modificato',
      size: 'Dimensione',
      dlcs: 'DLC',
      foundIn: 'Trovato in',
      change: 'Cambia profilo',
      reload: 'Rileggi il file',
      unknownDate: 'sconosciuto',
      bytes: 'byte',
    },
    read: {
      title: 'Cosa siamo riusciti a leggere',
      sections: 'sezioni',
      note: 'Alcune sezioni non sappiamo ancora cosa contengano, e i conteggi cambiano a ogni patch: nessun numero è cablato, né qui né nel resto dell’app. Le schermate mostrano quello che il file dichiara oggi.',
      diagnostics: 'Il file contiene qualcosa che non ci aspettavamo',
    },
    saveDiagnostics: {
      unexpectedKind: 'Una sezione non è quella attesa',
      sectionOverrun: 'Una sezione va oltre la fine del file',
      trailingBytes: 'Ci sono byte in più alla fine del file',
    },
    sections: {
      achievements: 'Achievement e segreti',
      counters: 'Contatori e marchi',
      levelCounters: 'Contatori per piano',
      items: 'Collezione oggetti',
      bosses: 'Boss incontrati',
      challenges: 'Sfide',
      bestiary: 'Bestiario',
      unknown: 'Da identificare',
    },
    errors: {
      title: 'Non riusciamo a leggere il profilo',
      retry: 'Riprova',
      noBackend: 'Il backend non ha risposto.',
      noActiveProfile: 'Nessun profilo attivo.',
      unknownProfile: 'Il profilo scelto non esiste più.',
      unreadableSave: 'Il salvataggio non si legge.',
      settingsNotWritable: 'Non riusciamo a ricordare la scelta.',
      unknownTarget: 'Quello che cerchi non esiste.',
      catalogUnavailable: 'Il catalogo del gioco non è disponibile.',
      storeUnavailable: "Il database dell'app non è disponibile.",
      wikiUnavailable: 'Il dataset della wiki non è disponibile.',
    },
  },
```

English (`en.ts`), same keys: routes — `Next steps`, `Completion`, `Unlock`, `Plan`, `Collection`, `Runs`, `Live`, `Wiki`, `Game profile`, `Tabs`, `About`; wikiCategories — `Items`, `Trinkets`, `Achievements`, `Bosses`, `Challenges`, `Characters`; sidebar — `Progress` / `Every entry reads the active profile.`, `Wiki` / `The Wiki works without the game or a save.`, `Settings` / `Where the app finds the game and the saves.`; placeholder — `Arrives with Next steps, Unlock and Plan.`, `Arrives with the Completion screen.`, `Arrives with the Collection screen.`, `Arrives with the run archive (M4).`, `Arrives with wiki pages in tabs and search.`, `Arrives with Settings and About.`, `Arrives with tabs that survive closing.`; gate — `Progress depends on the active profile: choose one to see this screen.`; indicator — `No active profile`, `Saves not found`, `slot`; profile — eyebrow `Screen 0`, title `Game profile`, intro `Not a step to go through once: it is the state that decides every number in the app. It stays open to read and change.`; chain — `The chain of three requirements`, `each level can be missing on its own`, `Steam`, `Game`, `Saves`, `found`, `not found`, `your choice`, `more than one`, `chosen`, `candidate files`; none — `No save found`, `Steam isn't in the system registry: without Steam we can't find the game folder, and without that we can't find the saves.`, `Steam is there, but the game isn't in any of its libraries: without the game folder we can't find the saves.`, `The game is there, but there is no save file in the known places.`, `Search again`, `Diagnostics — what we tried`; diagnostics — `Steam: no installation found`, `Game: not in Steam's libraries`, `Saves: no file in the known places`, `Unreadable path`, `Unreadable Steam manifest`; pick — `A choice is needed`, `until you choose, no profile is active`, `The profile you used no longer exists where it was. We didn't pick another in its place: a different profile's numbers, shown without saying so, are the error you never notice you have.`, `Edition`, `Slot`, `Found here`, `Modified`, `Size`, `most recent`, `The most recent is only a suggestion: you choose which profile to read.`, `Use this profile`, `Cancel`; sources — `Steam Cloud`, `Documents`, `Chosen by hand`; active — `Active profile`, `chosen by us · it was the only one`, `Modified`, `Size`, `DLC`, `Found in`, `Change profile`, `Read the file again`, `unknown`, `bytes`; read — `What we could read`, `sections`, `Some sections we don't know the content of yet, and the counts change with every patch: no number is hardcoded, here or anywhere in the app. The screens show what the file declares today.`, `The file holds something we didn't expect`; saveDiagnostics — `A section isn't the expected one`, `A section runs past the end of the file`, `There are extra bytes at the end of the file`; sections — `Achievements and secrets`, `Counters and marks`, `Per-floor counters`, `Item collection`, `Bosses met`, `Challenges`, `Bestiary`, `To identify`; errors — `We can't read the profile`, `Try again`, `The backend didn't answer.`, `No active profile.`, `The chosen profile no longer exists.`, `The save can't be read.`, `We can't remember the choice.`, `What you're looking for doesn't exist.`, `The game's catalogue isn't available.`, `The app's database isn't available.`, `The wiki dataset isn't available.`

- [ ] **Step 2: `ui/src/screens/ScreenHeader.vue`**

```vue
<script setup lang="ts">
import type { Component } from 'vue'

defineProps<{ icon: Component; title: string; eyebrow?: string }>()
</script>

<template>
  <header class="flex flex-col gap-2">
    <span v-if="eyebrow" class="text-caption text-subtle-foreground">{{
      eyebrow
    }}</span>
    <div class="flex items-center gap-2.75">
      <component :is="icon" class="size-6 shrink-0 text-foreground-soft" />
      <h1 class="text-title text-foreground">{{ title }}</h1>
    </div>
    <p v-if="$slots.default" class="max-w-155 text-row text-foreground-soft">
      <slot />
    </p>
  </header>
</template>
```

- [ ] **Step 3: `ui/src/screens/PlaceholderScreen.vue`**

```vue
<script setup lang="ts">
import { useRoute } from 'vue-router'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import { useMessages } from '@/i18n'
import ScreenHeader from './ScreenHeader.vue'

const route = useRoute()
const { t } = useMessages()
</script>

<template>
  <!-- A screen that isn't built yet says which sub-project brings it: honest, and removed
       screen by screen. -->
  <div class="flex max-w-250 flex-col gap-4">
    <ScreenHeader :icon="route.meta.icon" :title="t(route.meta.title)" />
    <EmptyCategory v-if="route.meta.arrives">{{
      t(route.meta.arrives)
    }}</EmptyCategory>
  </div>
</template>
```

- [ ] **Step 4: `ui/src/screens/ProgressGate.vue`**

```vue
<script setup lang="ts">
import { InfoIcon } from '@lucide/vue'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { useMessages } from '@/i18n'
import { useProfileStore } from '@/stores/profile'
import ProfileScreen from './ProfileScreen.vue'

const profile = useProfileStore()
const { t } = useMessages()
</script>

<template>
  <!-- DESIGN-BRIEF.md §4: Progress, until a choice is made, is the profile selection. The
       tab keeps its own name; the line above says why it shows this. -->
  <slot v-if="profile.isActive" />
  <div v-else class="flex flex-col gap-4">
    <Alert>
      <InfoIcon />
      <AlertDescription>{{ t('gate.needsProfile') }}</AlertDescription>
    </Alert>
    <ProfileScreen />
  </div>
</template>
```

- [ ] **Step 5: `ui/src/screens/profile/ChainCard.vue`**

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import {
  CardCollapsible,
  CardCollapsibleContent,
  CardCollapsibleTrigger,
} from '@/components/ui/card'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import type { SetupState } from '@/lib/ipc/types'
import { chainLinkLabel, linkStateLabel } from '@/lib/profile/profileLabels'
import { LinkState, chainLinks, editionShort } from '@/lib/profile/profileView'
import type { ChainRow } from '@/lib/profile/profileView'

const props = defineProps<{ setup: SetupState }>()
const { t } = useMessages()

const rows = computed(() => chainLinks(props.setup))

// A state is its badge's shape too: found and chosen are done, missing is an alert, the two
// that ask something of the user are neutral tags (cyan, the export's choice, is focus).
const badge: Record<LinkState, BadgeVariant> = {
  [LinkState.Found]: BadgeVariant.Done,
  [LinkState.Chosen]: BadgeVariant.Done,
  [LinkState.Missing]: BadgeVariant.Unexpected,
  [LinkState.YourChoice]: BadgeVariant.Tag,
  [LinkState.Several]: BadgeVariant.Tag,
}

const detail = (row: ChainRow): string => {
  const d = row.detail
  switch (d.kind) {
    case 'hint':
      return d.hint
    case 'count':
      return `${d.n} ${t('profile.chain.candidates')}`
    case 'profile':
      return `${editionShort(d.candidate.prefix)} · ${t('indicator.slot')} ${d.candidate.slot}`
    case 'none':
      return ''
    default:
      return assertNever(d)
  }
}
</script>

<template>
  <CardCollapsible :default-open="true">
    <CardCollapsibleTrigger>
      {{ t('profile.chain.title') }}
      <template #summary>{{ t('profile.chain.summary') }}</template>
    </CardCollapsibleTrigger>
    <CardCollapsibleContent class="flex flex-col gap-2">
      <div
        v-for="row in rows"
        :key="row.link"
        class="flex items-center gap-3 border border-hairline bg-data px-2.75 py-2.25"
      >
        <span class="w-30 shrink-0 text-row text-foreground">{{
          t(chainLinkLabel[row.link])
        }}</span>
        <span class="min-w-0 flex-1 truncate text-caption text-foreground-soft">{{
          detail(row)
        }}</span>
        <Badge :variant="badge[row.state]">{{
          t(linkStateLabel[row.state])
        }}</Badge>
      </div>
    </CardCollapsibleContent>
  </CardCollapsible>
</template>
```

- [ ] **Step 6: `ui/src/screens/profile/NoSavesCard.vue`**

```vue
<script setup lang="ts">
import { RefreshCwIcon, TriangleAlertIcon } from '@lucide/vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import { Button, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import type { MissingReason, SetupDiagnostic } from '@/lib/ipc/types'
import { missingReasonLabel } from '@/lib/profile/profileLabels'

defineProps<{ reason: MissingReason; diagnostics: SetupDiagnostic[] }>()
const emit = defineEmits<{ retry: [] }>()
const { t } = useMessages()

const diagnosticText = (d: SetupDiagnostic): string => {
  switch (d.kind) {
    case 'steamNotFound':
      return t('profile.diagnostics.steamNotFound')
    case 'gameNotFound':
      return t('profile.diagnostics.gameNotFound')
    case 'noSavesFound':
      return t('profile.diagnostics.noSavesFound')
    case 'unreadablePath':
      return `${t('profile.diagnostics.unreadablePath')} · ${d.name} · ${d.reason}`
    case 'malformedManifest':
      return `${t('profile.diagnostics.malformedManifest')} · ${d.name}`
    default:
      return assertNever(d)
  }
}
</script>

<template>
  <!-- The chain broke: say where, and what we tried. Choosing a folder by hand needs a
       command that accepts a path; until then there is no button that does nothing. -->
  <div class="flex flex-col gap-3">
    <Alert :variant="AlertVariant.Destructive">
      <TriangleAlertIcon />
      <AlertTitle>{{ t('profile.none.title') }}</AlertTitle>
      <AlertDescription>{{ t(missingReasonLabel[reason]) }}</AlertDescription>
    </Alert>
    <div class="flex flex-col gap-3 border border-border bg-sheet p-3">
      <div>
        <Button :variant="ButtonVariant.Outline" @click="emit('retry')">
          <RefreshCwIcon />{{ t('profile.none.retry') }}
        </Button>
      </div>
      <div
        v-if="diagnostics.length"
        class="flex flex-col gap-1 border border-hairline bg-data px-3 py-2.5"
      >
        <span class="text-label text-subtle-foreground">{{
          t('profile.none.diagnostics')
        }}</span>
        <span
          v-for="(d, i) in diagnostics"
          :key="i"
          class="text-caption text-foreground-soft"
          >{{ diagnosticText(d) }}</span
        >
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 7: `ui/src/screens/profile/CandidatesCard.vue`**

```vue
<script setup lang="ts">
import { CheckIcon, TriangleAlertIcon } from '@lucide/vue'
import { computed, ref } from 'vue'
import { Alert, AlertDescription, AlertVariant } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button, ButtonVariant } from '@/components/ui/button'
import {
  CardCollapsible,
  CardCollapsibleContent,
  CardCollapsibleTrigger,
} from '@/components/ui/card'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { i18n, useMessages } from '@/i18n'
import type { ActiveProfile, CandidateView } from '@/lib/ipc/types'
import { candidateSourceLabel } from '@/lib/profile/profileLabels'
import {
  editionShort,
  formatCount,
  formatModified,
} from '@/lib/profile/profileView'

const props = defineProps<{
  active: ActiveProfile
  candidates: CandidateView[]
  busy: boolean
}>()
const emit = defineEmits<{ choose: [id: string]; cancel: [] }>()
const { t } = useMessages()

// Nothing is preselected while a choice is pending: the suggestion stays a suggestion
// (§4.1). When changing an active profile, the current one starts selected.
const selected = ref<string | undefined>(
  props.active.kind === 'active' ? props.active.profile.id : undefined,
)

const savedGone = computed(
  () =>
    props.active.kind === 'needsChoice' &&
    props.active.reason.kind === 'savedProfileGone',
)

const rows = computed(() => {
  const now = new Date()
  const locale = i18n.global.locale.value
  return props.candidates.map((c) => ({
    ...c,
    edition: editionShort(c.prefix),
    modified: formatModified(c.modifiedUnix, now, locale),
    size: formatCount(c.sizeBytes, locale),
  }))
})

const confirm = () => {
  if (selected.value) emit('choose', selected.value)
}
</script>

<template>
  <CardCollapsible :default-open="true">
    <CardCollapsibleTrigger>
      {{ t('profile.pick.title') }} · {{ candidates.length }}
      <template #summary>{{ t('profile.pick.summary') }}</template>
    </CardCollapsibleTrigger>
    <CardCollapsibleContent class="flex flex-col gap-3">
      <Alert v-if="savedGone" :variant="AlertVariant.Destructive">
        <TriangleAlertIcon />
        <AlertDescription>{{ t('profile.pick.savedGone') }}</AlertDescription>
      </Alert>
      <RadioGroup v-model="selected" class="block">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead class="w-9" />
              <TableHead>{{ t('profile.pick.edition') }}</TableHead>
              <TableHead>{{ t('profile.pick.slot') }}</TableHead>
              <TableHead>{{ t('profile.pick.foundIn') }}</TableHead>
              <TableHead>{{ t('profile.pick.modified') }}</TableHead>
              <TableHead class="text-right">{{
                t('profile.pick.size')
              }}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow
              v-for="row in rows"
              :key="row.id"
              class="cursor-pointer"
              @click="selected = row.id"
            >
              <TableCell>
                <RadioGroupItem
                  :value="row.id"
                  :aria-label="`${row.edition} ${row.slot}`"
                />
              </TableCell>
              <TableCell>{{ row.edition }}</TableCell>
              <TableCell>{{ row.slot }}</TableCell>
              <TableCell>
                <span class="flex items-center gap-2">
                  <span class="text-caption text-subtle-foreground">{{
                    t(candidateSourceLabel[row.source])
                  }}</span>
                  <Badge v-if="row.suggested">{{
                    t('profile.pick.suggested')
                  }}</Badge>
                </span>
              </TableCell>
              <TableCell class="text-foreground-soft">{{
                row.modified ?? t('profile.active.unknownDate')
              }}</TableCell>
              <TableCell class="text-right text-foreground-soft"
                >{{ row.size }} {{ t('profile.active.bytes') }}</TableCell
              >
            </TableRow>
          </TableBody>
        </Table>
      </RadioGroup>
      <div class="flex flex-wrap items-center justify-between gap-3">
        <span class="text-caption text-subtle-foreground">{{
          t('profile.pick.hint')
        }}</span>
        <div class="flex gap-2">
          <Button
            v-if="active.kind === 'active'"
            :variant="ButtonVariant.Outline"
            @click="emit('cancel')"
            >{{ t('profile.pick.cancel') }}</Button
          >
          <Button :disabled="!selected || busy" @click="confirm">
            <CheckIcon />{{ t('profile.pick.use') }}
          </Button>
        </div>
      </div>
    </CardCollapsibleContent>
  </CardCollapsible>
</template>
```

- [ ] **Step 8: `ui/src/screens/profile/ProfileFact.vue`**

```vue
<script setup lang="ts">
defineProps<{ label: string; value: string }>()
</script>

<template>
  <div class="flex flex-col gap-0.75">
    <span class="text-label text-subtle-foreground">{{ label }}</span>
    <span class="text-row text-foreground">{{ value }}</span>
  </div>
</template>
```

- [ ] **Step 9: `ui/src/screens/profile/ActiveProfileCard.vue`**

```vue
<script setup lang="ts">
import { RefreshCwIcon, SaveIcon } from '@lucide/vue'
import { computed } from 'vue'
import { Badge } from '@/components/ui/badge'
import { Button, ButtonVariant } from '@/components/ui/button'
import {
  CardCollapsible,
  CardCollapsibleContent,
  CardCollapsibleTrigger,
} from '@/components/ui/card'
import { i18n, useMessages } from '@/i18n'
import type { ActiveProfile, SetupState } from '@/lib/ipc/types'
import { candidateSourceLabel } from '@/lib/profile/profileLabels'
import {
  editionLong,
  editionShort,
  formatCount,
  formatModified,
  gameName,
} from '@/lib/profile/profileView'
import ProfileFact from './ProfileFact.vue'

const props = defineProps<{
  active: Extract<ActiveProfile, { kind: 'active' }>
  game: SetupState['game']
}>()
const emit = defineEmits<{ change: []; reload: [] }>()
const { t } = useMessages()

const view = computed(() => {
  const locale = i18n.global.locale.value
  const profile = props.active.profile
  return {
    short: editionShort(profile.prefix),
    title: `${editionLong(profile.prefix)} · ${t('indicator.slot')} ${profile.slot}`,
    modified:
      formatModified(profile.modifiedUnix, new Date(), locale) ??
      t('profile.active.unknownDate'),
    size: `${formatCount(profile.sizeBytes, locale)} ${t('profile.active.bytes')}`,
    dlcs: props.game?.dlcs.map(gameName).join(' · ') ?? '',
    source: t(candidateSourceLabel[profile.source]),
  }
})
</script>

<template>
  <CardCollapsible :default-open="true">
    <CardCollapsibleTrigger>
      {{ t('profile.active.title') }}
      <template #summary>
        <Badge v-if="active.autoSelected">{{
          t('profile.active.autoSelected')
        }}</Badge>
      </template>
    </CardCollapsibleTrigger>
    <CardCollapsibleContent class="flex flex-wrap items-start gap-4.5">
      <div
        class="grid size-13 shrink-0 place-items-center border border-border bg-data text-label text-subtle-foreground"
      >
        {{ view.short }}
      </div>
      <div class="flex min-w-0 flex-1 flex-col gap-2.75">
        <span class="text-heading text-foreground">{{ view.title }}</span>
        <div class="flex flex-wrap gap-5">
          <ProfileFact :label="t('profile.active.modified')" :value="view.modified" />
          <ProfileFact :label="t('profile.active.size')" :value="view.size" />
          <ProfileFact
            v-if="view.dlcs"
            :label="t('profile.active.dlcs')"
            :value="view.dlcs"
          />
        </div>
        <span class="truncate text-label text-subtle-foreground"
          >{{ t('profile.active.foundIn') }} {{ view.source }}</span
        >
      </div>
      <div class="flex shrink-0 flex-col gap-1.75">
        <Button :variant="ButtonVariant.Secondary" @click="emit('change')">
          <SaveIcon />{{ t('profile.active.change') }}
        </Button>
        <Button :variant="ButtonVariant.Outline" @click="emit('reload')">
          <RefreshCwIcon />{{ t('profile.active.reload') }}
        </Button>
      </div>
    </CardCollapsibleContent>
  </CardCollapsible>
</template>
```

- [ ] **Step 10: `ui/src/screens/profile/SectionsCard.vue`**

```vue
<script setup lang="ts">
import { TriangleAlertIcon } from '@lucide/vue'
import { computed } from 'vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import {
  CardCollapsible,
  CardCollapsibleContent,
  CardCollapsibleTrigger,
} from '@/components/ui/card'
import { i18n, useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import type { SaveDiagnostic, SaveSummary } from '@/lib/ipc/types'
import { formatCount, sectionLabel } from '@/lib/profile/profileView'

const props = defineProps<{ summary: SaveSummary }>()
const { t } = useMessages()

const sections = computed(() => {
  const locale = i18n.global.locale.value
  return props.summary.sections.map((s) => {
    const label = sectionLabel(s.kind)
    return {
      kind: s.kind,
      name: label ? t(label) : s.kind,
      count: formatCount(s.count, locale),
    }
  })
})

const diagnosticText = (d: SaveDiagnostic): string => {
  switch (d.kind) {
    case 'unexpectedKind':
      return `${t('profile.saveDiagnostics.unexpectedKind')} (${d.expected}, ${d.found})`
    case 'sectionOverrun':
      return `${t('profile.saveDiagnostics.sectionOverrun')} (${d.section})`
    case 'trailingBytes':
      return t('profile.saveDiagnostics.trailingBytes')
    default:
      return assertNever(d)
  }
}
</script>

<template>
  <CardCollapsible :default-open="true">
    <CardCollapsibleTrigger>
      {{ t('profile.read.title') }}
      <template #summary
        >{{ summary.sections.length }} {{ t('profile.read.sections') }}</template
      >
    </CardCollapsibleTrigger>
    <CardCollapsibleContent class="flex flex-col gap-3">
      <div class="grid grid-cols-5 gap-2">
        <div
          v-for="s in sections"
          :key="s.kind"
          class="flex flex-col gap-1.5 border border-border bg-data px-2.5 py-2.25"
        >
          <span class="text-label text-subtle-foreground">{{ s.kind }}</span>
          <span class="text-row text-foreground">{{ s.name }}</span>
          <span class="text-caption text-foreground-soft">{{ s.count }}</span>
        </div>
      </div>
      <Alert
        v-if="summary.diagnostics.length"
        :variant="AlertVariant.Destructive"
      >
        <TriangleAlertIcon />
        <AlertTitle>{{ t('profile.read.diagnostics') }}</AlertTitle>
        <AlertDescription>
          <span
            v-for="(d, i) in summary.diagnostics"
            :key="i"
            class="block"
            >{{ diagnosticText(d) }}</span
          >
        </AlertDescription>
      </Alert>
      <p
        class="border border-border bg-data px-3 py-2.75 text-caption text-foreground-soft"
      >
        {{ t('profile.read.note') }}
      </p>
    </CardCollapsibleContent>
  </CardCollapsible>
</template>
```

- [ ] **Step 11: `ui/src/screens/profile/ProfileError.vue`**

```vue
<script setup lang="ts">
import { RefreshCwIcon, TriangleAlertIcon } from '@lucide/vue'
import { computed } from 'vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import { Button, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import type { IpcError } from '@/lib/ipc/types'

const props = defineProps<{ error: IpcError | null }>()
const emit = defineEmits<{ retry: [] }>()
const { t } = useMessages()

// `null` is a failure that isn't an IpcError at all: the backend never answered.
const message = computed((): string => {
  const e = props.error
  if (e === null) return t('profile.errors.noBackend')
  switch (e.kind) {
    case 'noActiveProfile':
      return t('profile.errors.noActiveProfile')
    case 'unknownProfile':
      return t('profile.errors.unknownProfile')
    case 'unreadableSave':
      return `${t('profile.errors.unreadableSave')} ${e.reason}`
    case 'settingsNotWritable':
      return `${t('profile.errors.settingsNotWritable')} ${e.reason}`
    case 'unknownTarget':
      return t('profile.errors.unknownTarget')
    case 'catalogUnavailable':
      return t('profile.errors.catalogUnavailable')
    case 'storeUnavailable':
      return `${t('profile.errors.storeUnavailable')} ${e.reason}`
    case 'wikiUnavailable':
      return t('profile.errors.wikiUnavailable')
    default:
      return assertNever(e)
  }
})
</script>

<template>
  <Alert :variant="AlertVariant.Destructive">
    <TriangleAlertIcon />
    <AlertTitle>{{ t('profile.errors.title') }}</AlertTitle>
    <AlertDescription class="flex flex-col items-start gap-2">
      {{ message }}
      <Button :variant="ButtonVariant.Outline" @click="emit('retry')">
        <RefreshCwIcon />{{ t('profile.errors.retry') }}
      </Button>
    </AlertDescription>
  </Alert>
</template>
```

- [ ] **Step 12: `ui/src/screens/ProfileScreen.vue`**

```vue
<script setup lang="ts">
import { SaveIcon } from '@lucide/vue'
import { computed, ref } from 'vue'
import { Skeleton } from '@/components/ui/skeleton'
import { useMessages } from '@/i18n'
import { LoadStatus, useProfileStore } from '@/stores/profile'
import ScreenHeader from './ScreenHeader.vue'
import ActiveProfileCard from './profile/ActiveProfileCard.vue'
import CandidatesCard from './profile/CandidatesCard.vue'
import ChainCard from './profile/ChainCard.vue'
import NoSavesCard from './profile/NoSavesCard.vue'
import ProfileError from './profile/ProfileError.vue'
import SectionsCard from './profile/SectionsCard.vue'

const profile = useProfileStore()
const { t } = useMessages()

// "Cambia profilo" shows the candidates over an active profile until one is chosen.
const changing = ref(false)

const noSaves = computed(() => {
  const a = profile.setup?.active
  return a?.kind === 'none' ? a : null
})
const activeProfile = computed(() => {
  const a = profile.setup?.active
  return a?.kind === 'active' ? a : null
})
const choosing = computed(
  () => profile.setup?.active.kind === 'needsChoice' || changing.value,
)

const choose = async (id: string) => {
  await profile.choose(id)
  changing.value = false
}
</script>

<template>
  <div class="flex max-w-250 flex-col gap-4">
    <ScreenHeader
      :icon="SaveIcon"
      :eyebrow="t('profile.eyebrow')"
      :title="t('profile.title')"
      >{{ t('profile.intro') }}</ScreenHeader
    >
    <ProfileError
      v-if="profile.status === LoadStatus.Failed"
      :error="profile.error"
      @retry="profile.load()"
    />
    <template v-else-if="profile.setup">
      <ChainCard :setup="profile.setup" />
      <NoSavesCard
        v-if="noSaves"
        :reason="noSaves.reason"
        :diagnostics="profile.setup.diagnostics"
        @retry="profile.load()"
      />
      <CandidatesCard
        v-else-if="choosing"
        :active="profile.setup.active"
        :candidates="profile.setup.candidates"
        :busy="profile.status === LoadStatus.Loading"
        @choose="choose"
        @cancel="changing = false"
      />
      <template v-else-if="activeProfile">
        <ActiveProfileCard
          :active="activeProfile"
          :game="profile.setup.game"
          @change="changing = true"
          @reload="profile.load()"
        />
        <SectionsCard v-if="profile.summary" :summary="profile.summary" />
      </template>
    </template>
    <div v-else class="flex flex-col gap-4">
      <Skeleton class="h-30 w-full" />
      <Skeleton class="h-50 w-full" />
    </div>
  </div>
</template>
```

- [ ] **Step 13: Verify** — `pnpm --filter ui exec vitest run` (typecheck waits for Task 7).

---

### Task 7: The router, the indicator and the shell

**Files:**
- Create: `ui/src/router/routes.ts`, `ui/src/router/index.ts`, `ui/src/components/shell/ProfileIndicator.vue`, `ui/src/App.vue`
- Modify: `ui/src/main.ts`, `ui/src/components/shell/NavBar.vue` (a `status` slot; `section` may be `null`), `ui/src/components/shell/TabItem.vue` (middle click closes)

- [ ] **Step 1: `ui/src/router/routes.ts`**

```ts
import type { Component } from 'vue'
import type { RouteRecordRaw } from 'vue-router'
import { TabOrigin } from '@/components/shell/tabs'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import PlaceholderScreen from '@/screens/PlaceholderScreen.vue'
import ProfileScreen from '@/screens/ProfileScreen.vue'
import {
  RouteName,
  routeArrives,
  routeIcon,
  routeOrigin,
  routePath,
  routeTitle,
} from './routeTable'

declare module 'vue-router' {
  interface RouteMeta {
    origin: TabOrigin
    title: MessageKey<MessageSchema>
    icon: Component
    needsProfile: boolean
    arrives?: MessageKey<MessageSchema>
  }
}

// The screens that exist. Every other route renders its placeholder until its sub-project.
const screens: Partial<Record<RouteName, Component>> = {
  [RouteName.Profile]: ProfileScreen,
}

export const routes: RouteRecordRaw[] = [
  { path: '/', redirect: { name: RouteName.NextSteps } },
  ...Object.values(RouteName).map(
    (name): RouteRecordRaw => ({
      path: routePath[name],
      name,
      component: screens[name] ?? PlaceholderScreen,
      meta: {
        origin: routeOrigin[name],
        title: routeTitle[name],
        icon: routeIcon[name],
        needsProfile: routeOrigin[name] === TabOrigin.Progress,
        arrives: routeArrives[name],
      },
    }),
  ),
]
```

- [ ] **Step 2: `ui/src/router/index.ts`**

```ts
import { createMemoryHistory, createRouter } from 'vue-router'
import { routes } from './routes'

// Memory history: a tab is a position and no one reads a URL in a Tauri window; the hash
// belongs to the development pages. The router only ever shows the active tab's location.
export const router = createRouter({ history: createMemoryHistory(), routes })
```

- [ ] **Step 3: `NavBar.vue`** — change the props to `defineProps<{ section: NavSection | null; focused: boolean }>()` and add `<slot name="status" />` immediately before the search trigger `<Button :variant="ButtonVariant.Field" …>`.

- [ ] **Step 4: `TabItem.vue`** — on the root `<div role…>` add `@mousedown.middle.prevent` and `@auxclick.middle="emit('close')"` (the first stops the browser's autoscroll cursor).

- [ ] **Step 5: `ui/src/components/shell/ProfileIndicator.vue`**

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import { cn } from '@/lib/cn'
import type { IndicatorView } from '@/lib/profile/profileView'

const props = defineProps<{ view: IndicatorView | null }>()
const emit = defineEmits<{ open: [] }>()
const { t } = useMessages()

// Which profile every number reads (DESIGN-BRIEF.md §4.1), one click from changing it.
const label = computed((): string => {
  const v = props.view
  if (v === null) return ''
  switch (v.kind) {
    case 'active':
      return [v.edition, `${t('indicator.slot')} ${v.slot}`, v.modified]
        .filter(Boolean)
        .join(' · ')
    case 'noProfile':
      return t('indicator.noProfile')
    case 'notFound':
      return t('indicator.notFound')
    default:
      return assertNever(v)
  }
})
</script>

<template>
  <Button
    :variant="ButtonVariant.Field"
    :size="ButtonSize.Compact"
    class="max-w-60 shrink-0"
    @click="emit('open')"
  >
    <Skeleton v-if="view === null" class="h-3 w-24" />
    <template v-else>
      <span
        :class="
          cn(
            'size-1.5 shrink-0',
            view.kind === 'active' ? 'bg-state-done' : 'bg-state-unexpected',
          )
        "
      />
      <span class="truncate text-foreground-soft">{{ label }}</span>
    </template>
  </Button>
</template>
```

- [ ] **Step 6: `ui/src/App.vue`**

```vue
<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import NavBar from '@/components/shell/NavBar.vue'
import ProfileIndicator from '@/components/shell/ProfileIndicator.vue'
import SectionSidebar from '@/components/shell/SectionSidebar.vue'
import SidebarItem from '@/components/shell/SidebarItem.vue'
import TitleBar from '@/components/shell/TitleBar.vue'
import type { NavSection } from '@/components/shell/navSection'
import {
  SidebarSection,
  isEntryActive,
  navSectionOf,
  sectionOfOrigin,
  sidebarEntries,
  sidebarHeaders,
  sidebarSectionOf,
} from '@/components/shell/sectionNav'
import type { SidebarEntry } from '@/components/shell/sectionNav'
import { SidebarWidth } from '@/components/shell/sidebarWidth'
import type { TabView } from '@/components/shell/tabs'
import { TooltipProvider } from '@/components/ui/tooltip'
import { i18n, useMessages } from '@/i18n'
import { indicator } from '@/lib/profile/profileView'
import {
  closeWindow,
  minimizeWindow,
  toggleMaximizeWindow,
  watchWindowFocus,
} from '@/lib/window/appWindow'
import {
  RouteName,
  locationTitle,
  routeOrigin,
} from '@/router/routeTable'
import ProgressGate from '@/screens/ProgressGate.vue'
import { useProfileStore } from '@/stores/profile'
import { useTabsStore } from '@/stores/tabs'

const router = useRouter()
const tabs = useTabsStore()
const profile = useProfileStore()
const { t } = useMessages()

const focused = ref(true)
let stopWatchingFocus: (() => void) | undefined
onMounted(async () => {
  void profile.load()
  stopWatchingFocus = await watchWindowFocus((value) => {
    focused.value = value
  })
})
onUnmounted(() => stopWatchingFocus?.())

// The router shows the active tab: selecting, closing or navigating a tab moves it.
watch(
  () => tabs.active?.location,
  (location) => {
    if (location) void router.replace(location)
  },
  { immediate: true },
)

const tabViews = computed<TabView[]>(() =>
  tabs.tabs.map((tab) => ({
    id: tab.id,
    label: t(locationTitle(tab.location)),
    origin: routeOrigin[tab.location.name],
  })),
)

// The sidebar shows the active tab's section, until the navbar or the cog picks another.
const browsing = ref<SidebarSection>(SidebarSection.Progress)
watch(
  () => tabs.active?.location.name,
  (name) => {
    if (name) browsing.value = sectionOfOrigin(routeOrigin[name])
  },
  { immediate: true },
)

const sidebarWidth = ref<number>(SidebarWidth.Default)
const header = computed(() => sidebarHeaders[browsing.value])
const entries = computed(() => sidebarEntries[browsing.value])

const showSection = (section: NavSection) => {
  browsing.value = sidebarSectionOf(section)
}

// Ctrl+click opens the entry in a new tab, as a browser does.
const openEntry = (entry: SidebarEntry, event: MouseEvent) => {
  if (event.ctrlKey) tabs.open(entry.location)
  else tabs.navigate(entry.location)
}

const indicatorView = computed(() =>
  profile.setup
    ? indicator(profile.setup.active, new Date(), i18n.global.locale.value)
    : null,
)
</script>

<template>
  <TooltipProvider>
    <div
      class="flex h-screen flex-col overflow-hidden bg-background text-foreground"
    >
      <TitleBar
        :tabs="tabViews"
        :active-id="tabs.activeId"
        :focused="focused"
        @select="tabs.select"
        @close="tabs.close"
        @move="tabs.move"
        @add="tabs.open()"
        @minimize="minimizeWindow"
        @toggle-maximize="toggleMaximizeWindow"
        @close-window="closeWindow"
      />
      <NavBar
        :section="navSectionOf(browsing)"
        :focused="focused"
        @update:section="showSection"
        @settings="browsing = SidebarSection.Settings"
        @about="tabs.navigate({ name: RouteName.About })"
      >
        <template #status>
          <ProfileIndicator
            :view="indicatorView"
            @open="tabs.navigate({ name: RouteName.Profile })"
          />
        </template>
      </NavBar>
      <div class="flex min-h-0 flex-1">
        <SectionSidebar
          v-model:width="sidebarWidth"
          :title="t(header.title)"
          :hint="t(header.hint)"
          class="border-y-0 border-l-0"
        >
          <template #icon><component :is="header.icon" /></template>
          <SidebarItem
            v-for="entry in entries"
            :key="entry.key"
            :active="isEntryActive(entry, tabs.active?.location)"
            @click="openEntry(entry, $event)"
          >
            <template #icon><component :is="entry.icon" /></template>
            {{ t(entry.label) }}
          </SidebarItem>
        </SectionSidebar>
        <main class="min-w-0 flex-1 overflow-auto px-5.5 pt-5 pb-15">
          <RouterView v-slot="{ Component, route }">
            <ProgressGate v-if="route.meta.needsProfile">
              <component :is="Component" />
            </ProgressGate>
            <component :is="Component" v-else />
          </RouterView>
        </main>
      </div>
    </div>
  </TooltipProvider>
</template>
```

- [ ] **Step 7: `ui/src/main.ts`**

```ts
import { createPinia } from 'pinia'
import { createApp } from 'vue'
import './assets/main.css'
import App from './App.vue'
import { i18n } from './i18n'
import { DevRoute } from './lib/constants/devRoutes'
import { router } from './router'

// The development pages (the Kit, the verification page) sit behind import.meta.env.DEV:
// the production build drops their imports entirely.
const mountKit = async () => {
  const { default: KitPage } = await import('./kit/KitPage.vue')
  createApp(KitPage).use(i18n).mount('#app')
}

const mountVerify = async () => {
  const { default: VerifyPage } = await import('./verify/VerifyPage.vue')
  createApp(VerifyPage).use(i18n).mount('#app')
}

const hash = window.location.hash
if (import.meta.env.DEV && hash === DevRoute.Kit) {
  void mountKit()
} else if (import.meta.env.DEV && hash === DevRoute.Verify) {
  void mountVerify()
} else {
  createApp(App).use(createPinia()).use(router).use(i18n).mount('#app')
}
```

- [ ] **Step 8: Verify statically** — run `pnpm --filter ui exec prettier --write src scripts`, then `pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test`. Expected: all green, `0 violations, 0 declared exemptions`.

- [ ] **Step 9: Commit Tasks 2, 6 and 7** (they only compile together)

```bash
git add crates/app/tauri.conf.json crates/app/capabilities ui/src ui/scripts/scan-conventions.mjs
git commit -m "feat(ui): the window becomes the app, with profile selection as its first screen"
```

---

### Task 8: Visual verification with the fixtures

- [ ] **Step 1:** `pnpm ui:dev` and open, one at a time, `/?fixture=none`, `/?fixture=pick`, `/?fixture=active` (with Playwright: scroll an element into view before pressing its coordinates).
- [ ] **Step 2:** In each scenario check and screenshot: the indicator's square and label; the Next steps tab showing the gate with profile selection below; the Profile screen from the cog (Settings sidebar) — chain badges, the none alert with diagnostics, the candidate table with nothing selected and "Usa questo profilo" disabled until a row is clicked; after choosing, the indicator turns active, the gate lifts on Next steps into its placeholder, the sections grid shows ten counts.
- [ ] **Step 3:** Tabs: `+` opens Next steps after the active tab; Ctrl+click on "Unlock" opens a new tab; middle click closes; closing the last tab leaves a fresh Next steps; dragging reorders; the sidebar follows the active tab's section; the navbar's Wiki/Progress switch changes the sidebar without navigating.
- [ ] **Step 4:** `#kit` and `#verify` still mount. Fix what differs before Task 9.

---

### Task 9: Documents, full check, production build

**Files:** `docs/frontend-conventions.md`, `docs/STATUS.md`, `CLAUDE.md`, `docs/BACKLOG.md`, the spec

- [ ] **Step 1: `docs/frontend-conventions.md`** — structure: add `router/` (routeTable, routes, index), `stores/` (tabs, tabModel, profile), `screens/` (and `screens/profile/`), `lib/window/`, `lib/profile/`, `lib/ipc/transport.ts` and `lib/ipc/fixtures/`, `verify/` beside `kit/`; the enforcement table gains "window API outside `src/lib/window/`"; the development-only directories are `src/kit/` and `src/verify/`; a paragraph on the transport and `?fixture=`.
- [ ] **Step 2: `docs/STATUS.md`** — under the design system: cycle 3 with its seven sub-projects, the first checked; a session log entry for 2026-09-11.
- [ ] **Step 3: `CLAUDE.md`** — State: "`ui/` is the shell, with profile selection; the verification page lives behind `#verify`"; the stack line: Pinia and Vue Router installed, TanStack with sub-project 3.
- [ ] **Step 4: `docs/BACKLOG.md`** — B14: choosing the game or saves folder by hand (a dialog plugin, a command that accepts a path and hands back a `SetupState`, the two buttons of `Schermate.dc.html`).
- [ ] **Step 5: The spec** — record the planning deviations listed under Global Constraints.
- [ ] **Step 6:** `pnpm check` → all green.
- [ ] **Step 7:** `pnpm --filter ui build`, then
  `grep -rl "rep_plus-2\|KitPage\|VerifyPage\|no fixture answers" ui/dist || echo "no fixtures, kit or verify"`.
- [ ] **Step 8: Commit**

```bash
git add docs CLAUDE.md
git commit -m "docs: cycle 3 decomposed and its first sub-project in the conventions, status and backlog"
```
