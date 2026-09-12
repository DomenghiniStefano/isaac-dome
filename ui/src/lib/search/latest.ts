// The palette asks again on every keystroke and the answers come back out of order: each
// question takes a number, and only the newest one's answer is shown.
export const latest = (): {
  next: () => number
  isCurrent: (token: number) => boolean
} => {
  let seq = 0
  return {
    next: () => ++seq,
    isCurrent: (token: number) => token === seq,
  }
}
