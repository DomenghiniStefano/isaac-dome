import type { MarkArtView } from '../types'

// Development only: images from the design pack under design-export/, reached through the
// fixtures and the Kit page, which a production build never imports. A clone without the
// pack gets empty globs and null URLs, and the screens fall back as they do without the game.
const widget = import.meta.glob<string>(
  '../../../../../design-export/isaacdome-design-pack/images/sheets/completion_widget/*_0[02].png',
  { eager: true, query: '?url', import: 'default' },
)
const lobby = import.meta.glob<string>(
  '../../../../../design-export/isaacdome-design-pack/images/sheets/onlinelobby/background_completion_delirium_0[02].png',
  { eager: true, query: '?url', import: 'default' },
)
const heads = import.meta.glob<string>(
  '../../../../../design-export/isaacdome-design-pack/images/sheets/coop_menu/main_*.png',
  { eager: true, query: '?url', import: 'default' },
)
const achievements = import.meta.glob<string>(
  '../../../../../design-export/isaacdome-design-pack/images/achievement/*.png',
  { eager: true, query: '?url', import: 'default' },
)
const items = import.meta.glob<string>(
  '../../../../../design-export/isaacdome-design-pack/images/items/*.png',
  { eager: true, query: '?url', import: 'default' },
)

const find = (files: Record<string, string>, suffix: string): string | null =>
  Object.entries(files).find(([path]) => path.endsWith(suffix))?.[1] ?? null

// The pack names a file after its id and then its name (`0001_you_unlocked_magdalene.png`,
// `familiar_0073_cube_of_meat.png`): the id is the prefix, the slug is only for people.
const findByPrefix = (
  files: Record<string, string>,
  prefix: string,
): string | null =>
  Object.entries(files).find(([path]) =>
    (path.split('/').pop() ?? '').startsWith(prefix),
  )?.[1] ?? null

const column = (files: Record<string, string>, stem: string): MarkArtView => ({
  normalUrl: find(files, `/${stem}_00.png`),
  hardUrl: find(files, `/${stem}_02.png`),
})

// One entry per column of ipc::BOSSES, in its order: the layer each symbol was cut from, as
// crates/ipc/src/mark_art.rs maps it. Delirium's is on the online lobby's sheet.
export const packMarkArt: MarkArtView[] = [
  column(widget, 'heart'), // Mom's Heart
  column(widget, 'polaroid'), // Isaac
  column(widget, 'upsidedowncross'), // Satan
  column(widget, 'star'), // Boss Rush
  column(widget, 'negative'), // Blue Baby
  column(widget, 'cross'), // The Lamb
  column(widget, 'megasatan'), // Mega Satan
  column(widget, 'greed'), // Greed
  column(widget, 'hush'), // Hush
  column(lobby, 'background_completion_delirium'), // Delirium
  column(widget, 'knife'), // Mother
  column(widget, 'dadsnote'), // The Beast
]

const range = (from: number, to: number): number[] =>
  Array.from({ length: to - from + 1 }, (_, i) => from + i)

// The co-op menu frame of each matrix row, as catalog's head map gives it: character ids 0
// to 19 draw frame id + 1, ids 21 to 37 frame id. The rows skip the forms the matrix doesn't
// have (Lazarus Risen, Black Judas, The Soul), and the "Jacob & Esau" row is Jacob.
const headFrames = [...range(1, 11), 14, 15, 16, 17, 19, 20, ...range(21, 37)]

export const packHeadUrl = (row: number): string | null => {
  const frame = headFrames[row]
  return frame === undefined
    ? null
    : find(heads, `/main_${String(frame).padStart(2, '0')}.png`)
}

// The two icon links the unlock payload carries (crates/ipc/src/icon.rs `to_path`), rewritten
// to the pack's file; any other link has no file here.
const AchievementLink = /^isaac:\/\/achievement\/(\d+)$/
const ItemLink = /^isaac:\/\/item\/(passive|active|familiar|trinket)\/(\d+)$/

const fourDigits = (id: string): string => id.padStart(4, '0')

export const packIconUrl = (url: string | null): string | null => {
  if (url === null) return null
  const achievement = AchievementLink.exec(url)
  if (achievement?.[1])
    return findByPrefix(achievements, `${fourDigits(achievement[1])}_`)
  const item = ItemLink.exec(url)
  if (item?.[1] && item[2])
    return findByPrefix(items, `${item[1]}_${fourDigits(item[2])}_`)
  return null
}
