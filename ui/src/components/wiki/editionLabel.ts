import { Dlc } from '@/lib/ipc/types'
import { dlcNames } from './dlcNames'

// The tag an edition-scoped passage carries: the editions' names in release order (the
// declaration order of Dlc), each once. No edition gives no text, and no tag is drawn.
export const editionLabel = (only: Dlc[]): string =>
  Object.values(Dlc)
    .filter((dlc) => only.includes(dlc))
    .map((dlc) => dlcNames[dlc])
    .join(' · ')
