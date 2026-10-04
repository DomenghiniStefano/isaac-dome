import type { EntityRef } from '@/lib/ipc/types'

// What a death line says, read for a person. The log writes two entities, `Killed by (9.0)
// spawned by (20.0)`: often the killer is a shot and the spawner the monster that fired it, so
// where there is room for one name it is the spawner's, and the page shows both.

export interface Death {
  killer: EntityRef
  spawner: EntityRef | null
}

/** The one entity a row names for a death: who spawned the killer, or the killer itself. */
export const whoKilled = (death: Death): EntityRef =>
  death.spawner ?? death.killer

/** An entity's name, or the log's own `id.variant` when the catalog does not know it. */
export const entityName = (entity: EntityRef): string =>
  entity.name ?? entity.raw
