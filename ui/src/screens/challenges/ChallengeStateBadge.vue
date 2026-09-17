<script setup lang="ts">
import { computed } from 'vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { useMessages } from '@/i18n'
import type { ChallengeStateView } from '@/lib/ipc/types'

// A challenge's state, in the app's own four tones. A blocked one says **how many** gates it is
// waiting for and the row spells out which: the reading of `unlocked_by` as "all of these" is
// not settled (spec 3.11 §4), so the claim has to be visible enough to be disbelieved.
const props = defineProps<{ state: ChallengeStateView }>()
const { t } = useMessages()

const variant = computed<BadgeVariant>(() => {
  switch (props.state.kind) {
    case 'done':
      return BadgeVariant.Done
    case 'available':
      return BadgeVariant.Now
    case 'blocked':
      return BadgeVariant.Blocked
    case 'unknown':
      return BadgeVariant.Unknown
  }
  return BadgeVariant.Unknown
})
</script>

<template>
  <Badge :variant="variant">
    <template v-if="state.kind === 'blocked'">{{
      t('challenges.blockedBy', { count: state.missing.length })
    }}</template>
    <template v-else-if="state.kind === 'done'">{{
      t('challenges.state.done')
    }}</template>
    <template v-else-if="state.kind === 'available'">{{
      t('challenges.state.available')
    }}</template>
    <template v-else>{{ t('challenges.state.unknown') }}</template>
  </Badge>
</template>
