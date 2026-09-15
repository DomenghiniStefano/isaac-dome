# IsaacDome — design system brief

Document to feed to Claude Design. It is not the project document
(`docs/PROJECT.md`) nor the operational context (`CLAUDE.md`): it's the extract needed
to design with, i.e. **what we show, with what data, in which states, and with which components**.

Living document: it gets updated when the backend gains real data or when the design system
makes decisions that affect the app. Module status lives in `docs/STATUS.md`.

**Date:** 2026-09-09 · **Backend status:** M1 closed on the Rust side and `catalog` complete
(plans A and B: names, sprites, quality, pools, achievements, challenges, bosses, origin
DLC); **M2, the unlock graph, closed on 2026-09-07** and **M3's plan queue on 2026-09-08**,
so the three graph screens are no longer designed against declared placeholders — the
values behind §7's types are real. `store` holds the Plan's goals and the queue. The
delivery path fixed in `docs/STATUS.md`, "Handoff to design" section, is complete:
`catalog` plan B ✓ → IPC contracts ✓ → this brief revised ✓ → **handed over 2026-09-09** ✓.
From here the contract in §7 is live: it changes only deliberately, and a change is handed
on rather than merely committed.

**Added on 2026-09-06:** three new product requirements, which land on the shell before
they land on individual screens — the app splits into **top-level sections** (§4),
**global search**, and **multiple tabs** with session restoration (§4.2). Logged
as B5 and B6 in `docs/BACKLOG.md`. They change none of the contracts in §7.

> **Superseded on 2026-09-15.** `pnpm design:export` is abandoned: the design is decided at
> runtime on the real screens now, not on an exported package. What survives of the pack is a
> **fixture corpus** — the committed IPC payloads, the two indexes and ten wiki pages, 28 files
> and 2.8 MB — which `pnpm ui:dev` reads to run without a backend. Its **6065 images are gone
> from the repository**, so everything below about sprites, sheets and their counts describes a
> package that no longer exists. The contracts in §7 are unaffected: they are the IPC's shape,
> and `ui/src/lib/ipc/types.ts` is generated from the Rust types regardless.

**This document travels with a package.** `pnpm design:export` produces
`isaacdome-design-pack/`: the real IPC payloads for every command, the game's images with
their index (family, id, name, rectangle, **real dimensions**), and ten sample wiki pages
already paired with their image. The last export reports **5,833 catalogued images** —
2,074 pulled straight out of the archives plus 3,759 pieces cut from the game's own sheets
by the rectangles its `.anm2` files declare — in **5,837 files, 36 MB** all told, since the
regular families are packed into atlases rather than written one file each. Wherever
there's a number below, that package has the file it came from.

---

## 0. What changed since 2026-09-02 (read before the rest)

Seven updates, and they change what can be designed **on real data** instead of assumptions.

1. **The game is installed.** `D:\SteamLibrary\...\The Binding of Isaac Rebirth`, Repentance+
   edition. The XML catalogs and graphic archives exist on this machine: they are no longer
   a theoretical blocker.
2. **The app launches and works.** The first time ever. The Steam → game →
   save → parser → matrix chain runs end-to-end on the real profile. All the values in §5.4
   are **read from it**, not estimated.
3. **And the game's sprites extract.** Fixed within the day: the extractor only read one
   archive format out of three. Now it reads all three — **14,751 resources extracted, 0 failed**,
   sprites and full catalogs included. Detail and inventory in §5.6.
4. **The catalog has names, all of them.** (2026-09-04) `catalog` reads `items.xml`,
   `players.xml`, and `stringtable.sta`: 909 out of 909 items with an English name and sprite, 41
   characters with a name, and **37 out of 41 with a headshot** cropped from the co-op menu sheet
   (§5.6). The "index without a name" case remains only as a transitional state before the
   catalog is read, not as missing data.
5. **The graph screens have a contract, and now they have the graph.** (2026-09-05,
   filled in 2026-09-07/08) Unlock, Next steps, and Plan are designed against TypeScript
   types fixed and pinned by tests (§7). Name, icon, unlock condition, done/not done, what
   it unlocks and origin DLC are **real data** from the catalog and the save; since M2 the
   graph's verdict is real too, and where it couldn't interpret a requirement it says
   **`partial`** — which is not "not known yet" and must never read as unlockable. The
   Plan's goals are actually saved, in an app database, and survive a restart; M3 added the
   queue (§7.6). None of the types changed when the values became real.
6. **The app splits into two, and the split is in the data, not the aesthetics.** (2026-09-06)
   **Wiki** needs nothing: the dataset is compiled into the binary, so it works
   without the game installed and without a save chosen. **Progress** needs both.
   The consequence to design for is that on a machine without Isaac the app **isn't empty**: it opens
   on the Wiki and works (§4).
7. **Almost every wiki page can have its own image.** (2026-09-06) `target_sprite`
   resolves a wiki reference to the game's graphic file: **1690 out of 1727 pages**, and
   the same function gives the small icon to every reference *inside* the text (§5.7). The
   Synergies and Interactions sections are made almost entirely of references: that's where
   the app becomes illustrated rather than typographic.

---

## 1. The app in one sentence

A Windows desktop app that reads local saves for *The Binding of Isaac: Repentance+* and
answers one question: **what am I missing, and what's worth playing tonight.**

Fan-made, offline, no account, no telemetry. It installs and works with no
configuration on any Steam copy of the game.

**The opening screen is "Next steps", not the collection.** If the app opens on a grid of
items we've just rebuilt the game's own menu, which the user already has.

> Naming note: the screen used to be called **"Tonight"**. That name is dropped: the one
> that stands is **"Next steps"**, and `docs/PROJECT.md` is already aligned. The word "tonight"
> stays instead in the *question* the app answers — it's the time constraint that makes the
> recommendations useful — but it's no longer the name of any screen.

---

## 2. Constraints that govern the design

These aren't preferences: they are product constraints, and each has a direct consequence for the UI.

| Constraint | Consequence for the design |
|---|---|
| **No game assets in the package** | Sprites and icons are extracted from the user's copy **at runtime**, after the first launch. Every component that shows an image must have a decent placeholder and work *without* the image. Never a layout that collapses if the sprite is missing. |
| **Degrade, never fail** | Every screen has a "I only know part of it" state. It isn't a rare error case: it's the normal case until the graph and the catalog exist. A consistent visual way is needed to say *"I don't have this data"*, distinct from *"this data is zero"*. |
| **Offline, no backend** | No login, no sync, no network states. The only network use is an optional dataset update. |
| **No telemetry** | No cookie banners, no consent prompts, no onboarding that asks for data. |
| **Read-only on saves** | No UI for writing, editing, or "repairing the save". Not even as a future idea. |
| **Different game versions** | Someone who only has Rebirth must not see goals they don't own. The UI hides by edition, it doesn't grey out with a lock icon. |
| **Desktop, not web** | Resizable window, desktop-app density. No mobile considerations. Reasonable minimum target: 1280×800. |
| **Dark-first, i18n from day one** | Dark theme as the real default (not a variant), and every string translatable IT/EN. No text baked into images. |

---

## 3. UI stack (already decided, not up for discussion)

- **Vue 3 + TypeScript**, Vite, Pinia, Vue Router, vue-i18n
- **shadcn-vue on Reka UI** — the components live **in the repo**, they get modified
- **Tailwind v4** — tokens and themes via CSS variables
- **TanStack Table** for filterable grids, **TanStack Virtual** for long lists
- **Lucide** for interface icons

Deliberately ruled out: PrimeVue, Element Plus, AG Grid. The design can therefore afford
custom-built components, but it must stay expressible with the shadcn primitives (Dialog, Popover,
Command, Tabs, Table, Badge, Toggle Group, Sheet, Progress, Skeleton).

The design system should deliver: a type scale, a palette with semantic tokens (including
the "done / unlockable now / blocked / unknown" states), spacing, table density, and the
variants of the components above.

---

## 4. The three sections, and the screens inside

The app has **three top-level sections**, and it isn't a matter of taste: they have three
different preconditions, and that's the thing that most shapes the shell's design.

| | **Progress** | **Tool** | **Wiki** |
|---|---|---|---|
| does it need the game installed? | yes — catalog, names, sprites | no, it degrades to ids | **no** |
| does it need a save chosen? | yes | **no** | **no** |
| where the data comes from | `.dat` + XML + `.a` archives | `log.txt`, the run archive, what you painted | dataset compiled into the binary |
| what it contains | the five screens in the table below | Live, Runs, Floor | ~1,727 pages: items, trinkets, achievements, bosses, challenges, characters |
| if there's nothing | shows the profile selection | answers anyway, with less in it | works in full, **without images** |

**This read "two sections" until 2026-09-15, and the sentence was right when it was
written.** What it got right is the method — a section is a precondition, not a folder — and
what it missed is that a third precondition had been accumulating inside *Progress*. Live
reads `log.txt` and Runs reads the run archive; neither has ever opened the `.dat`, and
`RunsScreen.vue` said so in a comment while sitting behind the profile gate anyway. They were
filed by resemblance — they are about playing — rather than by what they need, which is the
mistake this table exists to prevent.

The Floor screen (`docs/superpowers/specs/2026-09-15-floor-secret-rooms-design.md`) made it
visible by being a third screen of the same kind, and the F1 plan's first answer was a
per-route exception carved into `router/routes.ts`: `needsProfile` for the Progress origin
*except* this one. **The exception is the symptom; the section is the fix.** `needsProfile` is
still derived from the origin and from nothing else, and now with nothing carved out of it —
which is the property `sectionNav.test.ts` keeps.

Three consequences, all to be designed:

1. **On a machine without Isaac the app isn't empty.** It opens on the Wiki, which works on
   its own. It's the only piece of the product that depends on nothing, and it's also the
   first thing a new user sees if they installed the app before the game.
2. **The Wiki has two outfits.** The **text** is always there; the **images** come from the
   game's archives (§5.7), so without the game installed the pages are typographic. This isn't
   the same case as "sprite not yet extracted" in §5.6: there the image will eventually
   arrive, here it won't.
3. **Progress, until a choice is made, *is* the profile selection screen.** Not a modal
   wizard that blocks the app, not a separate screen to pass through once: it's the
   section's content, and it stays reachable afterward (§4.1).

**Settings** and **About** aren't a third section: they aren't places you navigate to, they
are destinations. They sit at the bottom of the shell bar and open as regular tabs (§4.3).

### The screens

Five live inside *Progress* and three inside *Tool*. Two live in neither, and it's worth
saying up front: **Profile selection** (0) is what *Progress* shows until a choice is made,
and **Search** (8) cuts across all three — it searches the wiki and progress together (§4.2).

The traffic light = availability of real data, not priority.

