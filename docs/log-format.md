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


## A launch is not a run, and the oldest log we have says so

Measured 2026-09-16 (B60). `samples/launches/20240305-rep179b-launch-no-run.log.txt` is a whole
`log.txt` in which **nobody started a run**: the game launched, played `cutscene 1 (Intro)`, and
shut down. Of the ten patterns in `crates/run/rules/events.json`, **exactly one line matches** —
line 69, the cutscene — and the fold produces no run at all, because an ending with no run open
belongs to no run.

It is not a rare shape. It is the first launch of most evenings, and it is the reason
`samples/logs/` means *logs of runs* and this file lives in `samples/launches/` instead: the guard
over the first folder requires a run from every log in it, and a runless one filed beside them
would turn that guard into one that cannot fail.

**And it is the only log in this repo from before the `+`** — Repentance **v1.7.9b**, March 2024,
where everything else here is Repentance+ of 2026. Whether the rules read an older version's lines
the same way had never been measured. **They do**: the same prefix, the same cutscene line, the
same single match, counted with `grep -cE` on the file before any test asserted it. One log is not
a survey of two years of patches, but it is no longer nothing.
