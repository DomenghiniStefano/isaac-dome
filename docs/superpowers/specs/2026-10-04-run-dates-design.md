# Run dates — a launch carries when it was written, and every run has a day

Card #92, born from B55's closure (2026-10-02). Design agreed with the owner on 2026-10-04.

## What it is for

The Runs screen orders runs and cannot date them: its intro says "the log carries no
timestamps, so runs are ordered by session". B55 settled where a date comes from without
Steam:

1. **an online session** — its folder name is a wall clock (`09_12_2026__13_34_26`);
2. **a `log.txt` the app reads live** — and
3. **the one `log.txt` found at startup** (the last launch; the game wipes the earlier ones) —
   both dated by the file's own modification time.

## What it is not

- **Not Steam's logs.** B55 closed that door; nothing here reads outside the game's folders.
- **Not a per-run clock.** All runs of one launch share the launch's date. A per-event clock
  would date only what the app watched live, and was weighed and dropped.
- **Not an invented date.** A source the app read before this migration stays undated, and
  the screen says so.

## A defect this fixes on the way

Every `log.txt` source, current or not, reaches the screen as `RunSource::Live`
(`StoredSource::run_source`). So every launch in the archive is drawn as one "live" group
(`runOrder.ts`), and the first run of two launches both get the key `live:#1` (`runKey.ts`) —
duplicate keys in a virtualized list. A date per launch means nothing while launches cannot be
told apart, so they are told apart here.

## The decisions

**One column, the file's mtime, refreshed on every read.** `written_unix` on `sources`,
seconds since the epoch, nullable. The live file and the file found at startup are one rule:
whatever read the file stamps its modification time. It dates a launch's **last write** — the
day a launch was played, which is what the screen shows.

**The day on screen, the time in a tooltip.** A session's time is when it *started* and a
launch's is when it *last wrote*: side by side in a column they would read as the same thing.
The day is true of both. The tooltip names which time it is.

**Time zones are the browser's.** No crate in the workspace handles local time, and the
folder name is a local wall clock. Rust sends an epoch for a launch and the name for a
session; the UI parses the name **as local time** — today it parses it as UTC, which was
right only while it was used to order and never shown.

## Store — migration 7

```sql
ALTER TABLE sources ADD COLUMN written_unix INTEGER;
```

Nullable and additive, migration 5's shape: existing rows get `NULL`, which reads as "not
dated" and is true for every one of them. The latest launch is dated on its next read if the
app resumes it; older launches stay `NULL`.

- `insert_log_source(key, written)` and `append_to_log(id, key, events, written)` take
  `Option<i64>` and write it **in the same transaction** as the offset and the events.
- **A missing mtime never erases a stored one**: `written_unix = COALESCE(?, written_unix)`.
- `StoredSource` gains `written_unix: Option<i64>`; `SELECT_SOURCE` and `source_row` read it
  (one contract, written once).

## log-watch

- `read.rs` gains `modified(path) -> Option<i64>`: the file's mtime in epoch seconds, `None`
  when the OS cannot say. Never an error — a date is not worth failing an ingest over.
- `Ingest::live_log` reads it and passes it to whichever store call it makes. `session` does
  not: a session's date is its name.

## IPC — `RunSource` gets a third variant

```rust
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum RunSource {
    /// The latest launch of `log.txt`: the one the game may be writing now.
    Live { written_unix: Option<i64> },
    /// An older launch. `id` is the source's row id, opaque: it only tells launches apart.
    Launch { id: i64, written_unix: Option<i64> },
    /// One online session, by its folder's name.
    Session { name: String },
}
```

`Live` gains a field, so its TypeScript changes with it; it was already tagged.
`StoredSource::run_source` takes whether the source is the latest launch — the store knows,
`archived_runs` reads every source and the latest `log` id once. A session row with no name
stays read as a launch, as today.

The id is a row number, not a path or an offset: the boundary rule is untouched. A JSON-shape
test pins `writtenUnix` (camelCase, inside struct variants) and the `launch` tag.

## UI

- **`sessionTime`** parses the name as local time (`new Date(y, m - 1, d, h, mi, s)`), with
  the same rollover guard.
- **`runTime(run)`**, new beside it: the instant a run is dated by, or `null` — the session's
  name, or a launch's `writtenUnix * 1000`.
- **`orderRuns`**: `Live` first, by decision; then sessions **and past launches** together,
  newest first by `runTime`; a source with no time keeps the place the archive gave it — the
  rule the function already has for unreadable names, now applied to undated launches too.
- **`runKey`**: `launch:<id>#<ordinal>`.
- **`RunsTable.vue`** gets a Date column: the day through `Intl.DateTimeFormat` in the UI's
  locale (`Sat 3 Oct`), the tooltip `started 13:34` or `last written 21:04`, and `—` with
  "recorded before dates were kept" for an undated source. Strings in `it` and `en`.
- **The intro** loses "the log carries no timestamps, so runs are ordered by session", in both
  languages.

## Documents

- `docs/architecture.md`: the migration count goes from 6 to 7, redrawn in the same commit as
  the migration.
- `CLAUDE.md`'s `store` row gains migration 7.
- `docs/BACKLOG.md` B55's closing paragraph points at this spec.
- The run-order comments that say "there is no timestamp on the wire" are rewritten in the
  commit that makes them false.

## Tests (first)

- **store**: migration 7 on a version-6 file keeps its archive and reads `NULL`; a write
  stamps the date; a `None` after a date keeps it; the latest launch reads as `Live`, an older
  one as `Launch` with its id.
- **log-watch**: an ingest stamps the file's mtime on its source.
- **ipc**: the JSON shape of all three variants.
- **ui**: local-time parsing (the hour that comes back is the hour in the name); ordering with
  past launches interleaved with sessions by date and an undated launch keeping its place;
  two launches' first runs with different keys; the day and tooltip formatting.

## What only a window can say

The column itself, its width, and the tooltip on a real archive. The card goes to UAT with
`NEEDS WINDOW`.
