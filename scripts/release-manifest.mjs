// Writes the `latest.json` that the updater reads, from the build that just came out.
//
// **This is a gate, not a cleaner.** Unlike `dev-reset.mjs` and `prune-incremental.mjs`, every
// check below refuses rather than warns, because what it guards is the one file that cannot be
// wrong: an updater manifest naming an asset the release does not carry, or carrying a signature
// from another build, fails for **every installation at once**, and the people it fails are the
// ones who cannot fix it. There is no CI to catch this later; there is this script and the
// person reading its output.
//
// It invents nothing. The version, the endpoint and the public key all come out of
// `crates/app/tauri.conf.json`, which is where Tauri itself reads them, so the manifest cannot
// disagree with the app that will be asked to install it. The owner and the repository are read
// out of the endpoint URL for the same reason -- a second place to write "DomenghiniStefano" is
// a second place for it to be wrong.
//
// Usage: `pnpm release:manifest [path-to-release-notes.md]`
import { existsSync, readFileSync, readdirSync, writeFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const configPath = join(root, 'crates/app/tauri.conf.json')
const nsisDir = join(root, 'target/release/bundle/nsis')
const outPath = join(root, 'target/release/latest.json')

// The platform key the plugin looks for. One, because the build produces one: a key for a
// platform nobody builds is a promise the release does not keep.
const PLATFORM = 'windows-x86_64'

const die = (message) => {
  console.error(`release-manifest: ${message}`)
  process.exit(1)
}

const config = JSON.parse(readFileSync(configPath, 'utf8'))
const version = config.version
const updater = config.plugins?.updater ?? {}

if (!version) die(`no "version" in ${configPath}`)

// **The placeholder guard.** `pubkey` ships empty so that a release build still starts -- an
// empty key fails closed, every download ending in `rejected` rather than in an unverified
// install -- but a release published with it would offer an update nobody can install. This is
// the line that stops that, and it is why the empty default is safe to keep in the repository.
if (!updater.pubkey) {
  die(
    'the updater has no public key yet. Generate one with `pnpm tauri signer generate -w ' +
      '$HOME/.tauri/isaacdome.key` and put the public half in crates/app/tauri.conf.json. ' +
      'See docs/release.md.',
  )
}

const endpoint = updater.endpoints?.[0]
if (!endpoint) die('the updater has no endpoint configured')

// `https://github.com/<owner>/<repo>/releases/latest/download/latest.json`
const owner = endpoint.match(
  /^https:\/\/github\.com\/([^/]+)\/([^/]+)\/releases\//,
)
if (!owner) die(`the endpoint is not a GitHub releases URL: ${endpoint}`)
const [, user, repo] = owner

if (!existsSync(nsisDir)) {
  die(`no NSIS bundle at ${nsisDir}. Run \`pnpm build\` first.`)
}

const files = readdirSync(nsisDir)
const setup = files.find((f) => f.endsWith('-setup.exe'))
if (!setup) die(`no *-setup.exe in ${nsisDir}`)

// **The version has to be in the file name.** A manifest saying 0.3.0 beside an installer built
// from 0.2.0 is the exact shape of a release that installs the wrong thing and then reports
// itself as up to date for ever after, because the plugin compares against what is running.
if (!setup.includes(version)) {
  die(
    `the installer is "${setup}" and the configured version is ${version}. ` +
      'Bump the version and build again, or delete the stale bundle.',
  )
}

const sigPath = join(nsisDir, `${setup}.sig`)
if (!existsSync(sigPath)) {
  die(
    `no signature beside the installer (${setup}.sig). The build ran without ` +
      'TAURI_SIGNING_PRIVATE_KEY, so this bundle cannot be published. See docs/release.md.',
  )
}

// **The signature has to carry the version it was signed for**, because `requireSignedVersion`
// is on in `tauri.conf.json` and an app with that flag rejects a signature without one —
// `MissingSignedVersion`, which reaches the user as `rejected`. The Tauri CLI writes the field
// from 2.11.5 onward, and this repository was one version short of that on 2026-09-20: the
// first signed build came out with `timestamp:` and `file:` and nothing else, which would have
// been an update every installation refused, discovered only after publishing.
//
// A minisign signature file is base64 over four lines; the third is the trusted comment, which
// the global signature covers, written as tab separated `key:value` pairs.
const signature = readFileSync(sigPath, 'utf8').trim()
const trusted = Buffer.from(signature, 'base64')
  .toString('utf8')
  .split('\n')
  .find((l) => l.startsWith('trusted comment:'))

if (!trusted) die('the signature has no trusted comment; it is not a minisign signature')

const signedVersion = trusted
  .split('\t')
  .map((f) => f.trim())
  .find((f) => f.startsWith('version:'))
  ?.slice('version:'.length)

if (!signedVersion) {
  die(
    'the signature does not say which version it was signed for, and `requireSignedVersion` ' +
      'is on — every installation would refuse this update. The Tauri CLI writes that field ' +
      'from 2.11.5; check `pnpm tauri --version` and build again. See docs/release.md.',
  )
}
if (signedVersion !== version) {
  die(
    `the signature was made for ${signedVersion} and the configured version is ${version}. ` +
      'Build again; this bundle belongs to another release.',
  )
}

const notesPath = process.argv[2]
if (notesPath && !existsSync(notesPath)) die(`no release notes at ${notesPath}`)

const manifest = {
  version,
  // Left out entirely rather than written empty: the screen shows a notes section only when
  // there is something in it, and an empty string is not nothing.
  ...(notesPath ? { notes: readFileSync(notesPath, 'utf8').trim() } : {}),
  pub_date: new Date().toISOString(),
  platforms: {
    [PLATFORM]: {
      signature,
      // The tag this release is about to be published under. `v` + the version, which is the
      // shape `docs/release.md` asks for and the only part of this file that depends on a step
      // that has not happened yet.
      url: `https://github.com/${user}/${repo}/releases/download/v${version}/${setup}`,
    },
  },
}

writeFileSync(outPath, `${JSON.stringify(manifest, null, 2)}\n`)
console.log(`release-manifest: wrote ${outPath} for ${version}`)
console.log(`  installer  ${setup}`)
console.log(`  tag        v${version}`)
console.log('  upload the installer, the MSI and this file to that release.')
