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

> **Section 7: challenge `n` is cell `n`, and cell 0 is unused** — measured 2026-09-17, so the
> 46 cells are `challenges.xml`'s 45 challenges plus an index nobody fills, the same shape as
> section 3's stages. The instrument is the catalog's own reward link: finishing a challenge
> grants a known set of achievements, so "the cell is set" and "every reward achievement is done"
> have to agree. On `20260915` they agree on **39 of 39** judged rows — **21 of them both true
> and 18 both false**, which is what makes the agreement a result and not an empty row of
> zeroes — while the off-by-one reading breaks 13 of the same 39.
> `cargo run -p ipc --example probe_challenges` is the instrument.

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
structural half of that reading, and it is pinned by
`the_online_bit_never_stands_without_the_first_level_bit`
in `crates/ipc/tests/marks_real.rs`. The two **levels** are the unmeasured half **except in
one column**: in **Greed**, bit 1 is **Ultra Greedier**, measured on 2026-09-12 on three days
with three different characters (Keeper, Judas, Magdalene), each time the right character's
cell.

**Outside Greed there is now exactly one observation, and it says "hard".** On 2026-09-20
Mother was beaten with T. Eden **on hard**, on a virgin cell, and `[449]` went **0 → 3** — bits
0 and 1 together, in one event. That is what `ipc` already bet on: B22 counts `hard` as a
*subset* of `normal` (`bits & 2` inside `bits & 3`), on the reasoning that a mark taken on hard
was taken at all. The bet is now corroborated instead of merely allowed.

It also dissolves the apparent tension with Greed, where the same bit was seen *replacing*
rather than adding (`1 → 2`, B58). A **mode** is alternative to the mode below it, so Greedier
overwrites Greed; a **difficulty** is not alternative to the run, so hard lights both. Bit 1 is
"the second level of that column", and what that level *is* differs per column.

**One observation does not name a bit**, so `graph::rules::MarkLevel` stays `Base` / `Second`
after the bits and not `Hard` after a meaning. What closes it is the opposite discriminator:
**a win on normal, on a cell still at zero**. A `1` there confirms the reading; a `3` refutes it
and sends the search elsewhere. The difficulty is **not in the log** — checked on
2026-09-20 across the whole file, the game never writes it, not even in the seed line — so it
has to be asked of the player while the run is fresh, or it is lost. The
property is kept by `winning_greedier_sets_the_second_bit_of_that_characters_greed_cell` in
`crates/ipc/tests/progress_real.rs`. Still **don't compute completion percentages**: what
forbids them now is B22/B23, not an unread bit.

> **A cell's bits are not latched flags, and the reason given above until 2026-09-17 was
> wrong.** It read "an online clear is also a clear, so bit 2 never stands without bit 0",
> which treats bit 0 as *the* cleared bit and makes a bare 2 a contradiction. The 638-era
> series says otherwise: four located cells go **1 → 2** across it — `Isaac × Greed` and
> `Cain × Greed` on 2024-02-23, `Isaac × TheLamb` and `BlueBaby × BossRush` on 2024-01-29 —
> which is bit 0 going *out* as bit 1 comes in. A value replaces the one before it rather
> than accumulating, which is also what makes `1 → 2` in Greed the same event as "Ultra
> Greedier implies Greed" rather than its refutation. So a cell holding 2 is an ordinary
> cell and **never** evidence that the tables address the wrong thing — that reading is what
> raised B58 — and the absence of 4 and 6 is an observation held over every sample, worth
> pinning as such, not a law derived from what the bits mean.

