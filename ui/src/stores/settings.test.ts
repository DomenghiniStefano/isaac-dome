import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { resetFixtures } from '@/lib/ipc/fixtures'
import {
  settingsAnswer,
  useAutostartRegistry,
} from '@/lib/ipc/fixtures/settings'
import { AutostartFailure, AutostartReason, IoReason } from '@/lib/ipc/types'
import type { IpcError } from '@/lib/ipc/types'
import { ScaleAction } from '@/lib/scale/shortcut'
import { useSettingsStore } from './settings'

// The fixtures write every setting they are handed; a refused write is the one case they have
// no scenario for, so the next write rejects with whatever `refusal.next` holds, once.
const refusal = vi.hoisted(() => ({ next: undefined as unknown }))
vi.mock('@/lib/ipc/settings', async (importOriginal) => {
  const real = await importOriginal<typeof import('@/lib/ipc/settings')>()
  const refusable =
    <A extends unknown[], R>(write: (...args: A) => Promise<R>) =>
    (...args: A): Promise<R> => {
      const next = refusal.next
      if (next === undefined) return write(...args)
      refusal.next = undefined
      return Promise.reject(next)
    }
  return {
    ...real,
    setScale: refusable(real.setScale),
    setStayInBackground: refusable(real.setStayInBackground),
    setResumeTabs: refusable(real.setResumeTabs),
    setAutoUpdate: refusable(real.setAutoUpdate),
  }
})
// The size is applied to the document's root, and a test has no document: what is under test
// is what the store says, not the pixels.
vi.mock('@/lib/scale/apply', () => ({ applyScale: vi.fn() }))

const notWritable: IpcError = {
  kind: 'settingsNotWritable',
  reason: { kind: 'io', reason: IoReason.PermissionDenied },
}

// Through the development fixtures, as the browser runs them. Starting with Windows is the one
// setting whose truth is outside the app — it is the registry, not `settings.json` — so what the
// store may never do is show a position nobody answered with.
beforeEach(() => {
  setActivePinia(createPinia())
  resetFixtures()
  refusal.next = undefined
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

describe('useSettingsStore, the size', () => {
  it('is applied, saved, and kept as the backend answered it', async () => {
    const store = useSettingsStore()
    await store.setScale(125)
    expect(store.scale).toBe(125)
    expect(settingsAnswer().scale).toBe(125)
    expect(store.saveFailed).toBe(false)
  })

  it('a size off the ladder is the default, not the nearest step', async () => {
    const store = useSettingsStore()
    await store.setScale(123)
    expect(store.scale).toBe(100)
  })

  it('a refused write keeps the size asked for and says the write failed', async () => {
    const store = useSettingsStore()
    refusal.next = notWritable
    await store.setScale(150)
    expect(store.scale).toBe(150)
    expect(store.saveFailed).toBe(true)
    expect(store.saveError).toEqual(notWritable)
  })

  it('walks the ladder one step at a time, and back to the default', async () => {
    const store = useSettingsStore()
    await store.step(ScaleAction.In)
    expect(store.scale).toBe(110)
    await store.step(ScaleAction.Out)
    await store.step(ScaleAction.Out)
    expect(store.scale).toBe(90)
    await store.step(ScaleAction.Reset)
    expect(store.scale).toBe(100)
  })
})

// The three switches that change what the app is doing now: moved first, saved after, and a
// failed write said rather than undone.
describe.each([
  {
    name: 'staying in the background',
    set: (s: ReturnType<typeof useSettingsStore>, on: boolean) =>
      s.setStayInBackground(on),
    shown: (s: ReturnType<typeof useSettingsStore>) => s.stayInBackground,
    saved: () => settingsAnswer().stayInBackground,
  },
  {
    name: 'resuming the tabs',
    set: (s: ReturnType<typeof useSettingsStore>, on: boolean) =>
      s.setResumeTabs(on),
    shown: (s: ReturnType<typeof useSettingsStore>) => s.resumeTabs,
    saved: () => settingsAnswer().resumeTabs,
  },
  {
    name: 'looking for updates',
    set: (s: ReturnType<typeof useSettingsStore>, on: boolean) =>
      s.setAutoUpdate(on),
    shown: (s: ReturnType<typeof useSettingsStore>) => s.autoUpdate,
    saved: () => settingsAnswer().autoUpdate,
  },
])('useSettingsStore, $name', ({ set, shown, saved }) => {
  it('moves the switch and saves it', async () => {
    const store = useSettingsStore()
    await set(store, false)
    expect(shown(store)).toBe(false)
    expect(saved()).toBe(false)
    expect(store.saveFailed).toBe(false)
    expect(store.saveError).toBeNull()
  })

  it('a refused write leaves the switch where it was moved and says so', async () => {
    const store = useSettingsStore()
    refusal.next = notWritable
    await set(store, false)
    expect(shown(store)).toBe(false)
    expect(saved()).toBe(true)
    expect(store.saveFailed).toBe(true)
    expect(store.saveError).toEqual(notWritable)
  })

  // Something that is not ours — the backend never answered — still failed, and there is no
  // sentence of ours to say about it.
  it('a failure that is not ours fails with no error to name', async () => {
    const store = useSettingsStore()
    refusal.next = new Error('the backend went away')
    await set(store, false)
    expect(store.saveFailed).toBe(true)
    expect(store.saveError).toBeNull()
  })

  it('the next write that lands clears the failure', async () => {
    const store = useSettingsStore()
    refusal.next = notWritable
    await set(store, false)
    await set(store, true)
    expect(store.saveFailed).toBe(false)
    expect(store.saveError).toBeNull()
    expect(saved()).toBe(true)
  })
})
