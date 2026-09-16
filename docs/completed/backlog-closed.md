# Completed — the backlog entries that closed

Every entry of `docs/BACKLOG.md` that finished, with what it measured. **Searched, never
read** — but searched often, because this is where the measurements are: the wiki's
`{{dlcset}}` dictionary, the four characters with no page, the item pools, the three
compression modes. An entry closes with the reason and the numbers, which is why none of this
is deleted.

They left `docs/BACKLOG.md` on 2026-09-16, where they were **31 of its 58 entries** — more
than half of a file whose job is to say what is left. Their one-line index stays there, so a
question that starts "was this ever looked at?" is answered without opening this.

---

## B1 — Item detail: primary and secondary effects (an **analysis** task) ✅ closed on 2026-09-05

Logged on 2026-09-05. Reference for shape/format: the wiki page for *False PHD*
(`bindingofisaacrebirth.fandom.com/wiki/False_PHD`).

**Closed the same day.** Report —
`docs/superpowers/reports/2026-09-05-b1-sources-effects-report.md`. What came out of it, that the entry
below didn't know:

- The fandom wiki is the **copy abandoned** by the 2023 migration: stuck at 2025, HTTP 403
  for non-browser clients. The live wiki is **`bindingofisaacrebirth.wiki.gg`**, license **CC BY-SA
  4.0**: the NC question dissolves. Attribution and share-alike bind the dataset.
- wiki.gg exposes **Cargo tables** queryable via API (`collectible`, `trinket`,
  `achievement`, `entity`, `challenge`, `version`): these are what the `{{i}}`, `{{t}}`,
  `{{e}}`, `{{s}}` templates use to resolve names, so the name → id map for internal links
  comes for free.
- Coverage by id: 719 items out of 721 (missing the internal variants 59 and 656) and 188 trinkets out of
  188; the Effects, Notes, Synergies, Interactions, Bugs sections are consistent across the 723 pages.
  A known error: *Tonsil* declares the id of *Broken Glass Cannon*.
  *(Corrected on 2026-09-05: it's not a wiki error. The page has two infoboxes — the current trinket
  97 and the Afterbirth+ collectible 474, replaced by Broken Glass Cannon in
  Repentance — and the Cargo table confirms it with the `dlc` bitmask. See
  `docs/superpowers/reports/2026-09-05-b1-sources-effects-report.md` and
  `docs/superpowers/reports/2026-09-05-wiki-dataset-report.md`.)*
- **EID** covers 100% of items and trinkets, Italian included, and has the Italian names
  that the stringtable doesn't; but it **has no license** and can't be redistributed without the author's
  consent. The Italian fandom wiki is made of empty skeleton pages.
- Decision: **dataset built at build time and shipped in the package**, one pass per release
  against the wiki (whose `robots.txt` asks for responsible traffic), optional update from
  GitHub. The implementation waits for the screen design, like B3.

Dataset implementation closed on 2026-09-05: report —
`docs/superpowers/reports/2026-09-05-wiki-dataset-report.md`.

*The original entry, for the record:*

### What we already have, from the game files

For each of the 909 items (425 passive, 170 active, 126 familiars, 188 trinkets):

| data | source | note |
|---|---|---|
| id, type | `items.xml` | key `(kind, id)` |
| name | `stringtable.sta` | 8 languages, no Italian |
| short phrase ("Worse pills + evil up") | `items.xml`, `description` attribute | this is the wiki's "quote"; 403 items in the base |
| quality (0–4) | `items_metadata.xml` | |
| tags (`summonable offensive nocantrip`…) | `items_metadata.xml` | |
| pool membership and weight | `itempools.xml` | 31 pools |
| 32×32 icon | `.a` archives | extracted at runtime |
| achievement that unlocks it, with condition in English | `items.xml` + `achievements.xml` | 370 items out of 909 |
| DLC of origin | id ranges | |
| seen or not in the save | section 4 | |

All of this is already in the `catalog` crate and, partly, on the IPC. An item detail with
these fields could be designed today.

### What's missing

The sections a wiki page has and the game files don't: **Effects** (the bulleted list of
actual effects, with numbers: "+0.6 damage per pill…"), **Notes**, **Synergies**,
**Interactions**, **Bugs**. The game doesn't write these anywhere: they're community
knowledge. They have to come from an external source, and choosing that source is the task.

### The analysis task

Compare the candidate sources and propose a choice, without implementing. For each:
license and attribution required, coverage (how many of the 909 items and the trinkets),
languages available, format and how parsable it is, how it's updated, whether it holds up against the
project's constraints (no API key, no account, offline after the first dataset, assets with
their license and attribution in the package).

Candidates known as of 2026-09-05:

1. **Fandom wiki (English).** Data collected today, to be verified in the task: the public
   MediaWiki API responds without credentials (`api.php?action=parse&page=…&prop=wikitext`), while
   the HTML rejects non-browser clients (HTTP 402). License declared by the site: **CC BY-NC-SA**.
   The wikitext has an infobox with `id` matching ours (`id = 654` for False PHD), `quote`,
   `description`, `quality`, `tags`, `dlc`, and the sections above as text with templates that
   link to our entities (`{{i|Little Baggy}}` item, `{{t|…}}` trinket, `{{c|…}}` character,
   `{{e|Black Heart}}` entity, `{{a|…}}` achievement, `{{s|Depths}}` floor). The `Collectibles`
   category is enumerable via the API. Still to understand: parsing of templates and tables, how
   stable the text is, what NC means for us.
2. **External Item Descriptions (EID)**, a community mod with concise descriptions of
   effects in many languages, maintained on GitHub. Still to verify: license, format (Lua), and
   granularity: it's one line per item, not the full sections.
3. **Wikis in other languages** (the Italian wiki exists but is separate and less complete): still to
   measure coverage and id alignment.
4. **Our own curation**: a versioned file written by hand starting from one of the sources above.
   Cost and maintenance to be estimated.

Questions the analysis report has to close: which source, under what license in the package,
where the dataset lives (in the package or downloaded on first run, with optional update
per constraint 4), how to signal to the user that the dataset is older than the game, and how
entity links become internal links in the app.

---

## B2 — A challenge's reward (implementation, small) ✅ closed on 2026-09-05

Logged on 2026-09-05. We know **how a challenge unlocks** (`Challenge.unlocked_by`) but
not **what completing it gives**: the game only writes it as a note on the reward achievement
("Beat Challenge #19" above achievement 62, *Epic Fetus*).

**Closed the same day**, a bounded task with no spec or plan: design discussed in chat, TDD, commits
`catalog:` and `ipc:`; logged in `docs/STATUS.md`, session of 2026-09-05. What came out
while executing it, that the original entry didn't know:

- The pattern **isn't just one**. The Repentance+ file (637 achievements) writes the reward in three
  forms: `Beat Challenge #N` comment (challenges 1–20, 20 achievements); `beat Challenge N (Name)`
  comment, sometimes with an `unlocks Percs/Overdose` tail (challenges 21–30, 10);
  `steam_description="Complete Challenge N."` attribute **with no comment at all** (challenges 36–44,
  9). With just the original entry's pattern, only 20 out of 39 challenges were covered.
- Challenges **31–35 and 45 have no trace** in the file: `rewards` stays empty, it can't be inferred
  by position.
- The "30 in the base file" count was wrong: `config.achievements.xml` in `config.a` has
  **20**, all with the hash mark. The 30 were the ones with and without the hash mark in the Repentance file.
- Done: `catalog::reward::challenge_beaten` (a strict parser, verb `beat`/`complete` +
  `challenge` + optional `#` + number; rejects `You unlocked Challenge #4` and `Unlocked a new
  challenge.`); `Achievement.steam_description`; `Challenge.rewards: Vec<AchievementId>` filled
  in by `Catalog::build`, sorted; `Diagnostic::RewardForUnknownChallenge` if the cited challenge
  doesn't exist; on the IPC `UnlockTarget::Challenge { id, name, rewards: Vec<u32> }` with the
  achievement **ids**, not the views (the frontend already has a node for achievement in
  `UnlockView` and looks it up by id; repeating the nodes would make the type recursive). Tests
  against real data: 39 challenges with a reward, 6 without; challenge 19 → 62, challenge 36 → 517,
  challenge 44 → 533, challenge 1 → 89.
- **Left out, on purpose**: using `steam_description` as a fallback `hint` for the nine
  achievements with no comment. That's presentation, and it's for the screen design to decide.

---

## B4 — An item's unlock tree (this is M2) ✅ closed on 2026-09-07

Logged on 2026-09-05. **Closed 2026-09-07** — report in
`docs/superpowers/reports/2026-09-07-unlock-graph-report.md`.

What execution answered, that the entry below got wrong: **the 283 English conditions were
never the source.** The wiki dataset already in the repo carries typed requirements for
641 of 641 achievements, so there is no prose parser and no rules file full of patterns;
the versioned rules that do exist are a generated inventory plus 79 hand-written verdicts
for the targets that reduce to nothing on their own. The recursion is real and it reads
its edges from the game's own `unlocked_by` links, never from the wiki.

*The original entry, for the record:*

Today the chain stops at the first step (item → achievement →
condition in English). A tree requires structured parsing of the 283 conditions into a
versioned rules file ("beat X with Y", "Beat Challenge #N", "collect N …") and the
recursive linking condition → character/boss/challenge → their unlocks. The `graph` field
in the IPC contracts is already planned as a stub for this. It's not a backlog task: it's
milestone M2, logged in `STATUS.md`.

---

## B5 — Global search: one thing searches all of them ✅ closed 2026-09-12

Logged on 2026-09-06, an explicit request. It has to find **anything the app knows about**:
an item, an achievement, a challenge, a character, a boss, a wiki page and what's
inside its sections, a node in the unlock tree, and the screens themselves.

Two surfaces, not one: the **palette** (`Ctrl+K`, opens from any screen, results
grouped by type, enter opens) for quick access, and the **Search screen** for the
complete set, filterable by type, for when the results don't fit in a palette. The palette leads
to the screen; the screen doesn't require going through the palette.

**Designed on 2026-09-12** in `docs/superpowers/specs/2026-09-12-screens-wiki-search-design.md`
as sub-project 3.5b, with 3.5a (the Wiki in tabs) closed the same day: the index, the ranking
and the two surfaces are Decisions 5 to 8 there, and the destination a result opens is a tab
location the Wiki half already defines. What stays open here is the implementation.

### What we already have

Almost all the material, which is why this entry is small:

| searchable data | source | how much |
|---|---|---|
| item and trinket names | `catalog` | 909 out of 909, with sprite and quality |
| character names | `catalog` | 41, 37 with a portrait |
| achievements and their condition in English | `catalog` | 637, 283 with a condition |
| challenges, with their reward | `catalog` | 45 |
| bosses | `catalog` | 103 |
| wiki section text | `wiki`, embedded dataset | Effects, Notes, Synergies, Interactions, Bugs, Behavior… |
| Unlock nodes, done/not done | `ipc` | 641 slots |

And above all: **the identity of a result already exists**. `Target` (§8 of `DESIGN-BRIEF.md`,
`ui/src/lib/ipc/types.ts`) is what a wiki reference points to and what `loadWiki`
accepts. A search result is a `Target` plus the context that explains why it matched:
there's no need to invent a parallel type, and "open the result" is the same action as "follow a
wiki link".

### What's missing

The index, the IPC command that queries it, and the two views. Nothing else.

### Closed as sub-project 3.5b

Implemented on 2026-09-12 (`docs/superpowers/reports/2026-09-12-screens-search-report.md`), and
every question below was answered where it said it would be:

- **The index lives in `crates/ipc/src/search.rs`**, pure, with the wiki half built once and
  held in managed state and the catalog half read per query — because "the game isn't
  installed" is never a cached answer.
- **Measured, not assumed**, as this entry asked: 1,727 pages flattened in **15 ms**, a query
  answered in **15–19 ms** over the whole dataset and the installed catalog, in a debug build.
  **FTS5 in `store` is not needed**, and nothing in the contract would change if it ever were.
- **Ranking**: six tiers over the fields, then the profile ("not done" before "done"), then the
  name — a total order the tests pin.
- **The language**: names stay English and screens are messages; the Screens rows are matched
  against the translated label, so one query hits both.
- **Degrading**: no catalog, no dataset, no profile and an unread section are five diagnostics
  in the payload, never an error.
- **The type on the IPC**: `SearchMatch` is tagged with `rename_all_fields` and its JSON shape
  is pinned; `SearchDiagnostic` and `ProgressMark` are fieldless, so they travel as bare
  strings — the correction this entry's own recommendation needed.

### Questions the task had to close

- **Where the index lives.** Recommendation: in `ipc`, built once from the catalog and
  `wiki::Dataset::embedded()`, held in managed state like the catalog. It stays a pure crate
  and doesn't touch disk.
- **The body of the wiki pages is full-text**, and it's the only part that isn't a list of short
  names: ~920 pages with several sections each. A linear scan or FTS5 in `store` — **to be measured
  before choosing**, not assumed: at these orders of magnitude the likely answer
  is that a scan is enough, and an FTS table would be an extra migration for nothing.
- **Ranking across different types.** An exact match on an item's name isn't worth as much as
  an occurrence buried in a Bugs section; and the active profile is legitimate ranking
  information ("not done" before "done"), not just a facet.
- **The language.** Item, character, and boss names **stay in English** even in Italian
  (§12 of the brief): they're the names the user searches by. But screens, actions, and facets have
  Italian names, and the same query has to be able to hit both.
- **What happens when the material is missing.** Constraint 5 applies: search **degrades**. Without
  a catalog (game not installed) it still finds screens and actions, and states what it isn't
  searching. It's not an error state.
- **The type on the IPC.** The result variants carry different data, so a tagged enum with
  `#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]` and the
  JSON shape pinned by a test — this is exactly the case where forgetting `rename_all_fields`
  makes TypeScript read `undefined` without anyone noticing.

