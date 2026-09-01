#!/usr/bin/env python3
"""
Etichette della sezione "counters" (523 valori u32) del salvataggio di Repentance+.

Fonte: enum EventCounter documentato da REPENTOGON (indici 0-284), verificato
contro i salvataggi reali. Gli indici oltre il 213 seguono uno schema regolare
- per ogni boss, un blocco di 19 celle: Bethany, Jacob&Esau e i 17 personaggi
Tainted - ricostruito qui e marcato come derivato, non documentato.

Le celle PROGRESSION_* non sono contatori ma maschere di bit:
    bit 0 (1) e bit 1 (2) = i due livelli del marchio di completamento
    bit 2 (4) = terzo livello, significato non ancora confermato
Distribuzione osservata su un profilo reale: 0, 1, 2, 3, 5, 7. Nessun altro valore.
"""

# --- personaggi, nell'ordine usato dai blocchi del salvataggio ---
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

# --- indici verificati (REPENTOGON) ---
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

# --- blocchi da 14 celle: i personaggi originali ---
BLOCKS_14 = [
    ("Mom's Heart", 27), ("Isaac", 41), ("Satan", 55), ("Boss Rush", 69),
    ("Blue Baby", 83), ("The Lamb", 97), ("Mega Satan", 116), ("Greed", 130),
    ("Hush", 144), ("Delirium", 173),
]

# --- celle singole di The Forgotten, aggiunto dopo i 14 ---
FORGOTTEN = {
    203: "Mom's Heart", 204: "Isaac", 205: "Satan", 206: "Boss Rush",
    207: "Blue Baby", 208: "The Lamb", 209: "Mega Satan", 210: "Greed",
    211: "Hush", 213: "Delirium",
}

# --- blocchi da 19 celle: Bethany, Jacob & Esau e i Tainted (derivato) ---
BLOCKS_19 = [
    ("Mom's Heart", 214), ("Isaac", 233), ("Satan", 252), ("Boss Rush", 271),
    ("Blue Baby", 290), ("The Lamb", 309), ("Mega Satan", 328), ("Greed", 347),
    ("Hush", 366),
    # Delirium e i boss di Repentance (Mother, The Beast) stanno da 386 in poi,
    # ma l'allineamento non e' ancora confermato: l'indice 385 e' un contatore
    # a se' (vale 49, fuori dall'intervallo di una maschera) e da li' la
    # regolarita' dei blocchi si interrompe. Da chiudere con altri salvataggi.
]

# Indici ancora senza etichetta: 385, e la coda 404-522, dove convivono
# almeno due famiglie diverse (valori 0-7 tipici delle maschere accanto a
# valori grandi tipo 53, 34, 13, che sembrano contatori per personaggio).
UNRESOLVED = [385] + list(range(404, 523))

MARK_BITS = {1: "livello 1", 2: "livello 2", 4: "livello 3"}


def label(index):
    """Nome dell'indice, piu' un flag: True se verificato, False se derivato."""
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
    """Matrice dei marchi: {boss: {personaggio: maschera}}."""
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
    print(f"\nmarchi almeno iniziati: {tot} su {len(bosses) * len(chars)}")
