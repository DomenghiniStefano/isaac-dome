import { Command } from '../../constants/commands'
import { assertNever } from '../../assertNever'
import { FloorScenario, floorAnswer } from './floor'
import type { CommandArgs, CommandName } from '../transport'
import type { IpcError, SetupState, Target } from '../types'
import { completionMatrix } from './completion'
import { candidates, noneSetup, setupWith, summary } from './profile'
import {
  autostartAnswer,
  resetSettingsFixture,
  sessionAnswer,
  setAutostartAnswer,
  setResumeTabsAnswer,
  setScaleAnswer,
  setSessionAnswer,
  setStayInBackgroundAnswer,
  settingsAnswer,
} from './settings'
import type { QueueOptions } from './queue'
import {
  QueueScenario,
  addToQueue,
  importGoals,
  moveInQueue,
  readQueue,
  removeFromQueue,
  resetQueue,
} from './queue'

export const FixtureScenario = {
  None: 'none',
  Pick: 'pick',
  Active: 'active',
} as const
export type FixtureScenario =
  (typeof FixtureScenario)[keyof typeof FixtureScenario]

// `?fixture=none|pick|active` on the development server; active when absent.
const ScenarioParam = 'fixture'
// `?art=none` answers every image URL as null: every user's first launch, before any art.
const ArtParam = 'art'
// `?catalog=none` answers the graph as a machine without the game gets it.
const CatalogParam = 'catalog'
// `?queue=empty|unavailable|unreadable` answers the plan queue in one of its other states.
const QueueParam = 'queue'
// `?collection=unread` answers the Collection as a save whose section 4 wasn't read.
const CollectionParam = 'collection'
// `?wiki=none` answers the wiki as a binary whose embedded dataset didn't load.
const WikiParam = 'wiki'
// `?floor=empty` answers the painted grid as one nobody has touched yet.
const FloorParam = 'floor'
const Off = 'none'
const Unread = 'unread'

const query = (): URLSearchParams =>
  new URLSearchParams(globalThis.location?.search ?? '')

const currentScenario = (): FixtureScenario => {
  const requested = query().get(ScenarioParam)
  return (
    Object.values(FixtureScenario).find((s) => s === requested) ??
    FixtureScenario.Active
  )
}

const currentQueueScenario = (): QueueScenario => {
  const requested = query().get(QueueParam)
  return (
    Object.values(QueueScenario).find((s) => s === requested) ??
    QueueScenario.Rows
  )
}

const currentFloorScenario = (): FloorScenario => {
  const requested = query().get(FloorParam)
  return (
    Object.values(FloorScenario).find((s) => s === requested) ??
    FloorScenario.Solve
  )
}

const artShown = (): boolean => query().get(ArtParam) !== Off
const catalogShown = (): boolean => query().get(CatalogParam) !== Off
const collectionRead = (): boolean => query().get(CollectionParam) !== Unread
const wikiShown = (): boolean => query().get(WikiParam) !== Off

// A profile chosen through select_profile stays chosen for the page's life, as in the app.
let chosenId: string | null = null

