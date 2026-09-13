<script setup lang="ts">
import { InfoIcon, TriangleAlertIcon } from '@lucide/vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import { useMessages } from '@/i18n'
import { Severity } from '@/lib/diagnostics/spec'
import type { DiagnosticEntry } from '@/lib/diagnostics/spec'

// Four screens drew alerts four ways, and the fourth copy already differed from the first
// in ways nobody decided. This is the one that draws them; what each screen keeps is its
// table, which is the part that is actually its own.
defineProps<{ entries: DiagnosticEntry[] }>()
const { t } = useMessages()

// A sentence can come from more than one key, each with its own values: the store reason is
// "what failed" plus "why". Joining them here rather than in Rust is what N2 was for.
const say = (parts: DiagnosticEntry['title']): string =>
  parts.map((p) => t(p.key, p.params)).join(' ')
</script>

<template>
  <template v-for="entry in entries" :key="entry.key">
    <!-- A note coexists with real rows: a line under them, never an alarm standing in for
         content that is actually there. -->
    <p
      v-if="entry.severity === Severity.Note"
      class="text-caption text-subtle-foreground tabular-nums"
    >
      {{ say(entry.body) }}
    </p>
    <Alert
      v-else
      :variant="
        entry.severity === Severity.Warning
          ? AlertVariant.Destructive
          : undefined
      "
    >
      <TriangleAlertIcon v-if="entry.severity === Severity.Warning" />
      <InfoIcon v-else />
      <AlertTitle class="tabular-nums">{{ say(entry.title) }}</AlertTitle>
      <AlertDescription
        :class="entry.action ? 'flex flex-col items-start gap-2' : undefined"
      >
        {{ say(entry.body) }}
        <slot v-if="entry.action" name="action" :action="entry.action" />
      </AlertDescription>
    </Alert>
  </template>
</template>
