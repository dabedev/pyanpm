# AGENTS.md

## Working Style

- Be direct, pragmatic, and implementation-oriented.
- Read the existing project before changing it. Match the local structure and conventions instead of inventing a parallel style.
- Keep changes scoped to the current task. Do not refactor unrelated code or reorganize files unless the task requires it.
- Do not make unstated assumptions. If a requirement is unclear and cannot be verified from this project or supplied documents, ask before encoding it as guidance.
- Surface blockers and tradeoffs plainly.

## Documentation Rules

- Store project documents under `.trae/documents/pyanpm/`.
- Keep user-facing project documents such as `README.md`, `RELEASE.md`, and `TODO.md` in the project root when they describe the overall `pyanpm` project.
- Keep specifications, plans, and architecture notes as Markdown files.
- When an attached or pasted spec has encoding damage, preserve the intended content and rewrite broken diagrams or formatting cleanly.

## User-Facing Text

- Keep user-facing UI text and strings concise and directly useful to the end user.
- Do not include development notes, architecture commentary, implementation details, framework names, or internal tool references in user-facing copy unless they are necessary for the user to complete a task.
- Prefer labels, actions, statuses, errors, and short helper text that explain what the user can do or what happened.
- Remove filler, redundant explanations, and non-actionable notes from user-facing surfaces.

## Project Context

- `pyanpm` is a Rust-first Roblox plugin manager with its desktop UI in the project root and shared domain logic in `crates/pyanpm-core`.
- Keep the Rust core reusable across CLI and desktop entrypoints.
- Keep the desktop shell thin and route business logic through shared Rust operations instead of duplicating behavior in the frontend.

## Code Conventions

- Prefer strong typing in both Rust and TypeScript.
- Keep the CLI, core library, and desktop shell clearly separated.
- Avoid `any` unless there is no practical typed alternative.
- Keep user-facing text concise, plain, and consistent across the desktop UI, notifications, settings, and docs.
- For frontend work, match the existing React and Vite structure in the `pyanpm` project root.
- For backend work, preserve clear command boundaries in the Tauri layer and keep domain logic in `pyanpm-core` where practical.

## Workflow

- Use `cargo` from the `pyanpm` project root for Rust work.
- Use `pnpm` from the `pyanpm` project root for desktop frontend work.
- Use fast search tools like `rg` where available.
- Run the most relevant verification after changes, such as `cargo test`, `cargo check`, `pnpm build`, `pnpm test`, or focused checks.
- Do not overwrite user changes. If the worktree is dirty, work around unrelated edits and only touch files needed for the task.
- Do not use destructive git commands unless explicitly requested.
