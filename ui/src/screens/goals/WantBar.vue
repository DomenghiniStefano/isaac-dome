<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { useSearch } from '@/composables/useSearch'
import { useMessages } from '@/i18n'
import { SearchLimit } from '@/lib/ipc/search'
import type { Target } from '@/lib/ipc/types'
import { wantable } from '@/lib/graph/wantLocation'

const emit = defineEmits<{ pick: [target: Target]; clear: [] }>()
const { t } = useMessages()
// The debounce and the command are `useSearch`'s, the palette's own: no second index and no
// second debounce, and the answer keeps the backend's order.
const { view, ask } = useSearch(SearchLimit.Palette)

const typed = ref('')
watch(typed, (query) => ask(String(query)))

// Only what the app can be asked for. The rule is `wantable`'s, tested there: a second list
// of kinds in this component would be a second answer to the same question.
const hits = computed(() => wantable(view.value?.hits ?? []))

const pick = (target: Target) => {
  typed.value = ''
  emit('pick', target)
}

const clear = () => {
  typed.value = ''
  emit('clear')
}
</script>

<template>
  <div class="flex flex-col gap-2">
    <div class="flex items-center gap-2">
      <Input v-model="typed" :placeholder="t('want.placeholder')" />
      <Button
        v-if="typed !== ''"
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Compact"
        @click="clear"
        >{{ t('want.clear') }}</Button
      >
    </div>
    <!-- The palette's row draws a *destination*, and a want is not one: the same thing is a
         wiki page, an Unlock row and a question about what to play, and only the third is
         this bar's business. So the row here is the name and its picture, nothing else. -->
    <ul v-if="hits.length > 0" class="flex flex-col gap-1">
      <li v-for="hit in hits" :key="hit.title">
        <Button
          :variant="ButtonVariant.Ghost"
          :size="ButtonSize.Compact"
          class="w-full justify-start gap-2"
          @click="pick(hit.target)"
        >
          <!-- The size has to come from here: `PixelSprite` draws the picture at whatever
               size the file is, and an achievement's sheet is not an item's icon. Without
               it the tall ones overlapped the rows under them, which no fixture showed
               until a transformation search put achievements next to items. -->
          <PixelSprite
            :url="hit.iconUrl"
            placeholder
            class="size-icon-compact shrink-0"
          />
          <span class="truncate">{{ hit.title }}</span>
        </Button>
      </li>
    </ul>
  </div>
</template>