| # | Screen | Answers | Data | Today |
|---|---|---|---|---|
| 1 | **Next steps** | The 5 things worth doing right now | save + graph | 🟢 **real** (§7.3): what is unlockable *now*, ordered by how much each opens up (`basis: 'fanOut'`). Empty, with a `noCatalog` diagnostic, when the game isn't installed |
| 2 | **Unlock** | What's missing, filterable on every facet | save + graph + catalog | 🟢 **real** (§7): 641 nodes — name, icon, condition (283 of 637), done, what it unlocks, origin DLC — plus a typed `missing[]` and the graph's verdict: `computed`, or `partial` where a requirement wasn't interpretable |
| 3 | **Plan** | My goals, in the order I mean to do them | graph + save diff | 🟢 **real** (§7.4, §7.6): saved goals, and a queue whose rows are the ones you asked for plus the prerequisites they dragged in. `expansion` is the one field still a declared stub |
| 4 | **Completion** | Character × mark matrix, and how readable it is (see §5.3) | **save only** | 🟢 **designable now** |
| 5 | **Collection** | Items never touched, by pool and quality | save + catalog | 🟢 **real** (§7.7): the save's item collection joined with the catalog's collectibles — name, sprite, quality, pools, origin, whether the collection holds it, and the achievement that locks it. Trinkets have no slot in the save, and aren't listed |
| 6 | **Runs** | What I have played, and how each one ended | run archive | 🟢 **real** (M4): the folded runs of every source, ordered with the open one first — character, outcome, floors, seed, the online mark — with four facets and a search over a virtualized table. The wire carries no timestamp: a session is ordered by its folder's name |
| 7 | **Live** | What I am playing, and what finishing it would open | log watcher + graph | 🟢 **real** (M4): the open run as four readings and no percentages, the character the log states, its row of the completion matrix, and the achievements this run would open — offered only where *every* still-missing requirement is a mark for the character being played |
| 9 | **Floor** | Where the secret room can be on the floor I'm on | what you painted + the game's cited placement rules | 🔴 F1 (`docs/superpowers/specs/2026-09-15-floor-secret-rooms-design.md`) |
| 8 | **Search** | Where this thing is, wherever the app knows it | catalog + wiki + save + graph | 🟢 designable now for catalog and wiki; graph nodes enter the results with M2 (B5) |
| 0 | **Profile selection** | What am I looking at, and how do I change it | **discovery only** | 🟢 **designable now** |

**The order this was designed in, and it held.** Profile selection and Completion first —
the two that could be filled entirely with real data, so the ones where the design was
verified instead of merely looking plausible. Then Collection, Unlock, Next steps and Plan:
real data behind a fixed contract (§7), with a *partial* state designed as such. Then Runs
and Live, which this paragraph expected to *"only need a shell: navigation, headers, empty
state, loading skeleton"* and which turned out to carry the judgments the log cannot make.
**Every screen in the table above is built as of 2026-09-15.** What the traffic light
measures is data, and it was never a measure of design.

**And now all of them can have the game's images.** The traffic light above measures
*data*; *assets* are an independent axis, and it turned green today (§5.6). It remains
true, though, that every component must also work without sprites: extraction happens on
the user's machine after the first launch, so the first design pass for every screen is
always the one without images.

### 4.1 Profile selection isn't a step: it's a permanent state

An explicit requirement, and it changes the shape of screen 0.

On a real machine there are **multiple saves and multiple game versions at once**: three
slots for the Repentance edition (`rep_`) and three for Repentance+ (`rep+`), six files
coexisting in the same folder. On this machine that's exactly the case.

From this, three design consequences, all mandatory:

1. **What I'm looking at must always be visible**, not just on first launch: edition,
   slot, and date of the active profile go in a persistent indicator in the shell, because
   every number in the app depends on that choice. A user reading "142 achievements
   missing" must be able to see *for which profile*.
2. **Switching profile is a routine action**, reachable from that indicator in one
   click: it isn't buried in settings and doesn't require redoing the first-launch flow.
3. **The app never silently falls back.** When the choice is ambiguous it says so and asks,
   instead of filling the gap with just any profile.

Screen 0, then, isn't a wizard you go through once: it's a **status screen** that's
empty on first launch and needs filling in, and afterward stays reachable and editable.

### The states to design, one by one

The backend returns exactly three outcomes, and each is a different screen:

| Outcome | When | What it must show |
|---|---|---|
| **No candidates** | Steam or the game not found, or no `.dat` at all | The point in the chain that broke (Steam → game → saves) and the manual fallback: you choose the folder |
| **Needs a choice** | Multiple candidates and no valid choice | The list of candidates to compare, with a **highlighted suggestion** (the most recent) that stays a suggestion: until one is chosen, no profile is active |
| **Active** | Choice made, or a single candidate | The active profile in plain view, and the way to change it |

Two clarifications the design must make visible:

- **Single candidate → automatic selection, but declared.** If there's no choice to make, we
  don't ask for one; the indicator, though, still shows what's being looked at. Automatic
  selection is allowed only when it's the only possible one.
- **A saved choice that no longer exists** (Steam Cloud moving files, a deleted slot) →
  it goes back to *"needs a choice"* **with the reason spelled out**, never to some other
  profile picked on its own. Showing the numbers of a different profile without saying so
  is the kind of bug a user never reports, because the app looks like it's working.

Comparing candidates needs help: the only clues available are **edition,
slot number, modification date, and size**. The file path is a supporting detail — useful in
"found here" and in diagnostics, never the main element of a row. On the IPC, candidates
travel with an **opaque id**, not a path: the frontend never handles file names.

---

### 4.2 The shell: multiple tabs and global search

Explicit requirement from 2026-09-06, and it lands on the same strip where §4.1 places
the profile indicator. In the backlog as **B5** (search) and **B6** (tabs).

**The shell is tabbed, like a browser.** Several tabs open at once, you switch between
them, reorder them, and close them. A tab isn't a section: it's a **position**. It can be one
of the screens, but also an item's detail, a wiki page, or a node in the unlock tree — and two
tabs can show the same screen with different filters, which is how two items get compared.
In the global settings, a preference: *keep tabs saved on close*, and on restart they reopen
where they were.

**A single, mixed bar.** Wiki tabs and Progress tabs sit **together**, in the same
bar: that's the requirement, and it's also the whole point of the feature — you compare a
wiki page with your own collection by switching from one tab to the other. Switching section
doesn't hide the other section's tabs: it moves the focus, it doesn't filter the bar.

Four design consequences, all mandatory:

1. **A tab must declare which section it comes from**, or the mixed bar becomes unreadable
   as soon as it passes five tabs. It needs a mark of origin on the tab itself — the
   section's icon, not color alone, which holds up neither in monochrome nor for someone
   who can't distinguish hues.
2. **Main navigation and the tab bar aren't the same thing.** The three sections and the
   screens inside them are the orientation (where I can go); the tabs are what I have open
   right now. The design has to say how they coexist without becoming two overlapping rows
   of tabs.
3. **There's exactly one active profile for the whole app**, not one per tab — and since
   2026-09-13, not one per *window* either. §4.1 establishes that every number in the app
   depends on that choice; tabs with different profiles side by side would break that promise,
   and so would two windows. The indicator therefore lives **outside** the tab bar, not inside
   a tab, and changing it updates every tab **in every window** together. The same holds for
   the interface's size and for the plan's queue: one app, one answer. Note that Wiki tabs
   **don't** depend on the profile: switching saves leaves them where they are, and that's
   correct.
4. **A restored tab may point at something that no longer exists** (profile changed,
   different edition, dataset updated). It opens **declaring the gap** — this is a state to
   design for, not an edge case: the tab doesn't silently disappear and doesn't turn into
   another view.
5. **A tab can be torn off into a window of its own, and dragged back** (2026-09-13, B15).
   Drag it past the strip and a light preview follows the cursor; release it on the desktop
   and it opens a window, release it on another window's strip — which draws a marker where it
   would land — and the two merge. **A secondary window is the whole app**: title bar, strip,
   navbar, sidebar, and the "+" that opens tabs of its own. Two rules bound it: the last tab
   of a window does not open a *second* window for itself (that window already is that tab),
   and a window whose last tab joins another one closes — except the first window, which keeps
   a fresh tab, because the bar is never empty.

**Search finds anything the app knows about**: items, trinkets, characters, bosses,
achievements with their condition, challenges, the text of wiki sections, nodes in the unlock
tree, and the screens themselves. Two surfaces:

- the **palette** (`Ctrl+K`), which opens from any screen, groups results by type
  and opens them on enter — it's shadcn's **Command** primitive, already listed in §3;
- the **Search screen** (no. 8 in the table above), for the full, type-filterable set
  when the results don't fit in a palette.

Two things the design must keep in mind. First: **in-game names stay in English**
even in the Italian interface (§12), while screens, actions, and facets have Italian names —
the same query hits both, and the grouping of results has to make that legible. Second: a
result coming from the **body of a wiki section** isn't like a match on an item's name — it
must be shown with the text fragment that matched and the section it came from, otherwise the
user can't tell why that row is there.

A search result, on the IPC, is a `Target` (§8) plus the match's context: opening a
result and following a wiki link are **the same action**, and they can look the same.

**Search sits above the three sections, not inside one.** The field lives in the shell and
is reachable from any tab; a result declares which section it comes from, because
"Brimstone" exists as a wiki page *and* as a Collection row, and those are two different
destinations. From here, the way search **degrades**, which is a state to design for:

- **without a profile** → no "done / not done" results, and search says so instead of
  simply returning fewer of them;
- **without the game installed** → the whole Wiki remains, but names come from page
  titles instead of the catalog, and there's no icon at all;
- **without either** → screens and actions remain, which is still a useful search:
  "where are the settings".

---

### 4.3 Settings and About

Two destinations, not a third section. They open as tabs, from the bottom of the shell
bar.

**Settings is, first and foremost, the data-provenance page.** That's the requirement that
makes it different from an ordinary preferences page, and it's also the most distinctive
thing the app can show: an app that reads someone else's files has to say **where every
number it displays came from**. One row per source, and each says the same thing: what it
is, where it lives, when it was last read, what version it has, and whether it's okay.

| source | what it declares | where the data already exists |
|---|---|---|
| **Save** | edition, slot, modification date, size, sections read (10 out of 10) | `setup_state`, `save_summary` |
| **Game** | folder, edition, owned DLCs, date of last Steam update | `setup_state.game` |
| **`.a` archives** | how many opened, compression mode, indexed entries | `extraction_report.archives` |
| **Catalog** | 909 items, 41 characters, 637 achievements, 45 challenges, 103 bosses, and diagnostics | `extraction_report.catalog` |
| **Wiki dataset** | snapshot date, latest known patch, comparison with the game's patch | `extraction_report.wiki` |
| **App database** | the only file we write, schema version, saved goals | `plan.storeAvailable` |
| **Preferences** | chosen profile, tabs saved on exit, language | `settings` |

