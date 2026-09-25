import { Command } from '../constants/commands'
import { call } from './transport'
import type { GraphViews, Target, WantView } from './types'

// N8: one call, one reading of the profile. They were two commands and the store asked
// for both together — which two commands could not honour, because between them a save
// written mid-load made the steps disagree with the list they are a filter over.
export const graphViews = (): Promise<GraphViews> => call(Command.GraphViews)

// B37: the graph read from the other end. `target` is the name of the Rust command's
// parameter, and it is the same `Target` a wiki page is keyed by — one vocabulary, two
// questions ("what is this?" and "what do I play for it?").
export const want = (target: Target): Promise<WantView> =>
  call(Command.Want, { target })
