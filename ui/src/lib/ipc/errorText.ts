import type { Message } from '@/i18n/message'
import { assertNever } from '@/lib/assertNever'
import { AutostartFailure, IoReason } from './types'
import type { IpcError, SaveReason, SettingsReason, StoreReason } from './types'

// One piece of the sentence: a key, and the values the translation puts inside it. The word
// order around a value is the translation's business, which is why the values travel as
// values and not inside a string built in Rust.
export interface MessagePart {
  key: Message
  params?: Record<string, unknown>
}

// Exported for the setup diagnostics, which name a path the app could not read and why (card
// #81, V9: that reason used to reach the screen as its raw wire value).
export const ioReasonKey = (reason: IoReason): Message => {
  switch (reason) {
    case IoReason.NotFound:
      return 'ipcReasons.ioNotFound'
    case IoReason.PermissionDenied:
      return 'ipcReasons.ioPermissionDenied'
    case IoReason.Other:
      return 'ipcReasons.ioOther'
    default:
      return assertNever(reason)
  }
}

const save = (reason: SaveReason): MessagePart => {
  switch (reason.kind) {
    case 'tooShort':
      return { key: 'ipcReasons.saveTooShort' }
    case 'badMagic':
      return { key: 'ipcReasons.saveBadMagic' }
    case 'io':
      return { key: ioReasonKey(reason.reason) }
    default:
      return assertNever(reason)
  }
}

const settings = (reason: SettingsReason): MessagePart => {
  switch (reason.kind) {
    case 'configDirUnknown':
      return { key: 'ipcReasons.settingsConfigDirUnknown' }
    case 'encoding':
      return { key: 'ipcReasons.settingsEncoding' }
    case 'io':
      return { key: ioReasonKey(reason.reason) }
    default:
      return assertNever(reason)
  }
}

// Exported because the Background screen says this one itself, with the switch in front of the
// user: there the sentence is the whole message, and on the verification page it is the second
// half of one.
export const autostartFailurePart = (reason: AutostartFailure): MessagePart => {
  switch (reason) {
    case AutostartFailure.WriteRefused:
      return { key: 'ipcReasons.autostartWriteRefused' }
    case AutostartFailure.WriteIgnored:
      return { key: 'ipcReasons.autostartWriteIgnored' }
    default:
      return assertNever(reason)
  }
}

export const storeReasonPart = (reason: StoreReason): MessagePart => {
  switch (reason.kind) {
    case 'dataDirUnknown':
      return { key: 'ipcReasons.storeDataDirUnknown' }
    case 'dataDirNotCreatable':
      return { key: 'ipcReasons.storeDataDirNotCreatable' }
    case 'unreadable':
      return { key: 'ipcReasons.storeUnreadable' }
    case 'newerSchema':
      return {
        key: 'ipcReasons.storeNewerSchema',
        params: { found: reason.found, supported: reason.supported },
      }
    case 'queueUnparseable':
      return { key: 'ipcReasons.storeQueueUnparseable' }
    default:
      return assertNever(reason)
  }
}

// What failed, and why, as keys. `null` is a failure that isn't an `IpcError` at all: the
// backend never answered.
export const ipcErrorParts = (e: IpcError | null): MessagePart[] => {
  if (e === null) return [{ key: 'ipcErrors.noBackend' }]
  switch (e.kind) {
    case 'noActiveProfile':
      return [{ key: 'ipcErrors.noActiveProfile' }]
    case 'unknownProfile':
      return [{ key: 'ipcErrors.unknownProfile' }]
    case 'unreadableSave':
      return [{ key: 'ipcErrors.unreadableSave' }, save(e.reason)]
    case 'settingsNotWritable':
      return [{ key: 'ipcErrors.settingsNotWritable' }, settings(e.reason)]
    case 'unknownTarget':
      return [{ key: 'ipcErrors.unknownTarget' }]
    case 'catalogUnavailable':
      return [{ key: 'ipcErrors.catalogUnavailable' }]
    case 'storeUnavailable':
      return [{ key: 'ipcErrors.storeUnavailable' }, storeReasonPart(e.reason)]
    case 'wikiUnavailable':
      return [{ key: 'ipcErrors.wikiUnavailable' }]
    // Nothing the user did and nothing they can do: the tabs are on screen either way, and
    // this is only ever seen on the verification page. It still gets a sentence, because a
    // branch that returned nothing would be the one case with no text at all.
    case 'sessionTooLarge':
      return [{ key: 'ipcErrors.sessionTooLarge' }]
    // The Background screen says this one itself, with the switch in front of the user. The
    // sentence exists for the verification page, where every error has to have one.
    case 'autostartNotWritable':
      return [
        { key: 'ipcErrors.autostartNotWritable' },
        autostartFailurePart(e.reason),
      ]
    // A window asked to install with nothing downloaded: a defect of ours, not something that
    // happened to the user, and the only error the updater can raise at all — everything they
    // can really run into is a phase on the Updates screen.
    case 'updateNotReady':
      return [{ key: 'ipcErrors.updateNotReady' }]
    default:
      return assertNever(e)
  }
}
