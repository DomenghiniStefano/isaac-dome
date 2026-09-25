import { describe, expect, it } from 'vitest'
import { RunFacet } from './runFacets'
import { runFacetValueLabel } from './runLabels'

// A key is its own translation here: what matters is which key, or that there was none.
const t = (key: string): string => {
  if (typeof key !== 'string')
    throw new Error(`no key for this value: ${String(key)}`)
  return `«${key}»`
}

describe('runFacetValueLabel', () => {
  it('words the values it knows', () => {
    expect(runFacetValueLabel(t, RunFacet.Outcome, 'won')).toBe(
      '«runs.outcome.won»',
    )
    expect(runFacetValueLabel(t, RunFacet.Source, 'live')).toBe(
      '«runs.source.live»',
    )
    expect(runFacetValueLabel(t, RunFacet.Source, 'session')).toBe(
      '«runs.source.session»',
    )
  })

  // Card #80, P10: the comment above the function promised this, and the code looked the
  // value up in a table and translated `undefined`.
  it('shows a value outside its set as it came', () => {
    expect(runFacetValueLabel(t, RunFacet.Outcome, 'drowned')).toBe('drowned')
    expect(runFacetValueLabel(t, RunFacet.Source, 'replay')).toBe('replay')
    expect(runFacetValueLabel(t, RunFacet.Online, 'splitscreen')).toBe(
      'splitscreen',
    )
  })
})
