// The sizes the interface is drawn at, as percentages: Discord's zoom levels, which are
// Chromium's own ladder (`docs/BACKLOG.md` B26). The same eleven numbers as
// `crates/ipc/src/settings.rs`, and the test beside this file reads them from there: two
// ladders that drift are a slider that saves a value the backend refuses.
export const scalePercents: number[] = [
  50, 67, 75, 80, 90, 100, 110, 125, 150, 175, 200,
]

// The size the design file is measured at, and what anything unknown reads as.
export const DefaultScale = 100

// A percentage from outside, made one of ours. Not the nearest step: a number off the
// ladder comes from a file we didn't write, and reading it as "about 125" would draw the
// app at a size nobody designed.
export const snapPercent = (percent: number): number =>
  scalePercents.includes(percent) ? percent : DefaultScale

const indexOf = (percent: number): number =>
  scalePercents.indexOf(snapPercent(percent))

// One step up or down, stopping at the ends: a shortcut is not a second scale.
export const nextPercent = (percent: number): number =>
  scalePercents[Math.min(indexOf(percent) + 1, scalePercents.length - 1)] ??
  DefaultScale

export const previousPercent = (percent: number): number =>
  scalePercents[Math.max(indexOf(percent) - 1, 0)] ?? DefaultScale

// The slider walks positions, not percentages: the ladder's uneven spacing is the slider's
// even spacing, which is what Discord draws.
export const stepIndex = (percent: number): number => indexOf(percent)
export const percentAt = (index: number): number =>
  scalePercents[index] ?? DefaultScale

export const scaleFactor = (percent: number): number =>
  snapPercent(percent) / 100

// How many times a 32px sprite is drawn. The game's art is pixel art: at a fraction of its
// size the browser smears it, so the multiple is a whole number, `round(2 x factor)`, and
// the boxes that *are* a sprite are sized from it rather than from their own rem value.
export const spriteMultiple = (percent: number): number =>
  Math.max(1, Math.round(2 * scaleFactor(percent)))
