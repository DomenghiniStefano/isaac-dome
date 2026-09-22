# IsaacDome

*Project document · v4 · written at M0 complete · September 2026*

> **This is the design, not the state.** It was written when M0 was the only milestone done
> and it still reads that way: §09's architecture lists seven modules where the repo has
> sixteen crates, and §11's screens are the ones that were planned. **The state lives in
> `docs/STATUS.md`** — M0, M2 and M4 are closed, M1 and M3 are in progress — and what is open
> lives in `docs/BACKLOG.md`. Checked through on 2026-09-15; two claims inside it were not
> merely old but wrong, and are corrected in place with the correction named.

A desktop app you can ship with an installer: anyone who owns The Binding of Isaac on
Steam opens it and sees their own progress, what they're missing, and what's worth
playing tonight. No account, no telemetry, no API key.

`Tauri 2 + Vue 3` · `shadcn-vue` · `Rust parser` · `Read-only` · `Offline` · `Zero
credentials` · `Windows installer` · `Verified format`

---

## 01 — Goal

The game shows you *what you have*. The wiki shows you *what exists*. Neither answers the
question that actually matters after a few hundred hours: **which run gets me closest to
the Dead God**. IsaacDome reads the profile saved on disk, cross-references it with the
unlock graph, and ranks what's missing by effort-to-payoff ratio.

It's not a personal tool: it's an app you download, install, and that works **with no
configuration** on any Steam install of the game. This constraint — "has to work at a
stranger's house" — drives half the technical decisions in this document.

### Non-goals

- **No writing to saves.** The `.dat` files open read-only: the checksum is never
  recomputed, so there is no code path capable of corrupting someone else's profile.
- **No accounts, no backend, no telemetry.** An app that reads game files and talks to
  nobody is also the only version of this app you could actually trust.
- **No API key** asked of the user or embedded in the binary.
- **No redistributed game assets** in the installer: images come from the user's own
  copy.
- It's not a streaming overlay: RebirthItemTracker already does that well.

## 02 — Data sources

Everything needed is already on the installing user's PC. Three local files, plus two
datasets downloaded once — neither with credentials.

| Source | Where | What it gives | Solidity |
|---|---|---|---|
| `persistentgamedata<1-3>.dat` | `Documents\My Games\Binding of Isaac Repentance+\` — with Steam Cloud active, `Steam\userdata\<id>\250900\remote\rep+persistentgamedata<n>.dat` | Completion marks, achievements, item collection, challenges, bosses, 523 game counters, bestiary | **Verified** |
| `save_backups\` | save subfolder | Dated backups the game creates on its own: a historical series of the profile, spanning months | **Verified** |
| `online_logs\` | save subfolder | One folder per online co-op session: the session's full log and a profile snapshot before and after | **Verified** |
| `log.txt` | same folder as the save | Current and historical runs: items with name, character and pool, seed, floors, who killed you, endings | **Verified, but fragile across patches** |
| Game XML and sprites | `resources-dlc3\`, inside the `.a` archives | Catalog: IDs, localized names, quality, pool, item and achievement icons | **Solid** |
| `localconfig.vdf` | `Steam\userdata\<id>\config\` | Hours played, read locally without touching any API | **Solid** |
| Unlock graph | dataset versioned with the app, updatable from GitHub | The unlock conditions, which **aren't in the game files** (section 06) | **Needs upkeep** |
| Global rarity *(optional)* | Steam `GetGlobalAchievementPercentagesForApp`, `gameid=250900` | Percentage of players who have each achievement. The only `ISteamUserStats` method that needs no key | **With reservations** |

> **Project rule.** Every source sits behind an *adapter* with its own schema version. If
> a patch breaks the `.dat` layout, the app degrades: it shows what the other sources can
> still tell it and flags the unreadable section, instead of refusing to start. On a
> distributed app this isn't elegance, it's the only alternative to a flood of bug
> reports.

## 03 — The save file format

Structure derived and verified against 28 real Repentance+ saves, spread over fourteen
months. **It's not the one existing tools describe**: the signature here is
`ISAACNGSAVE09R`, while the parsers already out there expect `06R` and stop before
reading a single byte.

```
0x00   "ISAACNGSAVE09R  "        signature, 16 bytes
0x10   u32                       changes on every save, meaning unknown
0x14   first section header
...
end-4  checksum, 4 bytes         CRC32 with a custom polynomial, not identified

