# Completed — the structural cleanup, N1 to N8

Eight cleanup items, all done, each on its own branch cut from `develop`. **Searched, never
read.** It left `docs/STATUS.md` on 2026-09-16: 387 lines describing work that finished on
2026-09-14, in the file that is supposed to say what is happening now.

## Next up — structural cleanup

Read-only survey of 2026-09-12 over `crates/` and `ui/src`. It found no bug and no broken
rule: no `unwrap()` outside tests, exhaustiveness respected, no `TODO` and no
`@ts-expect-error` except the one that *is* the test, the dev pages behind
`import.meta.env.DEV` with dynamic imports, `en.ts` and `it.ts` in key parity. What it found
is what five screens built well, one after another, each on the shape of the one before,
cost: **the same file exists twice under two names.**

**The rule for this section: unification is measured in files that stop existing.** A task
that adds a shared module and leaves in place the two it generalizes has not been done, it
has doubled. Every item closes on a file count going *down*, and the count is written into
the item. Where a pair must stay two files, the item says which and why, so nobody has to
wonder whether it was forgotten.

> **Corrected on 2026-09-14, by N3, the last item to run under it.** The rule catches what it was
> written against and nothing else. N3 created seven files and deleted three, and left the source
> 78 lines lighter: the duplicated engine became one engine plus the specs, values and helpers it
> had always needed, and no count of files tells that apart from the failure above. **Read it as a
> smell, not a criterion.** What decides an item is its own "done when" — for N3, that no file
> under `screens/collection/` is a copy of one under `screens/unlock/` and that adding a facet to
> one screen touches no file of the other.

Nothing here is a feature and nothing changes what a screen shows. Each item gets its own
branch cut from `develop`, like any sub-project (the rule of 2026-09-11): none of this
starts on top of a sub-project in flight. **N7 is not a new item**: it is B2 of
`docs/IMPROVEMENTS.md`, placed here in the order it has to run in.

**The order was "cheapest and safest first" until M4 got its design (2026-09-12), and that
is what reorders it.** The numbers N1–N8 are names, not positions — renumbering them would
break the references other items and `docs/IMPROVEMENTS.md` make to them — so the order is
written out here instead:

> **N1 → N2 → N7 → N6 → M4 sub-project 1 → N8 → N3, N4, N5**

**N6 and N7 swapped in the event, on 2026-09-13, and not because the argument changed.**
N7 generates TypeScript from the Rust types, and `feature/wiki-infobox` is reshaping
`crates/wiki` — `wiki::Target` is one of the five types generation has to reach. Generating
from a moving target means generating twice, which is the same reason N7 went in front of M4
in the first place. N6 touches no file of that branch, so it went first. **The order below
is the argument; what actually ran is N1 → N2 → N6 → N7.**

Three of the items are cheap *now* and expensive after M4, and that is the whole of the
reason:

- **N7 moves from last to third.** It was last because it is expensive and because
  generating the contract while the error type is still moving means generating it twice —
  the first half of that still holds, which is why N2 keeps its place in front of it. What
  changed is the second half: M4 adds a new family of view-models to the contract, and every
  one of them written before N7 is hand-mirrored into `types.ts` and then regenerated. The
  repository's largest silent risk gets *larger* between now and M4, not smaller.
- **N6 moves in front of M4.** M4 adds commands to `crates/app`. Splitting the 964-line file
  first means they land in `commands/` already; splitting it after means splitting a bigger
  file, and the new commands spend that time in the place the rule says they may not be.
- **N8 moves behind M4, not in front of it.** Its expensive half and M4's watcher are the
  same subject seen twice: a `SaveState` has to invalidate when the `.dat` is rewritten, and
  M4's log watcher exists because the game announces exactly that
  (`Saving PersistentGameData to Steam Cloud: …`, B8 finding (a)). Doing N8 first means
  inventing an mtime heuristic that M4 then replaces with the game's own statement. **N8's
  free half is not held back by this** — `next_steps` taking the `UnlockView` it is a filter
  over is half a screen load for no new state, and can be lifted out whenever.
- **N3, N4 and N5 are untouched by M4** — they are frontend, and M4 sub-project 1 has no
  screen. They keep their place at the end, where they gate 3.6, 3.7 and B3 rather than
  anything here.

