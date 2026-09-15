# Where the files are — the paths seen on real machines

What `discovery` has to cover, measured on real installs rather than assumed. **This left
`CLAUDE.md` on 2026-09-16.**

## Real-world paths

`discovery` has to cover at least these cases, all seen on real machines:

```
Documents\My Games\Binding of Isaac Repentance\        # without the +
Documents\My Games\Binding of Isaac Repentance+\       # with the +
Steam\userdata\<id>\250900\remote\rep_persistentgamedata<n>.dat
Steam\userdata\<id>\250900\remote\rep+persistentgamedata<n>.dat
```

With Steam Cloud active, the save is **not** in the Documents folder. What stays there is
`log.txt`, `options.ini`, and two useful subfolders: `save_backups\` (dated backups
created by the game: a free historical series) and `online_logs\` (one folder per online
co-op session, with the full log and two profile snapshots).

**`discovery` finds that folder on its own** since 2026-09-13, and until then it could not:
it walked `My Games` only to look for `.dat` files, so the folder surfaced only inside
`SaveSource::Documents { folder }` — that is, only when a save happened to live there, which
with Steam Cloud on it never does. `Discovery::game_data` names the three things in it and
never crosses the IPC: they are paths.

**`online_logs\` is not flat, and reading it as flat finds nothing.** Measured on 2026-09-13:

```
online_logs\sessions\MM_DD_YYYY__HH_MM_SS\      the session: log.txt, persistentgamedata1_{begin,end}.dat,
                                                sharedsave_{begin,end}.dat
online_logs\desyncs\MM_DD_YYYY__HH_MM_SS__Name\ a crash report — desync_log.txt, not a run log
online_logs\desyncs\sessions\MM_DD_YYYY__HH_MM_SS\   two more real sessions, nested here
```

A walk of the first level finds two directories and no sessions; one that takes any folder
under `desyncs\` files crash reports as runs. **28 sessions** on this machine today, not the
22 the M4 spec counted, and a session's log is `<folder>\log.txt`. The folder's name carries a
wall clock, which the log itself does not have.

Those two snapshots are **not** the same profile before and after, whatever the names say.
Measured on 2026-09-08: `persistentgamedata1_end.dat` is the local profile, counter for
counter identical to the dated backup of the same day, while
`persistentgamedata1_begin.dat` is the **other participant's** — a second real profile,
growing from 52 to 105 achievements across 21 sessions. That makes the folder the only
place with a profile at the *start* of the progression, which is where a table with many
zeros can actually be tested. `sharedsave_*.dat` sits in the same folder and is **not** in
our format: no `ISAACNGSAVE` magic anywhere in the file.

Note: online co-op uses a **separate shared profile** that grows with the group, distinct
from the personal one. It's a second progression the game shows nowhere.

**It does not follow that co-op is invisible in the personal save, and until 2026-09-12
this document said it was.** That claim — "a run ending in co-op leaves the personal
counters untouched" — was measured wrong: it read "no achievement moved" as "nothing
moved". A matched window around one online Greed run (Cain, won, the whole session inside
the window) says the personal save splits in two:

| moves in an online run | stays put |
|---|---|
| section 2's activity counters (20 of them) | achievements, items, challenges, bosses |
| the **completion mark**, with bit 2 set | sections 3, 5, 8 and 9 |
| bestiary tallies **1 and 2** | bestiary tallies **3 and 4** |

Section 8 staying put through a logged `playing cutscene 21` is the same trap in its
original form: the log announces a cutscene the personal save never records. **Read a
co-op session as evidence about counters, marks and the bestiary; never about unlocks.**

Local co-op (second controller) is a third case, and it is **not** settled. No day without
an online session has ever produced a bit 2, and the owner confirms local co-op happened at
least once, so the likely reading is that local co-op does not set it — by elimination, not
by measurement. What would settle it is one local co-op win with a snapshot either side.

**Do not try to identify those days from index 188.** That mask **accumulates**: a single
win adds its character's bit and leaves the previous one standing (`2 → 6` on 2026-09-01,
`4 → 12` in the B8 spike report), so two bits at once is two wins in the window, not two
players in one run. Something clears it — `6 → 4`, `4 → 0` are both observed — and what,
nobody has measured. This is why
`the_character_that_won_is_the_character_whose_mark_appeared` only reads windows with
exactly one bit: that restriction is load-bearing, not caution.

---