No new commands: all of this is already in the §7 and §5 payloads. The work is design
work, not backend work.

The rest of the settings is short and should stay that way: active profile (the same one
from §4.1), game folder with the manual fallback, language, tabs saved on exit.

**The interface's size is a setting, and the first one implemented** (2026-09-12, cycle
3.5c). It is a percentage on Discord's eleven zoom levels — 50 · 67 · 75 · 80 · 90 · 100 ·
110 · 125 · 150 · 175 · 200 — persisted in `settings.json` and applied before the first
paint. The mechanism is one custom property on the root: `html` is
`calc(16px * var(--app-scale))` and every size token is in rem, so nothing in a component
knows the size. The one exception is pixel art, which is drawn at a whole multiple of 32px
(`--sprite-multiple`) because the game's art smears at a fraction of its own size.

```typescript
// crates/ipc/src/settings.rs, mirrored in lib/ipc/types.ts
interface Settings {
  activeProfileId: string | null
  scale: number // a percentage, always one of the eleven; anything else reads as 100
}
```

Two commands: `settings()` and `setScale(percent)`, the second writing the file and
answering the settings as they now are. A size that isn't on the ladder is read as 100 —
a file we didn't write, not "about 125".


**About** carries the credits and licenses from §14. Three things deserve to be in plain
sight rather than in a footer, because they're the reason a stranger should trust an app
that opens their saves: **read-only on saves, no account, no telemetry.** These are
verifiable claims — the app has no code that writes to `.dat` files, by construction — and
they should be written as statements of fact, not as marketing reassurance.

---

## 5. The real data, today

### 5.1 `discovery` — first launch

**These are no longer a sketch: they are the TypeScript types the frontend uses today**
(`ui/src/lib/ipc/types.ts`). Design can rely on them: if a field isn't here, it doesn't exist.

```ts
type CandidateSource = 'steamCloud' | 'documents' | 'manual'

interface CandidateView {
  id: string                    // opaque: the frontend never sees a path
  prefix: 'rep' | 'rep_plus'    // save file edition
  slot: number                  // 1..3
  source: CandidateSource
  modifiedUnix: number | null   // can be missing
  sizeBytes: number
  suggested: boolean            // exactly one true, or none
  pathHint: string              // "found here", redacted of the Steam account id
}

type MissingReason = 'steamNotFound' | 'gameNotFound' | 'noSaves'

type ActiveProfile =
  | { kind: 'none'; reason: MissingReason }
  | { kind: 'needsChoice'
      reason: { kind: 'neverChosen' } | { kind: 'savedProfileGone'; was: string }
      suggested: string | null }
  | { kind: 'active'; profile: CandidateView; autoSelected: boolean }

interface SetupState {
  steam: { rootHint: string; libraries: number } | null
  game: { dirHint: string; edition: string; dlcs: string[] } | null
  candidates: CandidateView[]
  active: ActiveProfile
  diagnostics: SetupDiagnostic[]
}

type SetupDiagnostic =
  | { kind: 'steamNotFound' } | { kind: 'gameNotFound' } | { kind: 'noSavesFound' }
  | { kind: 'unreadablePath'; name: string; reason: string }
  | { kind: 'malformedManifest'; name: string }
```

And the types for the matrix and the summary, which underpin *Completion*:

```ts
type Cell =
  | { kind: 'known'; bits: number }       // 0,1,2,3,5,7 — a mask, not a count
  | { kind: 'unknown' }                   // we can't read it ≠ it isn't done
  | { kind: 'unexpected'; value: number } // outside the expected values: show as suspect

interface MarksMatrix {
  characters: {
    character: string
    group: string            // the file's three blocks: original, forgotten, later
    tainted: boolean         // added 2026-09-11: the base / Tainted grouping a player uses
    cells: Cell[]
    headUrl: string | null   // added 2026-09-11: the co-op menu head, null without the game
  }[]
  bosses: string[]
  art: { normalUrl: string | null; hardUrl: string | null }[]
                             // added 2026-09-11: art[i] draws bosses[i], null without the game
  totals: { cells: number; readable: number; unknown: number
            unexpected: number; started: number }
                             // started counts bit 0 or bit 1 since 2026-09-11: bit 2 alone
                             // draws nothing in the grid, so it doesn't count either
}

interface SaveSummary {
  profile: string
  sections: { kind: string; count: number }[]
  diagnostics: SaveDiagnostic[]
}
```

**What this means for design:** each of the three levels (Steam, game, save) can be
`null` **independently**. The wizard isn't a linear sequence: it's a list of three
requirements, each resolved automatically or fixed by hand. Each row needs three states:
*found · not found, you choose · more than one, which one?*

Note on `Cell`: the three variants must be rendered **visually distinct, not graded**.
`unknown` isn't "less done" than `known` with `bits: 0`: it's a different category
altogether. And `unexpected` is a value we didn't expect to read — it should be shown as
an anomaly, not hidden among the empties.
### 5.2 `core-save` — the save file

Ten sections, each with a `kind`, a `count` read from the file, and the raw bytes:

| kind | typical count | content |
|---|---|---|
| 1 | 642 | achievements and secrets (1 byte: done / not done) |
| 2 | 523 | game counters **and completion marks** |
| 3 | 14 | level counters, one cell per stage — index 0 is unused |
| 4 | 733 | item collection (1 byte) |
| 5 | 7 | yet to be identified |
| 6 | 104 | bosses met |
| 7 | 46 | challenges |
| 8 | 27 | yet to be identified |
| 9 | 2 | yet to be identified |
| 10 | variable | bestiary: four tallies over the same entities, self-describing |

> **The counts aren't constants.** A save from January 2025 declares **521** entries in
> section 2, recent ones **523**; June 2025 declared 641 achievements, 2026 declares
> 642. Patches add entries. No number should be hardcoded — **not even in the design**:
> don't design "642 achievements" as a fixed label, design "N achievements".

### 5.3 The mark matrix — the heart of *Completion*

Rows: **34 characters**, in three groups that the game itself treats differently —
14 originals (Isaac → Apollyon), The Forgotten on its own, and 19 made up of Bethany,
Jacob & Esau, and the 17 Tainted.

Columns: **12 bosses/endings**, the same twelve the game's own widget draws — Mom's Heart,
Isaac, Satan, Boss Rush, Blue Baby, The Lamb, Mega Satan, Greed, Hush, Delirium, **Mother**,
**The Beast**. The last three were located on 2026-09-08, on the historical series; before
that the grid was 34 × 10 and this document said so.

Each cell is a **bitmask**, not a number: bit 0 and bit 1 are the mark's two levels
(normal and hard mode), bit 2 is a third level **whose meaning is unconfirmed**.
Observed values: 0, 1, 2, 3, 5, 7.

**Two gaps to design for explicitly, not to hide:**

1. **Mother** and **The Beast** are located for the 14 original characters only. For The
   Forgotten and the 19 of the third group — **40 cells**, the bottom-right corner of the
   grid — the position is inferred from the spacing and confirmed by nothing, so the app
   reports them as *unknown*. They are not rare: it is a solid 20 × 2 block, and the design
   has to survive it being unreadable.
2. We don't know what the third mark level is.

So the matrix has *unknown* cells next to *empty* cells, and they're two different
things: "you never did it" ≠ "we can't read it". If the design conflates them, the app
lies. For the same reason **there's no reliable completion percentage yet**: the screen
must be able to show an honest, partial total, along the lines of *"marks at least
started: N out of M readable"*, not a made-up "73% Dead God".


### 5.4 Real values, read from the running app

These aren't orders of magnitude: they're the numbers the app printed today on this
machine's profile. They're here to size components against a real case.

**Discovery** — Steam at `C:\Program Files (x86)\Steam`, the game on a **second
library** (`D:\SteamLibrary\...`), edition `repentance_plus`. Four candidates, all Steam
Cloud:

| prefix | slot | size | suggested |
|---|---|---|---|
| `rep_plus` | 1 | 14,948 bytes | yes |
| `rep_plus` | 2 | 4,068 bytes | |
| `rep` | 1 | 14,172 bytes | |
| `rep` | 2 | 3,956 bytes | |

Two things the design must absorb: **the Steam library isn't the default one** — the
`pathHint` doesn't start with `C:\`, so don't truncate it from the left — and **size is a
strong clue**: 14 KB versus 4 KB separates, at a glance, a played profile from one just
started. It's worth more than the date.

**Save sections**, with the `count` values read from the file:

```
achievements 642 · counters 523 · level_counters 14 · items 733 · unknown5 7
bosses 104 · challenges 46 · unknown8 27 · unknown9 2 · bestiary 80
```

`bestiary: 80` varies by profile: it's the only section whose length depends on how much
you've played. The other nine don't.

**Mark matrix**, the total the screen must be able to state:

> **166 started out of 368 readable · 40 unknown · 0 suspect**

408 nominal cells (34 × 12), of which 368 are readable and 40 `unknown`. And the 40 are
**Mother and The Beast for The Forgotten and the third group's 19**: a rectangular gap in
the bottom-right corner, 20 rows by 2 columns, not scattered cells. It's the real case to
verify §5.3's design against — if the matrix renders that block indistinguishable from
"never done", it's lying while looking like it works.

Values observed in the readable cells, on the whole grid: `0` × 202, `3` × 108, `2` × 32,
`7` × 12, `1` × 11, `5` × 3. They are not spread evenly, and the shape of the screen
depends on that: among the 14 originals `3` is the most common value (85 of 168 cells) and
only 40 cells are empty, while among the 19 of the third group **161 cells out of 228 are
`0`** and 38 more are unknown. The top of the grid is dense, the bottom is nearly empty.
Five distinct values plus `unknown` plus `unexpected`: **seven visual states for one
cell**, in a grid of 408 that has to stay readable at a glance. It's the tightest design
problem on the screen.

### 5.5 `catalog` — how much we can say about names, today

**Rewritten on the evening of 2026-09-03: the catalogs are now read in full.** With
`unpack` complete and the resolver, the `items.xml` that wins precedence is Repentance's:
**909 entries** (425 passives, 170 actives, 126 familiars, 188 trinkets), and **every one
has its sprite** — asserted by a test against the real file, not a hand count.

