import { Dlc } from '@/lib/ipc/types'

// The editions' names as the game prints them. Game names stay in English in both
// languages, like item names, so they're data rather than messages.
export const dlcNames: Record<Dlc, string> = {
  [Dlc.Rebirth]: 'Rebirth',
  [Dlc.Afterbirth]: 'Afterbirth',
  [Dlc.AfterbirthPlus]: 'Afterbirth+',
  [Dlc.Repentance]: 'Repentance',
  [Dlc.RepentancePlus]: 'Repentance+',
}