---

## B7 — Bringing the comments into English too (implementation, mechanical but delicate) ✅ closed on 2026-09-07

Logged on 2026-09-06, an explicit request, right after `design-export` was
realigned to the repo's convention.

**Closed 2026-09-07.** No separate report: session log in `docs/STATUS.md`, entry
"2026-09-07 — B7: the repo's prose moves to English". What execution answered, that the
entry below left open:

- **The three extra surfaces (questions section, first bullet): yes, all three.**
  `assert!`/`panic!`/`eprintln!` messages went in the same pass as the comments;
  `CLAUDE.md`'s commit rule now says English, effective from this task's own commits
  onward — past history isn't rewritten.
- **`docs/PROJECT.md`: in.** Rewritten from `docs/progetto.html` as plain Markdown
  (translated, losing the CSS on purpose) as part of this same task, so the question
  the original entry asked about a `.html` file resolved itself: the file it's asking
  about doesn't exist anymore.
- **Verification: reread by hand, no scanner check added.** A `scan-conventions.mjs`
  rule to tell languages apart was considered and dropped — the deliberate exceptions
  below (business-facing diagnostic text pending i18n, deliberately-garbage test input)
  would make a blunt regex check noisier than useful.
- **Order and granularity: one crate/doc-group per commit, `CLAUDE.md` first**, exactly as
  planned — with heavy parallelization across subagents to fit the size (2,100+ comment
  lines across 116 files, ~30 `.md` files), not a single cheap model doing all of it in
  sequence: nuance mattered too much for that, per the entry's own warning below.
- **What stayed in Italian on purpose, found only while executing**: real business-facing
  diagnostic strings the app already returns today pending i18n (e.g. a database-version
  mismatch reason), and test input data deliberately shaped to look like garbage (invalid
  VDF/JSON literals, a lone accented character fed through percent-encoding). Neither is
  "prose for whoever develops" — one is what the app already tells a user, the other is
  data a test constructs, not a message a developer wrote.
- **A gap this task doesn't cover, found while executing**: `crates/unpack` has a handful
  of Italian *identifiers* (`disponibili`, `leggibili`, `tabella`, `PRECEDENZA`, `RADICI`,
  `VIVI`, `PICCO`, `SOGLIA_APERTURA`, `controllati`, `mancanti`) — B7 was scoped to
  comments and docs, never identifiers, so these were left alone. Not registered as its
  own backlog entry yet; small enough to fold into whoever touches that crate next.

*The original entry, for memory:*

Today the rule is **identifiers in English, prose in Italian**: comments, doc-comments,
`assert!` messages, text shown to the operator, documents in `docs/`, and commit
messages. This task flips the second half.

### How big it is

| surface | how much |
|---|---|
| comment lines in the crates (`//`, `///`, `//!`) | **2,110** across 109 files |
| long strings in tests (almost all `assert!` messages) | ~580 |
| `.md` files | **31** across the whole repo, including specs and plans in `docs/superpowers/` |

### Why it isn't a mechanical translation

The comments in this repo **don't describe what the code does: they say why**, and often
carry the only trace of a defect found in the field — "the first sheet is the wrong one
and you can't tell by eye", "a wrong path doesn't error, it goes silent", "the test encoded
the defect as the expectation". They're the valuable part of the file, not decoration: a
word-for-word translation that loses the nuance does **more harm** than leaving them in
Italian.

The opposite also holds, and it's the reason this hasn't been done until now: writing the
*why* in your own language comes out more precise and denser.

### The scope, decided on 2026-09-06

**The comments in the code and all `.md` files.** So that includes `CLAUDE.md`, `README.md`,
`docs/STATUS.md`, `docs/IMPROVEMENTS.md`, `docs/frontend-conventions.md`, `DESIGN-BRIEF.md`,
this file, and the specs and plans in `docs/superpowers/`.

### What this task is NOT

**It's not the app's language.** The interface will have a **language selector in
settings** with vue-i18n (already in the stack per `CLAUDE.md`, and "language" is already among the
settings in §4.3 of the brief): the strings the user reads live in the translation
files and aren't at stake here.

Hence the line separating the two, and it's a sharp one: **B7 is about prose for whoever develops**
— comments, documents, `assert!` messages, the text the tool prints to the console. The prose
for whoever uses the app goes through i18n, and the user chooses its language at runtime.

A consequence worth noting now: if the code goes to English, **English also becomes
the source language for i18n**, with Italian as the translation. Today §12 of the
brief keeps a hand-written IT/EN glossary precisely because that choice hadn't been made yet:
once it is, that glossary becomes the first translation file instead of a table in
a document.

### Questions that remain

- **The three surfaces that are neither comments nor `.md`**: `assert!` messages (~580), the
  text of developer tools' `eprintln!` calls, and **commit messages** from here
  on — today `CLAUDE.md` mandates them in Italian. All three are prose for whoever develops,
  so the likely answer is that they follow suit; but that's a decision, not an automatic consequence.
- **`docs/PROJECT.md`** isn't a `.md` but is the project document: in or out? *(This
  bullet originally named `docs/progetto.html`, the file this question was actually
  about; closing B7 renamed it to `docs/PROJECT.md` for consistency with the rest of this
  document, which reads oddly now that the file in question is, in fact, a `.md`. Answered
  below: in — rewritten as Markdown, translated, as part of this same task.)*
- **How it gets verified.** There's no linter that tells languages apart. Either a check in
  `ui/scripts/scan-conventions.mjs` (which already guards the rules no linter
  covers), or the rule stays written in `CLAUDE.md` and whoever writes enforces it.
- **In what order**, and at what commit granularity: one crate per commit is the obvious
  choice, but tests have to stay green at every step.

### The sensible way to do it

It's extensive, mechanical work that still requires judgment about the content: **one crate at a time, with
`pnpm check` after each one**. A good candidate to delegate to a cheaper model, with
the caveat that the rendering of the *why* needs a re-read — that's exactly the point where a rushed
translation loses the comment's value.

Before starting, the rule in `CLAUDE.md` needs updating, otherwise the repo ends up with
two conventions at once.

---

## B8 — What a real `log.txt` actually contains (a **spike**, blocks M4) ✅ closed on 2026-09-08

**Closed on 2026-09-08.** Report:
`docs/superpowers/reports/2026-09-08-b8-log-spike-report.md`, from one real run (Judas, hard,
Mega Satan, won) watched live with a throwaway probe. Both open questions answered — **rooms
are logged**, and the flush is immediate — plus five findings the entry wasn't looking for,
of which two change M4's design: the log **announces every save write** (132 times in one
run), and the save is written continuously *during* play rather than between sessions. Also
`from pool X` lies about the starting item, and rebuilding an inventory needs `catalog`
because actives replace one another.

**And one finding that isn't about M4 at all:** the game names every chunk of the save file
as it reads it, which puts names on the four sections `CLAUDE.md` lists as "to be
identified" and contradicts two we thought we knew. That's B9 below.

> **The third idea this entry parked was built on 2026-09-15, as F1.** B8 wrote that a
> secret-room finder "is **a different product**" and that the belief worth checking was that
> such tools reason on the map you have already explored plus placement rules, **not** on the
> seed. The belief was checked and holds; the scope decision was the owner's and is taken in
> the open in §1 of `docs/superpowers/specs/2026-09-15-floor-secret-rooms-design.md`. Report:
> `docs/superpowers/reports/2026-09-15-floor-report.md`. **This entry is not deleted**, because
> the reasoning in it is what makes the new decision legible.
>
> What F1 does **not** do is the half B8's own §5 calls the verification: the log's
> `N rooms in M loops` is not read, so the screen never says *the game generated 19 rooms and
> you have painted 15*. That half is **F2**, and it needs a measurement — which of the several
> generation attempts in a log describes the floor being played is a judgment, and it belongs
> in `run`'s fold measured against the five real logs, not assumed from the ordering.

Logged on 2026-09-08, out of the M4 brainstorming. **M4 shouldn't start before this
closes**: the whole log watcher would otherwise be designed on five line types nobody has
re-read since M0.

### What we already have

Five patterns, verified in M0 against a real log:

```
Adding collectible 225 (Gimpy) to player 0 (Cain) from pool treasure
RNG Start Seed: FYQ8 QQ8G (586324166) [New, 1]
Level::Init m_Stage 2, m_StageType 1 Seed 408474304
Game Over. Killed by (9.0) spawned by (84.0) damage flags (0)
playing cutscene 15 (Sheol).
```

So the run's seed is there, human-readable and numeric, with the run type; and every
**floor** carries its stage, stage type and its own seed.

### What's missing

**A log with a run in it.** The one on the dev machine
(`Documents\My Games\Binding of Isaac Repentance\log.txt`) is a launch-and-quit from
2024-03-05: 86 lines, no gameplay. Three things it shows anyway, none of which the
documented patterns mention:

- every line is prefixed `[INFO] - `;
- some entries **wrap over several physical lines** (the `Framebuffer Width:` block is one
  event over six lines), so "one line, one event" is false;
- **mods write into the same file** — this one has External Item Descriptions emitting
  `[INFO] - Lua Debug: …`. The rule file has to tell game events from the noise of
  third-party mods we don't control.

Two open questions that decide entire features, and that only a real log answers:

- **Does the log record rooms**, or only floors? Without room-level events there is no
  explored map, and nothing map-shaped can be built on top.
- **With what delay does the game flush to disk?** This decides whether a Live screen is
  actually live or a few rooms behind.

### The probe

Install the game, play one run to a death or a win, keep the `log.txt`. Classify it: how
many distinct line kinds, which events, at what granularity, how much mod noise. Half an
hour of play; the output is a document, no code kept.

### What the answer unblocks

The M4 design itself, plus three ideas raised on 2026-09-08: a live item tracker on a
second monitor (almost certainly yes), the explored map in real time (depends on rooms),
and a secret-room-finder-style helper (depends on rooms, *and* on verifying how those
tools actually work — the belief to check is that they reason on the map you have already
explored plus placement rules, **not** on the seed).

### Out of scope whatever the answer is

**Regenerating the map from the seed.** The seed doesn't contain the map; the map is what
the game's generator produces from it. Reproducing it means reimplementing that generator
bit-exact — its PRNG, the exact order of its calls, room selection from the `.stb` files,
pool weights — and it moves with every patch. One misplaced RNG call and everything after
it diverges. It would also change what the app *is*: everything here reads a file the user
already has, and the source of truth stays theirs; a generator makes the source of truth
the fidelity of our own clone, which we can't verify. That's the step from "if I'm wrong I
show incomplete data" to "if I'm wrong I show false data with a confident face".

Related, and worth deciding in the open rather than discovering halfway: a secret room
finder is **a different product**. IsaacDome answers "what am I missing, and what's worth
playing tonight"; that answers "where is the secret room right now".

> **Decided the other way on 2026-09-15**, in the open and by the owner, who asked for it:
> `docs/superpowers/specs/2026-09-15-floor-secret-rooms-design.md`. The paragraph above is
> kept because it is the reasoning the new decision answers, not a claim that was wrong. The
> belief this entry said to check — that those tools reason on the explored map plus
> placement rules and **not** on the seed — was checked against the reference site that day
> and holds; regenerating the map from the seed stays out of scope, unchanged.

### One finding already in hand

The loss window for log data is narrower than `docs/PROJECT.md` implies. The game rewrites
`log.txt` on the **next** launch, so the last session survives on disk until then —
verified on the dev machine, where a March 2024 log ending in
`Isaac has shut down successfully` is still intact two and a half years later. The rule is
not "the app must run while you play" but **"the app must run at least once between one
session and the next"**. An app that reads the whole current log at startup recovers the
last session; `RNG Start Seed` gives the dedup key that makes re-reading safe.

---

## B10 — The design export pack: what the design tool had to measure by hand (implementation, `design-export`) ✅ closed on 2026-09-15, declined

**Closed because its subject was retired.** `pnpm design:export` is abandoned: the design is
decided at runtime on the real screens now, and the pack's 6065 images left the repository with
it. Every item below is a request to make the *exported package* easier for a design tool to
read, and there is no longer an export or a tool reading it.

**Nothing in it was a defect in the app**, which the entry said on the day it was logged: "they
aren't bugs in the app". The one thing worth carrying forward is item 1's observation, because it
is about the game's own sprites and not about the pack — `completion_widget/paper_00.png` is
96×96 with the drawing at `x 0–84, y 3–82`, so anything that centres the frame centres empty
pixels. Whoever draws those sheets at runtime meets the same untrimmed frames; `ipc::sprite_png`
and `mark_art` are where that lands, not here.

The entry as it was:

**Needs:** the game — `pnpm design:export` reads `samples/packed`, so the pack cannot be regenerated without it.

Logged on 2026-09-10, from `design-export.md` inside the Claude Design export: the places
where the pack `pnpm design:export` produces forced the design tool to measure, crop or
guess. They aren't bugs in the app, but cycle 2's matrix cell leans on the first four, and
every later export repeats the work until the pack says what it knows.

1. **Sprites aren't trimmed**: `completion_widget/paper_00.png` is 96×96 with the drawing at
   `x 0–84, y 3–82`, so centring the frame centres empty pixels. Export trimmed frames, or a
   `trim: [x, y, w, h]` and `pivot: [x, y]` per frame in `sheets.json`.
2. **`sheets.json` has no content rectangle or pivot** (same fix).
3. **Delirium's mark isn't among the marks**: it lives in `onlinelobby/background_completion_delirium_*`,
   under another naming.
4. **The `_00`/`_02` tier is guessed from layer names**: an explicit `mark`, `tier` field.
5. **No usable card or card back**: `ui_cardfronts/outline.png` is a 16×24 outline.
6. **Papers come paired in one image** (`pausescreen_mystuff/paper.png`, `deedsmenu/paper.png`,
   `scoremenu/smallpaper_00.png`): one file per sheet, or declared 9-slice cuts.
