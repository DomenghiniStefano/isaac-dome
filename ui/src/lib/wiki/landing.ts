import type { CategorySample } from '@/lib/ipc/types'

// The landing hero's mosaic (design decision 8b): as many category samples as the band has
// room for, favouring one that actually draws a picture over one that would only fall back
// to its plain category icon — `iconUrl` is `null` either without the game, or for the three
// categories the game draws no picture for at all (transformations, stages, versions;
// `category_sample`'s own `None` arm). The owner asked for pictures and colour over icons
// ("le persone piacciono colori ed immagini"), so a mosaic that happened to lead with those
// three would show exactly the opposite of what was asked. `Array.prototype.sort` is stable
// (ES2019), so within "has a picture" and "doesn't" the categories keep the order `samples`
// already carries (`WIKI_PAGE_CATEGORIES`).
export const mosaicSamples = (
  samples: CategorySample[],
  max: number,
): CategorySample[] =>
  [...samples]
    .sort((a, b) => Number(b.iconUrl !== null) - Number(a.iconUrl !== null))
    .slice(0, max)
