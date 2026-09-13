import { Command } from '../constants/commands'
import { call } from './transport'

// The session document, opaque on the way there and back: its shape lives in
// `lib/window/sessionDocument.ts`, which is the only thing that parses it. The backend answers
// `null` when the setting is off, so nothing here has to check it twice.
export const windowSession = (): Promise<string | null> =>
  call(Command.WindowSession)

export const setWindowSession = (document: string | null): Promise<void> =>
  call(Command.SetWindowSession, { document })
