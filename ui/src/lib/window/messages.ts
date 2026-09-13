import type { Point } from '@/lib/drag/dragList'
import type { Tab } from '@/stores/tabModel'

// What windows say to each other. These never pass through Rust: they are frontend types on a
// frontend channel, not the IPC contract, and the rules that govern view-models do not apply.
// One Tauri event carries all of them, so a window has one listener and one exhaustive switch.
export const WindowEventName = 'isaac://window'

export const WindowMessageKind = {
  Ready: 'ready',
  Seed: 'seed',
  Docked: 'docked',
  Hovering: 'hovering',
  HoverLeft: 'hoverLeft',
  Focused: 'focused',
} as const
export type WindowMessageKind =
  (typeof WindowMessageKind)[keyof typeof WindowMessageKind]

// A tab as it travels between windows: everything a tab is **except its identity**, which the
// receiving window mints for itself — two windows must never hold the same tab id. Defined by
// subtraction on purpose: the day a tab gains a field, it crosses to the other window without
// a line changing here, and nothing in this module has to know what a tab is made of.
export type TabSeed = Omit<Tab, 'id'>

// "I exist and I hold nothing": broadcast by a window that is not `main` when it mounts.
export interface ReadyMessage {
  kind: typeof WindowMessageKind.Ready
  label: string
}

// The answer to Ready, from the window that created it.
export interface SeedMessage {
  kind: typeof WindowMessageKind.Seed
  tabs: TabSeed[]
  activeIndex: number
}

// A tab dropped on this window's strip. `at` is in desktop physical pixels: the receiver
// converts it, because only the receiver knows its own position and scale factor.
export interface DockedMessage {
  kind: typeof WindowMessageKind.Docked
  tab: TabSeed
  at: Point
}

// A tab is being dragged over this window's strip right now: draw the marker. Sent at most
// once per animation frame.
export interface HoveringMessage {
  kind: typeof WindowMessageKind.Hovering
  at: Point
}

export interface HoverLeftMessage {
  kind: typeof WindowMessageKind.HoverLeft
}

// Broadcast by every window when it gains the focus. With no z-order API (tauri#5656) this is
// the only thing that tells two overlapping windows apart, and it costs one string.
export interface FocusedMessage {
  kind: typeof WindowMessageKind.Focused
  label: string
}

export type WindowMessage =
  | ReadyMessage
  | SeedMessage
  | DockedMessage
  | HoveringMessage
  | HoverLeftMessage
  | FocusedMessage
