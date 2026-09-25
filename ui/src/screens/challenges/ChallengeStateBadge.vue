<script setup lang="ts">
import { computed } from 'vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import { challengeStateText } from '@/lib/challenges/challengeLabels'
import type { ChallengeStateView } from '@/lib/ipc/types'

// A challenge's state, in the app's own four tones. A blocked one says **how many** gates it is
// waiting for and the row spells out which: the reading of `unlocked_by` as "all of these" is
// not settled (spec 3.11 §4), so the claim has to be visible enough to be disbelieved.
const props = defineProps<{ state: ChallengeStateView }>()
const { t } = useMessages()

const variant = computed<BadgeVariant>(() => {
  const state = props.state
  switch (state.kind) {
    case 'done':
      return BadgeVariant.Done
    case 'available':
      return BadgeVariant.Now
    case 'blocked':
      return BadgeVariant.Blocked
    case 'unknown':
      return BadgeVariant.Unknown
    default:
      return assertNever(state)
  }
})

const text = computed(() =>
  props.state.kind === 'blocked'
    ? t('challenges.blockedBy', { count: props.state.missing.length })
    : t(challengeStateText[props.state.kind]),
)
</script>

<template>
  <Badge :variant="variant">{{ text }}</Badge>
</template>
