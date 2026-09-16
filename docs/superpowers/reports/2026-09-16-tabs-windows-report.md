# 3.7b — windows of tabs: report

**Branch** `feature/tabs-windows`, cut from `develop`. **Spec**
`docs/superpowers/specs/2026-09-15-tabs-session-design.md` (§5, §6, §11 item 2), **plan**
`docs/superpowers/plans/archive/2026-09-16-tabs-windows.md`.
`pnpm check` all green: **17 skips, 1483 real files touched**, which is the number that says the
tests on real data actually ran — this was executed in a second worktree, where `samples/` is a
junction and an unjunctioned one would have reported 0 while still passing.

**Three commits**, the suite green at each. What follows is what the execution measured or
changed, not what it built; the plan says what it built.

---

## What the window has not been asked yet

**Nothing here has been seen in a real Tauri window.** B39's tag is `nothing, then a window` and
B6's is the same, so this report is deliberately half a report — and this sub-project is the one
where that gap costs most, because the thing it fixes is a failure that is invisible on screen by
definition. What is covered by tests is every judgment: the election, the write decision, the
clamp, the document. What only a window can say is:

- [ ] two windows open, the app closed and reopened: two windows come back, in their places,
      holding what they held
- [ ] `main` closed while a second window lives: the session keeps being written, and reopening
      the app afterwards comes back to what the survivor was holding
- [ ] a window remembered on a screen that is no longer plugged in opens where it can be reached
- [ ] a window restored on a scaled monitor is the size it was, not twice it — the one bug the
      physical/logical asymmetry produces, and the one no test here can see
- [ ] a minimized window is still in the document after a restart (this is what `labels()` was
      added for, and only a window proves it)
- [ ] a tab torn off into a new window, then the app closed: the new window comes back

A `- [ ]` here is a caveat about this work, not a task for someone else.

## What the execution found that the plan did not have

**The hit test's `list()` cannot be the roster, and the reason is a comment it already carried.**
`list()` drops every window that is hidden or minimized, which is right for "where can a tab land"
and wrong for "which windows are open". A minimized window is still part of the session, and
reusing that call would have written a document that loses it — silently, and only visibly one
restart later. `labels()` is a second call on purpose, and the plan says so rather than leaving
the next reader to narrow one into the other.

**The same module says the roster cannot be trusted in the other direction, and that turned into
a real deadlock.** `getAllWebviewWindows` *keeps listing a webview for a while after it closes* —
measured on the machine on 2026-09-13, and written down there because a call on a stale window
threw inside a drag. A write postpones itself while a window in the roster has not yet said what
it holds, which is what stops two windows writing at once; with a ghost in the roster that
nobody's ledger has, that postponement never ends. **The session would then stop being written —
which is the exact failure this sub-project exists to remove, wearing a different coat.** Two
answers, both needed: a window that says `Closing` is struck from the roster, and the patience is
bounded at three attempts for a window that crashed without a word.

**A reload of `main` is not a launch, and the difference had to be found rather than assumed.**
`main` re-runs its whole mount whenever its webview reloads, which is every save on the
development server. It would then read the session it wrote a moment earlier and open a second
copy of every window in it, and again on the next reload. The only thing that tells a launch from
a reload is the roster — at a launch it is `main` alone — so the restore reads it first. This is a
defect that would have shown up as "the app opens too many windows" long before anyone traced it
to the session.

**The decision to write was extracted from the composable, and that is what made the sub-project
testable at all.** It began inside `useWindowSession`'s closure, where the repo has no way to
reach it: there is no DOM environment and no `@vue/test-utils`, and `onMounted` is not a thing a
Vitest file can drive. `decideSessionWrite(self, roster, ledger, patient)` is pure, so *"the
session keeps being written once main has been closed"* — the whole of 3.7b in one sentence — is
an assertion and not a thing you find out by closing a window and restarting the app. It was
mutated back to the old rule (`self !== MainLabel`) to check it turns red: exactly that one test
failed, and no other.

**The clamp's tests were mutated too**, `area > 0` to `area >= 0`, which makes every box count as
"on a monitor": three of the seven went red. They had passed on their first run against an
implementation written minutes earlier, which is not evidence of anything until it has been shown
able to fail.

## What was decided here rather than in the spec

- **The election is a pure function of the label set, not of who joined first.** Every window
  elects for itself from the roster it just read, so two windows holding the same labels in a
  different order reach the same writer without any window having to be told anything.
- **The box is stored in desktop physical pixels.** `windowPort.create` takes a physical position
  and a **logical** size — the asymmetry `windowSize()` already carries a comment about — so the
  clamp answers the scale factor of the monitor the window lands on and the caller divides. Got
  backwards, this opens a window twice the size it had, on a scaled screen only.
- **A stored box that is on no monitor keeps its size** and moves to the primary's work-area
  corner. The spec's words are "at the size it had": a window that comes back resized is not the
  window that was left, and one larger than the screen is still reachable by the title bar the
  corner puts on screen.
- **Work area, not screen.** A window restored under the taskbar has a title bar that cannot be
  grabbed, which is the same as not being restored.
- **A window whose every tab was unreadable is dropped whole**, not restored empty. An empty
  window is one the user never had, and opening it would put a landing page on their desktop they
  did not leave there. It is the one place where 3.7b is *less* forgiving than 3.7a, and for the
  same reason: a tab is dropped alone because the others are still what they were, and a window
  with nothing in it is not.
- **`SessionTooLarge` surfaces in the About dialog.** The spec says *a diagnostic in the existing
  list, not a dialog*, and there is no shell-level list yet: every `DiagnosticsList` in the repo
  is a screen's, fed by that screen's backend diagnostics, and the session is no screen's fact.
  About is the shell's own dialog and one click from every window. **It moves next to the switch
  that turns the session off when 3.6 builds Settings**, and this is written down here so that
  move is a move and not a rediscovery.

## The limit this carries, stated rather than found later

**The `SessionTooLarge` flag is per window**, because it is set where the write failed and only
the elected window writes. Someone opening About in another window sees nothing. This is right as
far as it goes — it is not the other window's failure — but it means the diagnostic is reachable
only from the window that happened to be elected, and if that window is then closed, the next
writer starts with a clean flag and re-learns it on its own first failure. Broadcasting it was
considered and refused for now: it is one more message on the channel for a case that §6 made
very unlikely by bounding the document, and the honest fix is 3.6's Settings screen, where the
session has a home.

## What it did not touch, and that is the point

`crates/store` (migration 3 already holds this), `crates/ipc`'s session settings,
`crates/app/src/commands/session.rs`, and every Tauri command. **3.7b adds no command, no
migration and no token.** `stores/tabs.ts` and `stores/tabModel.ts` did not move either: a
window's tabs were already a `Session`, and this only put several of them in a list.
