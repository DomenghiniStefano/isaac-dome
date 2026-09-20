# Fixtures

What `pnpm ui:dev` and `pnpm ui:test` read when there is no backend behind them. Everything
here is **data recorded once**, never regenerated: nothing in the repository produces it any
more, so a field a payload predates is filled in by the fixture that reads it, in
`src/lib/ipc/fixtures/`, and said once on the console.

| file                             | what it is                                                                                                           | who reads it               |
| -------------------------------- | -------------------------------------------------------------------------------------------------------------------- | -------------------------- |
| `index.json`                     | the ids, kinds and names of every item, trinket, achievement, boss and character, as the game's own files spell them | `collection.ts`, `wiki.ts` |
| `payload/unlock.json`            | the `unlock` view of the reference profile, 2026-09-08                                                               | `graph.ts`, `wiki.ts`      |
| `payload/next_steps.json`        | the `next_steps` view of the same profile                                                                            | `graph.ts`                 |
| `payload/extraction_report.json` | the wiki dataset's own provenance                                                                                    | `wiki.ts`                  |
| `wiki/*.json`                    | eleven wiki pages with their real text                                                                               | `wiki.ts`, `search.ts`     |

They sit **outside `src/`** on purpose and are reached through `import.meta.glob`, so that no
file of data joins the TypeScript project.

**No drawing is here, and none ever will be.** The app cuts its sprites from the user's own
copy of the game at runtime — that is the third promise in `CLAUDE.md` — so the development
server, which has no copy, answers `null` for every image. A fixture that shipped the game's
art would break that promise on every clone; `scripts/check-no-game-assets.mjs` fails the run
if one ever tries.

`ATTRIBUTION.md` covers the wiki pages: they are CC BY-SA 4.0, like `dataset/`, and they carry
their source with them.
