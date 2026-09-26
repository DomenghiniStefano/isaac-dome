import { describe, expect, it } from 'vitest'
import { cn } from './cn'

describe('cn', () => {
  it('keeps a font size beside a text colour', () => {
    expect(cn('text-body', 'text-foreground')).toBe('text-body text-foreground')
  })

  it('lets a later font size replace an earlier one', () => {
    expect(cn('text-body', 'text-caption')).toBe('text-caption')
  })

  it('lets a later spacing token replace an earlier one', () => {
    expect(cn('h-row', 'h-control')).toBe('h-control')
  })

  it('lets a later radius replace an earlier one', () => {
    expect(cn('rounded-input', 'rounded-cell')).toBe('rounded-cell')
  })

  it('lets a later colour replace an earlier one', () => {
    expect(cn('bg-primary', 'bg-secondary')).toBe('bg-secondary')
  })

  it('lets a later duration and easing replace earlier ones', () => {
    expect(cn('duration-tap ease-tap', 'duration-panel ease-panel')).toBe(
      'duration-panel ease-panel',
    )
  })

  it('lets a later animation replace an earlier one', () => {
    expect(cn('animate-panel-rise', 'animate-panel-drop')).toBe(
      'animate-panel-drop',
    )
  })

  it('lets a later opacity token replace an earlier one', () => {
    expect(cn('opacity-muted', 'opacity-disabled')).toBe('opacity-disabled')
  })

  it('still lets a number replace an opacity token', () => {
    expect(cn('opacity-muted', 'opacity-0')).toBe('opacity-0')
  })

  it('lets a later stacking layer replace an earlier one', () => {
    expect(cn('z-raised', 'z-overlay')).toBe('z-overlay')
    expect(cn('z-raised-corner', 'z-raised-header')).toBe('z-raised-header')
  })

  it('lets a later letter spacing replace an earlier one', () => {
    expect(cn('tracking-nav', 'tracking-caps')).toBe('tracking-caps')
  })

  it('drops falsy inputs', () => {
    expect(cn('text-body', false, undefined, 'text-foreground')).toBe(
      'text-body text-foreground',
    )
  })
})
