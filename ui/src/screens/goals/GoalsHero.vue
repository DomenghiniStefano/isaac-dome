<script setup lang="ts">
import { ListChecksIcon } from '@lucide/vue'
import { useMessages } from '@/i18n'
import ScreenHeader from '@/components/screen/ScreenHeader.vue'
import HeroBand from '@/components/screen/HeroBand.vue'

const { t } = useMessages()
</script>

<template>
  <!-- The band the screen opens on, the same one as Completion's and a wiki page's (spec §4.2).
       It carries the title and **nothing countable**: §5 declined a count at the top of this
       screen, and a band is not a reason to go looking for one. -->
  <HeroBand>
    <div class="relative">
      <ScreenHeader :icon="ListChecksIcon" :title="t('routes.goals')">{{
        t('goals.intro')
      }}</ScreenHeader>
    </div>
  </HeroBand>
</template>
