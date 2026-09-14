import { unlockKindText } from '@/components/graph/unlockKindText'
import { dlcNames } from '@/components/wiki/dlcNames'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { assertNever } from '@/lib/assertNever'
import type { DrawerLabels, ToolbarLabels } from '@/components/facets/labels'
import {
  CollectionFacet,
  CollectionSort,
  NoPool,
  QualityValue,
} from '@/lib/collection/collectionFacets'
import { ItemState } from '@/lib/collection/itemState'
import { OriginValue, TargetKind } from '@/lib/ipc/values'
import { Dlc } from '@/lib/ipc/types'

type Translate = (key: MessageKey<MessageSchema>) => string

export const collectionFacetTitle: Record<
  CollectionFacet,
  MessageKey<MessageSchema>
> = {
  [CollectionFacet.State]: 'collection.facet.state',
  [CollectionFacet.Quality]: 'collection.facet.quality',
  [CollectionFacet.Pool]: 'collection.facet.pool',
  [CollectionFacet.Kind]: 'collection.facet.kind',
  [CollectionFacet.Origin]: 'collection.facet.origin',
}

export const itemStateText: Record<ItemState, MessageKey<MessageSchema>> = {
  [ItemState.InCollection]: 'collection.state.inCollection',
  [ItemState.Available]: 'collection.state.available',
  [ItemState.Locked]: 'collection.state.locked',
  [ItemState.Unknown]: 'collection.state.unknown',
}

// The origin DLC's names are game data, the same as the wiki's editions; only "not stated" is
// ours to say.
const originName: Record<OriginValue, string | null> = {
  [OriginValue.Rebirth]: dlcNames[Dlc.Rebirth],
  [OriginValue.Afterbirth]: dlcNames[Dlc.Afterbirth],
  [OriginValue.AfterbirthPlus]: dlcNames[Dlc.AfterbirthPlus],
  [OriginValue.Repentance]: dlcNames[Dlc.Repentance],
  [OriginValue.None]: null,
}

const find = <T extends string>(values: Record<string, T>, value: string) =>
  Object.values(values).find((v) => v === value)

// A facet value in words. Quality numbers and pool names are shown as they are: a number is a
// number, a pool is the game's own word.
export const collectionFacetValueLabel = (
  t: Translate,
  facet: CollectionFacet,
  value: string,
): string => {
  switch (facet) {
    case CollectionFacet.State: {
      const state = find(ItemState, value)
      return state ? t(itemStateText[state]) : value
    }
    case CollectionFacet.Quality:
      return value === QualityValue.Unrated
        ? t('collection.qualityUnrated')
        : value
    case CollectionFacet.Pool:
      return value === NoPool ? t('collection.poolNone') : value
    case CollectionFacet.Kind: {
      const kind = find(TargetKind, value)
      return kind ? t(unlockKindText[kind]) : value
    }
    case CollectionFacet.Origin: {
      const origin = find(OriginValue, value)
      if (!origin) return value
      return originName[origin] ?? t('graph.originNone')
    }
    default:
      return assertNever(facet)
  }
}

// A state is never colour alone: the square carries its colour, the name says it. Unreadable
// wears the unknown hatch, as its badge does.
export const itemStateDot: Record<ItemState, string> = {
  [ItemState.InCollection]: 'bg-state-done',
  [ItemState.Available]: 'bg-state-now',
  [ItemState.Locked]: 'bg-state-blocked',
  [ItemState.Unknown]:
    'hatch-unknown border border-dashed border-state-unknown',
}

// The facets the drawer holds: the state has its own control above the table.
export const drawerFacets: CollectionFacet[] = [
  CollectionFacet.Quality,
  CollectionFacet.Pool,
  CollectionFacet.Kind,
  CollectionFacet.Origin,
]

export const sortOrder: CollectionSort[] = [
  CollectionSort.Quality,
  CollectionSort.Id,
  CollectionSort.Name,
]

export const sortText: Record<CollectionSort, MessageKey<MessageSchema>> = {
  [CollectionSort.Quality]: 'collection.sort.quality',
  [CollectionSort.Id]: 'collection.sort.id',
  [CollectionSort.Name]: 'collection.sort.name',
}

export const toolbarLabels: ToolbarLabels = {
  rows: 'collection.items',
  search: 'collection.search',
  sortBy: 'collection.sortBy',
  activeFilters: 'collection.activeFilters',
}

export const drawerLabels: DrawerLabels = {
  facets: 'collection.facets',
  activeFilters: 'collection.activeFilters',
  noFilters: 'collection.noFilters',
  reset: 'collection.reset',
}
