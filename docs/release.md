# Releasing, and how the app updates itself

Everything about publishing a version, in one place. Two of these steps happen once and the
rest happen every time.

**There is no GitHub Action, by decision** — the same decision that keeps `scripts/check`
instead of CI. The steps live here and nowhere else, which means this document going stale is
the failure mode to watch: the only thing that checks it is somebody reading it.

The design and what was verified to write it: `docs/superpowers/specs/2026-09-20-app-update-design.md`.

---

## Once, before the first release

### 1. The signing key

```powershell
pnpm tauri signer generate -w $HOME/.tauri/isaacdome.key
```

Give it a password. Two halves come out:

- **The private half stays off this repository, and it is not recoverable.** Lose it and every
  installation in the world stops being able to update — with no message and no way back except
  downloading an installer by hand. It needs a copy somewhere that is not this disk.
- **The public half goes into `crates/app/tauri.conf.json`**, as `plugins.updater.pubkey`. It is
  text, it belongs in the repository, and it is what lets an app refuse an installer that is not
  ours.

**Until it is filled in, `pubkey` is an empty string, and that is on purpose.** An empty key
fails closed: `PublicKey::decode` rejects it, so a download can only ever end in `rejected`,
never in an unverified install. A release build still starts, which is what an empty valid
string buys — and `pnpm release:manifest` refuses to write anything while it is still empty, so
the placeholder cannot reach a release by accident.

### 2. The repository has to be public

The endpoint is a plain URL that the app fetches with no credentials, because constraint 2 of
`CLAUDE.md` forbids a key in the binary and forbids asking the user for one. A **private**
repository's release assets need an `Authorization` header, so a private repository simply
cannot serve this.

`github.com/DomenghiniStefano/isaac-dome` was private until this task. Making it public exposes
the whole history, `docs/` included. It has never held a key, `samples/` has always been ignored,
and the signing key from step 1 must never be committed afterwards either.

---

## Every release

**The whole sequence, from the main worktree**, which holds `develop`. The steps below explain
each one; this is the order, and the git moves around them that the steps leave out:

```powershell
# on develop, pnpm check green
pnpm bump minor
git add package.json; git commit -m "chore: release 0.2.0"; git push origin develop

git switch master
git merge --ff-only develop
git push origin master

pnpm release --publish --notes <notes.md>     # in your own terminal, see below

git switch develop
```

Then, on the board, the cards in `DA RILASCIARE` go to `Done`: they are now in the hands of
somebody using the app. Cards still in `UAT` stay there even when their code shipped — approving
them is the owner's call, and the release does not make it.

- **`master` moves only here, and only when the owner has said to cut a release** (`CLAUDE.md`).
  The main worktree checks it out for the length of the publish and returns to `develop`
  straight after, so `develop` is back where every other worktree expects to find it.
- **The release carries all of `develop`**, `UAT` cards included — `master` is fast-forwarded,
  not cherry-picked. Look at what `UAT` holds before step 4 if that matters for this release.
- **Run the publish in a terminal of your own, not through an agent's shell.** It reads the key's
  password with `Read-Host -AsSecureString` and then asks you to type the version: an agent's
  shell has no interactive input, and the password is not something to hand to one anyway.
  Measured on 2026-09-23 through Claude Code's `!` prefix, which runs bash: the backslashes of a
  Windows `--notes` path were swallowed (`C:UsersstefaAppData…`) and the script stopped on "no
  release notes". Forward slashes survive both shells.

**Warnings every build prints, and which are expected.** A release build on 2026-09-23 (0.2.0)
printed four. Two were fixed the same evening — `INEFFECTIVE_DYNAMIC_IMPORT` on `main.ts`'s two
imports, and the 725 kB entry chunk, gone once the screens load when first opened — and two
remain, neither of which stops the build:

| warning | status |
|---|---|
| the bundle identifier `dev.isaacdome.app` ends with `.app` | macOS-only conflict; the app is Windows-only, and changing it moves `%APPDATA%\dev.isaacdome.app` (settings, `isaacdome.db`) out from under every installation |
| `PLUGIN_TIMINGS` | Vite's own profiling note, informational |

A warning that is **not** in this table is new, and worth reading before typing the version.

### 1. Bump the version

```powershell
pnpm bump minor                   # or patch, major, or an exact 1.2.3
```

