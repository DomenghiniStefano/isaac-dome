import { describe, expect, it } from 'vitest'
import { TargetView } from '@/lib/ipc/types'
import { targetPage } from './targets'

const targets = Object.values(TargetView)

describe('targetPage', () => {
  it('has a page for every target, which is the whole reason it exists', () => {
    for (const target of targets) expect(targetPage[target]).toBeTruthy()
  })

  it('sends each one somewhere of its own', () => {
    const pages = targets.map((target) => targetPage[target])
    expect(new Set(pages).size).toBe(pages.length)
  })

  it('points at the same wiki the rules were read from, over https', () => {
    for (const target of targets)
      expect(targetPage[target]).toMatch(
        /^https:\/\/bindingofisaacrebirth\.wiki\.gg\/wiki\//,
      )
  })
})
