import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { resetFixtures } from '@/lib/ipc/fixtures'
import { useAutostartRegistry } from '@/lib/ipc/fixtures/settings'
import { AutostartFailure, AutostartReason } from '@/lib/ipc/types'
import { useSettingsStore } from './settings'

// Through the development fixtures, as the browser runs them. Starting with Windows is the one
// setting whose truth is outside the app — it is the registry, not `settings.json` — so what the
// store may never do is show a position nobody answered with.
beforeEach(() => {
  setActivePinia(createPinia())
  resetFixtures()
})

describe('useSettingsStore, starting with Windows', () => {
  it('is not offered where there is no registry to answer', async () => {
    // The development server, and a development build of the app: no plugin, no entry, and a
    // switch that would write `target\debug\app.exe` into somebody's login if it worked.
    const store = useSettingsStore()
    await store.refreshAutostart()
    expect(store.autostartAvailable).toBe(false)
    expect(store.autostartUnavailable).toBe(AutostartReason.NotSupported)
    expect(store.autostart).toBe(false)
  })

  it('takes the position the registry answered with', async () => {
    useAutostartRegistry({ enabled: true, refuses: null })
    const store = useSettingsStore()
    await store.refreshAutostart()
    expect(store.autostartAvailable).toBe(true)
    expect(store.autostart).toBe(true)
  })

  it('moves the switch after the answer and not before', async () => {
    useAutostartRegistry({ enabled: false, refuses: null })
    const store = useSettingsStore()
    await store.refreshAutostart()
    await store.setAutostart(true)
    expect(store.autostart).toBe(true)
    expect(store.saveFailed).toBe(false)
  })

  it('asked on and refused leaves the switch off and says so', async () => {
    // A policy, an antivirus, a profile that cannot write there. The other switches on this
    // screen move first and report a failed write, because the app is already behaving that
    // way; this one is about a login that has not happened, so a switch left on would be a
    // promise nothing kept.
    useAutostartRegistry({
      enabled: false,
      refuses: AutostartFailure.WriteRefused,
    })
    const store = useSettingsStore()
    await store.refreshAutostart()
    await store.setAutostart(true)
    expect(store.autostart).toBe(false)
    expect(store.saveFailed).toBe(true)
    expect(store.saveError).toEqual({
      kind: 'autostartNotWritable',
      reason: AutostartFailure.WriteRefused,
    })
  })

  it('keeps the two failures apart, because they are two different things to do', async () => {
    // A write Windows took and then ignored: the value is there and the Startup tab's own
    // switch is off, which no write from here can change. Telling the user "a policy is
    // refusing it" would send them looking in the wrong place.
    useAutostartRegistry({
      enabled: false,
      refuses: AutostartFailure.WriteIgnored,
    })
    const store = useSettingsStore()
    await store.refreshAutostart()
    await store.setAutostart(true)
    expect(store.autostart).toBe(false)
    expect(store.saveError).toEqual({
      kind: 'autostartNotWritable',
      reason: AutostartFailure.WriteIgnored,
    })
  })
})