**The names.** In the DLC XMLs, `name` is a key (`#THE_SAD_ONION_NAME`), not a piece of
text. The actual text lives in `stringtable.sta`, which **isn't binary: it's XML**, 3,151
keys across 14 categories. It resolves **every key for the 909 items** in `items.xml`
(some names are literal, not keys) and 21 out of 21 in `players.xml`.
(The "two pickup placeholders" mentioned in an earlier version of this document,
`PILLS_HERE_NAME` and `TAROT_CARD_NAME`, weren't unresolved keys: they sit inside a
commented-out element of `items.xml`, which the XML parser ignores — they don't contribute
to the catalog, so there's nothing to resolve for them.) Languages: English, Japanese,
Korean, Simplified Chinese, Russian, German, Spanish, French. **No Italian** — and it
doesn't need one: in-game names stay in English by choice (§12), the app's Italian is the
interface's, and we write that ourselves.

**Achievements and challenges are the exception:** `achievements.xml` (637 entries) and
`challenges.xml` don't use keys, they carry literal English text. In exchange,
`achievements.xml` carries, as a comment above every row, the **unlock condition** in
current-tense English:

```xml
<!-- have 7 or more max red hearts at one time -->
<achievement id="1" text='You unlocked "Magdalene"' gfx="Achievement_Magdalene.png" />
```

And **370 out of 909 items declare `achievement="N"`**, i.e. which achievement unlocks
them: a graph edge already written by the game itself. Raw material for Unlock and the
graph work (M2).

**Consequence for design:** the *"I know the index, not the name"* case still needs
designing, but it's only the transitional state before the catalog is read: **once the
catalog is read, no entry is nameless**. **Quality** (from −1 to 4, read from
`items_metadata.xml`), each item's **pools** (from `itempools.xml`), and the **origin DLC**
have been in the catalog since plan B: the Collection's facets can be designed against
real values.

**Character headshots** (2026-09-04): `gfx/ui/coop menu.png` is a 192 × 224 sheet made
of **32 × 32** cells, and `catalog` knows which cell belongs to which character: **37 out
of 41**. They're the matrix's natural row label for Completion, in place of the name
alone. The four without a cell (Tainted Lazarus Risen, Dark Esau, Tainted Soul, Esau) are
forms the game's menu doesn't show as a separate choice: the row must still hold up with
just the name, and it's the same "sprites not yet extracted" state as §5.6, not one more
case.

**Real label length**, measured on the 403 base-edition names (the full 909 are now
measured the same way, and the scale doesn't change):

```
min 2 · median 11 · p90 15 · max 22
```

The longest ones: *The Ludovico Technique*, *Experimental Treatment*, *We Need To Go
Deeper!*, *Telepathy For Dummies*, *Mom's Bottle of Pills*. A name column sized for ~22
characters never truncates in English, and names stay in English even in the Italian
interface, so the limit doesn't change with the language. **Achievement text does,
though**: `You unlocked "X"` is a sentence, not a label, and it grows when translated.

### 5.6 The graphic assets: what exists, and how it reaches the screen

The most important paragraph for whoever is designing. **Rewritten on 2026-09-03:
extraction works.**

**What exists** in the user's copy, counted from `filelist.txt` (18,789 paths):

| folder | files | use in the app |
|---|---|---|
| `gfx/items/collectibles/` | 722 | icon for every item — Collection, Unlock |
| `gfx/items/trinkets/` | 189 | trinkets |
| `gfx/items/pick ups/` | 203 | cards, pills, pickups |
| `gfx/ui/achievement/` | 673 | icon for every achievement — Unlock screen |
| `gfx/ui/boss/` | 285 | boss portraits and character portraits |
| `gfx/characters/costumes/` | 1,675 | character sprites |
| `gfx/ui/main menu/` | 142 | game interface elements |
| `gfx/ui/completion_widget.png` | 1 | the game's original marks widget |

In total **10,545 graphic files**, and now they're reachable: a full pass over all the
archives extracts **14,751 resources with 0 failures**, for 1.0 GB of content.

**Real sprite dimensions**, measured on the extracted files — needed to size containers
without guessing:

| resource | size | how many |
|---|---|---|
| item / trinket icon | **32 × 32** | 908 out of 909 |
| *Monkey Paw* (trinket 20) | **128 × 32** | 1 — it's a strip of **four poses** |
| achievement icon | **263 × 176** (not square) | 637 out of 637, no exceptions |
| boss portrait | **192 × 192** | 94 out of 101 |
| double boss portrait | **384 × 192** | 6 — Pin, Polycephalus, Mega Fred, The Stain, Big Horn, The Pile |
| *Mother*'s portrait | **480 × 440** | 1 |
| character sprite (`character_*`) | **512 × 512** (costume sheet, not a portrait) | |
| `completion_widget.png` | **256 × 512** (sheet with all the marks) | plus `completion_widget_pause.png`, same size |

The count in the right-hand column is measured by reading the header of every extracted
PNG, on 2026-09-06, and it **contradicts what this table said before**: families that look
uniform aren't.

Three immediate consequences.

**The achievement icon isn't square** — it's a landscape rectangle, and a grid designed
around square cells will stretch or crop it. This, though, is the one family that's truly
uniform: 637 out of 637.

**Size exceptions need to be handled, not averaged away.** A square container that
assumes 192 × 192 squashes *Mother* and the six double portraits; a 32 px cell crops off
three quarters of *Monkey Paw*. They're eight cases out of a thousand and nobody notices
them at design time: that's why they sit in the package as **standalone files outside the
atlases**, where they're visible.

**`character_*.png` is a costume sheet**, not a ready-made portrait: for a character's
face you need the `PlayerPortrait_*` files in `gfx/ui/boss/`, not these.

**The matrix's column headers have a real image.** The ten mark columns are bosses, and
nine out of ten have their portrait among the 101 in the package. The mapping, though,
**isn't done by name**: in the game *Blue Baby* is called `???`, and the *Greed* mark is
earned by beating *Ultra Greed* — guessing it by resemblance would give you *Baby
Plum*. The map, written by hand with the reasoning behind every exception, is in `data/marks.json`
in the package. **Boss Rush is the only column without a portrait**, and it isn't a gap to
fill with a generic image: it isn't a boss, it's a timed room, and that spot wants a mark
we design ourselves.

**The map moved to `ipc` on 2026-09-11**, with the Completion screen
(`crates/ipc/src/mark_art.rs`, beside `BOSSES`): column → anm2 file → layer → frame, frame 0
for the normal symbol and frame 2 for the hard one. The app serves each symbol and each co-op
menu head as a crop of the user's own sheet, through the same `isaac://` links as the icons,
and the matrix carries them in `art` and `headUrl` (§5.1). `data/marks.json` stays in the
package as the design's copy; the bloodied-paper `symbolFallback` is not carried by the app,
whose fallback is the cell's own bars outfit.

**The game's own marks widget is reference material.** `completion_widget.png` contains
the visual vocabulary the game itself uses for marks: the same little sheet in four
treatments — plain, blood, blue, purple — plus two files of symbols. It's the most direct
source we have on how the game distinguishes mark levels, and it's worth looking at before
inventing a new coding for the matrix's cells (§5.4).

**And the cells already have a ready-made icon, one per state.** `completion_widget.anm2`
— which sits in the archives next to the sheet — declares **one layer per mark** and each
one's frames are its states: eleven symbols (`heart`, `polaroid`, `negative`,
`upsidedowncross`, `star`, `megasatan`, `greed`, `hush`, `knife`, `dadsnote`, `cross`),
**16 × 16**, each in two levels, plus six 96 × 96 backgrounds. In the package they're
already cropped in `images/sheets/completion_widget/`.

The game distinguishes the two levels by **changing the symbol**, not by tinting it: the
second level is a different row on the sheet, not the same drawing with a filter.

**But the cell isn't just the symbol: it's also the background, and the background
carries information.** The `Paper` layer has six frames, along two axes — the edge
(straight → torn → **bloodied**) and the surface (clean or marked).

**Delirium has a symbol of its own, and it is in another file.** `completion_widget.anm2`
has no Delirium layer, which is why for a while the package handed over the bloodied paper
under Delirium's name. The mark is named in **Repentance+'s online lobby**
(`main menu/onlinelobby.anm2`, layer `Completion_Delirium`), the only file in the game that
names all twelve: a small face, two eyes and three teeth, the one on the player cards of
the lobby screen. It ships as `onlinelobby/background_completion_delirium_02.png`.

Because it comes from a different sheet, that row — and only that row — also carries a
**`symbolFallback`**: `completion_widget/paper_02.png`, the cell's own background, which
really does turn bloody when the mark is taken. A grid that would rather not mix two
sheets has something true to fall back on; the face is the primary.

From here, all twelve columns have their symbol, and `data/marks.json` gives it alongside
the header portrait. The **`symbolSource` field says how we know it**, because not every
row carries the same weight: eleven come from a **layer's name** — *Knife* for Mother,
*DadsNote* for The Beast, *Completion_Delirium* from the lobby; *The Lamb* is by
**elimination**, the one symbol (`cross`) and the one column left over once every other
pairing is settled.

The two unassigned symbols used to be the evidence that the widget had two columns our
matrix didn't. That gap closed on 2026-09-08: the columns were located in the save, and
`knife` and `dadsnote` are now columns 10 and 11 like any other. What remains is the
bottom-right block of §5.3 — the same two columns for The Forgotten and the 19.

**How it reaches the screen, and why the placeholder stays mandatory.** No asset is in
the package: they get extracted **from the user's copy, at runtime, after the first
launch**. So:

1. **The version without images must be designed regardless, and first.** It's no longer
   the permanent case, but it's every user's first launch and the case for anyone with an
   incomplete installation. Every component must ship in two states.
2. **The placeholder must still distinguish two different gaps:** *image not yet
   extracted* (temporary) and *name unknown* (§5.5). They're different things and must read
   as different.
3. **Reserve the icon's space right from the layout**, with the real proportions from the
   table above, so the sprite's arrival doesn't shift the grid.

> One caveat that remains: `repentance.a` has 4,180 entries but **none of the paths in
> our `filelist.txt` reach them**. The archive's index only contains hashes, so without the
> exact name a resource is unreachable. The most recent Repentance sprites might therefore
> be missing until we get hold of an updated path list. It isn't a flaw in the reader: it's
> an incomplete name dictionary.

---

### 5.7 Every wiki page can have its own image

Added on 2026-09-06, and it's what makes the Wiki section illustrated rather than
typographic. The wiki dataset carries **text** and typed references; the images live in the
game's archives. `target_sprite` is the joint between them, and it resolves a reference to
the graphic file that corresponds to it.

**Real coverage, measured on the installed game and the embedded dataset:**

| reference type | where the image comes from | coverage |
|---|---|---|
| item | `gfx/items/collectibles/` | **719 out of 719** |
| trinket | `gfx/items/trinkets/` | **188 out of 188** |
| character | `PlayerPortrait_*`, plus the headshot from the co-op sheet | **32 out of 32** |
| achievement | `gfx/ui/achievement/` | **637 out of 641** |
| boss | the portrait, by entity key | **75 out of 102** |
| challenge | the icon of the **achievement it rewards** | **39 out of 45** |
| transformation, room, pickup, floor | — | **0**, and it's declared as such |

**In total: 1690 pages out of 1727 have an image, 97.9%.**

Two things the design must know.

**The boss is found by the file name, not by pairing them up.** `bossportraits.xml`
writes the portrait as `Portrait_20.0_Monstro.png`: the entity's type and variant are
*inside* the name, and it's the same key the wiki uses for its entity references. The 27
bosses left out are entity pages the game doesn't illustrate with a portrait — common
enemies, or bosses whose portrait doesn't declare the key. It isn't a gap to fill with a
generic image: it's one of the two placeholders from §5.6.

**Challenges have no art.** `gfx/challenge` doesn't exist in the game. The only image
that truly belongs to a challenge is the achievement won by completing it, and six
challenges out of 45 have no known one.

**What multiplies the images isn't the cover, it's the text.** Beyond the large image at
the top of the page, every reference **inside** the text can carry its small icon next to
the name. The Synergies and Interactions sections are made almost entirely of references:
the *Brimstone* page has **185** of them. That's where it gets decided whether the Wiki
looks like a text page or a page of the game itself — and it's the dense case that needs
designing, not the average one.

Fallout for the contract, worth mentioning because **it converges with an already-known
flaw**: designing the inline icons needs a command that returns images for a list of
references. It's the same shape needed to strip the base64 icons out of `unlock`'s 641 rows
(flaw C2 in §7.5): **one single command serves both the Wiki and the Unlock grid**. It
doesn't exist yet; it arrives with the real frontend, and §7's types don't change.

---

## 6. Unlock's facets

The app's densest screen: it needs designing even though the data arrives later.

One filter matters more than all the others: **"unlockable now"**. Three states always
visible: **done · unlockable now · blocked by N**.

| Facet | Values |
|---|---|
| Status | done · unlockable now · blocked |
| What it unlocks | passive item · active item · trinket · card · pill · character · challenge · room type · pickup |
| Required character | the 34 playable ones |
| Required ending | Mom · Mom's Heart · Isaac · ??? · Satan · Lamb · Mega Satan · Hush · Delirium · Mother · The Beast |
| Mode | normal · hard · Greed · Greedier |
| Effort shape | one run · N runs · streak · cumulative · depends on RNG |
| Quality and pool | 0–4 · treasure, boss, devil, angel, shop… |
| Origin DLC | Rebirth · Afterbirth · AB+ · Repentance · Rep+ |
| Global rarity | Steam percentage |

Sort orders that matter: **fan-out** (how much it unlocks downstream), missing steps,
rarity, item quality.

That's nine facets over a virtualized list of ~642 rows. It's the app's most serious
design problem: nine simultaneous filters without turning the screen into a control panel.
It's worth studying now, even though the implementation is M2.

Of the nine, some already have real data: the *done* state and what it unlocks are in
§7's contract; so is the origin DLC, but with **four values** (Rebirth, Afterbirth,
Afterbirth+, Repentance: the ones the catalog distinguishes by id threshold, Repentance+
doesn't have its own); quality and pool come from the catalog. The others deliberately have
no field: see "What's not included" in §7.

---

## 7. The graph's contracts

The three screens that depend on the unlock graph (M2) and the plan derived from it (M3)
are designed against **these types**, fixed on 2026-09-05 and pinned by tests on the JSON's
shape. They're copied from `ui/src/lib/ipc/types.ts`: if a field isn't here, it doesn't
exist. There's exactly one rule governing them: **data that exists arrives real, and what
cannot be computed says so in a variant of its own**, never as a value that looks computed.

**The graph exists now.** M2 closed on 2026-09-07 and M3's plan queue on 2026-09-08, so the
`{ kind: 'stub' }` this section used to be built around — "we'll fill it in later" — has
left the wire entirely. What takes its place is a harder question, not an easier one:
`partial` doesn't mean *not known yet*, it means **this node has a requirement we could not
interpret**, and it must never read as unlockable. One field is still honestly a stub,
`PlanExpansion` (§7.4), and the queue in §7.6 is what supersedes it in practice.

### 7.1 The node: one for three screens

Next steps and Plan are **selections** from Unlock, not different types.

```ts
type AchievementRef =
  | { kind: 'known'; id: number; text: string; hint: string | null; iconUrl: string | null }
  | { kind: 'unknown'; slot: number }

type ItemKindView = 'passive' | 'active' | 'familiar' | 'trinket'

// a target's identity, and the on-disk database format
type TargetKey =
  | { kind: 'item'; itemKind: ItemKindView; id: number }
  | { kind: 'character'; id: number }
  | { kind: 'boss'; id: number }
  | { kind: 'challenge'; id: number }

// the same identity as the interface shows it: name and icon resolved right now
type UnlockTarget =
  | { kind: 'item'; itemKind: ItemKindView; id: number; name: string; iconUrl: string | null }
  | { kind: 'character'; id: number; name: string; tainted: boolean }
  | { kind: 'boss'; id: number; name: string }
  | { kind: 'challenge'; id: number; name: string }

type OriginView = 'rebirth' | 'afterbirth' | 'afterbirthPlus' | 'repentance'

type GraphInfo =
  | { kind: 'computed'; availableNow: boolean; blockedBy: number; fanOut: number; stepsMissing: number }
  | { kind: 'partial'; blockedBy: number; fanOut: number; unknown: number }

// what a node is still missing, typed by the nature of the target: this is what the
// screen groups by, so it can say "1 character and 2 bosses" instead of "blocked by 3"
// `page` is the wiki page that says how *this* is unlocked, and it is what the badge's menu
// opens (3.5d). `null` means the dataset has no page for it: the name shows and does not
// link — never "no requirement". The four kinds below that are entities carry it; a gate, a
// mark, a counter and an uninterpreted label are conditions, so they carry none at all.
type RequirementView =
  // the two forms of a character share the game's name (achievements.xml writes
  // You unlocked "The Lost" for both): the flag is what tells them apart
  | { kind: 'character'; id: number; name: string; tainted: boolean; page: Target | null }
  | { kind: 'boss'; id: number; name: string; page: Target | null }
  | { kind: 'challenge'; id: number; name: string; page: Target | null }
  | { kind: 'item'; itemKind: ItemKindView; id: number; name: string; page: Target | null }
  | { kind: 'gate'; label: string }
  // one cell of the completion matrix: go and beat `column` with this character
  | { kind: 'mark'; character: number; characterName: string; column: MarkColumnView; level: MarkLevelView }
  // a tally and its threshold, with where the profile stands: the one requirement that is
  // not a wall — the content is already reachable, it only has to be played
  | { kind: 'counter'; label: string; current: number; atLeast: number }
  | { kind: 'unknown'; label: string }

interface UnlockNode {
  achievement: AchievementRef
  done: boolean               // from the save, section 1: REAL
  unlocks: UnlockTarget[]     // from the catalog: REAL; empty if it unlocks nothing known
  origin: OriginView | null   // DLC of the first item unlocked: REAL
  missing: RequirementView[]  // what stands in the way, one entry per requirement: REAL
  graph: GraphInfo            // REAL since M2 (2026-09-07)
}
```

Five things to read in the node:

- **`achievement.known` and `achievement.unknown` are two different states**, not two
  degrees of the same thing. `unknown` is an achievement the save knows about and the
  catalog doesn't (a patch newer than the file): it has a slot and a `done`, nothing else.
  It isn't an error and isn't missing data to flag: it's the "index known, name unknown"
  row from §5.5, and now it has a real number — 4 out of 641, 3 of them done, on this
  machine's profile.
- **`hint` is displayable text in English, and `null` is normal**: 354 out of 637
  achievements have no readable condition in the file. It shouldn't be rendered as
  "condition unknown".
- **`ItemKindView` and `OriginView` are bare strings, not objects with a `kind`.** There's
  one rule: a fieldless enum travels as a plain string; the tag only appears where the
  variants carry different data. They're *values* to display or filter on, not
  discriminators.
- **`graph: { kind: 'partial' }` is the hardest state in this document.** It does not mean
  "not computed yet" — that state no longer exists. It means the graph read this node's
  requirements and **could not interpret at least one of them**: `unknown` says how many.
  A `partial` node therefore knows something (`blockedBy`, `fanOut`) and is honest about
  not knowing the rest, and the one thing it must never look like is **unlockable now**.
  Treating it as "0 prerequisites" would send someone to play for an unlock that isn't
  there. The package contains a real one to look at: *"!Platinum God! OMG!"*, whose
  `missing` holds two `unknown` requirements (`"Collect"`, `"ending"`) beside a resolved
  character.
- **`missing` is the *why*, and it's typed on purpose.** `blockedBy: 3` is a number;
  `missing` says it's one character and two bosses, with names and ids. Group on it: the
  screen should say what stands in the way, not how much of it there is. `gate` and
  `unknown` carry only a label — they're conditions expressed in prose that we deliberately
  did not guess at.

### 7.2 Unlock

```ts
interface UnlockTotals { slots: number; done: number; known: number; unknown: number }

type UnlockDiagnostic =
  | { kind: 'slotsBeyondCatalog'; count: number }  // the save has more slots than the catalog
  | { kind: 'catalogBeyondSlots'; count: number }  // the catalog is newer than the save
  | { kind: 'noCatalog' }                          // game absent: every slot an unknown, partial node
  | { kind: 'noAchievementSection' }               // section 1 unreadable: NOT "zero done"

interface UnlockView {
  nodes: UnlockNode[]          // one per slot 1..N of section 1
  totals: UnlockTotals
  diagnostics: UnlockDiagnostic[]
}
```

`noAchievementSection` is the only diagnostic that zeroes out the screen, and it's a
state to design for separately: nodes and totals at zero do **not** mean "zero achievements
done", they mean that section of the file didn't get read. The other three coexist with
real nodes.

`totals.slots` is the raw flag count from the save, slot 0 included; the nodes are
`slots − 1` and `known + unknown = slots − 1`. On the reference profile
(`samples/live.rep+persistentgamedata1.dat`, 2026-08-31): **642 slots, 379 done, 637
known, 4 unknown**, one `slotsBeyondCatalog { count: 4 }` diagnostic. The app running on the
live Steam save already shows 381: the figure follows the file, it isn't a constant to
design against. Filters and sorting are frontend state (TanStack Table): the contract
carries the data, not the interface.

### 7.3 Next steps

```ts
// no fields: a bare string, like `OriginView`. One value today; a second basis
// (closeness, once the counters are understood) would arrive as another value here.
type StepsBasis = 'fanOut'

interface NextSteps {
  steps: UnlockNode[]   // at most 5, not done
  basis: StepsBasis
}
```

**Next steps changed meaning with M2**, and the change is the point: it used to be "the
first 5 not-done in slot order" — real, but not *recommended*. It is now **what is
unlockable right now, ordered by how much each one opens up** (`fanOut`). Two consequences
the screen has to carry: a node the graph can only call `partial` is **not** a step, because
we can't vouch for it; and without a catalog the list is **empty**, with a `noCatalog`
diagnostic saying why, rather than five rows of something else. An empty Next steps is a
state to design, not an error.

### 7.3.1 The want — naming what you're after (B37)

```ts
type WantState =
  | { kind: 'done' }              // already yours
  | { kind: 'availableNow' }      // nothing in the way: play it
  | { kind: 'chain'; steps: UnlockNode[]; unknown: number }
  | { kind: 'noProfile' }         // section 1 unread: the route is named, your position isn't

interface WantRoute { node: UnlockNode; state: WantState }

type WantedView =
  | { kind: 'target'; target: UnlockTarget }
  | { kind: 'achievement'; achievement: AchievementRef }
  | { kind: 'unresolved' }

type WantDiagnostic =
  | { kind: 'noCatalog' } | { kind: 'noProfile' }
  | { kind: 'nothingUnlocks' } | { kind: 'notUnlockable' }

interface WantView {
  wanted: WantedView
  routes: WantRoute[]
  diagnostics: WantDiagnostic[]
}
```

You name a thing — an item, a character, a boss, a challenge, or an achievement by its own
name — and the answer is the ordered series of what is still missing for it. Four things the
screen has to carry:

- **`routes` is a list.** A challenge can be named by two achievements: **14 of 45** are
  (measured 2026-09-13). Two ways in draw as two blocks, each with its own button; one is
  never picked for you.
- **`steps` is an order, not a set.** It is the order the Plan would play, produced by the
  queue's own `enqueue`: the preview and the "add to the Plan" button are one computation.
- **An empty `steps` never means "nothing missing".** That state is `availableNow` and says
  so. `unknown > 0` means the chain holds requirements the app cannot fully read.
- **Naming an achievement is how you ask for a mode.** *Greed Mode* and *Greedier* are not
  targets the catalog models; the achievement that grants them is, and `wanted` carries that
  case as a variant of its own.

The want is a place: `#/goals?want=item:105`, the same key a wiki page is addressed by, so
back, forward and tab restore all reach it.

### 7.4 Plan

```ts
type GoalId = string   // opaque, generated by the app, never built by the frontend

interface GoalView {
  id: GoalId
  key: TargetKey                 // what I want, as it's saved: identity and nothing else
  target: UnlockTarget | null    // the same thing resolved right now; null = unresolvable
  createdUnix: number
  note: string | null
}

interface PlanStep { goal: GoalId; node: UnlockNode; done: boolean }

type PlanExpansion =
  { kind: 'stub' } | { kind: 'computed'; steps: PlanStep[] }

type PlanDiagnostic =
  | { kind: 'storeUnavailable'; reason: string }   // the database won't open, and why
  | { kind: 'unreadableGoal'; id: GoalId }         // a row this version can't read
  | { kind: 'noCatalog' }                          // game absent: no target resolved
  | { kind: 'unresolvedGoal'; id: GoalId }         // a key the catalog no longer knows

interface PlanView {
  goals: GoalView[]
  expansion: PlanExpansion          // today always { kind: 'stub' }
  diagnostics: PlanDiagnostic[]
  storeAvailable: boolean           // = no storeUnavailable diagnostic
}
```

**The database only saves `key`; `target` gets created on every read.** A goal doesn't
carry the name and icon from the day it was created: the backend resolves them against
today's catalog, so a goal saved before the game was installed shows its name as soon as the
game is there. Consequence for design: **a Plan row must be able to appear without a name
and without an icon**, identified only by its `key`, and stay removable. This happens in two
cases, distinguished by the diagnostics: `noCatalog` (game absent — applies to every row,
and arrives once) and `unresolvedGoal` (that one particular id the catalog no longer knows,
for example after a patch).

Goals are **entities the user creates and saves**, few ("a couple") but with no
hardcoded limit, in an app database (`isaacdome.db`) and not in the game's save, which stays
read-only. They're per installation, not per profile: "I want Tainted Lost" holds for
whoever plays on that machine, and the Plan will show, profile by profile, what's missing.
Commands: `plan()`, `addGoal(key)`, `removeGoal(id)`, each responding with the updated
`PlanView`.

The Plan **degrades and says why**. `storeAvailable: false` with `storeUnavailable`
means "goals can't be seen or added": the text already comes from the app (for example
"database from a newer version (2 > 1)", the one case the user can act on), and the screen
must not render it as an empty list. `unreadableGoal` carries the id: the row stays in the
file and the UI can offer to remove it — and the same holds for `unresolvedGoal`, which is
instead a readable row with a key the catalog no longer knows. `expansion: stub` is the
normal state until M3 exists: goals are visible, steps aren't.

The errors from the write commands, tagged like all the others:

```ts
type IpcError =
  | { kind: 'noActiveProfile' }
  | { kind: 'unknownProfile'; id: string }
  | { kind: 'unreadableSave'; reason: string }
  | { kind: 'settingsNotWritable'; reason: string }
  | { kind: 'unknownTarget' }                    // addGoal: the catalog doesn't know the target
  | { kind: 'catalogUnavailable' }               // addGoal: can't verify it (game absent)
  | { kind: 'storeUnavailable'; reason: string } // addGoal/removeGoal: no database
```

### 7.5 What's real, and where the graph stops short

| field | source | status |
|---|---|---|
| `achievement.known.{id, text, hint, iconUrl}` | catalog | ✅ real |
| `achievement.unknown.slot` | save beyond the catalog | ✅ real |
| `done` | save, section 1, `slot[id]` | ✅ real |
| `unlocks[]` | catalog's reverse index | ✅ real |
| `origin` | DLC of the first item unlocked | ✅ real |
| `missing[]` | the wiki's typed requirements, resolved against the catalog | ✅ real |
| `graph` | `graph`, evaluated against the profile | ✅ real — `computed`, or `partial` where a requirement wasn't interpretable |
| `NextSteps.steps` | what is unlockable now, most fan-out first | ✅ real, `basis: 'fanOut'` |
| `GoalView.key` | app database | ✅ real |
| `GoalView.target` | catalog, resolved on every read | ✅ real, `null` if unresolvable |
| `QueueRow.{node, wanted, origins, stepsNotQueued}` | the saved queue, resolved through the graph | ✅ real |
| `PlanView.expansion` | — | 🔲 `stub`, and the queue (§7.6) is what stands in for it |

**What's not included, and why.** The §6 facets that depend on **interpreting** the 283
English-language conditions (required ending, mode, effort shape) and Steam rarity
(network) **have no field**: setting them as a permanent `null` would fake having the data.
Where the graph read a requirement but couldn't type it, that isn't a missing field either
— it's a `RequirementView` of kind `gate` or `unknown` carrying the raw label, and the node
above it says `partial`. The contract grows by addition once something exists to compute
them; components designed today won't break.

---

### 7.6 The queue: the Plan is an order, not a set

`PlanView.goals` is a **set** — the things you want. The queue is the **order** you intend
to do them in, and it's the part with rules to draw.

```ts
interface QueueRow {
  node: UnlockNode        // the same node Unlock draws, so the two can never disagree
  wanted: boolean         // you asked for this one, for itself
  origins: number[]       // the wanted achievements whose chain passes through this row
  stepsNotQueued: number  // prerequisites this row still needs that are NOT in the queue
}

type QueueDiagnostic =
  | { kind: 'storeUnavailable'; reason: string }
  | { kind: 'unreadable' }
  | { kind: 'completed'; count: number; wanted: number[] }
  | { kind: 'unresolved'; achievement: number }
  | { kind: 'goalsPending'; count: number }
  | { kind: 'noCatalog' }

interface QueueView {
  rows: QueueRow[]
  diagnostics: QueueDiagnostic[]
  storeAvailable: boolean
}
```

Four states the design has to tell apart, and the package's `queue.with_rows.json` contains
one of each so they can be looked at rather than imagined:

- **A row you asked for** — `wanted: true`, `origins: []`.
- **A row that arrived by itself** — `wanted: false`, `origins: [55]`: a prerequisite that
  a wish dragged in. It has to read as *serving* that wish, not as something you chose.
  Both can be true at once: you can ask for a step that also serves another wish.
- **A wish with work outside the queue** — `stepsNotQueued: 1`. This is the number that
  says "there's more to do than what you're looking at", and it is the reason the queue
  isn't just a list.
- **A row that isn't there any more** — no row at all, and a `completed` diagnostic in its
  place. A row never disappears without a word.

**The one rule the whole thing rests on: a move repairs, it never fails.** Drag a row and
the queue rearranges itself around the constraint — dependents are dragged along below it,
prerequisites gather above it, everything else keeps its relative order. There is no
rejected drop and no error toast to design, because there is no illegal move: prerequisites
are a wall the row stops against, not a refusal. What the graph can't compute carries no
constraint at all, so a `partial` row is never dragged and never walls.

**A drop names the row it lands under, never an index**: `queueMove(achievement, after)`, with
`after: null` for the top (since 2026-09-11; it took `to`, an index, before). The rows the view
leaves out — completed, unresolved — still sit in the saved queue, so a position on screen is not
a position in the file; and a row that drags its dependents along would shift any index past
them. What the design draws after a drop is the order the command answers with. When a rising
row stops short, the row right above it is the prerequisite that stopped it, and the Plan says
so in the queue's band.

---

### 7.7 The Collection: the save's item collection, joined with the catalog

Added on 2026-09-11 with the Collection screen (3.4): `collection()`.

```ts
interface CollectionView {
  items: CollectionItem[]      // collectibles only, by id
  pools: string[]              // the pools any listed item belongs to, in the catalog's order
  totals: { slots: number; items: number; inCollection: number }
  diagnostics: CollectionDiagnostic[]
}

interface CollectionItem {
  id: number
  kind: ItemKindView           // 'passive' | 'active' | 'familiar', never 'trinket'
  name: string
  iconUrl: string | null       // isaac://item/<kind>/<id>
  quality: number | null       // items_metadata.xml; null when unrated
  pools: string[]              // itempools.xml names, untranslated
  origin: OriginView | null
  inCollection: boolean | null // null: section 4 unread, or no slot for this id
  lock: LockView
}

// `page` is the achievement's wiki page, what the badge's menu opens (3.5d); `null` is "the
// dataset has no page", never "no achievement". `free` has no such key: nothing unlocks the
// item, so there is nothing to open.
type LockView =
  | { kind: 'free' }                                                                    // nothing unlocks it
  | { kind: 'unlocked'; achievement: number; text: string | null; page: Target | null } // its achievement is done
  | { kind: 'locked'; achievement: number; text: string | null; page: Target | null }   // not done: it can't appear
  | { kind: 'unknown'; achievement: number; text: string | null; page: Target | null }  // section 1 unread

type CollectionDiagnostic =
  | { kind: 'noCatalog' }
  | { kind: 'noCollectionSection' }
  | { kind: 'noAchievementSection' }
  | { kind: 'itemsBeyondSlots'; count: number }
```

**Section 4 is the save's item collection**: one slot per collectible id, the catalog's ids a
subset of its slots. What a set byte means in play — picked up, or merely seen — is not
measured, so the contract uses the section's own name, **in the collection**. **Trinkets have no
slot**, and the Collection lists collectibles only. **Unread is never "not in the collection"**:
a missing section, or a slot past its end, gives `inCollection: null`, and the design has to draw
that as unreadable, not as "never found". The design pack carries the real payload as
`contracts/payload/collection.json` once `pnpm design:export` has run on a machine with the game
and a save.

---

## 8. The wiki detail view

The text of the wiki sections (Effects, Notes, Synergies, Interactions, Bugs for items
and trinkets; Behavior, Strategies, Damage Scaling for bosses; condition and reward for
achievements and challenges) arrives as a **typed tree**, already resolved: every reference
to another entity is one of our own `id`s, not a text link. The dataset is embedded in the
binary (backlog item B1, implementation closed on 2026-09-05:
`docs/superpowers/reports/2026-09-05-wiki-dataset-report.md`), so it's always available, even
offline; some references stay unresolved (`Concept`) or come from templates the parser
doesn't know yet — they're the same kind of "declared unknown data" as the rest of the
contract, not an error to hide.

**The page has one figure, and the text inside it has many.** The entity's image at the
top (a 32×32 item to be scaled up with `image-rendering: pixelated`, a 192×192 boss
portrait, a 263×176 achievement icon that's *not* square) and the small icon next to every
`Ref` in the text: both come out of `target_sprite`, with coverage measured in §5.7. Ten
sample pages, already paired with their image and chosen half for known shape and half for
size — the longest, the shortest, the one with the most tables — sit in `wiki/samples/` in
the package.

TypeScript types, copied verbatim from `ui/src/lib/ipc/types.ts`:

```typescript
export const SectionKind = {
  Effects: 'effects',
  Notes: 'notes',
  Synergies: 'synergies',
  Interactions: 'interactions',
  Bugs: 'bugs',
  Behavior: 'behavior',
  ChampionVersions: 'championVersions',
  DamageScaling: 'damageScaling',
  Strategies: 'strategies',
  Difficulty: 'difficulty',
  Reward: 'reward',
  Unlockable: 'unlockable',
} as const
export type SectionKind = (typeof SectionKind)[keyof typeof SectionKind]

export const Style = { Plain: 'plain', Bold: 'bold', Italic: 'italic' } as const
export type Style = (typeof Style)[keyof typeof Style]

export const Dlc = {
  Rebirth: 'rebirth',
  Afterbirth: 'afterbirth',
  AfterbirthPlus: 'afterbirthPlus',
  Repentance: 'repentance',
  RepentancePlus: 'repentancePlus',
} as const
export type Dlc = (typeof Dlc)[keyof typeof Dlc]

// The identity of a wiki element: what a `ref` points to, and what
// `loadWiki` accepts to load a page.
export type Target =
  | { kind: 'item'; id: number }
  | { kind: 'trinket'; id: number }
  | { kind: 'character'; id: number }
  | { kind: 'achievement'; id: number }
  | { kind: 'challenge'; number: number }
  | { kind: 'entity'; id: number; variant: number; subtype: number }
  | { kind: 'transformation'; id: number }
  | { kind: 'stage'; name: string }
  | { kind: 'room'; name: string }
  | { kind: 'pickup'; name: string }

export type Inline =
  | { kind: 'text'; text: string; style: Style }
  | { kind: 'ref'; target: Target; label: string }
  | { kind: 'concept'; page: string; label: string }
  | { kind: 'edition'; only: Dlc[]; inline: Inline[] }

export interface ListItem {
  inline: Inline[]
  children: Block[]
}
export type Block =
  | { kind: 'paragraph'; inline: Inline[] }
  | { kind: 'list'; ordered: boolean; items: ListItem[] }
  | { kind: 'table'; header: Inline[][]; rows: Inline[][][] }
  | { kind: 'heading'; level: number; inline: Inline[] }

// No fields: a bare camelCase string. Which of the wiki's two collectible templates the
// page used — not the game's three-way item kind, because the wiki has no familiar
// template and writes familiars with the passive one.
export const CollectibleTemplate = {
  Passive: 'passive',
  Activated: 'activated',
} as const
export type CollectibleTemplate =
  (typeof CollectibleTemplate)[keyof typeof CollectibleTemplate]

export type Infobox =
  | {
      kind: 'item'
      quote: Inline[]                // the pickup quote; carries edition markup on 78 of 719
      template: CollectibleTemplate
      quality: number | null
      tags: string[]
      recharge: Inline[]            // `unlimited`, `one time`, `4s`, per-edition forms
      devilPrice: Inline[]
      shopPrice: Inline[]
      pools: Inline[]               // only what the wiki states: 45 of 720 pages
    }
  | { kind: 'trinket'; quote: Inline[]; tags: string[]; pools: Inline[] }
  | {
      kind: 'achievement'
      requirements: Inline[]
      notes: Inline[]               // an achievement has no sections: this is its only prose
      unlocks: Target | null
    }
  | {
      kind: 'boss'
      baseHp: number | null
      stageHp: Inline[]
      variant: number | null
      environment: Inline[]
      pool: Inline[]
    }
  | {
      kind: 'challenge'
      blindfolded: boolean
      hasShops: boolean
      hasTreasureRooms: boolean
      items: Inline[]
      trinkets: Inline[]
      pickups: Inline[]
      health: Inline[]
      curse: Inline[]
      goal: Inline[]
      character: Target | null      // the character the challenge forces, when it forces one
      unlocks: Target | null
    }
  | {
      kind: 'character'
      health: Inline[]
      damage: string
      tears: string
      range: string
      speed: string
      luck: string
      shotSpeed: string
      pickups: Inline[]
      collectibles: Inline[]
      parent: Target | null
    }

export interface Section {
  kind: SectionKind
  blocks: Block[]
}
export interface Entry {
  title: string
  revid: number
  // Three facts every kind declares, so they sit here and not in the variants.
  description: Inline[]  // the infobox's summary line, inline everywhere
  dlc: Dlc[]             // the codes the infobox declares; empty is not "exists everywhere"
  unlockedBy: Target | null  // what the WIKI states; null never means "free from the start"
  infobox: Infobox
  sections: Section[]
}

// No fields: a bare string, like `MissingReason`.
export type WikiMissingReason = 'schemaMismatch' | 'malformed'

// The embedded dataset's state: loaded (with the counts and the diagnostics) or not
// (with the reason). `unresolved` and `unknownTemplates` are totals of occurrences, not
// of pages: unresolved (occurrences) and unknown templates (occurrences).
export type WikiInfo =
  | {
      kind: 'loaded'
      snapshotAt: string
      lastKnownPatch: { number: string; date: string } | null
      counts: {
        items: number
        trinkets: number
        achievements: number
        bosses: number
        challenges: number
        characters: number
      }
      unresolved: number
      unknownTemplates: number
      gameNewerThanSnapshot: boolean | null
    }
  | { kind: 'missing'; reason: WikiMissingReason }
```

**The command.** `wikiEntry(target: Target): Promise<Entry | null>` loads a page; `null`
means "the dataset doesn't know this target" (no page, not an error); `wikiUnavailable` is
an `IpcError` of its own, for when the embedded dataset failed to load at all (incompatible
schema or malformed file — shouldn't happen in a consistent binary, but the contract accounts
for it).

**The index** (added 2026-09-12, cycle 3.5a). `wikiIndex(): Promise<WikiIndex>` answers once
per window with every page the dataset has — its identity, its own title, and the link to
its figure where the catalog draws one. It's what the category lists, the tab labels and the
small icon beside every reference on a page are read from: one load settles all three, and
"does this target have a page" with them. A dataset that didn't load is an empty index whose
`info` says why, not a rejection.

```typescript
export interface WikiPageRef {
  target: Target
  title: string
  iconUrl: string | null // null: no catalog, or the game draws nothing for this page
}
export interface WikiIndex {
  info: WikiInfo
  pages: WikiPageRef[] // the dataset's order: by kind, then by id
}
```

The icon link is `isaac://page/<kind>/<ids>` (`page/item/105`, `page/entity/20/0/0`), served
by the same protocol as every other icon through `target_sprite`. A page is a **tab
location**: `{ name: 'wiki', query: { category, page: 'item:105' } }` — the page key is the
target written as one string, never the page's content (B6).

**How it renders, in three lines:**

- A `ref` is an **internal** link to the entity it points at: the game's icon next to the
  text (item, trinket, character…), a click that calls `wikiEntry` on its `target` and
  replaces the detail being shown.
- A `concept` is plain text, not clickable today: it's a reference the dataset hasn't
  resolved into one of our own ids (an unknown template, or a page that doesn't exist in the
  dataset). A future "open on the wiki" for these cases is out of scope for this cycle (the
  "Out of scope for this cycle" section of the `docs/superpowers/specs/2026-09-05-wiki-dataset-design.md`
  spec).
- An `edition` is text scoped to certain editions (`only: Dlc[]`): it must be flagged — a
  label or a color per edition — not simply rendered as if it always applied.

**Search** (added 2026-09-12, cycle 3.5b). `search(query, limit): Promise<SearchView>` answers
one query over **one index**: the catalog's names, an achievement's own condition, wiki titles
and the body of wiki sections. A target both sides know is one document, so a name is never
listed twice; the catalog's name is its title and the wiki's is an alias when they differ.

```typescript
export type ProgressMark = 'done' | 'pending' | 'unknown' | 'none'
export type SearchDiagnostic =
  | 'noProfile' | 'noCatalog' | 'noWiki'
  | 'noAchievementSection' | 'noCollectionSection'

export type SearchMatch =
  | { kind: 'title' }
  | { kind: 'condition'; text: string }        // the achievement's own wording
  | { kind: 'section'; section: SectionKind    // the page's words around the match
      before: string; matched: string; after: string }

export interface SearchHit {
  target: Target
  title: string
  iconUrl: string | null
  hasPage: boolean          // the dataset has this page: a Wiki destination exists
  match: SearchMatch
  progress: ProgressMark
}
export interface SearchView {
  query: string
  hits: SearchHit[]         // ranked, at most `limit`
  total: number             // how many matched before the limit
  diagnostics: SearchDiagnostic[]
}
```

**The order is the backend's, and the frontend never re-sorts.** Six tiers — the title equal
to the query, starting with it, a word of it starting with it, containing every word, then the
condition, then a section — and inside a tier **not done before done**, which is what makes
the profile part of the ranking. Every word must be in the **same field**: "monstro spits"
finds nothing, because no one field holds both.

**A result is not a row: it is the destinations it opens.** One hit becomes a Wiki row (when
`hasPage`), an Unlock row for an achievement, a Collection row for an item, and the frontend
adds a Screens row for every page whose name matches. A row opens in the active tab, or beside
it with `Ctrl`. A list opened this way **starts from the name alone**: the Collection's default
states would otherwise answer "0 of 721" to a row the user just clicked.

No profile is a diagnostic, never a rejection: search answers before a save is chosen, and
says the marks are unknown. The mark is drawn on a row only when it says something about that
row — `unknown` reads the same on every row of the answer, and the diagnostic says it once.

---

## 9. Orders of magnitude (for sizing the grids)

Updated with the values read from the app (§5.4) and from the game's filelist (§5.6).

- 642 achievements · 733 items · 523 counters · 104 bosses · 46 challenges
- 34 playable characters · 12 mark columns · 11 endings
- Completion matrix: **34 × 12 = 408 cells**, of which 368 readable and 40 unknown
- **7 visual states per cell**: five observed values (0, 2, 3, 5, 7) + unknown + suspect
- 4 candidate save files on this machine (up to 6: 3 slots × 2 prefixes)
- item names: **median 11 characters, p90 15, max 22**
- catalog: **909 out of 909 items with a name and sprite**; 637 achievements, 283 with a
  readable unlock condition; 45 challenges; 103 bosses
- Unlock: **641 nodes** (slots 1–641), 4 of which unknown to the catalog on the reference
  profile; Next steps: 5 rows; Plan: a few goals, no limit
- game assets: **10,545 graphic files, 14,751 resources extracted with 0 failures**
- sprite sizes: item icon **32×32** (908 out of 909), achievement icon **263×176** and
  not square (637 out of 637), boss portrait **192×192** (94 out of 101) — the exceptions
  are in §5.6 and number eight in total, not zero
- **wiki: 1,727 pages**, 1,690 of which with an image (§5.7); the longest has 55 blocks and
  **185 references**, the shortest has 1 block and none
- **cropped game sheets**: 48 `.anm2` files under `gfx/ui/` yield **1,828 individual
  pieces** with the names the game gives them — 11 mark symbols × 2 levels, 24 hearts, 77
  minimap icons, the stat icons, the buttons
- **design package: 3,902 files**, 32 MB — every sprite exists both as a standalone file
  and inside an atlas; 909 items, 637
  achievements, 101 bosses, 41 characters, 347 interface pieces. The three large families
  travel as **atlases** (`oggetti.png` 4096×256, `achievement.png` 3945×7568,
  `boss.png` 4032×960); the complete list, with each sprite's rectangle and its real
  dimensions, is in `images/INDEX.json`
- long lists → virtualization is mandatory (TanStack Virtual)

---

## 10. States to design for every screen

These aren't edge cases: here they're the norm.

- **No active profile** — Steam not found, game not installed, or the choice still to be
  made. It affects **only Progress**: the Wiki works just the same (§4)
- **Wiki without the game installed** — all the text is there, the images aren't and won't
  arrive: it's different from "sprite not yet extracted", which is temporary instead (§4,
  §5.7)
- **Resources not yet extracted** — the app works but without sprites: all placeholders
- **Partial data** — a save section fails to read: show the rest and declare what's
  missing
- **Unknown data** — distinct from zero (see §5.3)
- **Partial graph** — the graph read this node and could not interpret at least one of its
  requirements (§7.1): the node is real, and so is part of the computation, but it must
  never read as unlockable. Distinct from "unknown" and from "loading": it won't arrive
  with a refresh
- **App database unavailable** — the Plan can neither read nor save goals, and says why
  (§7.4)
- **Loading** — skeleton, not a spinner: list sizes are known ahead of time
- **No results** — with filters active, and an obvious way to clear them
- **No search results** — distinct from the previous one: the query matched nothing, and it
  has to say **where** the search looked, because without a catalog search only covers
  screens and actions (§4.2)
- **Orphaned restored tab** — the reopened tab points to an entity that no longer exists
  (profile changed, different edition, dataset updated): it declares the gap, it doesn't
  disappear and doesn't turn into another view (§4.2)
- **Limited edition** — the user only has Rebirth: content is hidden, not struck through

---

## 11. What NOT to design

- A home screen with a grid of items (that's the game's own menu)
- Any account, login, profile, sync, or sharing screen
- Cloud settings, push notifications, social badges
- A save editor, "repair", "import/export profile"
- Decorative charts on data we don't have (the run archive doesn't exist yet)
- Slide-based onboarding: first launch is a task to solve, not a tour

