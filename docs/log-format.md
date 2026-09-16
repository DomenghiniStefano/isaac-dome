# `log.txt` — what the game writes while you play

Rewritten on every launch, so an untracked run is lost forever: the app has to be running while
you play. **This left `CLAUDE.md` on 2026-09-16**; the rule that stayed is that no pattern may be
anchored at the start of a line.

## log.txt

Rewritten on every game launch: untracked runs are lost forever, so the app has to be
running while you play. Verified lines:

```
[INFO] - Adding collectible 225 (Gimpy) to player 0 (Cain) from pool treasure
[INFO] - RNG Start Seed: FYQ8 QQ8G (586324166) [New, 1]
[INFO] - Level::Init m_Stage 2, m_StageType 1 Seed 408474304
[INFO] - Game Over. Killed by (9.0) spawned by (84.0) damage flags (0)
[INFO] - playing cutscene 15 (Sheol).
[INFO] - [Frame 74] Starting room transition (type 0)
```

**The `[INFO] - ` prefix is part of the line**, and until 2026-09-13 this block dropped it —
so did the M4 spec's table. Some lines carry a frame marker on top of it, and about 1% carry
no prefix at all (the `Framebuffer Width:` block, the library banners). **No pattern may be
anchored at the start of a line**, or it matches nothing at all.

One line gives the item's id and name, the character, and the pool; the death line gives
the killing entity and what spawned it. The patterns live in `crates/run/rules/events.json`,
embedded at build time and updatable without recompiling.

**The seed line has three kinds and they are not interchangeable**: `[New, …]`,
`[Continue, …]` — a run resumed from an earlier launch, logged with the seed it already had —
and `[Net, …]`, an online run. Reading a `Continue` as a fresh start abandons a run still being
played and counts it twice, so `run`'s fold decides by **seed**, not by label. `Net` is the only
free discriminator we have for co-op.


## How a floor was built, and the one mode that never says

Measured 2026-09-16 on the six logs in `samples/logs/` (F2). A floor's generation is a block, and
the summary line is the one worth reading:

```
[INFO] - Level::Init m_Stage 4, m_StageType 4 Seed 2649606411
[INFO] - delete 0 generated rooms.
[INFO] - generate...
[INFO] - place_room: shape 12
[INFO] - 19 rooms in 12 loops
[INFO] - placing rooms...
```

- **The summary belongs to the `Level::Init` above it**, and `Level::Init` is the only thing that
  says where one floor ends and the next begins.
- **`place_room: shape N` is not a room list.** There are four to six per floor, against a total
  of nineteen: they are the rooms that are not one cell.
- **`Level::Init`, `generate...`, the summary and `placing rooms...` come in equal numbers** in
  five of the six logs — 11, 10, 10, 4 and 1 — so the game never left a pass without its summary
  in anything we hold.
- **Greed mode describes no generation at all**: seven floors, seven `delete N generated rooms.`,
  and **zero** `generate...` or summary lines. It is not the online that silences it — the other
  `[Net]` log describes all eleven of its floors. A Greed floor has **no number**, which is not
  the same as a floor of no rooms.
- **A floor can be generated more than once**, and exactly one floor in the whole corpus is:
  `m_Stage 4, m_StageType 4` — Mines II, the only floor we hold that has an area of its own. Its
  two passes report **the same 19 rooms** in 12 and 14 loops, so nothing here can say which of
  them is the floor that was walked: `run` keeps both and picks neither.
- **Beware the memory-pool lines**, which say `allocate 1357 rooms.` and `delete 644 rooms.`
  dozens of times a run. A pattern loose enough to catch them reports a floor of 1357 rooms, and
  makes Greed — which generates nothing this way — the loudest mode in the archive.

**Still unmeasured, and the screen depends on it**: whether the count includes the Secret and
Super Secret rooms, which the game places in the `placing rooms...` phase *after* the summary.
The Floor screen exists to find the secret room, so "the game generated 19, you painted 15" means
two different things depending on the answer. Nothing in the log settles it; painting one floor
to exhaustion in the game does.

## A launch is not a run, and the oldest log we have says so

Measured 2026-09-16 (B60). `samples/launches/20240305-rep179b-launch-no-run.log.txt` is a whole
`log.txt` in which **nobody started a run**: the game launched, played `cutscene 1 (Intro)`, and
shut down. Of the ten patterns in `crates/run/rules/events.json`, **exactly one line matches** —
line 69, the cutscene — and the fold produces no run at all, because an ending with no run open
belongs to no run.

> **There are eleven patterns since 2026-09-16**, and this count was not re-run against the
> eleventh: `samples/launches/` is not on the machine that added `roomsGenerated`, so
> `the_launch_of_20240305_holds_one_event_and_it_is_the_intro` skipped there. The test is the
> instrument and it will speak on the machine that holds the file. A launch with no run announces
> no floor, so a generation summary in it would be a finding — not a number to fold in quietly.

It is not a rare shape. It is the first launch of most evenings, and it is the reason
`samples/logs/` means *logs of runs* and this file lives in `samples/launches/` instead: the guard
over the first folder requires a run from every log in it, and a runless one filed beside them
would turn that guard into one that cannot fail.

**And it is the only log in this repo from before the `+`** — Repentance **v1.7.9b**, March 2024,
where everything else here is Repentance+ of 2026. Whether the rules read an older version's lines
the same way had never been measured. **They do**: the same prefix, the same cutscene line, the
same single match, counted with `grep -cE` on the file before any test asserted it. One log is not
a survey of two years of patches, but it is no longer nothing.
