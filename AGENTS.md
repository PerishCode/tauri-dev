# AGENTS

## Purpose

`tauri-dev` is the standalone Tauri development CLI boundary. It owns local development orchestration primitives for Tauri applications: app launch plans, sidecar lifecycle plans, socket/inspect endpoint modeling, diagnostics, and health-check surfaces.

This repository is not a `stim.io` module. `stim.io` and other Tauri projects consume `tauri-dev` as an installed CLI, preferably through the published `install.sh` / `install.ps1` release assets once R2 is configured.

## Core Rules

- Keep the core model Tauri-generic. Do not introduce `stim`, chat, agent, message-ledger, or product-specific runtime semantics into `tauri-dev-core`.
- Prefer explicit `--config <path>` over implicit config discovery until at least two real consumers have shaped the stable convention.
- Keep `crates/core` free of CLI output and process side effects. It should expose config, state, diagnostics, socket, and plan primitives.
- Keep `crates/cli` as the installed binary boundary named `tauri-dev`.
- Release assets are sourced from R2. Before R2 is configured, release workflows may exist but should not be dispatched as production release paths.
- Consumer validation must use installed release assets, not `cargo install --path`, once a release exists.

## Common Commands

- Format: `cargo fmt --all --check`
- Test: `cargo test --locked --workspace`
- Clippy: `cargo clippy --locked --workspace --all-targets -- -D warnings`
- CLI smoke: `cargo run --locked -p cli -- doctor --config examples/minimal.toml`

## Repository Shape

- `crates/core/`: reusable model, config loading, diagnostics, sockets, and execution planning.
- `crates/cli/`: CLI parsing, text/JSON output, and process exit behavior.
- `scripts/manage/`: public installer entrypoints that are uploaded as release assets.
- `.github/scripts/`: workflow-only release helpers.
