# The window that finally held its run, and the eleventh chunk

**2026-09-15**, branch `feature/window-20260915`. A measurement, not a feature: one solo run
played by the owner, the window around it read, and a sweep of what the machine keeps.

---

## 1. The run

Eden, solo, Normal mode in the sense the owner reported it (**see §6 — which sense is open**).
`log.txt` holds one seed line, `[Continue, 1]`, and one `Level::Init m_Stage 12` — so the surviving
log is the *last* launch of a run resumed from an earlier one, and it covers The Void alone.

Steam's own `logs\gameprocess_log.txt` supplies the clock the log does not have: `isaac-ng.exe`
ran **23:37:36 → 00:43:01** on the 14th/15th and **20:46:34 → 21:04:08** on the 15th. The run
started in the first and finished in the second.

The window is the dated backup of the 14th (23:59, written *during* the first session) against the
live save of the 15th (21:04, identical to that day's backup). **A day wide, and not tight around
the run** — `live_probe` was not running. What licenses reading it as one run is that
`STREAK_COUNTER` rose by exactly one.

## 2. What moved

```
Achievements     [256] 0->1  [458] 0->1
Counters         MOM_KILLS 118->119, EDEN_TOKENS 67->68, STREAK_COUNTER 2->3,
                 DELIRIUM_KILLS 19->20, CHARACTER_LAST_RUN_WIN 513->512,
                 MOTHER_KILLS 5->6, cell 432 0->3, + 14 activity counters
                 DEATHS and BEST_STREAK did not move
LevelCounters    stages 3,4,5,6,7,8 and 12
Items            unchanged
Unknown5         unchanged
Bosses           unchanged
Challenges       unchanged
CutsceneCounters [2] 119->120  [22] 19->20
Unknown9         [1] 0->1
Bestiary         tally 1 +1105, tally 2 +937, tally 3 +30 (and 4 new keys), tally 4 +0
                 trailing word 45146 -> 45344
```

**Three independent facts name the same cell**, which is the method that pinned bases 423 and 457:
`MOTHER_KILLS` +1; cell **432**, which `cell_index` answers as *row 9, column Mother* and row 9 is
*Eden*; and `CHARACTER_LAST_RUN_WIN` ending on Eden's bit alone.

## 3. What it closes

**The agreement with the game's counters — M4's last open item, open since 2026-09-12.** One win in
the log, `STREAK_COUNTER` +1; no death in the log, `DEATHS` unmoved. Three tests hold it, each
mutated to check it can go red.

**The log's achievement ids are the save's achievement slots.** The log unlocks `'458'` and `'256'`
and exactly those two slots turn on. The 2026-09-08 test says in as many words that the numbering
is unknown — "whatever the numbering, two unlocks cannot leave 642 slots untouched" — because that
window holds two unlocks and **no** slot turning on. This one holds two of each, and they are the
same two numbers. It is the direct link between the archive's two sources and it cost nothing but a
window that contains its run.

**Section 8 is `CutsceneCounters`.** The game prints `Reading chunk 8 / Cutscene Counters`, which
this repo has always refused as a name — sections 3 and 6 carried wrong labels for months on that
kind of evidence. Cell 22 rising once for the one `playing cutscene 22 (The Void)` is the
measurement, and the second point on the identity mapping after 19.

## 4. What it does not close, and one thing it refutes

**Cell 2 of section 8 is not either hypothesis.** It rose by one.

- *"Cutscene 1 under an off-by-one"* is **refuted**: identity at 19 and at 22 cannot coexist with an
  offset at 1.
- *"A count of launches"* survives — exactly one launch began inside the window — but is not
  established, because a cutscene 2 could have played in the lost session.
- And the sharpest fact against the simple reading: **cell 1 did not move**, although the surviving
  log plays `playing cutscene 1 (Intro)`. So the section is not plainly "cutscene N at index N".

The experiment is two minutes: launch the game, reach the menu, quit without playing. If cell 2
moves, it counts launches.

**Bestiary tally 4: a third flat window, and the best of the three.** Zero deaths — no `Game Over`,
`DEATHS` unmoved — and tallies 1, 2 and 3 all moved, tally 3 gaining four new keys. The instrument
demonstrably spoke and tally 4 stayed silent *with the hypothesis's own precondition satisfied*.
Three for three, and still not confirmed: that needs one deliberate death, which is the only
observation that can falsify it.

**`CHARACTER_LAST_RUN_WIN` was cleared, not added to.** 513 → 512: Eden's bit was already standing
and Isaac's went out. `CLAUDE.md` says the mask accumulates and that what clears it is unmeasured;
this is a clear happening across a single win, which is a lead and not yet a rule.

**Section 9 cell 1 moved for the first time**, 0 → 1.

## 5. What the disk keeps — the sweep

The claim that started it was made from two folders and stated as though it came from all of them.
A full read-only sweep of the machine says the claim held, and found four things that did not
come with it.

**No per-run history for single-player**, confirmed against: every Steam library and userdata
account (there is one), the install tree, `%APPDATA%`, `%LOCALAPPDATA%`, `%PROGRAMDATA%`, every
`Binding of Isaac*` folder (the no-plus Documents folder does not exist here), Steam's `appcache`
and `compatdata`. The only run-scoped state is `rep+gamestate<N>.dat`, which exists while a run does
and is gone when it ends — it is absent right now, and the log says so: *"SteamCloud could not find
or open rep+gamestate1.dat. No Game State detected."*

What it did find, none of which this project knew:

| source | what it is |
|---|---|
| `Steam\logs\gameprocess_log.txt` | 93 exact launch/exit timestamps since 2025-06-26. **The log has no clock**; this is one, for solo sessions too |
| `Steam\appcache\stats\UserGameStats_<id>_250900.bin` | the achievement-unlock **timestamp** cache, with `UserGameStatsSchema_250900.bin` giving id to name |
| `…\common\The Binding of Isaac Rebirth\savedatapath.txt` | the game writes its own save-data and mods paths there, every launch. `discovery` guesses between two spellings today |
| `userdata\<id>\250900\remotecache.vdf` | sha1 and timestamps of the synced saves: change detection without opening a `.dat` |

They are registered as **B55**, **B56** and **B57** rather than acted on.

## 6. The eleventh chunk

The game names its chunks as it reads a profile, and there are **eleven**:

```
1 Achievements   2 Counters      3 Level Counters  4 Collectibles
5 Mini Bosses    6 Bosses        7 Challenge Counters
8 Cutscene Counters              9 GameSettings
10 Special Seed Counters        11 Bestiary Counters
```

**Our parser reads ten sections.** The bestiary is the game's chunk **11**, not its tenth, and
*Special Seed Counters* is a name this repo has never had. Either the file holds an eleventh
section we stop before, or one of our ten holds two of its chunks.

A lead, and no more than that: the bestiary payload's `words[20]` is the constant **11** in every
save, which is the number the game gives that chunk; and section 10's header — `count=80`,
`f2=320` — is on record as describing nothing in the layout we decoded. Both are consistent with
section 10 being *Special Seed Counters* with the bestiary appended, and with the "one word left
over after the last tally". None of it is measured. It belongs to **B9**, which is where it is
written down.

## 7. Open, and it is the owner's to answer

Cell 432 — Mother for Eden — went **0 → 3**: both bits, from a single Mother kill. Whether that is
a measurement of bit 1 outside the Greed column depends on what "in normal" meant:

- **not Greed** — then it says nothing about difficulty and bit 1 stays unmeasured;
- **Normal difficulty, not Hard** — then one Normal win set both bits, and **bit 1 is not "hard
  mode"**, which would be the first reading of that bit outside Greed.

`graph::rules::MarkLevel` is named `Base`/`Second` after the bits precisely so that this question
can stay open without a wrong label hardening underneath it.

---

## 8. The bits, after the owner said "hard" — and the property that would not hold

The owner answered §7: the run was on **Hard**, and with it *"all the symbols of normal Eden"*.
Both halves are checkable, and the second is the one that pays.

**Eden's row, before and after** (`cargo run -q -p core-save --example row_marks -- 9 <before> <after>`):

```
MomsHeart 7   Isaac 3   Satan 7   BossRush 3   BlueBaby 3   TheLamb 2
MegaSatan 3   Greed 3   Hush 2    Delirium 3   Mother 0->3  TheBeast 2
before: bit 0 in 8 columns, bit 1 in 11
after:  bit 0 in 9 columns, bit 1 in 12
```

So the run filled the **last cell of the row**, and after it Eden carries bit 1 in **all twelve**
columns while bit 0 is in nine. The owner's "all the symbols" and the bytes agree, from two
directions that know nothing about each other.

**And the whole matrix splits cleanly by column** (`--example mark_values`, save of 2026-09-15):

| | value 1 — bit 0 alone | value 2 — bit 1 alone |
|---|---|---|
| Greed | **7 cells** | **0** |
| the other eleven columns | **0** | **29 cells** |

A perfect mirror. In Greed the second bit is Ultra Greedier — measured on 2026-09-12 — so it
implies the first, and Greed without Greedier is the ordinary case. Outside Greed the implication
runs the other way: **bit 1 stands alone and bit 0 does not**, so bit 0 is the stronger of the two.
`MarkLevel` names them `Base` (bit 0) and `Second` (bit 1) after the bit positions, and this is the
reason those names must never be read as an order.

**What it does not establish.** A Hard clear set both bits at once, which is consistent with bit 0
being "on hard" and not more than consistent: a single observation cannot separate "bit 0 is hard"
from "bit 0 is something this run also did". Two cells outside Greed hold **5** — bits 0 and 2, no
bit 1 — and no reading here explains them.

**The property was written, failed, and was deleted rather than tuned.** Asserting the mirror over
`dated_series` goes red on the **June 2025** save twice: `Isaac × The Beast = 1`, and
`Isaac × Greed = 2`. Both shapes are absent from every 2026 save. The cheap explanation is not that
2025 was a stranger year but that **the tables do not address the same cells there** — 423 and 457
were derived from the 2026 series, that era declares 641 achievements against 642, and The Beast
did not exist before Repentance+. Narrowing the test to the era that suits it would have hidden
exactly that. It is **B58** instead, and the property will be worth writing once the bases are
checked against the older era.
