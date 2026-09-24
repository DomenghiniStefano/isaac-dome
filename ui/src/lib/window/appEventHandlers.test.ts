import { describe, expect, it, vi } from 'vitest'
import { AppEvent } from './appEvents'
import { appEventHandlers } from './appEventHandlers'

const stores = () => ({
  profile: { stopPicking: vi.fn(), load: vi.fn(async () => {}) },
  settings: { load: vi.fn(async () => {}) },
  queue: { load: vi.fn(async () => {}) },
  runs: { refresh: vi.fn(async () => {}) },
  live: { refresh: vi.fn(async () => {}) },
})

describe('appEventHandlers', () => {
  // Card #80, item 06: the run archive grows in the background and says so with
  // `runs-changed`. The handler was `() => undefined` under a comment saying Live and Runs were
  // still placeholders, so a Live screen kept showing what it read when it mounted.
  it('refreshes Live and Runs when the archive changes', () => {
    const s = stores()
    appEventHandlers(s)[AppEvent.RunsChanged]()
    expect(s.runs.refresh).toHaveBeenCalledOnce()
    expect(s.live.refresh).toHaveBeenCalledOnce()
  })

  it('reads the profile again, and closes the picker, when another window chose one', () => {
    const s = stores()
    appEventHandlers(s)[AppEvent.ProfileChanged]()
    expect(s.profile.stopPicking).toHaveBeenCalledOnce()
    expect(s.profile.load).toHaveBeenCalledOnce()
  })

  it('reads the settings and the plan again when they were written elsewhere', () => {
    const s = stores()
    const h = appEventHandlers(s)
    h[AppEvent.SettingsChanged]()
    h[AppEvent.PlanChanged]()
    expect(s.settings.load).toHaveBeenCalledOnce()
    expect(s.queue.load).toHaveBeenCalledOnce()
  })

  it('leaves the two events their own screens listen for to those screens', () => {
    const s = stores()
    const h = appEventHandlers(s)
    h[AppEvent.RollChanged]()
    h[AppEvent.UpdateChanged]()
    expect(s.runs.refresh).not.toHaveBeenCalled()
    expect(s.queue.load).not.toHaveBeenCalled()
  })
})
