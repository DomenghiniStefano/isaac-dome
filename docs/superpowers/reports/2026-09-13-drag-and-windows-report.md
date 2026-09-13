# One drag everywhere, and a tab that tears off into a window: report

**Plan:** `docs/superpowers/plans/archive/2026-09-13-drag-and-windows.md`
**Spec:** `docs/superpowers/specs/2026-09-13-drag-and-windows-design.md`
**Branch:** `feature/drag-and-windows`, worktree `C:/Projects/isaac-dome-drag`

## Task 1 — the spike

**Run on 2026-09-13, on this machine, with the owner's hand on the mouse. The answer is yes: the
webview keeps talking.** A tab dragged up and to the left, out of the window, released on the
desktop:

```
[spike] OUTSIDE move 100  -1  buttons 1 — outside events so far 1
[spike] OUTSIDE move  -6 -76  buttons 1 — outside events so far 81
[spike] OUTSIDE move -43 -56  buttons 1 — outside events so far 101
[spike] UP OUTSIDE the window at -45 -54 — moves 255 of which outside 105
```

Three things that together settle it, and each mattered:

1. **The coordinates go negative and keep changing.** They are not clamped at the window's edge,
   which is the shape the failure would have had: a stream of events all reading `0` or
   `innerWidth` would have looked like delivery and been none.
2. **`buttons` stays 1** through all 105 outside events: the webview still believes the drag is
   happening, so the state the gesture reads is not stale.
3. **The `pointerup` arrives outside**, at (-45, -54) — the event the whole question was about,
   since a gesture that never hears the release hangs with the button already up.

**So `lib/window/pointerSource.ts` keeps its DOM implementation and `crates/app` gains nothing.**
The Rust variant the spec describes (§2) is not needed on this machine; it stays written down
there, behind the same interface, in case another machine or a later WebView2 says otherwise.
The 10-second silence timeout stays as a safety net rather than as the mechanism.

**What this measurement does *not* say.** It is one machine, one WebView2 version, one monitor at
`dpr 1`, and one direction (up and left, into negative coordinates). It says nothing about a
second monitor at another scale factor, which is Task 17's ninth line.

**How the instrument was proved before its silence was read**: the probe first reported drags
*inside* the window (`UP inside the window … moves 83 of which outside 0`), so a later "no outside
events" would have been a measurement and not a broken listener.

**One defect the probe itself caused, worth remembering**: its first version called
`setPointerCapture` on the strip for **every** `pointerdown` in the document, which retargets
every click in the app — the "+" button and the tabs stopped answering, and it read as a bug in
the app. An instrument that changes what it measures is worse than no instrument.

**What the attempt did measure**, on 2026-09-13, before it was stopped:

- The app builds and runs from this worktree (`target/debug/isaac-dome.exe`, cold build ~13 min,
  the last link 32 s).
- **A real defect, found by running it rather than by reading it**: `watchAppEvents` called
  `listen()` with no `isTauri()` guard, and outside Tauri that does not answer "no" — it reaches
  into internals that are not there and throws `transformCallback of undefined` *from inside a
  mounted hook*, which takes the shell down with it on the development server. `preview.ts` had
  the same hole in `movePreview` / `hidePreview`. Fixed in `990c928`; the rule was already the
  repo's ("degrade, never fail"), only this family of modules had not been held to it.
- Two ways to lose an afternoon on this machine, worth writing down: a `pnpm dev` whose
  `beforeDevCommand` loses the port leaves **cargo processes holding the build-directory lock**,
  and the next build sits there with no output and no CPU until they are killed; and a browser
  tab left open on the dev server reconnects over HMR and reports its own errors into the same
  log, which is how the `isTauri` hole surfaced in the first place.

## What the machine found, 2026-09-13

The gesture was driven by the owner on a real window while the probe reported into the `pnpm dev`
terminal. **Five defects, none of which any test had caught**, and each one is a rule that was
wrong rather than a line that was mistyped:

1. **A drag that starts straight down never starts at all.** The threshold read the list's axis —
   x for a strip — so the one movement that tears a tab off was the one that could not begin a
   drag. It passed unnoticed in the browser because the scripted test nudged sideways by 6 px
   first: *the instrument made the gesture possible*. Now a drag begins on movement in any
   direction, and the axis decides only the hit test.
2. **The gesture waited for its own picture.** `detach` awaited the preview window's creation
   before installing the pointer watch, so a preview that never resolved took the whole gesture
   with it — no error, no window, nothing. The watch is what the gesture *is*; the preview is
   what it looks like. Watch first.
3. **The tear-off re-attached itself in a loop.** "Back over my own window means the tab comes
   home" fired one frame after detaching, because the tear band is crossed while the pointer is
   still inside the window. Dozens of detach/attach cycles per drag.
