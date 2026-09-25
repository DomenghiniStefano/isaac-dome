import type { MessageKey } from './messageKey'
import type { MessageSchema } from './messages/it'

// A key the Italian schema has, which is every key: `en` is typed against it. Declared here once
// because every module that names a string names it this way, and nineteen of them had written
// the alias out for themselves.
export type Message = MessageKey<MessageSchema>

// The `t` a pure function receives from `useMessages()`: it never calls vue-i18n itself, so the
// function can be tested with a stand-in that echoes the key.
export type Translate = (
  key: Message,
  params?: Record<string, unknown>,
) => string
