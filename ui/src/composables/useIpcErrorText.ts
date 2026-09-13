import { useMessages } from '@/i18n'
import { ipcErrorParts } from '@/lib/ipc/errorText'
import type { IpcError } from '@/lib/ipc/types'

// One sentence per IpcError, for every screen that has to say why a command failed. The
// mapping onto keys is a pure function next to the wire types and is tested there; this
// only joins the parts. `null` is a failure that isn't an IpcError at all: the backend
// never answered.
export const useIpcErrorText = () => {
  const { t } = useMessages()
  const errorText = (e: IpcError | null): string =>
    ipcErrorParts(e)
      .map((p) => t(p.key, p.params))
      .join(' ')
  return { errorText }
}
