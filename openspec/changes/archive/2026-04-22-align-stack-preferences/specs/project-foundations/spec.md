## RENAMED Requirements

- FROM: `### Requirement: Deployment target is Docker / Railway`
- TO: `### Requirement: Deployment target is an OCI container image`

## MODIFIED Requirements

### Requirement: Product shape is documented

The project SHALL document, in `openspec/project.md`, its product
identity: a personal Rust multi-agent AI companion controllable via a
local CLI and a web dashboard. This documentation MUST state: the
inspirations (`nanobot` and `OpenClaw`, cited by URL, with no API
parity contract), the deployment target (binary packaged as an OCI
container image, built with `podman` or a compatible tool, self-hosted
on a PaaS such as Railway), the single-operator nature (one operator
per instance), and what is NOT covered at the time of scoping (agent
loop, channels, providers, MCP, Honcho).

#### Scenario: project.md documents the product

- **WHEN** a contributor reads `openspec/project.md`
- **THEN** they find a section identifying `pois` as a Rust
  multi-agent CLI-plus-dashboard companion, citing `nanobot` and
  `OpenClaw` as non-contractual inspirations, and specifying an OCI
  container image on Railway as the deployment target

### Requirement: Deployment target is an OCI container image

The repository SHALL provide, at its root, a `Dockerfile` and a
`.dockerignore` that produce a Linux OCI image containing the `pois`
binary. The file is named `Dockerfile` (NOT `Containerfile`) so both
`podman` and `docker` read it by default. The image MUST:

- set `ENTRYPOINT` to an invocation that launches `pois gateway`;
- honour the `PORT` environment variable (default: `8080`) for the
  gateway listening port, so the image is compatible with Railway and
  similar PaaS;
- declare, via `VOLUME`, the `/data` path to signal that this
  directory is meant to be mounted on a persistent volume;
- set `ENV POIS_LOG_FORMAT=json` so deployed containers emit
  structured logs by default.

The primary local tooling is `podman`; `docker` (Docker Engine or
Docker Desktop) and `buildah` are accepted as compatible alternatives
that consume the same `Dockerfile` without modification.

#### Scenario: podman build produces a functional image

- **WHEN** a contributor runs `podman build -t pois .` at the
  repository root
- **THEN** the image builds without error and
  `podman run --rm pois --help` displays the CLI help page

#### Scenario: docker build produces the same image

- **WHEN** a contributor runs `docker build -t pois .` at the
  repository root with the same `Dockerfile`
- **THEN** the image builds without error and
  `docker run --rm pois --help` displays the CLI help page, so the
  `Dockerfile` is consumable by either CLI

#### Scenario: PORT is honored

- **WHEN** `podman run -e PORT=3000 -e POIS_ADMIN_USER=u -e
  POIS_ADMIN_PASS=p pois` runs
- **THEN** the gateway listens on port 3000

#### Scenario: The production image defaults to JSON logs

- **WHEN** `podman run --rm -e POIS_ADMIN_USER=u -e POIS_ADMIN_PASS=p
  pois` starts the gateway
- **THEN** stderr output is one JSON object per line, because
  `POIS_LOG_FORMAT=json` is set in the image environment

### Requirement: Foundation invariants evolve via OpenSpec

Any modification to an invariant listed above SHALL be raised
through an OpenSpec proposal that amends this specification via a
`MODIFIED Requirements` or `REMOVED Requirements` block. The
invariants in question are: product shape, toolchain, crate layout,
async runtime, error conventions, lint policy, `/data/` schema,
deployment target (OCI container image), dashboard authentication,
third-party asset sourcing, observability log format, repository
language, developer tooling pinning, and licensing. Silent
modifications of `openspec/project.md`, `rust-toolchain.toml`,
`Cargo.toml`, `mise.toml`, `Dockerfile`, `LICENSE`, or the templates
under `templates/` that touch these invariants MUST NOT be merged
without an associated proposal.

#### Scenario: A PR touches an invariant file without a proposal

- **WHEN** a contributor submits a PR that modifies
  `rust-toolchain.toml`, the invariant sections of
  `openspec/project.md`, the `rust` entry of `mise.toml`, the
  `rust-version` field of `Cargo.toml`, the auth sections of the
  `Dockerfile`, the `LICENSE` file, or the asset `<script>` /
  `<link>` URLs in `templates/`, without an associated OpenSpec
  proposal
- **THEN** review requires the proposal to be opened before merge

## ADDED Requirements

### Requirement: Third-party web assets load from public CDNs

The dashboard SHALL load third-party web assets (at minimum `htmx`
and `pico.css`, plus any future library of the same nature) from
public CDN URLs referenced directly in Askama templates. The
repository MUST NOT vendor copies of these assets under
`templates/`, `assets/`, or `static/`. Subresource integrity (`integrity`
attribute) is RECOMMENDED but not required at this stage.

#### Scenario: Templates reference htmx and pico.css from CDN

- **WHEN** a contributor inspects the Askama template that renders
  the dashboard base layout
- **THEN** they find `<script src="https://...">` for htmx and
  `<link href="https://...">` for pico.css, each pointing to a
  public CDN domain, and no relative path pointing to a local copy

#### Scenario: No vendored asset directory exists

