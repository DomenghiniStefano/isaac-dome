<script setup lang="ts">
import { InfoIcon, SaveIcon, SearchXIcon } from '@lucide/vue'
import { computed } from 'vue'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button, ButtonVariant } from '@/components/ui/button'
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyMedia,
  EmptyTitle,
} from '@/components/ui/empty'
import { Skeleton } from '@/components/ui/skeleton'
import { useMessages } from '@/i18n'
import { gateState } from '@/lib/profile/gateView'
import { missingReasonLabel } from '@/lib/profile/profileLabels'
import { useProfileStore } from '@/stores/profile'
import ProfileScreen from './ProfileScreen.vue'

const profile = useProfileStore()
const { t } = useMessages()
const emit = defineEmits<{ openProfile: [] }>()

// Every decision is in `gateView`, where it can be tested: this file only draws the answer.
const state = computed(() => gateState(profile.setup, profile.status))
const blockedReason = computed(() =>
  state.value.kind === 'blocked' ? state.value.reason : null,
)
</script>

<template>
  <slot v-if="state.kind === 'content'" />
  <div v-else-if="state.kind === 'waiting'" class="flex flex-col gap-4">
    <Skeleton class="h-10 w-full" />
    <Skeleton class="h-50 w-full" />
  </div>
  <!-- The chain broke before the saves, so there is no selection to show: naming the link
       that broke is the whole answer, and the sentence per reason already exists. The
       button only travels — choosing a folder by hand is B14, and lands where it leads. -->
  <Empty v-else-if="blockedReason">
    <EmptyMedia><SearchXIcon /></EmptyMedia>
    <EmptyTitle>{{ t('gate.blocked.title') }}</EmptyTitle>
    <EmptyDescription>{{
      t(missingReasonLabel[blockedReason])
    }}</EmptyDescription>
    <EmptyContent>
      <Button :variant="ButtonVariant.Outline" @click="emit('openProfile')">
        <SaveIcon />{{ t('gate.blocked.settings') }}
      </Button>
    </EmptyContent>
  </Empty>
  <!-- DESIGN-BRIEF.md §4.3: Progress, while there is a choice to make, *is* the profile
       selection. The line above it says why it shows this — and only here, because a read
       that failed is not a choice you have yet to make: there the error card speaks. -->
  <div v-else class="flex flex-col gap-4">
    <Alert v-if="state.kind === 'selection'">
      <InfoIcon />
      <AlertDescription>{{ t('gate.needsProfile') }}</AlertDescription>
    </Alert>
    <ProfileScreen />
  </div>
</template>
