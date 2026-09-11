import { Command } from '../../constants/commands'
import { assertNever } from '../../assertNever'
import type { CommandArgs, CommandName } from '../transport'
import type { IpcError, SetupState } from '../types'
import { completionMatrix } from './completion'
import { graphAnswers } from './graph'
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
// `?art=none` answers every image URL as null: every user's first launch, before any art.
const ArtParam = 'art'
// `?catalog=none` answers the graph as a machine without the game gets it.
const CatalogParam = 'catalog'
const Off = 'none'

const query = (): URLSearchParams =>
  new URLSearchParams(globalThis.location?.search ?? '')

const currentScenario = (): FixtureScenario => {
  const requested = query().get(ScenarioParam)
  return (
    Object.values(FixtureScenario).find((s) => s === requested) ??
    FixtureScenario.Active
  )
}

const artShown = (): boolean => query().get(ArtParam) !== Off
const catalogShown = (): boolean => query().get(CatalogParam) !== Off

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

// Every command that reads the save answers only with an active profile, as the backend does.
const whenActive = (scenario: FixtureScenario, read: () => unknown): unknown =>
  setupFor(scenario).active.kind === 'active'
    ? read()
    : Promise.reject(noActiveProfile)

const graph = () =>
  graphAnswers({ withArt: artShown(), withCatalog: catalogShown() })

type Handler = (
  args: CommandArgs | undefined,
  scenario: FixtureScenario,
) => unknown

// Only what the screens read today. Any other command is refused loudly: a screen built on a
// command with no fixture should fail on the development server, not render undefined.
const handlers: Partial<Record<CommandName, Handler>> = {
  [Command.SetupState]: (_args, scenario) => setupFor(scenario),
  [Command.SelectProfile]: (args, scenario) => {
    chosenId = String(args?.id ?? '')
    return setupFor(scenario)
  },
  [Command.SaveSummary]: (_args, scenario) =>
    whenActive(scenario, () => summary),
  [Command.Completion]: (_args, scenario) =>
    whenActive(scenario, () => completionMatrix(artShown())),
  [Command.Unlock]: (_args, scenario) =>
    whenActive(scenario, () => graph().unlock),
  [Command.NextSteps]: (_args, scenario) =>
    whenActive(scenario, () => graph().steps),
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