---

## 12. Glossary

To settle now, because it feeds directly into the i18n strings.

| IT | EN | Note |
|---|---|---|
| marchio di completamento | completion mark | the marks per character × ending |
| sbloccabile ora | unlockable now | the filter the whole app rests on |
| bloccato da N | blocked by N | N missing prerequisites |
| sconosciuto | unknown | unreadable data, ≠ zero |
| obiettivo | goal | what the user wants to unlock; it gets saved (§7.4) |
| condizione di sblocco | unlock condition | the node's `hint`: game text, in English |
| salvataggio / slot | save / slot | the game has 3 of them |
| edizione | edition | Rebirth → Repentance+ |
| pool | pool | don't translate: it's game jargon |
| run | run | don't translate |
| Wiki | Wiki | the section that works without the game and without a save (§4) |
| Progressi | Progress | the other section: everything that depends on the chosen save |
| provenienza dei dati | data provenance | the Settings page that says where every number comes from (§4.3) |
| tab | tab | not "scheda": in Italian "scheda" already means an entity's detail view |
| ricerca globale | global search | searches everything the app knows (§4.2) |
| palette | command palette | the `Ctrl+K` overlay; shadcn's Command primitive |
| fan-out | — | internal, don't expose it in the UI |

The names of items, characters, and bosses **stay in English** even in Italian: they're
the names that appear in the game and the ones the user searches for.

