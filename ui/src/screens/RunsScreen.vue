<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { runKeyOf } from '@/lib/runs/runLocation'
import { useRunsStore } from '@/stores/views'
import RunPage from './runs/RunPage.vue'
import RunsList from './runs/RunsList.vue'

// The Runs screen is the diary or one run's page, as the Wiki screen is a list or a page: which
// one is in the location (`?run=`), so a tab can hold a run and back returns to the list.
const route = useRoute()
const store = useRunsStore()

// The archive is not a view of the profile: it exists without one, and it is what the app read
// while nobody was watching. So it loads on mount and not on a profile becoming active — once,
// here, for both bodies.
void store.load()

const runKey = computed(() => runKeyOf(route.query))
</script>

<template>
  <RunPage v-if="runKey !== null" :run-key="runKey" />
  <RunsList v-else />
</template>
