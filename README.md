# pyanPM

`pyanPM` manages Roblox Studio plugins from a shared Rust core, a CLI, and a desktop app.

## Current Scope

- global `pyanpm.toml`
- generated `pyanpm.lock`
- `file:`, `path:`, and `git:` plugin sources
- installs into the Roblox Studio plugins directory
- CLI commands for `init`, `add`, `install`, `list`, `doctor`, `validate-source`, `diff`, `remove`, `reinstall`, `update`, `activity`, and `cache`
- desktop integration backed by the same Rust operations

## Git Sources

Git installs use your system `git`.

- The repository must contain `pyanpm.plugin.toml`.
- The lockfile stores the resolved commit SHA.
- Private repository access uses your existing Git credentials, SSH agent, or credential manager.

### Examples

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

Add from a subdirectory inside the repository:

```bash
pyanpm add git:https://github.com/org/plugin-repo.git --git-subdir packages/my-plugin
```

## Project Layout

- `crates/pyanpm-core` contains shared domain logic
- `crates/pyanpm-cli` contains the CLI entrypoint
- `src/`, `public/`, and `src-tauri/` in the project root contain the desktop UI and Tauri host
- `.trae/documents/pyanpm/` contains project plans, architecture notes, and related specs

## Development

- Run Rust commands from the `pyanpm` directory
- Run desktop `pnpm` commands from the `pyanpm` directory
