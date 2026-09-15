# Completed — the quality tasks of the 2026-09-05 review

Fourteen items, **all fourteen closed**, each with the criterion it had to meet. **Searched,
never read.** It was `docs/IMPROVEMENTS.md` until 2026-09-16: a finished document still listed
among the living ones, which is how it came to contradict `docs/STATUS.md` for nine days — E1
below closed the `unpack` spec correction on 2026-09-06 while a box in STATUS went on asking for
it, and E1 even opens by naming that box.

# Improvements — from the 2026-09-05 codebase review

Tasks that take the foundation from "solid" to "ready for an external user". They come out of
a quality review: architecture, IPC boundary, tests, local automation. They **do not** contain
new features: the M2 graph, the log watcher and the webapp live in the milestones of
`docs/STATUS.md`, and remain the project's real bottleneck.

Where a point is already tracked in `STATUS.md`, this says so and points there: this document
adds closing criteria and ordering, not a second list.

Every task has three fixed lines. *Why* is the cost of not doing it. *Done when* is
verifiable without opinions: if you can't say yes or no, the task isn't closed. *Weight* is S
(one session), M (two or three), L (a week or more).

**No CI.** This is a decision, not a gap: checks run on the writer's machine, before the
commit. For the decision to hold, the list of commands needs to live in one place and running
it needs to cost one command — that's what E2 (`scripts/check`) and A2 (the hook) do.

---

## Recommended order

1. **E2** a single list of verification commands — it's the multiplier for everything else:
   without CI, that list has to live in one place and run with one command.
2. **A2** pre-commit hook — gives that list a moment to run on its own.
3. **A3** scanner aligned with the document — small, and closes an unkept promise.
4. **B1** rule 5 — an hour's decision, then it's applied.
5. **D1** real tests that are skipped — already written, just need the samples put back.
6. **C1** streaming archives — the most concrete risk on "some stranger's machine".
7. The rest in parallel with the webapp, once there's real frontend code to serve.

---

## A — Check automation

- [x] **A2. Lightweight pre-commit hook.** *Weight S.*
      **Why:** without CI, the only thing stopping an unformatted file or a convention
      violation from getting into the history is someone remembering to run the checks. A
      hook does it for them, for the fast stuff.
      **What to do:** a script committed in `scripts/` (not in `.git/hooks`, which doesn't
      travel with the repo) that runs `cargo fmt --check` and `pnpm scan`, plus a command
      documented in the README to install it. No tests in the hook: too slow, those stay in
      `scripts/check`, run before wrapping up a piece of work.
      **Done when:** a fresh clone, after the install command, rejects a commit with an
      unformatted Rust file.
      **Closed on 2026-09-06:** `scripts/git-hooks/pre-commit`, installed with
      `git config core.hooksPath scripts/git-hooks`. The hooks live in a versioned folder
      instead of being copied into `.git/hooks`: they update along with the repo, and
      uninstalling them is `git config --unset core.hooksPath`.

- [x] **A3. Scanner aligned with what the document promises.** *Weight S.*
      **Why:** `docs/frontend-conventions.md` declares five rules "that get violated first"
      and says every new rule must be added to the scanner. Today the scanner covers rules 1,
      2 and 3; not 4 and 5. The document promises a check that doesn't happen.
      **What to do:** add three checks to `ui/scripts/scan-conventions.mjs`: raw `<button`
      and `<input` outside `src/components/ui/`; a union of two or more string literals
      (rule 5, see B1); visible strings in the template, with a declared heuristic and a
      list of exceptions. Update the "How these rules are checked" table in the document.
      **Done when:** the document's table and the script's `checks` array have the same
      rows, `pnpm scan` is green on the current code, and every exempted file is exempted by
      name, by check, and with a reason written in the script.
      **Closed on 2026-09-06:** three new checks, verified against a probe file that
      violates all three at once. Three exceptions declared in `EXEMPTIONS`, all on the
      verification pages (`App.vue`, `WikiInline.vue`). The heuristic's two limits — it's
      text, not an AST — are written into the document instead of being discovered at the
      first false positive.

---

## B — IPC contract

