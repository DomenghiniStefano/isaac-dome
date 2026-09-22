<script setup lang="ts">
import { ListChecksIcon } from '@lucide/vue'
import { useMessages } from '@/i18n'
import ScreenHeader from '../ScreenHeader.vue'

const { t } = useMessages()
</script>

<template>
  <!-- The band the screen opens on, the same grammar as Completion's and a wiki page's
       (spec §4.2): the light comes from the corner and falls back into the page, and the
       grain stands in for the shadow the skin does not have. It is the full width of the
       page box — the screen hands it the gutter rather than the band taking it, which is the
       one thing `WikiLanding` paid for the hard way.
       It carries the title and **nothing countable**: §5 declined a count at the top of this
       screen, and a band is not a reason to go looking for one. -->
  <header class="relative border-b border-hairline hero-wash px-5.5 py-5">
    <span class="pointer-events-none absolute inset-0 hero-grain" />
    <div class="relative">
      <ScreenHeader :icon="ListChecksIcon" :title="t('routes.goals')">{{
        t('goals.intro')
      }}</ScreenHeader>
    </div>
  </header>
</template>