7. **`sheet` paths contain spaces** (`gfx/ui/seed paper.png`).
8. **The co-op sheet's holes are undeclared**: a `characters.json` mapping character id → cell,
   with an explicit `null`.
9. **40 completion cells are unreadable** (already a real data gap, modelled as `unknown`).
10. **`unlock.json` derives the target from text**: 30 of 72 sampled rows match nothing.
11. **Boss portraits are indexed by sheet position**, the entity key only inside the `source`
    file name: a `target: { kind: 'entity', id, variant }` field.
12. **A typed target doesn't imply an image**: the list of holes of `target_sprite`, not only
    its aggregate coverage.
13. **`unlock.illustrated.json` inlines icons as base64**: references to paths (the C2 flaw).

---

## B13 — The marks map moves to `ipc` with the Completion screen (implementation, cycle 3) ✅ closed on 2026-09-11

**Closed with sub-project 3.2** (`docs/superpowers/specs/2026-09-11-screens-completion-design.md`):
the map is `crates/ipc/src/mark_art.rs`, the icon protocol serves `mark/<column>/<tier>` and
`head/<row>` as crops (`ipc::crop_png`, moved from `design-export`), and
`crates/ipc/tests/mark_art.rs` resolves every one of the twelve columns to two different
tiers. The real-archive twin, `mark_art_real.rs`, skips on a machine without the game and
has yet to run on one. The Delirium `symbolFallback` is not carried: the cell's own bars
outfit is the app's fallback.

Logged 2026-09-10, from cycle 2: `MarkCell` takes each column's symbol URLs as a prop, and
nothing in the app serves them yet. `crates/design-export`'s `marks.json` holds the column →
symbol map (with Delirium on the online-lobby sheet and its `symbolFallback`); DESIGN-BRIEF
§5.6 already says it belongs beside `BOSSES` in `ipc`. Needed: the map in `ipc`, the icon
protocol extended to the completion-widget sprites, and a test that every one of the twelve
columns resolves to two tiers.

---

## B16 — The brand mark in the navbar is the app icon, and the icon is `primary` (implementation, `ui` and `app`) ✅ closed on 2026-09-12

Logged 2026-09-12, from the owner's review of the bars.

### What we already have

- `shell/NavBar.vue` draws the brand as a bordered 14px square (`size-3.5 border-2
  border-primary bg-sheet`) beside the name: a placeholder from cycle 2, never meant as the
  logo.
- The app icon exists twice, in two colours: `crates/app/icons/` (the PNG/ICO/ICNS set Tauri
  bundles, 512×512 source) and `ui/public/favicon.svg`, the Dome silhouette with Isaac's face
  cut out, filled `#47c7ff` — a light blue that belongs to no token.

### What's missing

1. The square in the navbar becomes the app icon: the SVG inlined as a component under
   `ui/src/components/shell/` (or an `<img>` on the public asset), sized by a token, with
   `currentColor` so the focused / unfocused states keep working through the existing
   `group-data-[focused=false]` classes.
2. The icon's fill becomes **`primary`** (`#901800`, `ui/src/assets/theme/colors.css`) in
   every copy: `favicon.svg` and the bundled icon set. The PNG/ICO/ICNS are regenerated from
   the SVG (`pnpm tauri icon` takes one source image), not recoloured by hand, so the two
   never drift again.

### Done when

The navbar shows the Dome mark, red, and the taskbar and installer show the same one.

---

## B18 — No white flash at launch: a splash with the app logo (implementation, `app` and `ui`) ✅ closed on 2026-09-12

Logged 2026-09-12. Opening the built app shows a **white window** before the frontend paints:
the webview's default background, visible for as long as Vite's bundle takes to load and the
first commands take to answer. On a dark app it reads as a glitch.

### What we already have

- `crates/app/tauri.conf.json` declares one window (`decorations: false`) with no
  `backgroundColor` and `visible` left at its default `true`: the OS shows the frame before
  there is anything in it.
- `ui/index.html` is the bare Vite template: `<div id="app">` with no colour and no content
  until `main.ts` mounts.
- `--color-background: #150e0d` in `ui/src/assets/theme/colors.css`.
- The Dome mark (B16), which is what a splash would show.

### What's missing, in order of cost

1. **Kill the white**: `backgroundColor` on the window in `tauri.conf.json` matching
   `--color-background`, and the same colour on `<html>` in `index.html` so the page never
   paints white even when the webview is up before the CSS. The value lives in two places by
   necessity (the config is JSON, not CSS): a test that reads both and compares them.
2. **A splash**: the logo centred on the background, inside `index.html` so it's there before
   any JavaScript, removed by `App.vue` once the shell has its first answer from the backend
   (the setup state). Tauri's own pattern — `visible: false` in the config and
   `getCurrentWindow().show()` from the frontend — is the alternative when even the coloured
   empty frame is too much; it costs the window appearing later rather than empty.
3. Not a `splashscreen` second window: one window, the logo drawn in the same document,
   otherwise B15's window logic gains a case for nothing.

### Done when

Launching the installed app never shows a white surface; the first frame is the dark
background with the mark, and the shell replaces it without a jump.

---

## B24 — Clicking a section navigates, and the section lights up (implementation, `ui`, small) ✅ closed on 2026-09-12

Logged 2026-09-12, a product rule from the owner that **reverses Decision 5 of the shell spec**
(`docs/superpowers/specs/2026-09-11-screens-shell-profile-design.md`, "clicking Wiki or
Progress shows that section's items; it doesn't navigate"): clicking Wiki, Progressi or the
cog has to move the active tab **at once** to the section's first available page, not wait
for a click in the sidebar. And the section the user is in has to read as lit.

### What we already have

- `App.vue`: `showSection` only sets `browsing`, the sidebar's section; the cog does the
  same with `SidebarSection.Settings`. Nothing calls `tabs.navigate` until a sidebar entry is
  clicked. The entries per section are `sidebarEntries` in `shell/sectionNav.ts`, in order,
  so "the first" is `sidebarEntries[section][0]` — Next steps, the first wiki category, the
  Profile.
- The lit state exists on paper: `NavBar` sets `aria-current="page"` on the section equal to
  `navSectionOf(browsing)`, and `ButtonVariant.Section` styles it (`border-highlight`,
  `bg-data`, `text-highlight`). Two reasons it doesn't read as lit today: Settings marks
  **neither** button by design, so the cog never lights; and the treatment is a 2px top edge
  on the navbar's surface, which the owner didn't perceive as "on" — a design check on the
  Kit page before touching it.

### What's missing

1. `showSection` navigates the active tab to the section's first entry (`tabs.navigate`,
   `Ctrl`+click keeps opening a new tab as the sidebar does), and sets `browsing` as today;
   the cog does the same with Settings' first entry, the Profile. A page that needs a profile
   still shows the `ProgressGate` when none is active: the gate is the page, as today.
2. The lit section follows the **active tab's** section, which `browsing` already does
   through the watch; with navigation on click, the two agree at every moment and `browsing`
   may collapse into the tab's section alone.
3. A visible "on": the cog gets its own lit state when a Settings page is active (today it
   is a plain `Chrome` button), and the Section variant's on-state is checked against the
   design at the app's real size, not on the Kit page alone.
4. Decision 5 is amended in the spec, not silently contradicted by the code.

### Done when

Clicking Progressi from a wiki page shows Next steps without a second click; the button
under the cursor reads as lit, and the cog reads as lit on the Profile page.

---

## B25 — About is a dialog, not a page (implementation, `ui`, small) ✅ closed on 2026-09-12

Logged 2026-09-12, from the owner: what "Informazioni" has to say is short — the name, the
version, the licences and the dataset attribution — and a centred dialog carries it; a page
in a tab, with a route and a placeholder, is more than it needs.

### What we already have

- A route `RouteName.About` (`/about`, origin `TabOrigin.About`, placeholder text), opened in
  the active tab by the navbar's Info button (`App.vue`, `@about`), and counted under the
  Settings section by `sectionOfOrigin`.
- The `Dialog` primitive under `components/ui/dialog/` (cycle 1), unused by any screen yet.
- The content it will hold: the app name and version from `tauri.conf.json`, the font's
  CC BY 3.0 credit and the wiki's CC BY-SA 4.0 attribution that B11 says must ship.

### What's missing

1. The Info button opens a `Dialog` over the current tab — no navigation, no tab, no
   sidebar entry. Closes with Escape, the overlay, or its one button.
2. `RouteName.About` goes, with its placeholder, its `TabOrigin.About`, the `About` arm of
   `sectionOfOrigin` (exhaustiveness makes the compiler list every site) and the message
   keys the page alone used. A saved session (B6) never has to restore an About tab.
3. The version read once, from the backend or from a build-time constant, never typed in
   the frontend.

### Done when

Clicking Info shows the dialog on top of whatever tab is open, the tab strip doesn't
change, and no route named `about` exists.

---

## B26 — Scaling the whole interface from Settings (implementation, **pulled ahead**: it shapes every token) ✅ closed on 2026-09-12

Logged 2026-09-12, a product requirement from the owner, with a priority: the user picks the
size of the whole interface from Settings, with a **draggable slider over fixed steps**, and it
has to really work — every screen, every sprite, the chrome. The owner wants it **as early as
possible**, so that the variables are defined the scaled way once and no later screen is built
on values that don't scale. Tracked in `docs/STATUS.md` as sub-project 3.5c (3.5b is the
search half of 3.5, which this precedes).

**The reference is Discord's "Livello di zoom"** (four screenshots handed over the same day,
`Accessibilità › Densità visiva`): one slider, **50 · 67 · 75 · 80 · 90 · 100 · 110 · 125 ·
150 · 175 · 200**, the current value drawn in the accent colour above its tick, `Ctrl` `+` /
`Ctrl` `-` doing the same steps from anywhere, and a **preview card pinned at the top** of the
settings page that shows a slice of real UI (a message, a button, avatars) at the chosen
size while the page scrolls under it. "Freely" means that ladder — the browser's own zoom
levels — not a handful of sizes of ours. Discord keeps three other controls apart from the
zoom and so do we: the chat text size (our type scale, not exposed), the UI density
(compact / default / spacious — our `row-compact` / `row` / `row-wide` tokens, already there),
and the spacing between groups (no counterpart).

### What we already have

- Every size is a token in `ui/src/assets/theme/` (rule 2 of the frontend conventions): 38 px
  tokens in `spacing.css`, the type scale in px in `typography.css` ("a pixel font is crisp on
  whole pixels"), three in rem (`sprite`, `achievement`, `wiki-figure`), and Tailwind's own
  4px grid, which is rem-based and follows the root font size already. Because nothing is
  hardcoded in a component, scaling is a change to the token files, not to the screens — which
  is exactly why it's cheap now and expensive after cycle 3.
- `PixelSprite` draws 32×32 sprites at 2x with `image-rendering: pixelated`.
- `settings.json` through `crates/app/src/settings_file.rs` and `ipc::Settings`, holding only
  the active profile choice today; the Settings section of the sidebar (Profile, Tabs).
- A `Slider` primitive **doesn't exist** in `components/ui/` (23 primitives, none of them a
  slider); Reka UI has one to wrap in the kit's hand.

### The two mechanisms, and which one

1. **Webview zoom** (`getCurrentWebview().setZoom(f)`, Tauri 2, WebView2 honours it): one
   call, everything scales, the title bar included since it's ours. Costs: fractional zoom
   turns `pixelated` sprites uneven and the pixel font soft; the zoom is webview state, so a
   torn-off window (B15) has to re-apply it; it's invisible to the Kit page.
2. **A root scale**: the px tokens become rem (`13px` → `0.8125rem`), the root font size is
   `16px × step`, and the whole `@theme` follows; sprites keep their own rule — an integer
   multiple of 32 chosen from the step (`2x` at 1, `3x` at 1.5, `4x` at 2), never a fraction.
   Costs: one pass over the token files, a test that no px token survives except where a
   comment says why (borders, hairlines, the scrollbar).

Discord's ladder is Chromium's zoom ladder, which is what the first mechanism gives for
free and the second has to reproduce. Still, the second is the one to take: it keeps the
design rules inspectable in CSS, works on the Kit page and on fixtures (the preview card is
just the same tokens under a different root), survives a torn-off window without a re-apply,
and lets the sprites stay whole where zoom would smear them. Whole-pixel crispness of the
text is exact only at integer steps; at 67 or 125 the browser rounds per element, exactly
as Discord's text does at those steps — to be looked at on the Kit page, not assumed. What
the root scale must not do is invent a ladder of its own: the eleven values are the steps.

### The steps and the control

- Steps: Discord's eleven, **50 · 67 · 75 · 80 · 90 · 100 · 110 · 125 · 150 · 175 · 200**,
  a `const Scale = { … } as const` in `lib/`, the wire value the percentage as a number.
- A `Slider` primitive (Reka's, styled by the kit: flat track, the thumb a square, the tick
  labels above, the current one in the accent, no transition), snapping to the steps.
- `Ctrl` `+` / `Ctrl` `-` / `Ctrl` `0` from anywhere in the app, the same steps, the same
  persisted value: a shortcut is not a second scale.
- **The preview card pinned at the top of the settings page**, like Discord's: a fixed slice
  of real UI — a KPI tile, a row of the matrix with a sprite, a button — drawn at the chosen
  size while the rest of the page follows too; it stays in view when the page scrolls, so the
  slider and its effect are on screen together.
- Sprites: at each step the sprite side is `32 × round(2 × step)` device-independent
  pixels, an integer multiple — at 125 that's 3x (96px), not 80px — so pixel art never gets a
  fractional multiple whatever the step. The rule and its table live beside `Scale`.
- The value persists in `settings.json` (`ipc::Settings` grows `scale`, default 100, an
  unknown value reads as 100 — never a fatal error), is applied before the first paint (the
  splash of B18 is the moment) and on every window.

### Done when

