import type { MarkArt } from '@/components/marks/markVisual'

// Development only: the mark symbols from the design pack under design-export/. The Kit
// page is behind import.meta.env.DEV, so the production build never sees these files; a
// clone without the pack gets an empty glob, null art and the fallback outfit.
const widget = import.meta.glob<string>(
  '../../../design-export/isaacdome-design-pack/images/sheets/completion_widget/*_0[02].png',
  { eager: true, query: '?url', import: 'default' },
)
const lobby = import.meta.glob<string>(
  '../../../design-export/isaacdome-design-pack/images/sheets/onlinelobby/background_completion_delirium_0[02].png',
  { eager: true, query: '?url', import: 'default' },
)

const file = (
  files: Record<string, string>,
  stem: string,
  tier: string,
): string | undefined =>
  Object.entries(files).find(([path]) =>
    path.endsWith(`/${stem}_${tier}.png`),
  )?.[1]

const art = (files: Record<string, string>, stem: string): MarkArt | null => {
  const normal = file(files, stem, '00')
  const hard = file(files, stem, '02')
  return normal && hard ? { normal, hard } : null
}

// The five columns the Kit shows: Mom's Heart, Isaac, Boss Rush, Delirium, Mother.
export const kitMarkArt = {
  heart: art(widget, 'heart'),
  polaroid: art(widget, 'polaroid'),
  star: art(widget, 'star'),
  delirium: art(lobby, 'background_completion_delirium'),
  knife: art(widget, 'knife'),
}
