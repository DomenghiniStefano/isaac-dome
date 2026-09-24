import { describe, expect, it } from 'vitest'
import { MainLabel } from './windowPort'
import { soleWindowPort } from './soleWindow'

// Card #81, V10: a production build outside Tauri — which nothing ships, but the rule is that
// every module under `lib/window/` degrades there instead of throwing — gets one window and no
// messages, without carrying the development fake into the bundle.
describe('soleWindowPort', () => {
  it('is the one main window and nothing else', async () => {
    expect(soleWindowPort.label()).toBe(MainLabel)
    expect(soleWindowPort.isMain()).toBe(true)
    expect(await soleWindowPort.labels()).toEqual([MainLabel])
    const boxes = await soleWindowPort.list()
    expect(boxes.map((b) => b.label)).toEqual([MainLabel])
    expect(await soleWindowPort.self()).toEqual(boxes[0])
  })

  it('has one monitor, the window itself', async () => {
    const [monitor, ...more] = await soleWindowPort.monitors()
    expect(more).toEqual([])
    expect(monitor?.scaleFactor).toBe(1)
  })

  it('answers every request without throwing and without effect', async () => {
    const unlisten = await soleWindowPort.listen(() => {
      throw new Error('nobody can send to a window that is alone')
    })
    await soleWindowPort.broadcast({ kind: 'noop' } as never)
    await soleWindowPort.send(MainLabel, { kind: 'noop' } as never)
    await soleWindowPort.create('win-x', { x: 0, y: 0 }, { x: 1, y: 1 })
    await soleWindowPort.focus(MainLabel)
    await soleWindowPort.closeSelf()
    unlisten()
  })
})