Commit the result on `develop` and push it. **The root `package.json` is the only place the
version is written**, since 2026-09-23: `crates/app/tauri.conf.json` names that file as its
`version` (`"../../package.json"`), and Tauri reads the number from there. `crates/app/Cargo.toml`
keeps a version of its own and Tauri ignores it when the config names one, so there is nothing
to keep in step. `pnpm release:manifest` refuses a config that writes a number back, because
Tauri would build that one while the manifest announced the other.

**`pnpm bump`, not `pnpm version`.** The bare command commits *and tags* `v<version>` wherever
you are — `develop`, at this step — and step 4 then stops on "the tag already exists", because
the tag belongs on `master` and is created there. `bump` is `pnpm version --no-git-tag-version`,
which only rewrites the file. **Neither `.npmrc` nor `pnpm-workspace.yaml` can switch the tag
off**: measured on 2026-09-23 on pnpm 12.4.1, `git-tag-version=false` in the first and
`gitTagVersion: false` in the second are both ignored, and the flag is the only thing it obeys.

### 2. Build, signed

```powershell
pnpm release
```

That is `scripts/release-build.ps1`: it asks for the key's password, sets the two variables for
that one process, runs `pnpm build` and then step 3, and clears the password on the way out —
on a failed build and on Ctrl-C as well. **It stops there and touches nothing outside this
machine.** Step 4 is what publishes, and it is a flag away.

**Do not set those two variables permanently.** A password in the user environment lives in the
registry in plain text, readable by anything running as that user, which gives back most of what
choosing a password was for: the key file alone stops being useless to whoever grabs it. If you
need the steps by hand anyway:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = (Get-Content "$HOME\.tauri\isaacdome.key" -Raw).Trim()
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<the password>"
pnpm build
```

**The contents, through `TAURI_SIGNING_PRIVATE_KEY`.** The documentation says that variable takes
a path *or* the key itself; the contents are what its own name describes, and after the mistake
recorded below there is nothing to gain from taking the ambiguous half of that sentence twice.

`bundle.createUpdaterArtifacts` is on, so the NSIS setup comes out with a `.sig` beside it under
`target/release/bundle/nsis/`.

**What a build without the private key does — measured on 2026-09-20, not guessed.** It builds
**both installers to the end**, then fails on the signing step with exit code 1 and this message:

```
A public key has been found, but no private key.
Make sure to set `TAURI_SIGNING_PRIVATE_KEY` environment variable.
```

So the bundles exist and the `.sig` does not. Two things follow, and the second one cost a build:

- **A failed release still leaves installers on disk.** `target/release/bundle/` is not evidence
  that anything was signed, and step 3 is what tells the difference.
- **The bundler reads `TAURI_SIGNING_PRIVATE_KEY`, not `TAURI_SIGNING_PRIVATE_KEY_PATH`.** The
  `_PATH` name comes from the `signer generate` subcommand's own help and is not a substitute
  here. This document said `_PATH` for a few hours on that reasoning, and that is exactly the
  error the message above reports.

### 3. Write the manifest

```powershell
pnpm release:manifest                    # or: pnpm release:manifest notes.md
```

`pnpm release` already ran this for you; run it on its own when the build was fine and you want
to write the manifest again, or to attach release notes.

It reads the endpoint and the public key out of `tauri.conf.json` and the version out of the
root `package.json` — the same files Tauri reads, so the manifest cannot disagree with the app
that will be asked to install it — and writes `target/release/latest.json`.

It **refuses** rather than warns, and each refusal is a release that would have been broken for
everybody at once:

| it stops on | because |
|---|---|
| `pubkey` still empty | the release would offer an update nobody can install |
| `tauri.conf.json` names a version instead of `../../package.json` | Tauri would build that number while the manifest announced the other one |
| no `*-setup.exe` | there is nothing to publish; the build did not run |
| the installer's name does not carry the version | a manifest saying 0.3.0 beside a 0.2.0 installer installs the wrong build, and the app then reports itself up to date for ever, because the plugin compares against what is *running* |
| no `.sig` beside the installer | the build ran unsigned (step 2), and the signature is not optional |
| the signature carries no `version:` | `requireSignedVersion` is on, so every installation would refuse the update with `MissingSignedVersion`. The Tauri CLI writes that field **from 2.11.5**, which is why `package.json` asks for it |
| the signature's version is not this one | the bundle belongs to another release |

### 4. Publish

```powershell
pnpm release --publish            # or: pnpm release --publish --notes notes.md
```

**The notes are what an installed app shows under "Cosa cambia"**, so write them in the markdown
it reads: `#` headings, paragraphs, `-` or `1.` lists, `**bold**`, `` `code` `` and a `---` rule.
Rust reads that subset into blocks (`crates/ipc/src/release_notes.rs`) and never hands the window
markup, because `latest.json` is not signed; anything outside it — a link, a table, an image —
shows as the characters it was written with. Until 2026-09-23 the notes reached the screen as
one run of raw text, and 0.2.0's were the ones seen that way.

