## MODIFIED Requirements

### Requirement: Product shape is documented

The project SHALL document, in `openspec/project.md`, its product
identity: a personal Rust multi-agent AI companion controllable via a
local CLI and a web dashboard. This documentation MUST state: the
inspirations (`nanobot` and `OpenClaw`, cited by URL, with no API
parity contract), the deployment target (binary packaged as a Docker
image, self-hosting on a PaaS such as Railway), the single-operator
nature (one operator per instance), and what is NOT covered at the
time of scoping (agent loop, channels, providers, MCP, Honcho).

#### Scenario: project.md documents the product

- **WHEN** a contributor reads `openspec/project.md`
- **THEN** they find a section identifying `pois` as a Rust
  multi-agent CLI-plus-dashboard companion, citing `nanobot` and
  `OpenClaw` as non-contractual inspirations, and specifying
  Docker/Railway as the deployment target

### Requirement: Rust toolchain is pinned

The repository SHALL provide, at its root, a `rust-toolchain.toml`
file that pins the `stable` channel to version `1.95.0` (MSRV) and
includes the `rustfmt` and `clippy` components. The `pois` crate
MUST declare `edition = "2024"` in its manifest.

#### Scenario: rust-toolchain.toml exists and activates the toolchain

- **WHEN** a contributor clones the repository with `rustup` installed
  and runs `cargo --version`
- **THEN** the `1.95.0` stable toolchain is selected automatically

#### Scenario: The crate targets the 2024 edition

- **WHEN** a contributor inspects `Cargo.toml`
- **THEN** the `package.edition` field equals `"2024"` and
  `package.rust-version` equals `"1.95.0"`

### Requirement: Single-crate layout

The repository SHALL be a single Cargo crate named `pois`, with a
library at `src/lib.rs` and a binary at `src/main.rs`. No
`[workspace]` table SHALL exist at the root until a dependency
boundary is formally identified and accepted through a dedicated
OpenSpec proposal.

#### Scenario: Cargo.toml declares a single package named pois

- **WHEN** a tool parses the root `Cargo.toml`
- **THEN** it finds exactly one `[package]` table with
  `name = "pois"`, and no `[workspace]` table

#### Scenario: Introducing a second crate requires a proposal

- **WHEN** a contributor wants to split `pois` into multiple crates
- **THEN** they open an OpenSpec proposal that amends this
  requirement via a `MODIFIED Requirements` delta

### Requirement: Async runtime is tokio-only

The `pois` crate SHALL depend on `tokio` as its sole asynchronous
runtime. Any direct or indirect transitive dependency on `async-std`
or `smol` MUST be forbidden at the crate's application-code level.
The necessary tokio features (`rt-multi-thread`, `macros`, `net`,
`fs`, `signal`, `time`) MUST be activated explicitly rather than via
`full`.

#### Scenario: tokio is declared and configured

- **WHEN** a contributor inspects `Cargo.toml`
- **THEN** they find `tokio` with an explicit set of features, without
  the `full` feature

#### Scenario: No other async runtime is introduced

- **WHEN** a contributor runs `cargo tree -i async-std` and then
  `cargo tree -i smol`
- **THEN** both commands report that the queried crate is absent from
  the dependency graph

### Requirement: Error handling convention

The crate code SHALL use `thiserror` to define the error types
exported by runtime modules (gateway, data, config, and future
agent/channels/providers/mcp). The `main` entry point and the
initialisation path MAY use `anyhow` to aggregate errors up to the
user. Calls to `unwrap()` and `expect()` MUST be justified by a
`// SAFETY:` or `// NOTE:` comment documenting the invariant; without
such a comment they MUST be replaced by a typed error return.

#### Scenario: A runtime module exposes a dedicated error type

- **WHEN** a contributor inspects the public API of a runtime module
  (for example `gateway` or `config`)
- **THEN** the public functions that may fail return
  `Result<T, ModuleError>` where `ModuleError` is an enum deriving
  `thiserror::Error`

#### Scenario: An undocumented unwrap is visible in review

- **WHEN** a contributor introduces `.unwrap()` without a
  `// SAFETY:` or `// NOTE:` comment in non-test code
- **THEN** review demands either the comment, or its replacement by
  `?` or a typed error

### Requirement: Formatting and lint policy

The repository SHALL treat `cargo fmt --check` and
`cargo clippy --all-targets -- -D warnings` as the reference commands
for style and lint. The clippy rule `unwrap_used = "deny"` MUST NOT
be enabled globally at this stage, in order to preserve prototyping
fluency; enabling it will come through a dedicated proposal once the
surface stabilises.

#### Scenario: cargo fmt --check passes on a clean repository

- **WHEN** a contributor runs `cargo fmt --check` on a branch with no
  modifications
