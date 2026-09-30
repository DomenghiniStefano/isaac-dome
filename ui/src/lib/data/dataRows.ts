import type { Message } from '@/i18n/message'
import { assertNever } from '@/lib/assertNever'
import { storeReasonPart } from '@/lib/ipc/errorText'
import type { MessagePart } from '@/lib/ipc/errorText'
import { DataFile } from '@/lib/ipc/types'
import type {
  DataFileState,
  DataFileView,
  StoreContents,
} from '@/lib/ipc/types'

// What a file's state decides; the file's own name and hint are added on top.
interface StateLines {
  folder: string | null
  size: string | null
  // What stops the file being read normally — empty when it reads.
  status: MessagePart[]
  contents: MessagePart[]
  // A folder the platform would not name has nothing to open.
  canReveal: boolean
}

// One file of the Data page, as the screen draws it: the lines are decided here, per state,
// so the template only lays them out.
export interface DataRow extends StateLines {
  file: DataFile
  title: Message
  hint: Message
}

export interface DataFormat {
  size: (bytes: number) => string
  count: (n: number) => string
}

const titles: Record<DataFile, { title: Message; hint: Message }> = {
  [DataFile.Database]: { title: 'data.database', hint: 'data.databaseHint' },
  [DataFile.Settings]: { title: 'data.settings', hint: 'data.settingsHint' },
}

const counted = (key: Message, n: number, fmt: DataFormat): MessagePart => ({
  key,
  params: { count: fmt.count(n) },
})

// A queue that does not parse is "unreadable", never zero; the roll line is there only when
// a preset is saved.
const contentLines = (c: StoreContents, fmt: DataFormat): MessagePart[] => [
  counted('data.contents.goals', c.goals, fmt),
  c.queueRows === null
    ? { key: 'data.contents.queueUnknown' }
    : counted('data.contents.queue', c.queueRows, fmt),
  counted('data.contents.sessions', c.sessions, fmt),
  counted('data.contents.runs', c.runs, fmt),
  ...(c.rollSaved ? [{ key: 'data.contents.rollSaved' as const }] : []),
]

const stateLines = (state: DataFileState, fmt: DataFormat): StateLines => {
  switch (state.kind) {
    case 'present':
      return {
        folder: state.folderHint,
        size: fmt.size(state.sizeBytes),
        status: [],
        contents: state.contents ? contentLines(state.contents, fmt) : [],
        canReveal: true,
      }
    case 'notCreated':
      return {
        folder: state.folderHint,
        size: null,
        status: [{ key: 'data.notCreated' }],
        contents: [],
        canReveal: true,
      }
    case 'unreadable':
      return {
        folder: state.folderHint,
        size: null,
        status: [{ key: 'data.unreadable' }, storeReasonPart(state.reason)],
        contents: [],
        canReveal: true,
      }
    case 'folderUnknown':
      return {
        folder: null,
        size: null,
        status: [{ key: 'data.folderUnknown' }],
        contents: [],
        canReveal: false,
      }
    default:
      return assertNever(state)
  }
}

export const dataRow = (entry: DataFileView, fmt: DataFormat): DataRow => ({
  file: entry.file,
  ...titles[entry.file],
  ...stateLines(entry.state, fmt),
})
