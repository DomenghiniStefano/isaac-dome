<script setup lang="ts">
import {
  ToggleGroup,
  ToggleGroupItem,
  ToggleGroupType,
} from '@/components/ui/toggle-group'
import { useMessages } from '@/i18n'
import { RoomKindView } from '@/lib/ipc/types'

// The brush. Erasing is the same control with no kind picked, not a second button: painting
// and erasing are one gesture with two values, and a separate eraser would let the screen be
// in a state where neither is chosen.
const ERASE = 'erase'

defineProps<{ brush: RoomKindView | null }>()
const emit = defineEmits<{ pick: [brush: RoomKindView | null] }>()
const { t } = useMessages()

// The order rooms are offered in: the two that shape a floor first, then the special rooms a
// rule actually names, then the rest. A kind nobody paints still has to be here — a missing
// one would make a floor undrawable, not merely inconvenient.
const kinds = [
  RoomKindView.Start,
  RoomKindView.Normal,
  RoomKindView.Boss,
  RoomKindView.Treasure,
  RoomKindView.Shop,
  RoomKindView.Curse,
  RoomKindView.Challenge,
  RoomKindView.Sacrifice,
  RoomKindView.Arcade,
  RoomKindView.Library,
  RoomKindView.Miniboss,
  RoomKindView.Secret,
  RoomKindView.SuperSecret,
  RoomKindView.UltraSecret,
]

const onUpdate = (value: unknown): void => {
  if (value === ERASE || value === '' || value === null) {
    emit('pick', null)
    return
  }
  const kind = kinds.find((k) => k === value)
  emit('pick', kind ?? null)
}
</script>

<template>
  <ToggleGroup
    :type="ToggleGroupType.Single"
    :model-value="brush ?? ERASE"
    class="w-fit flex-wrap"
    @update:model-value="onUpdate"
  >
    <ToggleGroupItem :value="ERASE">{{ t('floor.erase') }}</ToggleGroupItem>
    <ToggleGroupItem v-for="kind in kinds" :key="kind" :value="kind">
      {{ t(`floor.room.${kind}`) }}
    </ToggleGroupItem>
  </ToggleGroup>
</template>