- **WHEN** a contributor inspects the repository root
- **THEN** there is no `assets/` or `static/` directory, and
  `templates/` contains only Askama `.html` files (no `.js`, `.css`,
  font, or image copies of third-party assets)

#### Scenario: A CDN version bump requires a proposal

- **WHEN** a contributor wants to change the pinned version of
  `htmx`, `pico.css`, or any future third-party asset referenced by
  CDN URL in Askama templates
- **THEN** they open an OpenSpec proposal that amends this
  requirement via a `MODIFIED Requirements` delta naming both the
  old and new version strings

### Requirement: Observability log format is controlled by POIS_LOG_FORMAT

The binary SHALL initialise `tracing-subscriber` with a layer
selected by the `POIS_LOG_FORMAT` environment variable. The
accepted values are `json` (one structured JSON object per line) and
`pretty` (human-readable, multi-line). When the variable is unset or
empty, the binary MUST default to `pretty`. When the variable is
set to an unrecognised value, the binary MUST refuse to start and
emit an error message to stderr naming the unknown value.

#### Scenario: Default format is pretty

- **WHEN** the binary starts without `POIS_LOG_FORMAT` set
- **THEN** log output uses the human-readable `pretty` layer

#### Scenario: json is honored

- **WHEN** the binary starts with `POIS_LOG_FORMAT=json`
- **THEN** every log line on stderr parses as a standalone JSON
  object

#### Scenario: Unknown value is rejected

- **WHEN** the binary starts with `POIS_LOG_FORMAT=xml`
- **THEN** the process exits with a non-zero status code and stderr
  contains the string `POIS_LOG_FORMAT` alongside the unknown value

### Requirement: Repository artefacts are written in English

Every file committed under the repository root SHALL be written in
English. This applies to source code, code comments, documentation
under `openspec/` and outside of it, proposal artefacts, task lists,
specs, `README.md`, and any future narrative file. The sole
exception is the archived change tree at
`openspec/changes/archive/`, which preserves the original language
of each change as immutable historical record. Live conversation
between the operator and an agent is explicitly out of scope
because it does not produce a committed file.

#### Scenario: A contributor reads any file under openspec/ outside archive/

- **WHEN** a contributor opens any file under `openspec/` other
  than a folder nested under `openspec/changes/archive/`
- **THEN** the file is written in English

#### Scenario: A PR introduces a non-English narrative block

- **WHEN** a contributor submits a PR whose diff contains non-English
  prose in any committed file outside `openspec/changes/archive/`
- **THEN** review requires the block to be translated before merge

#### Scenario: The archive preserves original language

- **WHEN** a contributor reads files under
  `openspec/changes/archive/`
- **THEN** the files appear in whatever language they were in at
  the time of archiving, unchanged

#### Scenario: Commit messages are in English

- **WHEN** a contributor proposes a commit whose message (subject
  and body) is written in a non-English natural language
- **THEN** review requires the commit message to be rewritten in
  English (via `git commit --amend` or squash) before merge

### Requirement: Repository ships under the MIT license

A `LICENSE` file SHALL exist at the repository root. The file MUST
contain the standard MIT licence text (as published on
`https://spdx.org/licenses/MIT.html`) with a copyright line that
reads exactly `Copyright (c) 2026 Supergeoff`. The `Cargo.toml`
`package.license` field SHALL be set to `"MIT"`.

#### Scenario: LICENSE file is at the repository root

- **WHEN** a contributor runs `ls LICENSE` at the repository root
- **THEN** the file exists, contains the string
  `Copyright (c) 2026 Supergeoff`, and contains the MIT permission
  notice paragraph starting with `Permission is hereby granted`

#### Scenario: Cargo.toml declares the MIT license

- **WHEN** a tool parses the root `Cargo.toml`
- **THEN** it finds `package.license = "MIT"`

### Requirement: Developer tooling is pinned via mise

The repository SHALL provide, at its root, a `mise.toml` file whose
`[tools]` table pins the Rust toolchain to the same version string
as `rust-toolchain.toml`'s `[toolchain].channel` and `Cargo.toml`'s
`[package].rust-version`. The three pins MUST be byte-identical so
that any contributor using `rustup` (via `rust-toolchain.toml`),
`mise install` (via `mise.toml`), or reading MSRV intent (via
`Cargo.toml`) sees the exact same Rust version. Developer tools
that do not participate in producing the `pois` binary (for example
agent CLIs installed for convenience) MAY be pinned to `"latest"`
or any floating version; they are explicitly out of scope of this
cross-check. The `[env]` table of `mise.toml` is likewise out of
scope.

#### Scenario: mise.toml pins the Rust version

- **WHEN** a contributor parses `mise.toml`
- **THEN** the `[tools]` table contains a `rust = "X.Y.Z"` entry

#### Scenario: All three Rust pins agree

- **WHEN** a contributor reads the Rust version string from
  `mise.toml` `[tools].rust`, `rust-toolchain.toml`
  `[toolchain].channel`, and `Cargo.toml` `[package].rust-version`
- **THEN** the three values are byte-identical

#### Scenario: A floating version is allowed for non-build-critical tools

- **WHEN** a contributor inspects `mise.toml` and finds a tool
  pinned to `"latest"` (for example
  `"npm:@anthropic-ai/claude-code"`)
- **THEN** the requirement is satisfied as long as that tool does
  not participate in producing the `pois` binary
