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
`docs/superpowers/plans/2026-09-05-b1-fonti-effetti-report.md`. What came out of it, that the entry
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
  `docs/superpowers/plans/2026-09-05-b1-fonti-effetti-report.md` and
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
`catalog:` and `ipc:`; logged in `docs/STATO.md`, session of 2026-09-05. What came out
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
screen design: it doesn't start before that handoff.

The "search" in this entry is the **filter inside a list**; searching across the whole app is a different
matter and is **B5**. The two meet only at the point where a global result opens the list
already filtered.

---

## B4 — An item's unlock tree (this is M2)

Logged on 2026-09-05. Today the chain stops at the first step (item → achievement →
condition in English). A tree requires structured parsing of the 283 conditions into a
versioned rules file ("beat X with Y", "Beat Challenge #N", "collect N …") and the
recursive linking condition → character/boss/challenge → their unlocks. The `graph` field
in the IPC contracts is already planned as a stub for this. It's not a backlog task: it's
milestone M2, logged in `STATO.md`.

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

**Closed 2026-09-07.** No separate report: session log in `docs/STATO.md`, entry
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
`docs/STATO.md`, `docs/MIGLIORIE.md`, `docs/frontend-conventions.md`, `DESIGN-BRIEF.md`,
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