- **THEN** the command returns exit code 0

#### Scenario: A clippy warning blocks the reference command

- **WHEN** code introduces a clippy warning
- **THEN** `cargo clippy --all-targets -- -D warnings` returns a
  non-zero exit code

### Requirement: /data persistence layout

The runtime SHALL treat a root directory `$POIS_DATA_DIR` (default:
`/data`) as the single source of truth for persistent state. This
directory MUST follow the layout below:

```
$POIS_DATA_DIR/
├── config.toml         # global configuration (TOML)
├── agents/             # one sub-directory per agent
│   └── <agent-id>/
│       ├── config.toml
│       ├── SOUL.md
│       ├── HEARTBEAT.md
│       └── tools/
├── honcho/             # Honcho client state (cache, tokens)
└── logs/               # runtime traces
```

At start-up, the binary MUST create the missing sub-directories
(`agents/`, `honcho/`, `logs/`) if they do not already exist, without
touching `config.toml` if it already exists. The detailed schema of
the `config.toml` files, `SOUL.md`, and `HEARTBEAT.md` is OUT OF
SCOPE for this capability and will be defined by dedicated proposals.

#### Scenario: The gateway creates the missing structure at boot

- **WHEN** `pois gateway` starts with `POIS_DATA_DIR` pointing to an
  empty directory
- **THEN** after boot the sub-directories `agents/`, `honcho/` and
  `logs/` exist, and `config.toml` is not created automatically

#### Scenario: POIS_DATA_DIR overrides the default

- **WHEN** the binary starts with `POIS_DATA_DIR=/tmp/pois-dev`
- **THEN** every read or write of state happens under
  `/tmp/pois-dev`, never under `/data`

### Requirement: Deployment target is Docker / Railway

The repository SHALL provide, at its root, a `Dockerfile` and a
`.dockerignore` that produce a Linux image containing the `pois`
binary. The image MUST:

- set `ENTRYPOINT` to an invocation that launches `pois gateway`;
- honour the `PORT` environment variable (default: `8080`) for the
  gateway listening port, so the image is compatible with Railway and
  similar PaaS;
- declare, via `VOLUME`, the `/data` path to signal that this
  directory is meant to be mounted on a persistent volume.

#### Scenario: docker build produces a functional image

- **WHEN** a contributor runs `docker build -t pois .` at the repository
  root
- **THEN** the image builds without error and `docker run --rm pois --help`
  displays the CLI help page

#### Scenario: PORT is honored

- **WHEN** `docker run -e PORT=3000 -e POIS_ADMIN_USER=u -e POIS_ADMIN_PASS=p pois`
  runs
- **THEN** the gateway listens on port 3000

### Requirement: Dashboard basic authentication

The gateway SHALL protect EVERY web-dashboard route with HTTP Basic
Authentication. The credentials MUST be read at start-up from the
environment variables `POIS_ADMIN_USER` and `POIS_ADMIN_PASS`. If
either variable is missing or empty, the binary MUST refuse to start
and emit an explicit error message to stderr. The `/health` route
MAY remain public so that Railway probes can reach it.

#### Scenario: The server refuses to start without credentials

- **WHEN** `pois gateway` is launched without `POIS_ADMIN_USER` or
  without `POIS_ADMIN_PASS`
- **THEN** the binary writes an error message to stderr naming the
  missing variables and exits with a non-zero status code

#### Scenario: A dashboard request without credentials is rejected

- **WHEN** an HTTP client calls the root route `/` without an
  `Authorization` header
- **THEN** the server responds with `401 Unauthorized` and a
  `WWW-Authenticate: Basic realm="pois"` header

#### Scenario: The /health route stays public

- **WHEN** an HTTP client calls `/health` without credentials
- **THEN** the server responds with `200 OK`

### Requirement: Foundation invariants evolve via OpenSpec

Any modification to an invariant listed above SHALL be raised
through an OpenSpec proposal that amends this specification via a
`MODIFIED Requirements` or `REMOVED Requirements` block. The
invariants in question are: product shape, toolchain, crate layout,
async runtime, error conventions, lint policy, `/data/` schema,
deployment target, and dashboard authentication. Silent
modifications of `openspec/project.md`, `rust-toolchain.toml`,
`Cargo.toml`, or `Dockerfile` that touch these invariants MUST NOT
be merged without an associated proposal.

#### Scenario: A PR touches an invariant file without a proposal

- **WHEN** a contributor submits a PR that modifies
  `rust-toolchain.toml`, the invariant sections of
  `openspec/project.md`, or the auth sections of the `Dockerfile`,
  without an associated OpenSpec proposal
- **THEN** review requires the proposal to be opened before merge
