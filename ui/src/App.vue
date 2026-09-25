<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import AboutDialog from '@/components/shell/AboutDialog.vue'
import NavBar from '@/components/shell/NavBar.vue'
import ProfileIndicator from '@/components/shell/ProfileIndicator.vue'
import SearchPalette from '@/components/search/SearchPalette.vue'
import SectionSidebar from '@/components/shell/SectionSidebar.vue'
import SidebarEdgeTab from '@/components/shell/SidebarEdgeTab.vue'
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
} from '@/lib/shell/sectionNav'
import type { SidebarEntry } from '@/lib/shell/sectionNav'
import { SidebarWidth, clampSidebarWidth } from '@/lib/shell/sidebarWidth'
import type { TabView } from '@/lib/shell/tabs'
import { TooltipProvider } from '@/components/ui/tooltip'
import { usePointerShortcut } from '@/composables/usePointerShortcut'
import { useShortcut } from '@/composables/useShortcut'
import { useFormat } from '@/composables/useFormat'
import { useToday } from '@/composables/useToday'
import { useMessages } from '@/i18n'
import type { Point } from '@/lib/drag/dragList'
import { indicator } from '@/lib/profile/profileView'
import { welcomeState } from '@/lib/profile/welcomeView'
import { shortcutAction } from '@/lib/scale/shortcut'
import {
  HistoryAction,
  historyAction,
  pointerHistoryAction,
} from '@/lib/shell/navigation'
import { togglesSidebar } from '@/lib/shell/sidebarToggle'
import {
  closeWindow,
  minimizeWindow,
  toggleMaximizeWindow,
  watchWindowFocus,
  windowSize,
} from '@/lib/window/appWindow'
import { appEventHandlers } from '@/lib/window/appEventHandlers'
import { watchAppEvents } from '@/lib/window/appEvents'
import { subscriptions } from '@/lib/window/subscriptions'
import {
  setSidebarCollapsed,
  setSidebarWidth,
  sidebarCollapsed,
  sidebarWidth as storedWidth,
} from '@/lib/window/layout'
import { useWindowSession } from '@/composables/useWindowSession'
import { routeOrigin } from '@/router/routeTable'
import ProgressGate from '@/screens/ProgressGate.vue'
import WelcomeScreen from '@/screens/welcome/WelcomeScreen.vue'
import { useProfileStore } from '@/stores/profile'
import { useQueueStore } from '@/stores/queue'
import { useSettingsStore } from '@/stores/settings'
import { tabLabel, tabLocation } from '@/stores/tabModel'
import { useTabsStore } from '@/stores/tabs'
import { useLiveStore, useRunsStore } from '@/stores/views'
import { useWikiStore } from '@/stores/wiki'

const router = useRouter()
const tabs = useTabsStore()
// Which instance of the screen is showing: one per history entry of each tab (see the template).
const screenKey = computed(() => `${tabs.activeId}#${tabs.active?.index ?? 0}`)
const profile = useProfileStore()
const wiki = useWikiStore()
// Read again when another window writes: the plan's queue, and the size of the interface.
const queue = useQueueStore()
const settings = useSettingsStore()
const { t } = useMessages()
const fmt = useFormat()

// Everything this window says to the others, and hears from them: a window born from a
// tear-off asks for its tabs here, and any window can be handed one.
useWindowSession()

const focused = ref(true)
const listening = subscriptions()
onMounted(async () => {
  void profile.load()
  await listening.add(() =>
    watchWindowFocus((value) => {
      focused.value = value
    }),
  )
  // What each event does is `appEventHandlers`' to say, where it is tested.
  await listening.add(() =>
    watchAppEvents(
      appEventHandlers({
        profile,
        settings,
        queue,
        runs: useRunsStore(),
        live: useLiveStore(),
      }),
    ),
  )
})
onUnmounted(listening.stop)

