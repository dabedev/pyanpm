# pyanPM

`pyanPM` is a Windows plugin manager for Roblox Studio plugins.

The CLI is the primary way to use `pyanPM`.
The desktop app is an optional companion for browsing state, running checks, and managing the same global config with a UI.

## Release Artifacts

`pyanPM` ships three Windows 10/11 x64 release artifacts:

- `CLI`: the required command-line tool
- `Companion App`: the optional desktop companion
- `Bundled CLI + Companion`: both together in one download

## Release Automation

GitHub Actions builds Windows release artifacts.

- `CI` runs on pushes and pull requests.
- `Release` runs on version tags like `v0.1.0` and uploads:
  - CLI
  - Companion App
  - Bundled CLI + Companion

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
