<script setup lang="ts">
import {
  ArrowLeftIcon,
  ArrowRightIcon,
  CogIcon,
  InfoIcon,
  SearchIcon,
} from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Kbd, KbdGroup } from '@/components/ui/kbd'
import { useMessages } from '@/i18n'
import { AppName } from '@/lib/constants/app'
import { AriaCurrent } from '@/lib/constants/aria'
import { KeyName } from '@/lib/constants/keyNames'
import BrandMark from './BrandMark.vue'
import {
  NavSection,
  navSectionIcon,
  navSectionLabel,
} from '@/lib/shell/navSection'

// `section` is the active tab's, `null` while a settings page is open; `settingsActive`
// says so, because the cog is a section like the other two and has to read as lit when you
// are in it (`docs/BACKLOG.md` B24).
const props = defineProps<{
  section: NavSection | null
  settingsActive: boolean
  focused: boolean
  canBack: boolean
  canForward: boolean
}>()
// The click's event travels with the section: Ctrl opens the section's first page in a new
// tab, as a sidebar entry does.
const emit = defineEmits<{
  'update:section': [section: NavSection, event: MouseEvent]
  search: []
  settings: [event: MouseEvent]
  about: []
  back: []
  forward: []
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
      class="flex h-full w-brand min-w-0 shrink items-center gap-2 border-r border-hairline pl-3"
    >
      <BrandMark class="text-primary group-data-[focused=false]:text-border" />
      <span
        class="truncate text-control text-foreground group-data-[focused=false]:text-faint-foreground"
        >{{ AppName }}</span
      >
    </div>
    <!-- Back and forward walk the active tab's own history, never the window's: each tab
         remembers where it has been, and the two arrows are the same gesture as the side
         buttons of the mouse and Alt with an arrow. -->
    <div class="flex items-center gap-0.5">
      <Button
        :variant="ButtonVariant.Chrome"
        :size="ButtonSize.Icon"
        :aria-label="t('shell.back')"
        :disabled="!canBack"
        @click="emit('back')"
      >
        <ArrowLeftIcon class="size-3.5" />
      </Button>
      <Button
        :variant="ButtonVariant.Chrome"
        :size="ButtonSize.Icon"
        :aria-label="t('shell.forward')"
        :disabled="!canForward"
        @click="emit('forward')"
      >
        <ArrowRightIcon class="size-3.5" />
      </Button>
    </div>
    <div class="flex h-full items-stretch gap-0.5">
      <Button
        v-for="s in sections"
        :key="s"
        :variant="ButtonVariant.Section"
        :size="ButtonSize.Section"
        :aria-current="current(s)"
        @click="emit('update:section', s, $event)"
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
      <!-- The cog is the third section: it navigates like the other two and lights up the
           same way while a settings page is open. -->
      <Button
        :variant="ButtonVariant.Chrome"
        :size="ButtonSize.Icon"
        :aria-label="t('shell.settings')"
        :aria-current="settingsActive ? AriaCurrent.Page : undefined"
        class="aria-[current=page]:bg-data aria-[current=page]:text-highlight"
        @click="emit('settings', $event)"
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