---

## 13. How to use this document

**The shell comes first**, which since 2026-09-06 is the part carrying the most
decisions: the three sections (§4), the mixed tab bar with its mark of origin, the profile
indicator, and the entry point to search (§4.2). It's also the one part that can't be put
off, because every screen lives inside it.

Then: **Profile selection** (which is *Progress* before a choice is made) and
**Completion** on real data, then **Collection** (real names and sprites, declared facets),
then **one wiki page** — the app's most illustrated piece and the easiest to fill with real
data —, then the study of Unlock's facets. The graph screens (Next steps, Unlock, Plan)
**have their graph**: M2 and M3 landed, so a node really does say *done · unlockable now ·
blocked by N*, with a fan-out, a missing-steps count and a typed list of what stands in the
way — and, where a requirement wasn't interpretable, it says `partial` rather than guess.
None of §7's contracts changed to accommodate any of that, which was the point of fixing
them first. Runs and Live stay shell: their data only exists at runtime (M4).

**The best test case we have is this one**, and it's real: four profiles to choose
between, two of them practically empty; a 34 × 12 matrix with 166 marks started and a block
of 40 cells we can't read; character headshots for 37 rows out of 41; and real sprites, at
the sizes from §5.6. A design that holds up here holds up the app. A design that needs
complete names to look good doesn't: before the catalog is read, the name is always
missing.

