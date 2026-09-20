# 3.13a — the frame: a screen fills the window, and folds when it can't

Every screen in `ui/` is drawn at one width and scrolls as one block. This sub-project builds the
frame that lets twenty-two of them stop doing that, and migrates one screen on it as the proof.

**It is the first of three.** The whole job — the frame, then the screens — does not fit in one
spec, and the frame has to exist before a screen migration means anything:

- **3.13a, this one** — the page box, the height chain, the thresholds, the sidebar's collapsed
  state, the window floor, the rules that hold it, and **Unlock** migrated entire.
- **3.13b** — the five remaining list screens: Collection, Challenges, Runs, Search, the wiki's
  category lists.
- **3.13c** — the cards, the forms, the wiki page, the welcome.

The sidebar's open/close button is **not** in any of the three. It is its own card, and §6 says
what this cycle leaves ready for it.

Decided in conversation on 2026-09-20. Five choices are the owner's and are marked as such: §3,
§4, §6, §7 and §8.

---

## 1. What is already built, read rather than assumed

- **Every screen opens the same way**: `<div class="flex max-w-250 flex-col gap-4">`. Twenty-two
  files, three values — `max-w-250` everywhere, `max-w-300` on `PlanScreen.vue`, `max-w-190` on
  `GoalsScreen.vue`. At scale 100 that is 1000, 1200 and 760 device pixels. Nothing centres it, so
  a wide window is content on the left and empty space on the right.
- **`ui/` holds eight media-query variants, in four files**: `FloorScreen.vue` (2 lines),
  `PlanScreen.vue` (2), `LiveScreen.vue` (1), `RunsScreen.vue` (1). Two of them — `lg:flex-row` on
  Floor and Plan — are already the shape this spec generalises: *when there is room, the aside goes
  beside instead of under*. They are the right idea measured against the wrong thing (§2).
- **The container-query precedent exists and is ours.** `TabItem.vue` is a `@container` and reads
  `--container-tab-narrow` from `ui/src/assets/theme/containers.css`, whose comment already records
  the rule this spec adopts wholesale: the number is in px, **measured at scale 100**, because a
  container query is compared against a used width.
- **The sidebar's width travels as a CSS variable, not as an inline pixel.**
  `SectionSidebar.vue` binds `--sidebar-width` in `:style` and the element carries
  `w-(--sidebar-width)`. The style sets the *variable*; the width is a class. A container variant
  therefore overrides it by ordinary cascade, with no `!important` and no JavaScript. §6 is free
  because of a decision taken in 3.5c for another reason.
- **The scroll body of every virtualized list is a fixed 35rem.** `VirtualRows.vue` carries
  `max-h-virtual-rows-body`, and `--spacing-virtual-rows-body: 35rem` sits in
  `ui/src/assets/theme/spacing.css:41`. Six components go through it: `UnlockTable`,
  `CollectionTable`, `ChallengesTable`, `RunsTable`, `SearchResults`, `WikiCategoryList`.
- **`<main>` is what scrolls today**: `min-w-0 flex-1 overflow-auto px-5.5 pt-5 pb-15` in
  `App.vue`, and the screen is a block growing inside it.
- **A table row is a CSS grid whose template is a named utility.** `UnlockTable.vue` uses
  `grid-cols-unlock` on both the header (line 29) and the row (line 54), and it is one of seven
  `@utility grid-cols-*` declarations in `ui/src/assets/utilities.css` — a `@utility`, not a
  `@theme` token, because a grid template is a rule and not a value. This is what makes §7 two
  edits rather than one.
- **The interface's scale is the root's font size.** `lib/scale/apply.ts` writes `--app-scale` on
  the document element and the root is `calc(16px * var(--app-scale))`, so every `rem` token
  follows the user's Ctrl+`+`. **Media queries do not follow it** — `rem` in a media query resolves
  against the initial 16px, never against the root — while container queries do. At scale 150 a
  media-query layout does not fold; a container-query layout folds sooner, which is correct.
- **No window declares a minimum.** `crates/app/tauri.conf.json` gives the window `width: 1280,
  height: 800` and nothing else. And windows are born in **two** places, not one:
  `crates/app/src/window.rs` builds from that config (first launch, tray, second launch), while a
  torn-off tab is built in the frontend, `new WebviewWindow` at
  `ui/src/lib/window/windowPort.ts:150`. A floor written in one place covers every window except
  the ones the user makes by hand.

