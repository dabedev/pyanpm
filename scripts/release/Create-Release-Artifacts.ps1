$ErrorActionPreference = "Stop"

param(
    [string]$RepoRoot = "",
    [string]$OutputDir = ""
)

if (-not $RepoRoot) {
    $RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
}

if (-not $OutputDir) {
    $OutputDir = Join-Path $RepoRoot ".dist\releases"
}

function Get-PyanpmVersion {
    param([string]$CargoTomlPath)

    $content = Get-Content -Path $CargoTomlPath -Raw
    $match = [regex]::Match($content, '(?ms)^\[workspace\.package\].*?^version\s*=\s*"([^"]+)"')
    if (-not $match.Success) {
        throw "Could not find workspace.package version in $CargoTomlPath"
    }

    return $match.Groups[1].Value
}

function Resolve-CompanionInstaller {
    param([string]$RepoRootPath)

    $candidateRoots = @(
        (Join-Path $RepoRootPath "target\release\bundle\nsis"),
        (Join-Path $RepoRootPath "src-tauri\target\release\bundle\nsis")
    )

    foreach ($candidateRoot in $candidateRoots) {
        if (-not (Test-Path $candidateRoot)) {
            continue
        }

        $installer = Get-ChildItem -Path $candidateRoot -Filter *.exe -File -Recurse |
            Sort-Object LastWriteTimeUtc -Descending |
            Select-Object -First 1

        if ($installer) {
            return $installer
        }
    }

    throw "Could not find a built companion installer under the Tauri bundle output."
}

$version = Get-PyanpmVersion -CargoTomlPath (Join-Path $RepoRoot "Cargo.toml")
$cliExe = Join-Path $RepoRoot "target\release\pyanpm.exe"

if (-not (Test-Path $cliExe)) {
    throw "Missing CLI binary at $cliExe. Run the CLI release build first."
}

$companionInstaller = Resolve-CompanionInstaller -RepoRootPath $RepoRoot

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

$tempRoot = Join-Path $OutputDir "_staging"
if (Test-Path $tempRoot) {
    Remove-Item -Path $tempRoot -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $tempRoot | Out-Null

$readmePath = Join-Path $RepoRoot "README.md"
$releaseNotesPath = Join-Path $tempRoot "RELEASES.txt"
@"
pyanPM release artifacts

- CLI: mandatory command-line tool for daily use
- Companion App: optional desktop companion for the CLI
- Bundled CLI + Companion: includes both artifacts together
"@ | Set-Content -Path $releaseNotesPath

$cliStage = Join-Path $tempRoot "cli"
New-Item -ItemType Directory -Force -Path $cliStage | Out-Null
Copy-Item -Path $cliExe -Destination (Join-Path $cliStage "pyanpm.exe") -Force
Copy-Item -Path $readmePath -Destination (Join-Path $cliStage "README.md") -Force
$cliArchive = Join-Path $OutputDir "pyanpm-cli-windows-x64-v$version.zip"
if (Test-Path $cliArchive) {
    Remove-Item -Path $cliArchive -Force
}
Compress-Archive -Path (Join-Path $cliStage "*") -DestinationPath $cliArchive

$companionArtifact = Join-Path $OutputDir "pyanpm-companion-windows-x64-v$version-setup$($companionInstaller.Extension)"
Copy-Item -Path $companionInstaller.FullName -Destination $companionArtifact -Force

$bundleStage = Join-Path $tempRoot "bundle"
New-Item -ItemType Directory -Force -Path $bundleStage | Out-Null
Copy-Item -Path $cliExe -Destination (Join-Path $bundleStage "pyanpm.exe") -Force
Copy-Item -Path $companionArtifact -Destination (Join-Path $bundleStage (Split-Path $companionArtifact -Leaf)) -Force
Copy-Item -Path $readmePath -Destination (Join-Path $bundleStage "README.md") -Force
Copy-Item -Path $releaseNotesPath -Destination (Join-Path $bundleStage "RELEASES.txt") -Force
$bundleArchive = Join-Path $OutputDir "pyanpm-bundled-windows-x64-v$version.zip"
if (Test-Path $bundleArchive) {
    Remove-Item -Path $bundleArchive -Force
}
Compress-Archive -Path (Join-Path $bundleStage "*") -DestinationPath $bundleArchive

Remove-Item -Path $tempRoot -Recurse -Force

Write-Host "CLI artifact: $cliArchive"
Write-Host "Companion artifact: $companionArtifact"
Write-Host "Bundled artifact: $bundleArchive"
