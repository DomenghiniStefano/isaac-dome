import { i18n } from '@/i18n'
import type { Locale } from '@/i18n/locale'
import {
  formatCount,
  formatDate,
  formatModified,
  formatPercent,
  formatRelativeDay,
} from '@/lib/profile/profileView'

// The one reader of the interface's locale for formatting (card #81, V9). The formatters stay
// pure in `lib/`, taking the locale as an argument and tested there; a screen asks this for
// them bound to the language it is being drawn in, and never reads `i18n.global.locale` itself.
// Reading it inside a `computed` keeps the reactivity: a language switch redraws the numbers.
export const useFormat = () => {
  const locale = (): Locale => i18n.global.locale.value
  return {
    locale,
    count: (n: number) => formatCount(n, locale()),
    date: (date: Date) => formatDate(date, locale()),
    percent: (percent: number) => formatPercent(percent, locale()),
    modified: (unix: number | null, now: Date) =>
      formatModified(unix, now, locale()),
    relativeDay: (unix: number | null, now: Date) =>
      formatRelativeDay(unix, now, locale()),
  }
}