## 2. Decision — a screen measures its own box, never the window

Every threshold in this design is a **container query** against a named size container. Media
query variants (`sm:` `md:` `lg:` `xl:` `2xl:`) are banned from `ui/`, and the eight that exist are
converted in this cycle so the rule starts at zero violations.

Two facts decide it, and either one alone would:

- **The window's width is not the content's.** The section sidebar is resized by the user between
  168 and 420 px (`components/shell/sidebarWidth.ts`). That is 252 px of difference, on a 640 px
  floor, that a media query cannot see: with the sidebar wide, a 1280 window leaves the content 860
  and a media query still calls it 1280.
- **A media query cannot see the interface's scale**, per §1. Scale is not a corner case here; it
  is a slider in Settings and three shortcuts, and at 150 % a fold that never happens is a layout
  broken in exactly the situation that needed it most.

**Rejected — a layout store in JavaScript.** A `ResizeObserver` on the page box, a composable
exposing the width, screens branching with `v-if`. It buys two real things: thresholds become pure
functions under Vitest, and a dropped column is not rendered at all rather than hidden. It costs an
observer per screen, reactivity churn through every frame of a sidebar drag, and a JavaScript
reimplementation of something the browser does natively. Kept in reserve for a single case, if one
turns up, where hiding a column measurably costs; not the frame.

**Rejected — media queries with tokenized breakpoints** (redefining `--breakpoint-*` in `@theme`
and using `sm:`/`md:`). It is the obvious road and it is wrong here for both reasons above. It is
written down because it is what the next person will reach for.

## 3. Decision (owner) — full width, with no reading cap anywhere

`max-w-*` leaves the root of all twenty-two screens and nothing replaces it. Data screens, forms,
settings and **the wiki page** all fill the box.

The cost is stated rather than discovered: on a 2560 px window a wiki paragraph is a line of some
200 characters, which is past what prose is comfortable at. The owner chose this on 2026-09-20 over
the alternative of capping prose and centring it. It is a decision, not an oversight, and a future
reading cap is a change to this section — not a bug fix.

## 4. Decision (owner) — the page box scrolls nothing, and a screen is one of two shapes

`<main>` becomes the **page box**: a flex column, `min-h-0`, `overflow-hidden`, carrying the
horizontal padding only. The vertical padding moves into the screen, because on a screen that fills
its height today's `pb-15` would be sixty pixels of nothing under a list that could have used them.
`<main>` is also the size container named `page` (`@container/page`) that every threshold in §5 is
measured against.

Under it, **two shapes, and every screen declares which one it is**:

| shape | root classes | who |
|---|---|---|
| **flowing** | `h-full overflow-y-auto` | Settings, Profile, Appearance, Goals, Updates, the wiki page — content flows, the screen scrolls |
| **filling** | `flex h-full min-h-0 flex-col overflow-hidden`, and **exactly one** descendant with `min-h-0 flex-1` | every screen with a list: the region marked `flex-1` takes the height that is left, and scrolls inside |

In `VirtualRows.vue`, `max-h-virtual-rows-body` becomes `min-h-0 flex-1` and
`--spacing-virtual-rows-body` is deleted from `spacing.css`.

**`min-h-0` on every link of the chain is not decoration.** A flex item's default `min-height:auto`
refuses to shrink below its content, so one missing `min-h-0` between the page box and the scroll
body makes `flex-1` grow instead of fit, and the list pushes the window instead of scrolling inside
it. It fails silently and it looks like the virtualizer's fault. Every filling screen's chain is
checked link by link during migration.

Two shapes and not twenty-two is what makes this checkable: "the root of a screen is one of these
two class sets" is a fact about one line of one template, and §9 makes a script read it.

## 5. Decision — three shared thresholds, named containers, private tokens only where measured

`containers.css` gains three tokens beside `--container-tab-narrow`:

