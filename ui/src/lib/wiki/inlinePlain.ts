import { assertNever } from '@/lib/assertNever'
import type { Inline } from '@/lib/ipc/types'

// An `Inline` tree flattened to the words a reader would read off it, with no styling and no
// links: for the few places a rich render can't go — inside a `<button>` that is already
// interactive (`WikiOutline`'s jump-to-section list), or a title used as a search key. Rust's
// `wiki::plain` reads the same tree the same way; this is its frontend twin, not a port of
// its code, because the two run on different trees (`Inline` here, `wiki::Inline` there).
export const inlinePlain = (inline: Inline[]): string =>
  inline
    .map((token): string => {
      switch (token.kind) {
        case 'text':
          return token.text
        case 'ref':
        case 'concept':
          return token.label
        case 'edition':
          return inlinePlain(token.inline)
        default:
          return assertNever(token)
      }
    })
    .join('')