The slider and `Ctrl` `+`/`-` move the whole app across the eleven steps with no element
left at its old size, the preview card shows it at the top while the page scrolls; a 32px
sprite is an integer multiple at every step; the value survives a restart; and the scanner
has a rule that a px token needs a reason.

---

## B28 — Unlock calls a Tainted character by its base name (bug, `ipc`; the data is right) ✅ closed on 2026-09-12

Logged 2026-09-12, from the owner's review of Unlock: rows such as *You unlocked "The Lost"*,
slot 484, "sbloccabile ora", for a character the profile already has. Traced the same day,
with the sample of 2026-07-09:

- **Slots 474–490 are the Tainted characters' unlocks.** `achievements.xml` writes their
  `text` exactly as the base character's — `You unlocked "The Lost"` — and says "Tainted"
  nowhere in the text; the form is only in `gfx="Achievement_TheLostB.png"` and in
  `steam_name="The Baleful"`. `players.xml` confirms it: player 10 (The Lost, portrait
  `Character_012_TheLost.png`) has `achievement="82"`, player 31 (portrait
  `Character_012b_TheLost.png`, the `b` of the Tainted form) has `achievement="484"`, and
  both carry the same name key `#THE_LOST_NAME`.
- **The save is right.** In the 2026-07-09 sample, slot 82 (The Lost) is 1 and slot 484
  (Tainted Lost) is 0; 474 (Tainted Isaac) and 478 (Tainted ???) are 1. The rows the owner
  listed — 479, 480, 484–490 — are Tainted Eve, Samson, Lost, Lilith, Keeper, Apollyon,
  Forgotten, Bethany, Jacob, all genuinely not unlocked on that profile.
- **The graph is right.** `rules/requirements.json` has 484's refs — Red Key (item 580),
  the stage Home, character 10 The Lost — so "sbloccabile ora" means the profile has the Red
  Key and The Lost: Tainted Lost is one Home visit away, which is true. "Nessuna condizione
  nel file" is right too: the game writes no comment condition for 474–490.
- **The label is wrong.** `crates/ipc/src/graph.rs` names a character target by
  `c.text(&character.name)` alone, and `catalog::Character` has the `tainted` flag the
  marks matrix already uses to tell the pair apart (`marks.rs`: "it's the pair that
  identifies the character"). Unlock drops it, so the row says "The Lost" for player 31, and
  the Character facet (`FacetId.Character`, the name as a string) folds the two forms into
  one value.

### What's missing

1. The character target carries `tainted` across the IPC (a field on the view, mirrored in
   `types.ts`), and the frontend says "Tainted Lost" / "Lost contaminato" — the game's own
   convention for the name, translatable, not baked into the Rust string.
2. The row's headline is the target, not the game's `text`: *You unlocked "The Lost"* is
   what the file says and stays available, but the name the player reads is the target's.
3. The Character facet keys on the pair (character id, or name + tainted), never the name
   alone; the sort by character too.
4. A test on the real catalog: achievement 484 resolves to a Tainted character and 82 to a
   base one, with different labels — the pair 82/484 is the fixture because it is the one
   the owner tripped on.

### Done when

The row for slot 484 reads as Tainted Lost, the facet lists the two Losts apart, and no
character row in Unlock shares its label with another.

---

## B31 — Dragging a row lifts the whole card, and one component does it everywhere (implementation, `ui`, after design) ✅ closed on 2026-09-13

**Closed on `feature/drag-and-windows`.** `lib/drag/dragList.ts` holds the decisions (threshold,
hit test, the grab's offset, the ghost's origin) with its own tests; `composables/useDragList.ts`
owns the choreography and adds `Escape`, which neither screen had; `components/ui/drag/DragGhost.vue`
draws the lifted copy — `aria-hidden`, because it is a picture of the row, not a second one. The
queue and the tab strip are its two callers and keep their own pure drop semantics. **The kit has
no shadow token and that is a decision** (`assets/theme/shadow.css`: `--shadow-*: initial`), so
the lift is a `primary` border on an opaque sheet. Looked at on the Kit, in the strip and in the
Plan, with the drag driven through the real DOM: the marker still names the landing, the drop is
still instant, `Escape` puts everything back and commits nothing.

Logged 2026-09-12, from the owner's review of the Plan: dragging a queue row doesn't feel
right. What the owner wants is the **whole card lifted and floating above the rest**,
following the pointer, and on release landing **exactly where the red marker says**, as it
does today — the marker stays, the feedback is added. And since two screens already drag,
this should be **one component** that does the lifting, reused wherever something is dragged.

### What we already have

- Two hand-written pointer drags of the same shape: the queue (`screens/plan/QueueCard.vue`,
  rows `QueueRow.vue`) and the tab strip (`shell/TabStrip.vue`). Both: a press becomes a
  drag past a threshold, the pointer is captured only then, the rows' rectangles are read
  once, a `drop` with an index and an edge (`Above` / `Below`, or a side for the tabs) is
  recomputed on every move, and nothing moves until the release — the pure functions
  `lib/plan/queueDrop.ts` (`dropEdge`, `dropAnchor`) and `shell/tabs.ts` (`dropSide`,
  `moveIndex`) decide where.
- The feedback during the drag: the grabbed row is only **dimmed in place**
  (`opacity-disabled`) and a line is drawn in the gap where the drop would land. The card
  never leaves its slot, so the eye has nothing to follow.
- A third gesture, the sidebar's resize (`shell/SectionSidebar.vue`), is the same pointer
  choreography without a drop target; B27 wants it generalised for the tables.

### What's missing

1. **A lifted ghost**: at the moment a press becomes a drag, a copy of the row — same
   size, drawn at the row's position — is rendered above everything (a `Teleport` to the
   body, `position: fixed`, the kit's raised shadow, no transition) and follows the pointer
   by the offset of the grab; the original stays dimmed in its slot; the marker keeps
   showing where the drop lands; on release the ghost disappears and the row appears where
   the marker was, as today. If the design wants the landing animated, that's the one
   motion token this needs; the drop itself stays instant and named by `move_after`.
2. **One composable** — `useDragList` or the like, under `composables/` — owning the
   threshold, the capture, the rect snapshot, the ghost and the `drop` computation, fed by
   two pure functions (where the pointer is over the list, and what anchor a drop means).
   The queue and the tab strip become its two callers with their own pure functions; the
   sidebar's resize stays apart, it has no list.
3. **Keyboard stays**: whatever the pointer does, a row can still be moved without it (the
   queue already repairs any move); the ghost is pointer feedback, not the mechanism.
4. **Test the pure parts, look at the rest**: the composable's threshold and drop logic
   in Vitest; the ghost on the Kit page with a list of three rows, and in the Plan.

### Done when

Grabbing a queue row lifts a copy that follows the pointer above the page while the red
marker still names the landing; releasing puts the row there; the tab strip drags through
the same composable; and no drag-specific pointer choreography is left in a screen.

---

## B32 — "Prossimi passi" is hard to read: a name that says what it is, and cards rewritten (implementation, `ui`, after design) ✅ closed on 2026-09-13

Logged 2026-09-12, from the owner's review: the Next steps screen is hard to understand. The
name itself doesn't say what the list is — the owner proposes **"Passi consigliati"** or
**"Obiettivi consigliati"** — and both the copy and the cards are to be redone.

### What we already have

- The screen (`screens/NextStepsScreen.vue`, `nextsteps/StepCard.vue`) is the app's landing
  page (`defaultLocation`), so it is the first thing a player reads. It shows at most five
  nodes, all unlockable now, sorted by fan-out — the ones that open the most downstream
  (spec 3.3a, Decision 3, "delegated" to this very review).
- **The copy is the engineer's**: the intro says "Al massimo cinque righe, tutte sbloccabili
  adesso: le cinque che aprono più cose a valle. Un nodo che il grafo sa dire solo parziale
  non è un passo, perché non possiamo garantirlo." — true, and unreadable as a first
  sentence. The empty states talk about the catalogue and the graph.
- **The card shows the file, not the goal**: a rank, the drawing, the achievement's `text`
  as the game writes it (*You unlocked "The Lost"* — and B28 showed that on the reference
  profile the five steps were **484, 488, 489, 479, 480**, all Tainted characters labelled
  with their base names, which is where the owner's confusion started), one `Tag` badge per
  unlocked target ("Samson · personaggio"), the state badge, "in coda" or the add button,
  and on the right a large number with the label "sblocca" — the fan-out, which nobody
  outside the graph calls that.
- The parts that are right and stay: the choice of the five (fan-out, unlockable now,
  partial excluded), the one-click add to the Plan, the reload on profile change.

### What's missing

1. **The name**: "Obiettivi consigliati" reads better than "Passi consigliati" — a step
   implies a sequence, and the five are independent; the decision is the design's, together
   with the route title, the sidebar entry and the tab label (`routes.nextSteps`).
2. **The intro in the player's words**: what the list is ("cinque cose che puoi sbloccare
   adesso, quelle che aprono di più"), not how it was computed; the computation goes to a
   tooltip or to the About dialog's promises.
3. **The card as a goal**: the headline is *what you get* (the target, in its form — "Tainted
   Lost", "Samson"), the achievement's condition under it in one line (the wiki's
   requirement where the file has none: "arriva a Home con The Lost e usa la Red Key"), the
   picture, then *why it's worth it* — "apre altre 23 cose" as a sentence, not a bare number
   with "sblocca" — and the one action, add to the Plan. The state badge is redundant on a
   list whose every row is unlockable now.
4. **Empty states in the same voice**: no game installed, nothing unlockable now, everything
   done.

### Done when

The landing page has a name a player understands, an intro of one sentence in their words,
and cards that read "what, how, why, add" top to bottom; and the reference profile's five
rows name the Tainted characters and their condition.

**Closed on 2026-09-13**
(`docs/superpowers/reports/2026-09-13-goals-and-achievement-detail-report.md`). The screen is
**"Obiettivi consigliati"** — a step implies a sequence and the rows are independent — its
intro says what the list is rather than how it was computed, and the body is grouped by the
reason a row is there (`Aprono di più`, `Ci sei quasi`), each group ending on Unlock with the
filter already picked. The card leads with what you get, drops the state badge that said
nothing on a list where every row is unlockable now, and links to the achievement's detail.

**The last clause is what cost the most.** "Their condition" could not come from the game
file: measured here, `achievements.xml` states an `unlock_condition` for 283 of 637
achievements, and for only **16 of the 119 unlockable now** — including none of the five.
This item had already named the answer ("the wiki's requirement where the file has none") and
it is now built: `AchievementRef::Known.hint` became `condition`, resolved from the file first
and the wiki second, and **all 637** carry a line. The rename was the point — widening `hint`
in place would have changed the meaning of a field two screens print under the label "indizio
del gioco:".

---

## B34 — A linked concept with no id is called a `Pickup`, and that name was the whole confusion (implementation, `wiki`) ✅ closed on 2026-09-14

**Closed on `feature/concept-not-pickup`**, after being read to be tagged and turning out not to
be finished. `Target::Pickup` is **`Target::Concept`**, the word `Resolution::Concept` and
`Inline::Concept` already used for the same thing — `inline.rs` defines it as *"a wiki page the
game gives no id"* — so the chain now reads one way through instead of changing its mind at the
generator. The key follows: `pickup:` is `concept:` in `corrections.json` (49 rows) and in the
regenerated `requirements.json`, because leaving the old prefix in the data file would have kept
the lie exactly where a human reads it.

**The measurement the 2026-09-13 correction asked for, and what it actually said.** Of the 49
targets the variant holds, **4 are in the wiki's pickup table** and 45 are not — `Hard mode` (38
uses), `Completion Mark` (19), `Greed Donation Machine` (12), `Donation Machine` (10), down to
`bed`, `rock` and `technology`. The first reading of that split was wrong and is worth keeping:
it is **not** "4 pickups and 45 non-pickups". `Coin`, `heart`, `pills` and `Blue Flies` are
pickups by any account; they are simply not rows of that Cargo table, which holds the tarot
cards under their formal names. The split is by table membership, and the honest reading is the
one that made the name: the 49 have **nothing in common except being wiki pages with no id**, and
that is what `Concept` says and `Pickup` did not.

**The two transformation verdicts kept their row and lost their reason.**
`transformation:Guppy` and `transformation:Beelzebub` read *"a transformation is three items, and
the model can't say 'N of these'"* — false since the transformations sub-project landed:
`requirements.json` gives both `at_least: 3`, and `Target::Transformation` resolves through
`threshold()` and never reaches `from_verdict`. **The rows stay**: `verdict_required` is `true`
for every target by decision, and removing the two turns
`every_target_that_needs_a_verdict_has_one` red — run and restored, not reasoned about. They now
say they are required of every target and never read.

**The 13 `concept:` references are untouched and must stay that way.** They are real requirements
this model cannot express — `ending`, `Bestiary`, `tainted character` — and the count going down
would be the bug, not the progress.

**What the rename cost, and what caught it.** 28 files, the wire type among them: `Target` is
generated into `ui/src/lib/ipc/types.ts`, and the frontend's `assertNever` turned every one of
its nine reading sites into a compile error rather than a silent empty branch. `dataset/wiki.json`
and `requirements.json` were **regenerated, never edited** — `pnpm wiki:build` then
`pnpm graph:rules`, both offline and neither needing the game. One test had been documenting the
mismatch it asserted past: `only_unreducible_targets_reach_the_inventory` failed with
*"a concept has no id"* while pinning the key `pickup:Hard mode`.

**Everything below is the entry as it stood**, including a `Done when` that belongs to the plan
that was thrown away — it asks for the filter the 2026-09-13 correction forbids.

> **Corrected on 2026-09-13. Half of this entry is closed and the other half is wrong.**
>
> - **The 4 `transformation:` references are done.** The wiki's sixteen transformation pages
>   are in the dataset, their item sets and counts travel in `requirements.json`, and
>   `Requirement::Threshold` answers them against the profile. Nodes 65, 161, 178 and 352 no
>   longer carry a hand-written `Verdict::Unknown`. Spec
>   `docs/superpowers/specs/2026-09-13-transformations-design.md`.
> - **The 13 `pickup:` references are NOT noise, and the filter below must not be built.**
>   Checked against the sentences they come from: node 69 reads "unlock all non-DLC secrets
>   and **endings**", 324 "Collect every entry in the **Bestiary**", 276 "as every character
>   (**tainted character**)". They are real requirements the model cannot express, which is
>   precisely what their `Verdict::Unknown` records. Dropping them removes each node's only
>   uninterpreted requirement, so the node stops being `Partial` and reads **available now** —
>   and the count of uninterpreted references falls, which looks like progress. The rule
>   drafted below ("only a concept the pickup table knows") was also far wider than this
>   entry asks: 45 of 49 pickup targets dropped, `Hard mode` (38 uses) among them.
>
>   What is actually wrong is the **name**: `Target::Pickup` is what a linked concept with no
>   id becomes, and calling it `Pickup` is what made "pickups that are not pickups" look like
>   the problem. The remaining work is that rename in `wiki`, with its own measurement.
>
> Everything below is the entry as written on 2026-09-12, kept for the reasoning.

Logged 2026-09-12, left out on purpose by
`docs/superpowers/specs/2026-09-12-graph-mark-requirements-design.md` §6: that sub-project
took the graph's uninterpreted references from **195 to 17**, and these are the 17.

They are not requirements nobody judged. They are **words that became targets**: the
generator walks a wiki sentence's inline tree and `Inline::Concept` promotes a linked
concept page to `Target::Pickup`, which is right for *Red Heart* and wrong for *collect*.

### What we already have

- The full list, measured against the real catalogue — 13 `pickup:` and 4
  `transformation:`, and the same walk prints it:

  | target | nodes it holds | |
  |---|---|---|
  | `pickup:ending` | 4 | a word, not a thing |
  | `pickup:Collect` | 3 | and `pickup:collect` separately, 2 more |
  | `pickup:Bestiary` | 2 | |
  | `pickup:collection` | 1 | |
  | `pickup:tainted character` | 1 | a class, not a target |
  | `transformation:Guppy` | 2 | a real transformation, three items behind it |
  | `transformation:Beelzebub` | 2 | the same |

- `crates/graph/src/generate.rs:26` is where the promotion happens, and the only place that
  has to change: the filter belongs to the **generator**, not to the curation.
- Every one of the 17 currently carries a hand-written `Verdict::Unknown` in
  `corrections.json` — so the graph is honest about them today, and this task is about
  stopping the noise at its source rather than fixing a wrong answer.

### Why it is not a five-minute change

`requirements.json` is a **committed artefact**. Changing the generator means regenerating
it with `pnpm graph:rules`, and the diff has to be read rather than trusted: the same filter
that drops `collect` must not drop a concept page that is a real target. The `derived`
discipline of `crates/wiki` is the model — the generated file and its inputs travel
together, and a test keeps them from drifting.

Note the two families are **not** the same problem. `pickup:` is noise and the answer is to
drop it. The two transformations are real: Guppy and Beelzebub each sit behind a count of
items (three Guppy items, three fly items), which is a threshold the model cannot say — the
same shape as `Verdict::Unknown { reason: "three Guppy items" }` already records. They may
stay unknown and that is a decision, not an omission.

### What's missing

1. Decide, per target, which of the two families it is — noise to filter, or a real
   requirement the model cannot express. Seventeen rows, read once.
2. Filter the noise in `generate.rs`, at the promotion, with the rule stated in code rather
   than a list of words.
3. Regenerate `requirements.json`, **read the diff**, and check that no target that used to
   resolve stopped resolving. `crates/graph/tests/coverage.rs` already asserts that a node
   with no typed reference is still a node the wiki has: that is the guard, and it has to go
   red if the filter is too wide.
4. Remove from `corrections.json` the verdicts that no longer have a target, so curation
   does not keep answering questions nobody asks.

### Done when

The uninterpreted references are 17 minus the ones judged to be noise, every remaining one
is a `Verdict::Unknown` with a reason that says *why the model cannot express it* rather
than *what the word was*, and the regenerated `requirements.json` is committed alongside the
generator change with the coverage test green.

---

## B35 — What a node unlocks links to its page too (implementation, `ipc` and `ui`, small) ✅ closed on 2026-09-13

Logged on 2026-09-12, while closing 3.5d. That sub-project made every **blocker** a link: a
requirement carries `page: Target | null` and the badge's menu opens it. The other half of the
same row is still text — Unlock's "Cosa sblocca" column and the Plan's queue rows draw an
`UnlockTarget` (an item, a character, a boss, a challenge) with no way to read about it.

What exists: `crates/ipc/src/wiki_target.rs`, one function from a catalog record to its wiki
`Target`, already used by search and by the requirements; `pageLocation()` on the frontend;
the `dropdown-menu` primitive and `WhyMenu.vue`.

What's missing: `page: Option<Target>` on `UnlockTarget` (four variants, the same rule — `Some`
only when `Dataset::entry` answers), the TypeScript mirror, and a decision about the gesture.
A target is a single thing, not a group, so a menu of one may be the wrong shape here: the
name itself could be the link. That decision belongs to the first look at 3.5d, not before it.

