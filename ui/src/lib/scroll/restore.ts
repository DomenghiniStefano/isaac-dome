// What a scrolling region does about the position it was left at, from how tall it is now.
//
// A region is mounted before its data: a page arrives empty and fills a tick or a command later.
// Setting the position the moment it mounts lets the browser clamp it to the little there is, and
// the page lands somewhere it never was. So the answer is one of three: nothing to restore, wait
// for more content, or go.

export const RestoreStep = {
  Done: 'done',
  Wait: 'wait',
  Go: 'go',
} as const
export type RestoreStep = (typeof RestoreStep)[keyof typeof RestoreStep]

export type RestoreAnswer =
  | { kind: typeof RestoreStep.Done }
  | { kind: typeof RestoreStep.Wait }
  | { kind: typeof RestoreStep.Go; top: number }

export interface RegionSize {
  scrollHeight: number
  clientHeight: number
}

export const restoreStep = (
  target: number | undefined,
  size: RegionSize,
): RestoreAnswer => {
  if (target === undefined || target <= 0) return { kind: RestoreStep.Done }
  if (size.scrollHeight - size.clientHeight < target)
    return { kind: RestoreStep.Wait }
  return { kind: RestoreStep.Go, top: target }
}
