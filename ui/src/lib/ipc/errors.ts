import type { IpcError } from './types'

// A rejection from a command is an IpcError when it carries its tag. Anything else (the
// backend never answered, a thrown Error) is not, and the stores keep it as `null`.
export const isIpcError = (e: unknown): e is IpcError =>
  typeof e === 'object' && e !== null && 'kind' in e
