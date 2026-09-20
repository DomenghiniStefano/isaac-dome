import { RoomKindView } from '@/lib/ipc/types'

// What a room looks like on the grid: the colour of its cell, and the drawing inside it.
//
// The colours are the game's own. Nothing on this screen paints *state* — no cell is done,
// unlockable or blocked — so the palette the player already knows from the minimap is free
// here, and it is the one palette they will never have to learn.
//
// The drawings are the fallback. When the game is installed the cell carries the icon cut out
// of `minimap_icons.png`; these take its place when it is not, and they are paths rather than
// characters so they stay sharp at 2rem and follow the ink of whatever fill they land on.

/**
 * Fill and ink together, never apart: a cell that got one without the other is a colour with
 * unreadable text on it. Written out per kind rather than built from the name, because
 * Tailwind reads these strings out of the source — a class assembled at runtime generates no
 * CSS at all, and the cell comes out transparent with nothing to say why.
 */
export const roomFill: Record<RoomKindView, string> = {
  [RoomKindView.Normal]:
    'bg-floor-room-normal text-floor-room-normal-foreground',
  [RoomKindView.Start]: 'bg-floor-room-start text-floor-room-start-foreground',
  [RoomKindView.Boss]: 'bg-floor-room-boss text-floor-room-boss-foreground',
  [RoomKindView.Miniboss]:
    'bg-floor-room-miniboss text-floor-room-miniboss-foreground',
  [RoomKindView.Treasure]:
    'bg-floor-room-treasure text-floor-room-treasure-foreground',
  [RoomKindView.Shop]: 'bg-floor-room-shop text-floor-room-shop-foreground',
  [RoomKindView.Arcade]:
    'bg-floor-room-arcade text-floor-room-arcade-foreground',
  [RoomKindView.Library]:
    'bg-floor-room-library text-floor-room-library-foreground',
  [RoomKindView.Curse]: 'bg-floor-room-curse text-floor-room-curse-foreground',
  [RoomKindView.Challenge]:
    'bg-floor-room-challenge text-floor-room-challenge-foreground',
  [RoomKindView.Sacrifice]:
    'bg-floor-room-sacrifice text-floor-room-sacrifice-foreground',
  [RoomKindView.Secret]:
    'bg-floor-room-secret text-floor-room-secret-foreground',
  [RoomKindView.SuperSecret]:
    'bg-floor-room-super-secret text-floor-room-super-secret-foreground',
  [RoomKindView.UltraSecret]:
    'bg-floor-room-ultra-secret text-floor-room-ultra-secret-foreground',
}

// The skull, drawn once: the miniboss is the same skull smaller, because it is the same thing
// in scale and not a second idea.
const SKULL =
  'M7.5 10.5a4.5 4.5 0 0 1 9 0v2.2a1.8 1.8 0 0 1-1.8 1.8H9.3a1.8 1.8 0 0 1-1.8-1.8z M10 10.8v.9 M14 10.8v.9 M10.2 14.5v2.6 M13.8 14.5v2.6'

// The three secrets are one question mark with one, two and three marks around it: they are a
// family, and drawing them as three unrelated signs would hide that.
const SECRET =
  'M9.3 9.1a2.8 2.8 0 1 1 4 2.6c-.9.5-1.2 1.1-1.2 2v.5 M12.1 17.4h.01'
const SUPER_SECRET = `${SECRET} M5.2 6.4h.01 M18.9 6.4h.01`
const ULTRA_SECRET = `${SUPER_SECRET} M12.1 3.6h.01`

/**
 * One path per kind on a 24x24 box, stroke only and no fill — the same drawing has to read on
 * the Normal Room's cream and on the Curse Room's violet, and a filled shape reads on one of
 * the two.
 *
 * The Normal Room draws nothing on purpose. It is the kind a floor is mostly made of, and a
 * mark in every cell would be a texture to see past rather than a thing to see.
 */
