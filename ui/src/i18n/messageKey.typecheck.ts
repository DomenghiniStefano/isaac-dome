import type { MessageKey } from './messageKey'
import type { MessageSchema } from './messages/it'

// Checked by `pnpm typecheck`, not by a test runner: an existing key is accepted and a
// missing one is rejected. If MessageKey ever widened to `string`, the directive below
// would go unused and the typecheck would fail.
export const existingKey: MessageKey<MessageSchema> = 'ui.close'
// @ts-expect-error the key does not exist in the schema
export const missingKey: MessageKey<MessageSchema> = 'ui.nope'
