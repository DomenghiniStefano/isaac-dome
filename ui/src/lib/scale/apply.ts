import { scaleFactor, spriteMultiple } from './steps'

// The two custom properties the whole interface is drawn from: the root's font size is
// `calc(16px * var(--app-scale))`, so every rem token follows, and the sprite tokens are
// `calc(32px * var(--sprite-multiple))`, so pixel art stays on whole multiples.
export const ScaleProperty = {
  Scale: '--app-scale',
  SpriteMultiple: '--sprite-multiple',
} as const
export type ScaleProperty = (typeof ScaleProperty)[keyof typeof ScaleProperty]

export const scaleProperties = (percent: number): Record<string, string> => ({
  [ScaleProperty.Scale]: String(scaleFactor(percent)),
  [ScaleProperty.SpriteMultiple]: String(spriteMultiple(percent)),
})

// Writing them on the root is the whole mechanism: no component knows the size.
export const applyScale = (
  percent: number,
  element: HTMLElement = document.documentElement,
): void => {
  for (const [property, value] of Object.entries(scaleProperties(percent)))
    element.style.setProperty(property, value)
}
