<script setup lang="ts">
import { CornerDownLeftIcon } from '@lucide/vue'
import { computed, ref, watch } from 'vue'
import {
  CommandDialog,
  CommandEmpty,
  CommandFooter,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import { Kbd, KbdGroup } from '@/components/ui/kbd'
import { useGestureModifiers } from '@/composables/useGestureModifiers'
import { useSearch } from '@/composables/useSearch'
import { useShortcut } from '@/composables/useShortcut'
import { useMessages } from '@/i18n'
import { EventKey } from '@/lib/constants/eventKeys'
import { KeyName } from '@/lib/constants/keyNames'
import { SearchLimit } from '@/lib/ipc/search'
import { SearchDiagnostic } from '@/lib/ipc/types'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import {
  RowGroup,
  matchingScreens,
  rowGroupOrder,
  screenEntries,
  searchRows,
} from '@/lib/search/rows'
import type { SearchRow } from '@/lib/search/rows'
import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'
import SearchRowContent from './SearchRow.vue'

const open = defineModel<boolean>('open', { required: true })
const tabs = useTabsStore()
const { t } = useMessages()
const { view, ask } = useSearch(SearchLimit.Palette)

// `Ctrl+K` from any tab: the palette is mounted once, in App.vue, so the window always has it.
useShortcut((event) => {
  if (!event.ctrlKey || event.key.toLowerCase() !== EventKey.K) return false
  open.value = true
  return true
})

// What is typed lives here, not in the Command: the query is debounced before it reaches the
// backend, and the answer that comes back is the backend's order, shown as it is.
const typed = ref('')
watch(typed, (query) => ask(query))
watch(open, (isOpen) => {
  if (!isOpen) typed.value = ''
})

const catalog = computed(
  () => !(view.value?.diagnostics ?? []).includes(SearchDiagnostic.NoCatalog),
)

// Five rows per group in the palette; the whole answer is the Search screen's job.
const PaletteCap = 5

const rows = computed(() =>
  searchRows(
    view.value?.hits ?? [],
    matchingScreens(screenEntries(t), typed.value),
    { catalog: catalog.value, cap: PaletteCap },
  ),
)

const groups = computed(() =>
  rowGroupOrder
    .map((group) => ({
      group,
      rows: rows.value.filter((row) => row.group === group),
    }))
    .filter((g) => g.rows.length > 0),
)

const groupLabel: Record<RowGroup, MessageKey<MessageSchema>> = {
  [RowGroup.Screens]: 'search.groups.screens',
  [RowGroup.Wiki]: 'search.groups.wiki',
  [RowGroup.Unlock]: 'search.groups.unlock',
  [RowGroup.Collection]: 'search.groups.collection',
}

const total = computed(() => view.value?.total ?? 0)
const hasQuery = computed(() => typed.value.trim() !== '')

// Opening goes through the list's own `select`, not a click: Reka replays the click on the
// item, so a click handler fires twice and opens two tabs. `select` says *that* a row was
// chosen but not *how*, so the modifier of the gesture that produced it is read from the window.
const { ctrl } = useGestureModifiers()

// Enter navigates the active tab; Ctrl opens the row beside it, as a browser does.
const openAt = (location: TabLocation) => {
  if (ctrl.value) tabs.open(location)
  else tabs.navigate(location)
  open.value = false
}

const go = (row: SearchRow) => openAt(row.location)

const allResults = () =>
  openAt({ name: RouteName.Search, query: { q: typed.value } })
</script>

<template>
  <CommandDialog
    v-model:open="open"
    v-model:search="typed"
    :filter="false"
    :title="t('routes.search')"
    :description="t('search.intro')"
  >
    <CommandInput :placeholder="t('search.placeholder')" />
    <CommandList>
      <CommandEmpty>{{ t('search.empty') }}</CommandEmpty>
      <CommandGroup
        v-for="group in groups"
        :key="group.group"
        :heading="t(groupLabel[group.group])"
      >
        <CommandItem
          v-for="row in group.rows"
          :key="row.key"
          :value="row.key"
          @select="go(row)"
        >
          <SearchRowContent :row="row" />
        </CommandItem>
      </CommandGroup>
      <CommandGroup v-if="hasQuery && total > 0">
        <CommandItem value="all-results" @select="allResults()">
          {{ t('search.allResults', { count: total }) }}
        </CommandItem>
      </CommandGroup>
    </CommandList>
    <CommandFooter>
      <span class="flex items-center gap-1">
        <Kbd><CornerDownLeftIcon /></Kbd>{{ t('search.hint.open') }}
      </span>
      <span class="flex items-center gap-1">
        <KbdGroup>
          <Kbd>{{ KeyName.Ctrl }}</Kbd>
          <Kbd><CornerDownLeftIcon /></Kbd>
        </KbdGroup>
        {{ t('search.hint.newTab') }}
      </span>
    </CommandFooter>
  </CommandDialog>
</template>