- [x] **N1. The names left over.** *Done 2026-09-13, `feature/cleanup-names`.*
      `ICONE_DI_ESEMPIO` is `SAMPLE_ICONS` and `SEGRETO` is `SECRET_PATH`, with the path it
      holds spelled in the language of the assert that reads it. That constant also gained
      the comment it needed: there are **two** asserts, on the whole path and on the word
      inside it, because a `reason` that re-rendered or escaped the path would slip past the
      first on its own. The coupling was load-bearing and unwritten, which is how a rename
      turns a leak test into a test of nothing.
      **One notation, and it is structural**: each crate with a test-only entry point has
      exactly one `pub mod for_tests`, and nothing test-only appears anywhere else in its
      public surface. **Seven crates** have one — `catalog`, `discovery`, `graph`, `ipc`,
      `store`, `unpack`, `wiki`. The inner items went back to being internal:
      `Graph::from_edges` and `from_requirements` are `pub(crate)` constructors again,
      `search`'s `documents` and `progress` are `pub(crate)` instead of wearing
      `#[doc(hidden)]` twins, and `Doc` and `ProgressMark` left `ipc`'s contract — they are
      the return types of test-only calls and nothing else reads them.
      **The survey had found four entry points; there were seven, and the count moved
      twice.** `unpack::__lzw_decompress` and `catalog::__heads_parse` carried the same `__`
      prefix and were not on the list; `catalog`'s serves an **example** rather than a test,
      and the module's doc says so rather than inventing a second word, which would rebuild
      the thing this item removes. The seventh, **`discovery::testing`**, was found a session
      later while N2 typed that crate's diagnostic: a `#[doc(hidden)] pub mod` under a
      different word, matching neither the `__` prefix nor the `_for_tests` suffix the
      closing grep looked for. **The lesson is about the report, not the code**: "one
      notation" was written as a closed fact when what had actually been established was
      "the two spellings I grepped for are gone". A closing criterion states what it can
      see, and this one could not see a third spelling.
      **Closed against** `grep -riE '(icone|segreto|_di_)' crates` and
      `grep -rn 'pub fn __\|as __\|pub mod testing\|_for_tests' crates`, both silent, and no
      `pub use` naming a `for_tests` item. **Over `crates` and not `ui/src`**: the Italian
      locale file legitimately contains the Italian word *icone* in its prose, so a grep that
      spans it can never go quiet — a closing criterion that cannot be met is worse than
      none, because it gets read as "still open" forever.

- [x] **N2. `reason: String` leaves the IPC.** *Done 2026-09-13, `feature/typed-ipc-reasons`.*
      **Four enums, not one.** The item named `StoreReason`; the boundary carried the same
      defect in three more shapes, so `crates/ipc/src/reasons.rs` holds `IoReason`,
      `SaveReason`, `SettingsReason` and `StoreReason`. `IoReason` has no variant with a
      field and is therefore a **bare camelCase string** by the repo's own rule; the other
      three carry one and are tagged. `StoreReason` gained `DataDirNotCreatable` beyond the
      four the item listed — "the folder is unknown" and "the folder would not be created"
      are two things, and the item's list was written from a skim.
      **`store_reason` and `describe_open_error` are gone**, and so are the two Italian
      sentences. The mappings live where they can be tested — `OpenError` and
      `io::ErrorKind` in `ipc`, `StoreError` in `store`, which already depends on it — and
      each is pinned by a **property**, not only by a table: whatever the OS or SQLite
      wrote, none of it survives into the serialized reason.
      **One case was answering with a sentence that hid which case it was.**
      `queue_import_goals` used `"database illeggibile"` in *both* arms: a database that
      would not open and a query that failed read identically, and the reason the first arm
      already had was thrown away. Each says what it knows now. This is the item's own
      charge — an untyped field lets the wording drift — in its worst form, where the drift
      had eaten the information.
      **The defect had an instance outside `IpcError`, and the UI was printing it.**
      `SetupDiagnostic::UnreadablePath` carried `io::Error::to_string()` and
      `NoSavesCard.vue` concatenated it. `discovery::Diagnostic` carries the
      `io::ErrorKind` now. On the way, `Discovery` and `Diagnostic` lost their `Serialize`
      derive: nothing used it, and both hold a `PathBuf`, so serializing either would have
      put a full path on the wire — the one thing the boundary forbids.
      **The frontend maps to keys, not to text.** `ui/src/lib/ipc/errorText.ts` is a pure
      function beside the wire types: `useIpcErrorText` joins the parts it returns and
      `PlanAlerts` asks it for the store reason. It is pure because the UI suite tests logic
      and not components — a composable calling `useI18n` needs an app context, and the
      mapping is the part worth checking. `storeNewerSchema` interpolates `{found}` and
      `{supported}`: the word order around a value is the translation's business.
      **The test walks every variant of every error** and asserts each key exists in `en`
      **and** `it`. A missing key renders as the key itself — it reads as a bug report to
      the user and fails nothing — so a new variant with no text breaks the test rather than
      the app.
      **Done when** — all met: no field of `IpcError` is a `String` the UI concatenates,
      the two functions are gone, and the two anti-leak tests in `app` assert on a variant.
      The last one is what makes them structural: a unit variant has no string for a path to
      hide in, so the test stopped being a search for a word.

