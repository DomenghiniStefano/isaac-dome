#!/usr/bin/env python3
"""
Labels for the "counters" section (523 u32 values) of the Repentance+ save file.

Source: EventCounter enum documented by REPENTOGON (indices 0-284), verified
against real saves. Indices past 213 follow a regular pattern - for each boss, a
block of 19 cells: Bethany, Jacob & Esau, and the 17 Tainted characters -
reconstructed here and marked as derived, not documented.

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
}

# --- 14-cell blocks: the original characters ---
BLOCKS_14 = [
    ("Mom's Heart", 27), ("Isaac", 41), ("Satan", 55), ("Boss Rush", 69),
    ("Blue Baby", 83), ("The Lamb", 97), ("Mega Satan", 116), ("Greed", 130),
    ("Hush", 144), ("Delirium", 173),
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
    # Delirium and the Repentance bosses (Mother, The Beast) sit at 386 onward,
    # but the alignment isn't confirmed yet: index 385 is a counter on its own
    # (it's 49, outside a mask's range), and from there the blocks' regularity
    # breaks down. To be closed with more saves.
]

# Indices still without a label: 385, and the 404-522 tail, where at least
# two different families coexist (values 0-7 typical of masks next to
# large values like 53, 34, 13, which look like per-character counters).
UNRESOLVED = [385] + list(range(404, 523))

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
