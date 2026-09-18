import { Command } from '../constants/commands'
import { call } from './transport'
import type { PresetView, RollView } from './types'

// The draw: what is on the card, the deck behind it, and what the preset is letting through.
export const roll = (): Promise<RollView> => call(Command.Roll)

// Draws again. The seed is the clock, and it is read in Rust: the frontend never sends one.
export const rollDraw = (): Promise<RollView> => call(Command.RollDraw)

// Every change to the preset is written immediately — there is no save button, by decision —
// and the answer is the whole view, so the counts on the panel follow the tick that caused it.
export const setRollPreset = (preset: PresetView): Promise<RollView> =>
  call(Command.SetRollPreset, { preset })
