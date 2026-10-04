import { describe, expect, it } from 'vitest'
import type { ChallengeRow, RewardView } from '@/lib/ipc/types'
import { challengeQueueTarget, isChallengeQueued } from './challengeQueue'

const reward = (achievement: number, done: boolean | null): RewardView => ({
  achievement,
  text: null,
  iconUrl: null,
  page: null,
  done,
})

const row = (rewards: RewardView[]): ChallengeRow =>
  ({ number: 1, name: 'Pitch Black', rewards }) as unknown as ChallengeRow

describe('challengeQueueTarget', () => {
  it('is the first reward, with whether it is earned', () => {
    expect(
      challengeQueueTarget(row([reward(80, false), reward(81, false)])),
    ).toEqual({ achievement: 80, done: false })
  })

  it('is earned when the first reward is', () => {
    expect(challengeQueueTarget(row([reward(80, true)]))).toEqual({
      achievement: 80,
      done: true,
    })
  })

  // Unread is never "earned": a save whose section 1 was not read still lets you queue it.
  it('reads a reward the save could not answer for as not earned', () => {
    expect(challengeQueueTarget(row([reward(80, null)]))).toEqual({
      achievement: 80,
      done: false,
    })
  })

  it('is nothing when the challenge unlocks nothing', () => {
    expect(challengeQueueTarget(row([]))).toBeNull()
  })
})

describe('isChallengeQueued', () => {
  // The row says "in coda" about the reward its Actions button adds, and no other: a second
  // reward in the queue must not make the row read queued while its button still offers Add.
  it('is the first reward being in the queue', () => {
    const r = row([reward(80, false), reward(81, false)])
    expect(isChallengeQueued(r, new Set([80]))).toBe(true)
    expect(isChallengeQueued(r, new Set([81]))).toBe(false)
  })

  it('is false for a challenge that unlocks nothing', () => {
    expect(isChallengeQueued(row([]), new Set([80]))).toBe(false)
  })
})
