import { describe, expect, it } from 'vitest'
import { dlcNames } from '@/components/wiki/dlcNames'
import { Dlc } from '@/lib/ipc/types'
import { OriginValue } from '@/lib/ipc/values'
import { originLabel } from './labels'
import type { Translate } from './labels'

// The label only ever reads a key back, so the key is what a test can assert on.
const t = ((key: string) => key) as Translate

describe('originLabel', () => {
  it('names the DLC, in the words the wiki uses for an edition', () => {
    expect(originLabel(t, OriginValue.Repentance)).toBe(
      dlcNames[Dlc.Repentance],
    )
  })

  // The catalog infers the origin from the id ranges and cannot always tell: that case is ours
  // to word, and it is the only one of the five that is.
  it('says so when the catalog cannot tell', () => {
    expect(originLabel(t, OriginValue.None)).toBe('graph.originNone')
  })

  it('shows a value outside the set as it came, rather than dropping it', () => {
    expect(originLabel(t, 'afterbirth-plus-plus')).toBe('afterbirth-plus-plus')
  })
})
