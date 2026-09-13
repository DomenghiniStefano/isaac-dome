<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import AboutDialog from '@/components/shell/AboutDialog.vue'
import NavBar from '@/components/shell/NavBar.vue'
import ProfileIndicator from '@/components/shell/ProfileIndicator.vue'
import SearchPalette from '@/components/search/SearchPalette.vue'
import SectionSidebar from '@/components/shell/SectionSidebar.vue'
import SidebarItem from '@/components/shell/SidebarItem.vue'
import TitleBar from '@/components/shell/TitleBar.vue'
import {
  SidebarSection,
  firstEntry,
  isEntryActive,
  navSectionOf,
  sectionOfOrigin,
  sidebarEntries,
  sidebarHeaders,
  sidebarSectionOf,
} from '@/components/shell/sectionNav'
import type { SidebarEntry } from '@/components/shell/sectionNav'
import { SidebarWidth } from '@/components/shell/sidebarWidth'
import type { TabView } from '@/components/shell/tabs'
import { TooltipProvider } from '@/components/ui/tooltip'
import { useShortcut } from '@/composables/useShortcut'
import { i18n, useMessages } from '@/i18n'
import type { Point } from '@/lib/drag/dragList'
import { indicator } from '@/lib/profile/profileView'
import { shortcutAction } from '@/lib/scale/shortcut'
import {
  closeWindow,
  minimizeWindow,
  toggleMaximizeWindow,
  watchWindowFocus,
  windowSize,
} from '@/lib/window/appWindow'
import { AppEvent, watchAppEvents } from '@/lib/window/appEvents'
import { useWindowSession } from '@/lib/window/session'
import { RouteName, routeOrigin } from '@/router/routeTable'
import ProgressGate from '@/screens/ProgressGate.vue'
import { useProfileStore } from '@/stores/profile'
import { useQueueStore } from '@/stores/queue'
import { useSettingsStore } from '@/stores/settings'
import { tabLabel } from '@/stores/tabModel'
import { useTabsStore } from '@/stores/tabs'
import { useWikiStore } from '@/stores/wiki'

const router = useRouter()
const tabs = useTabsStore()
const profile = useProfileStore()
const wiki = useWikiStore()
// Read again when another window writes: the plan's queue, and the size of the interface.
const queue = useQueueStore()
const settings = useSettingsStore()
const { t } = useMessages()

// Everything this window says to the others, and hears from them: a window born from a
// tear-off asks for its tabs here, and any window can be handed one.
useWindowSession()

const focused = ref(true)
let stopWatchingFocus: (() => void) | undefined
let stopAppEvents: (() => void) | undefined
onMounted(async () => {
  void profile.load()
  stopWatchingFocus = await watchWindowFocus((value) => {
    focused.value = value
  })
  // A window never learns of a write it did not make, so it is told. The profile carries
  // through to every screen that reads the save (`useOnActiveProfile`); the queue store is
  // read again wherever it is mounted.
  stopAppEvents = await watchAppEvents({
    [AppEvent.ProfileChanged]: () => void profile.load(),
    [AppEvent.SettingsChanged]: () => void settings.load(),
    [AppEvent.PlanChanged]: () => void queue.load(),
  })
})
onUnmounted(() => {
  stopWatchingFocus?.()
  stopAppEvents?.()
})

// A tab torn out of the strip. It leaves the bar at once and belongs to nobody until the drag
// ends: the store keeps it in flight, and the two endings below dispose of it.
const liftTab = (index: number) => {
  const tab = tabs.tabs[index]
  if (tab) tabs.liftOut(tab.id)
}

// Where it landed. A label is a strip — this window's own included — and null is the bare
// desktop, where it gets a window of its own, sized like this one: the size the user chose, in
// the place they dropped it.
const settleTab = async (target: string | null, at: Point) => {
  if (target !== null) await tabs.settleTo(target, at)
  else await tabs.settleInNewWindow(at, await windowSize())
}

// The router shows the active tab: selecting, closing or navigating a tab moves it.
watch(
  () => tabs.active?.location,
  (location) => {
    if (location) void router.replace(location)
  },
  { immediate: true },
)

// A page tab reads as its page's title once the wiki index knows it; every other label is
// a message.
const tabViews = computed<TabView[]>(() =>
  tabs.tabs.map((tab) => {
    const label = tabLabel(tab.location, wiki.titleOf)
    return {
      id: tab.id,
      label: typeof label === 'string' ? t(label) : label.text,
      origin: routeOrigin[tab.location.name],
    }
  }),
)

