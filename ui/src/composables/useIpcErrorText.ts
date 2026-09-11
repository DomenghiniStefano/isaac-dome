import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import type { IpcError } from '@/lib/ipc/types'

// One sentence per IpcError, for every screen that has to say why a command failed. `null` is a
// failure that isn't an IpcError at all: the backend never answered.
export const useIpcErrorText = () => {
  const { t } = useMessages()
  const errorText = (e: IpcError | null): string => {
    if (e === null) return t('ipcErrors.noBackend')
    switch (e.kind) {
      case 'noActiveProfile':
        return t('ipcErrors.noActiveProfile')
      case 'unknownProfile':
        return t('ipcErrors.unknownProfile')
      case 'unreadableSave':
        return `${t('ipcErrors.unreadableSave')} ${e.reason}`
      case 'settingsNotWritable':
        return `${t('ipcErrors.settingsNotWritable')} ${e.reason}`
      case 'unknownTarget':
        return t('ipcErrors.unknownTarget')
      case 'catalogUnavailable':
        return t('ipcErrors.catalogUnavailable')
      case 'storeUnavailable':
        return `${t('ipcErrors.storeUnavailable')} ${e.reason}`
      case 'wikiUnavailable':
        return t('ipcErrors.wikiUnavailable')
      default:
        return assertNever(e)
    }
  }
  return { errorText }
}
