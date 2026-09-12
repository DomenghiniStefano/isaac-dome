import { describe, expect, it } from 'vitest'
import colors from './theme/colors.css?raw'
import html from '../../index.html?raw'
import tauri from '../../../crates/app/tauri.conf.json'

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
})
