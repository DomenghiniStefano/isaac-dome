<script setup lang="ts">
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import WikiBlocks from '@/components/wiki/WikiBlocks.vue'
import { useMessages } from '@/i18n'
import type { Section, Target } from '@/lib/ipc/types'
import { sectionText } from './wikiLabels'

defineProps<{
  sections: Section[]
  iconFor?: (target: Target) => string | null
  canOpen?: (target: Target) => boolean
}>()
const emit = defineEmits<{ navigate: [target: Target, newTab: boolean] }>()
const { t } = useMessages()
</script>

<template>
  <!-- The sections in the dataset's order, each under its name for the reader; a page with
       none says so instead of ending under the card. -->
  <div v-if="sections.length > 0" class="flex flex-col gap-5">
    <section
      v-for="(section, index) in sections"
      :key="index"
      class="flex flex-col gap-2"
    >
      <h2 class="border-b border-hairline pb-1 text-control text-highlight">
        {{ t(sectionText[section.kind]) }}
      </h2>
      <WikiBlocks
        :blocks="section.blocks"
        :icon-for="iconFor"
        :can-open="canOpen"
        @navigate="(target, newTab) => emit('navigate', target, newTab)"
      />
    </section>
  </div>
  <EmptyCategory v-else>{{ t('wiki.states.noSections') }}</EmptyCategory>
</template>
