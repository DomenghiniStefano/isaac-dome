# The save file — what has been decoded, and how

The `.dat` format as measured, not as documented: nobody publishes this. Verified against 41 real
Repentance+ saves spread over fourteen months, and the reference implementation to translate from
is `reference/isaac_save.py`.

**This left `CLAUDE.md` on 2026-09-16.** It is 116 lines of measured fact that most sessions never
touch, and `CLAUDE.md` is loaded into every one of them. What stayed there is the rule this page
exists to serve — *the entry count is read from the file, never hardcoded* — with a pointer here.

## Save file format (decoded and verified)

Verified against 28 real Repentance+ saves spread over 14 months.
Reference implementation in `reference/isaac_save.py` — translate from there.

```
0x00   "ISAACNGSAVE09R  "   signature, 16 bytes   # existing tools look for 06R and fail
0x10   u32                  changes on every save, meaning unknown
0x14   first section header
end-4  checksum             CRC32 with a custom polynomial, NOT identified (irrelevant)
```

Section header: three little-endian `u32`s — `kind` (sequential 1..10), `f2` (= count × 4,
the "in-memory" size), `count`. Then the data: `count` entries, whose **on-disk** size
depends on the section.

| kind | count | bytes/entry | content |
|---|---|---|---|
| 1 | 642 | 1 | achievements and secrets |
| 2 | 523 | 4 | game counters **and completion marks** |
| 3 | 14 | 4 | level counters, one cell per stage — index 0 is unused |
| 4 | 733 | 1 | item collection |
| 5 | 7 | 1 | to be identified |
| 6 | 104 | 1 | bosses met |
| 7 | 46 | 1 | challenges |
| 8 | 27 | 4 | cutscene counters — **measured** 2026-09-15 |
| 9 | 2 | 4 | to be identified |
| 10 | variable | 8 | bestiary: four tallies over the same entities, self-describing |

> **The entry count is read from the file, NEVER hardcoded.** The June 2025 save declares
> 641 achievements, the 2026 ones declare 642: a patch added one. Any hardcoded count
> breaks itself.

> **A section's name is either structural or measured, never taken from a log line.**
> Sections 3 and 6 were renamed on 2026-09-09 (B9): they read `PerChar` and `CardsPills`,
> labels from watching bits flip, and both were wrong — 3 is stages and 6 is bosses. The
> game prints its own names when it loads a profile (5 is "Mini Bosses", 8 "Cutscene
> Counters", 9 "GameSettings"), which is strong evidence and **not** a measurement: those
> stay `Unknown` until the bytes are checked.
>
> **8 was checked on 2026-09-15 and is `CutsceneCounters`**: one solo run's log plays
> `playing cutscene 22 (The Void)` once and cell 22 rises by exactly one, the second point on
> the identity mapping after 19. 5 and 9 still have only the printed name. What the name does
> **not** settle: cell 2 also rose by one with no cutscene 2 in the surviving log, and cell 1
> did not move although the Intro played — so it is not plainly "cutscene N at index N".

> **The game reads ELEVEN chunks and we parse TEN sections**, and nothing yet explains the
> gap. Loading a profile it prints, in order: Achievements, Counters, Level Counters,
> Collectibles, Mini Bosses, Bosses, Challenge Counters, Cutscene Counters, GameSettings,
> **Special Seed Counters**, Bestiary Counters. So the bestiary is its chunk **11**, and
> chunk 10 carries a name this repo has never had. Either the file holds a section we stop
> before, or one of our ten holds two of its chunks — a lead with two coincidences behind it
> (the bestiary payload's constant `words[20]` is **11**, and section 10's header describes
> nothing in the layout we decoded). Nothing here is measured; it lives in B9.

### Counters and marks

Section 2 maps one-to-one to REPENTOGON's `EventCounter` enum. Labels in
`reference/isaac_counters.py`. The `PROGRESSION_*` cells aren't counters but **bitmasks**:
observed values are only 0, 1, 2, 3, 5, 7. Bits 0 and 1 are the mark's levels; **bit 2 is
"won online"**, measured on 2026-09-12 (see below). That the set holds no 4 and no 6 is the
structural half of that reading — an online clear is also a clear, so bit 2 never stands
without bit 0 — and it is pinned by `the_online_bit_never_stands_without_the_cleared_bit`
in `crates/ipc/tests/marks_real.rs`. The two **levels** are the unmeasured half **except in
one column**: in **Greed**, bit 1 is **Ultra Greedier**, measured on 2026-09-12 on three days
with three different characters (Keeper, Judas, Magdalene), each time the right character's
cell. What bit 1 means in the other eleven is still unmeasured, and `graph::rules::MarkLevel`
is therefore named `Base` / `Second` after the bits and not `Hard` after a meaning. The
property is kept by `winning_greedier_sets_the_second_bit_of_that_characters_greed_cell` in
`crates/ipc/tests/progress_real.rs`. Still **don't compute completion percentages**: what
forbids them now is B22/B23, not an unread bit.

**The matrix is 34 × 12** since 2026-09-08: the last three columns were located on the
historical series, not read off a document. Delirium for the 19 later characters starts at
**404** (not the 386 the pattern predicted), Mother for the 14 originals at **423**, The
Beast at **457**, and **491**/**492** are those two bosses' kills. Each base is pinned by
three independent facts on the day a cell changed: an achievement whose wiki requirement is
that boss, the kill counter rising by exactly as many as the new marks, and index **188** —
a bitmask of the characters that won the run, which names the row. Two properties in
`crates/ipc/tests/marks_real.rs` keep the tables answerable to the series.

Still open: documented names reach 284; **40 cells** (Mother and The Beast for The Forgotten
and the 19) sit inside 423–490 by spacing but are zero in every save collected, so they stay
`Unknown` rather than pointing at a guess — one run of Mother with a Tainted character
closes them. Index 385 is a counter on its own, 386–403 are eighteen cells never seen
moving, and 493–522 is a family of counters that move several per session.

### The bestiary (section 10)

Unlike every other section, it **describes itself** — read the declarations, don't assume
the shape. Measured 2026-09-09 on four saves across two editions; the header's `count=80` /
`f2=320` describe nothing in this layout and are ignored.

```
words[0..19]   twenty zeros
words[20]      11            constant in every save
words[21]      the total, exactly the sum of the four sizes below
words[22]      4             how many tallies follow
then 4 x ( id, size, size/4 records of (key, count) )
                             ids 4, 2, 3, 1 in that order
```

A size is in units of two bytes, a record is eight: `size / 4` records. Inside a tally the
keys are **strictly ascending, each entity once**; the key is
`(type << 20) | (variant << 8) | subtype`, the same triple `crates/wiki` indexes bosses by.
Read it as one list instead and you get three descents, 445 repeated keys and a dangling
word — all three are artefacts of ignoring the boundaries, and all three have a plausible
wrong explanation ready.

**The four tallies have no names**, and must not be given one from a guess: they hold the
same entities with different numbers against each, so they are four counts of one space and
telling them apart needs a matched window against a live run. Same for the **one word left
over** after the last tally, present in every save and growing (11,343 → 29,725 across the
samples). `Save::bestiary_tallies()` hands both back; `docs/STATUS.md` lists what closing
them needs.

