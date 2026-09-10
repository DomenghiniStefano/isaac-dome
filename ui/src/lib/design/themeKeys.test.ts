import { describe, expect, it } from 'vitest'
import { ThemeNamespace, themeKeys } from './themeKeys'

describe('themeKeys', () => {
  it('returns every name declared in the namespace, once', () => {
    const css = [
      '@theme {',
      '  --text-body: 14px;',
      '  --text-body--line-height: 1.5;',
      '  --text-caption: 12px;',
      '}',
    ].join('\n')
    expect(themeKeys(css, ThemeNamespace.Text)).toEqual(['body', 'caption'])
  })

  it('skips the namespace reset', () => {
    const css = '--radius-*: initial;\n--radius-input: 4px;'
    expect(themeKeys(css, ThemeNamespace.Radius)).toEqual(['input'])
  })

  it('keeps a multi-word name whole', () => {
    const css = '--spacing-row-compact: 24px;'
    expect(themeKeys(css, ThemeNamespace.Spacing)).toEqual(['row-compact'])
  })

  it('reads a hyphenated namespace', () => {
    const css = '--transition-duration-tap: 80ms;'
    expect(themeKeys(css, ThemeNamespace.TransitionDuration)).toEqual(['tap'])
  })

  it('ignores references, other namespaces and look-alike variables', () => {
    const css = [
      '--color-text-muted: #ffffff;',
      '--default-font-family: var(--font-pixel);',
      '--default-transition-duration: 0ms;',
      '--animate-tap-in: rise-in var(--transition-duration-tap) var(--ease-tap);',
      'font-size: var(--text-body);',
    ].join('\n')
    expect(themeKeys(css, ThemeNamespace.Text)).toEqual([])
    expect(themeKeys(css, ThemeNamespace.Font)).toEqual([])
    expect(themeKeys(css, ThemeNamespace.TransitionDuration)).toEqual([])
    expect(themeKeys(css, ThemeNamespace.Ease)).toEqual([])
  })
})
