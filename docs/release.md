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

### 1. Bump the version

`crates/app/tauri.conf.json`, the `version` field. **That file is the only place the version is
written**: `crates/app/Cargo.toml` keeps a version of its own and Tauri ignores it when the
config names one, so there is nothing to keep in step.

### 2. Build, signed

```powershell
pnpm release
```

That is `scripts/release-build.ps1`: it asks for the key's password, sets the two variables for
that one process, runs `pnpm build` and then step 3, and clears the password on the way out —
on a failed build and on Ctrl-C as well.

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

It reads the version, the endpoint and the public key out of `tauri.conf.json` — the same file
Tauri reads, so the manifest cannot disagree with the app that will be asked to install it — and
writes `target/release/latest.json`.

It **refuses** rather than warns, in four places, and each refusal is a release that would have
been broken for everybody at once:

| it stops on | because |
|---|---|
| `pubkey` still empty | the release would offer an update nobody can install |
| no `*-setup.exe` | there is nothing to publish; the build did not run |
| the installer's name does not carry the version | a manifest saying 0.3.0 beside a 0.2.0 installer installs the wrong build, and the app then reports itself up to date for ever, because the plugin compares against what is *running* |
| no `.sig` beside the installer | the build ran unsigned (step 2), and the signature is not optional |

### 4. Publish

Tag on `master`, create the release, and upload **three** files:

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
