<script setup lang="ts">
import { CogIcon, InfoIcon, SearchIcon } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Kbd, KbdGroup } from '@/components/ui/kbd'
import { useMessages } from '@/i18n'
import { AppName } from '@/lib/constants/app'
import { AriaCurrent } from '@/lib/constants/aria'
import { KeyName } from '@/lib/constants/keyNames'
import { NavSection, navSectionIcon, navSectionLabel } from './navSection'

// `null` when the sidebar shows a section the switch doesn't name (Settings).
const props = defineProps<{ section: NavSection | null; focused: boolean }>()
const emit = defineEmits<{
  'update:section': [section: NavSection]
  search: []
  settings: []
  about: []
}>()
const { t } = useMessages()

const sections = Object.values(NavSection)
const current = (s: NavSection) =>
  props.section === s ? AriaCurrent.Page : undefined
</script>

<template>
  <!-- Which part of the app I'm in (Chrome e Stati.dc.html, "Finestra"). The search field is
       a trigger: it opens the palette, it isn't the input. -->
  <nav
    :data-focused="focused"
    class="group flex h-navbar items-center gap-2.5 border-b border-hairline bg-navbar pr-2.5"
  >
    <div
      class="flex h-full w-brand shrink-0 items-center gap-2 border-r border-hairline pl-3"
    >
      <span
        class="size-3.5 border-2 border-primary bg-sheet group-data-[focused=false]:border-border"
      />
      <span
        class="text-control text-foreground group-data-[focused=false]:text-faint-foreground"
        >{{ AppName }}</span
      >
    </div>
    <div class="flex h-full items-stretch gap-0.5">
      <Button
        v-for="s in sections"
        :key="s"
        :variant="ButtonVariant.Section"
        :size="ButtonSize.Section"
        :aria-current="current(s)"
        @click="emit('update:section', s)"
      >
        <component :is="navSectionIcon[s]" />{{ t(navSectionLabel[s]) }}
      </Button>
    </div>
    <div class="min-w-0 flex-1" />
    <slot name="status" />
    <Button
      :variant="ButtonVariant.Field"
      :size="ButtonSize.Compact"
      class="w-search min-w-10 shrink"
      @click="emit('search')"
    >
      <SearchIcon class="size-3" />
      <span class="min-w-0 flex-1 truncate text-left">{{
        t('shell.search')
      }}</span>
      <KbdGroup>
        <Kbd>{{ KeyName.Ctrl }}</Kbd>
        <Kbd>{{ KeyName.K }}</Kbd>
      </KbdGroup>
    </Button>
    <div class="flex items-center gap-0.5">
      <Button
        :variant="ButtonVariant.Chrome"
        :size="ButtonSize.Icon"
        :aria-label="t('shell.settings')"
        @click="emit('settings')"
      >
        <CogIcon class="size-3.5" />
      </Button>
      <Button
        :variant="ButtonVariant.Chrome"
        :size="ButtonSize.Icon"
        :aria-label="t('shell.about')"
        @click="emit('about')"
      >
        <InfoIcon class="size-3.5" />
      </Button>
    </div>
  </nav>
</template>
