import { Command } from '../constants/commands'
import { call } from './transport'
import type {
  GoalId,
  NextSteps,
  PlanView,
  Target,
  TargetKey,
  UnlockView,
  WantView,
} from './types'

export const unlock = (): Promise<UnlockView> => call(Command.Unlock)

export const nextSteps = (): Promise<NextSteps> => call(Command.NextSteps)

export const plan = (): Promise<PlanView> => call(Command.Plan)

// What's saved is the key, not the view: the name and icon the Plan shows are
// resolved by the backend on every read. `target` is the name of the Rust command's parameter.
export const addGoal = (key: TargetKey): Promise<PlanView> =>
  call(Command.AddGoal, { target: key })

export const removeGoal = (id: GoalId): Promise<PlanView> =>
  call(Command.RemoveGoal, { id })

// B37: the graph read from the other end. `target` is the name of the Rust command's
// parameter, and it is the same `Target` a wiki page is keyed by — one vocabulary, two
// questions ("what is this?" and "what do I play for it?").
export const want = (target: Target): Promise<WantView> =>
  call(Command.Want, { target })
