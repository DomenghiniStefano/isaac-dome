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
#   pnpm release                      build and sign; touches nothing outside this machine
#   pnpm release --publish            and then tag and publish, after a confirmation
#   pnpm release --notes notes.md     release notes, for the manifest and the GitHub release
#
# **Publishing is opt-in on purpose.** Building and signing can be repeated all day; publishing
# cannot be taken back once somebody has fetched it. Keeping the irreversible half behind its own
# flag is what stops a build you were only trying out from becoming a release — which is the
# mistake this project came within one command of making on 2026-09-20.

param(
    [switch]$Publish,
    [string]$Notes
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$key = Join-Path $HOME '.tauri\isaacdome.key'

if (-not (Test-Path $key)) {
    Write-Host "No signing key at $key." -ForegroundColor Red
    Write-Host 'Generate one with: pnpm tauri signer generate -w $HOME\.tauri\isaacdome.key'
    Write-Host 'See docs/release.md.'
    exit 1
}
if ($Notes -and -not (Test-Path $Notes)) {
    Write-Host "No release notes at $Notes." -ForegroundColor Red
    exit 1
}

$version = (Get-Content (Join-Path $root 'crates\app\tauri.conf.json') -Raw | ConvertFrom-Json).version
$tag = "v$version"

# **Everything that can refuse, refuses before the build starts.** A publish that turns out to be
# impossible after a five-minute build and a typed password is a publish that gets forced through
# by hand the second time, which is how the guards stop being read at all.
if ($Publish) {
    gh auth status *> $null
    if ($LASTEXITCODE -ne 0) {
        Write-Host 'gh is not authenticated. Run `gh auth login` first.' -ForegroundColor Red
        exit 1
    }
    $branch = (git -C $root rev-parse --abbrev-ref HEAD).Trim()
    if ($branch -ne 'master') {
        # `CLAUDE.md`: a release is a tag on `master`, which is the public face and is kept level
        # with `develop`. Tagging anywhere else makes the landing page and the release disagree.
        Write-Host "A release is tagged on master, and HEAD is on $branch." -ForegroundColor Red
        Write-Host 'Bring master level with develop first. See docs/release.md.'
        exit 1
    }
    if ([int](git -C $root rev-list --count HEAD --not --remotes) -ne 0) {
        # The tag would name a commit nobody else can fetch, so the release would point at source
        # that does not exist for anyone but this machine.
        Write-Host 'HEAD has commits that are not pushed. Push first.' -ForegroundColor Red
        exit 1
    }
    if (git -C $root tag -l $tag) {
        Write-Host "$tag already exists. Bump the version in crates/app/tauri.conf.json." -ForegroundColor Red
        exit 1
    }
}

Write-Host "Building IsaacDome $version, signed." -ForegroundColor Cyan
Write-Host 'The version comes from crates/app/tauri.conf.json. Bump it there before a release.'

# `-AsSecureString` so the password is never echoed and never lands in the console history.
$secure = Read-Host -AsSecureString 'Password of the signing key'
$plain = [System.Net.NetworkCredential]::new('', $secure).Password
if (-not $plain) {
    Write-Host 'No password given. Nothing was built.' -ForegroundColor Red
    exit 1
}

# **`TAURI_SIGNING_PRIVATE_KEY`, and the key's contents.** Measured on 2026-09-20: setting
# `TAURI_SIGNING_PRIVATE_KEY_PATH` alone builds both installers and then fails with "A public key
# has been found, but no private key. Make sure to set `TAURI_SIGNING_PRIVATE_KEY`". That
# variable is the one the bundler reads; `_PATH` is named in the `signer` subcommand's help and
# is not a substitute for it. The contents rather than the path because that is what the
# variable is named for -- the documentation says it takes either, and there is nothing to gain
# from taking the ambiguous half of that sentence twice in one day.
$env:TAURI_SIGNING_PRIVATE_KEY = (Get-Content $key -Raw).Trim()
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $plain

try {
    Set-Location $root
    pnpm build
    if ($LASTEXITCODE -ne 0) {
        Write-Host 'The build failed. Nothing to publish.' -ForegroundColor Red
        exit $LASTEXITCODE
    }
    # The manifest step is its own gate and refuses on anything that does not add up -- a
    # missing signature above all, which is what a build run without these variables produces,
    # and a signature that does not say which version it was made for.
    if ($Notes) {
        node (Join-Path $root 'scripts\release-manifest.mjs') $Notes
    }
    else {
        node (Join-Path $root 'scripts\release-manifest.mjs')
    }
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
finally {
    # **Always**, including on a failed build and on Ctrl-C: neither the password nor the key
    # itself outlives the command that needed them.
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $null
    $env:TAURI_SIGNING_PRIVATE_KEY = $null
    $plain = $null
    $secure = $null
    [System.GC]::Collect()
}

if (-not $Publish) {
    Write-Host ''
    Write-Host "Built and signed. Nothing has left this machine." -ForegroundColor Green
    Write-Host "To publish: pnpm release --publish (it builds again, then tags and uploads)."
    exit 0
}

$setup = Get-ChildItem (Join-Path $root 'target\release\bundle\nsis') -Filter '*-setup.exe' | Select-Object -First 1
$msi = Get-ChildItem (Join-Path $root 'target\release\bundle\msi') -Filter '*.msi' | Select-Object -First 1
$manifest = Join-Path $root 'target\release\latest.json'
$assets = @($setup.FullName, $msi.FullName, $manifest)

# The repository comes out of the updater endpoint, which is the same string the app will fetch
# from -- so the release cannot be published somewhere the app does not look.
$endpoint = (Get-Content (Join-Path $root 'crates\app\tauri.conf.json') -Raw | ConvertFrom-Json).plugins.updater.endpoints[0]
if ($endpoint -notmatch '^https://github\.com/([^/]+)/([^/]+)/releases/') {
    Write-Host "The updater endpoint is not a GitHub releases URL: $endpoint" -ForegroundColor Red
    exit 1
}
$repo = "$($Matches[1])/$($Matches[2])"

Write-Host ''
Write-Host 'About to publish, and this is the part that cannot be undone.' -ForegroundColor Yellow
Write-Host "  repository  $repo"
Write-Host "  tag         $tag"
foreach ($a in $assets) {
    Write-Host ("  asset       {0}  ({1:N0} bytes)" -f (Split-Path $a -Leaf), (Get-Item $a).Length)
}
Write-Host ''
# **Typing the version, not "y".** The confirmation has to prove the version was read, because
# publishing the wrong one is the mistake worth a prompt at all -- and a habit of pressing y is
# not a confirmation.
$typed = Read-Host "Type the version to publish it, anything else to stop [$version]"
if ($typed -ne $version) {
    Write-Host 'Stopped. Nothing was tagged and nothing was uploaded.' -ForegroundColor Green
    exit 0
}

git -C $root tag -a $tag -m "IsaacDome $version"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git -C $root push origin $tag
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$ghArgs = @('release', 'create', $tag, '--repo', $repo, '--title', "IsaacDome $version")
if ($Notes) { $ghArgs += @('--notes-file', $Notes) } else { $ghArgs += '--generate-notes' }
gh @ghArgs @assets
if ($LASTEXITCODE -ne 0) {
    # The tag is already out. Saying so is the difference between a retry and a second tag.
    Write-Host "The release was not created, and $tag is already pushed." -ForegroundColor Red
    Write-Host "Fix and run: gh release create $tag --repo $repo ..." -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host ''
Write-Host "Published $tag to $repo." -ForegroundColor Green
Write-Host 'Check the endpoint answers: ' -NoNewline
Write-Host $endpoint