export const resetFixtures = (): void => {
  chosenId = null
  resetQueue()
  resetSettingsFixture()
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
const wikiUnavailable: IpcError = { kind: 'wikiUnavailable' }

// Every command that reads the save answers only with an active profile, as the backend does.
const whenActive = (scenario: FixtureScenario, read: () => unknown): unknown =>
  setupFor(scenario).active.kind === 'active'
    ? read()
    : Promise.reject(noActiveProfile)

// The graph's payloads and its 1,500 images load only when a screen asks for the graph: every
// read of the profile would otherwise wait for them.
const graph = async () => {
  const { graphAnswers } = await import('./graph')
  return graphAnswers({ withArt: artShown(), withCatalog: catalogShown() })
}

// The Collection's image index loads only when the Collection asks for it, like the graph.
const collection = async () => {
  const { collectionAnswer } = await import('./collection')
  return collectionAnswer({
    withArt: artShown(),
    withCatalog: catalogShown(),
    collectionRead: collectionRead(),
  })
}

// The wiki needs neither a profile nor the catalog: the pack's image index and sample pages
// load when the Wiki, or a tab label, first asks.
const wiki = async () => import('./wiki')

// The queue's nodes are the Unlock view's, as in the app: the same node on both screens.
const queueOptions = async (): Promise<QueueOptions> => ({
  scenario: currentQueueScenario(),
  withCatalog: catalogShown(),
  nodes: (await graph()).unlock.nodes,
})

const achievementArg = (args: CommandArgs | undefined): number =>
  Number(args?.achievement)

const afterArg = (args: CommandArgs | undefined): number | null =>
  args?.after === null || args?.after === undefined ? null : Number(args.after)

type Handler = (
  args: CommandArgs | undefined,
  scenario: FixtureScenario,
) => unknown

// Only what the screens read today. Any other command is refused loudly: a screen built on a
// command with no fixture should fail on the development server, not render undefined.
const handlers: Partial<Record<CommandName, Handler>> = {
  [Command.Settings]: () => settingsAnswer(),
  [Command.SetScale]: (args) => setScaleAnswer(Number(args?.percent)),
  [Command.SetStayInBackground]: (args) =>
    setStayInBackgroundAnswer(Boolean(args?.stay)),
  [Command.SetResumeTabs]: (args) => setResumeTabsAnswer(Boolean(args?.resume)),
  [Command.Autostart]: () => autostartAnswer(),
  [Command.SetAutostart]: (args) => setAutostartAnswer(Boolean(args?.on)),
  [Command.WindowSession]: () => sessionAnswer(),
  [Command.SetWindowSession]: (args) => {
    setSessionAnswer((args?.document as string | null) ?? null)
    return undefined
  },
  [Command.SetupState]: (_args, scenario) => setupFor(scenario),
  [Command.SelectProfile]: (args, scenario) => {
    chosenId = String(args?.id ?? '')
    return setupFor(scenario)
  },
  [Command.SaveSummary]: (_args, scenario) =>
    whenActive(scenario, () => summary),
  [Command.Completion]: (_args, scenario) =>
    whenActive(scenario, () => completionMatrix(artShown())),
  [Command.Runs]: async () => (await import('./runs')).runsAnswer(),
  [Command.Live]: async () => (await import('./runs')).liveAnswer(),
  // The floor reads no profile: it answers the drawing, whatever the save is doing.
  [Command.FloorCandidates]: (args) =>
    floorAnswer(currentFloorScenario(), args),
  [Command.GraphViews]: (_args, scenario) =>
    whenActive(scenario, async () => await graph()),
  [Command.Want]: (args, scenario) =>
    whenActive(scenario, async () =>
      (await import('./graph')).wantAnswer(
        { withArt: artShown(), withCatalog: catalogShown() },
        args?.target as Target,
      ),
    ),
  [Command.Collection]: (_args, scenario) =>
    whenActive(scenario, () => collection()),
  [Command.WikiIndex]: async () =>
    (await wiki()).wikiIndexAnswer({
      withArt: artShown() && catalogShown(),
      withWiki: wikiShown(),
    }),
  [Command.WikiEntry]: async (args) => {
    if (!wikiShown()) throw wikiUnavailable
    return (await wiki()).wikiEntryAnswer(args?.target as Target)
  },
  // Search needs no profile, as in the app: what it lacks travels as a diagnostic.
  [Command.Search]: async (args) => {
    const { searchAnswer } = await import('./search')
    return searchAnswer(
      {
        withArt: artShown(),
        withCatalog: catalogShown(),
        withWiki: wikiShown(),
      },
      String(args?.query ?? ''),
      Number(args?.limit ?? 0),
    )
  },
  [Command.Queue]: (_args, scenario) =>
    whenActive(scenario, async () => readQueue(await queueOptions())),
  [Command.QueueAdd]: (args, scenario) =>
    whenActive(scenario, async () =>
      addToQueue(await queueOptions(), achievementArg(args)),
    ),
  [Command.QueueRemove]: (args, scenario) =>
    whenActive(scenario, async () =>
      removeFromQueue(await queueOptions(), achievementArg(args)),
    ),
  [Command.QueueMove]: (args, scenario) =>
    whenActive(scenario, async () =>
      moveInQueue(await queueOptions(), achievementArg(args), afterArg(args)),
    ),
  [Command.QueueImportGoals]: (_args, scenario) =>
    whenActive(scenario, async () => importGoals(await queueOptions())),
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
