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

// `to` is where you dropped it. The row lands there when the graph allows it and as close
// as it allows otherwise: read the returned order, never assume it.
export const queueMove = (
  achievement: number,
  to: number,
): Promise<QueueView> => call(Command.QueueMove, { achievement, to })

// The one-off move from the goals saved before the queue existed. Nothing happens on its
// own: a read never writes.
export const queueImportGoals = (): Promise<QueueView> =>
  call(Command.QueueImportGoals)
