<script setup lang="ts">
import { RefreshCwIcon, SaveIcon } from '@lucide/vue'
import { computed } from 'vue'
import { Badge } from '@/components/ui/badge'
import { Button, ButtonVariant } from '@/components/ui/button'
import {
  CardCollapsible,
  CardCollapsibleContent,
  CardCollapsibleTrigger,
} from '@/components/ui/card'
import { useFormat } from '@/composables/useFormat'
import { useMessages } from '@/i18n'
import type { SetupState } from '@/lib/ipc/types'
import type { SettledProfile } from '@/lib/profile/settledProfile'
import { candidateSourceLabel } from '@/lib/profile/profileLabels'
import { editionLong, editionShort, gameName } from '@/lib/profile/profileView'
import ProfileFact from '@/components/data-state/ProfileFact.vue'

const props = defineProps<{
  active: SettledProfile
  game: SetupState['game']
}>()
const emit = defineEmits<{ change: []; reload: [] }>()
const { t } = useMessages()
const fmt = useFormat()

const view = computed(() => {
  const profile = props.active.profile
  return {
    short: editionShort(profile.prefix),
    title: `${editionLong(profile.prefix)} · ${t('indicator.slot', { slot: profile.slot })}`,
    modified:
      fmt.modified(profile.modifiedUnix, new Date()) ??
      t('profile.active.unknownDate'),
    size: t('profile.active.bytes', { count: fmt.count(profile.sizeBytes) }),
    dlcs: props.game?.dlcs.map(gameName).join(' · ') ?? '',
    source: t(candidateSourceLabel[profile.source]),
  }
})
</script>

<template>
  <CardCollapsible :default-open="true">
    <CardCollapsibleTrigger>
      {{ t('profile.active.title') }}
      <template #summary>
        <Badge v-if="active.autoSelected">{{
          t('profile.active.autoSelected')
        }}</Badge>
      </template>
    </CardCollapsibleTrigger>
    <CardCollapsibleContent class="flex flex-wrap items-start gap-4.5">
      <div
        class="grid size-13 shrink-0 place-items-center border border-border bg-data text-label text-subtle-foreground"
      >
        {{ view.short }}
      </div>
      <div class="flex min-w-0 flex-1 flex-col gap-3">
        <span class="text-heading text-foreground">{{ view.title }}</span>
        <div class="flex flex-wrap gap-5">
          <ProfileFact
            :label="t('profile.active.modified')"
            :value="view.modified"
          />
          <ProfileFact :label="t('profile.active.size')" :value="view.size" />
          <ProfileFact
            v-if="view.dlcs"
            :label="t('profile.active.dlcs')"
            :value="view.dlcs"
          />
        </div>
        <span class="truncate text-label text-subtle-foreground"
          >{{ t('profile.active.foundIn') }} {{ view.source }}</span
        >
      </div>
      <div class="flex shrink-0 flex-col gap-2">
        <Button :variant="ButtonVariant.Secondary" @click="emit('change')">
          <SaveIcon />{{ t('profile.active.change') }}
        </Button>
        <Button :variant="ButtonVariant.Outline" @click="emit('reload')">
          <RefreshCwIcon />{{ t('profile.active.reload') }}
        </Button>
      </div>
    </CardCollapsibleContent>
  </CardCollapsible>
</template>
