# Frees what a previous session left behind, so `pnpm dev` starts the code you just wrote.
#
# Two things take the launch away with no error and no hint:
#
#   - a vite left over from an interrupted `pnpm dev` or `pnpm ui:dev`. It is a grandchild
#     under two cmd.exe, so killing the session that started it orphans it instead of ending
#     it. Vite is pinned to 1420 with strictPort, because tauri.conf.json's devUrl names that
#     port and nothing else: the new one cannot step aside, it can only die.
#   - an IsaacDome still in the tray. The app survives its last window and
#     tauri-plugin-single-instance hands the launch to the process already there, which brings
#     the old window forward and exits -- you then read the previous build on screen.
#
# It never fails the build. What it cannot identify as ours it reports and leaves alone: a port
# held by something foreign is better seen in vite's own error than killed by us.

$ErrorActionPreference = 'Stop'
$port = 1420
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$did = $false

function Stop-Tree($procId, $what) {
	# The children first: vite hangs under two cmd.exe that outlive it, and a tauri dev holds
	# its app the same way. Not taskkill, which writes to stderr when a pid is already gone --
	# under 'Stop' that aborts the whole cleanup, and a pid going missing mid-run is the normal
	# case here: killing vite brings tauri dev and the app down with it.
	$alive = [bool](Get-Process -Id $procId -ErrorAction SilentlyContinue)
	foreach ($child in @(Get-CimInstance Win32_Process -Filter "ParentProcessId=$procId" -ErrorAction SilentlyContinue)) {
		Stop-Tree $child.ProcessId $null
	}
	Stop-Process -Id $procId -Force -ErrorAction SilentlyContinue
	if ($what -and $alive) { "dev-reset: killed $what (pid $procId)" }
}

function Get-Listeners($p) {
	if (Get-Command Get-NetTCPConnection -ErrorAction SilentlyContinue) {
		return @(Get-NetTCPConnection -LocalPort $p -State Listen -ErrorAction SilentlyContinue |
			ForEach-Object { $_.OwningProcess })
	}
	# Older or trimmed Windows: netstat is always there.
	return @(netstat -ano | Select-String ":$p\s" | Select-String 'LISTENING' |
		ForEach-Object { ($_ -split '\s+')[-1] })
}

try {
	foreach ($procId in (Get-Listeners $port | Sort-Object -Unique)) {
		$proc = Get-CimInstance Win32_Process -Filter "ProcessId=$procId" -ErrorAction SilentlyContinue
		if (-not $proc) { continue }
		$cmd = if ($proc.CommandLine) { $proc.CommandLine } else { '' }
		if ($cmd.ToLowerInvariant().Contains($root.ToLowerInvariant())) {
			Stop-Tree $procId "the dev server on $port"
			$did = $true
		} else {
			"dev-reset: port $port is held by $($proc.Name) (pid $procId), which is not this repo -- left alone"
			$did = $true
		}
	}

	# Two names for one app: `cargo run` builds the bin as the crate is named, while the bundle
	# takes tauri.conf.json's productName. Matching only one of them looks like it works right
	# up to the day the tray holds the other.
	$appNames = @('isaac-dome.exe', 'IsaacDome.exe')
	foreach ($proc in @(Get-CimInstance Win32_Process -ErrorAction SilentlyContinue | Where-Object { $appNames -contains $_.Name })) {
		Stop-Tree $proc.ProcessId "$($proc.Name) at $($proc.ExecutablePath)"
		$did = $true
	}

	# The socket goes with the process, but it goes a moment later, and vite starts now.
	for ($i = 0; $i -lt 30 -and (Get-Listeners $port).Count -gt 0; $i++) { Start-Sleep -Milliseconds 100 }

	if (-not $did) { "dev-reset: nothing to clean up" }
} catch {
	"dev-reset: could not check ($($_.Exception.Message)) -- continuing"
}

exit 0