| token | what crossing it means |
|---|---|
| `--container-compact` | below: one column; tables at their minimum column set; KPI strips stacked |
| `--container-regular` | the shape the screens are drawn at today |
| `--container-wide` | above: what sat under something may sit beside it — this is what Floor's and Plan's `lg:flex-row` become |

**Both containers carry a name, and every variant names the one it means**:
`@max-compact/page:hidden`, `@wide/page:flex-row`, `@max-compact/shell:w-sidebar-icons`. Tailwind
resolves an unnamed variant against the *nearest* container ancestor, which today would be right in
both cases — and would go wrong, silently and with nothing red, the first time any component
between a screen and `<main>` declares a `@container` of its own. A card that becomes a container
for its own reasons would take every `@max-compact:` inside it and measure it against the card.
That is the exact shape of failure this repo writes rules against, and the cost of avoiding it is
six characters per class.

Three sizes and not seven. A shared scale is the only thing that keeps twenty-two screens telling
the same story, and each extra step is another state nobody will ever look at.

A component whose break is a fact about **itself** rather than about the page — the tab is the one
that exists — declares its own token next to itself, with a comment naming the scale it was
measured at. That stays allowed and stays the exception.

**The three values are not in this spec.** They are measured on the densest screen during the
migration of §10, and the comment on each token records what it was measured against. A number
chosen at a desk here would be the same mistake as naming a `.dat` section from a guess.

## 6. Decision (owner) — the sidebar collapses to icons, in CSS, on an input nobody writes yet

The `<div class="flex min-h-0 flex-1">` that holds the sidebar and `<main>` becomes the second
size container named `shell` (`@container/shell`). **Its width is the window's and does not change when the sidebar
collapses** — which is the whole reason the threshold hangs there and not on `page`. Hung on
`page`, a collapse would widen the content, re-cross the threshold and oscillate.

The `<aside>` becomes `w-(--sidebar-width) @max-compact/shell:w-sidebar-icons`. Inside: labels take
`@max-compact/shell:hidden`, icons centre, every entry gains a tooltip carrying its name, and the
resize handle is hidden — dragging the edge of a collapsed sidebar means nothing.

**The collapse never writes the stored width.** It is a state that overrides it; widening the
window gives back the number the user chose.

The state has two inputs and both are CSS: collapsed when `@max-compact/shell` **or** when the
shell root carries `data-sidebar="collapsed"`. **Nothing writes that attribute in this cycle.** The
button is a separate card, and it will write that attribute and nothing else — no part of this
sub-project is reopened by it.

One consequence, stated because it is correct and will read like a defect: under the threshold the
button will not be able to re-open the sidebar. There is no room. That is the floor, not a broken
button.

## 7. Decision (owner) — a column drops by two edits, kept adjacent

Per table: a **pair of `@utility` declarations, adjacent in `utilities.css`** — `grid-cols-unlock`
and `grid-cols-unlock-narrow` — with a comment naming which columns fall and why those; and
`@max-compact/page:hidden` on the cells that fall, **in the header and in the row**, which in
`UnlockTable.vue` are twenty-five lines apart in one file.

**Rejected — collapsing the track to `0px` instead of hiding the cell.** It is tempting: the
narrow template becomes the single source of truth and the two edits become one, with no way to
drift. It is rejected because a zero-width cell stays in the accessibility tree, and a screen
reader would read columns the eye was told it could do without. Two edits to keep in step beat a
phantom column.

§9 takes the mechanical half of the risk. The rest is looked at, on the Kit page, like all
presentation in this repo.

## 8. Decision (owner) — the floor is 640 × 480, declared in both places

`minWidth: 640, minHeight: 480` goes into the window config in `crates/app/tauri.conf.json` **and**
into the options at `ui/src/lib/window/windowPort.ts:150`, so a torn-off window has the same floor
as the one it came from. Below it the operating system does not let the user go, so there is no
undesigned state to reason about.

At the floor, with the sidebar at its 168 minimum and the page box's horizontal padding, the
content has roughly 450 px at scale 100 — and less at larger scales, which the container queries
see. That is the width the compact layouts are designed against.

`ui/src/lib/window/preview.ts` opens a third window. It is examined during migration: if it is
deliberately small it keeps no floor, and the reason is written at the call.

## 9. What keeps it from coming back