// A tab torn out of the strip. It leaves the bar at once and belongs to nobody until the drag
// ends: the store keeps it in flight, and the two endings below dispose of it.
const liftTab = (index: number) => {
  const tab = tabs.tabs[index]
  if (tab) tabs.liftOut(tab.id)
}

// Where it landed. A label is a strip — this window's own included — and null is the bare
// desktop, where it gets a window of its own, sized like this one: the size the user chose, in
// the place they dropped it.
const settleTab = async (target: string | null, at: Point, origin: Point) => {
  if (target !== null) await tabs.settleTo(target, at)
  else await tabs.settleInNewWindow(origin, await windowSize())
}

// The router shows the active tab: selecting, closing, navigating a tab or walking its
// history moves it.
watch(
  () => tabs.location,
  (location) => {
    if (location) void router.replace(location)
  },
  { immediate: true },
)

// Back and forward, on the two gestures a browser has taught: the side buttons of the mouse
// and Alt with an arrow. Both walk the active tab's own history, so switching tab finds each
// one where it was left.
const walk = (action: HistoryAction) => {
  if (action === HistoryAction.Back) tabs.back()
  else tabs.forward()
}
useShortcut((event) => {
  const action = historyAction(event)
  if (action === null) return false
  walk(action)
  return true
})
usePointerShortcut(pointerHistoryAction, walk)

// A page tab reads as its page's title once the wiki index knows it; every other label is
// a message.
const tabViews = computed<TabView[]>(() =>
  tabs.tabs.map((tab) => {
    const location = tabLocation(tab)
    const label = tabLabel(location, wiki.titleOf)
    return {
      id: tab.id,
      label: typeof label === 'string' ? t(label) : label.text,
      origin: routeOrigin[location.name],
    }
  }),
)

// The sidebar shows the active tab's section, until the navbar or the cog picks another.
const browsing = ref<SidebarSection>(SidebarSection.Progress)
watch(
  () => tabs.location?.name,
  (name) => {
    if (!name) return
    // A search tab belongs to neither section: the sidebar stays where the user left it.
    const section = sectionOfOrigin(routeOrigin[name])
    if (section) browsing.value = section
  },
  { immediate: true },
)

// **Not this window's number.** The sidebar's width is one value for the app, kept in
// `lib/window/layout.ts` and written into the session beside the windows (3.7c): what is here is
// only the reading of it. `null` is "nobody ever sized it", which is the default and not a stored
// width; anything stored goes through the clamp, so a number written by an older build with other
// bounds comes back inside today's.
const sidebarWidth = computed<number>({
  get: () =>
    storedWidth.value === null
      ? SidebarWidth.Default
      : clampSidebarWidth(storedWidth.value),
  set: (px) => setSidebarWidth(px),
})
// Folded to its icons by the tab on its edge or by `Ctrl+B`; one value for the app, like the width.
// The shell being too narrow folds it too, in CSS alone, and never writes here.
const toggleSidebar = () => setSidebarCollapsed(!sidebarCollapsed.value)
const header = computed(() => sidebarHeaders[browsing.value])
const entries = computed(() => sidebarEntries[browsing.value])

// Ctrl+click opens the entry in a new tab, as a browser does.
const openEntry = (entry: SidebarEntry, event: MouseEvent) =>
  tabs.go(entry.location, event.ctrlKey)

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

// `Ctrl+B` folds the sidebar from anywhere — but only where there is one: over the welcome it
// would change a sidebar nobody can see, and the shell would come up folded for no reason shown.
useShortcut((event) => {
  if (takeover.value || !togglesSidebar(event)) return false
  toggleSidebar()
  return true
})

// The date moves at midnight, so "today" on the indicator becomes "yesterday" without a
// relaunch (card #80, P10).
const today = useToday()
const indicatorView = computed(() =>
  profile.setup
    ? indicator(profile.setup.active, today.value, fmt.locale())
    : null,
)

