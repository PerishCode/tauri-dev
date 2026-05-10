# tauri-dev

Tauri development orchestration CLI for app, sidecar, socket, inspect, and diagnostics loops.

`tauri-dev` is intentionally product-agnostic. A consumer such as `stim.io` provides an explicit config file; the CLI turns that into validated development plans and, in later phases, lifecycle execution.

## Install

Release installation is R2-backed. The public URL is intentionally not hardcoded in this scaffold.

```sh
curl -fsSL "$TAURI_DEV_RELEASES_PUBLIC_URL/stable/latest/install.sh" \
  | sh -s -- install --channel stable --public-url "$TAURI_DEV_RELEASES_PUBLIC_URL"
```

Beta releases use the same installer with `--channel beta`.

## Local Smoke

```sh
cargo run --locked -p tauri-dev-cli -- doctor --config examples/minimal.toml
cargo run --locked -p tauri-dev-cli -- inspect config --config examples/minimal.toml
```

## Boundary

- `tauri-dev-core` owns config, state, diagnostics, and plan primitives.
- `tauri-dev-cli` owns the installed command surface.
- Consumers own product-specific config and scripts.

Report parser gaps, rule noise in diagnostics, install issues, and missing Tauri-dev capabilities at:

https://github.com/PerishCode/tauri-dev/issues

