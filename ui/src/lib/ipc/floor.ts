import { Command } from '../constants/commands'
import { call } from './transport'
import type { FloorView, RoomKindView } from './types'

// The grid you painted, ranked by the game's own documented rules. Pure on the Rust side: it
// reads nothing, so calling it again with the same grid answers the same thing.
export const floorCandidates = (
  cells: (RoomKindView | null)[],
): Promise<FloorView> => call(Command.FloorCandidates, { cells })
