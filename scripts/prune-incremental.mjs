// Empties `target/debug/incremental` on a cadence, because nothing else will.
//
// **There is no built-in way to do this.** Cargo's automatic garbage collection is stable since
// 1.88 and cleans `~/.cargo` only -- the registry, the git checkouts, the downloaded sources --
// and never touches `target/`. Collecting `target/` is still an open tracking issue
// (rust-lang/cargo#13136, accepted as an idea and never implemented; the older #6229 says the
// same). The obvious third-party answer does not work either: `cargo-sweep` is marked
// unmaintained by its own author, and its issue #50 records that it leaves
// `target/<profile>/incremental` exactly as it found it.
//
// **Why a cadence and not a size threshold.** Cargo's own stable design for the global cache is
// `cache.auto-clean-frequency = "1 day"` -- time, not bytes -- and the reason applies here: asking
// "how big is it?" means walking the whole tree on every launch, which is the cost we are trying
// to avoid. A cadence pays nothing on the days it does not fire.
//
// **Why prune instead of disabling incremental.** Measured on 2026-09-16, on this workspace,
// touching one central crate and re-running `cargo clippy --all-targets`: 9s with incremental
// against 15s without, and 5s against 7s on a second pair in the opposite order. It earns its
// keep on the inner loop. What it does not do is ever clean up: 16.9 GB across 76,692 files had
// accumulated by that date. The problem is the accumulation, not the feature.
//
// **Where the seven days come from.** A full run of `scripts/check` that rebuilds nothing adds
// about 10 MB; a rebuild with one crate actually touched adds around 50 MB, measured over the six
// builds of that afternoon, which left 0.31 GB behind. So a week of real work lands in the low
// single-digit GB and a month is where it starts to hurt. Override with
// `ISAACDOME_INCREMENTAL_MAX_AGE_DAYS`, and `--force` ignores the cadence entirely.
//
// Like `dev-reset.mjs`, this is **a cleaner and not a gate**: it never fails the command it
// precedes, whatever happens below.
import { existsSync, readdirSync, rmSync, statSync, utimesSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const DEFAULT_MAX_AGE_DAYS = 7
const root = dirname(dirname(fileURLToPath(import.meta.url)))
const incremental = join(root, 'target', 'debug', 'incremental')
// Inside `target/`, which is git-ignored: the stamp is invisible to git, and `cargo clean` takes
// it away along with the thing it is stamping, so the next run starts the cadence over honestly.
const stamp = join(root, 'target', '.incremental-pruned')

const maxAgeDays = Number(process.env.ISAACDOME_INCREMENTAL_MAX_AGE_DAYS ?? DEFAULT_MAX_AGE_DAYS)
const forced = process.argv.includes('--force')
const dryRun = process.argv.includes('--dry-run')
// A person typed this, rather than a hook running it before a launch. It decides whether silence
// is acceptable: as `predev` it must not add noise to every `pnpm dev`, by hand it must answer.
const explicit = forced || dryRun

/** Bytes under a directory. Only ever called on a day we are about to delete it anyway. */
const sizeOf = (dir) =>
	readdirSync(dir, { withFileTypes: true }).reduce((total, entry) => {
		const path = join(dir, entry.name)
		// A symlink is followed nowhere: its target is somebody else's bytes.
		if (entry.isDirectory() && !entry.isSymbolicLink()) return total + sizeOf(path)
		if (!entry.isFile()) return total
		return total + statSync(path).size
	}, 0)

/** Days since the stamp, or `Infinity` when there is none — a first run always prunes. */
const daysSinceStamp = () => {
	if (!existsSync(stamp)) return Infinity
	const age = Date.now() - statSync(stamp).mtimeMs
	return age / (24 * 60 * 60 * 1000)
}

/** The unit a human would have used: "0.00 GB" reads as a failure when it is really 40 kB. */
const size = (bytes) => {
	if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(2)} GB`
	if (bytes >= 1024 ** 2) return `${(bytes / 1024 ** 2).toFixed(0)} MB`
	return `${(bytes / 1024).toFixed(0)} kB`
}

const touchStamp = () => {
	writeFileSync(stamp, '')
	const now = new Date()
	utimesSync(stamp, now, now)
}

const main = () => {
	if (!Number.isFinite(maxAgeDays) || maxAgeDays <= 0) {
		console.log(`prune-incremental: ISAACDOME_INCREMENTAL_MAX_AGE_DAYS is not a number of days, skipping`)
		return
	}
	if (!existsSync(incremental)) {
		// Nothing to prune, but the cadence still starts: a fresh clone should not prune on its
		// very first build just because it has never stamped.
		if (!existsSync(stamp)) touchStamp()
		if (explicit) console.log('prune-incremental: nothing to prune, the cache is not there')
		return
	}
	const days = daysSinceStamp()
	if (!forced && days < maxAgeDays) {
		// Silent as a hook, which runs before every launch; a person who typed the command gets
		// an answer, because a command that says nothing cannot be told from one that failed.
		if (explicit) {
			console.log(
				`prune-incremental: nothing to do, last pruned ${days.toFixed(1)} days ago (threshold ${maxAgeDays}d)`,
			)
		}
		return
	}
	const bytes = sizeOf(incremental)
	const since = Number.isFinite(days) ? `${days.toFixed(1)} days` : 'never pruned'
	if (dryRun) {
		console.log(`prune-incremental: would free ${size(bytes)} (${since}, threshold ${maxAgeDays}d)`)
		return
	}
	rmSync(incremental, { recursive: true, force: true })
	touchStamp()
	console.log(`prune-incremental: freed ${size(bytes)} of incremental cache (${since})`)
}

try {
	main()
} catch (error) {
	// A cleaner that fails a launch is worse than a cache that grows.
	console.log(`prune-incremental: skipped (${error?.message ?? error})`)
}

process.exit(0)
