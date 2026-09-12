<script setup lang="ts">
import { SaveIcon } from '@lucide/vue'
import { computed, ref } from 'vue'
import { Skeleton } from '@/components/ui/skeleton'
import { useMessages } from '@/i18n'
import { LoadStatus, useProfileStore } from '@/stores/profile'
import ScreenHeader from './ScreenHeader.vue'
import ActiveProfileCard from './profile/ActiveProfileCard.vue'
import CandidatesCard from './profile/CandidatesCard.vue'
import ChainCard from './profile/ChainCard.vue'
import NoSavesCard from './profile/NoSavesCard.vue'
import ProfileError from './profile/ProfileError.vue'
import SectionsCard from './profile/SectionsCard.vue'

const profile = useProfileStore()
const { t } = useMessages()

// "Cambia profilo" shows the candidates over an active profile until one is chosen.
const changing = ref(false)

const noSaves = computed(() => {
  const a = profile.setup?.active
  return a?.kind === 'none' ? a : null
})
const activeProfile = computed(() => {
  const a = profile.setup?.active
  return a?.kind === 'active' ? a : null
})
const choosing = computed(
  () => profile.setup?.active.kind === 'needsChoice' || changing.value,
)

const choose = async (id: string) => {
  await profile.choose(id)
  changing.value = false
}
</script>

<template>
  <div class="flex max-w-250 flex-col gap-4">
    <!-- No "Screen 0" and no settings copy: this is where you choose the save to play with
         (`docs/BACKLOG.md` B17). The welcome flow and the preview per save come with the
         design pass; what goes now is the wording that was false. -->
    <ScreenHeader :icon="SaveIcon" :title="t('profile.title')">{{
      t('profile.intro')
    }}</ScreenHeader>
    <ProfileError
      v-if="profile.status === LoadStatus.Failed"
      :error="profile.error"
      @retry="profile.load()"
    />
    <template v-else-if="profile.setup">
      <ChainCard :setup="profile.setup" />
      <NoSavesCard
        v-if="noSaves"
        :reason="noSaves.reason"
        :diagnostics="profile.setup.diagnostics"
        @retry="profile.load()"
      />
      <CandidatesCard
        v-else-if="choosing"
        :active="profile.setup.active"
        :candidates="profile.setup.candidates"
        :busy="profile.status === LoadStatus.Loading"
        @choose="choose"
        @cancel="changing = false"
      />
      <template v-else-if="activeProfile">
        <ActiveProfileCard
          :active="activeProfile"
          :game="profile.setup.game"
          @change="changing = true"
          @reload="profile.load()"
        />
        <SectionsCard v-if="profile.summary" :summary="profile.summary" />
      </template>
    </template>
    <div v-else class="flex flex-col gap-4">
      <Skeleton class="h-30 w-full" />
      <Skeleton class="h-50 w-full" />
    </div>
  </div>
</template>
