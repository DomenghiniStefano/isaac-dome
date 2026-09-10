import { describe, expect, it } from 'vitest'
import { Locale, resolveLocale } from './locale'

describe('resolveLocale', () => {
  it('picks Italian from a regional tag', () => {
    expect(resolveLocale(['it-IT', 'en'])).toBe(Locale.It)
  })

  it('picks the first language the app has', () => {
    expect(resolveLocale(['de-DE', 'en-GB'])).toBe(Locale.En)
  })

  it('ignores case', () => {
    expect(resolveLocale(['IT'])).toBe(Locale.It)
  })

  it('falls back to English when no language matches', () => {
    expect(resolveLocale(['de'])).toBe(Locale.En)
  })

  it('falls back to English with no languages at all', () => {
    expect(resolveLocale([])).toBe(Locale.En)
  })
})