Closes when: Unlock's "Cosa sblocca" cell and a queue row open the page of what they name, with
the same one gesture (click navigates, Ctrl opens beside), and an entry the dataset has no page
for is drawn as plain text rather than as a link that leads nowhere.

**Closed on 2026-09-13**, with the sub-project that made an achievement's wiki page its detail
(`docs/superpowers/reports/2026-09-13-goals-and-achievement-detail-report.md`). `page:
Option<Target>` sits on all four variants, `Some` only when `Dataset::entry` answers, through
the same `crates/ipc/src/wiki_target.rs` the requirements use; pinned on real data by
`what_a_node_unlocks_links_to_the_pages_the_dataset_has`.

**The decision this item asked for**: a single target is **its own link — the name is the
link**, not a menu of one. The menu shape belongs to the blocked badge, which names a group;
one thing needs no group. The rows it appears in are the profile block's "Cosa ottieni" and
the card's headline.

---

## B37 — Searching from the goal: "voglio giocare Greed Mode, cosa devo giocare?" (implementation, `ipc` and `ui`) ✅ closed on 2026-09-13

Logged and closed the same day. Spec
`docs/superpowers/specs/2026-09-13-goals-want-design.md`, plan
`docs/superpowers/plans/archive/2026-09-13-goals-want.md`, report
`docs/superpowers/reports/2026-09-13-goals-want-report.md`. Three things execution measured that
the entry below didn't know:

- **14 of the 45 challenges are named by more than one achievement.** `routes` being a list is
  not a precaution: `achievement_unlocking`'s `.find()` was hiding a second way in on a third
  of them.
- **The graph is flat, and flatter the further you get.** The deepest chain anywhere in the
  sample collection is four steps, on the *youngest* profile; on the reference profile it is
  one. A prerequisite already earned leaves the chain, so this answer gets shorter as a player
  advances.
- **The order is not `missing_chain`'s.** That returns a set in id order; the order that means
  something comes from `plan::Queue::enqueue`, which is why the preview builds a throwaway
  queue instead of sorting by hand — the preview and "metti nel Piano" are one computation.

*The original entry, for the record:*

Logged on 2026-09-13, an explicit request. Unlock reads one way today: from a node to what it
unlocks. Its four facets — state, what it unlocks *by kind*, origin, character — are all
properties of the node, and none of them is "this particular thing I want". A player doesn't
arrive with an achievement id; they arrive with a want: Greedier, The Forgotten, the D6.

