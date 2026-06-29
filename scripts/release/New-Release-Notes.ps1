param(
    [Parameter(Mandatory = $true)][string]$Version,
    [Parameter(Mandatory = $true)][string]$Tag,
    [Parameter(Mandatory = $true)][string]$CommitSha,
    [string]$ArtifactsDir,
    [string]$OutputPath
)

$ErrorActionPreference = "Stop"

if (-not $ArtifactsDir) {
    $ArtifactsDir = Join-Path (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path ".dist\releases"
}

if (-not $OutputPath) {
    $OutputPath = Join-Path $ArtifactsDir "RELEASE_NOTES.md"
}

$assetLines = @()
if (Test-Path $ArtifactsDir) {
    $assetLines = Get-ChildItem -Path $ArtifactsDir -File |
        Where-Object { $_.Name -ne (Split-Path $OutputPath -Leaf) } |
        Sort-Object Name |
        ForEach-Object { "- ``{0}``" -f $_.Name }
}

$bodyLines = @(
    "## Release",
    "",
    "- Version: ``$Version``",
    "- Tag: ``$Tag``",
    "- Commit: ``$CommitSha``",
    "",
    "## Assets",
    ""
)

if ($assetLines.Count -gt 0) {
    $bodyLines += $assetLines
} else {
    $bodyLines += "- No assets were found in ``$ArtifactsDir``."
}

$bodyLines += @(
    "",
    "## Verify On Windows",
    "",
    "1. Download the asset you want and ``SHA256SUMS.txt`` from this release.",
    "2. Run ``Get-FileHash .\\pyanpm-cli-windows-x64-v$Version.zip -Algorithm SHA256`` in PowerShell.",
    "3. Compare the reported hash with the matching line in ``SHA256SUMS.txt``.",
    "4. For ``.exe`` files, run ``Get-AuthenticodeSignature .\\filename.exe | Format-List Status, StatusMessage, SignerCertificate, TimeStamperCertificate``.",
    "",
    "## Notes",
    "",
    "- The ``.exe`` files in this release are Authenticode-signed in CI.",
    "- ``SHA256SUMS.txt`` lets you verify the downloaded bytes exactly."
)

Set-Content -Path $OutputPath -Value $bodyLines
