<script setup lang="ts">
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import WikiBlocks from '@/components/wiki/WikiBlocks.vue'
import { useMessages } from '@/i18n'
import type { Section, Target, UnlockNode } from '@/lib/ipc/types'
import { sectionAnchor } from './wikiOutline'
import { sectionText } from '@/lib/wiki/wikiLabels'

defineProps<{
  sections: Section[]
  iconFor?: (target: Target) => string | null
  canOpen?: (target: Target) => boolean
  nodeFor?: (target: Target) => UnlockNode | null
}>()
const emit = defineEmits<{ navigate: [target: Target, newTab: boolean] }>()
const { t } = useMessages()
</script>

<template>
  <!-- The sections in the dataset's order, each under its name for the reader; a page with
       none says so instead of ending under the card.

       Each heading carries the anchor the outline links to (`wikiOutline.ts`), and the
       margin above it is what the heading is scrolled to rather than its own top edge:
       without it the jump lands with the title flush against the top of the frame. -->
  <div v-if="sections.length > 0" class="flex flex-col gap-6">
    <section
      v-for="(section, index) in sections"
      :key="index"
      :id="sectionAnchor(index)"
      class="flex scroll-mt-5 flex-col gap-2.5"
    >
      <h2
        class="flex items-center gap-2.5 border-b border-secondary-edge pb-1.5 text-heading text-highlight"
      >
        <span class="h-4 w-1 shrink-0 bg-band" />{{
          t(sectionText[section.kind])
        }}
      </h2>
      <WikiBlocks
        :blocks="section.blocks"
        :icon-for="iconFor"
        :can-open="canOpen"
        :node-for="nodeFor"
        @navigate="(target, newTab) => emit('navigate', target, newTab)"
      />
    </section>
  </div>
  <EmptyCategory v-else>{{ t('wiki.states.noSections') }}</EmptyCategory>
</template>
