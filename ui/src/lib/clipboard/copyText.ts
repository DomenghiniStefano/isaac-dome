// Puts a text on the clipboard and says whether it took. Never throws: a refused clipboard is an
// answer the button shows, not an exception that reaches whatever clicked it. The browser's own
// clipboard is enough here — WebView2 grants it to the page — so no window API is imported.
export const copyText = async (text: string): Promise<boolean> => {
  const clipboard = globalThis.navigator?.clipboard
  if (clipboard === undefined) return false
  try {
    await clipboard.writeText(text)
    return true
  } catch {
    return false
  }
}