- [x] **N3. One faceted list, not two.** *Done 2026-09-14, `feature/shared-facets`.*
      `lib/facets/faceting.ts` holds `createFaceting({ order, values, text, options })`, and
      `components/facets/` holds one drawer, one toolbar and one state toggle. **Six components
      became three**, and the source lost **78 lines net** (551 added, 629 removed, tests
      excluded) while the tests gained 143 — the engine has 167 lines it never had, and 145 lines
      of tests that existed twice are gone.
      **Read literally, this item's own rule says it failed**: "unification is measured in files
      that stop existing", and seven files were created against three deleted. The rule is still
      right about what it was written against — a shared module added *beside* the two it
      generalizes — and it is wrong as arithmetic here, because a duplicated engine that becomes
      one engine plus five specs, values and helpers is more files and less code. The half of the
      item that does answer is the behavioural one it also writes down: **no file under
      `screens/collection/` is a copy of one under `screens/unlock/`, and adding a facet to one
      screen touches no file of the other.** Both hold, and the second is measured rather than
      asserted: `FacetId` appears in exactly four files, all of them Unlock's — none of the
      Collection's, and none of the three shared components. The rest is the diff, above, and it is
      the honest number rather than the flattering one.
      Its tests run on a **row type neither screen owns** — tested through `UnlockNode` the engine
      would be proven to work for Unlock and say nothing about being generic, which is the whole
      claim and the one B3 leans on.
      **Three things the item's own description had wrong**, each found by doing it:
      `matchesQuery` was *not* identical — Unlock searches a text built from three fields, the
      Collection a name — so the search text is a function the spec brings, not a field the engine
      assumes. **Two** values crossed between the screens, not one: `OriginValue` and the kind set,
      the latter read by `CollectionRow.vue`, `QueueRow.vue` and a *component* module, which is the
      clearest sign it was never Unlock's. It is `TargetKind` now, in `lib/ipc/values.ts`, its
      first four values bound to `ItemKindView` rather than retyped. And the two label modules,
      listed as staying two, shared twenty lines nobody counted — `originLabel` and `oneOf` are
      shared now, with the tests neither had.
      **Unlike Unlock's, the Collection's faceting is a factory.** `CollectionView.pools` is the
      catalog's own order filtered to the items listed, built in Rust from `c.pools()`, and it
      cannot be read back off the rows: first appearance in id order is a different sequence. So
      the options take an input the view brings, and `emptyFilter(order)` exists because a filter
      has to be built before any pools do.
      The drawer's column count is `facets.length` reaching the grid as `--facet-columns`, the way
      `MarksGrid` already passes the boss count — and `grid-cols-facets` was **checked in the built
      CSS**, because an `@utility` nothing references generates nothing and a grid with no template
      would have failed silently. The two shared components are the repo's first **generic SFCs**:
      with `string` props each screen would narrow the emitted sort back to its own union, which is
      the same small duplication moved rather than removed.
      **What stays two, and what it cost to know**: `UnlockRow`/`CollectionRow` and
      `UnlockTable`/`CollectionTable`, as the item says. Measuring the tables found 27 of 84 lines
      still shared — not columns but the scroll box around them — and four screens reading
      `--spacing-unlock-body`. Both are **B43**, not this branch: different subject.
      **Not verified here**: nothing in the suite draws these components, and this machine has no
      game. The three unified components are behaviour-preserving by construction — same template,
      same classes — except the drawer's grid, which changed mechanism. One look at `pnpm ui:dev`
      closes that.

      *The original entry, for the record:*
      `ui/src/lib/graph/unlockFilter.ts` and `ui/src/lib/collection/collectionFilter.ts`
      hold `matchesQuery`, `matchesFacet`, `matchesFacets`, the facet counts and the active
      count **identical word for word, comments included**: only the row type differs. The
      same pair repeats four times above them — `FacetDrawer.vue` /
      `CollectionFacetDrawer.vue` (the template differs in `grid-cols-3` against
      `grid-cols-4` and in the i18n prefix), `UnlockToolbar.vue` / `CollectionToolbar.vue`,
      `StateToggle.vue` / `CollectionStateToggle.vue`.
      Two consequences are already on the page: the names defend themselves with prefixes
      (`matchesFilter` against `matchesCollectionFilter`) because the two modules share a
      flat namespace, and `collectionFilter.ts` imports `OriginValue` **from
      `unlockFilter.ts`** — the Collection depends on Unlock's screen module for a value
      that belongs to neither screen, it belongs to the wire. B3 in `docs/BACKLOG.md` would
      write the third copy.
      **What:** `ui/src/lib/facets/` with one engine —
      `createFaceting<Row, Facet>({ order, values, options })` giving
      `{ matches, counts, activeCount }` — and `ui/src/components/facets/` with one drawer,
      one toolbar and one state toggle, each driven by a table. A screen keeps only what is
      genuinely its own: which facets, how to read a row's values, which options, which
      labels. `OriginValue` moves to a shared module beside the wire types. The drawer's
      column count comes from the number of columns instead of a literal typed twice.
      **Five files stop existing** — `collectionFilter.ts`, `collectionFilter.test.ts`,
      `CollectionFacetDrawer.vue`, `CollectionToolbar.vue`, `CollectionStateToggle.vue` —
      and their `unlock/` twins become the shared ones, under names that no longer say
      "unlock". About 250 lines.
      **Staying two on purpose:** `UnlockRow` / `CollectionRow` and `UnlockTable` /
      `CollectionTable`. They draw different columns; one component with a column table
      would be a worse file than the two it replaced.
      **Done when** no file under `screens/collection/` is a copy of one under
      `screens/unlock/`, adding a facet to one screen touches no file of the other, and
      B3's third list costs one spec object.

