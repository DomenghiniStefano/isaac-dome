import { describe, expect, it } from 'vitest'
import type { CandidatePreview } from '@/lib/ipc/types'
import { previewLines, unreadableNote } from './previewView'

const full: CandidatePreview = {
  achievements: { kind: 'read', done: 379, of: 640 },
  items: { kind: 'read', done: 612, of: 732 },
  marks: { kind: 'read', done: 92, of: 120 },
  unreadableCells: 0,
}

describe('previewLines', () => {
  it('gives one line per count, in the order the card reads', () => {
    expect(previewLines(full).map((l) => l.label)).toEqual([
      'welcome.card.achievements',
      'welcome.card.items',
      'welcome.card.marks',
    ])
    expect(previewLines(full)[0]).toMatchObject({ done: 379, of: 640 })
  })

  it('says a count could not be read instead of showing a zero', () => {
    const [line] = previewLines({ ...full, achievements: { kind: 'unread' } })
    expect(line?.done).toBeNull()
    expect(line?.of).toBeNull()
  })

  it('gives no lines at all when the file could not be parsed', () => {
    expect(previewLines(null)).toEqual([])
  })
})

describe('unreadableNote', () => {
  it('is silent on a whole file', () => {
    expect(unreadableNote(full)).toBeNull()
  })

  it('counts the cells when there are any', () => {
    expect(unreadableNote({ ...full, unreadableCells: 8 })).toEqual({
      key: 'welcome.card.unreadableCells',
      count: 8,
    })
  })

  it('speaks for a file that could not be read at all', () => {
    expect(unreadableNote(null)).toEqual({
      key: 'welcome.card.unreadable',
      count: 0,
    })
  })
})
