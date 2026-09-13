// The colour a window created at runtime opens on. The first window takes it from
// `crates/app/tauri.conf.json`; a window born from a tear-off is created by the frontend,
// where that file is not available — so it takes the `--color-background` token, the same
// source the config and `index.html` are kept equal to (`assets/background.test.ts`). A
// literal here would be a fourth copy, and the one nothing checks.

// Tauri wants a triple, not a string. Null for anything that is not `#rrggbb`: a window
// created with a bad colour would open white, and white is the one thing B18 is about.
export const hexToColor = (hex: string): [number, number, number] | null => {
  const m = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(hex.trim())
  if (!m) return null
  const [r, g, b] = [m[1], m[2], m[3]].map((part) => parseInt(part ?? '', 16))
  return r === undefined || g === undefined || b === undefined
    ? null
    : [r, g, b]
}

// Read from the document, so it follows the token and not a copy of it. Undefined when the
// stylesheet has not landed: a window without the hint still paints itself from its own
// `index.html`, a frame later.
export const windowBackground = (): [number, number, number] | undefined => {
  const token = getComputedStyle(document.documentElement).getPropertyValue(
    '--color-background',
  )
  return hexToColor(token) ?? undefined
}
