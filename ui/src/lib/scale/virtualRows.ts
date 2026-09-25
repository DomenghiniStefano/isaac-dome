// A virtualizer's items — an index and an offset — paired with the row living at that index,
// the offset carried as a ready CSS variable: what Unlock, the Collection, Search and the wiki's
// category list draw. A filter can shrink the row array after the virtualizer measured the old
// count, so an item whose index has nothing left at it is dropped, not drawn as `undefined`
// (`docs/BACKLOG.md` B43).
export interface VirtualItemLike {
  index: number
  start: number
}

export interface VisibleRow<T> {
  index: number
  start: number
  style: { '--row-start': string }
  row: T
}

export const visibleRows = <T>(
  items: VirtualItemLike[],
  rows: T[],
): VisibleRow<T>[] =>
  items.flatMap((item) => {
    const row = rows[item.index]
    return row === undefined
      ? []
      : [
          {
            index: item.index,
            start: item.start,
            style: { '--row-start': `${item.start}px` },
            row,
          },
        ]
  })

// The virtualizer's total size is a number of device pixels; the utility that reads it wants a
// CSS length.
export const totalHeightPx = (total: number): string => `${total}px`
