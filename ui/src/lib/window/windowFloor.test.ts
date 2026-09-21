import { describe, expect, it } from 'vitest'
import conf from '../../../../crates/app/tauri.conf.json?raw'
import { WindowFloor } from './windowFloor'

// Two files declare the floor and neither can see the other (spec 3.13a §8): the Tauri config is
// the recipe for the main window and the one the tray rebuilds, while a torn-off tab is built in
// the frontend from the constant below, by `windowPort.ts`. A floor written in one place only
// would hold for every window except the ones the user makes by hand.
//
// Read as `?raw` and parsed here, the way `lib/scale/rows.test.ts` reads the CSS it checks. Not
// `node:fs`: `vue-tsc` typechecks this file and `ui/` carries no Node types, and adding them to
// settle one import would be a dependency bought for a comment's worth of convenience.
const windows = (
  JSON.parse(conf) as {
    app: { windows: Array<{ minWidth?: number; minHeight?: number }> }
  }
).app.windows

describe('the window floor', () => {
  it('is 640 x 480, the size the compact layouts are designed against', () => {
    expect(WindowFloor.Width).toBe(640)
    expect(WindowFloor.Height).toBe(480)
  })

  it('is the same number in the Tauri config as in the constant', () => {
    const main = windows[0]
    expect(main?.minWidth).toBe(WindowFloor.Width)
    expect(main?.minHeight).toBe(WindowFloor.Height)
  })
})
