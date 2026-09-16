# 3.6a — the Tabs settings screen: what the session is, said where it is switched

**Sub-project of 3.6 (Settings and About).** It is the half of 3.6 that could be decided without
the owner; §5 says plainly which half could not, and leaves it.

## 1. What is already built, read rather than assumed

- **`RouteName.TabsSettings` exists and has no screen.** It is in `routeTable.ts` with a path
  (`/settings/tabs`), a title, an icon and `TabOrigin.Settings`; it is the fourth entry of the
  sidebar's Settings section. `router/routes.ts` maps names to components through a
  `Partial<Record<…>>` and falls back to `PlaceholderScreen` — so **the entry is reachable today
  and lands on a placeholder**. Nothing is broken; something is missing, and the sidebar has been
  pointing at it since 3.1.
- **The setting itself already works, end to end.** `resumeTabs` is in `settings.json`
  (`settings_file.rs`), `set_resume_tabs` writes it, and the store applies-first-saves-after like
  the scale and the tray switch. **And turning it off clears the stored session**: `set_resume_tabs`
  calls `guard.set_session(None)` when `resume` is false — checked, because the copy on screen
  promises it and a promise in an interface is worth checking rather than believing.
- **The switch is on the wrong screen.** It lives on **Background**, whose subject is what the app
  does when you close the last window — the tray. `background.intro` had to be written as *"what
  the app does when you close the last window, **and what you find when you open it again**"* to
  cover it: a sentence with an "and" in it, joining two subjects, which is the same smell as a
  commit that needs one.
- **Its copy has drifted behind the feature.** `background.resumeHint` says *"the first window
  reopens on the tabs it had"*. Since 3.7b that is **every** window, in its place, and since 3.7c
  it is the sidebar's width too. Nobody updated the sentence, and nothing could have: no test
  reads prose.
- **3.7b parked a diagnostic here on purpose.** Its report says `SessionTooLarge` is in the About
  dialog and *"moves next to the switch that turns the session off when 3.6 builds Settings"*.
  This is that.

## 2. Decision — the screen is the session's one honest place

The Tabs screen holds **one switch and the truth about what it stores**. It is the only place in
the app where the user can be told what is written down about how they were working, and the
About dialog's third promise — *one file written* — is what makes that worth saying rather than
assuming.

So the screen says, in plain sentences and not a schema: the windows you had and where they were,
which tabs were in each, what each tab was showing, and how wide the sidebar was. **Named, not
enumerated as fields**: a user reading a settings page is not reading a document format.

## 3. Decision — the switch moves, it is not copied

`resumeTabs` leaves Background entirely, and `background.intro` loses its second half. A setting
shown in two places is a setting with two states in the user's head, and the one that is not the
real one is whichever they looked at last.

Background keeps the tray switch alone, which is its whole subject.

## 4. Decision — the diagnostic sits next to the switch that caused it

`sessionStopped` moves from the About dialog to this screen. It is a fact about the session, the
screen is the session's, and the alternative — About — was chosen in 3.7b only because nothing
else existed yet. **The About dialog keeps nothing of it**: a diagnostic in two places is the same
mistake as a switch in two places.

Its limit is unchanged and stays worth stating: the flag is set in the window whose write failed,
which is the elected writer, so another window's Tabs screen shows nothing. That is a limit of
3.7b's design, not of this screen, and it is written down in both reports.

## 5. What this does not decide, and why

**B17's welcome flow is not here.** Items 2 and 3 of B17 — *the app asks which save to play with
and shows a preview of it* — are a product decision about what the app does the first time
somebody opens it, and the owner logged them as their own. Guessing at a welcome flow and
building it would be the expensive kind of wrong: it is the first thing a stranger sees.

**About is not redesigned.** B25 chose a dialog over a tab deliberately and the reason still
holds: what it has to say is short. All that happens to it here is that a diagnostic leaves.

## 6. Tests

**None that are new, and that is the finding rather than an omission.** This screen is wiring: a
switch bound to a store that is already tested, an alert bound to a ref that is already set where
it is set, and prose. There is no DOM environment and no `@vue/test-utils` in this repo — 3.7a
established that and 3.7b lived with it — so a component test would need a dependency added to
assert that a `<Switch>` is on screen.

What guards it instead, and is worth naming so nobody thinks it is nothing: **`pnpm typecheck`
fails on an i18n key that exists in one language and not the other**, and `pnpm scan` fails on a
visible string in a template. Between them, the two ways this change can actually break are
covered. The rest is seen in a window.

## 7. Files

**New.** `ui/src/screens/TabsSettingsScreen.vue`.

**Changed.** `ui/src/router/routes.ts` — the screen joins the map; `ui/src/screens/BackgroundScreen.vue`
— the switch leaves; `ui/src/components/shell/AboutDialog.vue` — the diagnostic leaves;
`ui/src/i18n/messages/{it,en}.ts` — the keys move and the drifted sentence is corrected.

**Unchanged.** Every crate, every command, `stores/settings.ts`, `lib/window/sessionHealth.ts`.
3.6a adds no command, no setting and no token: the setting it puts on screen has been in
`settings.json` since the tray landed.
