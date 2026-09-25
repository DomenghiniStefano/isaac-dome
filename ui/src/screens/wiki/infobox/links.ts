import type { InjectionKey } from 'vue'
import type { Infobox, Target } from '@/lib/ipc/types'

// What every row of a page's card needs to draw a reference and follow it: the two resolvers
// the page holds, and the way back up to it. Provided once by `WikiInfobox`, so the body of each
// kind does not hand the same three things to every row it draws.
export interface InfoboxLinks {
  readonly iconFor?: (target: Target) => string | null
  readonly canOpen?: (target: Target) => boolean
  navigate: (target: Target, newTab: boolean) => void
}

export const infoboxLinksKey: InjectionKey<InfoboxLinks> =
  Symbol('infoboxLinks')

// The infobox of one kind: what the body drawn for that kind receives.
export type InfoboxOf<Kind extends Infobox['kind']> = Extract<
  Infobox,
  { kind: Kind }
>
