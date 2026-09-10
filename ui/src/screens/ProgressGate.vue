<script setup lang="ts">
import { InfoIcon } from '@lucide/vue'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { useMessages } from '@/i18n'
import { useProfileStore } from '@/stores/profile'
import ProfileScreen from './ProfileScreen.vue'

const profile = useProfileStore()
const { t } = useMessages()
</script>

<template>
  <!-- DESIGN-BRIEF.md §4: Progress, until a choice is made, is the profile selection. The
       tab keeps its own name; the line above says why it shows this. -->
  <slot v-if="profile.isActive" />
  <div v-else class="flex flex-col gap-4">
    <Alert>
      <InfoIcon />
      <AlertDescription>{{ t('gate.needsProfile') }}</AlertDescription>
    </Alert>
    <ProfileScreen />
  </div>
</template>
