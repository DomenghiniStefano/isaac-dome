<script setup lang="ts">
import { inject } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { PoolMembershipView } from '@/lib/ipc/types'
import InfoboxRow from '../InfoboxRow.vue'
import { infoboxLinksKey } from './links'

// The pools the installed game lists this item in (design decision 4): `null` or
// `undefined` — no game, or a target the game has no pools concept for at all (a trinket,
// see `wiki_pools.rs`'s doc comment) — draws no row, "absent, not wrong". `[]` is a real
// answer (24 collectibles are in no pool) and draws the row with "none", same as every
// other fact this card states.
defineProps<{ pools?: PoolMembershipView[] | null }>()
const links = inject(infoboxLinksKey, null)
const { t } = useMessages()
</script>

<template>
  <InfoboxRow
    v-if="pools && pools.length === 0"
    :label="t('wiki.infobox.pools')"
  >
    <EmptyValue>{{ t('wiki.infobox.none') }}</EmptyValue>
  </InfoboxRow>
  <InfoboxRow
    v-else-if="pools && pools.length > 0"
    :label="t('wiki.infobox.pools')"
    class="gap-1.5 pt-1"
    value-class="flex flex-wrap items-baseline gap-x-3 gap-y-1"
  >
    <span
      v-for="pool in pools"
      :key="pool.label"
      class="inline-flex items-baseline gap-1"
    >
      <Button
        v-if="pool.target && (links?.canOpen?.(pool.target) ?? true)"
        :variant="ButtonVariant.Ref"
        :size="ButtonSize.Inline"
        @click="links?.navigate(pool.target, $event.ctrlKey)"
        >{{ pool.label }}</Button
      >
      <span
        v-else
        :class="
          cn(
            pool.target &&
              'border-b border-dotted border-secondary-edge text-subtle-foreground',
          )
        "
        >{{ pool.label }}</span
      >
      <span class="text-label text-subtle-foreground">· {{ pool.weight }}</span>
    </span>
  </InfoboxRow>
</template>
