import { invoke, isTauri } from '@tauri-apps/api/core'
import type { Command } from '../constants/commands'

export type CommandName = (typeof Command)[keyof typeof Command]
export type CommandArgs = Record<string, unknown>

// Every wrapper goes through here. Inside Tauri it is invoke(). On the development server in
// a plain browser there is no backend, so a development build answers from the fixtures; in
// a production build the branch is dead and the fixtures module is never bundled.
export const call = async <T>(
  command: CommandName,
  args?: CommandArgs,
): Promise<T> => {
  if (!isTauri() && import.meta.env.DEV) {
    const { answer } = await import('./fixtures')
    return answer<T>(command, args)
  }
  return invoke<T>(command, args)
}
