// Where a read stands. Its own file and nothing else: it was exported from `profile.ts`, so
// every other store imported a shared enum out of one particular store — which said, wrongly,
// that the profile owns the idea.
//
// No fields: an `as const` object, like every other value the app compares against.
export const LoadStatus = {
  Idle: 'idle',
  Loading: 'loading',
  Ready: 'ready',
  Failed: 'failed',
} as const
export type LoadStatus = (typeof LoadStatus)[keyof typeof LoadStatus]
