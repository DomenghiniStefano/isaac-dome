<script setup lang="ts">
import { computed } from 'vue'
import { Card, CardContent } from '@/components/ui/card'
import { Field, FieldDescription, FieldLabel } from '@/components/ui/field'
import { MultiSelect } from '@/components/ui/multi-select'
import { Switch } from '@/components/ui/switch'
import { useMessages } from '@/i18n'
import type { PresetView, RollRowView } from '@/lib/ipc/types'
import { rowOptions, selectionOf, toggledSelection } from './rollText'

// The preset's two `MultiSelect`s and two switches. Every change emits the whole `PresetView`
// and the screen writes it immediately: there is no save button, by decision, so a partial
// preset never sits unsaved between two changes.
const props = defineProps<{
  characters: RollRowView[]
  columns: RollRowView[]
  preset: PresetView
}>()
const emit = defineEmits<{ 'update:preset': [preset: PresetView] }>()
const { t } = useMessages()

const characterOptions = computed(() => rowOptions(props.characters))
const columnOptions = computed(() => rowOptions(props.columns))
const characterIds = computed(() => props.characters.map((row) => row.id))
const columnIds = computed(() => props.columns.map((row) => row.id))

const setCharacters = (picked: string[]) =>
  emit('update:preset', {
    ...props.preset,
    characters: toggledSelection(characterIds.value, picked),
  })
const setColumns = (picked: string[]) =>
  emit('update:preset', {
    ...props.preset,
    columns: toggledSelection(columnIds.value, picked),
  })
const setIncludeTaken = (value: boolean) =>
  emit('update:preset', { ...props.preset, includeTaken: value })
const setOnlyPlayable = (value: boolean) =>
  emit('update:preset', { ...props.preset, onlyPlayable: value })
</script>

<template>
  <Card>
    <CardContent class="flex flex-col gap-4">
      <div class="flex flex-wrap gap-2">
        <MultiSelect
          :label="t('roll.panel.characters')"
          :options="characterOptions"
          :picked="selectionOf(characters)"
          @update:picked="setCharacters"
        />
        <MultiSelect
          :label="t('roll.panel.columns')"
          :options="columnOptions"
          :picked="selectionOf(columns)"
          @update:picked="setColumns"
        />
      </div>
      <Field>
        <div class="flex items-center gap-2">
          <Switch
            id="roll-include-taken"
            :model-value="preset.includeTaken"
            @update:model-value="setIncludeTaken"
          />
          <FieldLabel for="roll-include-taken">{{
            t('roll.panel.includeTaken')
          }}</FieldLabel>
        </div>
        <FieldDescription>{{
          t('roll.panel.includeTakenHint')
        }}</FieldDescription>
      </Field>
      <Field>
        <div class="flex items-center gap-2">
          <Switch
            id="roll-only-playable"
            :model-value="preset.onlyPlayable"
            @update:model-value="setOnlyPlayable"
          />
          <FieldLabel for="roll-only-playable">{{
            t('roll.panel.onlyPlayable')
          }}</FieldLabel>
        </div>
        <FieldDescription>{{
          t('roll.panel.onlyPlayableHint')
        }}</FieldDescription>
      </Field>
    </CardContent>
  </Card>
</template>
