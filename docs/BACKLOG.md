# Backlog — tasks logged, not started

Things to do that came up during the work and were deliberately set aside. Each entry says what
already exists, what's missing, and what kind of task it is (analysis or implementation). When a task starts
it follows the usual cycle: spec → plan → execution → report, and the entry here closes with a
pointer to the report.

## What an entry needs before you can start it

Every open entry carries a **`**Needs:**` line under its heading**, added 2026-09-14 after a
session opened on a machine with no game installed and spent its first minutes finding out which
tasks were even possible. It answers one question — *what does this need beyond a clone of this
repo?* — and it always says why, because a label with no reason is the thing this project spends
its corrections on.

| value | means | who has it |
|---|---|---|
| `nothing` | pure crates, the frontend, the committed dataset, packaging | anyone who clones |
| `a real save` | a `.dat` in `samples/`, sometimes of a stated era | whoever plays, once |
| `the game` | the installed game: `samples/packed`, the catalog, the art | whoever has it installed |
| `a measurement` | the game **played**: a run, a matched window, `live_probe` | one session at the machine |

`, then a window` is added where finishing it means looking at the built app. It is **not** about
the game — the app runs and degrades without it — it is about the thing no test in this repo does,
which is draw a screen. It is the second most common reason a task cannot be closed remotely, and
the session log is full of entries that say *"not seen in a real Tauri window"*.

**The tag names what it takes to finish and believe the entry, not to start typing.** Plenty of
`the game` work can be written against fixtures; what it cannot do there is be verified, and a
half of a task that cannot be verified is not a half that should be shipped.

**Snapshot of 2026-09-14**, 24 open entries — it was 26 that morning. **B34 closed because
tagging it meant reading it** and it turned out not to be finished; B38 closed; B44 opened and
closed the same hour, as not a defect. Regenerate rather than trust this list — the command
prints each open entry's heading with its tag under it, and was run before it was written down:

```
grep -E '^## B[0-9]+ —|^\*\*Needs:\*\*' docs/BACKLOG.md | grep -A1 '^## ' | grep -B1 Needs
```

- **`nothing` (14)** — B6, B11, B12, B14, B15, B17, B27, B29, B30, B39, B40, B41, B42, B43
- **`a real save` (3)** — B21, B22, B23
- **`the game` (5)** — B3, B10, B19, B33, B36
- **`a measurement` (2)** — B9, B20

The `a measurement` bucket is the same subject as *"What only a machine with the game can answer"*
in `docs/STATUS.md`, which collects the ones that are instruments rather than entries. Closed
entries carry no tag: nobody goes looking for a task that is done.

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

## B3 — Lists and search: challenges and items (implementation, after design)

**Needs:** the game — it lists challenges and items, and both come out of the catalog the game's XML builds.

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

## B6 — Multiple tabs and session restore (implementation, after design)

**Needs:** nothing, then a window — tabs and the session document are `ui` and `store`; whether a restart reopens what you had is seen, not asserted.

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

**Settled on 2026-09-13, and built**: the recommendation below was taken whole, with the
background and tray work (`docs/superpowers/specs/2026-09-13-background-and-tray-design.md`).
The flag is `resumeTabs` in `settings.json`, the tabs are `store` migration 3 — one JSON
document in one row, an object with a `version` so the sidebar width and table sizes of B27 can
join it without a migration. What this entry still holds open: a **limit on the number of tabs**
and what happens when the bar can't fit them, and **how a restored tab states a gap** when its
target no longer exists — a tab whose *route* is gone is dropped alone today, but a tab pointing
at a missing item opens and lets the screen say so, which is the case that has never been seen
happen.

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

**Needs:** a measurement — naming a section is reading bytes move while the game runs; three of the four are still `Unknown` for want of a run.

Logged on 2026-09-08, out of the B8 spike. Evidence and the full table in
`docs/superpowers/reports/2026-09-08-b8-log-spike-report.md`.

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

## B11 — Third-party licences travel with the bundle (implementation, packaging)

**Needs:** nothing — `bundle.resources` and two licence files; a bundle build says whether they travel.

Logged 2026-09-10: Vite copies only the hashed `determination-*.ttf` into `ui/dist`;
`ui/src/assets/fonts/determination/license.txt` and `readme.txt` stay in the source tree,
while the font's readme requires all files of the archive to accompany any redistribution
and CC BY 3.0 requires attribution. `crates/app/tauri.conf.json` has no `bundle.resources`.
Same gap for `dataset/ATTRIBUTION.md`, which CLAUDE.md says ships in the package. Fix when
packaging: `bundle.resources` (or the files under `ui/public/`), plus the credit line in
About (cycle 3).

---

## B12 — Design system cycle 1 follow-ups (implementation, cycles 2 and 3)

**Needs:** nothing — frontend, `cn()` and the scanner. Items 3 and 10 are contrast and state readability: they need a look before they can be decided, not the game.

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
10. A disabled segmented control loses its on-state: a disabled active `TabsTrigger` or
    pressed `ToggleGroupItem` renders exactly like an unselected one, while `Checkbox`
    keeps its tick and `Switch` its thumb position — back to design (a faint edge or
    underline would keep "which one" readable without reading as enabled).

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

## B14 — Choosing the game or saves folder by hand (implementation, cycle 3)

**Needs:** nothing, then a window — it is the *broken* chain it serves, which is this machine's normal state; the dialog plugin and a folder are all it takes.

