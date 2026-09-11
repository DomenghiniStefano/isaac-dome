import { describe, expect, it } from 'vitest'
import { Dlc } from '@/lib/ipc/types'
import { editionLabel } from './editionLabel'

describe('editionLabel', () => {
  it('draws no tag for no edition', () => {
    expect(editionLabel([])).toBe('')
  })

  it('names a single edition', () => {
    expect(editionLabel([Dlc.Repentance])).toBe('Repentance')
  })

  it('orders editions by release, whatever the order given', () => {
    expect(editionLabel([Dlc.RepentancePlus, Dlc.Afterbirth])).toBe(
      'Afterbirth · Repentance+',
    )
  })

  it('names a repeated edition once', () => {
    expect(editionLabel([Dlc.Rebirth, Dlc.Rebirth])).toBe('Rebirth')
  })
})