No linter enforces any of this. `ui/scripts/scan-conventions.mjs` does, and **a rule added to the
document without a rule added to the script is a promise of a check that never happens** — which
`CLAUDE.md` already names as the thing not to do. Four rules, each with its own case in the
script's own suite, in order of how much they hold:

1. **No media-query variant** (`sm:` `md:` `lg:` `xl:` `2xl:`) anywhere in `ui/`. The most valuable
   of the four: it is §2's rejected approach, and it is what fingers type unprompted. The eight
   existing occurrences are converted in this cycle so the rule lands green.
   **And its other half**: `containers.css` does not reset Tailwind's own `--container-*` scale, so
   `@max-md/page:` is writable today and would slip past §5 entirely — a fourth threshold nobody
   declared, measured against a number nobody chose. The rule therefore also refuses a container
   variant whose size is not one of ours (`compact`, `regular`, `wide`, or a component's own
   declared token). Whether the cleaner answer is to overwrite the default scale to nothing in
   `@theme` instead of policing it in a script is left to the migration, which can try it and see
   what it breaks.
2. **A screen's root is one of §4's two shapes.**
3. **No `max-w-*` on a screen's root.** §3 removed the cap by decision; this stops it returning by
   habit from the screen next door.
4. **A file using a `grid-cols-*-narrow` utility and containing no `@max-compact/page:hidden`** has
   done half of §7. It does not prove the right columns fell; it proves both edits were made.

Rules 2 and 3 need the screen roots enumerated, so they are keyed on `ui/src/screens/**` and the
`EXEMPTIONS` array carries anything that legitimately differs, with its reason, per existing
practice.

## 10. What this cycle contains

- The floor, in both places (§8).
- `<main>` as the page box and the `page` container; the shell wrapper as the `shell` container.
- The two screen shapes, and the height chain through `VirtualRows.vue`;
  `--spacing-virtual-rows-body` deleted.
- The three threshold tokens, measured, with their comments.
- The sidebar's collapsed state, with `data-sidebar` left unwritten.
- The eight media-query variants converted.
- The four scan rules, each with its test.
- A new section in `docs/frontend-conventions.md`, and the five-rule summary in `CLAUDE.md`
  extended only if the count of non-negotiable rules changes.
- **`UnlockScreen` migrated entire**, as the proof: it is the only screen that exercises all three
  mechanisms at once — virtualized body, six columns, filter bar.

**Not in this cycle:** the other twenty-one screens (3.13b, 3.13c) and the sidebar's button.

**`docs/architecture.md` is not redrawn.** Its header pins crates, commands, events, routes and
migrations; this cycle changes none of the five. The rule says to redraw it in the same commit, so
"not needed" is recorded here rather than left to be inferred.

## 11. How it is verified

The frame is CSS, and CSS is looked at. Said plainly rather than dressed up as coverage:

- **Vitest** can hold one real thing: that the three tokens exist and are in increasing order,
  read from `containers.css?raw`. The precedent is `lib/scale/rows.test.ts`, which already reads
  `spacing.css?raw` and asserts against it.
- **The scan suite** holds §9's four rules, each with a positive and a negative case, per the
  script's existing shape.
- **The Kit page is the real verification, and container queries make it possible.** A screen at
  its compact width can be looked at **without resizing the window**: a 400 px box on `#kit` makes
  the component inside it genuinely fold. The cycle closes with Kit showing the migrated table at
  the three widths, side by side, and that is a thing a browser can be pointed at in one glance —
  which is more than the 3.9 defect got, where a header's totals landed 130 px past their columns
  and nothing red ever said so.
- **A window on it** is still the one check none of the above replaces, and the card carries the
  `NEEDS WINDOW` label until someone has dragged an edge.

## 12. What the migration has to measure, and write down

- The three threshold values, against the densest screen.
- Which Unlock columns fall at compact, in priority order, and the narrow grid template that
  matches them.
- Whether `preview.ts`'s window takes the floor or is exempt.
- Whether Tailwind's default `--container-*` scale is better overwritten to nothing in `@theme`
  than policed by §9's rule 1 — try it, and record what it breaks.
- Whether the page box's horizontal padding should itself shrink at compact — left open on
  purpose: it is a number to look at, not a decision to take on paper.
