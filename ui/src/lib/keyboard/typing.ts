// Whether a key press was somebody writing, rather than somebody reaching for a shortcut.
//
// A screen-wide single-key shortcut has exactly one way to go wrong: the moment a field lands
// on the same screen, every letter typed into it also fires the shortcut. Nothing fails, and
// nothing says why the grid keeps repainting itself — so the guard goes in before the field
// does, not after.

const FIELDS = ['INPUT', 'TEXTAREA', 'SELECT']

export const isTyping = (target: EventTarget | null): boolean => {
  if (target === null) return false
  const element = target as { tagName?: string; isContentEditable?: boolean }
  if (element.isContentEditable === true) return true
  return FIELDS.includes(element.tagName ?? '')
}
