import { Command } from '../constants/commands'
import { call } from './transport'
import type { CollectionView } from './types'

// The active profile's item collection joined with the catalog's collectibles.
export const collection = (): Promise<CollectionView> =>
  call(Command.Collection)
