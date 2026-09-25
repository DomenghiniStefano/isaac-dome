import { MainLabel } from './windowPort'
import type { MonitorArea, WindowBox, WindowPort } from './windowPort'

// A production build outside Tauri: nothing ships that way, but every module under
// `lib/window/` degrades there instead of throwing (the frontend conventions, rule 3). One
// window, one monitor the size of it, and messages that go nowhere because there is nobody to
// send them to. It exists so that `fakeWindows`, the development stand-in with a second window
// to tear off into, stays out of the production bundle.
//
// `MainLabel` is read inside the functions and never while this module loads: `windowPort.ts`
// imports this file, so reading it at the top level would meet it before it is defined.
const box = (): WindowBox => ({
  label: MainLabel,
  left: 0,
  top: 0,
  // Where there is no window at all (a test in Node), the size the fake uses.
  width: globalThis.innerWidth ?? 1280,
  height: globalThis.innerHeight ?? 800,
  scaleFactor: 1,
})

const monitor = (): MonitorArea => {
  const { left, top, width, height, scaleFactor } = box()
  return { left, top, width, height, scaleFactor }
}

export const soleWindowPort: WindowPort = {
  label: () => MainLabel,
  isMain: () => true,
  list: async () => [box()],
  labels: async () => [MainLabel],
  monitors: async () => [monitor()],
  create: async () => {},
  send: async () => {},
  broadcast: async () => {},
  listen: async () => () => {},
  focus: async () => {},
  closeSelf: async () => {},
  self: async () => box(),
}
