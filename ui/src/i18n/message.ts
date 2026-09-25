import type { MessageKey } from './messageKey'
import type { MessageSchema } from './messages/it'

// A key the Italian schema has, which is every key: `en` is typed against it. Every module that
// names a string names it with this.
export type Message = MessageKey<MessageSchema>

// The `t` a pure function receives from `useMessages()`: it never calls vue-i18n itself, so the
// function can be tested with a stand-in that echoes the key.
export type Translate = (
  key: Message,
  params?: Record<string, unknown>,
) => string
