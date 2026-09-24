<script setup lang="ts">
import { TriangleAlertIcon } from '@lucide/vue'
import { computed } from 'vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import {
  CardCollapsible,
  CardCollapsibleContent,
  CardCollapsibleTrigger,
} from '@/components/ui/card'
import { i18n, useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import type { SaveDiagnostic, SaveSummary } from '@/lib/ipc/types'
import { formatCount, sectionLabel } from '@/lib/profile/profileView'

const props = defineProps<{ summary: SaveSummary }>()
const { t } = useMessages()

const sections = computed(() => {
  const locale = i18n.global.locale.value
  return props.summary.sections.map((s) => {
    const label = sectionLabel(s.kind)
    return {
      kind: s.kind,
      name: label ? t(label) : s.kind,
      count: formatCount(s.count, locale),
    }
  })
})

const diagnosticText = (d: SaveDiagnostic): string => {
  switch (d.kind) {
    case 'unexpectedKind':
      return `${t('profile.saveDiagnostics.unexpectedKind')} (${d.expected}, ${d.found})`
    case 'sectionOverrun':
      return `${t('profile.saveDiagnostics.sectionOverrun')} (${d.section})`
    case 'trailingBytes':
      return t('profile.saveDiagnostics.trailingBytes')
    default:
      return assertNever(d)
  }
}
</script>

<template>
  <CardCollapsible :default-open="true">
    <CardCollapsibleTrigger>
      {{ t('profile.read.title') }}
      <template #summary
        >{{ summary.sections.length }}
        {{ t('profile.read.sections') }}</template
      >
    </CardCollapsibleTrigger>
    <CardCollapsibleContent class="flex flex-col gap-3">
      <div class="grid grid-cols-5 gap-2">
        <div
          v-for="s in sections"
          :key="s.kind"
          class="flex flex-col gap-1.5 border border-border bg-data px-2.5 py-2"
        >
          <span class="text-label text-subtle-foreground">{{ s.kind }}</span>
          <span class="text-row text-foreground">{{ s.name }}</span>
          <span class="text-caption text-foreground-soft">{{ s.count }}</span>
        </div>
      </div>
      <Alert
        v-if="summary.diagnostics.length"
        :variant="AlertVariant.Destructive"
      >
        <TriangleAlertIcon />
        <AlertTitle>{{ t('profile.read.diagnostics') }}</AlertTitle>
        <AlertDescription>
          <span v-for="(d, i) in summary.diagnostics" :key="i" class="block">{{
            diagnosticText(d)
          }}</span>
        </AlertDescription>
      </Alert>
      <p
        class="border border-border bg-data px-3 py-3 text-caption text-foreground-soft"
      >
        {{ t('profile.read.note') }}
      </p>
    </CardCollapsibleContent>
  </CardCollapsible>
</template>