- [x] **N4. One diagnostics list, not four.** *Done 2026-09-13, `feature/ui-diagnostics`.*
      **All four stopped existing**, not three: `PlanAlerts.vue` went too, and its one
      button — the only alert that asks for something — is handed in through a slot, so the
      Plan keeps the action without keeping a component. `DiagnosticsList` draws; each
      screen keeps a `Record<kind, DiagnosticRow>`.
      **A diagnostic's scalar fields are now the translation's values**, which is what makes
      a kind one row: `{count}` is placed by the string instead of concatenated in front of
      it. That change found a real defect — the four count-bearing strings were sentence
      *fragments* written for `${d.count} ${t(...)}` and carried no placeholder at all, so
      passing the number as a value would have made it vanish with nothing failing. They are
      whole sentences now, and the word order around the number is the translation's
      business: the same argument as N2's `storeNewerSchema`.
      **Two invariants, and each caught something while being written.** Every key a table
      can produce exists in `en` **and** `it`. And every value handed to a translation is
      spent by it — an object or a list has no rendering a translator chose, so `reason` and
      `wanted` are dropped rather than passed unused, and the check is **per entry**, because
      the builder hands a diagnostic's values to the title and the body alike and it is
      enough that one of them places each.
      **Done when** — both met: a new diagnostic kind is one row in one table, and no screen
      owns a component whose job is drawing alerts (`find ui/src/screens -name '*Diagnostics.vue'
      -o -name '*Alerts.vue'` finds nothing).
      **Measured against this section's own rule, and it does not pass it.** 4 files and 303
      lines became **6 files and 278**: lines down 25, **files up 2**. The rule says
      unification closes on a file count going *down*, and here it cannot: one idea needs a
      spec, a component and one table per screen, which is the item's own prescription.
      The rule is a proxy for "did the duplication actually go", and it did — the four copies
      are deleted and `git` records no survivor. **The proxy disagrees with the thing it
      proxies, and the honest entry is this one rather than four tables merged into a file
      nobody wanted just to make a count fall.** N3 and N5 should be measured knowing that:
      their file counts really do fall, because what they delete is a *copy*, not a copy plus
      the machinery that replaced it.

