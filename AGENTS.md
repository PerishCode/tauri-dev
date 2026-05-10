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

## Update / Compatibility Policy

- The CLI never carries compatibility shims. Renaming or reshaping `Manifest`, CLI flags, the inspect protocol, the stamp protocol, or the installer surface is a hard cutover — no aliases, no deprecation warnings, no best-effort parsing of older shapes.
- No internal migrations: there is no `state v1 → v2` translator, no schema-version field, no auto-rewrite of user `sidecar.toml`. Older configs that no longer parse must hard-fail with an error pointing the user at the latest README.
- The escape hatch on any breakage is fixed and must always work: `sidecar reset` (kill stamped processes) → `sidecar.sh|ps1 uninstall` → reinstall the latest release → re-author `sidecar.toml` per the latest README. This single path replaces every other compatibility guarantee.
- Versioning is `0.Y.Z` indefinitely. A `Y` bump is breaking by default; pre-1.0 SemVer carries the unstable contract for us — we do not promote to `1.0.0`.
- The update mechanism itself follows the same rule: the startup check is best-effort and silently swallows every failure mode (network, parse, clock, missing curl); `sidecar update` is a thin wrapper around the installer (`install.sh|ps1 update`) — it does not decompress, verify, or roll back.

## Build-time Stamps

`crates/cli` reads three optional build-time env vars via `option_env!` and bakes them into the binary; `.github/scripts/release/assets/package.{sh,ps1}` set all three from the release workflow:

- `SIDECAR_BUILD_VERSION` → `cli::version()` (defaults to `v<CARGO_PKG_VERSION>` for dev builds).
- `SIDECAR_BUILD_CHANNEL` → `cli::channel()` (`stable` / `beta` / `dev`; defaults to `dev`, which disables the startup check and `update` subcommand).
- `SIDECAR_BUILD_PUBLIC_URL` → fallback for the update check / subcommand when the runtime env var is absent.

The release workflows pass `RELEASE_CHANNEL` (`stable` for `release.yml`, `beta` for `release-beta.yml`) and the repo var `SIDECAR_RELEASES_PUBLIC_URL` into the build matrix steps so that every published binary is self-aware.

## Runtime Update Env Vars

- `SIDECAR_RELEASES_PUBLIC_URL` — overrides the build-time stamp for both check and update.
- `SIDECAR_CHANNEL` — overrides the build-time channel (e.g. flip a stable build to watch beta).
- `SIDECAR_NO_UPDATE_CHECK=1` — skip the startup check entirely.
- `SIDECAR_UPDATE_TTL=<n>[smhd]` — startup-check cache TTL; default `24h`, `0` = always fetch.

Cache file: `${XDG_STATE_HOME:-$HOME/.local/state}/sidecar/update-<channel>.json` on Unix, `%LOCALAPPDATA%\sidecar\update-<channel>.json` on Windows. It is single-key (`{checked_at, channel, latest_version}`) and may be deleted at any time.

## Installer Verbs

`scripts/manage/sidecar.{sh,ps1}` accept exactly: `install`, `update`, `uninstall`. There is no `upgrade` alias. The CLI's `sidecar update` subcommand downloads the canonical installer for the current channel and execs it with the `update` verb.

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
