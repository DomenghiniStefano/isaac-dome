import { describe, expect, it } from 'vitest'
import base from './base.css?raw'
import typography from './theme/typography.css?raw'

// The root's font size is the interface's scale (cycle 3.5c): every token is in rem, so the
// whole app follows it. What this file used to forbid — a font size on `html` — is now what
// it requires; the invariant is the same one it was written for, restated: **at factor 1 the
// root is 16px**, so a spacing step is 4px and `--text-body` is 14px, which is what the
// design file is drawn at.
function ruleBody(css: string, selector: string): string | undefined {
  return new RegExp(String.raw`(?:^|[\s}])${selector}\s*\{([^}]*)\}`).exec(
    css,
  )?.[1]
}

describe('base.css', () => {
  it('is read as source', () => {
    expect(base).toContain('@layer base')
  })

  it('makes the root font size the scale, from a 16px base', () => {
    const html = ruleBody(base, 'html')
    expect(html).toBeDefined()
    expect(html).toMatch(/font-size:\s*calc\(16px \* var\(--app-scale, 1\)\);/)
  })

  it('sets the document text size on the body instead', () => {
    expect(ruleBody(base, 'body')).toMatch(/font-size:\s*var\(--text-body\);/)
  })

  it('is drawn at 14px body text when the factor is 1', () => {
    // 0.875rem x 16px = 14px: the number the kit is measured in.
    expect(typography).toMatch(/--text-body:\s*0\.875rem;/)
  })
})
