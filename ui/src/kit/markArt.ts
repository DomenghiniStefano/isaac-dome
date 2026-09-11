import { markArtOf } from '@/components/marks/markVisual'
import { packMarkArt } from '@/lib/ipc/fixtures/art'

// Development only: the design pack's mark symbols, globbed once in the fixtures. The Kit
// page is behind import.meta.env.DEV, so the production build never sees them; a clone
// without the pack gets null art and the fallback outfit.
// The five columns the Kit shows: Mom's Heart, Isaac, Boss Rush, Delirium, Mother.
export const kitMarkArt = {
  heart: markArtOf(packMarkArt[0]),
  polaroid: markArtOf(packMarkArt[1]),
  star: markArtOf(packMarkArt[3]),
  delirium: markArtOf(packMarkArt[9]),
  knife: markArtOf(packMarkArt[10]),
}
