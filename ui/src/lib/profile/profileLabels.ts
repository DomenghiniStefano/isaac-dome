import type { Message } from '@/i18n/message'
import { CandidateSource, MissingReason } from '@/lib/ipc/types'
import { ChainLink, LinkState } from './profileView'

// Records over closed sets: a new value with no message fails to compile.
export const candidateSourceLabel: Record<CandidateSource, Message> = {
  [CandidateSource.SteamCloud]: 'profile.sources.steamCloud',
  [CandidateSource.Documents]: 'profile.sources.documents',
  [CandidateSource.Manual]: 'profile.sources.manual',
}

export const missingReasonLabel: Record<MissingReason, Message> = {
  [MissingReason.SteamNotFound]: 'profile.none.steamNotFound',
  [MissingReason.GameNotFound]: 'profile.none.gameNotFound',
  [MissingReason.NoSaves]: 'profile.none.noSaves',
  [MissingReason.NoSavesInChosenFolder]: 'profile.none.noSavesInChosenFolder',
}

export const chainLinkLabel: Record<ChainLink, Message> = {
  [ChainLink.Steam]: 'profile.chain.steam',
  [ChainLink.Game]: 'profile.chain.game',
  [ChainLink.Saves]: 'profile.chain.saves',
}

export const linkStateLabel: Record<LinkState, Message> = {
  [LinkState.Found]: 'profile.chain.found',
  [LinkState.Missing]: 'profile.chain.missing',
  [LinkState.YourChoice]: 'profile.chain.yourChoice',
  [LinkState.Several]: 'profile.chain.several',
  [LinkState.Chosen]: 'profile.chain.chosen',
}
