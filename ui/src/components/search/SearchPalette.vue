<script setup lang="ts">
import { CornerDownLeftIcon } from '@lucide/vue'
import { computed, ref, useTemplateRef, watch } from 'vue'
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
import { keyAfterAnswer } from '@/lib/search/highlight'
import { queryToRecall } from '@/lib/search/recall'
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
// The row the keyboard is on, mirrored from the listbox (see the keyboard section below).
const highlighted = ref<string | null>(null)
watch(typed, (query) => ask(query))
// Closing keeps the search (`queryToRecall`) and forgets the row, however it closes — `Esc` or
// a chosen row, which is why the Command is told to `keep-search`: an item clears it on select
// otherwise, before the palette ever sees it close. The palette reopens on the last search,
// selected, so typing replaces it and Enter opens its first row again. It is asked
// again on opening because the answer may have changed since — a game installed meanwhile —
// and that answer is also what puts the highlight back on the first row.
watch(open, (isOpen) => {
  if (isOpen) {
    ask(typed.value)
    return
  }
  typed.value = queryToRecall(typed.value)
  highlighted.value = null
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
const openAt = (location: TabLocation, newTab: boolean) => {
  if (newTab) tabs.open(location)
  else tabs.navigate(location)
  open.value = false
}

const go = (row: SearchRow) => openAt(row.location, ctrl.value)

// The last row of the list, and the only one that is not in `rows`: it goes to the Search
// screen with the query, rather than to a result.
const AllResultsKey = 'all-results'
const allResultsLocation = (): TabLocation => ({
  name: RouteName.Search,
  query: { q: typed.value },
})
const allResults = () => openAt(allResultsLocation(), ctrl.value)

// **Who owns the keyboard in here** (B65): the listbox owns the highlight — it draws it and
// the arrows move it — the palette owns *what the highlight is*, and Enter is read off it.
// The plain key stays the listbox's, which clicks the row and arrives at `@select`; the
// modified one is the palette's, because `ListboxRoot.onKeydownEnter` returns on a modifier
// before it clicks and never hears it at all.
const onHighlight = (payload: { value: unknown } | undefined) => {
  highlighted.value = typeof payload?.value === 'string' ? payload.value : null
}

const palette = useTemplateRef('palette')

// An answer replaces every row. The listbox highlighted the first row of the *previous* one,
// 120 ms ago, and that row has just unmounted — a highlight on a detached row is invisible
// and does not open. So it is put back, after the new rows are in the DOM, which is what
// `flush: 'post'` is for.
watch(
  rows,
  (list) => {
    const key = keyAfterAnswer(list, highlighted.value)
    highlighted.value = key
    if (key !== null) palette.value?.highlightItem(key)
  },
  { flush: 'post' },
)

const locationOf = (key: string | null): TabLocation | undefined => {
  if (key === AllResultsKey) return allResultsLocation()
  return rows.value.find((row) => row.key === key)?.location
}

const onKeydown = (event: KeyboardEvent) => {
  if (event.key !== EventKey.Enter) return
  if (!event.ctrlKey && !event.metaKey) return
  const location = locationOf(highlighted.value)
  if (location === undefined) return
  event.preventDefault()
  openAt(location, true)
}
</script>

<template>
  <CommandDialog
    ref="palette"
    v-model:open="open"
    v-model:search="typed"
    :filter="false"
    keep-search
    :title="t('routes.search')"
    :description="t('search.intro')"
    @highlight="onHighlight"
  >
    <!-- The keydown sits on the input because that is where the focus is: the listbox's own
         Enter handler runs first and ignores the modified key, so the two never both fire. -->
    <CommandInput
      :placeholder="t('search.placeholder')"
      select-on-focus
      @keydown="onKeydown"
    />
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
        <CommandItem :value="AllResultsKey" @select="allResults()">
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
