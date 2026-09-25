import type { IpcError } from './types'

// Every tag the contract declares for an `IpcError`, and only those. A `Record` over the union's
// tags, so a kind added in Rust and regenerated into `types.ts` fails the build here until it is
// listed — and a rejection carrying some other `kind` (a Tauri plugin's own error, a DOM
// exception) is not mistaken for one of ours and handed to a sentence that does not fit it.
const ipcErrorKinds: Record<IpcError['kind'], true> = {
  noActiveProfile: true,
  unknownProfile: true,
  unreadableSave: true,
  settingsNotWritable: true,
  unknownTarget: true,
  catalogUnavailable: true,
  storeUnavailable: true,
  wikiUnavailable: true,
  sessionTooLarge: true,
  autostartNotWritable: true,
  updateNotReady: true,
}

// A rejection from a command is an IpcError when it carries one of the contract's tags.
// Anything else (the backend never answered, a thrown Error, a foreign object with a `kind`) is
// not, and the stores keep it as `null`.
export const isIpcError = (e: unknown): e is IpcError =>
  typeof e === 'object' &&
  e !== null &&
  'kind' in e &&
  typeof e.kind === 'string' &&
  Object.hasOwn(ipcErrorKinds, e.kind)

// What a store keeps of a rejection: ours, or nothing it can say anything about.
export const asIpcError = (e: unknown): IpcError | null =>
  isIpcError(e) ? e : null
