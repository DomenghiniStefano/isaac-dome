// Development only: the design pack's achievement drawings and item sprites, for the graph's
// fixtures. Kept apart from art.ts because these are some 1,500 files: only a command that
// draws the graph should pay for them, not every read of the profile.
const achievements = import.meta.glob<string>(
  '../../../../../design-export/isaacdome-design-pack/images/achievement/*.png',
  { eager: true, query: '?url', import: 'default' },
)
const items = import.meta.glob<string>(
  '../../../../../design-export/isaacdome-design-pack/images/items/*.png',
  { eager: true, query: '?url', import: 'default' },
)

// The pack names a file after its id and then its name (`0001_you_unlocked_magdalene.png`,
// `familiar_0073_cube_of_meat.png`): the id is the prefix, the slug is only for people.
const findByPrefix = (
  files: Record<string, string>,
  prefix: string,
): string | null =>
  Object.entries(files).find(([path]) =>
    (path.split('/').pop() ?? '').startsWith(prefix),
  )?.[1] ?? null

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
