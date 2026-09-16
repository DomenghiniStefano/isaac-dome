import type { Point } from '@/lib/drag/dragList'
import type { WindowMessage } from './messages'
import type { MonitorArea, WindowBox, WindowPort } from './windowPort'

// `?windows=fake` invents a second window to the right of the first, so a tear-off and a dock
// can be watched on the development server. Every message is logged rather than sent: there is
// nobody to send it to. Without the parameter there is one window and nothing to tear off,
// which is what the other fixtures expect.
const FakeParam = 'windows'
const FakeValue = 'fake'

const mainBox: WindowBox = {
  label: 'main',
  left: 0,
  top: 0,
  width: 1280,
  height: 800,
  scaleFactor: 1,
}

const otherBox: WindowBox = {
  label: 'win-fake',
  left: 1300,
  top: 60,
  width: 900,
  height: 700,
  scaleFactor: 1,
}

// One screen, wide enough to hold both fake windows, so a stored box invented here is never
// clamped away by the very fixture that made it.
const fakeMonitor: MonitorArea = {
  left: 0,
  top: 0,
  width: 2560,
  height: 1040,
  scaleFactor: 1,
}

export const fakeWindows = (): WindowPort => {
  // **`window` itself can be missing**, not only Tauri: Vitest runs these modules in node, where
  // there is no document to read a query parameter from. The rule this file lives under is that
  // it degrades instead of throwing, and an environment with no DOM is the case it did not
  // cover — the fake simply turns its logging off there.
  const on =
    typeof window !== 'undefined' &&
    new URLSearchParams(window.location.search).get(FakeParam) === FakeValue
  const handlers: ((m: WindowMessage) => void)[] = []
  const say = (what: string, detail: unknown) => {
    if (on) console.info(`[windows] ${what}`, detail)
  }
  return {
    label: () => mainBox.label,
    isMain: () => true,
    list: () => Promise.resolve(on ? [mainBox, otherBox] : [mainBox]),
    labels: () =>
      Promise.resolve(on ? [mainBox.label, otherBox.label] : [mainBox.label]),
    monitors: () => Promise.resolve([fakeMonitor]),
    create: (label: string, at: Point, size: Point) => {
      say('create', { label, at, size })
      return Promise.resolve()
    },
    send: (label: string, message: WindowMessage) => {
      say('send', { label, message })
      return Promise.resolve()
    },
    broadcast: (message: WindowMessage) => {
      say('broadcast', message)
      for (const handler of handlers) handler(message)
      return Promise.resolve()
    },
    listen: (handler: (m: WindowMessage) => void) => {
      handlers.push(handler)
      return Promise.resolve(() => {
        const at = handlers.indexOf(handler)
        if (at >= 0) handlers.splice(at, 1)
      })
    },
    focus: (label: string) => {
      say('focus', label)
      return Promise.resolve()
    },
    closeSelf: () => {
      say('closeSelf', null)
      return Promise.resolve()
    },
    self: () => Promise.resolve(mainBox),
  }
}
