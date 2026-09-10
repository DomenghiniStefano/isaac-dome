# Backlog — tasks logged, not started

Things to do that came up during the work and were deliberately set aside. Each entry says what
already exists, what's missing, and what kind of task it is (analysis or implementation). When a task starts
it follows the usual cycle: spec → plan → execution → report, and the entry here closes with a
pointer to the report.

---

## B1 — Item detail: primary and secondary effects (an **analysis** task) ✅ closed on 2026-09-05

Logged on 2026-09-05. Reference for shape/format: the wiki page for *False PHD*
(`bindingofisaacrebirth.fandom.com/wiki/False_PHD`).

**Closed the same day.** Report —
`docs/superpowers/plans/2026-09-05-b1-sources-effects-report.md`. What came out of it, that the entry
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
  `docs/superpowers/plans/2026-09-05-b1-sources-effects-report.md` and
  `docs/superpowers/plans/2026-09-05-wiki-dataset-report.md`.)*
- **EID** covers 100% of items and trinkets, Italian included, and has the Italian names
  that the stringtable doesn't; but it **has no license** and can't be redistributed without the author's
  consent. The Italian fandom wiki is made of empty skeleton pages.
- Decision: **dataset built at build time and shipped in the package**, one pass per release
  against the wiki (whose `robots.txt` asks for responsible traffic), optional update from
  GitHub. The implementation waits for the screen design, like B3.

Dataset implementation closed on 2026-09-05: report —
`docs/superpowers/plans/2026-09-05-wiki-dataset-report.md`.

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

## B3 — Lists and search: challenges and items (implementation, after design)

Logged on 2026-09-05. The data is there; what's missing is the IPC commands that list challenges and
items with B1's fields and the filterable grid (TanStack Table) on the frontend side. It depends on the
screen design, so it waits on the design system rather than on the handoff — which
happened on 2026-09-09.

The "search" in this entry is the **filter inside a list**; searching across the whole app is a different
matter and is **B5**. The two meet only at the point where a global result opens the list
already filtered.

---

## B4 — An item's unlock tree (this is M2) ✅ closed on 2026-09-07

Logged on 2026-09-05. **Closed 2026-09-07** — report in
`docs/superpowers/plans/2026-09-07-unlock-graph-report.md`.

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

## B5 — Global search: one thing searches all of them (implementation, after design)

Logged on 2026-09-06, an explicit request. It has to find **anything the app knows about**:
an item, an achievement, a challenge, a character, a boss, a wiki page and what's
inside its sections, a node in the unlock tree, and the screens themselves.

Two surfaces, not one: the **palette** (`Ctrl+K`, opens from any screen, results
grouped by type, enter opens) for quick access, and the **Search screen** for the
complete set, filterable by type, for when the results don't fit in a palette. The palette leads
to the screen; the screen doesn't require going through the palette.

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

### Questions the task has to close

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

## B6 — Multiple tabs and session restore (implementation, after design)

Logged on 2026-09-06, an explicit request. The shell behaves **like a browser**: several tabs
open together, you switch between them, reorder them, close them. In the
global settings, a preference: **keep tabs saved when the app closes**, and on restart they
reopen where they were.

### What can live in a tab

Any view, not just the main sections: one of the screens, but also an item's detail,
a wiki page, a node of the unlock tree. Two tabs can show the
same screen with different filters, and two items get compared side by side by switching from one
tab to the other — which is the reason the feature exists.

### The two constraints that shape it

1. **A tab saves the identity of the view, never its content.** Route plus parameters plus serializable
   view state (filters, sorting, position); never the view-model, never base64 icons, never a file
   path. On restore, the tab reloads from the backend just like on first
   launch. It's the same boundary as always: only resolved view-models and opaque ids cross the IPC.
2. **The active profile stays global, not per tab.** §4.1 of `DESIGN-BRIEF.md` establishes
   that every number in the app depends on which save is being read and that the indicator is
   a single one, persistent in the shell. Tabs with different profiles side by side would break that
   promise: switching profile updates **all** the tabs.

### Where it's saved

A declared fork, with a recommendation:

- **The flag** "reopen tabs on startup" in `settings.json` (`crates/app/src/settings_file.rs`),
  next to the other global preferences: it's a preference, and it lives where preferences live.
- **The list of tabs** in a **`store` migration**. It's structured state — one row per
  tab, with its identity serialized the way it already happens for the `TargetKey` of targets — and
  it will grow with the view types. `isaacdome.db` has a versioned schema for exactly this, and it's already the only
  file the app writes. Putting it in `settings.json` would mean evolving by hand a
  format that has no migrations.

### Questions the task has to close

