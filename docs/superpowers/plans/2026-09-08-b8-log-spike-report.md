# B8 — What a real `log.txt` actually contains

**Spike, closed 2026-09-08.** The backlog entry asked for a document and no code kept. The
document is this; the code was kept anyway, on purpose and as two `core-save` examples,
because both turned out to be instruments rather than scaffolding: `live_probe.rs` answers
questions that need a *running* game, and `dump_sections.rs` prints the real section table,
which B9 has to re-measure. Neither is a test and neither ships.

Evidence in `samples/logs/` (git-ignored): the complete log of one run,
`20260908-run-megasatan-judas.log.txt` (33,250 lines, 1.57 MB), and three probe reports,
`probe-1.tsv` to `probe-3.tsv`.

**Method.** `core-save/examples/live_probe.rs` watched `log.txt` and both
`.dat` files every 500 ms while a run was actually being played, reporting each save change
through `core_save::diff`. The run: Judas, hard mode, seed `YKF6 QDN6`, Basement to
Catacombs to Dank Depths to Utero to Scarred Womb to Sheol to Dark Room and **Mega Satan**,
won. Two achievements unlocked during the observation, which is what made the timing
questions answerable at all.

---

## The two questions the backlog said only a real log could answer

**1. Does the log record rooms?** **Yes.** Every transition is logged with a frame number
and the destination's identity:

```
[Frame 449] Starting room transition (type 0)
Room 4.21()
Room 1.1075(New Room)
Room 1.2(Start Room)
```

343 transitions in this run, 170 distinct `New Room` entries. Anything map-shaped is
therefore possible. It also logs `Spawn Entity with Type(N), Variant(N), Pos(x,y)` — 768 of
them — and `Entity Pickup Initialized`, neither of which the five documented patterns
mention.

**2. What is the flush delay?** **None worth designing around.** The file grew every
0.5-2 s during play, in bursts of 40 to 1,200 bytes, measured continuously for over 45
minutes. A Live screen can be live.

---

## Five things nobody was looking for

### a. The log announces every save write

```
Saving PersistentGameData to Steam Cloud: rep+persistentgamedata1.dat.
```

**132 times in a single run.** The game tells us, in words, exactly when it wrote the save.
A watcher does not need to poll the file or diff it speculatively: it can read the line and
then read the save. This is a better trigger than `notify` on the directory, because it
also says *which* file.

### b. The save is an event stream, not a between-sessions snapshot

`docs/PROJECT.md` assumes the `.dat` is what you read between sessions. It isn't: the game
rewrites it every few seconds *during* a run, one or two counters at a time. Observed live,
via `core_save::diff`, with the counter labels from `reference/isaac_counters.py`:

```
POOP_DESTROYED              [5]   10994 -> 10997
ROCKS_DESTROYED             [2]   24168 -> 24172
SECRET_ROOMS_WALLS_OPENED   [199]  1874 -> 1876
BLOOD_DONATION_MACHINE_USED [17]    133 -> 136
```

So a slice of what M4 wanted from `log-watch` — "something changed, refresh" — is available
from a format we control and already parse, instead of from text patterns in a versioned
rule file. **The limit belongs beside the finding:** these are *lifetime* counters. They say
what you have done, never how this run went. Run structure stays the log's job.

### c. The log leads, the save follows

Achievement 208 was announced by the log the moment it was earned and did **not** appear in
the save until the run closed, minutes later. Achievement 19, earlier in the same run,
reached the save within seconds. The two sources are not synchronous, and the gap is not
fixed.

**The design consequence:** the log for responsiveness, the save for truth. A UI that shows
an unlock from the log is right to do so, and must not read the save's silence as a
contradiction.

### d. `from pool X` is not a provenance you can trust

The character's starting item is emitted with exactly the shape of a real pickup, and with a
pool that did not produce it:

```
333  RNG Start Seed: YKF6 QDN6 (2913253616) [New, 1]
354  Room 1.2(Start Room)
358  Adding collectible 34 (The Book of Belial) ... from pool treasure
364  [Frame 74] Starting room transition (type 0)
```

