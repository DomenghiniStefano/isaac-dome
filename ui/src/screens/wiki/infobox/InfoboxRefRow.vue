<script setup lang="ts">
import { computed, inject } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import { FigureSize } from '@/components/wiki/figureSize'
import { useMessages } from '@/i18n'
import type { Target } from '@/lib/ipc/types'
import { pageKey } from '@/lib/wiki/pageKey'
import { useWikiStore } from '@/stores/wiki'
import InfoboxRow from '../InfoboxRow.vue'
import { titleOrKey } from '../infoboxRefs'
import { infoboxLinksKey } from './links'

// A row whose whole fact is a reference, not a word inside a longer sentence: what unlocks
// this page, what it unlocks, a transformation's contributors (design decision 9,
// "references keep their links and their small figures"). `WikiInline` draws no picture
// inside prose by design (its own doc comment) — this is the one place a reference stands
// alone, so it earns the same `WikiFigure` a name list gives one, at `Row` size.
//
// An empty `targets` draws the row's own "none": the caller that would rather hide the row
// entirely on an empty list (a transformation's `contributors`, where "none" would read as
// a measured claim nobody made) still wraps this component in its own `v-if`, the same way
// it wrapped the plain row before.
const props = defineProps<{ label: string; targets: Target[] }>()
const wiki = useWikiStore()
const links = inject(infoboxLinksKey, null)
const { t } = useMessages()

const rows = computed(() =>
  props.targets.map((target, index) => ({
    key: pageKey(target) ?? String(index),
    target,
    title: titleOrKey(target, wiki.titleOf),
    icon: wiki.iconFor(target),
    opens: links?.canOpen?.(target) ?? true,
  })),
)
</script>

<template>
  <InfoboxRow
    :label="label"
    class="gap-1.5 pt-1"
    value-class="flex flex-col gap-2"
  >
    <template v-if="rows.length > 0">
      <span v-for="row in rows" :key="row.key" class="flex items-center gap-2">
        <WikiFigure
          :target="row.target"
          :url="row.icon"
          :size="FigureSize.Row"
        />
        <Button
          v-if="row.opens"
          :variant="ButtonVariant.Ref"
          :size="ButtonSize.Inline"
          @click="links?.navigate(row.target, $event.ctrlKey)"
          >{{ row.title }}</Button
        >
        <span
          v-else
          class="border-b border-dotted border-secondary-edge text-subtle-foreground"
          >{{ row.title }}</span
        >
      </span>
    </template>
    <EmptyValue v-else>{{ t('wiki.infobox.none') }}</EmptyValue>
  </InfoboxRow>
</template>