This is steps 2 and 3 again, followed by the tag and the upload. **Publishing is behind its own
flag on purpose**: building and signing can be repeated all day, and publishing cannot be taken
back once somebody has fetched it. `pnpm release` on its own never touches the network.

Everything that can refuse does so **before** the build, so a five-minute build and a typed
password are never spent on a publish that was impossible from the start:

| it stops on | because |
|---|---|
| `gh` not authenticated | the upload would fail at the very end |
| HEAD is not `master` | a release is a tag on `master`, and tagging elsewhere makes the landing page and the release disagree |
| commits not pushed | the tag would name a commit nobody else can fetch |
| the tag already exists | the version was not bumped — or it was bumped with a bare `pnpm version`, which tags on the spot (step 1) |

Then it prints the repository, the tag and the three files with their sizes, and asks you to
**type the version** — not to press `y`. Publishing the wrong version is the mistake worth a
prompt at all, and a habit of pressing `y` is not a confirmation. Anything else stops it, with
nothing tagged and nothing uploaded.

The repository it publishes to is read out of the updater endpoint, so a release cannot land
somewhere the app does not look.

By hand, the same thing is: tag on `master`, create the release, and upload **three** files:

- `IsaacDome_<version>_x64-setup.exe` — what the updater downloads, and what a new user installs
- `IsaacDome_<version>_x64_en-US.msi` — the alternative for a machine that wants an MSI
- `latest.json` — what the endpoint serves

The tag is `v<version>`, and the manifest already points at it: that is the one thing in the file
that depends on a step which has not happened yet, so publishing under a different tag breaks the
download link.

---

## Why NSIS and not the MSI

Both are built and both are published, and only the NSIS setup is ever downloaded by the app.
NSIS installs per-user and needs no elevation; the MSI asks for administrator rights, which an
update running quietly behind a window cannot obtain. `installMode` is `passive`, the plugin's
own default: a progress bar, no questions, and the app restarts itself afterwards.

## What happens on the user's machine

1. At launch, **only if "aggiorna automaticamente" is on**, the app fetches the endpoint. With
   the switch off nothing is requested at all — the setting is about the request, not about a
   notice.
2. If the manifest announces a newer version, the app downloads it in the background and
   verifies the signature. **Verification is not optional and cannot be turned off.**
3. The screen says "pronta da installare". Nothing is installed until the user presses the
   button.
4. The button launches the installer, and **Windows ends the app's process at that moment** —
   that is a limitation of Windows installers, not a choice. The installer restarts the app when
   it is done.

**`requireSignedVersion` needs a Tauri CLI of at least 2.11.5**, and `package.json` pins that
floor for exactly this reason. The version the signature was made for lives in minisign's trusted
comment, which the global signature covers; a CLI older than 2.11.5 writes only `timestamp:` and
`file:` there, and an app with the flag on rejects such a signature outright. Measured on
2026-09-20 on 2.11.4: the first signed build produced a signature with no version, which would
have been an update every installation refused — and nothing would have said so until it was
published. Step 3 now opens the signature and checks, so it cannot happen quietly again.

`requireSignedVersion` is on. The manifest travels over TLS but is **not itself signed**, so
without that flag anyone able to serve a crafted response could pair an inflated version number
with the URL and signature of an older release and force a downgrade to a genuine, validly
signed, outdated build. It is off by default upstream because it rejects releases signed before
the Tauri CLI began recording the version in the signature's trusted comment — **this project had
published none when it was turned on**, so it cost nothing, and it can never be turned on that
cheaply again.

## What no test covers

`scripts/check` does not run `pnpm build`, by design — it downloads WiX and NSIS and takes
minutes. So the signing configuration, the bundle names and this whole chain are exercised only
by a real release. The first one has to be watched by a person: build, publish, install the
older version on a machine, and confirm it finds, downloads, verifies and installs the newer one.
Until that has been done once, the feature is written and not proven.
