import { describe, expect, it } from 'vitest'
import { buttonType } from './buttonType'

describe('buttonType', () => {
  it('types a plain button, so one inside a form does not submit it', () => {
    expect(buttonType('button', undefined)).toBe('button')
  })

  it('leaves an anchor alone: type is not an anchor attribute', () => {
    expect(buttonType('a', undefined)).toBeUndefined()
  })

  it('leaves the child alone: as-child renders the node the caller passed', () => {
    expect(buttonType('button', true)).toBeUndefined()
  })

  it('leaves a component alone: only a real <button> carries the default', () => {
    expect(buttonType({ name: 'RouterLink' }, undefined)).toBeUndefined()
  })
})
