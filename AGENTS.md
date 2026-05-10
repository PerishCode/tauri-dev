# AGENTS

## Purpose

`sidecar` is the standalone home for an IPC-based sidecars project manager. It owns two product-neutral abstractions:

1. **Stamp args** — `--sidecar-stamp-{app,namespace,mode,source}` flags appended to every spawned sidecar so the CLI can discover, status-check, and stop them cross-platform.
2. **Inspect bridge** — a single-shot line-JSON request/response over a Unix socket (TCP fallback) for talking to a running sidecar's inspect server.

This repository is not a `stim.io` module. `stim.io` and other consumers install `sidecar` as a published CLI through the R2-backed `install.sh` / `install.ps1` entrypoints.

## Core Rules

- Keep `crates/core` product-neutral. No `stim`, `tauri`, chat, agent, or message-ledger semantics may leak in.
- Keep `crates/core` free of CLI output and process side effects. It exposes config (`Manifest`), diagnostics, plan, socket parser, stamp protocol, process discovery, and inspect client.
- Keep `crates/cli` as the installed binary boundary named `sidecar`.
- Use explicit `--config <path>`. No default config filename is reserved.
- Release assets are R2-backed. `SIDECAR_RELEASES_*` repo vars/secrets must be present before any release workflow can run.
- Consumer validation must use installed release assets, not `cargo install --path`, once a release exists.

## Common Commands

- Format: `cargo fmt --all --check`
- Test: `cargo test --locked --workspace`
- Clippy: `cargo clippy --locked --workspace --all-targets -- -D warnings`
- CLI smoke: `cargo run --locked -p cli -- doctor --config examples/minimal.toml`
- Plan: `cargo run --locked -p cli -- plan --config examples/minimal.toml --format json`

## Repository Shape

- `crates/core/`: `Manifest` config, diagnostics, plan, socket parser, stamp protocol, process discovery, inspect client.
- `crates/cli/`: CLI parsing, lifecycle execution (`start`/`stop`/`restart`/`status`/`list`/`reset`), `inspect <sidecar> <event> [payload]`, output formatting, exit behavior.
- `scripts/manage/`: public installer entrypoints uploaded as release assets.
- `.github/scripts/`: workflow-only release helpers.

## Stamp args protocol

Canonical flag names (consumers must accept and ignore them on their sidecar binaries):

```
--sidecar-stamp-app=<sidecar.name>
--sidecar-stamp-namespace=<project.namespace>
--sidecar-stamp-mode=<sidecar.mode>           # e.g. dev / runtime
--sidecar-stamp-source=tool:sidecar
```

Discovery uses only these flags via `ps -axo pid=,command=` on Unix; the implementation is in `crates/core/src/process.rs`.

## Inspect bridge

Wire format (one line per direction):

```
request:  {"event":"...","payload":<json>}\n
response: {"ok":true,"data":<json>}\n
       or {"ok":false,"error":"..."}\n
```

Default transport is Unix (`unix:///absolute/path.sock`). TCP is reserved for non-Unix fallback only.

The implementation is `crates/core/src/inspect.rs`. The CLI orchestration is `commands::inspect` in `crates/cli/src/commands.rs`.
