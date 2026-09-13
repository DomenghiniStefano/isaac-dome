# One drag everywhere, and a tab that tears off into a window: report

**Plan:** `docs/superpowers/plans/2026-09-13-drag-and-windows.md`
**Spec:** `docs/superpowers/specs/2026-09-13-drag-and-windows-design.md`
**Branch:** `feature/drag-and-windows`, worktree `C:/Projects/isaac-dome-drag`

## Task 1 — the spike

**Not run yet, and it needs a hand on the mouse.** The measurement is what the webview does once
the cursor has left the window, so it needs a real cursor: synthetic events cannot leave the
window, and driving the OS cursor would take the mouse away from whoever is at the machine. The
probe is written (plan, Task 1, steps 1–7) and the phases that do not depend on the answer were
built first — only `lib/window/pointerSource.ts` waits on it.

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

## Verification on the machine

(filled in by Task 17)
