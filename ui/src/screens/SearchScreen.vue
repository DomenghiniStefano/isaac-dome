<script setup lang="ts">
import { SearchIcon } from '@lucide/vue'
import { useDebounceFn } from '@vueuse/core'
import { ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { Input } from '@/components/ui/input'
import { useMessages } from '@/i18n'
import { Timing } from '@/lib/constants/timing'
import { singleQuery } from '@/lib/search/queryParam'
import { RouteName } from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'
import ScreenHeader from './ScreenHeader.vue'

const route = useRoute()
const tabs = useTabsStore()
const { t } = useMessages()

// The query lives in the tab's location: typing navigates it, so the tab *is* the search and
// survives as one (spec 3.5, Decision 8). The field keeps what is typed while the debounce
// waits, so the caret never jumps.
const typed = ref(singleQuery(route.query.q) ?? '')

const navigate = useDebounceFn((q: string) => {
  tabs.navigate({ name: RouteName.Search, query: q === '' ? {} : { q } })
}, Timing.SearchDebounce)

watch(typed, (q) => void navigate(q))

// A location opened from elsewhere — the palette's "all results" row — brings its own query.
watch(
  () => route.query.q,
  (value) => {
    const q = singleQuery(value)
    if (q !== null && q !== typed.value) typed.value = q
  },
)
</script>

<template>
  <div class="flex max-w-250 flex-col gap-4">
    <ScreenHeader :icon="SearchIcon" :title="t('routes.search')">{{
      t('search.intro')
    }}</ScreenHeader>
    <Input v-model="typed" :placeholder="t('search.placeholder')" />
  </div>
</template>
