// One pending timeout, which a new one replaces and which can be called off. What a debounce and
// a deadline both are underneath, kept in one place so neither holds a timer id of its own.
export interface ReplaceableTimeout {
  set: (run: () => void, ms: number) => void
  clear: () => void
}

export const replaceableTimeout = (): ReplaceableTimeout => {
  const pending: { id: ReturnType<typeof setTimeout> | null } = { id: null }
  const clear = (): void => {
    if (pending.id !== null) clearTimeout(pending.id)
    pending.id = null
  }
  return {
    set: (run, ms) => {
      clear()
      pending.id = setTimeout(run, ms)
    },
    clear,
  }
}
