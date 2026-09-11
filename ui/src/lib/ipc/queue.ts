import { Command } from '../constants/commands'
import { call } from './transport'
import type { QueueView } from './types'

export const queue = (): Promise<QueueView> => call(Command.Queue)

// Every mutation answers with the whole view: a move can reorder much of the queue, and
// the backend hands back the new truth rather than teaching the frontend to replay the
// repair. The parameter names are the Rust commands'.
export const queueAdd = (achievement: number): Promise<QueueView> =>
  call(Command.QueueAdd, { achievement })

export const queueRemove = (achievement: number): Promise<QueueView> =>
  call(Command.QueueRemove, { achievement })

// `after` is the row it was dropped under, `null` for the top: a row and not an index, because
// the view leaves completed and unresolved rows out and its positions are not the file's. The
// row lands there when the graph allows it and as close as it allows otherwise: read the
// returned order, never assume it.
export const queueMove = (
  achievement: number,
  after: number | null,
): Promise<QueueView> => call(Command.QueueMove, { achievement, after })

// The one-off move from the goals saved before the queue existed. Nothing happens on its
// own: a read never writes.
export const queueImportGoals = (): Promise<QueueView> =>
  call(Command.QueueImportGoals)
