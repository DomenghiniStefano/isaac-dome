<script setup lang="ts">
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import type { Section } from '@/lib/ipc/types'
import { SectionKind } from '@/lib/ipc/types'
import { outlineOf, type OutlineItem } from './wikiOutline'
import { inlinePlain } from '@/lib/wiki/inlinePlain'
import { sectionText } from '@/lib/wiki/wikiLabels'

const props = defineProps<{ sections: Section[] }>()
const { t } = useMessages()

const items = computed(() => outlineOf(props.sections))

// `Other`'s label is its own title — the same one `WikiSections` renders as the heading —
// read as plain text: the row is a `<button>` already, so a reference inside it would be a
// button inside a button, which is what `WikiInline`'s links would draw.
const label = (item: OutlineItem): string =>
  item.kind === SectionKind.Other
    ? inlinePlain(item.title)
    : t(sectionText[item.kind])

// The page scrolls in the screen's own container, so the heading is reached by asking the
// element to come into view rather than by a hash the memory router would have to carry.
// A heading that isn't in the document yet simply doesn't move: nothing is thrown.
const jump = (id: string) => {
  document.getElementById(id)?.scrollIntoView({ block: 'start' })
}
</script>

<template>
  <!-- Nothing at all on a page with one section: `outlineOf` decides that, and its test
       says why — an index of a single entry is furniture, not navigation. -->
  <Card v-if="items.length > 0">
    <CardHeader>
      <CardTitle>{{ t('wiki.outline') }}</CardTitle>
    </CardHeader>
    <CardContent class="flex flex-col items-start gap-0.5 p-1.5">
      <Button
        v-for="item in items"
        :key="item.id"
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Row"
        class="w-full justify-start px-2 text-left text-row text-foreground-soft"
        @click="jump(item.id)"
        >{{ label(item) }}</Button
      >
    </CardContent>
  </Card>
</template>
