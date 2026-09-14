import { unlockKindText } from '@/components/graph/unlockKindText'
import { dlcNames } from '@/components/wiki/dlcNames'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { assertNever } from '@/lib/assertNever'
import {
  CollectionFacet,
  NoPool,
  QualityValue,
} from '@/lib/collection/collectionFilter'
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
