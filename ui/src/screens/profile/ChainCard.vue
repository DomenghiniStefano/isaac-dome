<script setup lang="ts">
import { computed } from 'vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import {
  CardCollapsible,
  CardCollapsibleContent,
  CardCollapsibleTrigger,
} from '@/components/ui/card'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import type { SetupState } from '@/lib/ipc/types'
import { chainLinkLabel, linkStateLabel } from '@/lib/profile/profileLabels'
import { LinkState, chainLinks, editionShort } from '@/lib/profile/profileView'
import type { ChainRow } from '@/lib/profile/profileView'

const props = defineProps<{ setup: SetupState }>()
const { t } = useMessages()

const rows = computed(() => chainLinks(props.setup))

// A state is its badge's shape too: found and chosen are done, missing is an alert, the two
// that ask something of the user are neutral tags (cyan, the export's choice, is focus).
const badge: Record<LinkState, BadgeVariant> = {
  [LinkState.Found]: BadgeVariant.Done,
  [LinkState.Chosen]: BadgeVariant.Done,
  [LinkState.Missing]: BadgeVariant.Unexpected,
  [LinkState.YourChoice]: BadgeVariant.Tag,
  [LinkState.Several]: BadgeVariant.Tag,
}

const detail = (row: ChainRow): string => {
  const d = row.detail
  switch (d.kind) {
    case 'hint':
      return d.hint
    case 'count':
      return `${d.n} ${t('profile.chain.candidates')}`
    case 'profile':
      return `${editionShort(d.candidate.prefix)} · ${t('indicator.slot')} ${d.candidate.slot}`
    case 'none':
      return ''
    default:
      return assertNever(d)
  }
}
</script>

<template>
  <CardCollapsible :default-open="true">
    <CardCollapsibleTrigger>
      {{ t('profile.chain.title') }}
      <template #summary>{{ t('profile.chain.summary') }}</template>
    </CardCollapsibleTrigger>
    <CardCollapsibleContent class="flex flex-col gap-2">
      <div
        v-for="row in rows"
        :key="row.link"
        class="flex items-center gap-3 border border-hairline bg-data px-3 py-2"
      >
        <span class="w-30 shrink-0 text-row text-foreground">{{
          t(chainLinkLabel[row.link])
        }}</span>
        <span
          class="min-w-0 flex-1 truncate text-caption text-foreground-soft"
          >{{ detail(row) }}</span
        >
        <Badge :variant="badge[row.state]">{{
          t(linkStateLabel[row.state])
        }}</Badge>
      </div>
    </CardCollapsibleContent>
  </CardCollapsible>
</template>
