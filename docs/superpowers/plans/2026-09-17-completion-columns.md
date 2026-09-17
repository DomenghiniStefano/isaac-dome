# The Completion screen counts twice — implementation plan

> **For agentic workers:** the steps are checkboxes because they are executed in order, one
> test cycle each. This repo executes plans **inline**, not one subagent per task.

**Goal:** close B22 item 4 and B23 — the marks grid grows a second number column (normal and
hard, each over the readable cells) and the KPI strip loses the tile that counts cells.

**Architecture:** the reading is a pure function in `ui/src/lib/completion/completionView.ts`
with its own Vitest coverage; the grid and the strip only draw what it returns. The two
columns are two grid tracks in `grid-cols-matrix`, so the group header's totals and the
footer's land under their own heading instead of being labelled twice.

**Tech Stack:** Vue 3 SFC, Tailwind v4 (`@utility`, `@theme` tokens), vue-i18n, Vitest.

**Spec:** `docs/BACKLOG.md`, entries **B22** (items 1–3 landed 2026-09-17, item 4 open) and
**B23**. The layout itself is *not* in `design-export/design-system/Schermate.dc.html` — that
file has one total column, which is why B22 item 4 says "layout from Schermate.dc.html, which
today has one column". The decisions below are this plan's, and they are the part a window
has to judge.

## Global constraints

- **No `<style>` in SFCs**, no hardcoded visual constants: a width is a token in `@theme`,
  the column track is `@utility grid-cols-matrix` in `ui/src/assets/utilities.css`.