- **A restored tab that points to something that no longer exists** — the user changed profile or
  edition, the wiki dataset was updated, the item doesn't exist in that version of the
  game. The tab opens **stating the gap**, it doesn't silently vanish and doesn't turn into
  a different view: it's the same principle as "unknown data ≠ zero data".
- **A limit on the number of tabs**, and what happens when the bar can't fit them anymore.
- **How a new tab opens**: from search (B5), from a wiki link, from a node
  in the tree, and with what gesture — middle click and `Ctrl+click` are the browser-like expectation.
- **What the first launch does**, when there are no saved tabs yet: it opens on *Next
  steps*, a single tab, as it does today.

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
`docs/superpowers/plans/2026-09-08-b8-log-spike-report.md`, from one real run (Judas, hard,
Mega Satan, won) watched live with a throwaway probe. Both open questions answered — **rooms
are logged**, and the flush is immediate — plus five findings the entry wasn't looking for,
of which two change M4's design: the log **announces every save write** (132 times in one
run), and the save is written continuously *during* play rather than between sessions. Also
`from pool X` lies about the starting item, and rebuilding an inventory needs `catalog`
because actives replace one another.

**And one finding that isn't about M4 at all:** the game names every chunk of the save file
as it reads it, which puts names on the four sections `CLAUDE.md` lists as "to be
identified" and contradicts two we thought we knew. That's B9 below.

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

### One finding already in hand

The loss window for log data is narrower than `docs/PROJECT.md` implies. The game rewrites
`log.txt` on the **next** launch, so the last session survives on disk until then —
verified on the dev machine, where a March 2024 log ending in
`Isaac has shut down successfully` is still intact two and a half years later. The rule is
not "the app must run while you play" but **"the app must run at least once between one
session and the next"**. An app that reads the whole current log at startup recovers the
last session; `RNG Start Seed` gives the dedup key that makes re-reading safe.

---

## B9 — Re-identify the save's sections from the game's own names (implementation, delicate)

Logged on 2026-09-08, out of the B8 spike. Evidence and the full table in
`docs/superpowers/plans/2026-09-08-b8-log-spike-report.md`.

### What we found

Loading a profile, the game prints `Reading chunk N` followed by that chunk's **name**, in
file order, for **eleven** chunks. Our table in `CLAUDE.md` has ten sections, four of them
marked "to be identified". The four have names, and two of the six we thought we knew are
contradicted:

- `Kind::PerChar` (3, 14 x 4) — the game calls it **Level Counters**, not one value per
  original character. Fourteen stages fits at least as well as fourteen characters.
- `Kind::Unknown5` (5, 7 x 1) — **Mini Bosses**.
- `Kind::CardsPills` (6, 104 x 1) — the game calls it **Bosses**. The catalog has **103**
  bosses; 104 cells fit that better than cards and pills.
- `Kind::Unknown8` (8, 27 x 4) — **Cutscene Counters**.
- `Kind::Unknown9` (9, 2 x 4) — **GameSettings**.
- `Kind::Bestiary` (10) — the game reads **two** chunks here, Special Seed Counters then
  Bestiary Counters. Our section 10's header declares `count=80, f2=320` and the parser
  hands it 11,016 bytes: the payload holds both, with no second header between them.

The four positions we are sure of (1, 2, 4, 7) all agree with the game's names, which is
what makes the rest worth acting on.

### Why it isn't just a rename

**A log line is not a measurement.** The names are strong evidence about *what the game
thinks it is reading*, not proof of what each cell means, and the existing labels may have
come from watching bits flip in M0. Section 6 is the sharp case: if it is Bosses, then
`SaveDiff.cards_pills` — a public field — has been reporting boss kills under a card's name
since the day it was written.

### Measured on 2026-09-08 (evening): steps 1 and 2 are answered