- [x] **B1. Close the gap on rule 5 (string unions).** *Weight S.*
      **Why:** the document bans `type X = 'a' | 'b'` "wherever it appears", but
      `ui/src/lib/ipc/types.ts` declares `CandidateSource`, `ItemKindView`, `OriginView`,
      `StepsBasis` and `prefix` exactly that way, and `App.vue` compares against literals.
      The comments justify it as "a value, not a discriminator", which is the Rust rule: but
      the frontend rule makes no such distinction. The document and the code say two
      different things.
      **What to do:** apply the rule to wire types too, in the form the wiki section of the
      same file already uses (`const X = { … } as const` plus the derived type). That way
      the rule has no exceptions to declare and the scanner has no list to maintain.
      **Done when:** the rule 5 check in A3 passes with no exceptions, and the document
      describes exactly what the scanner checks.
      **Closed on 2026-09-06:** approach (b) applied across the board, including `prefix`,
      which became `SavePrefix`. Seven unions replaced with `as const` objects; nothing
      changes on the wire. Rule 5 has no declared exceptions, and the document now also
      says what stays out of scope: the tag of a tagged union, which TypeScript narrows on
      its own.

- [x] **B2. Generate `types.ts` from the Rust types.** *Done 2026-09-13 as N7,
      `feature/generated-contract`.*
      **Why:** the contract lives twice, in the `#[serde]` attributes and in the
      hand-written `types.ts`. The shape tests in `crates/ipc/tests/` are the glue and they
      work, but every new type means two edits and an extra test, and an oversight is
      silent.
      **What to do:** a one-session spike with `ts-rs` and `specta` on the `ipc` types,
      paying attention to the cases the tests deliberately pin today: `rename_all_fields`,
      `miniZ`, fieldless enums as bare strings, transparent `GoalId`. If the generation
      renders all of them faithfully, adopt it and `types.ts` becomes a generated,
      committed file, with a check in `scripts/check` that fails if it's stale. The shape
      tests stay where the case is delicate.
      **Spike run 2026-09-13 (N7), and the answer is "four of the five".** `ts-rs` 12.0.1,
      on copies of the real types with the real serde attributes:
      - `rename_all_fields` → `autoSelected: boolean`. **Faithful.**
      - `miniZ` → `{ "kind": "miniZ" }`. **Faithful** — serde's own rename is read, so the
        case B9 turned into a rule is not something the generator can get wrong.
      - transparent `GoalId` → `export type GoalId = string`. **Faithful.**
      - **Doc comments survive as JSDoc**, which is better than faithful: the knowledge
        that today lives in `types.ts` comments ("`miniZ` is not a typo") moves next to the
        Rust type and stops being a second copy that can drift.
      - Several types can share one `export_to`, so the output is **one file**, not one per
        type. No import graph, no barrel, no change to any call site.
      - **A fieldless enum renders as a string union** — `"steamNotFound" | "gameNotFound"
        | "noSaves"` — and that is the one gap. Frontend rule 5 forbids it, the scanner
        catches it, and every component compares against `MissingReason.SteamNotFound`, so
        the union would break call sites as well as the rule.
      **Only `ts-rs` was evaluated, and `specta` was not, on purpose.** The gap is not in
      either tool: both target idiomatic TypeScript, and the `const … as const` form rule 5
      asks for is this repo's own convention. A second general-purpose generator would emit
      the same union. What the gap needs is a post-step, not a different crate.
      **So the shape is: generate, then rewrite the one case.** `ts-rs` writes the raw file
      during `cargo test`; a script turns each bare string union into the `const … as const`
      pair and formats; `scripts/check` regenerates into a temp file and fails if the
      committed `types.ts` differs.
      **One decision the spike surfaced and the item had not.** Five types cross the
      boundary from crates that are not `ipc`: `core_save::Kind` — the one `CLAUDE.md`
      records as having changed the wire with the suite green — plus `wiki::Target`,
      `discovery::SavePrefix`, `Edition` and `Dlc`. Generation has to cover exactly those,
      so `ts-rs` reaches `core-save`, `wiki` and `discovery`. Declaring them by hand in the
      post-step instead would leave the documented incident hand-written, which is the one
      outcome that makes the whole item pointless.
      **Done when:** changing an enum in Rust without regenerating makes `scripts/check`
      fail, and `types.ts` has no more hand edits in its subsequent history.
      **Closed 2026-09-13.** Both criteria met and the first one demonstrated rather than
      assumed: `#[serde(rename = "bestiaryy")]` on one variant of `core_save::Kind` — a change
      that alters the wire and leaves every Rust call site compiling — fails `cargo-test` and
      `ipc-types` together, and the diff names the line. The spike's shape held: generate, then
      rewrite the one case, then prettier.
      **The spike's count of foreign types was wrong, and in the direction that mattered.** It
      said five; there are nine (`wiki::SectionKind`, `CollectibleTemplate` and `Style` beside
      `Target`, plus `core_save::marks::CharacterGroup`). Having chosen to derive on the real
      types, the compiler named the missing four in one pass — the alternative the spike
      rejected, declaring them by hand in the post-step, would have left four types
      hand-written and silent instead of one.
      **Four defects found on the day it landed**, listed with N7 in `docs/STATUS.md`: two
      fields typed `string` that are unions (`GameView.edition`, `SectionCount.kind`), one
      missing `Infobox` variant that made a wiki page throw (now **B40**), and `ProgressMark`
      exposed only through `for_tests` while being a field of `SearchHit`. None of them was
      visible to any test, which is the whole of the argument the item was making.
      **Two consequences to live with.** A `///` on a wire type is UI source and obeys
      `pnpm scan`; and `ts-rs` prints a permanent warning about `#[serde(transparent)]`, which
      it ignores and whose effect it reproduces anyway — left standing, because
      `no-serde-warnings` would hide the next one too.

