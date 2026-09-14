# M4, sub-project 2 — the two screens that draw the archive

**Status:** three decisions taken in conversation on 2026-09-14, plus one requirement added
mid-answer ("filtrabili"). Everything past §1 follows from them and was not walked through
one by one — it is the part most worth disagreeing with.

**Why now.** Sub-project 1 deliberately left both screens out: *"`Live` and `Runs` are both
views of this model, and designing them first would let a layout shape the IPC contract."*
The contract exists and is generated since N7, the archive fills itself from `online_logs\`
and follows `log.txt`, and `Run` and `Live` are still the shell's two placeholders. The
reason for the exclusion has expired.

---

## 1. The three decisions, as taken

**Run is a diary, not a dashboard.** One row per run, in order: the character, how it ended,
how many floors, the seed. Expanded, the items it collected. The screen answers *what
happened*, not *am I improving* — the second question was offered and declined, and it is
worth writing down why the offer was weak: without a clock in the log and without the floor a
death happened on, a "bilancio" could only have grouped by killer and character, which is a
thinner answer than it sounds.

**Live shows the run in progress and what finishing it would open.** The bigger of the two
options, and the one that makes the screen worth a route: the archive alone can say what you
are holding, the graph can say what it is worth. §3 is where this gets difficult, and the
difficulty is not the join — it is the character.

**Abandoned runs stay in the list, marked.** The archive keeps them by decision; a run you
walked away from is still a run you played. `RunOutcomeView::Abandoned` already exists and
`Open` is not a failure — §4 of sub-project 1's spec says it must never be drawn as one.

**And the list is filterable** — said in one word, mid-answer, and it lands on work already
done: N3 unified the faceted engine (`ui/src/lib/facets/faceting.ts`) precisely so a third
list could use it without copying Unlock's. This is that third list, before B3's.

## 2. Run — what a row carries

Everything here exists on the wire today (`ipc::RunView`), which is the test of whether this
screen can be built without touching the contract:

| column | from | note |
|---|---|---|
| character | `character: Option<String>` | `None` when no item line ever named it — the seed line does not |
| outcome | `RunOutcomeView` | `Won { ending }`, `Died { killer }`, `Abandoned`, `Open` |
| floors | `floors: u32` | how many, not which: the archive counts them |
| seed | `seed_words` | the two words the game prints, which is how a player names a run |
| online | `online: bool` | the only free discriminator for co-op |
| source | `RunSource` | `Live`, or the session folder's wall clock (`09_12_2026__13_34_26`) |

Expanded: `starting_items`, `collected`, `held_active` — each an id with a name when the
catalog is there, and an id alone when it is not, which the row must survive rather than hide.
`achievements` is the ids the log announced during the run.

**Order.** A run is `(source, ordinal)` and there is no timestamp on the wire. The session
folder's name carries a wall clock; `Live` does not. So the list is ordered newest source
first, and inside a source by ordinal descending — and the spec says this out loud because
"sort by date" is the obvious thing to ask for and the data cannot do it.

**Facets** (the shared engine, pure functions over the rows): outcome, character, online,
source. Search over the seed and the character. Nothing counted server-side: `RunsView`
carries the rows, the facets are the frontend's.

## 3. Live — and the character problem, which is the whole design

The run in progress is the archive's `Open` run on the `Live` source: character, floor count,
starting items, what it has collected, the active it holds. That half is drawing what exists.

The other half — *what finishing this run would open* — is a join with the unlock graph, and
the graph is ready for it: `RequirementView::Mark { character, character_name, column, level }`
exists since 2026-09-12 and says "beat this column with this character".

**The join is by character, and the archive has only a name.** `crates/run/rules/events.json`
captures `player (?<player>\d+) \((?<character>[^)]*)\)` — and `player` is the **slot**, 0 or 1
in co-op, not the character's id. The name is the game's own, and **the game gives a Tainted
character the base form's name**: this repo already paid for that once, when 141 of 396
character references resolved to nothing and "Ultra Greedier as Keeper" picked Tainted Keeper
(2026-09-12, fixed by resolving through the wiki's id instead of the name).

So Live cannot say *which* Cain you are playing, and three answers are possible:

1. **Say both, and say that it is both.** "Se finisci: Cain — o Tainted Cain" with the two
   sets of marks. Honest, ugly, and never wrong.
2. **Resolve by starting items.** Tainted Cain starts with Bag of Crafting; the fold already
   keeps `starting_items`. This is an **inference**, and inferences that name things are what
   this repo spends its corrections on — it needs a measurement over real logs before it can
   be believed, not a plausible rule.
3. **Ask the log again.** Unmeasured: whether any line in a real Tainted run says so. One run
   with a Tainted character, with the log kept, answers it.

**This spec chooses 1, and registers 3 as the measurement that would replace it.** Option 2 is
explicitly refused as a shipped guess, and may return with data behind it.

**What Live shows when the profile is not readable or the graph is absent**: the run, and
nothing about unlocks — the diagnostics `RunsView` already carries say why, and the screen
must not read as "this run opens nothing".

## 4. What crosses the IPC, and what does not

**Run needs nothing new.** That is the point of §2's table.

**Live needs one thing**: the marks a character is still missing, for the character the run
names. The shape is *not* decided here — the two candidates are a new field on `RunsView`
(the backend does the join) or a second command answered by the graph (the frontend asks once
the character is known). The second keeps the archive's view free of the graph, which is the
boundary sub-project 1 drew on purpose; the first keeps the screen from asking twice, which is
what N8 just finished undoing everywhere else. **N8's lesson points at the first**, and the
plan should say so with the reason rather than inherit it from this sentence.

## 5. Out of scope, named so nobody has to guess

- **Durations.** There is no clock in `log.txt`. Anything that looks like time on this screen
  would be invented.
- **Which floors, in order.** The archive counts floors; the events have more, and exposing
  them is its own decision.
- **Per-run statistics over the series** — the "bilancio" this conversation declined.
- **Deleting or editing a run.** The archive is a record of what the log said.

## 6. Done when

- `Run` lists the archive with the six columns of §2, filterable by the shared engine, with
  abandoned runs present and marked, and an expanded row that survives a missing catalog.
- `Live` draws the open run, and — for a character whose name resolves — what finishing it
  would open, saying both forms when the name is ambiguous.
- Both routes stop being placeholders in the shell.
- The empty states are the archive's own diagnostics, not invented sentences: no log folder,
  store unavailable, unreadable events, no catalog.
- Seen in a window. The archive on this machine has 28 sessions to draw, so this screen can be
  looked at without playing anything.