Judas *starts* with Book of Belial; nobody picked it up. You cannot filter by pool, because
the same run has legitimate `treasure` pickups. **The only discriminator is position: an
`Adding collectible` between `RNG Start Seed` and the run's first `Starting room
transition` is the character's starting item, whatever pool it claims.** The rule holds for
the hard cases too — Eden, who starts with random items, and the Tainted, who start with an
active *and* a passive, produce several lines inside that same window.

Cost of not knowing: a tracker reporting "items found in treasure rooms" over-counts by one
in *every* run, and no test notices, because the data is perfectly well-formed.

### e. Reconstructing an inventory needs the catalog

`Adding collectible` is a stream of pickups, not an inventory. Passives and familiars
accumulate; **actives replace one another**. This run picked up five actives — Book of
Belial, Anarchist Cookbook, Book of Revelations, The Bible, Book of Revelations again — and
held exactly one at the end. Summing the lines gives a player holding five books and four
Ball of Bandages, instead of one active and one Super Bandage Girl.

The missing piece already exists: `catalog` knows the kind of all 909 items. But this is a
rule the live tracker needs *inside* it, not a presentation detail.

---

## The unplanned finding: the game names the save's sections

Loading a profile, the log prints this, once per chunk, in file order:

```
Reading chunk 1   Reading Achievements
Reading chunk 2   Reading Counters
Reading chunk 3   Reading Level Counters
Reading chunk 4   Reading Collectibles
Reading chunk 5   Reading Mini Bosses
Reading chunk 6   Reading Bosses
Reading chunk 7   Reading Challenge Counters
Reading chunk 8   Reading Cutscene Counters
Reading chunk 9   Reading GameSettings
Reading chunk 10  Reading Special Seed Counters
Reading chunk 11  Reading Bestiary Counters
```

**The four sections `CLAUDE.md` lists as "to be identified" have names**, and two sections
we thought we knew are contradicted. The positions we are certain of — 1, 2, 4 and 7 — all
agree, which is what makes the rest credible:

| kind | our label | count x on-disk | the game's name |
|---|---|---|---|
| 1 | achievements | 642 x 1 | Achievements (agrees) |
| 2 | counters | 523 x 4 | Counters (agrees) |
| 3 | `PerChar`, "to be identified" | 14 x 4 | **Level Counters** |
| 4 | items | 733 x 1 | Collectibles (agrees) |
| 5 | `Unknown5` | 7 x 1 | **Mini Bosses** |
| 6 | `CardsPills` | 104 x 1 | **Bosses** — conflict |
| 7 | challenges | 46 x 1 | Challenge Counters (agrees) |
| 8 | `Unknown8` | 27 x 4 | **Cutscene Counters** |
| 9 | `Unknown9` | 2 x 4 | **GameSettings** |
| 10 | `Bestiary` | header says 80, parser gets 11,016 bytes | **Special Seed + Bestiary Counters** |

**Why eleven chunks and ten headers.** Measured with `Save::parse` on
`samples/20260908.rep+persistentgamedata1.dat`: the file carries ten headers, kinds 1..10,
and the parse reports **zero diagnostics** — the bytes are fully accounted for. But section
10's header declares `count=80, f2=320` while the parser hands it **11,016 bytes**, i.e.
everything up to the checksum. A scan of that tail finds no further header in the usual
three-`u32` form. So section 10's payload holds *both* of the game's last two chunks, with
no header between them: the game counts eleven, the file has ten, and both are right.

**Blast radius, measured.** `Kind::CardsPills` is referenced only in `diff.rs`
(`SaveDiff.cards_pills`, a public field name) and in two count assertions in
`core-save/tests/real_saves.rs`; `Kind::PerChar` only in the tests. Neither crosses the IPC
nor drives product behaviour. **This is a semantics bug, not a data one** — but
`cards_pills` is a wrong name in a public type, and 104 entries against the catalog's 103
bosses fits "Bosses" considerably better than "cards and pills".

**Not closed here, on purpose.** Renaming the sections and splitting section 10 is its own
task, on a table that has to be re-derived from content rather than swapped on the strength
of a log line. Logged as backlog entry B9.

---

## Cross-checks the run handed us for free

Three confirmations of things the project had only inferred from its own code:

- **`unlock steam achievement '19'`** — the log names the unlock with *our* number. The
  section-1 mapping `slot[id]` had been pinned by cross-referencing 169 of 171 items; now
  the game states it outright. Same again for 208.
- **`MARK/Mega Satan/Judas [119] 0 -> 3`** — index 119 falls in the "Mega Satan" block
  starting at 116, and `119 - 116 = 3` is Judas in that block's character order. The block
  came from REPENTOGON via `reference/isaac_counters.py` and had never been verified by
  *doing the thing and watching the right cell light up*. It has been now.
- **The mark's two bits are written together on hard mode.** `CLAUDE.md` records bits 0 and
  1 as the mark's two levels, with the third unexplained. The run was hard mode and the cell
  went `0 -> 3` in a single write, so hard grants both levels at once. Measured, not assumed.

## One thing that does not fit

`CHARACTER_LAST_RUN_WIN [188] 4 -> 12`, on a winning Judas run. Judas is character id 3, and
the previous value 4 matches no character played. Either REPENTOGON's label for index 188
means something other than a character id, or it encodes something else entirely. Recorded
as an observed discrepancy, to be closed by collecting wins with other characters.

## Noise, and what it costs

**87% of the log is animation warnings** — 29,089 lines out of 33,222, all of the form
`[INFO] - [warn] no animation named FloatShootSide`. Note the doubled severity: a `[warn]`
payload inside an `[INFO]` envelope, so a parser keying on `[INFO]` alone selects nothing
useful. There was **no third-party mod noise at all** on this machine (zero `Lua Debug`
lines), unlike the 2024 log the backlog entry was written from — so mod noise is a real case
but not a constant, and the rule file has to handle its absence as cheerfully as its
presence.

The other high-volume kinds, for sizing a parser: `Spawn Entity` (768), `Entity Pickup
Initialized` (572), `Lua mem usage` (355), `SpawnRNG seed` (353), `Starting room transition`
(343), `Room N.N(...)` (264 across its three forms), `TriggerBossDeath` (149), `Saving
PersistentGameData` (132).

## What this unblocks

M4's design can start. The live item tracker is a yes. The explored map is a yes, because
rooms are logged. The secret-room finder remains a different product, as the backlog entry
says, and nothing here changes that.

**One correction to the backlog entry's own framing.** It says the loss window is "the app
must run at least once between one session and the next". That still holds, and this run
adds a reason it matters less than feared: the save is written continuously during play, so
even a session whose log is lost leaves its lifetime counters behind. What is lost with the
log is the run's *shape* — seed, floors, rooms, items with their pools, the death — never
the fact that you played.
