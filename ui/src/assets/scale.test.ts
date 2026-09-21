import { describe, expect, it } from 'vitest'
import containers from './theme/containers.css?raw'
import radius from './theme/radius.css?raw'
import spacing from './theme/spacing.css?raw'
import typography from './theme/typography.css?raw'

// The interface scales because the root's font size is the scale and every size token is in
// rem (`docs/superpowers/specs/2026-09-12-screens-scale-design.md`, Decision 4). A px token
// that slips back in doesn't fail anything: it just stays its own size while everything else
// moves, which is the defect this test exists to catch.
const declarations = (css: string, prefix: string): [string, string][] =>
  [
    ...css.matchAll(
      new RegExp(String.raw`(--${prefix}-[a-z0-9-]+)\s*:\s*([^;]+);`, 'g'),
    ),
  ].map(([, name, value]) => [name ?? '', (value ?? '').trim()])

// The px values that stay px, each with the reason it does — the same shape as the
// scanner's exemptions: an exception with no reason is an untracked one.
const keepsPx: Record<string, string> = {
  '--spacing-scrollbar': "the OS's furniture in the app's clothes: one size",
  '--radius-cell': 'a radius of 2.5px is a smudge, and the skin is flat',
  '--radius-input': 'the same',
  '--container-tab-narrow':
    'a container query is compared against a used width, and the number is a fact about the drawn tab',
  '--container-compact':
    'the same: a threshold is compared against a used width, and this one is measured at scale 100 (spec 3.13a §5)',
  '--container-regular': 'the same',
  '--container-wide': 'the same',
  '--container-sidebar-room':
    "the same, and measured on the sidebar rather than on the page: where it stops being worth its width is the sidebar's own fact",
}

describe('the size tokens', () => {
  it('are in rem, so the root font size moves them', () => {
    for (const [name, value] of [
      ...declarations(spacing, 'spacing'),
      ...declarations(typography, 'text'),
    ]) {
      if (name in keepsPx) continue
      // The sprite tokens are whole multiples of a pixel size, which is the other way of
      // not being a fixed size (Decision 3).
      if (value.includes('var(--sprite-multiple)')) continue
      if (value.includes('var(--spacing-mark-symbol)')) continue
      expect(value, `${name} is ${value}`).not.toMatch(/\d(px)\b/)
    }
  })

  it('name the px they keep, and say why on the spot', () => {
    const source = [spacing, radius, containers].join('\n')
    for (const [name, reason] of Object.entries(keepsPx)) {
      expect(source, `${name} is declared`).toContain(name)
      expect(reason.length, `${name} has a reason`).toBeGreaterThan(0)
    }
    // Every px left in the three files belongs to one of them, to a comment, or to a
    // sprite's whole multiple.
    const lines = source
      .split('\n')
      .filter((line) => /^\s*--[a-z0-9-]+\s*:[^;]*\d+px/.test(line))
      .filter((line) => !line.includes('var(--sprite-multiple)'))
    for (const line of lines) {
      const name = /(--[a-z0-9-]+)\s*:/.exec(line)?.[1] ?? ''
      expect(Object.keys(keepsPx), `${name} is a declared exception`).toContain(
        name,
      )
    }
  })

  it('draw pixel art at a whole multiple of its own size', () => {
    const byName = new Map(declarations(spacing, 'spacing'))
    expect(byName.get('--spacing-sprite')).toBe(
      'calc(32px * var(--sprite-multiple))',
    )
    expect(byName.get('--spacing-mark-symbol')).toBe(
      'calc(16px * var(--sprite-multiple))',
    )
    // The cell holds the symbol, so it is measured from it: at 125 the multiple rounds up,
    // and a cell in rem would be smaller than what sits inside it.
    expect(byName.get('--spacing-mark-cell')).toContain(
      'var(--spacing-mark-symbol)',
    )
  })
})
