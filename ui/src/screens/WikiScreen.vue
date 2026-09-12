<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useMessages } from '@/i18n'
import { singleQuery } from '@/lib/search/queryParam'
import { WikiCategory } from '@/router/routeTable'
import { LoadStatus } from '@/stores/profile'
import { useWikiStore } from '@/stores/wiki'
import ProfileError from './profile/ProfileError.vue'
import WikiCategoryList from './wiki/WikiCategoryList.vue'
import WikiLanding from './wiki/WikiLanding.vue'
import WikiPage from './wiki/WikiPage.vue'

const route = useRoute()
const wiki = useWikiStore()
const { t } = useMessages()

// The index once per window: the store refuses a second load while one is ready or running.
void wiki.loadIndex()

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
    @retry="wiki.loadIndex()"
  />
  <WikiLanding v-else-if="missing" />
  <WikiPage v-else-if="page !== null" :page-key="page" :category="category" />
  <WikiCategoryList v-else-if="category !== null" :category="category" />
  <WikiLanding v-else />
</template>