- [x] **B3. Shared fixtures for the `ipc` tests.** *Weight S.*
      **Why:** the shape tests hand-build fake nodes, achievements and goals. The literal
      strings in the assertions are correct and stay: they're the wire spec. But
      **fixtures** repeated across different files drift apart over time.
      **What to do:** survey helpers like `node(true)` in `crates/ipc/tests/*.rs`; where the
      same fake object is built in more than one file, move it to
      `crates/ipc/tests/common/`. Don't touch the assertions.
      **Done when:** no fake achievement or target is defined in more than one test file.
      **Closed on 2026-09-06, with a different outcome than expected.** The survey found no
      duplicated fixture: `node()` only lives in `graph.rs`, `item()` only in `goals.rs`,
      and `app`'s `goal()` points at a boss, not the same object. The only thing that
      *looked* duplicated were two `catalog()` and two same-named `ITEMS` in `graph.rs` and
      `catalog_view.rs`: two **different** fixtures, for two different questions (names
      from the string table versus achievements and unlocks). Merging them would have
      collapsed two things that need to stay separate, so they were renamed instead —
      `catalog_with_string_keys` and `catalog_with_achievements` — and no
      `crates/ipc/tests/common/` was born: it would have been a folder with nothing in it.

---

## C — Memory and performance

- [x] **C1. Streaming archives, one shared `ResourceSet`.** *Weight L.*
      Already in `STATUS.md`, "Open blockers", first entry. Here only the closing criterion.
      **Why:** about 1.3 GB loaded into RAM to open the archives, reopened by multiple
      commands. On a machine with 8 GB and the game running, this is the most likely case
      of "it closes without saying anything" — a violation of constraint 5.
      **What to do:** in `unpack`, an in-memory index and per-entry reads (memory-mapped
      file or `seek` + `read`), then a single `ResourceSet` in the Tauri state, shared by
      `extraction_report`, `unlock`, `plan`, `add_goal` and `remove_goal`.
      **Done when:** a test measures the peak memory of opening the eight archives under a
      threshold declared in the `unpack` spec, and `grep ResourceSet::open` in the `app`
      crate finds a single call.
      **Closed on 2026-09-06.** `Archive::open` reads the header and index, keeps the
      `File` open, and reads an entry's bytes when needed. Measured with a global allocator
      in `crates/unpack/tests/streaming.rs`: opening eight archives and 19,473 entries goes
      from ~1.3 GB to a peak of **1.4 MB**, against a declared threshold of 64 MB. A second
      test rereads 942 real entries across all three compression modes and checks that each
      one produces the length the index declares.
      The non-obvious part: **an entry's compressed length is written nowhere**, and the
      decompressors stop on their own. The upper bound is the next offset (contiguous
      data), and the last entry ends where the index begins. The read is positional
      (`seek_read` / `read_at`), not `seek` + `read`: a single `ResourceSet` is shared by
      multiple commands, and the file cursor would be a race between threads.
      On the `app` side, `ResourcesState` is the only call to `ResourceSet::open`. The
      "game not installed" case isn't cached: someone who opens the app before installing
      the game shouldn't have to restart it.

