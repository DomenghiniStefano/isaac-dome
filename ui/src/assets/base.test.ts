import { describe, expect, it } from 'vitest'
import base from './base.css?raw'

// Tailwind's spacing is calc(var(--spacing) * n) with --spacing at 0.25rem, and the kit
// draws on whole 4px steps. A font size on the root moves rem with it: at 14px every step
// is 3.5px. Measured on the Kit page on 2026-09-10 before this test existed: a 14px
// checkbox where the kit draws 16, and a 10.5px gap.
function ruleBody(css: string, selector: string): string | undefined {
  return new RegExp(String.raw`(?:^|[\s}])${selector}\s*\{([^}]*)\}`).exec(
    css,
  )?.[1]
}

describe('base.css', () => {
  it('is read as source', () => {
    expect(base).toContain('@layer base')
  })

  it('leaves the root font size to the browser, so a spacing step stays 4px', () => {
    const html = ruleBody(base, 'html')
    expect(html).toBeDefined()
    expect(html).not.toMatch(/font-size\s*:/)
  })

  it('sets the document text size on the body instead', () => {
    expect(ruleBody(base, 'body')).toMatch(/font-size:\s*var\(--text-body\);/)
  })
})
