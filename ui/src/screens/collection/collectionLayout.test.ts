import { describe, expect, it } from 'vitest'
import spacing from '@/assets/theme/spacing.css?raw'
import { collectionRowHeight } from './collectionLayout'

// The virtualizer positions rows with a number; the rows are drawn with a token. If the two ever
// disagree, rows overlap or leave gaps and nothing else fails.
describe('collectionRowHeight', () => {
  it('is the height of the row-wide token the rows are drawn at', () => {
    expect(spacing).toMatch(
      new RegExp(`--spacing-row-wide:\\s*${collectionRowHeight}px;`),
    )
  })
})