// Whether this window draws the welcome instead of the shell. Every case is decided in
// `welcomeView`, where it is tested; here it only chooses which half of the template runs.
const welcome = computed(() =>
  welcomeState(profile.setup, profile.status, profile.picking),
)
const takeover = computed(() => welcome.value.kind !== 'hidden')
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
        :bare="takeover"
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
      <!-- The welcome is the whole window while it is up: no navigation bar, no sidebar, no
           tabs. `useWindowSession()` keeps running in the setup above, so this window goes on
           holding its tabs behind it — which is why the takeover is a state and not a route. -->
      <WelcomeScreen v-if="takeover" :state="welcome" />
      <template v-else>
        <NavBar
          :section="navSectionOf(browsing)"
          :settings-active="browsing === SidebarSection.Settings"
          :focused="focused"
          :can-back="tabs.canBack"
          :can-forward="tabs.canForward"
          @back="tabs.back"
          @forward="tabs.forward"
          @update:section="
            (section, event) => openSection(sidebarSectionOf(section), event)
          "
          @search="paletteOpen = true"
          @settings="openSection(SidebarSection.Settings, $event)"
          @about="aboutOpen = true"
        >
          <template #status>
            <!-- The indicator opens the same welcome, over the app: one implementation of the
               choice, and the card you picked last time is the one already selected. -->
            <ProfileIndicator :view="indicatorView" @open="profile.pick()" />
          </template>
        </NavBar>
        <!-- The `shell` container is the window's width, and stays the window's width when the
             sidebar collapses inside it — which is why the sidebar's threshold hangs here and not
             on `page`, where a collapse would widen the content, re-cross the threshold and
             oscillate (spec 3.13a §6). -->
        <!-- `group/shell` is the sidebar's second input: the shell carries `data-sidebar` while
             `sidebarCollapsed` is on — whether the edge tab or `Ctrl+B` set it — and the CSS does
             the folding (spec 3.13a §6, card #54). -->
        <div
          :data-sidebar="sidebarCollapsed ? 'collapsed' : undefined"
          class="group/shell @container/shell flex min-h-0 flex-1"
        >
          <SectionSidebar
            v-model:width="sidebarWidth"
            :title="t(header.title)"
            :hint="t(header.hint)"
            class="border-y-0 border-l-0"
          >
            <template #icon><component :is="header.icon" /></template>
            <template #edge>
              <SidebarEdgeTab
                :collapsed="sidebarCollapsed"
                @toggle="toggleSidebar"
              />
            </template>
            <SidebarItem
              v-for="entry in entries"
              :key="entry.key"
              :active="isEntryActive(entry, tabs.location)"
              :label="t(entry.label)"
              @click="openEntry(entry, $event)"
            >
              <template #icon><component :is="entry.icon" /></template>
            </SidebarItem>
          </SectionSidebar>
          <!-- The page box (spec 3.13a §4): it scrolls nothing and pads nothing. The padding is
               the screen's, on the box that scrolls, so every scrollbar sits on the window's edge
               (card #63). It is also the `page` container every threshold is measured against. -->
          <main class="@container/page min-h-0 min-w-0 flex-1 overflow-hidden">
            <!-- **One instance of a screen per history entry of a tab** (#79): without the key the
                 router reuses one component for every tab on the same route, and what a screen
                 holds locally walks from one tab into the next. The index is safe in the key
                 because typing does not move it: a refinement of the same view replaces the entry
                 in place (`tabModel`'s `goTo`). -->
            <RouterView v-slot="{ Component, route }">
              <ProgressGate v-if="route.meta.needsProfile">
                <component :is="Component" :key="screenKey" />
              </ProgressGate>
              <component :is="Component" v-else :key="screenKey" />
            </RouterView>
          </main>
        </div>
        <AboutDialog v-model:open="aboutOpen" />
        <SearchPalette v-model:open="paletteOpen" />
      </template>
    </div>
  </TooltipProvider>
</template>
