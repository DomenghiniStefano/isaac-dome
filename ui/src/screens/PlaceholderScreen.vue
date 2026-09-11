<script setup lang="ts">
import { useRoute } from 'vue-router'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import { useMessages } from '@/i18n'
import ScreenHeader from './ScreenHeader.vue'

const route = useRoute()
const { t } = useMessages()
</script>

<template>
  <!-- A screen that isn't built yet says which sub-project brings it: honest, and removed
       screen by screen. -->
  <div class="flex max-w-250 flex-col gap-4">
    <ScreenHeader :icon="route.meta.icon" :title="t(route.meta.title)" />
    <EmptyCategory v-if="route.meta.arrives">{{
      t(route.meta.arrives)
    }}</EmptyCategory>
  </div>
</template>
