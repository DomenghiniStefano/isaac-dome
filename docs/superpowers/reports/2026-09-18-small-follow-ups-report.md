# The small follow-ups — report (2026-09-18)

Branch `feature/small-follow-ups`, plan
`docs/superpowers/plans/2026-09-18-small-follow-ups.md`. No sub-project number: it closes
**seven of B12's ten** design-system follow-ups and **both halves of B65**, the palette's
`Ctrl+Enter`. There is no spec document — the two backlog entries are the spec, and where they
turned out to be wrong this report says so rather than the entry quietly changing shape.

Eight commits, one per item. `pnpm check` green on the branch: **1129 Rust and 652 UI tests**,
155 skips, 452 real files touched.

---

## What the entries got wrong, which is the part worth keeping

**Item 9 named a prop that does something else.** The entry asks for "`getValueLabel` with an
i18n string". On `reka-ui@2.10.4`, `ProgressRoot` has *two* of them: `getValueLabel` is rendered
as **`aria-label`** and defaults to `"40%"`, while the one rendered as **`aria-valuetext`** —
the text that replaces the value a screen reader computes — is `getValueText`. Implemented to
the letter, the item would have set a label the caller was already setting and left the bar
still announcing a percentage that counts the hatched segment as *not done*, which is the exact
lie the segment exists to prevent. Done as a `value-text` string prop wired to `getValueText`.

**And item 9 has no caller.** The Completion KPIs count against `readable` — B23 decided that
the unreadable cells are not part of the bar — so **no bar in the app has an unknown share
today**; the only hatched one is the Kit's. The prop is therefore exercised there, and the
primitive's comment says who has to pass it. A prop with one dev-only caller is worth the
sentence that explains why, which is why this paragraph exists.

**Item 1 was bigger than "both classes survive a merge".** With `opacity-muted` and
`opacity-disabled` both on the element, the two selectors have the same specificity: the winner
is whichever rule Tailwind emitted later in the stylesheet, so the caller's order decides
nothing at all. `cn()` merges them now, and because the token names *join* the built-in
`opacity` class group rather than replacing it, `opacity-0` still wins over a token in either
order. Nothing in `src/` composes two of them today — this is a guard, not a repair, and the
test says both halves.

**Item 5's last rule found two classes that had stopped existing.** `@theme` resets `--text-*`,
`--font-weight-*`, `--radius-*` and `--shadow-*` to `initial`, so `text-sm` and friends compile
to nothing. Two `text-sm` were in `ui/src/screens/goals/WantAnswer.vue`, inert since the scale
was reset, and nothing anywhere would ever have said so: they generate no CSS, they break no
type, and they render at the inherited size, which happens to be the same 0.875rem. They are
`text-body` now — the size they were already drawing at — so the screen does not move and the
class finally means something.

## What is in the tree

- **`cn()` knows opacity** (`lib/cn.ts`, `lib/design/themeKeys.ts`, two tests).
- **vue-i18n's flags** in `ui/vite.config.ts`. Measured on two builds, same command:
  `index` goes **687,984 → 679,234 bytes** and `createVueI18n`, the legacy factory, stops
  appearing in the bundle. `__INTLIFY_DROP_MESSAGE_COMPILER__` stays at its default and the
  config says why — dropping the compiler needs messages pre-compiled to AST, and ours are TS
  objects whose `{named}` and `a | b` are read at runtime.
- **`Button` is `type="button"`** unless the caller says otherwise (`buttonType.ts`, four
  tests). There is no `<form>` in the app, which is why it was worth doing now: the day one
  arrives, every Button inside it submits, and nothing would say so.
- **`--color-field-hover`**, the same `#241812` under a name a field owns, so a change to how a
  table row highlights stops dragging every Select trigger with it.
- **`Alert` is `role="status"` by default** — an assertive live region interrupts a screen
  reader mid-word, which is right for the answer to an action and wrong for a diagnostic drawn
  with the page. Every call site was read: eleven are drawn with their screen and stay polite,
  four answer an action and say so. `sessionStopped` stays polite deliberately: it can be true
  when the screen opens, so it is a state, not an event.
- **`Progress` takes `value-text`**, as above.
- **The scanner** (`scan-conventions.mjs`): stacked `dark:` variants, `outline-none` outside
  `components/ui/`, the reset default scales, and five more attributes on the literal-variant
  rule. **10 fixtures → 18**, including the three that must *not* be caught — `bg-sky-dark`,
  `rounded-full`, and our own `text-body`, which lives in the same namespace as the dead
  `text-base`.
- **The palette owns Enter** (B65), below.

## B65: the decision, and why it is one decision

The listbox owns the highlight — it draws it, the arrows move it. **The palette owns what the
highlight *is*, and Enter is read off it.**

- **Plain `Enter` stays the listbox's.** It clicks the highlighted row, which arrives at
  `@select`: one path and one open, the property that stops a click from opening two tabs.
- **`Ctrl+Enter` is the palette's.** `ListboxRoot.onKeydownEnter` opens with
  `if (event.ctrlKey || event.metaKey || event.altKey) return`, *before* the click, so the
  library never hears the modified key. The palette answers it on a `keydown` on the input; the
  library's own handler runs first and returns without preventing anything, so the two can
  never both fire.
- **The highlight is put back when an answer lands.** `ListboxFilter` highlights the first item
  on every keystroke and the rows arrive 120 ms later, so the row it chose belongs to the
  previous answer and unmounts — and a detached row is neither drawn nor clickable, which is
  the predicted reason plain Enter did nothing on a backend result. `keyAfterAnswer` keeps the
  user's own position when the row survived the new answer, and takes the first row otherwise.

`Command` and `CommandDialog` relay exactly one function out (`highlightItem`) and one event in
(`highlight`). No collection, no element, nothing a caller could hold stale.

## What this repo cannot say about any of it

`ui/` has no `@vue/test-utils` and no DOM environment: **no test mounts a component**. Four of
the eight changes live inside a `.vue` file, so what is checkable was moved out — `buttonType`,
`keyAfterAnswer`, `cn`'s merge — and the rest is `pnpm typecheck`, `pnpm scan` and a window.

The window checks are in `docs/STATUS.md`, *"What only a window can say"*, as a thirteenth
group of seven. Two of them need only `pnpm ui:dev`, one needs a screen reader, and four are the
palette's keyboard — including **plain `Enter` on a row that arrived after the debounce**, which
B65 predicted was broken. The fix does not prove it was: that line is now a regression check
rather than a diagnosis, and only a window settles it either way.

## What stays open

**B12 items 3 and 10** — the keyboard highlight's contrast in Select and Command, and a disabled
segmented control that renders exactly like an unselected one. Both are "back to design": a
value chosen without looking at a window would be a guess with a commit under it.

**Half of item 4.** The half about items mounting after the search changed closed itself when
`Command.vue` started watching `allItems.size`. What is left is `CommandItem` not pruning its id
from its group's set on unmount — a leak rather than a wrong answer, since a stale id scores
zero and cannot keep a group visible — and `CommandInput`'s unconditional auto-focus.
