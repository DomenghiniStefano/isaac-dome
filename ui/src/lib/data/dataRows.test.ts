import { describe, expect, it } from 'vitest'
import { DataFile } from '@/lib/ipc/types'
import type { DataFileView, StoreContents } from '@/lib/ipc/types'
import { dataRow } from './dataRows'

// Formatters that show what they were given, so a test reads which value reached which line.
const fmt = {
  size: (bytes: number) => `${bytes} B`,
  count: (n: number) => `#${n}`,
}

const folderHint = String.raw`C:\Users\<user>\AppData\Roaming\x`

const contents: StoreContents = {
  goals: 4,
  queueRows: 17,
  sessions: 38,
  runs: 112,
  rollSaved: true,
}

const database = (state: DataFileView['state']): DataFileView => ({
  file: DataFile.Database,
  state,
})

describe('dataRow', () => {
  it('names each file with its own title and hint', () => {
    const db = dataRow(database({ kind: 'folderUnknown' }), fmt)
    const settings = dataRow(
      { file: DataFile.Settings, state: { kind: 'folderUnknown' } },
      fmt,
    )
    expect([db.title, db.hint]).toEqual(['data.database', 'data.databaseHint'])
    expect([settings.title, settings.hint]).toEqual([
      'data.settings',
      'data.settingsHint',
    ])
  })

  it('shows a present database with its folder, size and every count', () => {
    const row = dataRow(
      database({ kind: 'present', folderHint, sizeBytes: 2048, contents }),
      fmt,
    )
    expect(row.folder).toBe(folderHint)
    expect(row.size).toBe('2048 B')
    expect(row.status).toEqual([])
    expect(row.contents).toEqual([
      { key: 'data.contents.goals', params: { count: '#4' } },
      { key: 'data.contents.queue', params: { count: '#17' } },
      { key: 'data.contents.sessions', params: { count: '#38' } },
      { key: 'data.contents.runs', params: { count: '#112' } },
      { key: 'data.contents.rollSaved' },
    ])
    expect(row.canReveal).toBe(true)
  })

  it('says a queue that does not parse is unreadable, never zero', () => {
    const row = dataRow(
      database({
        kind: 'present',
        folderHint,
        sizeBytes: 1,
        contents: { ...contents, queueRows: null },
      }),
      fmt,
    )
    expect(row.contents).toContainEqual({ key: 'data.contents.queueUnknown' })
    expect(row.contents.map((p) => p.key)).not.toContain('data.contents.queue')
  })

  it('leaves the roll line out when no preset is saved', () => {
    const row = dataRow(
      database({
        kind: 'present',
        folderHint,
        sizeBytes: 1,
        contents: { ...contents, rollSaved: false },
      }),
      fmt,
    )
    expect(row.contents.map((p) => p.key)).not.toContain(
      'data.contents.rollSaved',
    )
  })

  it('shows a settings file with no contents list', () => {
    const row = dataRow(
      {
        file: DataFile.Settings,
        state: { kind: 'present', folderHint, sizeBytes: 412, contents: null },
      },
      fmt,
    )
    expect(row.size).toBe('412 B')
    expect(row.contents).toEqual([])
  })

  it('says a file not created yet, keeps its folder, and still reveals', () => {
    const row = dataRow(database({ kind: 'notCreated', folderHint }), fmt)
    expect(row.folder).toBe(folderHint)
    expect(row.size).toBeNull()
    expect(row.status).toEqual([{ key: 'data.notCreated' }])
    expect(row.canReveal).toBe(true)
  })

  it('says an unreadable database and why, in the store’s own words', () => {
    const row = dataRow(
      database({
        kind: 'unreadable',
        folderHint,
        reason: { kind: 'newerSchema', found: 9, supported: 6 },
      }),
      fmt,
    )
    expect(row.status).toEqual([
      { key: 'data.unreadable' },
      {
        key: 'ipcReasons.storeNewerSchema',
        params: { found: 9, supported: 6 },
      },
    ])
    expect(row.canReveal).toBe(true)
  })

  it('offers no reveal for a folder the platform would not name', () => {
    const row = dataRow(database({ kind: 'folderUnknown' }), fmt)
    expect(row.folder).toBeNull()
    expect(row.status).toEqual([{ key: 'data.folderUnknown' }])
    expect(row.canReveal).toBe(false)
  })
})
