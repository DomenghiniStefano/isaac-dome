import type { Message } from '@/i18n/message'

// The pools a player meets most, worded. The word itself is `itempools.xml`'s pool name, which
// the log writes on every item line; the other twenty of the 31 the game declares are shown as
// written, which is what an unknown word gets too.
const worded: Record<string, Message> = {
  treasure: 'runs.pool.treasure',
  shop: 'runs.pool.shop',
  boss: 'runs.pool.boss',
  devil: 'runs.pool.devil',
  angel: 'runs.pool.angel',
  secret: 'runs.pool.secret',
  library: 'runs.pool.library',
  curse: 'runs.pool.curse',
  goldenChest: 'runs.pool.goldenChest',
  redChest: 'runs.pool.redChest',
  beggar: 'runs.pool.beggar',
}

export const poolLabel = (pool: string): Message | null =>
  Object.hasOwn(worded, pool) ? worded[pool] : null
