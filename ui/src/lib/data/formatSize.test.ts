import { describe, expect, it } from 'vitest'
import { SizeUnit, sizeUnit } from './formatSize'

describe('sizeUnit', () => {
  it('keeps bytes below one kilobyte', () => {
    expect(sizeUnit(0)).toEqual({ value: 0, unit: SizeUnit.Byte })
    expect(sizeUnit(1023)).toEqual({ value: 1023, unit: SizeUnit.Byte })
  })
  it('switches to kilobytes at 1024', () => {
    expect(sizeUnit(1024)).toEqual({ value: 1, unit: SizeUnit.Kilobyte })
    expect(sizeUnit(1536)).toEqual({ value: 1.5, unit: SizeUnit.Kilobyte })
  })
  it('switches to megabytes at 1024 × 1024', () => {
    expect(sizeUnit(1024 * 1024)).toEqual({
      value: 1,
      unit: SizeUnit.Megabyte,
    })
    expect(sizeUnit(5 * 1024 * 1024 + 512 * 1024)).toEqual({
      value: 5.5,
      unit: SizeUnit.Megabyte,
    })
  })
})