Five questions the first round of visuals should answer:

1. How do you distinguish, at a glance and without relying on color alone, the **seven
   states** of a matrix cell (§5.4)? Look at `completion_widget.png` first: the game already
   has a coding for it.
2. What does a list row look like **before the sprites are extracted**, and when the item
   **has no known name** — just an index? These are two different gaps and neither should
   look like an error (§5.5, §5.6).
3. Where does the **active profile indicator** live in the shell, given that every number
   in the app depends on that choice and changing it is a routine action (§4.1)?
4. How do you draw a node the graph can only call **`partial`** (§7.1) so that it never
   reads as *unlockable now*? The node is real — name, icon, done, what it unlocks, even a
   `blockedBy` count — but at least one of its requirements is a condition we could not
   interpret, and `unknown` says how many. It is the opposite of a loading state: nothing
   further is coming. Drawn as "0 prerequisites" it would send someone off to play for an
   unlock that isn't there.
   *Answered in the app on 2026-09-11 (sub-project 3.3a), for design to overturn:* the blocked
   colours, a **dashed** edge and the lock — never gold, never the star. A partial node is never
   a step and is never counted among the unlockable; its badge's tooltip names the
   requirements the graph couldn't interpret, as the file wrote them.

5. How do the **tab bar**, the **active profile indicator**, and the **search entry point**
   coexist in the same top strip (§4.2)? They're three things with three different meanings —
   what I have open, what I'm looking at, how I find everything else — and the temptation to
   merge them into a single row of tabs would make all three unreadable. The opposite also
   holds: three stacked strips eat into the usable height of a 1280×800 window.

