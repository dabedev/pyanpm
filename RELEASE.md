# Release

This document covers the maintainer setup and release process for GitHub Releases.

## What The Release Workflow Does

The GitHub Actions `Release` workflow:

1. Resolves the requested tag.
2. Verifies that the tag matches `Cargo.toml`.
3. Verifies that `src-tauri/tauri.conf.json` uses the same version.
4. Runs the core Rust and desktop checks.
5. Builds the CLI and the NSIS desktop installer.
6. Authenticode-signs the Windows `.exe` files in CI.
7. Packages release assets.
8. Generates `SHA256SUMS.txt` and the GitHub Release description.
9. Publishes a GitHub Release and uploads only the intended release assets automatically.

## Required GitHub Secrets

The release workflow expects these repository secrets:

- `WINDOWS_SIGNING_CERTIFICATE_BASE64`: base64-encoded `.pfx` certificate for Authenticode signing
- `WINDOWS_SIGNING_CERTIFICATE_PASSWORD`: password for the `.pfx`
- `WINDOWS_SIGNING_TIMESTAMP_URL`: optional timestamp URL. If omitted, the workflow uses `http://timestamp.digicert.com`.

If the signing secrets are missing, the release job fails instead of publishing unsigned `.exe` files.

## Standard CI

Normal CI is handled by `.github/workflows/ci.yml`.

It runs automatically on:

- pushes to `main`
- pull requests

The local command set that matches CI is:

```powershell
pnpm install --frozen-lockfile
cargo test -p pyanpm-core
cargo check --workspace
pnpm run check
pnpm run test
pnpm run build
pnpm run build:companion
```

## Publish A Release From A Tag

Use this path for the normal release flow.

1. Update `Cargo.toml`.
2. Update `src-tauri/tauri.conf.json` to the same version.
3. Commit the version change.
4. Push the branch.
5. Create and push the matching tag.

Example:

```powershell
git tag v0.1.0
git push origin main
git push origin v0.1.0
```

The GitHub Release is created automatically when the tag push reaches GitHub.

## Run The Release Workflow Manually

The workflow also supports `workflow_dispatch` in the GitHub Actions UI.

Use it when the tag already exists and you want to republish from that tag.

Rules:

- the input tag must start with `v`
- the input tag must match the version in `Cargo.toml`
- the Tauri app version must match that same version

## Published Assets

Each GitHub Release publishes these files:

- `pyanpm-cli-windows-x64-v<version>.zip`
- `pyanpm-companion-windows-x64-v<version>-setup.exe`
- `pyanpm-bundled-windows-x64-v<version>.zip`
- `SHA256SUMS.txt`

The release description shown on GitHub is generated from `RELEASE_NOTES.md` content during CI, but `RELEASE_NOTES.md` is not uploaded as a downloadable asset.

## Verify A Download On Windows

1. Download the asset and `SHA256SUMS.txt` from the same GitHub Release.
2. Run a SHA256 check in PowerShell.
3. Compare the result to the matching line in `SHA256SUMS.txt`.
4. For `.exe` files, also verify the Authenticode signature.

SHA256 example:

```powershell
Get-FileHash .\pyanpm-cli-windows-x64-v0.1.0.zip -Algorithm SHA256
```

Signature example:

```powershell
Get-AuthenticodeSignature .\pyanpm-companion-windows-x64-v0.1.0-setup.exe | Format-List Status, StatusMessage, SignerCertificate, TimeStamperCertificate
```

`Status` should be `Valid`.
