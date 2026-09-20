import { Command } from '../constants/commands'
import { call } from './transport'
import type { UpdateView } from './types'

// Updating the app. The three of these are the only door: nothing in a component talks to the
// updater plugin, and the frontend never learns an endpoint, a URL or a byte count.

// The held phase and the running version. **No network**: it answers the same on a machine
// that has never been online, which is what lets the screen open instantly.
export const updateStatus = (): Promise<UpdateView> =>
  call(Command.UpdateStatus)

// Looks, and downloads whatever it finds — the button, and the same path the launch takes.
// Answers the view as it stands when it is over; the windows follow the download through the
// `update-changed` event while it runs.
export const checkUpdate = (): Promise<UpdateView> => call(Command.CheckUpdate)

// **On Windows this never comes back when it works**: the installer is launched and the
// process ends. A rejected promise means the bytes were not there to install, which is a
// defect of ours; an installer that would not run is not an error, it is the next phase.
export const installUpdate = (): Promise<void> => call(Command.InstallUpdate)
