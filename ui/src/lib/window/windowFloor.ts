// The smallest window the layout is designed against (spec 3.13a §8). Logical pixels, and **not**
// a theme token: this is a window's size, which the interface's scale never moves — a floor that
// grew with the scale would let the user shrink the app below its own design by making the text
// bigger.
export const WindowFloor = {
  Width: 640,
  Height: 480,
} as const
