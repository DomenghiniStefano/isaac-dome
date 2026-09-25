import { computed, ref, watch } from 'vue'
import {
  SidebarSection,
  firstEntry,
  sectionOfOrigin,
  sidebarEntries,
  sidebarHeaders,
} from '@/lib/shell/sectionNav'
import type { SidebarEntry } from '@/lib/shell/sectionNav'
import { shownSidebarWidth } from '@/lib/shell/sidebarWidth'
import {
  setSidebarCollapsed,
  setSidebarWidth,
  sidebarCollapsed,
  sidebarWidth as storedWidth,
} from '@/lib/window/layout'
import { routeOrigin } from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'

// The section sidebar: which section it shows, how wide it is, whether it is folded, and where
// its entries go. Mounted once, by App.vue.
export const useSidebar = () => {
  const tabs = useTabsStore()

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
  // `lib/window/layout.ts` and written into the session beside the windows (3.7c): what is here
  // is only the reading of it (`shownSidebarWidth`).
  const width = computed<number>({
    get: () => shownSidebarWidth(storedWidth.value),
    set: (px) => setSidebarWidth(px),
  })
  // Folded to its icons by the tab on its edge or by `Ctrl+B`; one value for the app, like the
  // width. The shell being too narrow folds it too, in CSS alone, and never writes here.
  const toggle = () => setSidebarCollapsed(!sidebarCollapsed.value)
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

  return {
    browsing,
    width,
    collapsed: sidebarCollapsed,
    toggle,
    header,
    entries,
    openEntry,
    openSection,
  }
}
