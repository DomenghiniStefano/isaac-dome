# 3.7c — the measured sizes: report

**Branch** `feature/tabs-sizes`, cut from `develop`. **Spec**
`docs/superpowers/specs/2026-09-15-tabs-session-design.md` (§8, §11 item 3), **plan**
`docs/superpowers/plans/archive/2026-09-16-tabs-sizes.md`. `pnpm check` all green: **17 skips,
1483 real files touched**, in the second worktree where `samples/` is a junction.

**Two commits**, the suite green at each. It is the smallest of 3.7's three branches and the last
of them, and the reason it waited until here is that it needed a document to live in.

---

## What the window has not been asked yet

**Nothing here has been seen in a real Tauri window**, which for 3.7c means one decision is
unjudged rather than merely unverified — see *one value, shared live* below. What only a window
can say:

- [ ] the sidebar sized, the app closed and reopened: it comes back at that width, on the first
      paint and not by snapping to it a beat later
- [ ] two windows open: dragging one window's edge moves the other's, which is decision 1 and is
      the thing to judge rather than assume
- [ ] a torn-off window opens with its creator's sidebar and not with the default
- [ ] a width stored by a build with other bounds opens inside today's

A `- [ ]` here is a caveat about this work, not a task for someone else.

## The measurement that changed the scope

§8 of the spec names **two** keys, `sidebarWidth` and `tables`, and §11 calls 3.7c "the sidebar's
width and each table's size". Only the first has a value to store. B27's own ordering puts *fill
the page* first and *resizable by hand* second, and **neither is built**: `grep -rn
"useResize\|resizable"` over `ui/src` finds nothing, and the one resize gesture in the app is the
sidebar's, exactly as B27 described it on 2026-09-12 — four days on, unchanged.

So `tables` is **not** added. A named place for a value nothing produces is one more thing every
reader has to ask about and nothing to store, and adding it would have let B27 look closed when
two of its three parts are untouched. The entry stays open and now says which two.

## The decisions, and the one that costs something

**One value, shared live.** §8 puts `sidebarWidth` beside `windows` and not inside one, so the
document holds a single number. Two windows holding different widths would mean the document
silently keeps whichever was written last, with the user finding out which at the next start — so
the value is shared at runtime as well, and dragging one window's edge moves the others'. **This
is the cost, stated rather than discovered**: it may well read as wrong in front of two windows,
and if it does, the fix is not a second stored value but a per-window width in the document, which
is a different §8. It is written here so that a change of mind is a decision and not a
rediscovery.

**No version bump.** The module's own comment predicted this key by name — *"3.7a did not move it,
and neither will 3.7c's sidebar width"* — and the rule behind it holds: an older app ignores a key
beside `windows` and is wrong about nothing. A test pins that the version stayed at 2.

**The document refuses nonsense, the component enforces the bounds.** `readSession` keeps a finite
number and drops everything else; `clampSidebarWidth` puts it between 168 and 420 where the
sidebar is drawn. Splitting it the other way would have put the design file's pixels inside a
parser — and it also means a width written by a build with different bounds comes back inside
today's rather than being trusted.

**The echo guard does not depend on a flush.** A window that hears `Layout` sets the shared ref,
which wakes its own watcher, which would broadcast it back and wake everyone else's: one drag,
a round of messages per window, for ever. The first attempt was a synchronous flag around the
assignment — **and it would not have worked**, because Vue's `watch` flushes on the next tick and
the flag is already down by then. Recording the value as *announced* before setting it is
independent of when the watcher runs.

## What it did not touch, and that is the point

`components/shell/sidebarWidth.ts` and `SectionSidebar.vue` did not move: the gesture, the bounds,
the keyboard step and the double-click reset were right and already tested, and all 3.7c decided
was where the number they produce lives. No Rust, no IPC command, no migration, **and no number**:
3.7c adds not one constant.

**Nothing under `ui/src/lib/` imports from `ui/src/components/`** — 0 occurrences before this
change and 0 after, checked both ways. It is why the shared ref holds a bare `number | null` with
no default: the default is the sidebar's, the sidebar is a component, and `App.vue` is the one
place allowed to know both.

## 3.7 is closed as a spec

Three branches, three reports, one spec:

| | what it made a tab, a window or the app keep |
|---|---|
| 3.7a | a tab owns what it is showing — facets, sort, selection, scroll — across a tear-off and a restart |
| 3.7b | the session is windows of tabs, and the window that writes it is elected |
| 3.7c | the sidebar you sized is the sidebar you get back |

**And none of the three has been seen in a window.** That is one gap, not three, and it is the
same one B6 and B39 both carry as `nothing, then a window`.
