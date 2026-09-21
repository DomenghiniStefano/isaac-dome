import { computed, type ComputedRef } from 'vue'
import { useProfileStore } from '@/stores/profile'

// The one URL no row carries: the picture drawn where a row's own icon could not be
// resolved (B69). It comes with the setup state — the app cannot build an `isaac://` path
// outside the Tauri crate — and it is `null` whenever the game is not there to serve it,
// which is what keeps a machine without the game exactly as quiet as it was.
export const useUnknownIcon = (): ComputedRef<string | null> => {
  const profile = useProfileStore()
  return computed(() => profile.setup?.unknownIconUrl ?? null)
}
