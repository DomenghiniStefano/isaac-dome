import type { MessageSchema } from '@/i18n/messages/it'
import type { MessageKey } from '@/i18n/messageKey'
import { assertNever } from '@/lib/assertNever'
import type {
  IoReason,
  IpcError,
  SaveReason,
  SettingsReason,
  StoreReason,
} from './types'

// One piece of the sentence: a key, and the values the translation puts inside it. The word
// order around a value is the translation's business, which is why the values travel as
// values and not inside a string built in Rust.
export interface MessagePart {
  key: MessageKey<MessageSchema>
  params?: Record<string, unknown>
}

const io = (reason: IoReason): MessageKey<MessageSchema> => {
  switch (reason) {
    case 'notFound':
      return 'ipcReasons.ioNotFound'
    case 'permissionDenied':
      return 'ipcReasons.ioPermissionDenied'
    case 'other':
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
      return { key: io(reason.reason) }
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
      return { key: io(reason.reason) }
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
    default:
      return assertNever(e)
  }
}
