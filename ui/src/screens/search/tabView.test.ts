import { describe, expect, it } from 'vitest'
import { RowGroup } from '@/lib/search/rows'
import { searchView } from './tabView'

describe("Search's reading", () => {
  it('is nothing picked when there is nothing to read', () => {
    expect(searchView.empty()).toEqual({ picked: [], offset: null })
  })

  it('reads back the groups that were picked', () => {
    expect(searchView.read({ picked: [RowGroup.Wiki] })?.picked).toEqual([
      RowGroup.Wiki,
    ])
  })

  // A group that has since gone is dropped, not kept: a pick nobody can see filters every row
  // away and reads as a search that found nothing.
  it('drops a group it no longer has', () => {
    expect(
      searchView.read({ picked: [RowGroup.Wiki, 'seances'] })?.picked,
    ).toEqual([RowGroup.Wiki])
  })

  it('refuses a record that is not a reading', () => {
    expect(searchView.read({ picked: 'wiki' })).toBeNull()
  })
})
