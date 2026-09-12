// Key caps as Windows prints them. They read the same in every language the app speaks,
// so they're data, not translations. Arrows and Enter are icons: Determination has no
// glyph for them.
export const KeyName = {
  Ctrl: 'Ctrl',
  K: 'K',
  Esc: 'Esc',
  Plus: '+',
  Minus: '−',
  Zero: '0',
} as const
export type KeyName = (typeof KeyName)[keyof typeof KeyName]
