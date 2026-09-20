import { TargetView } from '@/lib/ipc/types'

/**
 * The wiki page that explains each target, for the reader who wants the room and not just the
 * rule. The rules themselves all cite the Secret Room page — that is where they were read —
 * so the other two would have no reference at all without this.
 *
 * Checked to answer 200 on 2026-09-20. They are written out rather than built from the target's
 * name: a page that gets renamed has to be corrected here, and a name assembled at runtime
 * would keep pointing confidently at nothing.
 */
export const targetPage: Record<TargetView, string> = {
  [TargetView.Secret]: 'https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room',
  [TargetView.SuperSecret]:
    'https://bindingofisaacrebirth.wiki.gg/wiki/Super_Secret_Room',
  [TargetView.UltraSecret]:
    'https://bindingofisaacrebirth.wiki.gg/wiki/Ultra_Secret_Room',
}
