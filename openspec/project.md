# Project: pois

## Purpose

`pois` is a personal Rust multi-agent AI companion, controllable via
a local CLI and a web dashboard, self-hostable in Docker (Railway
target). Freely inspired by [nanobot](https://github.com/HKUDS/nanobot)
(HKUDS) and [OpenClaw](https://github.com/openclaw/openclaw) — not a
faithful port, no API parity contract.

Single-operator per instance: the sole operator administers their
agents through the CLI on their own machine and/or through the
dashboard protected by basic auth from a distance.

## Tech stack

- **Language**: Rust
- **Edition / MSRV**: `edition = "2024"`, `rust-version = "1.95.0"`
- **Async runtime**: `tokio` (single, explicit features, no `full`)
- **Web**: `axum` + `askama` (compile-time typed templates) + `htmx` (CDN) + `pico.css` (CDN)
- **Config**: `serde` + `toml` (TOML everywhere — global and per-agent)
- **CLI**: `clap` (derive, env)
- **Observability**: `tracing` + `tracing-subscriber` (JSON in prod via `POIS_LOG_FORMAT=json`, compact in dev)
- **Errors**: `thiserror` in runtime modules, `anyhow` tolerated in `main` / init
- **Build / distribution**: Cargo (single crate, `[[bin]]` + `[lib]`), multi-stage Dockerfile (`rust:1.95-slim` → `debian:bookworm-slim`)

## Project structure

Single crate. One unique Cargo package named `pois` with library and
binary in the same manifest. Any multi-crate workspace goes through
a dedicated OpenSpec proposal.

```
pois/
├── Cargo.toml
├── Cargo.lock              # committed
├── rust-toolchain.toml     # pinned channel 1.95.0
├── Dockerfile
├── .dockerignore
├── .gitignore
├── src/
│   ├── main.rs             # clap::parse + subcommand dispatch
│   ├── lib.rs              # public re-exports
│   ├── cli/
│   │   ├── mod.rs          # Cli / Command / run
│   │   └── gateway.rs      # `pois gateway` subcommand
│   ├── gateway/
│   │   ├── mod.rs          # axum router + serve()
│   │   ├── auth.rs         # basic auth middleware (subtle + base64)
│   │   ├── health.rs       # GET /health (public)
│   │   └── views.rs        # askama handlers
│   ├── config/
│   │   └── mod.rs          # GlobalConfig (stub — port-config)
│   ├── data/
│   │   └── mod.rs          # ensure_layout() of $POIS_DATA_DIR
│   └── errors.rs           # AppError (thiserror)
├── templates/              # askama, typed at compile time
│   ├── base.html
│   └── index.html
└── openspec/               # change proposals and promoted specs
```

## Conventions

- **Errors**: `Result<T, ModuleError>` via `thiserror` in runtime
    modules (`gateway`, `config`, `data`, future `agent`,
    `channels`, `providers`, `mcp`). `anyhow` allowed only in
    `main` and the init path.
- **unwrap / expect**: tolerated only when accompanied by a
    `// SAFETY:` or `// NOTE:` comment documenting the invariant.
    Otherwise, a typed error return is required.
- **Style / lint**: `cargo fmt --check` and
    `cargo clippy --all-targets -- -D warnings` are the reference.
    `unwrap_used = "deny"` is NOT enabled globally at this stage.
- **Tests**: TDD via the Superpowers `test-driven-development`
    skill when business logic calls for it; no smoke test is
    required while the surface stabilises.
- **tokio dependencies**: explicit features
    (`rt-multi-thread`, `macros`, `net`, `fs`, `signal`, `time`),
    never `full`. No direct or indirect dependency on `async-std`
    or `smol` should appear in `cargo tree`.

## Persistence

All persistent state lives under `$POIS_DATA_DIR` (default: `/data`).
The runtime creates the missing sub-directories at boot without
overwriting `config.toml` if it already exists.

```
$POIS_DATA_DIR/
├── config.toml             # global configuration (schema: port-config)
├── agents/
│   └── <agent-id>/
│       ├── config.toml     # local agent config
│       ├── SOUL.md         # persona / identity (port-agent-loop)
│       ├── HEARTBEAT.md    # rolling memory (port-agent-loop)
│       └── tools/          # agent-specific tools
├── honcho/                 # Honcho client cache / tokens (integrate-honcho)
└── logs/                   # runtime traces
```

The internal schema of each file (global and local `config.toml`,
`SOUL.md`, `HEARTBEAT.md`) is decided by the dedicated OpenSpec
proposals cited above.

## Deployment

- **Primary target**: Railway (PaaS). Any Docker-compatible PaaS
    (Fly, Render, …) works — the Dockerfile is standard.
- **Image**: multi-stage, runtime base `debian:bookworm-slim`,
    goal < 100 MB.
- **Port**: honours the `PORT` env var (default `8080`).
- **Volume**: `/data` declared as `VOLUME` to signal the expected
    persistent mount.
- **Dashboard authentication**: HTTP Basic Auth, credentials read
    at boot from `POIS_ADMIN_USER` / `POIS_ADMIN_PASS`. Absence or
    empty value = refusal to start with non-zero exit code.
    `/health` remains public for probes.

## Inspirations (non-contractual)

- **nanobot** (HKUDS) — <https://github.com/HKUDS/nanobot>:
    concepts of *soul*, *heartbeat*, *channels*, *provider routing*,
    MCP.
- **OpenClaw** — <https://github.com/openclaw/openclaw>: agent
    architecture, tool orchestration.

Neither is ported line by line; their SHAs / tags are not pinned.
