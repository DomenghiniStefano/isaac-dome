# One drag everywhere, and a tab that tears off into a window: report

**Plan:** `docs/superpowers/plans/2026-09-13-drag-and-windows.md`
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

**What the owner found by using it**, and what it changed: a tab must be draggable even when it is
the only one, and it must leave the strip as soon as it is torn off. Both are in the spec now
(§5), and they replaced two rules of mine — the second one removed a class of bug rather than
fixing an instance of it, since a tab in flight exists in exactly one place.

## Verification on the machine

(filled in by Task 17: the eleven checks, once the gesture is settled)
