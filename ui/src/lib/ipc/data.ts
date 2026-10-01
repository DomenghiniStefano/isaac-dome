import { Command } from '../constants/commands'
import { call } from './transport'
import type { DataFile, DataView } from './types'

// Where the app writes and what is there. The folder arrives as a hint with the username
// masked; the frontend never holds a path.
export const dataLocation = (): Promise<DataView> => call(Command.DataLocation)

// Explorer on the file, selected. The file is named, never located: the backend resolves it.
export const revealDataFile = (file: DataFile): Promise<void> =>
  call(Command.RevealDataFile, { file })