4. **A closed window is still listed, and answers `window not found`.** `getAllWebviewWindows()`
   keeps a window for a while after it closes; asking it anything throws, and thrown from inside
   a drag it killed the gesture. A window that is not there is one fewer target, not an error.
   This also explains a mismatch seen earlier and left open: the gesture counted four windows
   while the desktop showed one.
5. **"Cancelled" and "ended" cannot be told apart by "is the drag still running".** A guard added
   for `Escape` watched exactly that, so every successful release undid itself — the tab popped
   out and straight back. `useDragList` now reports a cancellation explicitly, and a release never
   reaches that path.

**A sixth thing that turned out not to be a defect, and the measurement that said so.** The card
was suspected of passing behind the tabs, and a 16×16 window at `0,0` looked like a preview born
broken. Measured with `EnumWindows` on the app's own process: the preview is `200x39 at 876,484`
— exactly where it was asked for — with `WS_EX_TOPMOST` set, and the 16×16 is a window the
webview keeps for itself, present whether or not a drag ever happens. The DOM ghost, measured in
the browser, is `z-50` on `fixed` with no stacking context between it and the body. **Both are in
front by construction**, so what was seen was the gap between them: at the first tear-off the
card still had to be created, and during those hundreds of milliseconds nothing followed the
cursor at all. The card is now built when the drag *starts*, off-screen and hidden, so leaving
the strip costs only a `show`.

**The duplication, and the three wrong answers before the right one.** A tab dropped on another
window's strip arrived twice. The first two fixes were reasonable and did nothing, which is the
useful part of the story:

1. *"`emitTo` with a bare label matches window, webview and pair"* — true, and named the target
   by kind. Still duplicated.
2. *"stale listeners from a day of hot reloads"* — plausible, so the app was restarted clean.
   Still duplicated.
3. The probe then said it plainly: `win-…svsf DOCK at 0 had 0` and `win-…skdo DOCK at 1 had 1` —
   **two different windows docked the same tab**. Not one message delivered twice: one message
   heard by everyone. `listen` defaults to `{ kind: 'Any' }`, so every window had been hearing
   every message on the channel, whoever it was addressed to. Each window now listens for its own
   label.

The reason it survived so long is worth keeping: **the other messages were idempotent by
accident**. `Ready` is answered once because the debt is consumed; `Seed` is ignored by a window
that is no longer pending. `Docked` had nothing of the kind, so it was the only one that showed.
A protocol where being heard twice is harmless hides a transport that delivers twice.

**Two more, from the same session.** A window that gave away its last tab closed *before* its
newborn asked for its seed — and the debt lives in the creator's memory, so the new window opened
with an empty bar; a creator now waits until the seed is taken, with a timeout of its own. And
the newborn's wait was three seconds: not a delay but a deadline, paid only by a window nobody
owes anything to — which is every window that **reloads**. It is 700 ms now: whoever answers is
another window of the same process.

**What the owner found by using it**, and what it changed: a tab must be draggable even when it is
the only one, and it must leave the strip as soon as it is torn off. Both are in the spec now
(§5), and they replaced two rules of mine — the second one removed a class of bug rather than
fixing an instance of it, since a tab in flight exists in exactly one place.

## Verification on the machine

Driven by the owner on a real window, 2026-09-13. **Four of the eleven checks pass**, and three
defects were found and fixed in the doing — each one a rule, not a slip.

- [x] **1. A tab torn onto the bare desktop opens a window there.** The window is now drawn
      **around the tab**: where the hand holds it inside the tab, and where a tab sits inside a
      window, are both measured while the tab is still in the strip, and the new window is placed
      so the tab lands under the cursor. It used to put its own top-left corner there, which the
      owner named exactly: *"preferisco si posizionasse lì la tab e poi si crei la scheda"*. The
      same change fixed a unit confusion nobody had noticed: creation coordinates are logical and
      the point we hold is the desktop's physical ones — the same number until a screen is
      scaled. The window is now born hidden, placed in physical pixels, then shown.
- [x] **2. A tab dragged back onto another window's strip merges there**, arrives once, and the
      window it leaves closes. It used to arrive **always at the end**, wherever it was dropped,
      and that was a race of mine: at the release the origin told the target "the tab left" —
      taking with it the index the marker was aiming at — and only then handed the tab over. The
      target now keeps its aim until the tab has landed on it.
- [x] **3. `Escape` in mid-flight puts the tab back where it sat.** Right the first time.
- [x] **The close button on the last tab closes the window** (a secondary one; the first keeps
      its landing tab). The old rule — "the bar is never empty", so the last tab is replaced by a
      fresh one — had survived the owner's decision that an emptied window closes. For a
      secondary window those are the same sentence, and it now says so.

Still to check: docking into a *second* secondary window; two windows on the Plan and a move in
one; a profile change and a scale change crossing windows; two monitors at different scale
factors; the target window closed while a tab is in flight; and the cold start's first tear-off.