**The matrix is 34 × 12** since 2026-09-08: the last three columns were located on the
historical series, not read off a document. Delirium for the 19 later characters starts at
**404** (not the 386 the pattern predicted), Mother for the 14 originals at **423**, The
Beast at **457**, and **491**/**492** are those two bosses' kills. Each base is pinned by
three independent facts on the day a cell changed: an achievement whose wiki requirement is
that boss, the kill counter rising by exactly as many as the new marks, and index **188** —
a bitmask of the characters that won the run, which names the row. Two properties in
`crates/ipc/tests/marks_real.rs` keep the tables answerable to the series.

**Index 188 is indexed by the game's character id, not by our row order**, measured
2026-09-20 and not knowable before. The 34-row order in `ipc::marks::CHARACTERS` puts T. Eden
at row **26**; the bit that lit when T. Eden won was **30**, which is what
`crates/graph/rules/requirements.json` calls that character (`{"kind":"character","id":30}`).
The two readings were indistinguishable while every measured window was an *original*
character, because there rows and ids coincide — Magdalene is row 1 and id 1. It took a
Tainted character to separate them, and any future use of 188 has to map through the id.

**Mother's group of 34 closed on 2026-09-20**, on the window this section had been asking for:
T. Eden beat Mother and `[449]` was the **only** cell to move in all of 423..=490. Row 26 is
`+11` into the 19-block, so the block starts at **438** and the single cell left over in
437..=456 is The Forgotten's, at **437**. All three facts hold and none of them is loose:
achievement **567**, whose requirement in the rules file is literally *Mother* + *Tainted
Eden*; `[491] 6 → 7`, up by exactly the one new mark; and 188 as above. One character was
enough where Delirium needed four, because this block is bracketed on **both** sides by
measured bases — 423..=436 below, 457 above — with no slack for it to sit anywhere else.
`mothers_group_of_34_tiles_from_its_own_base_to_the_beasts` in
`crates/core-save/tests/marks_layout.rs` is that arithmetic, and it needs no sample.

**Those bases hold in the 638 era too, measured 2026-09-17** (B58), and that is what says
they hold in the 641 era nobody can walk. Section 2 is **496** cells in the 638 era, **521**
in the 641 and **523** in the 642, so it only ever grew, and a base derived on the newest era
says nothing on its own about the ones before it. The 638-era series re-derives both by the
same method, on a different profile:

| window | mark cells | that boss's kills |
|---|---|---|
| 2024-02-08 → 02-15 | `[423] 0→2`, Mother + Isaac | `[491] 0→1` |
| 2024-02-23 → 03-05 | `[457] 0→1`, `[466] 0→2`, Beast + Isaac and + Eden | `[492] 0→2` |
| 2024-03-05 → 06-06 | `[461] 0→2`, Beast + Blue Baby | `[492] 2→3` |

Two of the three are exact, with no slack. So 423 and 457 are measured at **both ends** and
the era between them is **bracketed**: a cell inserted before 423 by one patch and removed by
the next is not how patches work. It is an inference from two measured eras and not a third
measurement, which is the most a machine holding one 641-era snapshot can say.

**What the kill counters cannot see is an off-by-one, and this is where that was measured.**
They compare *counts*: with Mother's base moved to 422, the cell that lights on 2024-02-15
is read as Magdalene's instead of Isaac's, the counter still rises by one, and every
real-data property stays green. Only index 188 separates them, and it needs a window where
one character won and one mark appeared — the 638 series offers exactly one, an Azazel window
that says nothing about Mother. What catches 422 is arithmetic on the tables themselves:
Delirium's 19-block runs 404..=422, so a Mother base one lower makes two cells answer the same
index. `no_two_cells_of_the_matrix_share_an_index` and
`the_three_derived_blocks_tile_against_their_neighbours` in `crates/core-save/tests/marks_layout.rs`
are that check, and they need no sample at all — which matters, because `samples/` is
per-machine and a table's arithmetic is not.

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

Still open: documented names reach 284; **20 cells** — The Beast for The Forgotten and the
19, at 471–490 by spacing — are zero in every save collected, so they stay `Unknown` rather
than pointing at a guess. **It was 40 until 2026-09-20**, when a window on T. Eden beating
Mother closed the other half; what closes this one is the same run against The Beast. The
symmetry with Mother's group says where each of the 20 sits, but an inference from one worked
example is not a window, and a derived index must not be told from a measured one only by
reading the git log. Index 385 is a counter on its own, 386–403 are eighteen cells never seen
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

**Tally 1 is what you met and tally 2 is what you killed** — measured the same evening over six
windows, each with a **counted** number of a **named** entity, the name coming from the log's
`Spawn Entity with Type(n), Variant(v)` lines and the count from the player. The instrument B9
asked for was *kill a known enemy a known number of times*; the half it never had is **a known
number left alive**, and that is what separates the two.

| window | met | killed | tally 1 | tally 2 |
|---|---|---|---|---|
| Clotty, died on purpose | 2 | 0 | +2 | +0 |
| Ministro, died on purpose | 3 | 0 | +3 | +0 |
| Grilled Clotty, died on purpose | 3 | 0 | +3 | +0 |
| Round Worm | 3 | 3 | +3 | +3 |
| Skinny, one of them a champion | 4 | 4 | +4 | +4 |
| Round Worm + Stoney, **one worm left alive, no Stoney touched** | 3 + 2 | 2 + 0 | +3 / +2 | +2 / — |

Six for six. The three death windows are the ones that read wrong until the last: dying on purpose
without killing anything moves tally 1 alone, which had looked like *"1 counts kills and 2 is
asleep"*.

**A champion counts on the entity's own key.** The log spawned all four Skinnies as `226.0.0` and
the file counted four on one row, so the colour is not in the key.

**The totals contradict the behaviour, and that is not resolved.** Across the whole save, tally 2
is larger than tally 1 on **213** shared keys, smaller on **132** and equal on **91** — and *met*
can never be fewer than *killed*. So the six windows describe today's rule and the accumulated
numbers do not obey it. The economical explanation is that one of the two changed meaning in a
patch; **it is a hypothesis, and the practical consequence is not**: the totals may not be used to
reason about what the tallies mean, only their movements may.

**Tally 3 is the one still unnamed.** It moved `+5` on Clotty and `+2` on Ministro, and **zero in
the other four windows**, including every one with kills in it. It is smaller than tally 1 on
**308** of the 311 keys they share. Nothing collected so far tells what it counts, and it must not
be given a name from a guess.

Same for the **one word left over**
after the last tally, present in every save and growing (11,343 → 29,725 across the samples) —
and since 2026-09-16 known to move **+8 on a launch with no run at all**, which is most of the
+11 once attributed to a whole Greed run. `Save::bestiary_tallies()` hands both back;
`docs/STATUS.md` lists what closing them needs.

