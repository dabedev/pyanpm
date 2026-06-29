# pyanPM

`pyanPM` is a Windows plugin manager for Roblox Studio plugins.

The CLI is the primary way to use `pyanPM`.
The desktop app is an optional companion for browsing state, running checks, and managing the same global config with a UI.

## Release Artifacts

`pyanPM` ships three Windows 10/11 x64 release artifacts:

- `CLI`: the required command-line tool
- `Companion App`: the optional desktop companion
- `Bundled CLI + Companion`: both together in one download

Every GitHub Release also includes:

- `SHA256SUMS.txt` for download verification
- `RELEASE-METADATA.txt` with the exact commit SHA
- `RELEASE_NOTES.md` with the release summary and Windows verification steps

The workflow signs the Windows `.exe` files before publishing them.

## CI

GitHub Actions runs `CI` automatically on every push to `main` and on every pull request.

The CI workflow runs the same main checks you should run locally before pushing:

```powershell
pnpm install --frozen-lockfile
cargo test -p pyanpm-core
cargo check --workspace
pnpm run check
pnpm run test
pnpm run build
pnpm run build:companion
```

## Publish A Release

The `Release` workflow publishes a GitHub Release automatically when you push a version tag like `v0.1.0`.

1. Update the version in `Cargo.toml`.
2. Update the version in `src-tauri/tauri.conf.json` to match.
3. Commit the version change.
4. Create and push the tag:

```powershell
git tag v0.1.0
git push origin main
git push origin v0.1.0
```

You can also run the workflow manually from the GitHub Actions page with `workflow_dispatch`, but the tag still has to exist and match the versions in the repo.

More maintainer setup details are in `RELEASE.md`.

## Verify A Download On Windows

Download the asset you want and `SHA256SUMS.txt` from the same GitHub Release.

Check the SHA256 hash in PowerShell:

```powershell
Get-FileHash .\pyanpm-cli-windows-x64-v0.1.0.zip -Algorithm SHA256
```

Compare the reported hash with the matching line in `SHA256SUMS.txt`.

Check the signature on an `.exe` asset in PowerShell:

```powershell
Get-AuthenticodeSignature .\pyanpm-companion-windows-x64-v0.1.0-setup.exe | Format-List Status, StatusMessage, SignerCertificate, TimeStamperCertificate
```

`Status` should be `Valid`.

## What It Manages

- global `pyanpm.toml`
- generated `pyanpm.lock`
- Roblox Studio plugin installs into the current user's Plugins folder
- plugin sources from `file:`, `path:`, and `git:`

## Quick Start

1. Download the `CLI` release, or the `Bundled CLI + Companion` release.
2. Put `pyanpm.exe` somewhere on your `PATH`.
3. Run:

```bash
pyanpm init
```

4. Add one or more plugins.
5. Run:

```bash
pyanpm install
```

## Common Commands

Initialize the global config:

```bash
pyanpm init
```

Add a local plugin model:

```bash
pyanpm add file:C:\Plugins\my-plugin.rbxm
```

Add a local plugin package folder:

```bash
pyanpm add path:C:\Plugins\my-plugin
```

Install all managed plugins:

```bash
pyanpm install
```

List managed plugins:

```bash
pyanpm list
```

Run health checks:

```bash
pyanpm doctor
```

Preview updates:

```bash
pyanpm update my-plugin --dry-run
```

Apply updates:

```bash
pyanpm update --all
```

## Git Sources

Git installs use your system `git`.

- The repository must contain `pyanpm.plugin.toml`.
- The lockfile stores the resolved commit SHA.
- Private repository access uses your existing Git credentials, SSH agent, or credential manager.

Add from the default branch:

```bash
pyanpm add git:https://github.com/org/plugin-repo.git
```

Add from a branch:

```bash
pyanpm add git:https://github.com/org/plugin-repo.git --git-ref-kind branch --git-ref main
```

Add from a tag:

```bash
pyanpm add git:https://github.com/org/plugin-repo.git --git-ref-kind tag --git-ref v1.2.0
```

Add from a commit:

```bash
pyanpm add git:https://github.com/org/plugin-repo.git --git-ref-kind commit --git-ref 0123abcd
```

Add from a subdirectory inside a repository:

```bash
pyanpm add git:https://github.com/org/plugin-repo.git --git-subdir packages/my-plugin
```

## Companion App

The Companion App is optional.

Use it when you want a UI for:

- managed plugin list
- doctor results
- diff state
- cache inspection
- activity history
- settings for the global config path and Roblox Plugins folder

The Companion App uses the same shared core and the same global config as the CLI.

## Build From Source

Run Rust commands from the `pyanpm` directory:

```bash
cargo check --workspace
cargo test -p pyanpm-core
```

Install desktop dependencies and build the companion app:

```bash
pnpm install
pnpm run build
pnpm run build:companion
```

Build the CLI release binary:

```bash
pnpm run build:cli
```