Logged 2026-09-11, from sub-project 3.1: when the chain breaks (Steam missing, the game not
found, no saves) the profile screen says where and offers "Riprova", but not the two buttons
of `Schermate.dc.html` ("Scegli la cartella del gioco", "Scegli la cartella dei
salvataggi"). A button that does nothing is worse than none, so they wait for what makes
them work: the Tauri dialog plugin with its capability, a command that accepts a folder and
hands back a `SetupState` (the path travels inward only, never back out), the chosen folder
persisted in the settings file, and `discovery` trying it before its own search.

---

## B15 — Tearing a tab off into its own window, and back (implementation, after 3.7) — 🟡 built on 2026-09-13, **not yet measured on the machine**

**Needs:** nothing, then a window — the one open question is whether WebView2 keeps delivering pointer events outside the window, and only a hand on a mouse answers it.

**Built on `feature/drag-and-windows`, ahead of 3.7 rather than after it** (spec
`docs/superpowers/specs/2026-09-13-drag-and-windows-design.md`): the window port and its fake,
the handshake that seeds a newborn window, docking with the marker drawn in the target's strip,
the two closing rules, the payload-free events that make the profile, the scale and the plan the
app's rather than the window's, the hit test with its DPI conversion, the preview page, and the
gesture itself.

**What is still open is the one thing the entry said would decide it**: whether WebView2 keeps
delivering pointer events with the cursor outside the window. It needs a real window and a hand
on the mouse, and it has not been run. `lib/window/pointerSource.ts` is written as if the answer
were yes, behind an interface that is the only thing the other answer changes, with a 10-second
silence timeout that **cancels** the drag rather than landing a tab nobody released. The plan's
Task 1 is written to be run by the owner; Task 17's eleven checks wait on the same session.

Logged 2026-09-11, a product requirement from the owner: drag a tab out of the window and, on
drop, it opens in a new window; drag it back over the first window's tab strip and the two
merge again — **exactly as a browser does**.

### What we already have

- Tabs own locations (`stores/tabModel.ts`, pure and tested): open, close, move, navigate,
  "the bar is never empty". A tab is `{ id, location }`, never the view's content (B6), which
  is what makes it movable between windows at all.
- `TabStrip` drags within the strip (cycle 2's `moveIndex`); the title bar is ours
  (`decorations: false`, `lib/window/appWindow.ts` the only module that talks to the window).

### What the Tauri 2 documentation says (researched 2026-09-11)

- **A window at runtime:** `new WebviewWindow(label, { url, x, y, width, height, decorations,
  visible })`, permission `core:webview:allow-create-webview-window`; a capability whose
  `"windows"` glob matches the new label covers it without re-registration.
  ([webviewWindow](https://v2.tauri.app/reference/javascript/api/namespacewebviewwindow/),
  [capability](https://v2.tauri.app/reference/acl/capability/))
- **The cursor outside the window:** `cursorPosition()` (JS) / `cursor_position()` (Rust),
  desktop physical pixels, stable since 2.1.0, permission `core:window:allow-cursor-position`.
  ([window API](https://v2.tauri.app/reference/javascript/api/namespacewindow/))
- **Which window is under a point:** `getAllWebviewWindows()` with `outerPosition()` /
  `outerSize()` / `isMinimized()`; **no z-order API** (tauri#5656), so overlapping windows are
  ambiguous and the hit test is ours.
- **Between windows:** `emitTo(label, event, payload)` and `listen`.
- **The two traps.** `setPointerCapture()` in WebView2 is unreliable once the cursor leaves
  the control (microsoft-ui-xaml #8677, #8753), so a DOM-only drag can lose its `pointerup`
  outside the window; and `startDragging()` can't be started on a window that didn't receive
  the mousedown, so the torn-off window follows the cursor through `setPosition` from the
  origin window, not through the OS move.
- **Prior art to evaluate, not yet trusted:** `tauri-plugin-drag-as-window`
  (crabnebula-dev/drag-rs) starts a native OS drag of an element that becomes a window and has
  a `dragBack` for re-docking — which would sidestep the pointer-capture trap. Its maintenance,
  licence and behaviour with `decorations: false` have to be checked by making it answer, on
  the machine, before it enters `Cargo.toml`.

### The constraints that shape it

1. **The active profile is window-global today** (§4.1, §4.2): with two windows it becomes the
   app's, and every window's indicator and Progress screens must follow a change made in
   another. The profile store reads backend state already; what's missing is the event that
   says "it changed".
2. **B6 saves tabs**: with tear-off, a saved session is windows of tabs, not one list. Doing
   B15 after 3.7 means the persisted shape is decided once, knowing it.
3. **A tab dropped outside every window opens a window; a tab dropped on a strip joins it**;
   a window whose last tab leaves closes, and the main window never does (the bar is never
   empty is the same rule, one level up).
4. **Not verifiable on the development server**: fixtures run in one browser tab. The checks
   are a real `pnpm dev` window, with a mouse, on Windows at mixed DPI.

### Questions the task has to close

- Native drag (the plugin) or DOM pointer events with `cursorPosition` polling — decided by a
  spike on the machine, not by the documentation alone.
- The re-dock gesture: hover over the strip with a grace period (the WinUI write-up uses
  ~240 ms so a tab doesn't snap back instantly), or drop only.
- Where a torn-off tab's window opens: under the cursor at the drop, sized like the origin.
- Whether a secondary window has the navbar and sidebar, or tabs and content only.

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

## B17 — The profile screen is a welcome flow, not "Screen 0" (implementation, after design) ⏳ copy done on 2026-09-12

**Needs:** nothing, then a window — the welcome flow draws against fixtures (`?fixture=pick`); a real preview wants a real profile.

**Item 1 is done** (2026-09-12): the eyebrow "Schermata 0" and the intro that called the
screen a permanent state are gone, and the sidebar hint no longer says the app finds the game
from there. The screen now says what it is for — choose the save you are playing with.
**Items 2 and 3 stay open**: the welcome flow with a preview per save, and what remains under
Settings, are a design pass (sub-project 3.6).

Logged 2026-09-12, a product decision from the owner: the profile screen's copy describes a
settings page, and the owner wants the opposite — **on launch the app asks which save to
play with and shows a preview of it**, a welcome flow.

### What we already have

- `screens/ProfileScreen.vue` with `ScreenHeader` eyebrow `profile.eyebrow` ("Schermata 0" /
  "Screen 0") and `profile.intro` ("Non è un passaggio da attraversare una volta: è lo stato
  che decide ogni numero dell'app. Resta consultabile e modificabile per sempre."), in
  `ui/src/i18n/messages/{it,en}.ts`.
- The sidebar hint `sidebar.settingsHint` ("Da qui l'app trova gioco e salvataggi."), wired in
  `shell/sectionNav.ts`.
- The candidate list with its opaque ids, `SetupState`, and auto-selection when there is
  one save (sub-project 3.1).

### What's wrong with the copy, as decided

- "Schermata 0" is a design-file label, not something a user reads.
- "Da qui l'app trova gioco e salvataggi" and the intro are **false** as product: the screen
  isn't a permanent state page one may consult, it's the first thing the user meets and it
  should behave as a welcome, letting them pick the save and see what it holds before the
  rest of the app opens on it.

### What's missing

1. Drop the eyebrow and the intro; rewrite the sidebar hint. Copy comes from the design, not
   from the code.
2. A welcome flow: at launch, with more than one candidate, the app opens on the choice with
   a **preview** per save (the KPI strip already drawn by Progress: achievements, items,
   marks, last played); with one candidate, straight to the app as today. The flow stays
   reachable afterwards from the profile indicator, and B14's manual folders live in it.
3. The settings entry stops pretending to be this screen: what remains under "Impostazioni"
   is decided with the design.

### Done when

A first launch on a machine with two saves shows the two, each with its numbers, and no text
anywhere says "Schermata 0". Design first (`Schermate.dc.html`), then the usual spec → plan.

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

## B19 — The marked cell sits on the game's paper, not on a flat panel (implementation, `ipc` and `ui`)

**Needs:** the game — the paper sheet is a game asset, extracted from the user's copy at runtime, and the entry forbids a colour that resembles it.

Logged 2026-09-12, from the owner's review of Completion: a taken mark is drawn on a flat
light panel, and it should sit on the game's own paper sheet, the one the completion widget
draws under every symbol.

**Confirmed by the owner on 2026-09-12, with the reference**: the design's matrix cell drew
the mark on a sheet **taken from the game**, and that is the sheet to use — not a colour that
resembles it. Same rule as B33 for the achievement drawing.

### What we already have

- `marks/MarkCell.vue` paints a marked cell with `bg-mark-paper`, the token
  `--color-mark-paper: #e9dadf` in `colors.css` — a colour standing in for an image.
- The image exists on the same sheet the symbols come from: `completion_widget/paper_00.png`
  (the plain paper) and `paper_02.png` (the bloodied one, the state the game shows once a
  mark is taken; `design-export`'s `mark_symbol_fallback` describes it). B10's first item
  already measured it: 96×96 with the drawing at `x 0–84, y 3–82`, untrimmed.
- The icon protocol already crops this sheet for `mark/<column>/<tier>` (`ipc::mark_art`,
  `ipc::crop_png`).

### What's missing

1. One more address on the protocol, `mark/paper/<tier>` or the like, cropping the paper
   frame from `completion_widget.anm2` the way the symbols are cropped — the frame read
   from the anm2, not a hand-typed rectangle. Whether plain and bloodied are both used, or
   only one, is the design's call; the tier decides which.
2. `MarkCell` draws the paper as the cell's background image (a CSS variable bound by the
   template, per the frontend rules) and the symbol over it; the token `mark-paper` stays as
   the fallback colour for a machine without the game, or goes if the bars outfit covers it.
3. `mark_art.rs`'s test extends to the paper: it resolves, and to a different rectangle
   from every symbol.

### Done when

On a machine with the game, a marked cell shows the paper under its symbol, and the Kit
page's legend row shows the same. Without the game, nothing changes.

---

## B20 — "Non leggibile" leaves the matrix: close the 40 cells (measurement, then `core-save` and `ui`)

**Needs:** a measurement — one run of Mother with a Tainted character closes the 20 × 2 block; the dated series has been walked and cannot.

Logged 2026-09-12, a product decision from the owner: the matrix shouldn't say "non
leggibile". The state has to be resolved, not restyled.

### What we already have

- **40 cells** the app can't place: Mother and The Beast for The Forgotten and the 19
  Tainted characters, a 20 × 2 block in the bottom-right corner. Spacing puts them inside
  423–490 (Mother for the 14 originals starts at 423, The Beast at 457), but they are zero
  in every save collected, so `counter_index` answers `None` and the cell draws `unknown`
  (CLAUDE.md, "Counters and marks"; `docs/STATUS.md`).
- The rendering of that state: the dashed hatched cell in `MarkCell.vue`, the legend entry
  `completion.legend.unknown`, the KPI `completion.grid.unreadable`, the tooltip
  `completion.cell.unknown` ("la colonna non è localizzata per questo personaggio"), all in
  `i18n/messages/{it,en}.ts`.
- The two properties in `crates/ipc/tests/marks_real.rs` that pin every located base to the
  historical series.

### What's missing

1. **The measurement, which no code can replace**: one run of Mother, then of The Beast, with
   a Tainted character (or The Forgotten), with a backup of the save before and after. The
   cell that changes names the base, and the same three facts that pinned 404, 423 and 457
   have to hold — the achievement, the kill counter rising by one, index 188 naming the row.
   The backups in `save_backups\` are the series; `samples/` gets the two dated files.
2. Then the tables in `core-save` take the two bases, `counter_index` stops returning `None`
   for those rows, and the fixture counts in the spec (408 with 40 unknown) become 408 with
   0 — the `marks_real.rs` properties have to stay green on the whole series.
3. Then the UI drops what only those cells needed: the legend row "non leggibile" and the
   "non leggibili" KPI, whose count would be permanently zero. `MarkCell`'s `unknown` visual
   and the `nothingReadable` message **stay**: they are the degrade-never-fail path for a
   save whose counters section is missing or truncated, and that is a constraint, not a
   label to remove. What changes is that a healthy save never shows it.

### Done when

A full save shows no hatched cell and the legend has four entries; a truncated fixture still
draws every cell unknown and says why.

---

## B21 — A mark taken in multiplayer says so (`ipc` and `ui`; **two of three measured 2026-09-12**)

**Needs:** a real save — the bit only exists from the era the profile first won online, so a 642-era save is what makes the tests speak. The third measurement (local co-op) still wants a run.

Logged 2026-09-12, a product requirement from the owner: the matrix has to show whether a
mark was taken in multiplayer or alone. Today a cell knows only its level.

**Two of the three measurements this entry asked for were made the same day**, on a live
online Greed run; the third was briefly thought closed and is not. The implementation is
unblocked either way — what it draws does not depend on the local co-op answer.

### What was measured

A matched window around one online co-op run — Greed Mode, Cain, won, the whole session
inside the window — plus the dated series and the 22 folders in `online_logs\`.

1. **Bit 2 is "won online".** ✅ The run took `Greed × Cain` from 2 to 7, lighting bit 2 on
   exactly the cell its boss and character name. Across the series every date on which any
   cell gained bit 2 has an `online_logs\` session of the same day (6 of 6, over the period
   those folders cover), while ~60 marks taken on days without one gained bits 0 and 1
   only. The sharpest case is 2026-08-31: same day, same character, `Satan × Magdalene`
   3 → 7 **with** the bit and `Greed × Magdalene` 0 → 3 **without** — so it is a property of
   the run, not of the day.
2. **The personal save does record it.** ✅ Not "zero difference": 20 activity counters, the
   mark itself, and bestiary tallies 1 and 2 all moved. What did *not* move is achievements,
   items, challenges, bosses and sections 3, 5, 8, 9 — which is the half of the old claim
   that was right, and the half that made the whole claim look right.
3. **Local co-op — still open.** ❌ It was briefly called closed on the same day, on the
   argument that a local co-op win is the only thing that makes index 188 name two
   characters at once, which made 2026-07-22 and 2026-09-01 identifiable co-op days. **That
   argument is wrong**: 188 accumulates. A single win adds its bit and leaves the previous
   one standing — `2 → 6` on 2026-09-01, and the `4 → 12` the B8 spike report already
   recorded as "one thing that does not fit" — so two bits is two wins in the window, not
   two players in one run. The evidence that survives is weaker and by elimination: no day
   without an online session has produced a bit 2, and the owner confirms local co-op
   happened at least once. Settling it needs one local co-op win with a snapshot either
   side, the same protocol as point 2.

So the name is **"won online"**, narrower than the "multiplayer" this entry assumed, and
`sharedsave_*.dat` is *not* needed: that branch of the entry is dead.

### Still not closed by this

- **2026-06-29 and 2026-07-06** hold a bit 2 with no session folder. Neither is a
  counterexample — `online_logs\` starts on 2026-08-24, and on 07-06 index 188 names one
  character while two took marks, so the bit sits on a run the mask does not describe. What
  would settle it is knowing whether the game **rotates** `online_logs\` or keeps every
  session: if it rotates, the two holes explain themselves.
- The two **levels** (bits 0 and 1) are still unmeasured. B22 is that question.

### What to implement

- `ipc::marks::Cell::Known` grows the reading beside the level — the field names the bit,
  so `online: bool`, not `multiplayer: bool`.
- Tooltip and legend stop saying "terzo livello, significato non confermato" and say won
  online; the cell gets a visual from the design (a corner glyph or a second head, **not** a
  colour: colour is the level).
- Percentages stay forbidden, but for B22/B23's reason now, not because a bit is unread.

### Done when

The matrix draws "won online" on the cells that carry bit 2, the tooltip and legend stop
calling it a third level, and the reading is pinned by a test on the series — the structural
half already is, by `the_online_bit_never_stands_without_the_cleared_bit` in
`crates/ipc/tests/marks_real.rs`.

---

## B22 — Two counts per row: normal and hard, where hard implies normal (implementation, `ipc` and `ui`)

**Needs:** a real save — "hard implies normal" is logic over bits, but the two counts per row are only answerable against a profile.

Logged 2026-09-12, two product rules from the owner, one entry because they are the same
number:

1. **A mark taken on hard counts as taken on normal too.** Beating a boss on hard is the
   harder of the two, so a character who has the hard mark has the normal one whatever
   bit 0 says.
2. **Rows and footer show two numbers, not one**: how many bosses at normal, how many at
   hard, each over the readable cells.

### What we already have

- One tally everywhere: `started / readable`, started meaning "bit 0 or bit 1"
  (`ui/src/lib/completion/completionView.ts` `tallyOf`, `crates/ipc/src/marks.rs`
  `CharacterRow.started`). `MarksGrid.vue` draws it once per row, once per group header and
  once per boss in the footer; `CompletionKpis.vue` sums it.
- The cell already obeys rule 1 in its drawing: `markVisual.ts` picks the hard sprite when
  bit 1 is set, whatever bit 0 says. What doesn't obey it is the reading: `CellStatus` has
  `Normal`, `Hard` and `Both`, the tooltip says "normale e hard" for 3, and value 2 reads
  "hard" as if normal were missing.
- The observed values include a bare 2: the game does write hard without normal, so rule 1
  is a rule of ours, not a reading of the file. The count keeps the bit; the label doesn't.

### What's missing

1. **The reading**: `CellStatus.Both` goes; `Hard` means "hard, hence normal", and the
   tooltip and legend say so once ("hard" — the legend's "normale e hard" row disappears).
   `cellReading` is pure and tested: the expected values come from the rule above.
2. **The tally**: `Tally` becomes `{ normal, hard, readable, complete }` with
   `normal = cells where bit 0 or bit 1`, `hard = cells where bit 1`, so `hard ≤ normal ≤
   readable` always — a property to test on the fixtures and on `marks_real.rs`'s series.
   `complete` means `hard === readable`: the row is done when every boss is done on hard,
   which is what the game's own widget means by a full row. Whether a second, weaker
   "complete at normal" colour exists is the design's call.
3. **The IPC**: `CharacterRow.started` becomes `normal` and `hard` (a contract change,
   handed on with the mirror in `types.ts` and `summary_shape.rs`'s kin pinned).
4. **The grid**: two number columns on the right of every row, two in the group header, two
   per boss in the footer, and the KPI strip splits the same way. Layout from
   `Schermate.dc.html`, which today has one column.

### Done when

A row with twelve hard marks reads 12/12 · 12/12; a row with value 2 in one cell reads
1/12 · 1/12, not 0 and 1; the legend has no "normale e hard"; and no test derives its
expected number from the previous single count.

---

## B23 — The Completion KPIs: no "120 celle", no "40 non leggibili" (implementation, after design, with B20 and B22)

**Needs:** a real save, then a window — the tiles are what the strip above the matrix reads, and the matrix needs a profile to have a number in it.

Logged 2026-09-12, from the owner's review of the strip above the matrix
(`screens/completion/CompletionKpis.vue`, spec §"KPIs"): of the four tiles, two say nothing
the owner wants to read.

| tile today | value | verdict |
|---|---|---|
| marchi iniziati | started / readable | keeps its place, split by B22 |
| normale + hard | "120 celle", no denominator | **useless**: a count of cells with both bits, meaningful only while the bits were unread |
| personaggi completi | complete / characters | stays |
| non leggibili | "40 / 408 celle" | **not wanted**: the 40 are a gap in our tables (B20), not a fact about the player |

### What replaces them

With B22's two numbers and B20's closed cells the strip is the same three facts the rows
show, summed: **normal marks / total**, **hard marks / total**, **complete characters /
characters** — three tiles, or four with the design's fourth if it has one (the obvious
candidate is "characters complete at normal", the weaker of the two `complete`s). Each keeps
its denominator and no percentage is drawn (§5.3).

The `Unknown` tone and the `unknown` count don't vanish from the model: while a section is
missing the tiles read 0 / 0 and the alert above the grid says why (B20's rule). What goes
is the tile that puts an unreadable count on the same line as the player's progress.

### Done when

The strip on a full save has no tile whose value is a bare count of cells, and none whose
label says "non leggibili". Design first: it's the same pass that draws B22's columns.

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

## B27 — A table fills the page, or the mouse sizes it and the size is remembered per table (implementation, `ui` and `app`)

**Needs:** nothing, then a window — a table that fills the page or is sized by the mouse is judged at the size a window actually has.

Logged 2026-09-12, a product rule from the owner: a table either **scales with the page it
is open in**, or it is **resizable with the mouse**; in the second case its size is saved in
state, **per table**, so it never has to be resized again after a restart.

### What we already have

- Two virtualized tables, Unlock (`screens/unlock/UnlockTable.vue`) and the Collection
  (`screens/collection/CollectionTable.vue`), both scrolling inside a body capped at
  `max-h-unlock-body`, the token `--spacing-unlock-body: 560px`: a fixed height chosen from
  the design file, blind to the window. On a tall window the page ends with empty space
  under the table; on a short one the table scrolls inside a page that also scrolls.
- Their columns are already fluid: `grid-cols-unlock` and `grid-cols-collection` in
  `utilities.css` give the image and badge columns a token width and the text columns `fr`
  shares, so the width follows the page today. What doesn't is the height.
- One resize gesture exists, the sidebar (`shell/SectionSidebar.vue`, pointer events,
  `clampSidebarWidth`), whose width is shell state **not persisted** by decision, deferred to
  the sub-project that saves the session (3.7).
- Persistence: `settings.json` through `ipc::Settings` (one field today) — and the store,
  where a document per concern already lives (the queue).

### The two options, and the order

1. **Fill the page** first, because it is the option that needs no state: the table's body
   takes the height left under the facets and the header (`flex-1 min-h-0` on the page's
   column, the scroller `h-full`), and `--spacing-unlock-body` goes. The Plan's queue and the
   marks matrix are checked against the same rule. TanStack Virtual doesn't care: the
   scroller's height is measured, not declared.
2. **Resizable by hand** where filling isn't right (a table under other content, a split
   pane): a grip on the table's bottom edge, the sidebar's gesture generalised into one
   composable (`useResize`), a minimum and a maximum in tokens, and the height stored **per
   table** under a key that names it (`unlock`, `collection`, …), never a shared number.
3. **Where it is saved**: with the sidebar's width, in the same place 3.7 chooses for the
   session — one `layout` document, keyed by table, in the store or in `settings.json`; the
   decision is 3.7's, this entry only fixes that the table sizes belong in it. A missing or
   unknown key reads as "fill the page". The value is in device-independent pixels and is
   scaled with B26, not stored scaled.

### Done when

Unlock and the Collection use the whole height of a tall window and nothing under them
scrolls twice; a table given a grip keeps the size it was dragged to across a restart, each
table its own; and no fixed body height survives in the tokens.

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

## B29 — The Collection's filter: no "Faccette", no values with nothing behind them (implementation, `ui`, after design) ⏳ the two sight fixes done on 2026-09-12

**Needs:** nothing, then a window — one filter bar with a fold and multi-select dropdowns: a new primitive and a design pass, both frontend.

**The two things wrong on sight are fixed** (2026-09-12), on both screens: the drawer says
"Filtri" / "Filters", and a value with nothing behind it is no longer offered at all — it used
to sit there with a 0 and a disabled checkbox. **The shape the owner wants stays open**: one
filter bar with a fold for the less important controls, and multi-select dropdowns in place of
the checkbox columns, which needs a new primitive and the design pass that redraws the bar for
Unlock and the Collection at once.

Logged 2026-09-12, from the owner's review of the Collection's filter drawer
(`screens/collection/CollectionFacetDrawer.vue`): the presentation is to be redone, and two
things are wrong on sight.

- **"Faccette"** (`collection.facets`, and `unlock.facets` on Unlock) is the design file's
  word for the control, not a word a player uses. The drawer needs a name that says what it
  does — filter — or no title at all.
- **Values that match nothing are listed.** `collectionFacetOptions` offers every value of
  each facet's set, so the quality "non valutato" and the origin "non indicata" appear with
  a 0 and can't be picked (the drawer's own comment: "a value that would give nothing, and
  isn't picked, can't be picked"). The owner reads them as noise: a value with nothing behind
  it in this profile's view is not offered. Same rule for Unlock's drawer, which shares the
  mechanism (`facetLabels.ts`, `FacetDrawer.vue`).

### The shape the owner wants

- **All the filters together**, in one place, not a drawer per screen with columns of
  checkboxes: the state toggle, the search and the facets are one filter bar.
- **The important ones visible, the rest behind "mostra più filtri"** (or the like): the
  bar shows two or three controls at rest and unfolds the others on request; which ones are
  important is decided per screen in the design (for the Collection, the state and the
  quality; for Unlock, the state and what it unlocks).
- **Pool, quality, origin, kind as multi-select dropdowns**: a `Select`-like control that
  takes several values, shows the picked ones on its trigger ("Qualità · 3, 4") and the
  count beside each option, instead of a column of checkboxes. The kit has `Select`
  (single) and `Checkbox`; a multi-select is a new primitive, Reka's `Combobox` or
  `Listbox` with `multiple`, styled by the kit, with its own Kit page row.

The pure functions stay: `collectionFacetCounts` already knows which values are empty and
what picking one would give. What changes is the presentation — the options are the values
with a count, in the catalog's order, inside a multi-select — and the wording, from the
design pass that redraws the bar for both screens at once.

### Done when

Both screens filter from one bar with a fold for the less important controls; pool, quality,
origin and kind are multi-select dropdowns; neither shows a value with 0 unless it is
currently picked; and the word "Faccette" appears nowhere in the app.

---

## B30 — Where the app writes, in Settings, and movable (implementation, `app`, `store` and `ui`)

**Needs:** nothing, then a window — the folder, its size and the button that moves it are `app`, `store` and `ui`; the move is watched once.

Logged 2026-09-12, from the owner while reading the About dialog: About says the app writes
one file, `isaacdome.db`, in the app's data folder — and the owner wants to **see that folder
and be able to move it**, which is a setting, not a credit.

### What goes where

About keeps what **identifies and credits** the app: the name, the version, the fan-made
line, the licences, and the three promises — the promises are the product's contract and the
first thing a stranger should read, so they stay where a stranger looks. What moves is the
**fact about this machine**: which folder, how big, and the button that changes it. A path is
something you act on; a promise is something you read.

### What we already have

- `crates/app/src/lib.rs`: `StoreState` opens `app_data_dir()/isaacdome.db` once, lazily, and
  keeps it in managed state; `settings_file.rs` writes `settings.json` in `app_config_dir()`.
  Neither path has ever crossed the IPC, and **neither may**: a path carries the Windows
  username (CLAUDE.md, "Don't cross the IPC boundary").
- `store` with its versioned schema and two migrations, and the queue as one JSON document.
- The Settings section of the sidebar (Profile, Tabs, Appearance since 3.5c).

### The constraint that shapes it, and the way out

A path can't cross the boundary as a path — but the user has to see *where* their data is, or
the setting is a button with no subject. The way out is the one `discovery` already uses for
save files: **the view carries a hint, not a path** — the folder's display name and its parent
in short form, enough to recognise it, never the full string. The dialog that changes it is
the Tauri dialog plugin (B14 needs the same plugin), and what comes back travels **inward
only**: the frontend asks "move it here" with the handle the dialog gave, and the backend
answers with a new hint.

### What's missing

1. A **Data** page under Settings: the folder's hint, the database's size, and what is in it
   (the queue, the goals — a row each, from `store`), plus the same for `settings.json`.
2. A command that **moves** it: close the handle in `StoreState`, copy the file, verify it
   opens at the destination, write the new location in `settings.json`, and only then remove
   the old one. It is the app's own data, so the order is copy → verify → switch → delete,
   never move → hope.
3. The location in `settings.json` (`dataDir`), read at startup by `StoreState`; absent means
   the default, which is what every install has today.
4. **What happens when the saved folder is gone** on the next launch — an external drive, a
   folder the user deleted: the app says so on the Data page and falls back to the default
   rather than failing to open, and the goals and the queue read as unavailable, which is a
   state `plan` and `queue` already have.

### Done when

The Data page names the folder in words, says how large the file is, and moving it leaves the
queue and the goals intact at the new place; a folder that has gone missing is said, not
crashed on; and no full path has crossed the IPC boundary.

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

## B33 — The achievement drawing sits on the game's own backing, as the game shows it (implementation, `ipc` and `ui`)

**Needs:** the game — the backing "has to be looked for in the game's files", which is the whole point of the entry.

Logged 2026-09-12, from the owner's review of the cards: the achievement picture is drawn on
a flat colour, and it should sit on **the image the game itself puts behind it** — the
right one has to be looked for in the game's files, not approximated.

### What we already have

- `components/graph/AchievementArt.vue` draws the 263×176 achievement picture on
  `bg-mark-paper`, the same flat token B19 replaces under the marks; its own comment says
  "the way the game shows it on a note", which names the intent and not the sheet.
- The picture itself is served through the icon protocol from the user's own copy, so the
  backing can come the same way: a crop of a sheet, resolved by an anm2, addressed by the
  protocol, with the bars fallback when the game isn't there.
- The game shows an achievement in two places, with two backings: the **unlock popup** at
  the end of a run (the achievement drawing on a paper note that slides in) and the
  **achievements page of the pause / stats menu** (the drawing on a paper-like frame per
  slot). Which of the two the card should imitate is the design's call; the sheet behind
  each is the measurement.

### What's missing

1. Find, in `resources/packed`, the anm2 and sheet the game uses for the achievement popup
   and for the achievements menu — a `design-export` probe listing the candidates under
   `gfx/ui/achievement` and `gfx/ui/...menu` with their frame rectangles, so the choice is
   made on the real files. The name is recorded here once it is measured, not guessed now.
2. One more address on the icon protocol for the backing, cropped from its frame; the
   `AchievementArt` component draws it under the picture with the game's own padding, on
   every screen that shows an achievement (Next steps, Unlock, Plan, the Kit page).
3. The flat token stays only as the fallback without the game.

### Done when

On a machine with the game, an achievement card shows the drawing on the same backing the
game does, and the entry names the sheet and frame it came from.

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

## B36 — A mark and a counter say which boss they mean, and link to it (measurement, then `ipc` and `ui`)

**Needs:** the game — the ten bosses' keys come from the dataset and need none, but the counters resolve through the catalog, and that half does.

Logged on 2026-09-12, with 3.5d. `RequirementView::Mark` names a cell of the completion matrix
("beat Delirium with Cain") and `Counter` a threshold on a tally; both draw in the blocked menu
as names you cannot follow, because neither carries a page.

Why it isn't done: the twelve columns are not twelve entities. Ten of them plausibly map to a
boss the dataset has a page for, but **Boss Rush** is a room-and-event and **Greed** is a game
mode, and a column → entity table written from the names would be a curation nobody measured —
exactly what `docs/STATUS.md` keeps `Unknown` rather than guessing. The counters' labels are the
bosses' own names, which is a second, easier case: those could resolve through the catalog.

What it needs: the entity key of each of the ten bosses, taken from the same place
`wiki_target::boss` takes it — the portrait's file name — rather than from a name match; and a
decision, in words, for the two that are not entities.

Closes when: a mark's entry in the blocked menu opens the boss's page for the ten that have one,
the other two say what they are without pretending to be entities, and the mapping is pinned by
a test that reads it from the catalog rather than from a literal table.

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

## B39 — A tab carries its state between windows: filters, scroll, what it was showing (implementation, `ui`, after 3.7's shape)

**Needs:** nothing, then a window — what a tab carries between windows is `ui`; that it survives the move is seen.

Logged 2026-09-13, from the owner while checking the tear-off: *"si devono tenere anche filtri,
scroll ecc quando tratti uno spostamento di tab"*. A tab dragged into another window arrives at
the right page and **forgets everything about how it was being read** — the Unlock facets, the
search text, the sort, where the virtualized table was scrolled to.

### Why it is not a small fix

What crosses between windows today is `TabSeed` — everything a tab *is*, minus its identity. And
a tab, today, **is a location**: a route name and a query. Everything else lives somewhere that
is not the tab:

- the **facets and the search text** are the screens' own `ref`s, recreated when the screen
  mounts (`screens/unlock/`, `composables/useSearch.ts`);
- the **scroll offset** belongs to the DOM element, and to `@tanstack/vue-virtual`'s measurement
  of it;
- the **view stores** (`stores/graph.ts`, `collection.ts`, `wiki.ts`) are per window, and keyed
  by nothing: two tabs on the same screen already share them.

So "a tab keeps its state" means **a tab owns its state**, which is a different shape from the
one the shell has had since 3.1. It is the same shape 3.7 needs in order to save a session — a
tab that can be written down and read back — which is why this waits for that decision rather
than inventing a second one.

### What it probably looks like

1. A tab's state becomes an object it owns: `{ location, view? }`, where `view` is a small,
   serializable record a screen declares for itself (facets, query, sort, scroll offset).
2. A screen reads it on mount and writes it back as it changes — through one composable, so no
   screen invents its own storage, and so the shape is uniform enough for 3.7 to persist.
3. `TabSeed` needs no change at all: it is `Omit<Tab, 'id'>`, so the day a tab holds its view the
   view crosses windows with it. That property was built in on purpose (2026-09-13) and this is
   the case it was built for.
4. Scroll is the awkward one: an offset only means something against a list of the same length,
   so it is restored **after** the data is there, and a list that changed underneath keeps the
   top rather than guessing.

### Done when

A tab dragged into another window comes back showing what it was showing: the same facets, the
same text in the search, the same sort, and the same place in the list — and the same is true of
a tab that survives a restart, because it is the same mechanism.

---

## B40 — A transformation's infobox has no rows (implementation, `ui`, small)

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

## B41 — Starting with Windows, so no run is lost to a launch the app missed (implementation, `ipc`, `app` and `ui`, after design)

**Needs:** nothing, then a window — the registry, a pure `launch_intent`, and a switch. Closing it wants an installed build, a logout and a login.

Logged on 2026-09-14, from the owner: *"aggiungiamo opzione avvio al lancio in impostazioni in
modo che a prescindere da quando apro il gioco IsaacDome può essere sempre aperto e leggere tutte
le run"*. Design written the same day —
`docs/superpowers/specs/2026-09-14-autostart-design.md`. It is §11 of the background design coming
due: starting with Windows was deferred there *"to the same conversation as M4's watcher"*, and
the watcher has landed.

### What already exists

Everything except the login entry. The app outlives its windows, the tray brings one back, a
second launch is handed to the instance already running, and `start_archive` backfills
`online_logs\sessions\` and then watches `log.txt` — backfill and live being one function is what
makes a late start harmless *within* a game launch.

### What is missing, precisely

`log.txt` is rewritten at every launch of the game, so the hole is narrow and real: **a game
launch followed by another game launch, with the app never having run in between.** Play, quit,
play again, open IsaacDome — the first session is gone and nothing can bring it back. The switch
closes that one case, and the prose under it has to say that rather than promise "no run is ever
lost".

### The three decisions the design takes

1. **The registry is the only source of truth.** `ipc::Settings` gains no field. `is_enabled()`
   reads `StartupApproved\Run` as well as `Run`, so the app can see the user disable the entry
   from Task Manager's Startup tab — a mirrored boolean in `settings.json` would report "on" for a
   login that never happens.
2. **A login launch is silent.** The plugin writes one argument into the Run value; `setup` builds
   the tray and starts the archive either way, and only opens a window when that argument is
   absent. The decision is `ipc::launch_intent`, a pure function, because `app` is not tested.
3. **The switch is inert in development builds.** `current_exe()` in a `pnpm dev` run is
   `target\debug\app.exe`, and a switch flipped once while testing leaves that path in the
   developer's login, surviving `cargo clean` and failing silently at every boot.

### Closes when

The Background screen carries a third switch, first of the three; an installed build turned on
and logged out of comes back with the icon in the tray, no window, and the archive already
following the log; disabling the entry from Task Manager shows as off the next time the screen is
opened; turning the switch off twice raises nothing; and a `pnpm dev` run leaves the registry
untouched.

**Not closed by the above, and not lost either**: the uninstaller does not remove the Run value —
nothing tells a running app it is being uninstalled, and there is no installer configuration in
the repo yet (`tauri.conf.json` says `"targets": "all"` and nothing more). The day that
configuration is written, an NSIS uninstall hook deletes the value from both keys. That is part of
this entry, not a new one.

---

## B42 — Two Cargo tables are downloaded, committed, and read by nothing (implementation, `wiki`, small)

**Needs:** nothing — `wiki-snapshot`'s query and `dataset/raw/cargo/`, both committed.

Logged on 2026-09-13, noticed while adding the transformations' five fields to the same
query. `crates/wiki-snapshot/src/api.rs` downloads ten Cargo tables; `Raw::load` puts seven
of them in `Tables`, plus `version`. **`player.json` and `stage.json` are written to
`dataset/raw/cargo/` and nothing ever opens them.**

### Why it is worth an entry rather than a deletion

`player` carries **`parent`** — the relation from a Tainted character to the base form it is
a variant of. That is exactly the fact behind the two identity bugs M2 found on 2026-09-12:
the game gives a Tainted character the base form's name, so by name alone 141 of 396
character references resolved to nothing and "Ultra Greedier as Keeper" picked T. Keeper.
Resolution goes by the wiki's id now, and this table is a second, independent source for the
same relation — `Infobox::Character.parent` reads it from the page, and nobody has checked
the two against each other.

`stage` carries `chapter`, which nothing needs today; the 1444 `stage:` references resolve by
name and have no entries at all.

### What it needs

A decision, not code first: either the two tables earn a reader — `player.parent` as a
cross-check on the character map, with the disagreements counted the way the transformations'
two item lists are — or they leave the query. What must not continue is the third state, a
committed artefact nobody reads and no test would notice going stale.

### Closes when

Either both tables are read by something with a test that would fail if they stopped
agreeing, or they are gone from `TABLES` and from `dataset/raw/cargo/`, with the reason in
the commit body.

---

## B43 — Four screens virtualize a list under Unlock's name (implementation, `ui`, small)

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

