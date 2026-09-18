import { uniq } from 'lodash-es'

// The Tailwind namespaces whose custom names `cn()` has to know. A namespace is the
// prefix of a theme variable: `--text-body` belongs to `text`.
export const ThemeNamespace = {
  Text: 'text',
  Font: 'font',
  Tracking: 'tracking',
  Spacing: 'spacing',
  Radius: 'radius',
  Ease: 'ease',
  Animate: 'animate',
  TransitionDuration: 'transition-duration',
  Opacity: 'opacity',
} as const
export type ThemeNamespace =
  (typeof ThemeNamespace)[keyof typeof ThemeNamespace]

// `--<namespace>-<name>:` at a declaration. The name stops before a `--` sub-property, so
// `--text-body--line-height` still names `body`; the `--text-*: initial` reset has no name
// and doesn't match; `var(--text-body)` isn't followed by a colon and doesn't either.
const declaration = (namespace: ThemeNamespace) =>
  new RegExp(
    `--${namespace}-([a-z0-9]+(?:-[a-z0-9]+)*)(?:--[a-z0-9-]+)?\\s*:`,
    'g',
  )

export const themeKeys = (css: string, namespace: ThemeNamespace): string[] =>
  uniq([...css.matchAll(declaration(namespace))].map(([, name]) => name ?? ''))
