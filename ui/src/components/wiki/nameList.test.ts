import { describe, expect, it } from 'vitest'
import type {
  Block,
  Inline,
  ListItem,
  Target,
  UnlockNode,
} from '@/lib/ipc/types'
import { Dlc, Style } from '@/lib/ipc/types'
import { isNameItem, isNameList, nameStatus, RefArt, refArt } from './nameList'

const ref = (label: string): Inline => ({
  kind: 'ref',
  target: { kind: 'achievement', id: 1 },
  label,
})
const text = (t: string): Inline => ({
  kind: 'text',
  text: t,
  style: Style.Plain,
})
const item = (...inline: Inline[]): ListItem => ({ inline, children: [] })

describe('isNameItem', () => {
  it('is an item that holds one reference and nothing else', () => {
    expect(isNameItem(item(ref('Cain')))).toBe(true)
  })

  it('looks through the edition every item of a `dlc =` list carries', () => {
    const inEdition: Inline = {
      kind: 'edition',
      only: [Dlc.Repentance],
      inline: [ref('Breakfast')],
    }
    expect(isNameItem(item(inEdition))).toBe(true)
  })

  it('is not an item whose reference opens a sentence', () => {
    expect(isNameItem(item(ref("Guppy's Paw"), text(', since…')))).toBe(false)
  })

  it('is not an item of text alone, even a name that did not resolve', () => {
    expect(isNameItem(item(text('Not An Achievement')))).toBe(false)
  })

  it('is not an item with two references', () => {
    expect(isNameItem(item(ref('A'), ref('B')))).toBe(false)
  })

  it('is not an empty item', () => {
    expect(isNameItem(item())).toBe(false)
  })
})

describe('refArt', () => {
  it('draws an achievement as its drawing, which needs the paper under it', () => {
    expect(refArt({ kind: 'achievement', id: 1 })).toBe(RefArt.Drawing)
  })

  it('draws every other target as pixel art', () => {
    const sprites: Target[] = [
      { kind: 'item', id: 1 },
      { kind: 'trinket', id: 1 },
      { kind: 'character', id: 0 },
      { kind: 'entity', id: 84, variant: 0, subtype: 0 },
    ]
    for (const target of sprites) expect(refArt(target)).toBe(RefArt.Sprite)
  })
})

describe('isNameList', () => {
  const list = (...items: ListItem[]): Block => ({
    kind: 'list',
    ordered: false,
    items,
  })

  it('is a list whose every item is a name', () => {
    expect(isNameList(list(item(ref('Cain')), item(ref('Judas'))))).toBe(true)
  })

  it('keeps an item whose unlock condition sits under it a name', () => {
    const withCondition: ListItem = {
      inline: [ref('Judas')],
      children: [{ kind: 'paragraph', inline: [text('Defeat Satan')] }],
    }
    expect(isNameList(list(withCondition))).toBe(true)
  })

  it('is not a list with one item of prose among the names', () => {
    expect(isNameList(list(item(ref('Cain')), item(text('Not a name'))))).toBe(
      false,
    )
  })

  it('is not an ordered list, which counts steps and not names', () => {
    expect(
      isNameList({ kind: 'list', ordered: true, items: [item(ref('Cain'))] }),
    ).toBe(false)
  })

  it('is not an empty list', () => {
    expect(isNameList(list())).toBe(false)
  })

  it('is not a paragraph', () => {
    expect(isNameList({ kind: 'paragraph', inline: [ref('Cain')] })).toBe(false)
  })
})

describe('nameStatus', () => {
  const node = (id: number, done: boolean): UnlockNode => ({
    achievement: {
      kind: 'known',
      id,
      text: `t${id}`,
      condition: null,
      iconUrl: null,
    },
    done,
    unlocks: [],
    origin: null,
    missing: [],
    graph: {
      kind: 'computed',
      availableNow: true,
      blockedBy: 0,
      fanOut: 0,
      stepsMissing: 0,
    },
  })
  const nodes = new Map([
    [3, node(3, true)],
    [4, node(4, false)],
  ])
  const nodeFor = (t: Target): UnlockNode | null =>
    t.kind === 'achievement' ? (nodes.get(t.id) ?? null) : null

  it('says an achievement the save has done is done', () => {
    const status = nameStatus({ kind: 'achievement', id: 3 }, nodeFor)
    expect(status.node?.achievement).toMatchObject({ id: 3 })
    expect(status.done).toBe(true)
  })

  it('carries the node of one not done yet, and does not call it done', () => {
    const status = nameStatus({ kind: 'achievement', id: 4 }, nodeFor)
    expect(status.node).not.toBeNull()
    expect(status.done).toBe(false)
  })

  it('says nothing without a profile to ask', () => {
    expect(nameStatus({ kind: 'achievement', id: 3 }, undefined)).toEqual({
      node: null,
      done: false,
    })
  })

  it('says nothing about a name the profile does not know', () => {
    expect(nameStatus({ kind: 'item', id: 105 }, nodeFor)).toEqual({
      node: null,
      done: false,
    })
  })
})
