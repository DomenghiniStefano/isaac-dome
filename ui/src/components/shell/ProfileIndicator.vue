<script setup lang="ts">
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import { cn } from '@/lib/cn'
import type { IndicatorView } from '@/lib/profile/profileView'

const props = defineProps<{ view: IndicatorView | null }>()
const emit = defineEmits<{ open: [] }>()
const { t } = useMessages()

// Which profile every number reads (DESIGN-BRIEF.md §4.1), one click from changing it.
const label = computed((): string => {
  const v = props.view
  if (v === null) return ''
  switch (v.kind) {
    case 'active':
      return [v.edition, `${t('indicator.slot')} ${v.slot}`, v.modified]
        .filter(Boolean)
        .join(' · ')
    case 'noProfile':
      return t('indicator.noProfile')
    case 'notFound':
      return t('indicator.notFound')
    default:
      return assertNever(v)
  }
})
</script>

<template>
  <Button
    :variant="ButtonVariant.Field"
    :size="ButtonSize.Compact"
    class="max-w-60 shrink-0"
    @click="emit('open')"
  >
    <Skeleton v-if="view === null" class="h-3 w-24" />
    <template v-else>
      <span
        :class="
          cn(
            'size-1.5 shrink-0',
            view.kind === 'active' ? 'bg-state-done' : 'bg-state-unexpected',
          )
        "
      />
      <span class="truncate text-foreground-soft">{{ label }}</span>
    </template>
  </Button>
</template>
