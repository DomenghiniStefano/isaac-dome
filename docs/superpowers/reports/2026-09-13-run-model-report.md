# M4 sub-project 1a — the run model — report

**Branch** `feature/run-model`, cut from `develop` on 2026-09-13 after
`feature/generated-contract` merged. Spec
`docs/superpowers/specs/2026-09-12-m4-run-model-design.md`, plan
`docs/superpowers/plans/archive/2026-09-13-run-model.md`. **41 tests in `run`**, 9 in
`test-support`; `scripts/check` green on the result.

**What it does.** `crates/run` turns the lines of `log.txt` into typed events and folds them
into runs — what was played, with what, and how it ended. Pure: no disk, no clock, no database,
no screen. `Tail` turns bytes into complete lines and reads a shorter file as a relaunch;
`rules/events.json` maps a line to one of nine events and judges nothing; the fold makes every
judgment.

**What it does not do.** Everything in 1b: the watcher, the store's fourth migration, run
identity and the `(source, ordinal)` key, the backfill of `online_logs\`, the agreement with the
save's own counters — which needs a save beside the archive, and `run` never sees one — and
every view-model.

---

## 1. Why the sub-project split

The spec covers a pure crate, a watcher, a migration, a backfill, an agreement with the save,
and the view-models. Writing one plan for that produces a document nobody executes in a
session. 1a is finished software on its own: a crate with a tested contract that 1b calls.

The split was made when the plan was written, not when the work ran long.

## 2. Three corrections to a spec that was one day old

All three came from opening the files rather than re-reading the notes.

| | |
|---|---|
| the store migration | the **fourth**, not the third: the window session took the third on 2026-09-13 |
| the log lines | every one carries `[INFO] - `, some also `[Frame 74] `; the spec's table and `CLAUDE.md` dropped both |
| the sample logs | `Game Over` is in **none** of the four, so `Died` has no real-data coverage at all |

The second is the one that would have cost a morning: a pattern anchored at the start of a line
matches nothing, and the symptom is an empty archive rather than an error.

The third is the one that matters most. The spec said the three logs "cover every outcome except
`Died`, which needs one more log", which reads as a small gap. `grep -c "Game Over"` returns 0 on
all four files. `no_sample_log_contains_a_death` asserts that absence and **fails the day a log
with a death arrives** — a suite that passed quietly would have read as coverage that is not
there.

## 3. The finding the spec had no line for

**The seed line labels itself, and there are three labels.** Measured across the four logs:
`[New, 1]`, `[Continue, 1]`, `[Net, 1]`.

A `Continue` is a run resumed from an earlier launch, and the game logs it **with the seed the
run already had**. The spec's rule for the one inferred outcome — a `RunStarted` arriving while
the previous run has neither a death nor an ending means `Abandoned` — would therefore mark a run
that is still being played as abandoned, and then count it twice. On the logs we have this is not
hypothetical: `20260912-solo-judas.log.txt` opens with a `Continue`.

So the fold decides by **seed**, not by label: the same seed on a run that is still open is that
run resumed. The label is kept and typed anyway, because `Net` is the only free discriminator we
have for an online run — and §3 of the spec leaves open whether co-op counts toward
`STREAK_COUNTER [22]`. That question is 1b's, and the data it needs is now carried.

`SeedKind::Unknown` keeps a word we have never met rather than dropping the run around it.

## 4. A test found a bug in the guard that fixes that

The first version of the resume rule read **any** repeated seed as a resumption. A seed can be
replayed deliberately, and the run that used it may already have ended — so the rule swallowed
the second run. `a_replayed_seed_after_the_run_ended_is_a_second_run` was written before the
code and failed against it; the fix is the `Outcome::Open` half of the condition, which is
load-bearing and says so in a comment.

## 5. Where the judgments live, and what each costs to get wrong

The rules file maps text to events and makes no decision, because it is data a user can edit:
a rule that could decide meaning would put untestable logic outside the tested crate. Every
judgment is in the fold, and each has a test that names its cost:

- **The starting item.** `from pool treasure` is a lie on Judas's Book of Belial, and the line
  is shaped exactly like a real pickup. Position is the only discriminator — an item added
  before the first room transition is the character's own. Getting it wrong over-counts
  treasure-room finds by one in every run, with well-formed data and nothing failing. Asserted
  on the real line in `judas_starts_with_the_book_of_belial_and_it_is_not_a_treasure_find`.
- **The inventory.** Actives replace one another; the run B8 watched picked up five and ended
  holding one.
- **The character.** Not in the seed line — it arrives with the first item, and a run with no
  item line keeps it `None` rather than guessing.
- **Events before the first seed belong to no run.** The intro cutscene sits at line 77 of the
  megasatan log and the seed at line 333; read into a run, it would be that run's ending.

## 6. What the real logs confirmed

Every count pinned in `logs_real.rs` was measured with `grep -c` first and matched what the
patterns produce: 1 seed line, 10 floors, 34 collectibles, 343 room transitions, 2 achievements,
132 save writes. A pinned number is a fixture of an era and the era is in the file name; the
comment says how each was derived, so a disagreement later is a question about which side moved.

## 7. What 1b inherits

- `Tail::advance` and `Tail::restarted` — the watcher supplies bytes and a length, nothing else.
- `Rules::parse` returning `RulesError`, so a user's bad file is reported and the embedded file
  takes over. `Rules::version()` is what `store` keeps beside a derived run.
- `Run::fold(events, kinds)`, with `ItemKinds` to be implemented over the real catalog in `ipc`.
- `SeedKind::Net`, for the co-op question the agreement with `STREAK_COUNTER [22]` runs into.