- [x] **C2. Icons out of the `unlock` rows.** *Weight M.*
      **Closed on 2026-09-08, with a different shape than this entry proposed.** The entry
      asked for a second command serving icons for a list of ids. That would have moved a
      cache into the UI — decide what's visible, ask for those, keep them, don't ask twice,
      drop them on scroll — which is code we'd write and test in TypeScript, in the one
      place the project says receives only resolved JSON.
      Instead the app registers a **URI scheme**. A row still carries `iconUrl`, but it is
      now a short link (`isaac://achievement/19`, `isaac://item/passive/92`) that an
      asynchronous protocol handler serves from the `ResourceSet`. The browser does the
      lazy loading, the caching and the de-duplication; the UI writes `<img :src>`.
      **The measured outcome:** `unlock` went from ~7 MB to **415 KB**, and `next_steps` —
      the app's opening screen — from 124 KB to **3 KB**. The 94% figure that motivated
      this was measured, not estimated: 226 KB of base64 in a 240 KB twenty-node excerpt.
      **Two things this shape gets for free.** The TypeScript type doesn't change at all
      (`iconUrl` is still `string | null`), so the frontend was never blocked on it. And
      no file path crosses the IPC boundary, which the id-keyed reference guarantees by
      construction where a path-keyed one would not.
      **Where the pieces are:** `ipc::IconRef` with its `to_path`/`parse` round trip and
      `icon_source` (pure, 5 tests); `icon_url` and the handler in `crates/app`, which is
      also the only place that knows Windows rewrites the scheme to
      `http://isaac.localhost/`. **Done when** is met by
      `crates/ipc/tests/unlock_size.rs`: the real payload under a declared ceiling, no
      `data:image` anywhere in it, and at least one link actually present — because a
      ceiling alone would pass just as happily on a payload with no icons at all.
      **One thing to remember at release time:** `tauri.conf.json` says `"csp": null`. When
      a real CSP arrives it must allow `img-src` from this scheme, or every icon vanishes
      with no error and no failing test. Written in the code, next to the registration.

---

## D — Test data

