# 3.7a — a tab owns its state: report

**Branch** `feature/tabs-state`, cut from `develop`. **Spec**
`docs/superpowers/specs/2026-09-15-tabs-session-design.md`, **plan**
`docs/superpowers/plans/archive/2026-09-15-tabs-own-their-state.md`.
`pnpm check` all green: **7 skips, 1483 real files touched**, which is the number that says the
tests on real data actually ran — this was executed in a second worktree, where `samples/` is a
junction and an unjunctioned one would have reported 0 while still passing.

**Ten tasks, ten commits, the suite green at each.** What follows is what the execution measured
or changed, not what it built; the plan says what it built.

---

## What the window has not been asked yet

**Nothing here has been seen in a real Tauri window.** B39's tag is `nothing, then a window` and
B6's is the same, so this report is deliberately half a report. What is covered by tests is the
pure half — the model, the document, the readers, the composable, the offset rule — and what
only a window can say is:

- [ ] a filtered, sorted, scrolled tab dragged into another window arrives filtered, sorted and
      scrolled where it was
- [ ] back and forward inside a tab restore the facets each entry was read with
- [ ] the app closed with several tabs open reopens them showing what they were showing, and
      going back in a restored tab lands on the screen's empty state (§6 of the spec says this is
      what the bound costs)
- [ ] twenty tabs shrink, then the strip scrolls, the active one is always visible, and a tab can
      still be torn off a scrolled strip
- [ ] on `?catalog=none` and `?fixture=none` a restored tab still opens and says what it has

A `- [ ]` here is a caveat about this work, not a task for someone else.

## What the execution found that the plan did not have

**`fakeWindows` did not degrade without a DOM, and every test that touches tabs died on it.**
`lib/window/` lives under one rule — *these modules degrade outside Tauri instead of throwing* —
and the case it did not cover is an environment with no `window` at all. It read
`window.location.search` at module load, so importing the tabs store threw before any test's
first line. Fixed where the rule is, not worked around in the test: the fake simply turns its
logging off when there is no document. **The rule is one case wider now**, and that is the whole
change.

**There is no `@vue/test-utils` and no DOM environment in this repo**, which the plan assumed
when it wrote `useTabView`'s tests as a mounted component. No dependency was added: the
composable uses `ref` and `watch` and no lifecycle hook, so it is exercised inside an
`effectScope` with a real Pinia. Five tests, nothing mocked. **This is worth keeping in mind for
any future composable**: one that reaches for `onMounted` cannot be tested here at all without a
decision nobody has taken.

**The scanner reads comments as code.** `pnpm scan`'s rule for calling Tauri commands outside
`lib/ipc/` is `/\binvoke\s*\(/` against the whole file, so a comment saying *"a screen never
calls `invoke()`"* trips the rule that forbids calling it. The comment was reworded and the
branch is clean, but the wart is real: a rule that cannot tell a mention from a call will
eventually refuse a correct explanation of itself. Registered as **B59**.

**A run's key had a collision waiting in it.** The expression moved out of `RunsTable.vue` —
the screen needs the same answer twice now that the selection is remembered — and moving it made
the shape visible: it read `live` for a launch and the folder's name for a session, so a session
actually called `live` produced the key of a launch. The archive keys those apart on purpose (a
launch of `log.txt` has no name, which is why `sources.key` is `NULL` for it), so the key gained
a colon and a test holds the two apart. Nothing depended on the old string — it was a Vue
`:key` and nothing else — but it is persisted from now on, which is why it was worth fixing
before and not after.

**Half of "the strip shrinks, then scrolls" was already built**, found by reading before
planning on top of it and corrected in the spec the same day: `TabItem` already carried
`max-w-tab-max min-w-tab-min shrink grow-0 basis-tab-max`, both tokens existed, the
`tab-intrinsic` utility existed so a strip sized by its tabs does not collapse every tab to the
minimum, and a container query already dropped the close button and then the icon under 64px.
**No token was added.** What is new is the scroll and the scroll-into-view.

**And the tear-off needed nothing at all for a scrolled strip**, which was checked rather than
hoped: every measurement in the drag path goes through `getBoundingClientRect`, which is in
viewport coordinates and already accounts for the scroll. No offset anywhere is computed from a
tab's index or from the strip's left edge.

## The decisions this execution stands on

**The record lives on the history entry, not on the tab.** Going back is going back to what you
were looking at, and a reading hung off the tab would hand you the facets of the screen you came
*from*. It costs nothing to persist, because the document already stores entries, and it costs
nothing to tear off, because `TabSeed` is `Omit<Tab, 'id'>` — written by subtraction on
2026-09-13 so *"a tab gaining a field needs no line here"*. **This is the case that was built
for**, and a test pins it rather than trusting the comment.

**The shell holds the record as `unknown` and each screen validates its own.** `read` answering
`null` is not an error; it is this app meeting a record written by a version of itself that no
longer exists. A record that cannot be read **is dropped and the entry survives** — which is
B6's open question answered: a tab whose *route* is gone still falls whole, because there is
nothing left to open, but a tab whose *reading* is unreadable opens on its screen and lets the
screen say what it has.

**The document is bounded by construction, not by a number.** It keeps the view of each tab's
current entry only. `MAX_SESSION_BYTES` is 64 KiB and its own comment budgets *"fifty tabs of
route names and queries"*; a record on each of a tab's fifty entries is a different sum, and the
failure mode is the worst kind — `set_window_session` answers `IpcError::SessionTooLarge`, the
frontend swallows it because *"a dialog because a session didn't save would be worse than the
session not saving"*, and you find out at the next restart with nothing having gone wrong on
screen. **Taking that error out of the swallowed set is 3.7b**, where the document grows again.

**The Collection's empty reading is its *default* filter and not the empty one** — it opens on
what has not been found — and the reader never falls back to it. A filter the user cleared is a
filter they cleared, and reading an empty stored filter as "nothing stored" would put the opening
filter back on every restart. A test says so, because the two are one character apart in the
code and a world apart on screen.

**An offset is a distance into a list of a given length**, so the length travels with it and a
list that changed underneath keeps the top. One component has owned every virtualized scroll box
since B43, so the rule is in one place and is a pure function with its own test.

## What is deliberately not here

- **Floor's painted grid stays a scratchpad.** `stores/floor.ts` says persisting it *"would
  outlive the floor it describes"*, and that still holds. The cost is stated rather than
  discovered: a Floor tab dragged into another window arrives empty.
- **The wiki's category list is the fifth virtualized list** and has no reading of its own. It
  keeps neither its scroll nor anything else; the four screens the spec names are the four that
  had state to keep.
- **The session is still one window.** Version stays 1: an entry gaining an optional key is a
  part an older app can ignore. 3.7b is where the top level stops saying `tabs` and starts
  saying `windows`, which is a bump, and where the writer stops being `main` — **`main` can be
  closed while other windows live**, so "only main writes" means the session silently stops being
  written from that moment. That finding belongs to the spec and is not fixed here.
- **No Rust.** 3.7a adds no command, no migration and no IPC type.
