<script setup lang="ts">
import { RefreshCwIcon, SaveIcon, TriangleAlertIcon } from '@lucide/vue'
import { computed, ref } from 'vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import { Button, ButtonVariant } from '@/components/ui/button'
import { RadioGroup } from '@/components/ui/radio-group'
import { useMessages } from '@/i18n'
import type { WelcomeState } from '@/lib/profile/welcomeView'
import { LoadStatus } from '@/stores/loadStatus'
import { useProfileStore } from '@/stores/profile'
import NothingFound from './NothingFound.vue'
import SaveCard from './SaveCard.vue'

defineProps<{ state: WelcomeState }>()
const profile = useProfileStore()
const { t } = useMessages()

// Nothing is preselected while a choice is pending: the suggestion stays a suggestion
// (DESIGN-BRIEF.md §4.1). Changing an active profile starts on the one in use.
const active = computed(() =>
  profile.setup?.active.kind === 'active' ? profile.setup.active.profile : null,
)
const selected = ref<string | undefined>(active.value?.id)

const candidates = computed(() => profile.setup?.candidates ?? [])
const busy = computed(() => profile.status === LoadStatus.Loading)

const confirm = () => {
  if (selected.value) void profile.choose(selected.value)
}
</script>

<template>
  <!-- The first thing anyone meets, and the whole window while it is up: no sidebar and no
       tabs, because the app has not opened on anything yet (B17). It is a state above the
       router, never a route — here a route is a tab, and the session would restore it. -->
  <main
    class="flex min-h-0 flex-1 flex-col items-center overflow-auto px-5.5 py-10"
  >
    <div class="flex w-full max-w-250 flex-col gap-6">
      <template v-if="state.kind === 'choose'">
        <div class="flex flex-col gap-1">
          <h1 class="text-title">{{ t('welcome.title') }}</h1>
          <p class="text-body text-foreground-soft">
            {{ t('welcome.subtitle') }}
          </p>
        </div>
        <Alert v-if="state.savedGone" :variant="AlertVariant.Destructive">
          <TriangleAlertIcon />
          <AlertDescription>{{ t('welcome.savedGone') }}</AlertDescription>
        </Alert>
        <RadioGroup v-model="selected" class="grid grid-cols-2 gap-3">
          <SaveCard
            v-for="candidate in candidates"
            :key="candidate.id"
            :candidate="candidate"
            :selected="selected === candidate.id"
            @click="selected = candidate.id"
          />
        </RadioGroup>
        <div class="flex flex-wrap items-center justify-between gap-3">
          <span class="text-caption text-subtle-foreground">{{
            t('welcome.hint')
          }}</span>
          <div class="flex gap-2">
            <Button
              v-if="active"
              :variant="ButtonVariant.Outline"
              @click="profile.stopPicking()"
              >{{ t('welcome.cancel') }}</Button
            >
            <Button :disabled="!selected || busy" @click="confirm">
              <SaveIcon />{{ t('welcome.use') }}
            </Button>
          </div>
        </div>
      </template>

      <NothingFound
        v-else-if="state.kind === 'empty'"
        :reason="state.reason"
        :diagnostics="profile.setup?.diagnostics ?? []"
        @retry="profile.load()"
      />

      <Alert
        v-else-if="state.kind === 'failed'"
        :variant="AlertVariant.Destructive"
      >
        <TriangleAlertIcon />
        <AlertTitle>{{ t('welcome.failed.title') }}</AlertTitle>
        <AlertDescription>
          <Button :variant="ButtonVariant.Outline" @click="profile.load()">
            <RefreshCwIcon />{{ t('welcome.failed.retry') }}
          </Button>
        </AlertDescription>
      </Alert>
    </div>
  </main>
</template>
