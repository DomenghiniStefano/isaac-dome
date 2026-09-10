export const Locale = { It: 'it', En: 'en' } as const
export type Locale = (typeof Locale)[keyof typeof Locale]

const locales: readonly string[] = Object.values(Locale)

const isLocale = (tag: string): tag is Locale => locales.includes(tag)

// `it-IT` becomes `it`: the app speaks one Italian and one English, not regional variants.
const primarySubtag = (language: string): string =>
  (language.split('-')[0] ?? '').toLowerCase()

// The first of the user's languages the app has, English otherwise. The preference in
// Settings arrives with that screen and will take precedence over this.
export const resolveLocale = (languages: readonly string[]): Locale =>
  languages.map(primarySubtag).find(isLocale) ?? Locale.En
