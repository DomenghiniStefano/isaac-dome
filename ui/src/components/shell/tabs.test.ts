import { describe, expect, it } from 'vitest'
import { DropSide, dropSide, moveIndex } from './tabs'

describe('dropSide', () => {
  it('lands before a tab when the pointer is on its left half', () => {
    expect(dropSide(110, 100, 40)).toBe(DropSide.Before)
  })

  it('lands after a tab when the pointer is on its right half', () => {
    expect(dropSide(135, 100, 40)).toBe(DropSide.After)
  })

  it('counts the exact middle as after', () => {
    expect(dropSide(120, 100, 40)).toBe(DropSide.After)
  })
})

describe('moveIndex', () => {
  it('moves a tab right, after the target', () => {
    expect(moveIndex(0, 2, DropSide.After)).toBe(2)
  })

  it('moves a tab right, before the target', () => {
    expect(moveIndex(0, 2, DropSide.Before)).toBe(1)
  })

  it('moves a tab left, before the target', () => {
    expect(moveIndex(3, 1, DropSide.Before)).toBe(1)
  })

  it('moves a tab left, after the target', () => {
    expect(moveIndex(3, 1, DropSide.After)).toBe(2)
  })

  it('leaves a tab dropped on itself where it is', () => {
    expect(moveIndex(2, 2, DropSide.Before)).toBe(2)
    expect(moveIndex(2, 2, DropSide.After)).toBe(2)
  })
})
