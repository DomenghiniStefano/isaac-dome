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

