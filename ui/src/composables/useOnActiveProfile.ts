import { computed, watch } from 'vue'
import { useProfileStore } from '@/stores/profile'

// A screen that reads the save loads again whenever the active profile changes: what it shows
// belongs to one profile and is never left on screen under another.
export const useOnActiveProfile = (load: () => Promise<void>): void => {
  const profile = useProfileStore()
  const activeId = computed(() => {
    const active = profile.setup?.active
    return active?.kind === 'active' ? active.profile.id : null
  })
  watch(
    activeId,
    (id) => {
      if (id !== null) void load()
    },
    { immediate: true },
  )
}
