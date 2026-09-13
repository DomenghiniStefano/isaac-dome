import { describe, expect, it } from 'vitest'
import { en } from '@/i18n/messages/en'
import { it as itMessages } from '@/i18n/messages/it'
import { ipcErrorParts } from './errorText'
import type { IoReason, IpcError } from './types'

// Every key the mapping can produce has to exist in both locales. A missing one renders as
// the key itself, which reads as a bug report to the user and fails nothing.
const has = (messages: Record<string, unknown>, key: string): boolean =>
  key
    .split('.')
    .reduce<unknown>(
      (node, part) =>
        node && typeof node === 'object'
          ? (node as Record<string, unknown>)[part]
          : undefined,
      messages,
    ) !== undefined

const IO: IoReason[] = ['notFound', 'permissionDenied', 'other']

// Every variant of every error, so a new one with no text breaks this and not the app.
const EVERY_ERROR: IpcError[] = [
  { kind: 'noActiveProfile' },
  { kind: 'unknownProfile', id: 'x' },
  { kind: 'unreadableSave', reason: { kind: 'tooShort' } },
  { kind: 'unreadableSave', reason: { kind: 'badMagic' } },
  ...IO.map((reason): IpcError => ({
    kind: 'unreadableSave',
    reason: { kind: 'io', reason },
  })),
  { kind: 'settingsNotWritable', reason: { kind: 'configDirUnknown' } },
  { kind: 'settingsNotWritable', reason: { kind: 'encoding' } },
  ...IO.map((reason): IpcError => ({
    kind: 'settingsNotWritable',
    reason: { kind: 'io', reason },
  })),
  { kind: 'unknownTarget' },
  { kind: 'catalogUnavailable' },
  { kind: 'storeUnavailable', reason: { kind: 'dataDirUnknown' } },
  { kind: 'storeUnavailable', reason: { kind: 'dataDirNotCreatable' } },
  { kind: 'storeUnavailable', reason: { kind: 'unreadable' } },
  {
    kind: 'storeUnavailable',
    reason: { kind: 'newerSchema', found: 7, supported: 1 },
  },
  { kind: 'storeUnavailable', reason: { kind: 'queueUnparseable' } },
  { kind: 'wikiUnavailable' },
]

describe('an IpcError becomes message keys, never a sentence from Rust', () => {
  it('says what failed, with nothing to add when there is nothing', () => {
    expect(ipcErrorParts({ kind: 'catalogUnavailable' })).toEqual([
      { key: 'ipcErrors.catalogUnavailable' },
    ])
  })

  it('adds the reason as a second part when the error carries one', () => {
    expect(
      ipcErrorParts({
        kind: 'unreadableSave',
        reason: { kind: 'io', reason: 'permissionDenied' },
      }),
    ).toEqual([
      { key: 'ipcErrors.unreadableSave' },
      { key: 'ipcReasons.ioPermissionDenied' },
    ])
  })

  // The whole point of N2: the two versions arrive as numbers, so the sentence that names
  // them is written in the locale file and not with `format!` in Rust.
  it('hands a newer schema its two versions as values', () => {
    expect(
      ipcErrorParts({
        kind: 'storeUnavailable',
        reason: { kind: 'newerSchema', found: 7, supported: 1 },
      }),
    ).toEqual([
      { key: 'ipcErrors.storeUnavailable' },
      {
        key: 'ipcReasons.storeNewerSchema',
        params: { found: 7, supported: 1 },
      },
    ])
  })

  it('has a text in both locales for every variant it can produce', () => {
    for (const error of EVERY_ERROR) {
      for (const part of ipcErrorParts(error)) {
        expect(has(en, part.key), `en is missing ${part.key}`).toBe(true)
        expect(has(itMessages, part.key), `it is missing ${part.key}`).toBe(
          true,
        )
      }
    }
  })

  it('answers for no backend at all', () => {
    expect(ipcErrorParts(null)).toEqual([{ key: 'ipcErrors.noBackend' }])
  })
})
