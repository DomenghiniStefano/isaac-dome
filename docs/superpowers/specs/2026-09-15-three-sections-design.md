# Three sections — Progress, Tool, Wiki

**Status:** decided in conversation on 2026-09-15, on a request to reorder the navbar and
put a **Tool** section between Progress and the Wiki. The reordering was the request; the
precondition that makes Tool a section rather than a folder is the part worth disagreeing
with, and it is §1.

**Why now.** The Floor screen's F1 plan
(`docs/superpowers/plans/2026-09-15-floor-grid-and-rules.md`, Task 8 Step 7) had to write
this to get itself past the profile gate:

```ts
needsProfile: routeOrigin[name] === TabOrigin.Progress && name !== RouteName.Floor,
```

The plan is honest about why — *"`TabOrigin.Progress` means the screen is gated behind a
chosen profile. **That is wrong for this screen**"* — and it solves it with an exception.
An exception carved into a derivation is a report that the derivation's input is missing a
case. The input was missing a section.

---

## 1. A section is a precondition, not a folder

`DESIGN-BRIEF.md` §4 has always said this, and its two-section table was built on it: the
Wiki needs nothing, Progress needs the game and a chosen save. The claim was correct and the
inventory had drifted under it.

|  | **Progress** | **Tool** | **Wiki** |
|---|---|---|---|
| needs the game? | yes | no, it degrades to ids | no |
| needs a save chosen? | **yes** | **no** | **no** |
| reads | `.dat` + XML + `.a` | `log.txt`, the run archive, your drawing | the embedded dataset |

**Tool is the section of screens that answer without the save.** Three of them:

- **Live** — `crates/app/src/commands/runs.rs`'s `live` already handles
  `IpcError::NoActiveProfile` by returning `LiveGraph::NoProfile`, with a doc comment saying
  *"a missing profile and a missing game are two different sentences, and both leave the run
  on screen."* That branch was unreachable from the UI: `ProgressGate` replaced the screen
  before it could draw.
- **Runs** — the `runs` command takes `store`, `catalog` and `archive`, and neither the graph
  nor the profile. `RunsScreen.vue` carries the sentence *"the archive is not a view of the
  profile: it exists without one."* It was gated anyway.
- **Floor** — the user's own drawing plus cited placement rules. Nothing else.

What stays in Progress is exactly what opens the `.dat`: Next steps, Completion, Unlock,
Plan, Collection. The split is not a judgment call at any row, which is the evidence that
the line is real and not a label.

## 2. What this buys, stated as the thing that can be checked

`router/routes.ts` keeps the line it already had:

```ts
needsProfile: routeOrigin[name] === TabOrigin.Progress,
```

**with no exception, and that is the deliverable.** The F1 plan's `&& name !== RouteName.Floor`
is deleted rather than generalised. A screen added to Progress is gated because it reads the
save; one added to Tool is not because it does not. Whoever adds the next screen has to answer
that question to file it at all, which is the part a comment cannot enforce.

## 3. Order, and where the order lives

Progress, then Tool, then Wiki. `NavBar.vue` draws `Object.values(NavSection)`, so **the order
of the object's keys is the order on screen**: there is no index to keep in step, and moving a
section is moving its line. Pinned by a test, because the next reader will look for a number.

## 4. What does not change

- **`TabOrigin` stays the single source** of both the sidebar section and the gate. The
  alternative — a `routeNeedsProfile` table beside `routeOrigin` — was considered and dropped:
  two tables that must agree is the shape this spec exists to remove.
- **Paths move** (`/tool/live`, `/tool/runs`, `/tool/floor`) and **nothing breaks**, because a
  saved tab is a `TabLocation` — a route *name* and its query — never a path. No `store`
  migration.
- **Search still sits above the sections** and belongs to none (`sectionOfOrigin` returns
  `null`). Three instead of two changes nothing there.
- **Settings is still not a section.** It is a destination reached from the cog, and
  `navSectionOf` still returns `null` while it is open.

## 5. Tests

- **Every section lists every route of its origin in its sidebar.** This test existed for
  Settings alone; it is generalised to Progress, Tool and Settings. A route that exists and is
  in no sidebar is a screen reachable only by typing its path, and a count never said so.
  (The Wiki is excluded: its entries are categories of one route.)
- **The gate covers exactly the screens that read the save** — the two origin sets, named.
  Adding a screen breaks it, which is the point: the break is the question being asked.
- **The navbar draws Progress, Tool, Wiki, in that order.**

## 6. Out of scope, named

- **Floor's screen.** This sub-project brings Floor's *route* — name, path, title, origin,
  icon, sidebar entry — so that F1 lands on a structure instead of inventing one. The screen
  component and its entry in the `screens` record of `routes.ts` stay with F1; until then Floor
  renders `PlaceholderScreen`, like every screen not yet built.
- **The `routeArrives` entries for Runs and Live**, which still name the sub-project that was
  going to bring them although both are real. Stale before this change and untouched by it.
- **The `🔴 M4` traffic light** on rows 6 and 7 of `DESIGN-BRIEF.md` §4, same reason: it
  measures data availability, and M4's own pass owns it.
