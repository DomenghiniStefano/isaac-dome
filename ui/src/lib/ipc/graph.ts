import { invoke } from '@tauri-apps/api/core'
import { Command } from '../constants/commands'
import type {
  GoalId,
  NextSteps,
  PlanView,
  TargetKey,
  UnlockView,
} from './types'

export const unlock = (): Promise<UnlockView> => invoke(Command.Unlock)

export const nextSteps = (): Promise<NextSteps> => invoke(Command.NextSteps)

export const plan = (): Promise<PlanView> => invoke(Command.Plan)

// What's saved is the key, not the view: the name and icon the Plan shows are
// resolved by the backend on every read. `target` is the name of the Rust command's parameter.
export const addGoal = (key: TargetKey): Promise<PlanView> =>
  invoke(Command.AddGoal, { target: key })

export const removeGoal = (id: GoalId): Promise<PlanView> =>
  invoke(Command.RemoveGoal, { id })