- **No string unions**: `const X = { … } as const`, never `type X = 'a' | 'b'`.
- **No visible string in a template**: everything through `t()`, keys in both `it.ts` and
  `en.ts` (`en` is typechecked against `it`'s schema).
- **Test-first**, and the expected value comes from the entry, never from the code's output.
  B22's own "Done when" is the source: *a row with twelve hard marks reads 12/12 · 12/12; a
  row with value 2 in one cell reads 1/12 · 1/12, not 0 and 1.*
- `pnpm check` green before anything is called done, and the UI test floor in
  `scripts/test-floor` raised in the same commit that adds the tests.

## Decisions this plan takes (what a spec would hold)

1. **Each column carries its own denominator.** `12/12 · 12/12`, not `12/12 · 12`. It is what
   B22's "Done when" writes, and it is what lets the two columns be read independently once
   they are no longer side by side in one slot.
2. **The header names them once**: `normale` and `hard` over the two tracks, replacing the
   single `normale · hard` label. The words are the legend's and the KPI strip's, so the
   screen says "hard" one way only.
3. **The group header's two numbers sit in two `w-matrix-total` right-aligned spans**, so
   they land under the two headings — the group header is a sibling of the rows inside the
   same `w-max` column, so the alignment is free. The unreadable count moves to the left of
   the pair, where it stops being mistaken for a third total.
4. **The footer becomes two rows, not two numbers per cell.** A boss column is 40px: a
   stacked pair fits but nothing says which line is which. Two rows put the answer in the
   name column that is already there — `Personaggi con il marchio` / `Di cui in hard`, which
   also states the subset relation B22 exists for.
5. **The done colour is per number, not per row.** A number that equals its denominator is
   done; `0/0` is not done, it is unreadable. Today one colour is decided by
   `tally.complete` (hard everywhere) and painted on the single slot. Per number, Isaac reads
   full at normal and short at hard, which is exactly what he is.
6. **The strip is three tiles**, not four: marks at normal, marks at hard, complete
   characters. B23 offers a fourth ("characters complete at normal") and this plan declines
   it — B22 has just made `complete` mean hard everywhere, and losing Isaac to it is the
   point of the change; a second, weaker complete tile beside it gives the number back on the
   same line and makes the strip argue with itself.
7. **`unknown` and `cells` leave `CompletionKpis`.** B23 keeps the unreadable count in the
   model, and it stays where it is *read*: every group header prints its own, every cell says
   it, and the alert above the grid fires when nothing at all is readable. A field on the KPI
   interface that no tile draws is the shape B42 is an entry about.

---

### Task 1: the two columns are a reading, not a format string

**Files:**
- Modify: `ui/src/lib/completion/completionView.ts`
- Test: `ui/src/lib/completion/completionView.test.ts`

**Interfaces:**
- Consumes: `Tally` (`{ normal, hard, readable, complete }`), already there.
- Produces: `TallyTone`, `TallyColumn`, `TallyColumns`, `tallyColumns(tally: Tally):
  TallyColumns` — Task 2 draws them.

- [ ] **Step 1: write the failing tests** — append to `completionView.test.ts`:

```ts
describe('tallyColumns', () => {
  // B22's "Done when": a row with twelve hard marks reads 12/12 · 12/12, and a row with a
  // bare 2 in one cell reads 1/12 · 1/12 — not 0 and 1. Both numbers carry the denominator
  // because each column is read on its own.
  it('gives every readable cell to both columns when the row is hard everywhere', () => {
    expect(tallyColumns(rowTally(row('Magdalene')))).toEqual({
      normal: { value: 12, readable: 12, tone: TallyTone.Full },
      hard: { value: 12, readable: 12, tone: TallyTone.Full },
    })
  })

  it('is full at normal and short at hard one mark from the end', () => {
    expect(tallyColumns(rowTally(row('Isaac')))).toEqual({
      normal: { value: 12, readable: 12, tone: TallyTone.Full },
      hard: { value: 11, readable: 12, tone: TallyTone.Partial },
    })
  })

  it('counts a bare 2 in both columns', () => {
    const cells: Cell[] = row('Isaac').cells.map((_, i) => ({
      kind: 'known',
      bits: i === 0 ? 2 : 0,
    }))
    expect(tallyColumns(rowTally({ ...row('Isaac'), cells }))).toEqual({
      normal: { value: 1, readable: 12, tone: TallyTone.Partial },
      hard: { value: 1, readable: 12, tone: TallyTone.Partial },
    })
  })

  // 0 === 0 is not "full": a row we cannot read has to look unreadable, not finished.
  it('calls a row with nothing readable unreadable, never full', () => {
    const cells: Cell[] = row('Isaac').cells.map(() => ({ kind: 'unknown' }))
    expect(tallyColumns(rowTally({ ...row('Isaac'), cells }))).toEqual({
      normal: { value: 0, readable: 0, tone: TallyTone.Unreadable },
      hard: { value: 0, readable: 0, tone: TallyTone.Unreadable },
    })
  })
})
```

Add `TallyTone` and `tallyColumns` to the import block at the top of the file.

- [ ] **Step 2: run them and watch them fail**

Run: `pnpm --filter ui test -- --run completionView`
Expected: FAIL, `tallyColumns is not a function` / no export `TallyTone`.

- [ ] **Step 3: implement, under `rowTally`/`columnTallies` in `completionView.ts`**

```ts
// What a number is worth reading as: full, on its way, or not readable at all. `0/0` is the
// case the tone exists for — an equality test alone would call an unreadable row finished.
export const TallyTone = {
  Full: 'full',
  Partial: 'partial',
  Unreadable: 'unreadable',
} as const
export type TallyTone = (typeof TallyTone)[keyof typeof TallyTone]

export interface TallyColumn {
  value: number
  readable: number
  tone: TallyTone
}

export interface TallyColumns {
  normal: TallyColumn
  hard: TallyColumn
}

const toneOf = (value: number, readable: number): TallyTone => {
  if (readable === 0) return TallyTone.Unreadable
  return value === readable ? TallyTone.Full : TallyTone.Partial
}

const column = (value: number, readable: number): TallyColumn => ({
  value,
  readable,
  tone: toneOf(value, readable),
})

// B22 item 4: two columns, each over the readable cells. The denominator is stated twice
// because the columns are read apart — `hard` under its own heading is not "the number after
// the dot", it is how many bosses reached the second level.
export const tallyColumns = (tally: Tally): TallyColumns => ({
  normal: column(tally.normal, tally.readable),
  hard: column(tally.hard, tally.readable),
})
```

- [ ] **Step 4: run them and watch them pass**

Run: `pnpm --filter ui test -- --run completionView`
Expected: PASS, 4 new tests.

---

### Task 2: the grid draws two columns (B22 item 4)

**Files:**
- Modify: `ui/src/assets/utilities.css:27-32` (`@utility grid-cols-matrix`)
- Modify: `ui/src/components/marks/MarksGrid.vue`
- Modify: `ui/src/i18n/messages/it.ts` (`completion.grid`), `ui/src/i18n/messages/en.ts`

**Interfaces:**
- Consumes: `tallyColumns`, `TallyTone`, `TallyColumn` from Task 1.
- Produces: nothing downstream; the screen already renders this component.

- [ ] **Step 1: the track** — in `ui/src/assets/utilities.css`, the matrix utility gains a
      second total column and its comment says what the two are:

```css
/* The completion matrix: the name column, one mark cell per boss, then the row's two totals —
   how many bosses have a level at all, and how many have the second (B22). The column count is
   data, bound from the template as --matrix-columns. */
@utility grid-cols-matrix {
  grid-template-columns:
    var(--spacing-matrix-name)
    repeat(var(--matrix-columns), var(--spacing-mark-cell))
    var(--spacing-matrix-total)
    var(--spacing-matrix-total);
}
```

- [ ] **Step 2: the words** — in `it.ts`, `completion.grid` loses `levels` and gains three
      keys; `en.ts` the same:

```ts
// it.ts
grid: {
  character: 'Personaggio',
  normal: 'normale',
  hard: 'hard',
  unreadable: 'non leggibili',
  columnTotals: 'Personaggi con il marchio',
  columnTotalsHard: 'Di cui in hard',
},
```

```ts
// en.ts
grid: {
  character: 'Character',
  normal: 'normal',
  hard: 'hard',
  unreadable: 'unreadable',
  columnTotals: 'Characters with the mark',
  columnTotalsHard: 'Of those, at hard',
},
```

- [ ] **Step 3: the component.** In `MarksGrid.vue`'s script, `tallyClass`/`tallyLabel` go and
      a tone record takes their place:

```ts
import {
  CellStatus,
  MatrixGroup,
  TallyTone,
  cellReading,
  columnTallies,
  matrixGroups,
  tallyColumns,
} from '@/lib/completion/completionView'
import type { TallyColumn } from '@/lib/completion/completionView'

// Complete in the done colour, nothing readable fainter than a number with a denominator.
// Gold is not used here: gold means unlockable now, and nothing else. A record over the whole
// set, so a tone with no colour fails to compile.
const toneClass: Record<TallyTone, string> = {
  [TallyTone.Full]: 'text-state-done-foreground',
  [TallyTone.Partial]: 'text-subtle-foreground',
  [TallyTone.Unreadable]: 'text-faint-foreground',
}

const columnsOf = (tally: Tally) => tallyColumns(tally)
const label = (c: TallyColumn): string => `${c.value}/${c.readable}`
```

**`GroupView` carries its `Tally`, not three loose numbers.** `groupView` already computes
`tallyOf(cells)` and then spreads it into `normal`, `hard` and `readable`; the header needs
the same pair of columns a row needs, and `tallyColumns` takes a `Tally`. So in
`completionView.ts`:

```ts
export interface GroupView {
  group: MatrixGroup
  rows: GroupRow[]
  first: string
  last: string
  // The group's own cells, read the way a row's are: the header draws tallyColumns(tally).
  tally: Tally
  unknown: number
}
```

and `groupView` returns `{ group, rows, first, last, tally, unknown: cells.filter(isUnknown).length }`.
`matrixGroups`'s existing test asserts the group's numbers — update it to read `tally` in the
same step, expected values unchanged.

- [ ] **Step 4: the template.** Header row — the single trailing `<span>` becomes two:

```html
<span class="justify-self-end pr-0.5 text-label text-subtle-foreground">{{
  t('completion.grid.normal')
}}</span>
<span class="justify-self-end pr-0.5 text-label text-subtle-foreground">{{
  t('completion.grid.hard')
}}</span>
```

Group header — the unreadable count moves before the totals, and the two totals are
right-aligned in the width of their own columns so they land under the headings:

```html
<span
  v-if="group.unknown > 0"
  class="ml-auto text-label text-state-unknown-foreground tabular-nums"
  >{{ group.unknown }} {{ t('completion.grid.unreadable') }}</span
>
<span
  :class="
    cn(
      'w-matrix-total pr-0.5 text-right text-label tabular-nums',
      group.unknown > 0 ? '' : 'ml-auto',
      toneClass[columnsOf(group.tally).normal.tone],
    )
  "
  >{{ label(columnsOf(group.tally).normal) }}</span
>
<span
  :class="
    cn(
      'w-matrix-total pr-0.5 text-right text-label tabular-nums',
      toneClass[columnsOf(group.tally).hard.tone],
    )
  "
  >{{ label(columnsOf(group.tally).hard) }}</span
>
```

Row — the single tally span becomes two, written out. A `v-for` over a two-element list would
save one line of class string and cost the reader the answer to "which column is this":

```html
<span
  :class="
    cn(
      'justify-self-end pr-0.5 text-label tabular-nums',
      toneClass[columnsOf(entry.tally).normal.tone],
    )
  "
  >{{ label(columnsOf(entry.tally).normal) }}</span
>
<span
  :class="
    cn(
      'justify-self-end pr-0.5 text-label tabular-nums',
      toneClass[columnsOf(entry.tally).hard.tone],
    )
  "
  >{{ label(columnsOf(entry.tally).hard) }}</span
>
```

Footer — one row becomes two, each with its own name in the left column and an empty span per
total column:

```html
<div
  class="mt-2 grid grid-cols-matrix items-center justify-items-center gap-0.5 border-t border-border pt-1.5"
>
  <span class="justify-self-start pr-1.5 text-label text-highlight">{{
    t('completion.grid.columnTotals')
  }}</span>
  <span
    v-for="(tally, b) in totals"
    :key="b"
    :class="cn('text-micro tabular-nums', toneClass[columnsOf(tally).normal.tone])"
    >{{ label(columnsOf(tally).normal) }}</span
  >
  <span />
  <span />
</div>
<div class="grid grid-cols-matrix items-center justify-items-center gap-0.5 pt-1">
  <span class="justify-self-start pr-1.5 text-label text-subtle-foreground">{{
    t('completion.grid.columnTotalsHard')
  }}</span>
  <span
    v-for="(tally, b) in totals"
    :key="b"
    :class="cn('text-micro tabular-nums', toneClass[columnsOf(tally).hard.tone])"
    >{{ label(columnsOf(tally).hard) }}</span
  >
  <span />
  <span />
</div>
```

- [ ] **Step 5: the gates**

Run: `pnpm typecheck && pnpm ui:test && pnpm lint && pnpm scan`
Expected: PASS. `scan` is the one that catches a hardcoded pixel or a bare string slipped into
the template.

- [ ] **Step 6: look at it** — `pnpm ui:dev`, Completion. **Not `pnpm dev`**: the Tauri launch
      spends the three one-shot checks in `docs/STATUS.md` ("First, and it happens once") and
      nobody is reading them. Check: the two headings sit over their columns; a group's two
      numbers land under them; the footer's two rows say which is which; Isaac is full at
      normal and short at hard.

- [ ] **Step 7: commit**

```bash
git add ui/src/lib/completion/completionView.ts ui/src/lib/completion/completionView.test.ts \
        ui/src/assets/utilities.css ui/src/components/marks/MarksGrid.vue \
        ui/src/i18n/messages/it.ts ui/src/i18n/messages/en.ts scripts/test-floor
git commit -m "feat(ui): the matrix counts twice, normale and hard, each over what it can read"
```

---

### Task 3: the strip says the three things the rows say (B23)

**Files:**
- Modify: `ui/src/lib/completion/completionView.ts` (`CompletionKpis`, `completionKpis`)
- Modify: `ui/src/lib/completion/completionView.test.ts`
- Modify: `ui/src/screens/completion/CompletionKpis.vue`
- Modify: `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts`

**Interfaces:**
- Consumes: nothing from Task 2.
- Produces: `CompletionKpis` without `unknown` and `cells`; `CompletionScreen.vue` keeps
  reading `kpis.readable` for its alert and needs no change.

- [ ] **Step 1: change the test first** — in `completionView.test.ts`, the reference
      expectation loses two fields, and the comment above it keeps the counted numbers it
      still uses:

```ts
describe('completionKpis on the reference profile', () => {
  it('states the totals §5.4 printed, split in two, and no percentage', () => {
    expect(completionKpis(reference)).toEqual({
      normal: 166,
      hard: 152,
      readable: 368,
      completeCharacters: 2,
      characters: 34,
    })
  })
})
```

- [ ] **Step 2: run it and watch it fail**

Run: `pnpm --filter ui test -- --run completionView`
Expected: FAIL — the received object still holds `unknown: 40` and `cells: 408`.

- [ ] **Step 3: the model.** In `completionView.ts`, `CompletionKpis` loses `unknown` and
      `cells`, `completionKpis` stops computing them, and the comment says where the
      unreadable count went rather than leaving it looking dropped:

```ts
// The strip splits the same way the rows do. `both` went with `CellStatus.Both` (B22), and
// the unreadable count went with B23: a tile that says "40 / 408 celle" puts a gap in our own
// tables on the same line as the player's progress. It is not lost — every group header
// prints its own, every unreadable cell says so, and the alert above the grid speaks when
// nothing at all can be read.
export interface CompletionKpis {
  normal: number
  hard: number
  readable: number
  completeCharacters: number
  characters: number
}
```

`const cells = matrix.characters.flatMap((r) => r.cells)` stays — `tallyOf(cells)` needs it —
and `isUnknown` stays exported to `groupView`, which still counts it.

- [ ] **Step 4: the strip.** `CompletionKpis.vue` drops the fourth tile and the grid becomes
      three columns; the comment above the template loses "four numbers of four natures":

```html
<!-- Three numbers of three natures, each with its own denominator and none fused into a
     percentage (DESIGN-BRIEF.md §5.3); the explanations live in the tooltips. The count of
     unreadable cells is not one of them (B23): it is a gap in our tables, not a fact about
     the player, and the grid says it where it happens. -->
<div class="grid grid-cols-3 gap-2.5">
```

The `KpiTone` import stays (the complete tile uses `KpiTone.Done`); the fourth `KpiTile` block
and its `#explain` slot go.

- [ ] **Step 5: the words.** In `it.ts` and `en.ts`, `completion.kpi` loses `cells`,
      `unknown` and `unknownExplain`. Nothing else reads them — checked with
      `grep -rn "kpi.unknown\|kpi.cells" ui/src`, which returns only the tile being deleted.

- [ ] **Step 6: run the gates**

Run: `pnpm typecheck && pnpm ui:test && pnpm lint && pnpm scan`
Expected: PASS. A missed key is a typecheck error, not a silent blank, because `en.ts` is
checked against `it.ts`'s schema.

- [ ] **Step 7: look at it** — `pnpm ui:dev`, Completion: three tiles across the same width,
      the strip no longer says "non leggibili", the group headers still do.

- [ ] **Step 8: commit**

```bash
git add ui/src/lib/completion/completionView.ts ui/src/lib/completion/completionView.test.ts \
        ui/src/screens/completion/CompletionKpis.vue \
        ui/src/i18n/messages/it.ts ui/src/i18n/messages/en.ts
git commit -m "feat(ui): the Completion strip counts marks and characters, not cells"
```

---

### Task 4: the record, and the floor

**Files:**
- Modify: `scripts/test-floor` (`UI_TESTS`)
- Modify: `docs/BACKLOG.md` (B22 heading and item 4, B23 closed)
- Modify: `docs/STATUS.md` (the M1 line, and the window checks this produces)
- Create: `docs/superpowers/reports/2026-09-17-completion-columns-report.md`

- [ ] **Step 1: the whole suite**

Run: `pnpm check`
Expected: green, and a line printed for the raised UI test count. Paste it into
`scripts/test-floor`. `cargo` is untouched by this sub-project, so `RUST_TESTS` must not move
— if it does, something was deleted and that is the finding, not a number to update.

- [ ] **Step 2: close the entries in `docs/BACKLOG.md`.** B22's heading becomes
      `✅ closed on 2026-09-17, built and **not yet seen in a window**` and item 4 gets its
      ✅ with what was decided (each column its own denominator, the footer's two rows, the
      per-number colour). B23 the same, with decision 6 above written where the entry asks the
      question — the fourth tile was offered and declined, and why.

- [ ] **Step 3: add the checks to `docs/STATUS.md`**, under *"What only a window can say"*, as
      a group of its own. They are the part this plan cannot answer:
      - the two headings sit over their columns at every window width, and the strip scrolls
        rather than crushing the name column
      - a group's two numbers land under the two headings, not beside them
      - the footer's two rows read as one statement and not as two tables
      - Isaac reads full at normal and short at hard, and the colours say so
      - three tiles fill the strip's width as four did

- [ ] **Step 4: the report**, `docs/superpowers/reports/2026-09-17-completion-columns-report.md`:
      what was decided and why, what the tests pin, and what is left to a window. A report is
      true on its day and is never re-checked — the live list is STATUS's.

- [ ] **Step 5: commit the documents**

```bash
git add docs/BACKLOG.md docs/STATUS.md docs/superpowers/plans/2026-09-17-completion-columns.md \
        docs/superpowers/reports/2026-09-17-completion-columns-report.md scripts/test-floor
git commit -m "docs: the Completion screen counts twice, and what only a window can say grows by five"
```

- [ ] **Step 6: merge into `develop`** — `--no-ff` from `feature/completion-columns`, with
      `pnpm check` green, then delete the branch locally and on the remote after verifying at
      that moment that it holds nothing `develop` does not
      (`git rev-list --count develop..<b>` **and** `git rev-list --count <b> --not --remotes`).
      **Stop there: `master` is frozen.**

- [ ] **Step 7: move the two cards** on the Trello board from `In Progress` to `UAT` with
      `NEEDS WINDOW`, so the board and this file say the same thing.