- [x] **D1. Put back the samples the real tests look for.** *Weight S.*
      Already in `STATUS.md`, "To investigate".
      **Why:** `real_saves.rs` (in `core-save` and in `ipc`) and `cross_check.rs` skip with
      a note because the dated files they look for are no longer in `samples/`. The suite's
      green doesn't count them.
      **What to do:** restore the files from the game's backups in `save_backups\`, or
      rewrite the tests against the samples that are present, where the pinned values are
      derived from the spec and not from the code's output.
      **Done when:** `cargo test -p ipc` and `cargo test -p core-save` don't print `skip:`
      lines for missing samples. Skips remain legitimate for a tool absent from the
      machine — `cross_check`'s Python reference — which say the independent check didn't
      happen.
      **Closed on 2026-09-06:** the 2024 and January 2025 files no longer exist in
      `save_backups\`, so the tests were brought back onto the series that's actually there.
      A hidden bug had been sitting behind those very skips: on Windows, `python` is a
      **Microsoft Store alias** that launches, prints "install from the Microsoft Store",
      and exits with 49, so `cross_check` took that as "the reference is broken" and
      failed. Now it asks the interpreter to say a word of our choosing before trusting it.

- [x] **D2. Historical series in `samples/` and a diff test.** *Weight M.*
      Linked to the "`samples/` doesn't contain the M0 collection's saves" entry in
      `STATUS.md`.
      **Why:** the plan that updates itself rests on the diff between two saves. Today the
      diff is only exercised on two dates. `save_backups\` holds about thirty dated backups
      of the same profile: a free historical series that no test uses.
      **What to do:** copy the backups into `samples/` under the dated name already in use.
      A test in `core-save` that walks the series in order and checks that achievements
      never regress (a bit set to 1 never goes back to 0) and that the section count is
      monotonic. It's a property test on the format, not a pinned value.
      **Done when:** the test runs on at least ten consecutive snapshots and skips with a
      note where it finds fewer than two.
      **Closed on 2026-09-06:** 33 snapshots from 2025-06-26 to 2026-09-05 (slot 1). Two
      property tests: the diff reports exactly the bits turned on between two snapshots,
      and the series never regresses. The first one immediately found a real edge case —
      in the transition from 641 to 642 slots, the new slot comes out already unlocked, and
      "didn't exist before" counts as off, not as out of the comparison. The code already
      did it this way, on purpose; the test's first draft didn't.

- [x] **D3. A test on real data declares which portion of the domain it runs on.** *Weight S.*
      Rule already stated in `STATUS.md`, "Tests that pass on a non-representative sample".
      Here it's made mechanical.
      **Why:** the `unpack` tests all ran on `config.a`, the only archive the decompressor
      knew how to open. No skip, no red, main function broken.
      **What to do:** a shared skip helper that always prints, even when the sample is
      present, the name of the file used; and in `scripts/check`, a count of the `skip:` and
      `sample:` lines in `cargo test --workspace`'s output, summarized at the bottom.
      **Done when:** `cargo test --workspace`'s output lists every real file touched and
      every test skipped, and `scripts/check` reports the two numbers.
      **Closed on 2026-09-06:** the `test-support` crate, a dev-dependency of `core-save`,
      `discovery`, `unpack`, `catalog` and `ipc`. No real test opens `samples/` by hand any
      more: `sample()`, `sample_bytes()`, `packed_dir()`, `packed_file()` and
      `dated_series()` always declare what they used. Today `scripts/check` counts 247 real
      files touched and 1 skip.

- [x] **D4. Synthetic fixture for the game's leftover folder.** *Weight S.*
      Already in `STATUS.md`, "To investigate".
      **Why:** the "leftovers only, no executable" case is no longer reproducible on this
      machine, so today it isn't covered by anything.
      **What to do:** a test in `discovery/tests/pure.rs` that builds an empty
      `steamapps\common\The Binding of Isaac Rebirth\` in a temp directory and checks that
      it isn't reported as an installation.
      **Done when:** the test exists, is green, and depends on nothing outside the temp
      directory.
      **Closed on 2026-09-06:** it lives in `discovery/tests/discover.rs`, not in `pure.rs`
      as this line said: `pure.rs` covers pure functions, while the tests that build a fake
      folder tree already all live in `discover.rs`, next to the other degradation cases of
      `find_game`.

---

## E — Documentation and hygiene

- [x] **E1. Corrected `unpack` spec.** *Weight S.*
      Already in `STATUS.md`: it still declares `0x07 u8 versione = 0x01`, disproven on
      2026-09-03.
      **Done when:** the spec describes the byte as the compression mode with the three
      known values, and cites the commit that discovered it.
      **Closed on 2026-09-06:** the header line is corrected and below it there's a
      "2026-09-03 correction" box with the table of four values and commit `33ded43`. The
      spec also says *why* the error went unnoticed: `versione = 0x01` was true for the only
      two archives tested at the time.
      **And the box it points at stayed open until 2026-09-15.** This entry opens by saying
      *"already in `STATUS.md`"*, closing it did not close that one, and for nine days the two
      documents said opposite things about the same file — one with a date on it. Nothing
      compares them, which is the whole lesson: a cross-reference is only as good as the pass
      that walks back along it.

- [x] **E2. A single list of verification commands, in `scripts/check`.** *Weight S.*
      **Why:** the three Rust commands and the four `pnpm` ones are in `CLAUDE.md` and in
      the conventions, i.e. in documents meant for people working on the code with an
      assistant. Someone who just clones the repo won't find them, and without CI nobody
      runs them in their place.
      **What to do:** a `scripts/check` script that runs them all in order and reports what
      failed; a "Verify" section in the README that points to it, along with the hook's
      install command (A2).
      **Done when:** the README, the hook, and `CLAUDE.md` all point to the same script, and
      the list of commands exists in one place.
      **Closed on 2026-09-06:** `scripts/check`, also available as `pnpm check`. Found while
      writing it: `cargo test` **hides** the output of passing tests, so the `skip:` lines
      weren't showing up at all — the advice "after `cargo test` look for those lines", in
      `CLAUDE.md` and in the README, could never have worked. The script uses
      `-- --nocapture` and counts the lines.

- [x] **E3. Update Node to 22 LTS and enable pnpm via corepack.** *Weight S.*
      Already in `STATUS.md`. The version needs to be pinned in `package.json` (`engines`),
      so anyone cloning finds out right away if they're on a different major.
      **Done when:** `node --version` and the `engines` field declare the same major, and
      the pnpm version lives in a single place (`packageManager`).
      **Closed on 2026-09-06:** Node 22.22.0 and pnpm 10.33.0 were already installed; the
      pin was missing. `engines: node >=22 <23` in `package.json` plus
      `engine-strict=true` in `.npmrc`, without which pnpm treats `engines` as a suggestion
      and someone cloning wouldn't notice.

---

## Outside this document, on purpose

- **CI.** An explicit choice, not a gap: no pipeline, no `.github/workflows`. This is a
  one-person project and the checks are local and fast; the place to guarantee them is the
  writer's machine (E2 and A2), not an external service. If the choice ever changes, it
  changes in one place: `scripts/check` is already the list a pipeline would run.
- **The unlock graph (M2).** It's what would raise the project's standing more than
  anything else here, but it's a milestone, not an improvement. It lives in `STATUS.md`.
- **The real webapp and the design system.** The handoff happened on 2026-09-09 and the
  design is under way; the webapp starts when the design system lands. Which also moves
  **B2** above from "later" to "next": with the contract live, a `types.ts` generated from
  the Rust types stops being tidiness and becomes the thing that would have caught a
  variant rename reaching the wire unannounced.
- **Cleaning up `App.vue`.** It's a declared verification page and will be replaced. The
  frontend rules apply to code written from here on: A3 enforces them from the start, and
  the existing verification files are exempted by name in the scanner, not by oversight.
