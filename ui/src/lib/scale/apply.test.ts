import { describe, expect, it } from 'vitest'
import { ScaleProperty, applyScale, scaleProperties } from './apply'

describe('scaleProperties', () => {
  it('is the factor and the sprite multiple, the two the tokens read', () => {
    expect(scaleProperties(125)).toEqual({
      [ScaleProperty.Scale]: '1.25',
      [ScaleProperty.SpriteMultiple]: '3',
    })
    expect(scaleProperties(100)).toEqual({
      [ScaleProperty.Scale]: '1',
      [ScaleProperty.SpriteMultiple]: '2',
    })
  })

  it('reads a size it never wrote as the default', () => {
    expect(scaleProperties(137)).toEqual(scaleProperties(100))
  })
})

describe('applyScale', () => {
  // No DOM in the suite (the tests run on node), so the element is a stand-in with the one
  // method the function uses: what is worth testing here is that both properties are written.
  it('writes both properties on the element it is given', () => {
    const written = new Map<string, string>()
    const element = {
      style: { setProperty: (k: string, v: string) => written.set(k, v) },
    } as unknown as HTMLElement
    applyScale(50, element)
    expect([...written]).toEqual([
      [ScaleProperty.Scale, '0.5'],
      [ScaleProperty.SpriteMultiple, '1'],
    ])
  })
})
