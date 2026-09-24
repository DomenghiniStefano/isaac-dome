import { describe, expect, it } from 'vitest'
import type { ChallengeRow, RewardView } from '@/lib/ipc/types'
import { queueableReward } from './challengeQueue'

const reward = (achievement: number, done: boolean | null): RewardView => ({
  achievement,
  text: null,
  iconUrl: null,
  page: null,
  done,
})

const row = (rewards: RewardView[]): ChallengeRow =>
  ({ number: 1, name: 'Pitch Black', rewards }) as unknown as ChallengeRow

describe('queueableReward', () => {
  it('is the first reward, while it is neither earned nor queued', () => {
    expect(
      queueableReward(row([reward(80, false), reward(81, false)]), []),
    ).toBe(80)
  })

  // Card #80, P3: "+" stayed on a challenge whose reward was already in the queue.
  it('is nothing once that reward is queued', () => {
    expect(queueableReward(row([reward(80, false)]), [80])).toBeNull()
  })

  it('is nothing once it is earned, or when there is no reward', () => {
    expect(queueableReward(row([reward(80, true)]), [])).toBeNull()
    expect(queueableReward(row([]), [])).toBeNull()
  })

  // Unread is never "earned": a save whose section 1 was not read still lets you queue it.
  it('is still offered when the save could not say whether it is earned', () => {
    expect(queueableReward(row([reward(80, null)]), [])).toBe(80)
  })
})
