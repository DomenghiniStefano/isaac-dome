import type { DataView } from '../types'
import { DataFile } from '../types'

// Development only: the Data page as a machine that has been used for a while — both files
// there, a database with something in every table.
const folderHint = String.raw`C:\Users\<user>\AppData\Roaming\dev.isaacdome.app`

export const dataAnswer = async (): Promise<DataView> => ({
  files: [
    {
      file: DataFile.Database,
      state: {
        kind: 'present',
        folderHint,
        sizeBytes: 1_843_200,
        contents: {
          goals: 4,
          queueRows: 17,
          sessions: 38,
          runs: 112,
          rollSaved: true,
        },
      },
    },
    {
      file: DataFile.Settings,
      state: { kind: 'present', folderHint, sizeBytes: 412, contents: null },
    },
  ],
})
