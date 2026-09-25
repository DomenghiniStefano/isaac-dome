import type { Message } from '@/i18n/message'
import { camelCase } from 'lodash-es'
import { it } from '@/i18n/messages/it'

// A rule's sentence in the language of the app.
//
// The backend sends each rule with the sentence it was read from, and that sentence is the
// English wiki's: it is the citation, and it stays in `crates/floor/rules/placement.json` as
// written. What the screen shows is the translation, keyed by the rule's id — camel-cased,
// because a message path is dotted and an id is kebab. In English the translation *is* the
// quote, and `ruleText.test.ts` holds the two equal, so a quote edited in the rules file fails
// a test here and the Italian gets looked at again.
//
// **A rule nobody translated yet is `null`, never a missing key**: `t()` on a key that is not
// there prints the key itself, and a dotted path on screen is worse than an English sentence.
// The caller shows the quote instead.

const keyIn = (
  tree: Record<string, string>,
  prefix: string,
  id: string,
): Message | null => {
  const name = camelCase(id)
  return Object.hasOwn(tree, name) ? (`${prefix}.${name}` as Message) : null
}

/** The key of what the rule says, or `null` when it has no translation. */
export const ruleTextKey = (id: string): Message | null =>
  keyIn(it.floor.ruleText, 'floor.ruleText', id)

/** The key of why the grid cannot judge the rule, or `null` when it has none. */
export const ruleNoteKey = (id: string): Message | null =>
  keyIn(it.floor.ruleNote, 'floor.ruleNote', id)
