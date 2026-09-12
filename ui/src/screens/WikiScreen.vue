<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useMessages } from '@/i18n'
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
const single = (value: unknown): string | null =>
  typeof value === 'string' ? value : null
const page = computed(() => single(route.query.page))
const category = computed((): WikiCategory | null => {
  const wanted = single(route.query.category)
  return Object.values(WikiCategory).find((c) => c === wanted) ?? null
})
</script>

<template>
  <ProfileError
    v-if="wiki.status === LoadStatus.Failed"
    :error="wiki.error"
    :title="t('wiki.states.failedTitle')"
    @retry="wiki.loadIndex()"
  />
  <WikiPage v-else-if="page !== null" :page-key="page" :category="category" />
  <WikiCategoryList v-else-if="category !== null" :category="category" />
  <WikiLanding v-else />
</template>