// The sidebar shows the active tab's section, until the navbar or the cog picks another.
const browsing = ref<SidebarSection>(SidebarSection.Progress)
watch(
  () => tabs.active?.location.name,
  (name) => {
    if (!name) return
    // A search tab belongs to neither section: the sidebar stays where the user left it.
    const section = sectionOfOrigin(routeOrigin[name])
    if (section) browsing.value = section
  },
  { immediate: true },
)

const sidebarWidth = ref<number>(SidebarWidth.Default)
const header = computed(() => sidebarHeaders[browsing.value])
const entries = computed(() => sidebarEntries[browsing.value])

// Ctrl+click opens the entry in a new tab, as a browser does.
const openEntry = (entry: SidebarEntry, event: MouseEvent) => {
  if (event.ctrlKey) tabs.open(entry.location)
  else tabs.navigate(entry.location)
}

// Clicking a section goes to its first page at once, with no second click in the sidebar:
// this reverses Decision 5 of the shell spec, on purpose (`docs/BACKLOG.md` B24). The
// sidebar follows the tab through the watch above, so `browsing` needs no setting here.
const openSection = (section: SidebarSection, event: MouseEvent) => {
  openEntry(firstEntry(section), event)
}

// Informazioni is a dialog over the tab, not a tab of its own (`docs/BACKLOG.md` B25).
const aboutOpen = ref(false)

// The palette is mounted once, here: Ctrl+K reaches it from any tab, and so does the navbar's
// search field, which is a trigger and not an input.
const paletteOpen = ref(false)

// `Ctrl` `+` / `-` / `0` move the interface's size from anywhere, on the same ladder and the
// same saved value as the slider in Settings: a shortcut is not a second scale.
void settings.load()
useShortcut((event) => {
  const action = shortcutAction(event)
  if (action === null) return false
  void settings.step(action)
  return true
})

const indicatorView = computed(() =>
  profile.setup
    ? indicator(profile.setup.active, new Date(), i18n.global.locale.value)
    : null,
)
</script>

<template>
  <TooltipProvider>
    <div
      class="flex h-screen flex-col overflow-hidden bg-background text-foreground"
    >
      <TitleBar
        :tabs="tabViews"
        :active-id="tabs.activeId"
        :focused="focused"
        :incoming="tabs.incoming"
        @select="tabs.select"
        @close="tabs.close"
        @move="tabs.move"
        @add="tabs.open()"
        @aim="tabs.aim"
        @lift="liftTab"
        @settle="settleTab"
        @put-back="tabs.putBack"
        @minimize="minimizeWindow"
        @toggle-maximize="toggleMaximizeWindow"
        @close-window="closeWindow"
      />
      <NavBar
        :section="navSectionOf(browsing)"
        :settings-active="browsing === SidebarSection.Settings"
        :focused="focused"
        @update:section="
          (section, event) => openSection(sidebarSectionOf(section), event)
        "
        @search="paletteOpen = true"
        @settings="openSection(SidebarSection.Settings, $event)"
        @about="aboutOpen = true"
      >
        <template #status>
          <ProfileIndicator
            :view="indicatorView"
            @open="tabs.navigate({ name: RouteName.Profile })"
          />
        </template>
      </NavBar>
      <div class="flex min-h-0 flex-1">
        <SectionSidebar
          v-model:width="sidebarWidth"
          :title="t(header.title)"
          :hint="t(header.hint)"
          class="border-y-0 border-l-0"
        >
          <template #icon><component :is="header.icon" /></template>
          <SidebarItem
            v-for="entry in entries"
            :key="entry.key"
            :active="isEntryActive(entry, tabs.active?.location)"
            @click="openEntry(entry, $event)"
          >
            <template #icon><component :is="entry.icon" /></template>
            {{ t(entry.label) }}
          </SidebarItem>
        </SectionSidebar>
        <main class="min-w-0 flex-1 overflow-auto px-5.5 pt-5 pb-15">
          <RouterView v-slot="{ Component, route }">
            <ProgressGate v-if="route.meta.needsProfile">
              <component :is="Component" />
            </ProgressGate>
            <component :is="Component" v-else />
          </RouterView>
        </main>
      </div>
      <AboutDialog v-model:open="aboutOpen" />
      <SearchPalette v-model:open="paletteOpen" />
    </div>
  </TooltipProvider>
</template>
