<script setup lang="ts">
import { InfoIcon } from '@lucide/vue'
import { computed } from 'vue'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Skeleton } from '@/components/ui/skeleton'
import { useMessages } from '@/i18n'
import { LoadStatus, useProfileStore } from '@/stores/profile'
import ProfileScreen from './ProfileScreen.vue'

const profile = useProfileStore()
const { t } = useMessages()

// Before the first answer nothing is known about the profile: saying "choose one" then would
// be false, so the gate waits. A failed read is known, and goes to the profile screen.
const waiting = computed(
  () => profile.setup === null && profile.status !== LoadStatus.Failed,
)
</script>

<template>
  <!-- DESIGN-BRIEF.md §4: Progress, until a choice is made, is the profile selection. The
       tab keeps its own name; the line above says why it shows this. -->
  <slot v-if="profile.isActive" />
  <div v-else-if="waiting" class="flex flex-col gap-4">
    <Skeleton class="h-10 w-full" />
    <Skeleton class="h-50 w-full" />
  </div>
  <div v-else class="flex flex-col gap-4">
    <Alert>
      <InfoIcon />
      <AlertDescription>{{ t('gate.needsProfile') }}</AlertDescription>
    </Alert>
    <ProfileScreen />
  </div>
</template>
