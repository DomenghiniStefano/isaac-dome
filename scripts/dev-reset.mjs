// Entry point for scripts/dev-reset.ps1.
//
// Node and not `sh`: pnpm runs a script through cmd.exe, where `sh` exists only if a Git Bash
// happens to be on PATH. It is on mine and it was not on the shell that types `pnpm dev`, so
// the predev died before tauri ever started -- the command this was supposed to protect.
// Node is there by construction: pnpm just ran this file with it.
//
// The work itself is Windows-only -- it reads listening sockets and process command lines --
// and everywhere else this is a no-op rather than a failed predev.
import { spawnSync } from 'node:child_process'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

if (process.platform === 'win32') {
	const script = join(dirname(fileURLToPath(import.meta.url)), 'dev-reset.ps1')
	spawnSync('powershell', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', script], {
		stdio: 'inherit',
	})
}

// Never fails the launch it precedes, whatever happened above.
process.exit(0)
