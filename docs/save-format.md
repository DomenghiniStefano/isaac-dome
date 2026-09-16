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
0x10   u32                  sometimes zero, meaning unknown   # NOT "changes on every save"
0x14   first section header
end-4  checksum             CRC32 with a custom polynomial, NOT identified (irrelevant)
```

### `0x10` is zero more often than a hash could be (measured 2026-09-16)

The line above used to read *"changes on every save"*, and so did `reference/isaac_save.py`, the
implementation everything here was translated from — both corrected together, because a claim left
in the source anyone translates from is the one that comes back. It does not hold on the Jan–Jun
2024 series:

| | |
|---|---|
| saves read | 16 — the 15 `rep_` of the 2024 series, plus `20250112` (`rep+`) |
| `0x10 == 0` | **4** — `20240119`, `20240215`, `20240223`, and `20250112` |
| pairs whose payload is identical | **0** — every consecutive pair differs, in length or in bytes |

So zero is **not** "this save changed nothing", which was the obvious explanation and is the one
the data refuses: the payload moved across all fifteen pairs, and three of them wrote a zero
anyway. Nor is it a hash-like value that happened to land on zero — four times in sixteen is not
2⁻³² four times over. **Zero is a state the field takes**, and what puts it there is unknown; it
stays `unknown_0x10` in `parse.rs`, which reads it raw and validates nothing.

Two things this touches, neither of them a defect today:

- **The untouched slot.** `…persistentgamedata2.dat` is byte-identical across all fourteen 2024
  backups — a save slot nobody ever played — and its `0x10` is zero too. It is the only **empty
  profile** this project has seen, and nothing in `samples/` covers that shape: every sample is a
  profile with progress, while the app has to open at a stranger's house on the day they install
  the game.
- **`live_probe`'s stock sentence.** When bytes move and no decoded field does, the example says
  `0x10` "is the candidate" — resting on the claim this section just removed. The trailing
  checksum at `end-4` changes on every write and is the nearer explanation; the comment now says
  both.

**What this does not say.** The claim it corrects came from the M0 spike's 28 `rep+` saves of
2025–2026, which are not on this machine — it may well have held on all of them, and one `od` at
offset 16 over that series says whether zero is a 2024 shape or a general one. Until then this is
a measurement on the series that was read, which is exactly as far as it goes.

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

#### The untouched profile is the clean view of that gap (measured 2026-09-16)

A save slot the game created and **nobody ever played** — `samples/empty/`, byte-identical across
fourteen 2024 backups, B61 — turns out to be an instrument rather than a fixture, for one reason:
**everything that is not structure is zero**, so there is nothing to read past.

Nine of the ten sections hold no set byte at all. The bestiary does, and it is the only section
whose payload length moves — **128 bytes here against 7032** in the played profile of the same day,
while its header declares `count = 80` on **both**. So the bestiary's `count` is not the number of
entities recorded; the payload is self-describing and the header counts something else, which is
the second of the two coincidences above stated as a measurement.

Of those 128 bytes, the first 80 are zero and the **first non-zero word is `words[20]`, and it is
`11`** — the constant the paragraph above names, here with no entity data anywhere near it. Eleven
more words follow it, 48 bytes in all:

```
words[20..32] = 11, 0, 4, 4, 0, 2, 0, 3, 0, 1, 0, 5
```

**Nothing here is named**, and the rule about guessing applies with full force: this is what the
bytes are, not what they mean. What changed is that the question now has a clean instrument, which
is what B9 was missing. `crates/core-save/tests/empty_profile.rs` pins the zeros and the position
of the constant, so a future decoding starts from a measurement and a change in that region is
loud.

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

**Index 188 is cleared when a run is lost**, measured 2026-09-16 on a deliberate death:
`268435968` → `0`, which is `0x10000100` going to nothing. So it is the *current* run's
winners and not a cumulative record, and reading it after a loss finds an empty mask rather
than the last win's. In the same window **`STREAK_COUNTER [22]` reset from 4 to 0** — the
second point on that counter, whose first was +1 for a win on 2026-09-15, and the one that
says it is a streak and not a total.

**The two streak counters need two consecutive losses to be told apart**, measured the same
evening on a second deliberate death. A single loss shows only half of it:

| | first loss | second loss |
|---|---|---|
| `STREAK_COUNTER [22]` | **4 → 0** | unchanged at 0 |
| `NEGATIVE_STREAK_COUNTER [113]` | unchanged at 0 | **0 → 1** |

So the first loss *breaks* the positive streak without opening the negative one, and the second
opens it. Both predictions made before the second run — that 22 and 188 would stay at 0 — held,
which is what licenses reading 113's move as the new fact rather than as one of theirs.

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

**Tally 4 counts the deaths an entity caused you** — measured 2026-09-16, and it is the first
of the four to earn a name. The owner died once, on purpose, in the Basement. Across the whole
save **three keys moved and all three are the same entity**, `15.0.0` = Clotty:

```
tally 1  Clotty  1090 -> 1092  (+2)
tally 3  Clotty     2 -> 7     (+5)
tally 4  Clotty     2 -> 3     (+1)   <- exactly one, on the killer, and on no other key
```

**Confirmed the same evening by a second death to a different enemy**, which is the window that
rules out the weak reading *"it is Clotty's row that moves"*. Judas, killed by a Ministro
(`305.0.0`), and tally 4 gains a **new key** rather than incrementing an old one:

```
tally 1  Ministro  577 -> 580  (+3)
tally 3  Ministro    6 -> 8    (+2)
tally 4  Ministro    0 -> 1    (+1)   <- a new record; Clotty stayed at 3
```

Tally 4 went 159 records to 160, which is *an entity that had never killed you now has, once* —
and it is the growth pattern the whole hypothesis rested on (130/240 on 06-29, 142/277 on 08-05,
154/319 on 09-08: slowly, and by whole new keys).

The hypothesis had stood since 2026-09-09 against three windows in which tally 4 did not move —
and all three had **zero deaths in them**, so the instrument had never been handed the thing it
reacts to. The first window that did produced exactly the predicted number. `DEATHS [10]` rose
269 → 270 in the same window, which is what says the death reached the file at all.

**Tallies 1 and 2 part company here for the first time**: 1 gained 2 and **2 gained nothing**,
in a window where enemies were killed. The reading recorded until then was that *"1 and 2 move
constantly and in both modes"* — true of every window collected, because every one of them had
both moving. One short run ending in a death is the case that separates them; what 2 is waits
on another.

Read together on one entity the window says 5, 2 and 1 for tallies 3, 1 and 4 — the shape B9
always asked for, *kill a known enemy a known number of times and read which moves by how much*,
which needed a **named** killer to be worth anything.

**Two entities are enough to kill every reading in which 1 and 3 bound each other:**

| | tally 1 | tally 3 |
|---|---|---|
| Clotty | +2 | **+5** |
| Ministro | **+3** | +2 |

Neither is ever the larger, so they are not *seen* against *killed* in either direction — nothing
can be killed more often than it was met. Any pair of names implying an order is out, on two rows
that took four minutes to collect.

**The other three tallies still have no names**, and must not be given one from a guess: they
hold the same entities with different numbers against each. Same for the **one word left over**
after the last tally, present in every save and growing (11,343 → 29,725 across the samples) —
and since 2026-09-16 known to move **+8 on a launch with no run at all**, which is most of the
+11 once attributed to a whole Greed run. `Save::bestiary_tallies()` hands both back;
`docs/STATUS.md` lists what closing them needs.