6. How do the tabs of the **three sections** (§4) coexist in a single tab bar?
   Comparing a wiki page with your own collection is the whole reason the tabs exist, so
   filtering the bar by section is off the table. It needs a mark of origin that holds up
   with ten tabs open and that isn't just a color.
7. What does a wiki page **packed with references** look like (§5.7)? *Brimstone* has 185
   of them, and each one can carry its 32×32 icon next to the name. Designed for the average
   case, that page becomes a wall of text; designed well, it's the most beautiful screen in
   the app.

If anything here contradicts `docs/PROJECT.md`, the project document wins — and it
should be flagged, because it means this brief has gone stale.

---

## 14. Credits, licenses, and the three promises

These go in the **About** screen (§4.3), and some also go into the distribution package.

**IsaacDome is fan-made and not affiliated** with Nicalis or Edmund McMillen. This must
be said, and it's the first line of About.

**The game's assets aren't ours and don't travel with us.** The app extracts them from
the user's copy, at runtime, after the first launch: it's constraint 3 of the project, and
the reason every component must be designed without images too (§5.6). The images in the
design package are working material, not redistributable.

**The wiki text is CC BY-SA 4.0**, from *bindingofisaacrebirth.wiki.gg*. The license
requires attribution and share-alike: source, URL, license, and **snapshot date** must be
shown, not just declared in a file somewhere. The date matters to the user too: it says how
old what they're reading is relative to the game patch they have installed (§4.3).

**The three promises**, which deserve to be designed for and not hidden in a footer:

- **Read-only on saves.** Not "we're being careful": the module that opens `.dat` files
  contains no write code, by construction, and the checksum is never recalculated.
- **No account, no backend, no telemetry.** The app works offline; the network is only
  used for an optional dataset update, one the user may never even trigger.
- **A single file written**, `isaacdome.db`, in the app's data folder. Nothing else on the
  user's disk is touched.

What's left is listing the third-party libraries and their licenses (SQLite, miniz, Reka
UI, Tailwind, Lucide…). It's a list generated from `Cargo.lock` and `pnpm-lock.yaml` when
preparing the release, not something to write by hand now: what design needs to know is
**that it needs room for a long list**, not which lines it will contain.