# section header: 3 × little-endian u32
kind    sequential 1..10
f2      "in-memory" size = count × 4
count   number of entries        # on-disk size varies by section
```

| Section | Entries | Bytes | Content |
|---|---|---|---|
| 1 | 642 | 1 | Achievements and secrets |
| 2 | 523 | 4 | Game counters **and completion marks** |
| 3 | 14 | 4 | Level counters, one cell per stage — index 0 is unused |
| 4 | 733 | 1 | Item collection |
| 5 | 7 | 1 | To be identified |
| 6 | 104 | 1 | Bosses met |
| 7 | 46 | 1 | Challenges |
| 8 | 27 | 4 | To be identified |
| 9 | 2 | 4 | To be identified |
| 10 | variable | 8 | Bestiary: four tallies over the same entities, self-describing |

The entry count **is read from the file, never hardcoded**. The proof is in the backups:
the June 2025 save declares **641** achievements, the 2026 ones declare **642**. A patch
added one, and any parser with that number hardcoded would have broken itself.

The checksum hasn't been identified, and it doesn't need to be: the app opens read-only
and never has to recompute it. It's a problem that only exists for whoever writes to the
file.

### The counters are already documented

Section 2 maps **one to one** to the `EventCounter` enum published by REPENTOGON. The
verification leaves no room for doubt: index 0 is named `NULL` and equals 0, index 4 is
`SUPER_SPECIAL_ROCKS_DESTROYED` and equals 1, index 8 is `UNKNOWN_EVENT_8` and equals 0.
Deaths, deals with the devil, Eden tokens, best streak, bosses killed: all labeled without
guessing anything.

And above all, the **completion marks live in there**, as blocks ordered by boss and by
character. They aren't counters but **bitmasks**: the observed values are only 0, 1, 2, 3,
5 and 7. Two bits are the mark's levels; the third shows up rarely and remains to be
explained.

> **What's still open.** Documented names reach index 284. From there on the pattern is
> regular — a 19-cell block per boss, with Bethany, Jacob & Esau and the 17 Tainted — and
> holds up to Hush. The tail beyond it was read on 2026-09-08: Delirium for the 19 later
> characters starts at **404**, Mother for the 14 originals at **423**, The Beast at
> **457**, and **491**/**492** are those two bosses' kills. Mother's own group of 34 was
> read on 2026-09-20, on a window where T. Eden beat Mother: The Forgotten at **437** and
> the 19 from **438**. What stays unread is **20 cells** — The Beast for The Forgotten and
> the 19 — which the spacing puts inside 471–490 but which are zero in every save collected,
> so they are drawn as *unknown* rather than guessed; one run of The Beast closes them.
> Index 385 is a counter on its own, 386–403 are eighteen cells never seen moving, and
> 493–522 is a family of counters that move several per session.
>
> Sections **5, 8 and 9** — all small — still have only the name the game prints when it
> loads a profile, which is evidence and not a measurement, so `core-save` still calls
> them `Unknown`. Section 3 was measured and renamed on 2026-09-09, and the bestiary's
> **layout** is read (see §03's table); what each of its four tallies counts is not.

## 04 — What the log gives us

The `.dat` is a snapshot: it says *what* you've unlocked, never *how*. Run history only
exists if you build it yourself, and the only mod-free source is `log.txt`, which the
game rewrites on every launch. Hence the constraint: **the app has to be running while
you play**, and persist every run into its own local archive.

The content has been verified against a real log, and it's richer than expected:

```
Adding collectible 225 (Gimpy) to player 0 (Cain) from pool treasure
RNG Start Seed: FYQ8 QQ8G (586324166) [New, 1]
Level::Init m_Stage 2, m_StageType 1 Seed 408474304
Game Over. Killed by (9.0) spawned by (84.0) damage flags (0)
playing cutscene 15 (Sheol).
```

One single line gives **the item's id and name, the character, and the pool**; the death
line gives both the entity that killed you *and* the one that spawned it; the seed
carries the run type with it (new, resumed, online); the cutscenes identify the endings.
The biggest risk flagged in v2 — a log too thin to reconstruct runs in Repentance+ — **is
closed: the Runs section is feasible**.

The strings change between versions, so the patterns live in a versioned rule file, not
scattered through the code — and they can be updated without recompiling an app someone
else has already installed.

> **A gift from online co-op.** Every online session leaves its own full log *and* a
> profile snapshot before and after in `online_logs\`. Something the app should show also
> comes out of this: online co-op uses a **separate shared profile**, which starts from
> zero and grows with the group. In the sample analyzed it went from 13 to 100
> achievements in one summer, while the personal profile stayed untouched. It's a second
> progression bar, invisible to the player today.

## 05 — Graphics and metadata

Resources live in `.a` archives inside `resources-dlc3\packed\`. Once extracted, you'll
find all the app's material in there.

| Resource | Contains | What it's for |
|---|---|---|
| `items.xml` | ID, name, quality, tags, stats the item modifies | Catalog, quality filters, "what it does to my stats" |
| `itempools.xml` | Which pools each item appears in and with what weight | "Where can I find it": treasure, boss, devil, angel… |
| `achievements.xml` | Achievements, text, graphics | Names and cards for the Unlock section |
| `challenges.xml` | Each challenge's rules and reward | Challenges screen |
| `entities2.xml` | Enemies and bosses, IDs and variants | Bestiary and "what actually kills me" |
| `stringtable` | Localized names and descriptions | UI in the user's language, with nothing translated by hand |
| `gfx\items\collectibles\*.png` | Every item's sprite | Real icons, no placeholders |
| `gfx\ui\achievement\*.png` | Achievement and mark cards | Completion matrix, Unlock section |

Extraction can't be something the user has to do: nobody is going to run a command-line
tool after installing an app. Two paths, in order — **call the official Resource
Extractor**, which already sits in the `tools\` folder of anyone's install (to be
reconfirmed on Rep+), or **implement a reader for the `.a` format in Rust**, which is
simple and removes every external dependency. The second is more work but it's the right
one for a public app.

> **A license constraint, not just an elegance one.** Shipping an installer means **no
> Nicalis asset can sit inside the package**. The app extracts from the user's own copy
> and builds a local cache: legally clean and, incidentally, lighter to download too. The
> same applies to the unlock dataset: if it's derived from the wiki, it ships with its
> license and its attribution.

## 06 — The unlock graph

This is the piece that decides whether the app is interesting or just another viewer.
**Unlock conditions don't exist in the game files**: `achievements.xml` only has name,
graphics, and a "hidden" flag, the "how you get it" is hardcoded in the engine. So the
graph — *"the D6 comes from there, and unlocks this other thing in turn"* — has to be
built externally.

It doesn't start from zero:
[IsaacUnlockFinder](https://github.com/nerstak/IsaacUnlockFinder) already has a dependency
graph covering roughly **403 out of 637 achievements**, drawn from the wiki and exported
via GraphViz `dot` → JSON. That's the bootstrap; the rest is completed with the
**MediaWiki API of `bindingofisaacrebirth.wiki.gg`**, which is open, needs no key and no
registration — one "update dataset" command instead of hours of copy-paste.

It's the only dataset the project actually has to maintain. It's also its real value: two
apps reading the same save only differ in how much they know about what the save doesn't
say.

## 07 — Unlock: facets and sort orders

One filter matters more than all the others combined: **"unlockable now"**. The graph
knows which achievements already have every prerequisite satisfied. Three states, always
visible: *done · unlockable now · blocked by N*.

| Facet | Values | Used for |
|---|---|---|
| Status | done · unlockable now · blocked | The starting question |
| What it unlocks | passive item · active item · trinket · card · pill · character · challenge · room type · pickup | "I just want new characters" |
| Character required | the 34 playable | "Tonight I'm playing Azazel" |
| Ending required | Mom · Mom's Heart · Isaac · ??? · Satan · Lamb · Mega Satan · Hush · Delirium · Mother · The Beast | Planning the run's route |
| Mode | normal · hard · Greed · Greedier | Isolating what needs hard mode |
| Shape of the effort | one run · N runs · streak · cumulative · RNG-dependent | "I've got an hour": rule out streaks and cumulatives |
| Quality and pool | 0–4 · treasure, boss, devil, angel, shop… | Aiming for items that actually change runs |
| DLC of origin | Rebirth · Afterbirth · AB+ · Repentance · Rep+ | Whoever doesn't have every DLC only sees their own |
| Global rarity | Steam percentage | "The rarest thing I can pull off tonight" |

The sort orders that matter are four: **how much it unlocks downstream** (the graph's
fan-out — doing it first is objectively more efficient), steps remaining, rarity, and the
quality of the unlocked item.

*A note of honesty on rarity: the public endpoint returns internal identifiers, and the
readable names would come from `GetSchemaForGame`, which requires the key we don't want to
ask for. So this needs a mapping table compiled once and shipped with the dataset. If it
becomes a burden, it's the first thing to cut: it's a nice filter, not an essential one.*

## 08 — The plan that updates itself

The trick is not turning it into a to-do list. **The plan is derived, not typed.** The
user adds a *goal* — "unlock Tainted Lost", "the D6" — and the app expands the
prerequisites from the graph in topological order. The save file stays the source of
truth: every time the `.dat` is re-read the app diffs it against the previous state, new
achievements tick themselves off, and the plan recomputes. It can never fall out of sync
because there's nothing to keep in sync.

The piece no other tool does, and the one that in my view justifies the whole app:
**grouping steps by run**. The graph knows every goal's constraints — character, mode,
ending — so it can say *"you close these 4 goals in the same run, with Azazel, in hard
mode, going all the way to ???"*. It's a constrained packing problem, tractable, and it
turns a list into an evening's plan.

Free corollary: when the game closes, the app knows exactly what changed, and can close
the session with a summary — *3 steps out of 7, 4 remaining*.

## 09 — Architecture

**`discovery`** *(Rust)* — Finds Steam from the registry
(`HKCU\Software\Valve\Steam`), the libraries from `libraryfolders.vdf`, the game from
`appmanifest_250900.acf`. For saves it has to cover four real cases: folder with and
without the `+`, and `rep_` or `rep+` prefix under `userdata`. Manual fallback at every
step.

**`unpack`** *(Rust)* — Reads the game's `.a` archives and extracts XML and sprites into
the user's local cache. Once, with a progress bar.

**`core-save`** *(Rust)* — Parser for the `.dat`, translated from `reference/isaac_save.py`.
Opens read-only, validates the signature, maps the sections from the metadata. No write
function anywhere in the module, by construction.
*(This read "derived from the Kaitai spec" until 2026-09-15, which named the one approach
the module's own design rejected: "no Kaitai-generated code… Alternatives discarded: Kaitai
(external tool + rigid generated code for 80 lines of parsing)".)*

**`log-watch`** *(Rust)* — File watching with `notify` and incremental reading. The
parsing rules are versioned data, not code.

**`graph`** *(Rust)* — Unlock graph: every node's status, computing "unlockable now",
fan-out, expanding a goal into a plan, packing by run.

**`store`** *(SQLite)* — A single file in the app's data folder, six tables over four
migrations: `goals` and `plan_queue` (the user's plans), `window_session`, and `sources`,
`events` and `runs` (the run archive).
*Two of the four things this line promised are not there and were never written*: **profile
snapshots** are still an open item in `STATUS.md`, and the **normalized catalog** is not in
SQLite at all — `catalog` reads the game's XML into memory at launch, because a cache would
have to be invalidated against a game that patches itself.

**UI** *(Vue 3 + Pinia)* — Dark-first, i18n from day one. Receives already-resolved data:
knows nothing about offsets or log strings.

**Rust does everything that touches disk**, Vue only ever receives already-interpreted
JSON. If the format changes tomorrow, only one module changes — and on an app installed
by other people that's the difference between a patch and a rewrite.

## 10 — Components

There's only one criterion: **the components have to be ours**. This app is 90% dense
grids, matrices and filterable lists — exactly the stuff no "complete" library gets
right out of the box, and someone else's design system would force it into their look.
Better to start from accessible primitives and dress them ourselves.

### Frontend

| Choice | Role | Why |
|---|---|---|
| **shadcn-vue** on **Reka UI** | Base components | You copy the components into the repo and they become yours: no dependency changing the look out from under you. Same philosophy as react-native-reusables on the Steam app. Active and up to date, *new-york* style. |
| **Tailwind v4** | Styling | Supported by shadcn-vue; the theme tokens become the single place dark-first lives. |
| **TanStack Table** | Unlock, Collection, Challenges | Headless sorting, facets and filters: the logic is theirs, the rendering stays ours. It's also the engine behind shadcn-vue's Data Table, so it arrives already integrated. |
| **TanStack Virtual** | Long grids | 733 items and 642 achievements with real icons don't all mount in the DOM at once. Without virtualization, the Collection stutters. |
| **Lucide** | Icons | shadcn's native set; the game's own icons come from the extracted sprites instead. |
| **Pinia** · **Vue Router** · **vue-i18n** | State, screens, languages | The Vue standard, nothing to debate. i18n is needed from day one because the app is public. |
| shadcn-vue's **Chart** component | The few charts | Here charts are the exception, not the rule: not worth a dedicated library until one is genuinely needed. |

### Backend (Rust)

| Crate | Role |
|---|---|
| `steamlocate` | Finds Steam, the libraries and the game folder, on every system. Saves us from hand-writing `libraryfolders.vdf` parsing — and it's half of the `discovery` module. |
| `winreg` | Windows fallback when automatic discovery isn't enough. |
| `keyvalues-parser` | `localconfig.vdf`, for hours played read locally. |
| `quick-xml` | The game's XML files: items, pools, challenges, entities, stringtable. |
| `notify` | The `log.txt` watcher. |
| `rusqlite` (bundled) | Run archive, snapshots, catalog. The database stays in Rust: the frontend never touches disk. |
| `serde` | The bridge to Vue. |

> **A temptation to resist.** Reading the `.dat` **doesn't need a parser combinator**
> like `nom`. The format is three integers and a slice of bytes: `u32::from_le_bytes` and
> slices are enough, and the reference Python parser is eighty lines long. Adding a
> dependency here is complexity nobody asked for.

### Dropped, with reasons

- **PrimeVue, Element Plus, Naive UI**: great if you want a back-office app that looks
  like a back-office app. Here, aesthetics are part of the product and the ready-made
  components would have to be rewritten anyway.
- **AG Grid**: very powerful, but the functions that make it interesting sit behind an
  enterprise license — not an option on a free app distributed to anyone.
- **A charting library**, for now: it gets added when a screen actually asks for one, not
  before.

## 11 — Screens

| Screen | Answers | Source |
|---|---|---|
| **Next steps** | The 5 things worth doing right now, ranked by how much they unlock | save + graph |
| **Unlock** | What's missing, filterable and sortable on every facet from section 07 | save + graph + catalog |
| **Plan** | My goals, expanded into steps that tick themselves off | graph + save diff |
| **Completion** | Character × mark matrix, and how much of that matrix we can actually read | save |
| **Collection** | Items never touched, by pool and quality | save + catalog |
| **Runs** | Win rate by character, most common ending and killer, streaks | run archive |
| **Live** | What I've collected in this run, current plan | log watcher |
| **Profile selection** | Which save I'm looking at, and how to switch it | discovery |
| **Search** | Where this thing lives: items, achievements, challenges, characters, bosses, wiki pages, unlock-tree nodes, and the screens themselves | catalog + wiki + save + graph |

> **Three of those rows have since been decided differently**, and one of them was decided
> *against*. **Runs** does not answer "win rate by character, most common ending and killer,
> streaks": M4's spec chose a **diary** — a row per run, what happened — and declined the
> scoreboard, because with no clock in the log and no floor recorded on a death the tally could
> only group by killer and character. **Next steps** is called *Obiettivi consigliati* (B32) and
> groups by the reason a row is suggested. And the table is missing **Wiki** and **Floor**, which
> did not exist when it was written. Everything else in it was built as described.

**Profile selection** sits at the bottom of the list because it's the least flashy, not
because it's optional: on a real machine six saves coexist — three slots for the `rep_`
edition and three for `rep+` — and every number the app shows depends on which one is
being read. So it isn't a first-launch step you go through once and forget, but a
permanent state: the active profile stays visible in the shell and switches from there in
one click. When the choice is ambiguous the app asks and explains why; it never silently
falls back to a different profile.

**The shell behaves like a browser.** Several tabs open together, switch between them,
reorder them, close them; a global setting — *keep tabs saved on close* — and on restart
they reopen where they were. A tab isn't a section but a **location**: it can be a screen,
but also an item's detail, a wiki page, or a node of the unlock tree, and two tabs can
show the same screen with different filters. That's how you compare two items without
losing your place. Two constraints: a tab saves the view's **identity** and never its
content — on restore it reloads from the backend, just like on first launch — and the
**active profile stays a single one for the whole window**, because tabs with different
profiles side by side would break the previous paragraph's promise.

**Search** reaches across everything the app knows, not one section at a time: catalog
names, the text of wiki sections, unlock-tree nodes, and the screens themselves. Two
surfaces: a keyboard palette for quick access, and a screen for the full set, filterable
by type. A result is a reference to one of our own entities, so opening it and following a
wiki link are the same action. The degrade constraint holds here too: without the game
installed, search still finds screens and actions, and states what it isn't searching.

## 12 — Distribution

From the moment an installer exists, half the work stops being the parser: it becomes
everything between the double-click and the first useful screen.

- **Guided first launch**: find Steam → find the game → find the saves (and if there's
  more than one, let the user pick the slot) → extract the resources → ready. Every step
  with a manual fallback, because there will always be a second-disk install or a
  non-Steam copy. But slot selection doesn't end here: the *Profile selection* screen
  remains, viewable and changeable at any time.
- **Different versions**: some people only have Rebirth, some stopped at Afterbirth+,
  some have Repentance+. The parser recognizes the version and the UI hides what that
  user doesn't own, instead of showing them goals that don't exist for them.
- **Updates**: Tauri has an updater backed by GitHub Releases — free, static, no server.
  Releases are signed with the updater's own key (a signature, not a service credential).
- **SmartScreen**: an unsigned installer shows the "unknown publisher" warning to the
  first users. A code-signing certificate costs a few hundred euros a year. A realistic
  alternative for a v1: distribute from GitHub Releases, explain the warning in the
  README, consider signing only if the app gains traction.
- **Name and disclaimer**: fan-made project, not affiliated with Nicalis or Edmund
  McMillen, stated plainly in the about screen.
- **Linux and macOS**: Isaac is on both and Tauri compiles for both. Only the `discovery`
  module would change. Not in v1, but worth not writing code that rules it out.

## 13 — Roadmap

**M0 ✓ — Format spike — done.** Format decoded and a working Python parser against 28
real saves. Counters labeled, the mark matrix reconstructed, the log verified. Four small
sections, the counter tail and the bestiary were left unidentified at the time; where they
stand now is in §03 and, day by day, in `docs/STATUS.md`.

**M1 — Rust parser, discovery, unpack.** The core that makes the app installable by
anyone: finding Steam and the game, extracting the resources, reading the save. First
screen: Completion.

**M2 — Graph and Unlock section.** Bootstrapping the dataset, "unlockable now", every
facet and sort order. This is where the app stops being a viewer.

**M3 — Derived plan.** Goals, expansion into steps, automatic diff against the save,
grouping by run.

**M4 — Log watcher and run archive.** Live runs and historical stats. Value keeps growing
from here with no further work.

**M5 — Public release.** Installer, updater, i18n, stranger-proof onboarding, README with
the SmartScreen warning. The first user who isn't you is the real test.

## 14 — Risks

| Risk | Impact | Mitigation |
|---|---|---|
| ~~`.dat` layout changed by Repentance+~~ | Closed | Decoded in M0; counts read from the file, never hardcoded |
| ~~Log too thin to reconstruct runs~~ | Closed | Verified against a real log: items, character, pool, floors, killer and endings are all there |
| Installs different from yours (disks, DLC, slots, non-Steam) | High | Discovery with manual fallback at every step; test on at least one machine that isn't yours before M5 |
| A future patch breaks the parser or the log | Medium | Versioned adapters, rules in updatable data files, partial degradation |
| Maintaining the unlock graph | Medium | Bootstrap from an existing dataset + updates via the MediaWiki API; dataset kept separate from code |
| SmartScreen warning on the installer | Medium | Explicit README in v1; certificate only if the app gains traction |
| Licenses: game assets and wiki content | Medium | No asset in the package; wiki dataset with its own license and attribution |
| Corrupting a user's save | Critical | No write API anywhere in the code; read-only opening |

## 15 — Next step

M0 is closed and the parser exists. The next step is **M1**: port the parser to Rust
inside the Tauri skeleton, add `discovery` and `unpack`, and get the first screen up —
Completion, which by now is just a matter of drawing a grid we already know how to fill.

To do in parallel, unhurried and without code: **close out the remaining cells**. Every
session played produces a new dated backup, and every backup narrows the field on
Delirium, Mother, The Beast, and the four still-anonymous sections.

---

Technical sources:
[isaac-faq · directories and save files](https://github.com/Zamiell/isaac-faq/blob/main/directories-and-save-files.md) ·
[isaac-completion-fix](https://github.com/jadenkeener/isaac-completion-fix) ·
[IsaacUnlockFinder](https://github.com/nerstak/IsaacUnlockFinder) ·
[isaac-save-viewer](https://github.com/Zamiell/isaac-save-viewer) ·
[RebirthItemTracker](https://github.com/Hyphen-ated/RebirthItemTracker) ·
[REPENTOGON · PersistentGameData](https://repentogon.com/PersistentGameData.html) ·
[REPENTOGON · enum EventCounter](https://repentogon.com/enums/EventCounter.html) ·
[Rebirth Wiki · Modding Tools](https://bindingofisaacrebirth.wiki.gg/wiki/Modding_Tools) ·
[ISteamUserStats](https://steamapi.xpaw.me/ISteamUserStats)
