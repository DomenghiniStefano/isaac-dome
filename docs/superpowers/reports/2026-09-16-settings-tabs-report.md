# 3.6a — the Tabs settings screen: report

**Branch** `feature/settings-tabs`, cut from `develop`. **Spec**
`docs/superpowers/specs/2026-09-16-settings-tabs-design.md`. There is no separate plan: the spec
is four decisions and the work is one screen, and a plan would have restated it.
`pnpm check` all green: **17 skips, 1483 real files touched**.

---

## What the window has not been asked yet

**Nothing here has been seen in a real Tauri window**, and this one is presentation, so the gap is
larger in proportion than in 3.7's three branches:

- [ ] the Settings sidebar's fourth entry opens a screen and not a placeholder
- [ ] the switch turns the session off and on, and the failure alert appears when the write fails
- [ ] Background reads as one subject now that it holds one switch
- [ ] the "what is saved" lines read as sentences and not as a list of fields

## Three things read rather than assumed

**The route existed and the screen did not.** `RouteName.TabsSettings` has been in `routeTable.ts`
since 3.1 with a path (`/settings/tabs`), a title, an icon and `TabOrigin.Settings`, and it is the
fourth entry of the sidebar's Settings section. `router/routes.ts` maps names to components
through a `Partial<Record<…>>` with `PlaceholderScreen` as the fallback — **so the entry has been
reachable and landing on a placeholder for five days**, and nothing was broken enough to notice.

**The copy promised something, and the code keeps it.** *"Off … what was saved is deleted"* is a
promise in an interface, which is worth checking rather than believing: `set_resume_tabs` calls
`guard.set_session(None)` when `resume` is false. It is true, and now that it has been read once
it is written down where the next reader will find it.

**The copy also said something that had stopped being true.** *"The first window reopens on the
tabs it had"* — since 3.7b that is **every** window, in its place, and since 3.7c the sidebar's
width too. The sentence drifted behind the feature over two branches on the same day, and
**nothing in this repo could have caught it**: no test reads prose, and the doc-reference check
reads paths, not claims. It is corrected here, and the general shape is worth naming: a feature
that changes what a setting does leaves its description behind, and the description is the only
part of it the user reads.

## The decisions

**The switch moved, it was not copied.** A setting shown in two places has two states in the
user's head and the one that is not real is whichever they looked at last. Background keeps the
tray switch, which is its whole subject; its intro loses the clause — *"and what you find when you
open it again"* — that was only there because the wrong switch was on the screen. **An intro that
needs an "and" joining two subjects is the same smell as a commit that needs one.**

**The diagnostic moved too**, from the About dialog to beside the switch, which is what 3.7b's
report said would happen when this screen existed. A diagnostic in two places is the same mistake
as a setting in two places, so About keeps none of it. Its limit is unchanged and still worth
stating: the flag is set in the window whose write failed — the elected writer — so another
window's Tabs screen shows nothing.

**What is saved is said in sentences.** Somebody reading a settings page is not reading a document
format, so the four lines name the windows and where they were, the tabs of each and which was in
front, how each tab was being read, and the sidebar's width. It is the only place the app can tell
somebody what it writes down about how they were working, and About's third promise — *one file
written* — is what makes that worth saying rather than assuming.

## No new test, stated as a finding

The screen is wiring: a switch bound to a store that is already tested, an alert bound to a ref
that is already set where it is set, and prose. There is no DOM environment and no
`@vue/test-utils` here — 3.7a established that and 3.7b lived with it — so a component test would
mean adding a dependency to assert that a `<Switch>` is on screen.

What guards it instead, named so nobody reads this as "untested": **`pnpm typecheck` fails on an
i18n key present in one language and missing from the other** (Italian is the schema and English
is typed against it), and **`pnpm scan` fails on a visible string in a template**. Those are the
two ways this change can actually break. The rest is a window.

## What it did not decide

**B17's welcome flow is untouched.** Items 2 and 3 — *the app asks which save to play with and
shows a preview of it* — are a product decision about the first thing a stranger sees, and the
owner logged them as their own. **This is the piece of 3.6 that is deliberately left**, and saying
so is the point: 3.6 is not closed, and a Tabs screen existing must not make it look closed the
way a `tables` key would have made B27 look closed.

**About is not redesigned.** B25 chose a dialog over a tab on purpose and the reason holds. All
that happened to it is that a diagnostic left.
