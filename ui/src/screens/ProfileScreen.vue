<script setup lang="ts">
import { vScrollMemory } from '@/directives/scrollMemory'
import { SaveIcon } from '@lucide/vue'
import { computed } from 'vue'
import ScreenSkeleton from '@/components/data-state/ScreenSkeleton.vue'
import { SkeletonBlock } from '@/components/data-state/skeletonBlock'
import { useMessages } from '@/i18n'
import { LoadStatus } from '@/stores/loadStatus'
import { useProfileStore } from '@/stores/profile'
import ScreenHeader from './ScreenHeader.vue'
import ActiveProfileCard from './profile/ActiveProfileCard.vue'
import ChainCard from './profile/ChainCard.vue'
import ProfileError from './profile/ProfileError.vue'
import SectionsCard from './profile/SectionsCard.vue'

const profile = useProfileStore()
const { t } = useMessages()

const activeProfile = computed(() => profile.activeProfile)
</script>

<template>
  <!-- Since 3.8 this is not where you choose: the welcome is, above the router, and the
       indicator opens it. What is left here is where the numbers come from — the chain, the
       save in use, and what the file let us read. The candidate table and the "nothing
       found" card went with the choice; this screen is only ever open with a profile. -->
  <div
    v-scroll-memory="'page'"
    class="flex h-full flex-col gap-4 overflow-y-auto px-5.5 pt-5 pb-15"
  >
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
      <template v-if="activeProfile">
        <ActiveProfileCard
          :active="activeProfile"
          :game="profile.setup.game"
          @change="profile.pick()"
          @reload="profile.load()"
        />
        <SectionsCard v-if="profile.summary" :summary="profile.summary" />
      </template>
    </template>
    <ScreenSkeleton
      v-else
      untitled
      :blocks="[SkeletonBlock.ProfileCard, SkeletonBlock.TallCard]"
    />
  </div>
</template>
