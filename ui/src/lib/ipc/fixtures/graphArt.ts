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
const AchievementFile = /\/(\d{4})_[^/]*\.png$/
const ItemFile = /\/((?:passive|active|familiar|trinket)_\d{4})_[^/]*\.png$/

// Indexed once by that prefix. A scan of every path for every link cost 1.7 seconds per answer
// with art — on each graph and queue command — where a lookup costs nothing.
const byPrefix = (
  files: Record<string, string>,
  name: RegExp,
): Map<string, string> =>
  new Map(
    Object.entries(files).flatMap(([path, url]) => {
      const prefix = name.exec(path)?.[1]
      return prefix ? [[prefix, url] as const] : []
    }),
  )

const achievementFiles = byPrefix(achievements, AchievementFile)
const itemFiles = byPrefix(items, ItemFile)

// The two icon links the unlock payload carries (crates/ipc/src/icon.rs `to_path`), rewritten
// to the pack's file; any other link has no file here.
const AchievementLink = /^isaac:\/\/achievement\/(\d+)$/
const ItemLink = /^isaac:\/\/item\/(passive|active|familiar|trinket)\/(\d+)$/

const fourDigits = (id: string): string => id.padStart(4, '0')

export const packIconUrl = (url: string | null): string | null => {
  if (url === null) return null
  const achievement = AchievementLink.exec(url)
  if (achievement?.[1])
    return achievementFiles.get(fourDigits(achievement[1])) ?? null
  const item = ItemLink.exec(url)
  if (item?.[1] && item[2])
    return itemFiles.get(`${item[1]}_${fourDigits(item[2])}`) ?? null
  return null
}
