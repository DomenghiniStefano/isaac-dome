<script setup lang="ts">
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import WikiInline from '@/components/wiki/WikiInline.vue'
import { useMessages } from '@/i18n'
import type { ChallengeRow, Target } from '@/lib/ipc/types'

// What a challenge asks you to do, and whether you do it blindfolded.
defineProps<{ row: ChallengeRow }>()
const emit = defineEmits<{ navigate: [target: Target, newTab: boolean] }>()
const { t } = useMessages()
</script>

<template>
  <span class="flex min-w-0 items-center gap-1.5">
    <!-- `WikiInline` is a fragment and takes no class of its own: this span sets the measure
         and the truncation, the way every other caller wraps it. -->
    <span v-if="row.goal" class="min-w-0 truncate text-caption">
      <WikiInline
        :inline="row.goal"
        @navigate="(target, newTab) => emit('navigate', target, newTab)"
      />
    </span>
    <EmptyValue v-else>{{ t('challenges.noCondition') }}</EmptyValue>
    <span
      v-if="row.blindfolded"
      class="shrink-0 text-label text-subtle-foreground"
      >{{ t('challenges.blindfolded') }}</span
    >
  </span>
</template>
