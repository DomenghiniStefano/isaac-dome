// The value sets the UI defines over the wire's fields, for the screens that share them.
//
// They lived in Unlock's filter module, and the Collection imported them from there: a screen
// depending on another screen for a value that belongs to neither. Both are *ours* rather than
// the wire's — each adds the case the wire says with `null` — so they sit beside the generated
// types instead of inside them.

import { ItemKindView, OriginView } from './types'

// What a row is about, by kind. "Nothing" is a value of its own: 231 nodes on the reference
// profile unlock nothing the catalog knows, and that is something to filter on. The first four
// are `ItemKindView` itself, bound to it rather than retyped, which is why the Collection can
// read an item's kind against this set: a variant renamed in Rust breaks the build here.
export const TargetKind = {
  Passive: ItemKindView.Passive,
  Active: ItemKindView.Active,
  Familiar: ItemKindView.Familiar,
  Trinket: ItemKindView.Trinket,
  Character: 'character',
  Boss: 'boss',
  Challenge: 'challenge',
  Nothing: 'nothing',
} as const
export type TargetKind = (typeof TargetKind)[keyof typeof TargetKind]

// The origin DLC as the catalog infers it, plus the rows it can't say for: the wire writes that
// case as `null`, and a facet needs a value to offer.
export const OriginValue = {
  Rebirth: OriginView.Rebirth,
  Afterbirth: OriginView.Afterbirth,
  AfterbirthPlus: OriginView.AfterbirthPlus,
  Repentance: OriginView.Repentance,
  None: 'none',
} as const
export type OriginValue = (typeof OriginValue)[keyof typeof OriginValue]

export const originOrder: OriginValue[] = [
  OriginValue.Rebirth,
  OriginValue.Afterbirth,
  OriginValue.AfterbirthPlus,
  OriginValue.Repentance,
  OriginValue.None,
]
