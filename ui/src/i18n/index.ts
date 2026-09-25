import { createI18n, useI18n } from 'vue-i18n'
import { Locale, resolveLocale } from './locale'
import type { Translate } from './message'
import { en } from './messages/en'
import { it, type MessageSchema } from './messages/it'

export const i18n = createI18n<[MessageSchema], Locale, false>({
  legacy: false,
  locale: resolveLocale(navigator.languages),
  fallbackLocale: Locale.En,
  messages: { [Locale.It]: it, [Locale.En]: en },
})

// vue-i18n's own t() accepts any string, so a misspelled key would compile and render the
// key itself. Components call this instead: the key is narrowed to the paths that exist.
// Global scope, so no component needs a local i18n instance.
// A message with a value in it (`{name}`) takes the values as a second argument: the word
// order around the value is the translation's business, not the caller's.
export const useMessages = (): { t: Translate } => {
  const { t } = useI18n({ useScope: 'global' })
  return {
    t: (key, params) => (params ? t(key, params) : t(key)),
  }
}
