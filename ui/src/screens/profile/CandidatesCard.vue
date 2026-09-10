<script setup lang="ts">
import { CheckIcon, TriangleAlertIcon } from '@lucide/vue'
import { computed, ref } from 'vue'
import { Alert, AlertDescription, AlertVariant } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Button, ButtonVariant } from '@/components/ui/button'
import {
  CardCollapsible,
  CardCollapsibleContent,
  CardCollapsibleTrigger,
} from '@/components/ui/card'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { i18n, useMessages } from '@/i18n'
import type { ActiveProfile, CandidateView } from '@/lib/ipc/types'
import { candidateSourceLabel } from '@/lib/profile/profileLabels'
import {
  editionShort,
  formatCount,
  formatModified,
} from '@/lib/profile/profileView'

const props = defineProps<{
  active: ActiveProfile
  candidates: CandidateView[]
  busy: boolean
}>()
const emit = defineEmits<{ choose: [id: string]; cancel: [] }>()
const { t } = useMessages()

// Nothing is preselected while a choice is pending: the suggestion stays a suggestion
// (§4.1). When changing an active profile, the current one starts selected.
const selected = ref<string | undefined>(
  props.active.kind === 'active' ? props.active.profile.id : undefined,
)

const savedGone = computed(
  () =>
    props.active.kind === 'needsChoice' &&
    props.active.reason.kind === 'savedProfileGone',
)

const rows = computed(() => {
  const now = new Date()
  const locale = i18n.global.locale.value
  return props.candidates.map((c) => ({
    ...c,
    edition: editionShort(c.prefix),
    modified: formatModified(c.modifiedUnix, now, locale),
    size: formatCount(c.sizeBytes, locale),
  }))
})

const confirm = () => {
  if (selected.value) emit('choose', selected.value)
}
</script>

<template>
  <CardCollapsible :default-open="true">
    <CardCollapsibleTrigger>
      {{ t('profile.pick.title') }} · {{ candidates.length }}
      <template #summary>{{ t('profile.pick.summary') }}</template>
    </CardCollapsibleTrigger>
    <CardCollapsibleContent class="flex flex-col gap-3">
      <Alert v-if="savedGone" :variant="AlertVariant.Destructive">
        <TriangleAlertIcon />
        <AlertDescription>{{ t('profile.pick.savedGone') }}</AlertDescription>
      </Alert>
      <RadioGroup v-model="selected" class="block">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead class="w-9" />
              <TableHead>{{ t('profile.pick.edition') }}</TableHead>
              <TableHead>{{ t('profile.pick.slot') }}</TableHead>
              <TableHead>{{ t('profile.pick.foundIn') }}</TableHead>
              <TableHead>{{ t('profile.pick.modified') }}</TableHead>
              <TableHead class="text-right">{{
                t('profile.pick.size')
              }}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow
              v-for="row in rows"
              :key="row.id"
              class="cursor-pointer"
              @click="selected = row.id"
            >
              <TableCell>
                <RadioGroupItem
                  :value="row.id"
                  :aria-label="`${row.edition} ${row.slot}`"
                />
              </TableCell>
              <TableCell>{{ row.edition }}</TableCell>
              <TableCell>{{ row.slot }}</TableCell>
              <TableCell>
                <span class="flex items-center gap-2">
                  <span class="text-caption text-subtle-foreground">{{
                    t(candidateSourceLabel[row.source])
                  }}</span>
                  <Badge v-if="row.suggested">{{
                    t('profile.pick.suggested')
                  }}</Badge>
                </span>
              </TableCell>
              <TableCell class="text-foreground-soft">{{
                row.modified ?? t('profile.active.unknownDate')
              }}</TableCell>
              <TableCell class="text-right text-foreground-soft"
                >{{ row.size }} {{ t('profile.active.bytes') }}</TableCell
              >
            </TableRow>
          </TableBody>
        </Table>
      </RadioGroup>
      <div class="flex flex-wrap items-center justify-between gap-3">
        <span class="text-caption text-subtle-foreground">{{
          t('profile.pick.hint')
        }}</span>
        <div class="flex gap-2">
          <Button
            v-if="active.kind === 'active'"
            :variant="ButtonVariant.Outline"
            @click="emit('cancel')"
            >{{ t('profile.pick.cancel') }}</Button
          >
          <Button :disabled="!selected || busy" @click="confirm">
            <CheckIcon />{{ t('profile.pick.use') }}
          </Button>
        </div>
      </div>
    </CardCollapsibleContent>
  </CardCollapsible>
</template>
