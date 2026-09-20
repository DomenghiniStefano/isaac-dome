import { Command } from '../constants/commands'
import { call } from './transport'
import type { FloorView, RoomIconView, RoomKindView } from './types'

// The grid you painted, ranked by the game's own documented rules. Pure on the Rust side: it
// reads nothing, so calling it again with the same grid answers the same thing.
export const floorCandidates = (
  cells: (RoomKindView | null)[],
): Promise<FloorView> => call(Command.FloorCandidates, { cells })

// The game's own minimap icons, one row per room kind. Asked once when the screen opens and
// never again: the pictures do not change with what is painted, and the answer is empty on a
// machine without the game — which is not a failure, it is the screen drawing its own symbols.
export const roomIcons = (): Promise<RoomIconView[]> => call(Command.RoomIcons)
