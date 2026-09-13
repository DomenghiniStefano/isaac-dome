import type { MessagePart } from '@/lib/ipc/errorText'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'

export type Message = MessageKey<MessageSchema>

// How loudly a diagnostic is drawn. A `note` is a line under the rows — it coexists with
// real content and is not a failure; `info` and `warning` are alerts that stand in for the
// content. No fields: an `as const` object, like every other value on the wire.
export const Severity = {
  Note: 'note',
  Info: 'info',
  Warning: 'warning',
} as const
export type Severity = (typeof Severity)[keyof typeof Severity]

// The one alert that asks the user for something. Named rather than a boolean, so a second
// action has somewhere to go and the screen's slot can tell them apart.
export const DiagnosticAction = { ImportGoals: 'importGoals' } as const
export type DiagnosticAction =
  (typeof DiagnosticAction)[keyof typeof DiagnosticAction]

// One row of a screen's table. A `note` has no title; everything else has both.
export interface DiagnosticRow {
  severity: Severity
  title?: Message
  body: Message
  action?: DiagnosticAction
}

// One diagnostic resolved to keys. `title` and `body` are arrays because a sentence can come
// from more than one key — the store reason is "what failed" plus "why" — and joining them
// in Rust is exactly what N2 removed.
export interface DiagnosticEntry {
  key: string
  severity: Severity
  title: MessagePart[]
  body: MessagePart[]
  action?: DiagnosticAction
}

// The diagnostic's own scalar fields are the translation's values: { kind: 'slotsBeyondCatalog',
// count: 3 } hands {count} to the string. The tag is not a value, and neither is a field a
// sentence could not place — an object or a list has no rendering a translator chose, so
// `reason` and `wanted` are dropped here rather than handed over unused.
const valuesOf = (d: object): Record<string, unknown> =>
  Object.fromEntries(
    Object.entries(d).filter(
      ([k, v]) =>
        k !== 'kind' && (typeof v === 'number' || typeof v === 'string'),
    ),
  )

// Builds the entries for one screen from its table. A kind the table maps to `null` is one
// the screen says elsewhere — under the rows themselves — and drawing it here would say the
// same thing twice.
export const entriesFrom = <D extends { kind: string }>(
  diagnostics: D[],
  table: Record<D['kind'], DiagnosticRow | null>,
): DiagnosticEntry[] =>
  diagnostics.flatMap((d) => {
    const row = table[d.kind as D['kind']]
    if (!row) return []
    const params = valuesOf(d)
    return [
      {
        key: d.kind,
        severity: row.severity,
        title: row.title ? [{ key: row.title, params }] : [],
        body: [{ key: row.body, params }],
        ...(row.action ? { action: row.action } : {}),
      },
    ]
  })
