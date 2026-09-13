import { describe, expect, it } from 'vitest'
import colors from './theme/colors.css?raw'
import html from '../../index.html?raw'
import tauri from '../../../crates/app/tauri.conf.json'
import { hexToColor } from '@/lib/window/windowBackground'

// The window's background is declared in three places by necessity — a CSS token, the
// Tauri window's config (JSON, not CSS), and the document itself, which the webview paints
// before any stylesheet arrives. If they drift, the app opens on a colour that isn't its
// own, or flashes white (`docs/BACKLOG.md` B18), and nothing else fails.
const token = /--color-background:\s*(#[0-9a-f]{6})\s*;/.exec(colors)?.[1]

describe('the window opens on the app colour', () => {
  it('has a background token to begin with', () => {
    expect(token).toBeDefined()
  })

  it('paints the document with it before any stylesheet', () => {
    expect(html).toMatch(
      new RegExp(String.raw`<html[^>]*background-color:\s*${token}`),
    )
  })

  it("gives the window the same colour, so the frame isn't white either", () => {
    const [window] = tauri.app.windows
    expect(window?.backgroundColor?.toLowerCase()).toBe(token)
  })

  // The window is declared there and built by Rust on demand (`crates/app/src/window.rs`), so
  // that the tray and the first launch open the same window from the same recipe. Were it
  // created at startup too, Tauri would build it *and* `setup` would try to build a second one
  // under the same label — a startup error, which this test explains before anyone reads it.
  it('leaves the window for Rust to create', () => {
    const [window] = tauri.app.windows
    expect(window?.create).toBe(false)
    expect(window?.label).toBe('main')
  })

  // A window born from a tear-off is created at runtime, where the config's colour is not
  // available: it takes the token itself, as a triple, so there is no fourth copy of the
  // value to drift. The conversion is the only new thing, and it is pinned here rather than
  // in its own file, because this is the drift it would cause.
  it('turns the token into the triple a window is created with', () => {
    expect(hexToColor('#150e0d')).toEqual([21, 14, 13])
    expect(hexToColor('#ffffff')).toEqual([255, 255, 255])
    expect(token).toBeDefined()
    expect(hexToColor(token ?? '')).not.toBeNull()
  })

  it('answers null for anything that is not a six-digit hex colour', () => {
    expect(hexToColor('')).toBeNull()
    expect(hexToColor('150e0d')).toBeNull()
    expect(hexToColor('#150e0')).toBeNull()
    expect(hexToColor('#gggggg')).toBeNull()
  })
})
