import type { Ref } from 'vue'
import { createContext } from 'reka-ui'

export interface CommandFilterState {
  search: string
  filtered: {
    /** How many items match the search. */
    count: number
    /** Item id to 1 when it matches, 0 when it doesn't. */
    items: Map<string, number>
    /** Groups with at least one matching item. */
    groups: Set<string>
  }
}

export const [useCommand, provideCommandContext] = createContext<{
  allItems: Ref<Map<string, string>>
  allGroups: Ref<Map<string, Set<string>>>
  filterState: CommandFilterState
}>('Command')

export const [useCommandGroup, provideCommandGroupContext] = createContext<{
  id?: string
}>('CommandGroup')