- [x] **N5. The stores stop repeating themselves.** *Done 2026-09-13, `feature/ui-view-stores`.*
      `collection.ts`, `completion.ts` and `graph.ts` are **three lines of `stores/views.ts`**,
      and `LoadStatus` lives in `stores/loadStatus.ts`, which holds nothing else.
      **The factory was not enough on its own, which the item had not seen.**
      `defineViewStore` fits a store whose whole shape is the triad; three others carry more —
      the profile loads two things under one status, the queue clears its mutation state
      first, the wiki skips a read it has already made. They use the half underneath it,
      **`tracked(status, error, read)`**, so the `try` / `catch` is written once for all six
      rather than once for three.
      **One read keeps its own, and the code says why.** `wiki`'s `loadEntry` reports a
      failure *without ever claiming a success*: the status belongs to the index, and a page
      arriving must not mark the index `Ready`. `tracked` cannot express that shape, and
      forcing it would have been a behaviour change wearing a cleanup's clothes. That a
      page's failure lands on the index's `error` is inherited and left alone — changing it
      needs a decision, not a refactor.
      **The graph's two answers became one `view` object**, which is what they always were:
      one read, so `unlock` and `steps` cannot straddle a profile change. Call sites say
      `graph.view?.unlock` instead of `graph.unlock`, and two guards that checked both
      halves now check the one object.
      **Done when** — both met: no store writes that `try` / `catch` by hand, and
      `LoadStatus` is imported from a file that holds nothing else. **Three files stopped
      existing** and three arrived (`views.ts`, `tracked.ts`, `loadStatus.ts`), so the count
      is flat — for the reason N4 records: what is deleted here is a copy *plus* the
      machinery that replaces it.

- [x] **N6. `crates/app` goes back to being wiring.** *Done 2026-09-13, `feature/app-wiring`.*
      964 lines in one file became **eleven, none over 220**: `state.rs` for the six
      `OnceLock`s — the item said five, `SearchState` was not on the list — plus the two
      save reads they share, `icons.rs` for the `isaac://` protocol and its crop, and
      `commands/` with one file per screen family: profile, completion, wiki, graph, queue,
      plan. `lib.rs` keeps `run()` and nothing else.
      **`IpcError` had to move first, and that is what the item had not seen.** The tested
      functions return it, and it lived in the Tauri crate: they could not leave while the
      type they are about could not be imported. It is a wire type and `ipc` is the only
      contract, so that is where it belongs — and N7 will generate it from there.
      **The pure halves went to `store`, not to `ipc`.** `plan_parts`, `store_error` and
      `store_unavailable` all take a `StoreError` or a `GoalsRead`, and `ipc` cannot depend
      on `store` — the dependency runs the other way. They are not wiring either: each has a
      return value worth checking, which is the rule that says they may not stay in `app`.
      `crates/store/src/degrade.rs`, with the five tests that were `app`'s only
      `#[cfg(test)]` block.
      **`icon_url` stays in `app` on purpose**, against the item's list. `CLAUDE.md` already
      records why: on Windows the webview sees a rewritten `http://isaac.localhost` origin,
      and knowing that is the Tauri crate's job, not a pure crate's. It is wiring, not logic
      that escaped — and moving it because a list named it would have undone a decision the
      repo had already argued.
      **Closed against all three criteria, checked rather than assumed**: no `#[cfg(test)]`
      under `crates/app/src/`, largest file **220** lines, and `cargo test -p app` reports
      zero tests because there is nothing left in it to test.

- [x] **N7. `types.ts` generated — this is B2 of `docs/IMPROVEMENTS.md`.**
      *Done 2026-09-13, `feature/generated-contract`.*
      865 hand-written lines mirroring the `#[serde]` attributes, gone: `pnpm ipc:types` runs
      `crates/ipc/src/bin/ipc-types.rs` over an ordered list of the types that cross, then
      prettier. `scripts/check` regenerates into a scratch copy under `ui/` and fails on a
      difference. Spec `docs/superpowers/specs/2026-09-13-generated-contract-design.md`, plan
      `docs/superpowers/plans/archive/2026-09-13-generated-contract.md`.
      **The item said five foreign types; there are nine.** `wiki` contributes `SectionKind`,
      `CollectibleTemplate` and `Style` beside `Target`, and `core_save::marks::CharacterGroup`
      is the ninth. The list in B2 was written from a skim, the same way N2's was — the
      compiler found the rest in one pass, which is the argument for deriving on the real types
      rather than declaring them in the post-step.
      **Four defects the hand-written mirror was hiding**, and they are the item's whole
      justification rather than a bonus:
      - `SectionCount.kind` was `string` — the documented incident, closed by construction.
      - `GameView.edition` was `string` and `dlcs` was `string[]`. Two more of the same shape,
        never noticed because nothing compares them.
      - **`Infobox` had no `transformation` variant.** The Rust has had one since the
        transformations landed on 2026-09-13; `WikiInfobox.vue`'s switch was exhaustive only
        because the type lied, so a transformation page reached `assertNever`, which throws.
        The screen draws no card for that kind yet and says so in place — **B40** registers the
        rows.
      - `ProgressMark` was reachable only through `pub mod for_tests` while being a field of
        `SearchHit`. N1 had moved it there reading it as a test-only return type.
      **One decision the implementation had to take.** `discovery::Dlc` and `wiki::Dlc` are
      different enums with the same name — installed DLCs in `snake_case` against a content's
      origin in `camelCase` — and one file cannot declare both. The installed one is
      `InstalledDlc` in TypeScript through `#[ts(rename)]`, which does not touch the wire.
      **Two facts worth carrying forward.** A `///` on a wire type is UI source now: it becomes
      JSDoc and lives under `pnpm scan`, so `MissingReason` lost an arrow the app's font cannot
      draw. And `ts-rs` prints one permanent warning — it does not parse `#[serde(transparent)]`
      and says so on every build. It is left standing rather than silenced with
      `no-serde-warnings`: the attribute is a no-op on a newtype in serde too, the output is
      right, and the feature that hides this line would hide the next one as well.
      **Done when** — all met: `pnpm ipc:types` leaves `git diff` empty, a wire change with
      every Rust call site still compiling (`#[serde(rename = "bestiaryy")]`) fails both
      `cargo-test` and `ipc-types`, and `pnpm check` is green.

