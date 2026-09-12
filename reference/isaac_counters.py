#!/usr/bin/env python3
"""
Labels for the "counters" section (523 u32 values) of the Repentance+ save file.

Source: EventCounter enum documented by REPENTOGON (indices 0-284), verified
against real saves. Indices past 213 follow a regular pattern - for each boss, a
block of 19 cells: Bethany, Jacob & Esau, and the 17 Tainted characters -
reconstructed here and marked as derived, not documented.

Updated 2026-09-08: the Delirium block for the 19, and the Mother and The Beast
blocks for the 14 originals, were located on the historical series (35 saves).
Method, for each cell: the day it changed, cross-checked three ways - the boss
(an achievement whose wiki requirement is that boss unlocked the same day), the
count (491 and 492 are Mother's and The Beast's kills, and they rose by exactly
as many as the new marks) and the character (188 is a bitmask of the characters
that won the run; it read Magdalene for base+1 and Cain for base+2).

The PROGRESSION_* cells aren't counters but bit masks:
    bit 0 (1) and bit 1 (2) = the two levels of the completion mark
    bit 2 (4) = third level, meaning not yet confirmed
Distribution observed on a real profile: 0, 1, 2, 3, 5, 7. No other value.
"""

# --- characters, in the order used by the save file's blocks ---
CHARS_14 = [
    "Isaac", "Magdalene", "Cain", "Judas", "Blue Baby", "Eve", "Samson",
    "Azazel", "Lazarus", "Eden", "The Lost", "Lilith", "Keeper", "Apollyon",
]
CHARS_19 = [
    "Bethany", "Jacob & Esau",
    "T. Isaac", "T. Magdalene", "T. Cain", "T. Judas", "T. Blue Baby", "T. Eve",
    "T. Samson", "T. Azazel", "T. Lazarus", "T. Eden", "T. The Lost", "T. Lilith",
    "T. Keeper", "T. Apollyon", "T. Forgotten", "T. Bethany", "T. Jacob & Esau",
]

# --- verified indices (REPENTOGON) ---
NAMED = {
    0: "NULL", 1: "MOM_KILLS", 2: "ROCKS_DESTROYED", 3: "TINTED_ROCKS_DESTROYED",
    4: "SUPER_SPECIAL_ROCKS_DESTROYED", 5: "POOP_DESTROYED", 6: "PILLS_EATEN",
    7: "XIII_DEATH_CARD_USED", 8: "UNKNOWN_EVENT_8", 9: "ARCADES_ENTERED",
    10: "DEATHS", 11: "ISAAC_KILLS", 12: "SHOPKEEPER_KILLED", 13: "SATAN_KILLS",
    14: "SHELLGAMES_PLAYED", 15: "ANGEL_DEALS_TAKEN", 16: "DEVIL_DEALS_TAKEN",
    17: "BLOOD_DONATION_MACHINE_USED", 18: "SLOT_MACHINES_BROKEN",
    19: "UNKNOWN_EVENT_19", 20: "DONATION_MACHINE_COUNTER", 21: "EDEN_TOKENS",
    22: "STREAK_COUNTER", 23: "BEST_STREAK", 24: "BLUE_BABY_KILLS",
    25: "LAMB_KILLS", 26: "MEGA_SATAN_KILLS",
    111: "BOSSRUSHS_CLEARED", 112: "UNKNOWN_EVENT_112",
    113: "NEGATIVE_STREAK_COUNTER", 114: "EDENS_BLESSINGS_NEXT_RUN",
    115: "GREED_DONATION_MACHINE_COUNTER",
    158: "HUSH_KILLS", 187: "DELIRIUM_KILLS", 188: "CHARACTER_LAST_RUN_WIN",
    189: "UNKNOWN_EVENT_189", 190: "DAILYS_PLAYED", 191: "DAILY_RUN_IS_ACTIVE",
    192: "DAILYS_STREAK", 193: "DAILYS_WON", 194: "RAINBOW_POOP_DESTROYED",
    195: "BATTERIES_COLLECTED", 196: "CARDS_USED", 197: "SHOP_ITEMS_BOUGHT",
    198: "CHESTS_OPENED_WITH_KEY", 199: "SECRET_ROOMS_WALLS_OPENED",
    200: "BLOOD_CLOT_ITEM_AQUIRED", 201: "RUBBER_CEMENT_ITEM_AQUIRED",
    202: "BEDS_USED", 212: "GREED_COINS_DONATED_FORGOTTEN",
    # Located on the series, 2026-09-08: they sit right after the mark blocks of
    # the two bosses Repentance added, in the same order as the blocks.
    491: "MOTHER_KILLS", 492: "BEAST_KILLS",
}

