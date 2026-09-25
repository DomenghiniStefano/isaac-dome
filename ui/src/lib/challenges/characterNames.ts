import type { ChallengeRow } from '@/lib/ipc/types'

// The character facet stores the wiki's id, and the names are on the rows: the list is the only
// place that can label one — the same shape Unlock has for its own character facet (B28). A row
// without a character, or whose character has no page to name it, adds nothing, so its label
// falls back to the value rather than to a guess.
export const challengeCharacterNames = (
  rows: ChallengeRow[],
): Map<string, string> =>
  new Map(
    rows.flatMap((row) =>
      row.character !== null &&
      row.character.kind === 'character' &&
      row.characterName !== null
        ? [[String(row.character.id), row.characterName] as const]
        : [],
    ),
  )
