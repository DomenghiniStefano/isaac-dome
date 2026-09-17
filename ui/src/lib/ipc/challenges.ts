import { Command } from '../constants/commands'
import { call } from './transport'
import type { ChallengesView } from './types'

// The active profile's challenges, joined with the catalog's `challenges.xml` and the wiki's
// page for each one.
export const challenges = (): Promise<ChallengesView> =>
  call(Command.Challenges)
