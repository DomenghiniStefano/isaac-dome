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

// `margin` is the space above the list inside its scroller: zero when the list scrolls on its
// own, the hero and the filters when the page does. The virtualizer counts offsets from the
// scroller's top, so a row is placed that much higher, back at its own place in the list.
export const visibleRows = <T>(
  items: VirtualItemLike[],
  rows: T[],
  margin = 0,
): VisibleRow<T>[] =>
  items.flatMap((item) => {
    const row = rows[item.index]
    const start = item.start - margin
    return row === undefined
      ? []
      : [
          {
            index: item.index,
            start,
            style: { '--row-start': `${start}px` },
            row,
          },
        ]
  })

// The virtualizer's total size is a number of device pixels; the utility that reads it wants a
// CSS length.
export const totalHeightPx = (total: number): string => `${total}px`
