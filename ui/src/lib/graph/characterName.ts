import type { Translate } from '@/i18n/message'
import type { RequirementView, UnlockNode, UnlockTarget } from '@/lib/ipc/types'

// The two forms of a character, base and Tainted, carry the same name: `achievements.xml`
// writes `You unlocked "The Lost"` for both, and `players.xml` tells them apart by the `b`
// in the portrait's file name (`docs/BACKLOG.md` B28). So a label that dropped the flag
// would put two different characters on screen under one word.
export interface CharacterForm {
  name: string
  tainted: boolean
}

// The name stays the game's, in English like every game name; which form it is, is a
// message, because the word order differs between languages.
export const characterLabel = (t: Translate, form: CharacterForm): string =>
  form.tainted ? t('graph.taintedName', { name: form.name }) : form.name

// What a filter stores for a character: the id, stable and the same in every language —
// never the name, which the two forms share.
export const characterValue = (character: { id: number }): string =>
  String(character.id)

// A missing character by its facet value, and nothing for every other requirement.
const characterEntry = (
  requirement: RequirementView,
): [string, CharacterForm][] =>
  requirement.kind === 'character'
    ? [
        [
          characterValue(requirement),
          { name: requirement.name, tainted: requirement.tainted },
        ],
      ]
    : []

// Every character the nodes are missing, by that value: what turns a facet's picks back
// into names.
export const characterForms = (
  nodes: UnlockNode[],
): Map<string, CharacterForm> =>
  new Map(nodes.flatMap((node) => node.missing.flatMap(characterEntry)))

// What a target is called on screen. Every kind but one is the game's own name; a character
// needs its form as well, or the Tainted one reads as the base.
export const targetName = (t: Translate, target: UnlockTarget): string =>
  target.kind === 'character' ? characterLabel(t, target) : target.name