export const roomSymbol: Record<RoomKindView, string> = {
  [RoomKindView.Normal]: '',
  [RoomKindView.Start]: 'M12 5.5 19 18.5H5z',
  [RoomKindView.Boss]: SKULL,
  [RoomKindView.Miniboss]:
    'M9 12a3 3 0 0 1 6 0v1.4a1.3 1.3 0 0 1-1.3 1.3h-3.4A1.3 1.3 0 0 1 9 13.4z M10.8 12.2v.7 M13.2 12.2v.7',
  [RoomKindView.Treasure]:
    'M12 4.2l2.5 5.4 5.8.7-4.3 4 1.2 5.7-5.2-2.9-5.2 2.9 1.2-5.7-4.3-4 5.8-.7z',
  [RoomKindView.Shop]:
    'M12 3.8a8.2 8.2 0 1 0 0 16.4 8.2 8.2 0 0 0 0-16.4z M12 7.2v9.6 M14.4 9.4c0-1.1-1.1-1.7-2.4-1.7s-2.4.6-2.4 1.7 1.1 1.7 2.4 1.7 2.4.7 2.4 1.8-1.1 1.7-2.4 1.7-2.4-.6-2.4-1.7',
  [RoomKindView.Arcade]: 'M5.5 6.5h13v11h-13z M9 10h.01 M15 14h.01 M12 12h.01',
  [RoomKindView.Library]:
    'M6 5.5h8.5a2.5 2.5 0 0 1 2.5 2.5v10.5H8.5A2.5 2.5 0 0 1 6 16z M8.6 5.5v13',
  [RoomKindView.Curse]:
    'M6.5 6.5l11 11 M17.5 6.5l-11 11 M6.5 6.5h2.2 M17.5 17.5h-2.2',
  [RoomKindView.Challenge]:
    'M6 18.5L16.5 6 M8 6l10.5 12.5 M4.6 17.2l2.4 2.4 M17 19.6l2.4-2.4',
  [RoomKindView.Sacrifice]: 'M4.5 18.5l3.2-7 3.1 7 3.2-7 3.1 7 M4.5 18.5h15',
  [RoomKindView.Secret]: SECRET,
  [RoomKindView.SuperSecret]: SUPER_SECRET,
  [RoomKindView.UltraSecret]: ULTRA_SECRET,
}

/**
 * The palette, three rows of swatches rather than fourteen rows of list: the kinds that shape
 * a floor first, then the special rooms a placement rule actually names, then the rest.
 *
 * The rows are fixed, and they are the order the keys run in — reading left to right is
 * reading 1 to 9. A kind keeps its seat whether or not the game is installed, only the drawing
 * on it changes, so a palette learned on one machine is the same palette on another.
 */
export const paletteRows: readonly (readonly RoomKindView[])[] = [
  [
    RoomKindView.Normal,
    RoomKindView.Start,
    RoomKindView.Boss,
    RoomKindView.Miniboss,
    RoomKindView.Treasure,
  ],
  [
    RoomKindView.Shop,
    RoomKindView.Arcade,
    RoomKindView.Library,
    RoomKindView.Curse,
    RoomKindView.Challenge,
  ],
  [
    RoomKindView.Sacrifice,
    RoomKindView.Secret,
    RoomKindView.SuperSecret,
    RoomKindView.UltraSecret,
  ],
]

/**
 * The key that picks a kind, shown on the swatch. Ten digits and then four letters, in the
 * order the palette reads: painting a floor is fourteen choices made over and over, and a trip
 * to the mouse for each of them is the cost this removes.
 */
export const paletteKey: Record<RoomKindView, string> = {
  [RoomKindView.Normal]: '1',
  [RoomKindView.Start]: '2',
  [RoomKindView.Boss]: '3',
  [RoomKindView.Miniboss]: '4',
  [RoomKindView.Treasure]: '5',
  [RoomKindView.Shop]: '6',
  [RoomKindView.Arcade]: '7',
  [RoomKindView.Library]: '8',
  [RoomKindView.Curse]: '9',
  [RoomKindView.Challenge]: '0',
  [RoomKindView.Sacrifice]: 'Q',
  [RoomKindView.Secret]: 'W',
  [RoomKindView.SuperSecret]: 'E',
  [RoomKindView.UltraSecret]: 'R',
}

/** The key that picks no kind at all, which is how the brush becomes an eraser. */
export const ERASE_KEY = 'Backspace'

/**
 * What a key press means to the palette: a kind, `null` for the eraser, or `undefined` for a
 * key that is none of ours.
 *
 * The three answers are not two. A handler that folded "not ours" into "erase" would rub out a
 * room every time somebody typed into a field on this screen, and the day that field arrives
 * nothing here would fail.
 */
export const brushFor = (key: string): RoomKindView | null | undefined => {
  if (key === ERASE_KEY) return null
  const upper = key.toUpperCase()
  const found = Object.keys(paletteKey).find(
    (kind) => paletteKey[kind as RoomKindView] === upper,
  )
  return found === undefined ? undefined : (found as RoomKindView)
}
