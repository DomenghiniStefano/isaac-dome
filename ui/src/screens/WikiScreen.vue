<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useQueueOffer } from '@/composables/useQueueOffer'
import { useMessages } from '@/i18n'
import { singleQuery } from '@/lib/search/queryParam'
import { WikiCategory } from '@/router/routeTable'
import { LoadStatus } from '@/stores/loadStatus'
import { useWikiStore } from '@/stores/wiki'
import ProfileError from '@/components/data-state/ProfileError.vue'
import WikiCategoryList from './wiki/WikiCategoryList.vue'
import WikiLanding from './wiki/WikiLanding.vue'
import WikiPage from './wiki/WikiPage.vue'

const route = useRoute()
const wiki = useWikiStore()
const { queue } = useQueueOffer()
const { t } = useMessages()

// The index once per window: the store refuses a second load while one is ready or running.
void wiki.loadIndex()

// The save's state per page (design decision 6), read again on mount and whenever the chosen
// save changes: it depends on the save, unlike the index above, which does not. The queue is read
// with it, for both bodies that offer to change it: a page's own "+" and the list's Actions.
useOnActiveProfile(async () => {
  await Promise.all([wiki.loadProgress(), queue.load()])
})

// The location's query decides the view (spec 3.5, Decision 3): a page, a category's list,
// or the landing. A query value the router hands as an array or null is no value.
const page = computed(() => singleQuery(route.query.page))
const category = computed((): WikiCategory | null => {
  const wanted = singleQuery(route.query.category)
  return Object.values(WikiCategory).find((c) => c === wanted) ?? null
})
// A dataset that didn't load has no lists and no pages: every query shows the landing,
// which says so, rather than an empty list that reads as a category with no pages.
const missing = computed(() => wiki.index?.info.kind === 'missing')
</script>

<template>
  <ProfileError
    v-if="wiki.status === LoadStatus.Failed"
    :error="wiki.error"
    :title="t('wiki.states.failedTitle')"
    class="mx-5.5 mt-5"
    @retry="wiki.loadIndex()"
  />
  <WikiLanding v-else-if="missing" />
  <WikiPage v-else-if="page !== null" :page-key="page" :category="category" />
  <WikiCategoryList v-else-if="category !== null" :category="category" />
  <WikiLanding v-else />
</template>