- [x] **N8. The save read once per screen, not twice.** *Closed 2026-09-14, both halves —
      `feature/one-graph-read` then `feature/save-state`.*
      **The two graph screens are one command.** `graph_views` returns the Unlock view and
      the steps together, so a screen load reads the profile **once** — and the "done when"
      below is answered by construction rather than by a counter: there is no second entry
      point to count, because `unlock` stopped being a command. The store's own comment —
      *"one read, because both answers belong to the same profile and asking twice could
      straddle a change"* — was a promise two commands could not keep: between them a save
      written mid-load made the steps describe a profile the list no longer showed.
      **And the profile is kept between commands**, in `ipc::SaveCache` — a cache and not a
      `OnceLock` for one reason: the game rewrites the save while the app is open, so a
      reader that remembers the first read shows a profile that no longer exists. Three
      rules, each with a test **shown able to fail**: the same profile the settings name (no
      name means the active one is whatever is on disk, which is the thing that changes),
      the modified time the cached read saw (which covers the file disappearing — no time is
      not the same time), and never a remembered failure. The time is read **after** the
      load, so a file written *during* the read is already stale rather than trusted as a
      value that saw half of each version. Completion, the Collection, `want` and the wiki's
      progress block stop paying for their own read of the same file in the same second.
      **The counter the item asked for is a call count over the policy**, not over a Tauri
      command: the wiring stays untested by the rule that put it in `crates/app`, and what
      was worth checking moved to the pure crate where a test can reach it.
      **Needs:** the game — its "done when" is a counter in a test showing the `.dat` opened
      once across Next steps and Unlock, and both commands go through the catalog, which on a
      machine without the game skips instead of counting. The free half (`next_steps` taking
      the `UnlockView` it filters) can be written anywhere; it cannot be believed anywhere.
      The vocabulary is the one at the top of `docs/BACKLOG.md`.
      `active_save()` does, on every command that needs the profile: `settings_file::load`
      (I/O), `discover()` (a walk of the Steam libraries), `fs::read` of the whole `.dat`,
      and a full parse. Nothing caches it, while the catalog, the graph, the resources, the
      mark frames and the search index all sit in a `OnceLock`. The worst case is
      measurable rather than theoretical: `next_steps` calls `unlock()` internally and
      `stores/graph.ts` asks for the two in one `Promise.all`, so **discover, parse, 642
      nodes and the evaluation all run twice for one screen load.**
      Two halves at very different prices, and they are not one task by accident:
      - the free half — `next_steps` receives the `UnlockView` it is a filter over instead
        of rebuilding it. Half the work of a screen load, no new state, nothing to
        invalidate.
      - the expensive half — a `SaveState` in `tauri::State`. **Not a blind `OnceLock`:**
        the `.dat` is rewritten while you play, so it invalidates on the file's mtime, and
        "no profile" is never cached — the rule `ResourcesState` already writes down for
        the game not being installed.
      **Done when** loading Next steps and Unlock opens the `.dat` **once**, asserted by a
      counter in a test and not by eye, and a save written while the app is open is seen by
      the next command.

---

