# The signed release build, without ever writing the password down.
#
# **This exists so nobody is tempted to set the two signing variables permanently.** A password
# in the user's environment lives in the registry in plain text, readable by anything running as
# that user -- which gives back most of what choosing a password was meant to buy: the key file
# alone stops being useless to whoever grabs it. Here the password is asked for, held in one
# process's environment for the length of one build, and cleared on the way out, whether the
# build succeeded or not.
#
# It contains no secret and is committed on purpose. The private key it points at is not in this
# repository and never will be -- see `.gitignore` and `docs/release.md`.
#
# Run it with `pnpm release`.

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$key = Join-Path $HOME '.tauri\isaacdome.key'

if (-not (Test-Path $key)) {
    Write-Host "No signing key at $key." -ForegroundColor Red
    Write-Host 'Generate one with: pnpm tauri signer generate -w $HOME\.tauri\isaacdome.key'
    Write-Host 'See docs/release.md.'
    exit 1
}

$version = (Get-Content (Join-Path $root 'crates\app\tauri.conf.json') -Raw | ConvertFrom-Json).version
Write-Host "Building IsaacDome $version, signed." -ForegroundColor Cyan
Write-Host 'The version comes from crates/app/tauri.conf.json. Bump it there before a release.'

# `-AsSecureString` so the password is never echoed and never lands in the console history.
$secure = Read-Host -AsSecureString 'Password of the signing key'
$plain = [System.Net.NetworkCredential]::new('', $secure).Password
if (-not $plain) {
    Write-Host 'No password given. Nothing was built.' -ForegroundColor Red
    exit 1
}

$env:TAURI_SIGNING_PRIVATE_KEY_PATH = $key
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $plain

try {
    Set-Location $root
    pnpm build
    if ($LASTEXITCODE -ne 0) {
        Write-Host 'The build failed. Nothing to publish.' -ForegroundColor Red
        exit $LASTEXITCODE
    }
    # The manifest step is its own gate and refuses on anything that does not add up -- a
    # missing signature above all, which is what a build run without these variables produces.
    node (Join-Path $root 'scripts\release-manifest.mjs')
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
finally {
    # **Always**, including on a failed build and on Ctrl-C: the password does not outlive the
    # command that needed it.
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $null
    $plain = $null
    $secure = $null
    [System.GC]::Collect()
}