# --- 14-cell blocks: the original characters ---
#
# Greed (base 130), 2026-09-12: bit 1 of a cell in this column is Ultra Greedier, not
# merely "the harder level". Measured on three days of the dated series, each with a
# different character and each time that character's own cell: Keeper (142, 0 -> 3),
# Judas (133, 1 -> 3), Magdalene (131, 0 -> 3). The claim is about this column only.
BLOCKS_14 = [
    ("Mom's Heart", 27), ("Isaac", 41), ("Satan", 55), ("Boss Rush", 69),
    ("Blue Baby", 83), ("The Lamb", 97), ("Mega Satan", 116), ("Greed", 130),
    ("Hush", 144), ("Delirium", 173), ("Mother", 423), ("The Beast", 457),
]

# --- single cells for The Forgotten, added after the 14 ---
FORGOTTEN = {
    203: "Mom's Heart", 204: "Isaac", 205: "Satan", 206: "Boss Rush",
    207: "Blue Baby", 208: "The Lamb", 209: "Mega Satan", 210: "Greed",
    211: "Hush", 213: "Delirium",
}

# --- 19-cell blocks: Bethany, Jacob & Esau, and the Tainted (derived) ---
BLOCKS_19 = [
    ("Mom's Heart", 214), ("Isaac", 233), ("Satan", 252), ("Boss Rush", 271),
    ("Blue Baby", 290), ("The Lamb", 309), ("Mega Satan", 328), ("Greed", 347),
    ("Hush", 366),
    # Not 385 as the regular pattern predicted: the block starts at 404, and four
    # characters agree on it - Bethany (+0), Jacob & Esau (+1), T. Cain (+4),
    # T. Azazel (+9), each on a day DELIRIUM_KILLS also rose.
    ("Delirium", 404),
    # Mother and The Beast for the 19 later characters, and for The Forgotten, are
    # inside 423-490 by the spacing (the two 14-blocks are exactly 34 = 14+1+19
    # apart), but every candidate cell is zero in every save collected so far, so
    # nothing distinguishes one layout from another. Left out on purpose.
]

# Indices still without a label:
#   385      a counter on its own, value 49, unchanged across the whole series
#   386-403  eighteen cells, zero in every save, family unknown
#   437-456  Mother for The Forgotten and the 19: located by spacing, unverified
#   471-490  the same for The Beast
#   493-522  a family of counters that move together, several per session
UNRESOLVED = (
    [385] + list(range(386, 404)) + list(range(437, 457))
    + list(range(471, 491)) + list(range(493, 523))
)

MARK_BITS = {1: "level 1", 2: "level 2", 4: "level 3"}


def label(index):
    """Name of the index, plus a flag: True if verified, False if derived."""
    if index in NAMED:
        return NAMED[index], True
    if index in FORGOTTEN:
        return f"MARK/{FORGOTTEN[index]}/The Forgotten", True
    for boss, base in BLOCKS_14:
        if base <= index < base + 14:
            return f"MARK/{boss}/{CHARS_14[index - base]}", True
    for boss, base in BLOCKS_19:
        if base <= index < base + 19:
            return f"MARK/{boss}/{CHARS_19[index - base]}", False
    return f"UNKNOWN_{index}", False


def marks(counters):
    """Mark matrix: {boss: {character: mask}}."""
    out = {}
    for boss, base in BLOCKS_14:
        out.setdefault(boss, {})
        for i, ch in enumerate(CHARS_14):
            out[boss][ch] = counters[base + i]
    for idx, boss in FORGOTTEN.items():
        out.setdefault(boss, {})["The Forgotten"] = counters[idx]
    for boss, base in BLOCKS_19:
        out.setdefault(boss, {})
        for i, ch in enumerate(CHARS_19):
            out[boss][ch] = counters[base + i]
    return out


if __name__ == "__main__":
    import sys
    from isaac_save import Save

    c = Save(sys.argv[1]).u32s("counters")
    m = marks(c)
    bosses = [b for b, _ in BLOCKS_14]
    chars = CHARS_14 + ["The Forgotten"] + CHARS_19

    width = max(len(x) for x in chars) + 1
    print(" " * width + "".join(f"{b[:9]:>11s}" for b in bosses))
    for ch in chars:
        row = "".join(f"{m[b].get(ch, 0):>11d}" for b in bosses)
        print(f"{ch:<{width}s}{row}")

    tot = sum(1 for b in bosses for ch in chars if m[b].get(ch, 0))
    print(f"\nmarks at least started: {tot} out of {len(bosses) * len(chars)}")