The analysis below used a source nobody had opened yet — **`online_logs\sessions\`**, 21
folders each holding a `log.txt` and two profile snapshots. `persistentgamedata1_end.dat`
is **our** profile (it matches the dated sample of the same day, counter for counter);
`persistentgamedata1_begin.dat` is **the other participant's**, a second real Repentance+
profile that grows from **52 to 105 achievements** across the series. A second profile at
the opposite end of the progression is exactly what these questions needed. The
`sharedsave_*.dat` files in the same folders are **not** in our format — no `ISAACNGSAVE`
magic anywhere in them — so the co-op shared profile stays unread for now.

- **Section 6 is Bosses. Settled, and not by the log.** 104 cells, boss ids 0..103, and the
  catalog has 103 bosses. On the beginner profile **56 of 104 are set, and the 48 that
  aren't are exactly the late and alt-path roster**: The Lamb, Mega Satan, Delirium, Mother,
  Dogma, The Beast and the whole Repentance list from Reap Creep to Cadavra. A player who
  never went down the alt path, read straight off the cell ids. On our own profile 97 of 104
  are set and the seven gaps are *The Matriarch*, *Cadavra*, **Raglich** — a boss that is
  unused in the game — plus four that look like variants recorded under another id
  (*Ultra Greed*, *Ultra Greedier*, *Mom (Mausoleum)*, *Mom's Heart (Mausoleum)*).
  So `SaveDiff.cards_pills` has been reporting boss encounters under a card's name, exactly
  as feared.
- **Section 3 is stages, and "one value per original character" is refuted.** **Index 0 is
  zero in every save we hold** — impossible for a table whose first row would be Isaac, the
  most-played character. Indices 1..12 carry 309, 298, 199, 206, 162, 208, 107, 109, 19, 67,
  50, 137, and index 1 (309) sits right on the number of runs the profile has. In a matched
  window — one game launch, the live save against the last backup — the indices that moved
  are **exactly** the stages the log declared with `Level::Init m_Stage`: 1, 2, 3, 4, 5, 6,
  7, 8, 10, 11. Fourteen cells = stages 1..13 plus an unused 0.
- **Section 8 is cutscenes, with one cell left over.** Same matched window: the log played
  cutscene 1 and cutscene 19, and section 8 moved at **index 19** (+1) and index 2 (+1).
  Index 19 is the identity mapping. Index 2 is the profile's largest cell (110) and rises
  once per launch, which is also how often the log prints `playing cutscene 1` — so either
  it is cutscene 1 under an off-by-one that index 19 contradicts, or it counts launches. One
  more solo run with a known ending separates them.
  *Careful with co-op*: of fifteen windows taken from the session folders, five had a
  cutscene in the log and no movement in section 8 at all. All five are run endings played
  in **online co-op**, whose progression goes to the shared profile. Co-op windows are not
  evidence about the personal save — which is itself worth knowing.
- **Section 10: the proposed split at byte 320 is wrong**, and so was the list that
  replaced it. `count` is 80 and `f2` is 320 in every file, but the payload is not two
  chunks meeting there — the keys run **straight across index 80 without a
  discontinuity**, so byte 320 falls in the middle, not on a boundary. That much holds.
  The reading that followed — "0..19 are zero, 20..24 are five small counters, and from
  index 25 the rest is a sorted key → count list of 1,364 pairs" — does not.

  **Measured on 2026-09-09, on four saves across two editions.** The section is
  **self-describing**, and reading it as one list is what made it look unsorted:

  ```
  words[0..19]   twenty zeros
  words[20]      11            constant in every save
  words[21]      the total, exactly the sum of the four sizes below
  words[22]      4             how many tallies follow
  then 4 x ( id, size, size/4 records of (key, count) )
                 ids 4, 2, 3, 1 in that order in every save
  ```

  A size is in units of two bytes and a record is eight, so `size / 4` records. Inside a
  tally the keys are **strictly ascending and each entity appears once**. Read as a single
  list from index 25, the same bytes show three descents and 445 repeated keys — those are
  the three intermediate `(id, size)` headers and the fact that the same entity is counted
  in several tallies. Both artefacts vanish when the boundaries are read from the file.

  So "20..24 are five counters" was really `11`, the total, the tally count, and then the
  first tally's own `(id, size)`.

  **One word is left over** after the last tally, in every save: 11,343 (Jan 2024) rising
  to 29,725 (Jan 2025). It is not slack — it grows with the profile. It nearly went
  unnoticed: the throwaway script that mapped the layout had an off-by-one that consumed
  it, and it came back only because the Rust reader disagreed with the script.

  The key decodes as before: **`(type << 20) | (variant << 8) | subtype`** — `0x00A00000`
  is type 10 variant 0, `0x02600100` is type 38 variant 1 — and the largest type, 951, is
  inside the game's entity range. What the four tallies count is **not known**: they hold
  the same entities with different numbers against each one. The triple is the same one
  `crates/wiki` already indexes bosses by (`Dataset::boss_key`), so the join exists the day
  the tallies have meanings.

  **Read by `core-save` since 2026-09-09** — `Save::bestiary_tallies()`, module
  `crates/core-save/src/bestiary.rs`, eight real-data properties in
  `crates/core-save/tests/bestiary.rs`. The tallies keep the id the file gives them and are
  given no names, for the same reason sections 5, 8 and 9 are still `Unknown`.

### What is left to do

1. **The one cell in section 8**: solo run, known ending, watch index 2. Twenty minutes.
   Needs a machine with the game running, so it is the item that travels worst.
2. ~~**Then rename**, together.~~ **Sections 3 and 6 renamed on 2026-09-09**, on a machine
   without the game — everything they needed was already measured. `Kind::PerChar` →
   `Kind::LevelCounters`, `Kind::CardsPills` → `Kind::Bosses`, `SaveDiff.cards_pills` →
   `SaveDiff.bosses`, plus the tables in `CLAUDE.md`, `docs/PROJECT.md` and
   `DESIGN-BRIEF.md`. Sections 5, 8 and 9 keep their `Unknown` names on purpose: they have
   the game's word for it and nothing else, which is the distinction the enum's own doc
   comment now states.
   **The blast radius recorded above was wrong in one place.** It said `Kind` "neither
   crosses the IPC nor drives product behaviour"; it *does* cross, inside
   `ipc::SectionCount`, so a variant's name is a wire value. Nothing caught it because the
   TypeScript mirror types that field as `kind: string` rather than a union — a type wide
   enough to hide a contract change. Now pinned by `crates/ipc/tests/summary_shape.rs`,
   ten variants against ten strings, with a second test that goes red if an eleventh
   variant is added without a row.
3. ~~**Decide what section 10 becomes.**~~ **Its structure is read as of 2026-09-09.**
   `Save::bestiary_tallies()` returns the four tallies, each a list of
   `(EntityId, count)` in key order, plus what the section declared about itself and the
   one word the layout doesn't account for. The reading is checked by the invariants that
   found it — the declared total accounts for every tally, the section is consumed but for
   that word, each tally is ascending and holds an entity once — so it keeps working on a
   patch and says so when it can't.
   **What is left needs the game**, and is listed in `docs/STATUS.md`: which of the four
   tallies counts what, and what the trailing word is. Both want a matched window — play a
   run, compare the save against the backup taken before it — which is the same instrument
   that answered sections 3 and 8, and it cannot be done from samples.

### Done when

Every one of the eleven chunks has a name backed by a measurement of its own, the table in
`CLAUDE.md` has no "to be identified" rows left, and no public name in `core-save` says
something the bytes contradict.

### What it is NOT

Not an occasion to start *interpreting* the newly named sections. Naming section 8
"Cutscene Counters" doesn't oblige anyone to decode which cutscene is which; it obliges us
to stop calling it `Unknown8`.

---

## B10 — The design export pack: what the design tool had to measure by hand (implementation, `design-export`)

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

## B11 — Third-party licences travel with the bundle (implementation, packaging)

Logged 2026-09-10: Vite copies only the hashed `determination-*.ttf` into `ui/dist`;
`ui/src/assets/fonts/determination/license.txt` and `readme.txt` stay in the source tree,
while the font's readme requires all files of the archive to accompany any redistribution
and CC BY 3.0 requires attribution. `crates/app/tauri.conf.json` has no `bundle.resources`.
Same gap for `dataset/ATTRIBUTION.md`, which CLAUDE.md says ships in the package. Fix when
packaging: `bundle.resources` (or the files under `ui/public/`), plus the credit line in
About (cycle 3).

---

## B12 — Design system cycle 1 follow-ups (implementation, cycles 2 and 3)

Logged 2026-09-10, a list:

1. `cn()` doesn't register `--opacity-*` tokens (`opacity-muted opacity-disabled` both
   survive a merge) — add `classGroups.opacity` read from `theme/opacity.css`, with a
   test.
2. vue-i18n feature flags aren't defined in `ui/vite.config.ts`
   (`__VUE_I18N_LEGACY_API__` stays in the bundle, legacy API not tree-shaken) — add the
   `define` entries per vue-i18n's optimization guide.
3. Keyboard highlight contrast in Select and Command items: `secondary` #3A251D on
   `popover` #1B120E is about 1.28:1; Reka's Select gives items real DOM focus, so the
   highlight replaces the focus ring — back to design (an inset `ring` edge would fit
   "cyan means focus").
4. `Command` filters only when the search text changes; items mounted after a change
   (async palette results) leave their group hidden and `CommandEmpty` beside results;
   `CommandItem` doesn't prune its id from the group set on unmount; no tests for the
   filter; `CommandInput` always auto-focuses (make it a prop) — for the cycle 3 palette.
5. Scanner gaps: the literal-attribute check skips `position`, `align`, `side`
   (constants exist); the `dark:` pattern misses stacked variants like `hover:dark:`;
   `outline-none` on focusable elements isn't scanned; classes of the reset default
   scales (`text-sm`, `rounded-md`, `font-bold`, `shadow-*`) aren't flagged although they
   generate nothing.
6. `SelectTrigger` hover uses `row-hover`, a row's role, on a field — a `field-hover`
   role or `secondary`.
7. `Button` has no default `type="button"`: inside a future `<form>` every Button
   submits.
8. `Alert` uses `role="alert"` for diagnostics rendered with the page (assertive on
   mount); `role="status"` may fit.
9. `Progress` doesn't expose the unknown segment to assistive tech (`getValueLabel` with
   an i18n string).