**What exists.** The traversal itself is done: `graph::Eval::missing_chain(achievement,
profile)` returns exactly the ordered list of what still stands between a profile and a node,
and the queue already uses it (`crates/app/src/commands/queue.rs:142,214`) to enqueue an
achievement with its chain. `UnlockTarget` names the four kinds of thing a node unlocks and
carries its wiki page (B35, in 3.6's plan); the search index of 3.5b already resolves a typed
name to a thing; `unlockFilter.ts` keeps the facets as pure functions.

**What's missing** is the way in, and it is four things, not one:

1. **A facet on a named value, not on a kind** — "unlocks = Greedier", not "unlocks =
   character". The values run to the hundreds, so it's a typeahead, which Unlock's drawer
   doesn't have (and which B29 already wants for Collection).
2. **Target → node**, the reverse of `target_of`. It has to answer for a target unlocked by
   more than one achievement, and for a target no achievement unlocks at all — plenty are
   already yours, or come from playing rather than from a node.
3. **The answer is a chain, not a filtered table.** Filtering Unlock's rows to the chain's ids
   is the cheap half; the readable half is an ordered series of cards and one gesture to put
   the whole thing in the plan, which `enqueue` already accepts as a chain.
4. **Not every want is a node.** "Play Greed Mode" is a game mode and Boss Rush a
   room-and-event — the same two exceptions as B36. A goal search that silently returns
   nothing for Greed is worse than one that says Greed Mode isn't unlocked, Greedier is, and
   here's how. The vocabulary of wants the app accepts gets written down, not inferred from
   the catalog.

**Decide before starting:** whether this is a facet inside Unlock or the second entry point of
the Goals screen ("voglio…" beside the recommended goals). After 3.6 rewrites that screen
(B32) the second looks more likely — the answer is a chain of cards, and a table is not a
chain.

Closes when: naming a thing you want — a character, an item, a challenge, and the modes the app
has decided it accepts — gives the ordered list of what's still missing for it, each row
linking to its page, with one gesture that puts the whole chain in the plan; a want nothing
unlocks says so in the player's terms instead of showing an empty table; and the target → node
lookup is pinned by a test on the real catalog covering its three cases — one node, several,
none.

---

## B38 — Three pickup quotes ship an undecoded HTML entity (implementation, `wiki`, small) ✅ closed on 2026-09-14

**Closed on `feature/decode-entities`.** `&comma;`, `&colon;` and `&apos;` join the closed list
in `inline.rs`; item 469 reads `:(`, item 601 "Tears up, you feel forgiven", trinket 138
"t's broken"; `grep -c '&[a-zA-Z][a-zA-Z0-9]*;' dataset/wiki.json` is **0**, and the rebuilt
dataset travelled in its own commit, four lines in it.

**The work was narrower than this entry describes, and the reason is worth keeping.** A closed
list already existed — `fn entity` in `inline.rs`, with `nbsp`, `times`, `amp`, `lt`, `gt`,
`quot`, `ndash`, `mdash` — and it was missing exactly the three that ship. So this was never
"add a decoder", it was **three rows in a list that was already the right shape**, and the entry
prescribed building the thing that was there. Reading the code before the entry is what found
that.

**What the entry asked for and was right about**: an entity outside the list is counted now, in
`Diagnostics::unknown_entities`. The text is **kept** rather than dropped — it is the wiki's
content, and `&` is an ordinary character in it — so the counter is the whole of the fix: nothing
counting them is how the three shipped. A run is only counted when it is *shaped* like an entity
(a letter then alphanumerics, or `#` and digits); without that an ordinary `&` in prose
("R&D; more") would fill the counter that exists to show a real gap. On the snapshot it is empty,
and a unit test shows it able to speak rather than leaving that to trust.

**One thing found on the way, and it is not ours**: trinket 138's quote reads
`t's broken9Reroll your dest` — truncated at both ends with a stray digit in the middle. The raw
page is intact (6240 bytes, a well-formed infobox) and says exactly that, so the parser is
faithful and **the wiki's own page is corrupt**. Registered as **B44** rather than guessed at.

---

Logged on 2026-09-13, found by `crates/ipc/tests/wiki_agrees_with_catalog.rs`: of the quote
disagreements between the wiki and the game, this is the only one that is **ours**. Every
other one is the wiki being behind the game — five items still carrying the pre-Repentance
wording ("penetrative shot" for "piercing shots"), a plural, and TMTRAINER's deliberately
corrupted string.

### What was measured

`dataset/raw/pages/` holds **36 HTML entities over 7 pages**, five distinct: `&nbsp;` (29),
`&times;` (4), and one each of `&comma;`, `&colon;`, `&apos;`. Only **three reach
`dataset/wiki.json`**, and all three are pickup quotes:

| entry | in the dataset | should read |
|---|---|---|
| item 469, Depression | `&colon;(` | `:(` |
| item 601, Act of Contrition | `Tears up&comma; you feel forgiven` | `Tears up, you feel forgiven` |
| trinket 138, 'M | `t&apos;s broken…` | `t's broken…` |

The other two never arrive: `&nbsp;` and `&times;` sit in text the parser already discards.

### Why the wiki writes them

They protect characters that template syntax would otherwise eat — a comma or a colon
inside a template argument, an apostrophe against italic markup. MediaWiki decodes them when
it renders; our inline parser passes them through as literal text, so they reach the screen
as `&comma;`.

### What it needs

Decoding in `crates/wiki/src/inline.rs`, on a **closed list of the entities actually seen**,
with anything outside the list counted in `Diagnostics` rather than passed through. Not a
general HTML-entity decoder — the input is wikitext, not HTML, and a decoder that also ate
`&amp;lt;` would be inventing a rule nobody measured. Not a regex over the finished string
either: the decoding belongs where the text is read, or the same entity comes back the next
time a field is added.

### Closes when

The three entries read as `:(`, `,` and `'`; `grep -c '&[a-zA-Z][a-zA-Z0-9]*;' dataset/wiki.json`
is zero; an entity the list does not cover is counted and visible instead of shipped; and the
rebuilt `dataset/wiki.json` travels in its own commit, as every regenerated artefact does.

---

## B40 — A transformation's infobox has no rows (implementation, `ui`, small) ✅ closed on 2026-09-14

**Closed on `feature/small-three`** (`c4dfee6`). The card draws the three rows, `contributors`
linking like any other target.

**Measuring the dataset before drawing changed the design, and that is the part worth keeping.**
The entry asks for three rows and says only that `requires` must say nothing rather than "3" when
it is `null`. On the sixteen pages: `requires` is `null` on **one** (Adult, whose transformation
is taking three pills and not picking up items) and `target` is empty on **fourteen**. `InfoboxRow`
draws "nessuno" for an empty value — true of a character with no starting items, **a claim nobody
measured** about a transformation. So a row is drawn only where the page filled it, and a page
that filled none of the three carries no card at all — which is what the suppressed variant's own
comment said an empty card would do.

The decision lives in `hasRows`, a pure function with its test, because nothing in `ui/` mounts a
component: there is no `@vue/test-utils` and no DOM environment in the suite, so a rule left in
the template is a rule no test can see.

**And it cannot be looked at yet**, which is how **B46** was found the same evening: `pageKey` and
`categoryOf` still answer `null` for a transformation, so no route in the app opens the page this
card draws. The card is finished and unreachable; what is missing is not in this entry.

**Needs:** nothing, then a window — the three fields are in the embedded dataset and on the wire already; only the card is missing.

Logged 2026-09-13, found by N7: generating the contract added the `transformation` variant to
`Infobox`, and `WikiInfobox.vue` had no case for it. Before that the union did not carry the
variant at all, so the switch reached `assertNever` — a transformation page's infobox **threw**
rather than degrading.

The card is not drawn for now, which is the honest state and not the intended one: the variant
carries `requires` (how many contributors are needed, `null` when the page does not say it in a
form we can read), `contributors` (the items and trinkets that count, in page order) and
`target` (what the transformation acts on). All three are in the dataset and on the wire, and
nothing puts them on screen.

### Closes when

A transformation page draws a card with those three rows, `contributors` linking like any other
`Target`, and `requires` saying nothing rather than "3" when it is `null` — the Rust comment on
that field records why a default would be invisible against the pages that do say it.

---

## B43 — Four screens virtualize a list under Unlock's name (implementation, `ui`, small) ✅ closed on 2026-09-14

**Closed on `feature/small-three`** (`0f4e6aa`), both halves as the entry demands. `VirtualRows`
owns the scroll box, the total height and the window; the four screens lose 26 lines each and
bring only their rows. The columns stay two files — N3's reasoning is untouched. The tokens say
*virtualized list*: `--spacing-unlock-body` is `--spacing-virtual-rows-body` and `--unlock-total`
is `--virtual-rows-total`.

**The rename was checked in the built CSS, not in the source** — a utility nothing references
generates nothing, and the grid would have collapsed in silence. What is still only argued and
not seen: that the four screens scroll as they did. Nothing in the suite draws them.

**Needs:** nothing, then a window — the scaffolding and the two tokens are `ui`; that four screens still scroll with the rows they had is what only a window says.

Logged on 2026-09-14, measured while closing N3. N3 declares `UnlockTable.vue` and
`CollectionTable.vue` as **staying two on purpose** — they draw different columns, and one
component with a column table would be a worse file than the two it replaced. That holds, and
the measurement behind this entry does not contradict it: of their 84 lines, 57 differ. What the
other 27 are is the point.

### What was measured

The shared part is not columns, it is the **scaffolding around them**: `useScaledRows`, the
`scroller` ref, the `visible` computed, the `--…-total` style binding and the three nested divs
that give a virtualized list its scroll box and its absolutely-positioned window. The composable
that does the arithmetic was factored out long ago; the template around it was not.

And it carries a name that stopped being true. **`--spacing-unlock-body` and `--unlock-total`
are read by four screens** — `CollectionTable.vue`, `SearchResults.vue`,
`WikiCategoryList.vue` and Unlock itself. A token named after one screen and used by four is the
same fault N3 spent a sub-project on, one layer down: the Collection asking for
`max-h-unlock-body` reads as a mistake to anyone who has not been told it isn't.

### What it needs

Both halves or neither, because the rename alone would leave the duplication and the extraction
alone would carry the wrong name into the shared file. A `VirtualRows.vue` (or a slot on one) that
owns the scroll box, the total height and the window, taking the rows and giving back the visible
ones; and the two tokens renamed to say *virtualized list* rather than *unlock*.

### Closes when

No screen's table repeats the scroll-box scaffolding, no token named `unlock` is read outside
`screens/unlock/`, and the four screens still scroll with the rows they had — which is the part a
test cannot say and `pnpm ui:dev` can.

### What it is NOT

A merge of `UnlockTable` and `CollectionTable`. The columns stay two files; N3's reasoning for
that is unchanged and this entry does not reopen it.

## B44 — The `'M` quote looks broken because it is meant to (analysis) ✅ closed on 2026-09-14, not a defect

Opened and closed the same hour, and kept because the wrong diagnosis is the useful part.

Trinket 138 (`'M`) ships the quote `t's broken9Reroll your dest` — missing its opening, a stray
digit in the middle, cut short at the end. Found while closing B38, it was filed as *the wiki's
own page is corrupt*, on the evidence that the raw page is intact (6240 bytes, a well-formed
infobox) and says exactly that on line 4.

**That was wrong, and the answer was in the same file, forty lines further down.** The page's
Trivia says it:

> The description is a combination of Broken Remote's, Dataminer's and The D6's descriptions,
> "It's broken", "109", and "Reroll your destiny". This may be because it triggers when an active
> item is used (like Broken Remote), is a glitch-themed item (like Dataminer) and it rerolls the
> active item (like D6).

So `t's broken` is the tail of *It's broken*, `9` the tail of *109*, and `Reroll your dest` a
truncated *Reroll your destiny*. The trinket is named after the Generation I Pokémon glitch `'M`
— the page says that too — and carries the `glitch` nav tag. **Looking broken is the content.**
The snapshot is faithful to the page, the parser to the snapshot, and the app should show it
exactly as it is.

### What it produced instead of a fix

`the_glitch_themed_trinkets_quote_is_meant_to_look_broken` in `crates/wiki/tests/real.rs`, which
pins the string and carries the wiki's explanation in its doc comment. It was shown able to fail:
replacing the expectation with the plausible-looking *"It's broken. Reroll your destiny"* turns it
red. **A distorted string with no guard invites exactly one wrong edit**, and the next reader
would have made it with the best of intentions.

### The lesson, which is why this entry stays

The repo already knew the shape. **B38's own second paragraph** lists "TMTRAINER's deliberately
corrupted string" among the quote disagreements that are not defects — one paragraph above the
work being done when this was filed. Isaac has text that is meant to look broken, it is written
down, and it was still read as corruption.

Two rules came out of it, both of which this project already holds and neither of which was
applied here: **read the whole page before calling it corrupt** — the infobox was read and the
Trivia three sections below was not — and **a thing that looks wrong on data we did not write is
a hypothesis, not a finding**, until the source is asked. It was the owner who asked it.


---

## B45 — Four characters have no page in the snapshot, and nothing notices (implementation, `wiki-snapshot`) ✅ closed on 2026-09-14

**Closed on `feature/plural-characters`**, the same evening it was opened. A kind names its
templates, plural; `{{infobox characters}}` is split where it is extracted into the two ordinary
character infoboxes it stands for, following the template's own source — `dlc`, `description`,
`unlocked by` and `hidden` are shared, everything else including `id` belongs to one form. The
refetch brought **34 character pages instead of 30**, the dataset **40 forms instead of 32**, and
the eight that were missing — Jacob, Esau, The Forgotten, The Soul, Tainted Forgotten, Tainted
Soul, Tainted Lazarus, Dead Tainted Lazarus — are pages now.

**Both guards landed, which is the half this entry said was worth more than the four pages.**
Every character the repo names in `dataset/corrections.json` must have a page, and every Cargo
row with an id must have one too — 1610 rows checked across items, trinkets, achievements,
challenges and transformations, with a floor just under it so a table that stopped being read
cannot make the test pass by checking nothing. And the mechanism that hid all of this is counted:
an `{{infobox …}}` whose template name is in no kind was skipped in silence, so a page could be
downloaded and parsed into **zero entries** without a word. It now lands in
`Diagnostics::unknown_infoboxes`.

**What the doing said that the entry did not know.** The graph was right the whole time: the
regenerated `requirements.json` differs only in `snapshotAt` and `maxRevid`, because those 47
references were resolving by id through the corrections map while the pages did not exist. A
missing page never broke anything that a test was watching — it only meant a row a screen draws
with a target whose page cannot be opened. And the four pages cannot state the second form's
parent at all: the plural template has no `parent` parameter, so `player.json` knows something
the page cannot say. The cross-check reports that silence apart from disagreement, with the four
names pinned in page order.

**Needs:** nothing — the snapshot's own index says it, and closing it is one `pnpm wiki:fetch`
against the wiki, which no part of the app ever talks to.

Found on 2026-09-14 while closing B42's first half, and not by looking for it. The cross-check
between `player.json` and the pages could compare 32 named forms and left **8 rows out**. Those
eight are not eight facts, they are one: **`Jacob & Esau`, `The Forgotten`, `Tainted Forgotten`
and `Tainted Lazarus` have no page in `dataset/raw/pages/character/`** and no entry in
`dataset/raw/index.json`. `player`, a Cargo table rather than a page fetch, still knows all of
them.

### What was measured

- `dataset/raw/pages/character/`: **30 files**; `dataset/wiki.json`: **32 characters** (Judas and
  Lazarus each carry a second infobox, for Black Judas and Lazarus Risen).
- The four missing names are in `dataset/corrections.json` with their ids — 16, 19, 29, 35 — so
  **a reference to them resolves**; what does not exist is the page it resolves to.
- `crates/graph/rules/requirements.json` holds **47 references** to those four ids: 17 to The
  Forgotten, 15 to Jacob & Esau, 8 to Tainted Forgotten, 7 to Tainted Lazarus. Every one of them
  is a requirement a screen draws, with a target whose page the wiki screen cannot open.

### The cause, measured the same evening

`wiki-snapshot` enumerates a kind's pages with `generator=embeddedin` over
`Template:Infobox character` (`pages_url`, `PageKind::template`). A page that states its character
another way is not *missed* by that query, it is **not in it** — which is why the fetch reports no
error: `Pending` only fails on a page the server listed and never delivered.

The wiki was asked what those four pages transclude, and all four answer the same thing:
**`Template:Infobox characters`, plural**, plus `Template:Infobox characters/infobox`. Not the
singular the fetch enumerates. The plural template holds **two forms in one infobox**, the second
one's parameters suffixed ` 2` — from `Tainted Lazarus`:

```
{{infobox characters
 | name 2         = Dead Tainted Lazarus
 | id             = 29
 | health         = {{hearts|red=3}}
 | health 2       = {{hearts|soul=2}}
 | collectibles   = {{i|Flip}}
 | collectibles 2 = [[File:Collectible Flip 2 icon.png|20x20px]] [[Flip]]
}}
```

Which closes the arithmetic exactly: the four pages are **the four that carry two characters** —
Jacob & Esau, The Forgotten + The Soul, Tainted Forgotten + Tainted Soul, Tainted Lazarus + Dead
Tainted Lazarus — and 4 pages × 2 forms is the 8 `player` rows the cross-check could not compare.
Nothing is missing from the wiki; one template name is missing from our query.

**So the work is larger than adding a template to the list.** The pages the singular template
brings carry one form each, and where a page holds two — Judas and Black Judas — they are two
separate `{{infobox character}}` blocks, which `extract_infoboxes` already handles. The plural
template is a shape the parser has never seen: **one block, two forms, told apart by a ` 2`
suffix on every parameter**. Both halves are needed, and the second is where the test-first work
is.

### Closes when

The four pages are in the snapshot **and** the fetch can no longer lose a character silently: the
set of characters the repo knows (`dataset/corrections.json`, `player.json`) is checked against the
pages fetched, and a name with no page fails the snapshot rather than waiting for a cross-check
written for something else to trip over it. The same guard belongs to every kind that has a Cargo
table to be counted against.

---

## B46 — A transformation has a page and no way to open it (implementation, `ui`, then a design decision) ✅ closed on 2026-09-14

**Closed on `feature/transformation-pages`.** `pageKey` writes `transformation:1`,
`categoryOf` answers the seventh category, and the IPC index carries the sixteen pages — it was
listing **six kinds of seven**, which no test noticed because the test that checks the index
against the dataset summed the same six.

**The seventh category was not a decision in the end, it was forced.** `pageLocation` needs a
category as much as a key, so the alternative to adding one was leaving the pages shut. What the
entry framed as a design call — a card in the sidebar or not — had already been answered by the
mechanism.

**And opening them made them askable, which they are not.** `wantable` and `wantLocation` derived
from `pageKey` on purpose: "has a wiki page" and "the graph can grant it" were the same set of
kinds, and the comment said a second list would be a second answer to the same question. It is not
the same question any more — nothing unlocks a transformation, you collect three items — so
`canBeWanted` is its own switch now, and a `Target` variant added later breaks the build there.
**The test that caught it was already written and already right**: it lists a transformation among
the hits `wantable` must drop, and it went red the moment the page key appeared.

**What the window added, for the third time today:** the wiki's own intro sentence lists what the
copy holds and stopped at achievements. Nothing in the suite reads that sentence.

**Needs:** nothing, then a window — the dataset answers already and the whole gap is in `ui/`;
whether the wiki's sidebar grows a seventh category is a design call, not a code one.

Found on 2026-09-14 while looking at B40's card in a window, and it is the reason that card could
not be looked at. Two pure functions in `ui/src/lib/wiki/` still say a transformation has no page:

```ts
// pageKey.ts — "The four kinds the dataset has no page for get no key"
case 'stage': case 'room': case 'concept': case 'transformation':
  return null
// category.ts — categoryOf, same four
```

`pageLocation` needs both, so it returns `null` and **no route in the app can open a
transformation page**. `WikiCategory` has six values and none of them is transformations, so the
sidebar has no door either.

### Why the premise is stale rather than wrong

It was true when it was written. The wiki-search spec of 2026-09-12 says it in as many words —
*"stages, rooms, pickups and transformations have no page (`Dataset::entry` returns `None` by
construction), so they have no key"* — and **the transformations sub-project of 2026-09-13 made
it false**: sixteen pages entered the dataset and `Dataset::entry` gained
`Target::Transformation { id } => self.transformations.get(id)`. Rust answers; the frontend still
refuses to ask. The two `null`s are not a bug in either file, they are a premise nobody went back
to after the fact it rested on changed — and the `assertNever` that guards those switches cannot
see it, because the variant is handled, just handled as *nothing*.

**B40's card is the visible cost**: it was written, tested and merged on 2026-09-14, and nothing
in the app can reach a page that would draw it.

### Closes when

`pageKey` and `categoryOf` answer for a transformation, `pageLocation` builds a location for one,
and opening a `transformation:` reference draws the page with B40's card. The seventh sidebar
category is the design half: sixteen pages are a small list and the wiki's landing counts them
already (`packPages` has a `transformations` count that nothing displays), so the choice is
whether they get a card of their own or stay reachable only by link and by search.

---

## B48 — An empty category says "no page with this name" when nothing was searched (implementation, `ui`, small) ✅ closed on 2026-09-15

**Closed on `fix/empty-category`.** The decision is a pure function — `emptyList(total, query)` in
`ui/src/lib/facets/emptyList.ts` — and `WikiCategoryList.vue` only draws its answer. A category whose
unfiltered count is `0` now says **"Questa categoria non ha pagine."** (`wiki.emptyCategory`, added
to both locales), and the *Azzera la ricerca* button appears only where there is a search to clear.

**The two halves are independent, which the entry did not say.** The sentence follows the total, the
button follows the query, because they answer different questions: an empty category is a fact about
the data, and a query is the only thing a reset can undo. That is what settles the corner the closing
criterion does not name — a query typed *into* an empty category: the sentence still says the
category is empty, because no other name would have answered either, and the button is still there,
because the box has something in it. Discriminating on the query alone would have kept the wrong
sentence in that corner; discriminating on the total alone would have left a typed search with no
way back.

**Not seen in a window** when it was written. Five tests pinned the states, and the suite,
typecheck, lint, format and scan were green; `pnpm ui:dev` was not run, so the fixture's empty
transformations category was never actually looked at.

**Both halves closed the same day (evening).** What this entry reported as still live — the same
shape in `CollectionScreen` and `UnlockScreen` — is fixed, and in the Collection it was **not
theoretical**: a machine *without the game* gets the view with `items: []` and a `noCatalog`
diagnostic, so the list said "Nessun oggetto con questi filtri." under a banner explaining that
the game is missing. `emptyList` moved to `ui/src/lib/facets/emptyList.ts` and takes its two
message keys, with `isFiltering` in place of the query, because here a reset clears the facets
too.

**And it was looked at**, on `?catalog=none`: with the Collection's opening filter the sentence is
right and the button clears the two chips the toolbar shows; cleared, the sentence stays and the
button is gone. The wiki's empty category still says it is empty with no reset after the move.
Unlock's is a **guard** and says so — a machine with no catalog still receives every node, counted
as unread, so the state is not reachable there today.

**Needs:** nothing — `WikiCategoryList.vue` and one message; seeing it wants `pnpm ui:dev` with the
fixtures, where a category is empty for a reason the real dataset never has.

Found on 2026-09-14 while looking at B46's new category. With the design pack's fixtures the
transformations list is empty — the pack has no such pages, which the fixture says in as many
words — and the screen draws **"Nessuna pagina con questo nome."** with an *Azzera la ricerca*
button, while the search box is empty and nothing was ever typed.

It is the same fault the Collection was corrected for: **what was never read must not be drawn as
"not found"**. A list with `0 / 0` and no query is not a search that failed, it is a category with
nothing in it, and the two want different sentences — one of them offers a button that undoes
nothing.

Reachable only where a category can be genuinely empty. That was nowhere until B46 added the
seventh one, and in the shipped dataset it is still nowhere: sixteen transformations always
answer. What it costs to leave is a wrong sentence in front of anyone running the frontend on
fixtures, which is every design pass.

### Closes when

An empty list with no query says the category is empty and offers no reset; an empty result *with*
a query keeps the sentence and the button it has. Both states are in `pnpm ui:dev`, so both can be
seen without the game.

---

## B49 — A block-level template reaches the screen as its own source (implementation, `wiki`) ✅ closed on 2026-09-15

**Closed with no contract change, and that is the finding.** The entry had carried since
2026-09-08 the claim that the rest needs *"a way to say «a template wrapping blocks», and that is a
`Block` variant"* — the sentence that made the second half a design decision and kept it waiting
for one. It is false, and what says so is a census rather than an argument: every template in
`dataset/raw/` that opens on one line and closes on another, by family and by the shape of what it
holds. Seventeen spans, four families, and each family already has a shape the contract can say.

- The two **`X synergy`** templates (6 + 1) open **on a list item**, always, and every line of
  their content is a `**` line: that is `ListItem { inline, children }`, which has existed since
  the first parser. The wrapper is re-closed at the end of its sentence — so the inline pass reads
  the single-line shape it already models, and one place keeps building the *"with what"* label —
  and the lines below stay the children they already were.
- All nine multi-line **`{{bug|…}}`** sit under `== Bugs ==`, which the tree carries as
  `SectionKind::Bugs`, and the single-line case had been dropped inline since `CONTENT_WRAPPERS`
  existed. Modelling the multi-line one would have said the same thing twice.
- **`scroll box`** (one use, The Lost's seeds) is `column list`'s family. It was not in the count
  above because nobody had enumerated the spans — the families were known from the offenders they
  left behind, which is a different list.

**Raw template syntax 35 → 9**, and the 9 are the genuine text: eight `<math>` formulas and
Keeper's `and}}` typo. Nothing is left that the parser could have understood.

**Two numbers were wrong in this entry and are corrected here**, both of them counts nobody could
have noticed being wrong: multi-line `{{bug|…}}` is **9 spans**, not 4 (4 was the count of the
*offenders* they left, which is neither the same list nor the same size), and the genuine
remainder is **9**, not 8 — the 8 was measured on 2026-09-08 and a formula arrived after it, under
an assertion pinned at `<= 35` that could not see its own remainder drift.

**One rule the fix needed and the entry did not know**: the pre-pass may only touch a template
that **spans lines**. 538 of the 547 `{{bug|…}}` close on the line they opened on, most of them
inside a list item, and moving one of those onto a line of its own cuts the item in two — a pass
that repaired one family by breaking five hundred. `a_wrapper_that_closes_on_its_own_line_is_left_where_it_is`
is that fence.

*The history that led here, kept:*

**`column list` is gone** (`fix/column-list`): it is unwrapped into the list it already holds,
before the line-by-line pass, because the pass cannot see a template that spans a dozen lines and
`parse_template_at` can. Beelzebub's page lists its flies. Raw template syntax **87 → 35**, one
family exactly, and `Diagnostics::orphan_closers` **50 → 0** in the same move: the lone `}}` lines
it counted were that wrapper's closers and no longer exist to be dropped.

**What is left is the half that is not layout**, and it is the half that needs the contract:
`Book of Virtues synergy` 6, `Book of Belial synergy` 1, multi-line `{{bug|…}}` 4 — templates whose
content is prose, not a list, so unwrapping them would lose what they say. Plus the 8 that are
genuine text and never go to zero. The `Block` variant this entry asks for is for those.

**Two newlines decided the fix, and the second one broke a test before it was right.** A
parameter's value arrives trimmed, so the content's own newline has to be put back or the first
`**` lands on the line the wrapper opened on; and adding one *after* the content unconditionally
leaves a blank line where the wrapper closed — which flushes the list, the very cut this pass has
a rule against, arriving from the other side. The test that caught it was written for that rule in
September and is now load-bearing for a change it never saw coming.

*(The `**Needs:**` line is gone with the closure: a closed entry carries no tag. It read "nothing —
the wikitext is committed, the defect is in `blocks.rs`, and the last step is a contract decision
about one `Block` variant", and the last clause is the half that turned out not to exist.)*

Reported by the owner on 2026-09-14, from Beelzebub's page in the running app: where the list of
contributing enemies should be, the page prints

```
{{column list | width = 15em | content =
Pooter
Super Pooter
…
}}
```

**The defect was known and counted; what was new is that it is now in front of a reader.** It is
the 75 offenders `text_nodes_carry_no_raw_template_syntax` pins in `crates/wiki/tests/real.rs`: a
template whose content is block-level opens on one line and closes several lines below, and
`blocks.rs` walks the wikitext line by line, so `parse_template_at` never sees it as one template.
`column list` is 51 of them, `Book of Virtues synergy` 6, `bug` 4.

Until B46 those offenders lived on item pages, in sections a reader reaches after the fold. The
transformation pages put one at the top of a short page, where it is the first thing you read.

### What it needs

The parser has to be able to say *a template wrapping blocks*, which is a new `Block` variant and
therefore a change to the contract the design is built on. `column list` is pure layout and could
be dropped whole; `{{bug|…}}` is content and the crate already models it in the single-line case.
Deciding which of the two shapes each family takes is the design half.

### Closes when

No text node in the dataset carries `{{` or `}}` that the parser could have understood — the
threshold in `text_nodes_carry_no_raw_template_syntax` falls to what genuine text leaves behind (8
as of 2026-09-14: the `<math>` formulas and one wiki typo) — and Beelzebub's page lists its
enemies as a list.

---

## B50 — A transformation has no picture (analysis, then `ipc`) ✅ closed on 2026-09-14, measured

**Measured on the machine with the game, and the answer is that it stays without one.** The
measurement is written into `icon.rs` beside the decision, which is what this entry asked for.

**How it was measured, after the first instrument turned out to be mute.** A `grep` over the packed
archives finds nothing for `transform` — and nothing for `gfx/items` either, which is certainly
there: **the archives index paths by hash, not by name**, so that search could never have answered.
The dictionary is `samples/filelist.txt` (18,789 paths), and the truth is `ResourceSet::read`,
which was asked directly.

**What the game holds.** 115 paths mention a transformation, and they are two families:

- the **costume** Isaac wears, `gfx/characters/costumes/transformation_*.png` — read from
  `afterbirthp.a`: `transformation_adulthood.png` is 3,770 bytes and `transformation_bookworm.png`
  3,234, so the files are real and reachable;
- an **animation** per transformation, `n020_transformation mushroom.anm2` through
  `n034_transformation_spiderbaby.anm2`, also read from `afterbirthp.a`.

**What it does not hold: an icon.** A costume is the layer a character wears, not a portrait of the
transformation, and an `.anm2` is an animation — every other target we draw is a single sprite or a
crop of a sheet.

**And the names are the game's own, not the wiki's.** Twelve of them for sixteen pages, with holes
in the numbering (029–031 are absent from the dictionary): `mushroom`, `angel`, `mom`, `poop`,
`drugs`, `evilangel`, `iwata`, `baby`, `bob`, `bookworm`, `adulthood`, `spider`. Guessing which of
the sixteen each one is — is `mom` *Yes Mother?*, is `evilangel` *Leviathan*, what is `iwata`? — is
precisely the kind of naming this repo has paid to undo twice (sections 3 and 6, a mark's bit 2).
**Serving a picture would start from that guess**, so it is not served.

### What would reopen it

A source that states the mapping rather than suggesting it — the game's own `PlayerForm` enum
through REPENTOGON, or a file that names both — plus a crop convention per sheet, the way the marks
and the character heads already have one. Then it is an implementation task and not an inference.

**Needs:** the game — the question is whether the user's own copy holds anything to draw, and only
`unpack` over `samples/packed` can answer it.

Reported by the owner on 2026-09-14: the sixteen transformation pages draw no sprite anywhere —
not in the category list, not on the page, not in a search row. `icon.rs` groups
`Target::Transformation` with stages, rooms and concepts as targets with no icon, which was true
when nothing could open one.

**Whether it can be fixed at all is unmeasured.** No wiki image is ever shipped (the package
carries none by decision), so a picture would have to come from the game's own archives. Whether
those hold anything for a transformation — a costume sprite, an icon, an animation — was **not**
established: a `grep` over the packed archives found nothing, and then found nothing for
`gfx/items` either, which is certainly there. The index is compressed and the instrument is mute,
so that search says nothing at all. The answer wants `unpack`, which reads the index properly.

### Closes when

Either a transformation carries an `iconUrl` served from the user's copy like every other target,
or `icon.rs` keeps it at `None` with the measurement written down — what was looked for, in which
archive, and what was found — so nobody greps for it a second time.

---

## B51 — The sentence that says how you become a transformation is thrown away (implementation, `wiki`, small) ✅ closed on 2026-09-14

**Closed on `feature/transformation-preamble`**, the same evening it was reported. The preamble
is kept for this kind alone and lands ahead of the infobox's `description`. Adult's page now
reads *"Adult is a transformation added in The Binding of Isaac: Afterbirth †, turns Isaac into an
adult upon taking three Puberty pills. +1 Red Heart container."*

**The order was measured, not assumed.** The infobox's `description` restates the Effects section
on all sixteen pages — Guppy's is empty — and on none of them says how the transformation happens,
so it goes second and nothing is dropped. The space between two sentences written in two places
belongs to neither, and is put in by the parser.

**And the entry was half right about the cause.** The missing sentence was one half; the other was
mine, from the same afternoon: B40's `hasRows` decided whether the transformation's three rows had
anything to say, and was used to decide whether **the card** is drawn — while the card also holds
the description and what unlocks it, the two facts that live on the entry. Adult's description went
dark along with the rows it does not have. `hasCard` is a second question now, asked separately,
and the rows keep their own rule.

**Needs:** nothing, then a rebuild — the fix is in the parser and travels with a
`pnpm wiki:build`.

Reported by the owner on 2026-09-14, on Adult: the page says nothing about how the transformation
happens. The wiki does say it — *"turns Isaac into an adult upon taking three Puberty pills"* — in
the page's **preamble**, and the parser drops preambles by design: *"it's the 'X is a passive
item…' sentence that `catalog` already covers"* (`page.rs`).

**For a transformation the preamble is not that sentence.** It is where the wiki states the
requirement, and the parser already reads it: `transformation::requires` finds "Pick up 3 …" in
exactly that text and keeps the digit. It keeps the number and throws away the sentence that
carries it.

Fifteen of the sixteen survive that, because their requirement comes back as structured data —
`requires: 3` and the contributors list, which B40 now draws. **Adult is the one where it does
not**: no count the parser can read, no contributors, so after B40's rule (a row only where the
page filled it) the page carries the effects, the notes, and nothing about pills.

### Closes when

A transformation's entry carries its preamble, and Adult's page says how you become an adult. The
rule stays what it is for every other kind — this is a kind whose preamble is content, not a
repetition of the catalog.

---

## B52 — What `n` means in a `{{dlc|…}}` code, and the 1832 spans waiting on it (analysis, then `wiki`) ✅ closed on 2026-09-15

**Answered: `n` means *not in* — removed from that edition on.** The query cost one `?action=raw`
and returned more than the entry hoped for, so nothing below had to be inferred. Report —
`docs/superpowers/reports/2026-09-15-dlc-ranges-report.md`.

`Template:Dlc/format` names every code in prose, which is where the meaning is written down:
row 7 is `nr`, `alt=(except in Repentance and Repentance+)`, titled **Removed in Repentance**.
And `Template:Dlcset` is the **whole dictionary** — a `#switch` from thirty codes to a five-bit
mask, with the bit order stated in a comment of its own:

```
<!-- <Repentance †><Repentance><Afterbirth †><Afterbirth><Rebirth> -->
 |  1 | na       =  1 <!-- 00001 -->
 | 24 | r        = 24 <!-- 11000 -->
 | 31 | n | x |  = 31 <!-- 11111 -->
 | 0 <!-- invalid string! -->
```

So a code is a **run of transitions** over the five editions, not a set of them: `r` names
Repentance *and* Repentance+, `a+nr` names Afterbirth † alone, and a bare `n` — which the corpus
never uses and the old parser read as Rebirth — names **every** edition. Rebirth is bit 0,
Repentance+ bit 4; outside the thirty the wiki itself answers `0`, which is why `Editions::parse`
returns `None` there rather than tokenizing.

What the closure changed, all measured on the committed snapshot:

- **`unknownDlcCodes` 1832 → 0.** All seventeen distinct codes the corpus uses are in the switch.
- **The infobox was wrong too, and nothing said so**: the same splitter read its `dlc` parameter,
  and 1078 of the 1083 values came out too narrow — `r` lost Repentance+ on 531 pages, `a+` lost
  three editions on 292, `a` four on 254. Tonsil's `a+nr` gained Rebirth and Repentance outright.
- **Abyss was never a contradiction.** The wiki narrows a span by its page (`{{context test}}`),
  so `nr+` on an item that exists from Repentance names Repentance — *removed in Repentance+*.
  847 of the 4831 spans narrow this way; the page's context is the **first** infobox's, because
  `{{page dlc}}` carries an `{{assert once}}`.
- The reading is pinned against 720 live rows by
  `the_cargo_dlc_integer_is_the_infobox_code_through_the_wikis_own_switch`: the Cargo `dlc`
  integer *is* `{{dlcset}}`'s output. That also closed the open question in
  `crates/ipc/examples/dlc_mask.rs` — Blue Cap's mask is 31, "no range declared", not "exists in
  Rebirth".
- It opened **B53**: `Diagnostics::merge` was dropping `unknown_infoboxes`, and with that fixed
  the dataset reports three pages carrying `{{infobox monster}}`.

The entry as it was written, kept because its reasoning is what the query confirmed:

**Needs:** nothing to measure the corpus, **one query** to answer it — the wiki's own
`Template:Dlc`, which only `wiki-snapshot` may ask. Everything below was measured on the committed
`dataset/raw/` on 2026-09-15.

Found while wiring `{{bug|dlc=…}}` into the edition it declares. The arm read the positional
argument and never a named one, so 204 of the 547 `{{bug|…}}` showed a defect of one edition to
every reader; fixing that meant reading a `dlc` code, and reading a code meant finding out that
> **The denominators below are the lowercase spelling only**, measured 2026-09-15 in the round-3
> document check. `{{dlc|…}}` occurs **4831** times in `dataset/raw/`, of which 4168 are `{{dlc`
> and 663 are `{{Dlc` — and `template.rs` lowercases a template's name before matching, so the
> parser has always read all 4831. The counts here and the 2434/1734 split are a subset, and the
> conclusions drawn from them are not affected: the 1690 empty edition nodes were counted in the
> built dataset, which has no such gap. Worth recording because the same family of entries counts
> `{{bug` in **both** cases (547, of which 393 lowercase) and this one in one.

**this parser understands 2434 of the 4168 `{{dlc|…}}` uses and silently mis-handled the other
1734**.

### What was shipping, and is not any more

Each unreadable code opened an `Inline::Edition` whose `only` was **empty** — a span declaring
itself valid in *no* edition. **1690 of them were in `dataset/wiki.json`**, out of 4928 edition
nodes: one in three. `WikiInline.vue` draws no badge for an empty `only`
(`v-if="token.only.length"`), so the reader saw the sentence with nothing to say which edition it
belongs to, and nothing anywhere recorded that a code had been dropped.

Since 2026-09-15 `Out::close` unwraps such a frame instead of emitting it — the words are kept,
the node is not — and `dlc_codes` counts the code. `meta.diagnostics.unknownDlcCodes` now carries
**1832** of them: `nr` 1190, `nr+` 235, `na+` 96, `a+nr` 84, `anr` 77, `na` 75, `rnr+` 43, `ana+`
21, and five rarer ones. (1832 against 1734 occurrences in the wikitext is not a disagreement: a
page belonging to two forms is read once per entry.)

### Why the existing splitter is not the answer

`Dlc::parse_codes` splits the **infobox** parameter, where concatenated codes are the set of
editions an entry exists in, and it would happily turn `nr` into `[Rebirth, Repentance]`. Applying
it here would have shipped 1734 labels nobody measured, and the corpus says they would be wrong:

- Every edition code appears **both bare and with a leading `n`** — `r` 1687 / `nr` 1155,
  `r+` 446 / `nr+` 207, `a+` 165 / `na+` 149, `a` 134 / `na` 69 — which is not what a set looks
  like.
- A bare `n` appears **zero** times in 4168 uses. If `n` were Rebirth, an inline marker for
  Rebirth would exist somewhere.
- **Abyss settles it.** The item exists only in Repentance — `dlc = r` in its own infobox — and
  its page carries a line marked `{{dlc|nr+}}`. Read as a set, that line is valid in Rebirth,
  an edition where the item is not.

So `n` modifies the code beside it, and every observed string decomposes cleanly under that
reading (`anr` = a, nr; `a+nr` = a+, nr; `rnr+` = r, nr+; `nar` = na, r). **What it modifies is
unmeasured** — "new in" and "not in" both fit the shape and mean opposite things, which is exactly
why this is an entry and not a patch.

### Closes when

`Template:Dlc` has been read and `n` is named from it, in writing, with the source quoted; the
codes are split accordingly; and `unknownDlcCodes` falls to what genuinely unknown codes leave
behind. If the answer turns out to be "not in", note that the sentences currently shown
unqualified are shown to the *wrong* readers, which raises this above a labelling task.

---

## B53 — Three pages carry `{{infobox monster}}` and the parser skips them (analysis, then `wiki`) ✅ closed on 2026-09-15, not worth a shape

**Closed the same evening, and the reason is not "it is minor".** Each of the three boxes was read
where it sits, and **all three sit inside something the page already loses, or that the wiki
itself does not draw**:

| page | where the box is | what that means |
|---|---|---|
| `collectible/Blood_Puppy` | the **foot of the page**, after `{{nav\|…}}`, marked **`hidden = yes`** | the wiki does not render it: it is there to register the entity in Cargo |
| `collectible/My_Shadow` | at the head of `== Friendly Charger ==` | a level-2 section with an unrecognized title, discarded whole |
| `character/Tainted_Jacob` | at the head of `== Dark Esau ==` | the same, and it is 28 lines of real behaviour |

So **the infobox is the wrong half to rescue.** Reading one would put a box on screen — a name, an
id, a one-line `behavior` — for a subject whose text the app has thrown away, or one its own source
hides. A card with no page behind it is worse than no card.

And Dark Esau's `behavior` is not even a loss on its own terms: *"Charges at Tainted Jacob. Upon
collision, transforms him into The Lost for the remainder of the floor"* is a summary of what
`== Dark Esau ==` says at length, and of two sentences the page's kept text already carries.

**What the three boxes were pointing at is real, and it is a section, not an infobox** — see
**B54**. The counter stays, because a fourth page adopting the template is still something to be
told about; it now reads 3 for a reason that is written down.

**Needs:** nothing — the pages are in the committed `dataset/raw/`.

Logged on 2026-09-15, while closing B52. The counter that reports it — `unknownInfoboxes`, added
on 2026-09-14 by B45 so a page whose infobox has no kind would be **loud** — was shipping `{}`,
because `Diagnostics::merge` never mentioned it: a page's counters are merged into the snapshot's,
and a field the merge forgets reads zero however often it fires. `merge` destructures its argument
with no `..` now, so the next one breaks the build instead.

With it merged the snapshot reports `infobox monster: 3`:

| page | what the box describes |
|---|---|
| `character/Tainted_Jacob` | **Dark Esau**, `id = 866`, `is mini-boss = yes` — the entity that chases Tainted Jacob |
| `collectible/Blood_Puppy` | the familiar itself as an entity, `id = 802`, `hidden = yes` |
| `collectible/My_Shadow` | the shadow it spawns, `id = 23`, `subtype = 1`, with `base hp` |

Three infoboxes, and none of the three pages is lost — each also carries the infobox of its own
kind, so the entry exists and only the monster's own box is dropped. What is dropped with it is a
`behavior` sentence and, for Dark Esau, the only structured statement the snapshot has about an
entity the game gives no achievement to.

**This is not `Infobox entity`** (B47's 3801 occurrences) and should not be folded into that
decision without checking: `monster` is a fourth template name, it appears on *collectible* and
*character* pages rather than on pages of its own, and its rows describe something the page is
about rather than the page itself. Whether an entry should carry a second, subordinate box at all
is the product question here, and it is the owner's.

### Closes when

Either the three boxes are read into something the app can draw, or the entry says in writing why
a monster box on somebody else's page is not worth a shape — with the count re-measured, so
"three" is not a number from a day that has passed.

---


---

## B60 — A log with no run in it, and the test that says there is no such log (implementation, `log-watch` and `test-support`, small) ✅ closed on 2026-09-16, **and its own premise was wrong**

Found on 2026-09-16 on the second machine, by looking at what it keeps rather than at what a task
needed. `Documents\My Games\Binding of Isaac Repentance\log.txt`, 4025 bytes, last written
2024-03-05: the game was launched, it played `cutscene 1 (Intro)`, it shut down.

**The entry said the guard over `samples/logs/` was too strong** — every log there has to yield at
least one run, which it called a property of the three logs that had been collected rather than of
logs. It proposed weakening the quantifier to *at least one log in the folder*.

**That was wrong, and writing the weakened version is what showed it.** With the new file in
`samples/logs/`, the weakened guard went red on this machine for a reason the strong one never
would: the folder holds exactly one log here and it is the runless one, so *"at least one holds a
run"* is legitimately false. Worse than red — on a machine that **does** have the three run logs, a
rules file that stopped matching would zero every one of them, and the weakened guard would then
report the same "no log holds a run" state. A guard that cannot tell a broken rules file from a
thin sample is not a guard.

**What was actually missing was a place to put a log that is not a run**, which this repo had
already solved once: `samples/windows/` exists so that the halves of a matched window cannot be
reached by anything walking the series. So:

| | |
|---|---|
| `samples/logs/` | logs **of runs**. The strong guard stays exactly as it was: each one must yield a run |
| `samples/launches/` | a `log.txt` in which nobody started a run. New, with `launch_samples()` and `launch_sample()` in `test-support`, declaring `sample: launches/…` like everything else |

`declared_logs_in` is now shared by both, because a helper that declares which file it used is
precisely the thing that must not exist twice with two behaviours — the crate's own docstring says
so, about itself.

**What the launch measures**, and it answers the entry's open question about the era:

- `the_launch_of_20240305_holds_one_event_and_it_is_the_intro` — **exactly one event**,
  `Ended { cutscene: 1, name: "Intro" }`. The count was derived from the file before it was
  asserted: `grep -cE` with each of the ten patterns in `crates/run/rules/events.json` finds one
  matching line, line 69, and zero for the other nine. Falsified on purpose before being trusted —
  set to 2, it reports `left: 1, right: 2`.
- `a_launch_nobody_played_speaks_and_folds_into_no_run` — the two halves that have to hold
  together: the events are **not** empty, and the fold produces **no run**. An instrument that
  reports nothing proves nothing until it has been shown able to report something.
- **The era question is answered: nothing moved.** This is the only log in the repo from before
  the `+` — Repentance **v1.7.9b**, 2024 — and the rules match it the same way. `docs/log-format.md`
  says so now instead of describing one era silently.

**And it opened B62.** The first test written for this was not the guard at all: it asked whether a
launch that was read and holds no run is *cached* as an empty list. It is not — `cached_runs`
answers `None`, which is the same value as "never folded". That is a defect in the archive's
representation, its fix is a schema migration, and it left as its own entry.
