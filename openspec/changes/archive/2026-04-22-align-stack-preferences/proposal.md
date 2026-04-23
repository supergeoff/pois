## Why

P1 (`translate-openspec-to-english`) delivered a fully-English OpenSpec
baseline without touching any semantics. Several policies discussed
during that work are ripe to codify now that the baseline is stable:
podman-first container tooling, public-CDN sourcing of third-party web
assets, `POIS_LOG_FORMAT` as a normative requirement (today only
narrative), a repository-wide language invariant (English everywhere
except live conversation), a tooling narrative under
`openspec/project.md` (`mise`, cargo, container CLIs), and the
`LICENSE` file itself (MIT, copyright `Supergeoff`). A non-fatal YAML
parse warning in `openspec/config.yaml` is cleaned at the same time
because it shares the context-consolidation theme.

These items are grouped in a single change because they share the
same audience (the single operator), the same capability
(`project-foundations`), and because they cross-reference each other
(the podman-first scenarios reuse the log-format env var; the
tooling narrative mentions both; the language rule is the policy
foundation for every future artefact).

## What Changes

- **MODIFIED** — deployment requirement: the existing
  "Deployment target is Docker / Railway" requirement is renamed to
  "Deployment target is an OCI container image" and rewritten so
  scenarios exercise `podman build` / `podman run` as the primary
  tooling, with `docker` (and optionally `buildah`) named as
  compatible alternatives that consume the same `Dockerfile` without
  modification. The file is NOT renamed to `Containerfile`.
- **NEW** — `Third-party web assets load from public CDNs`: `htmx`
  and `pico.css` SHALL be referenced via public CDN URLs from Askama
  templates; no vendored copy MAY live under `assets/`, `static/`,
  or `templates/`. A CDN version bump SHALL go through a dedicated
  OpenSpec proposal (codifies the existing `"bump via OpenSpec
  proposal"` comment in `templates/base.html`).
- **NEW** — `Observability log format is controlled by POIS_LOG_FORMAT`:
  the binary SHALL honour the `POIS_LOG_FORMAT` environment variable
  (`json` or `pretty`), with `pretty` as default; the production
  Dockerfile SHALL set `json`.
- **NEW** — `Repository artefacts are written in English`: every file
  committed under the repository root SHALL be written in English,
  with the sole exception of archived change folders under
  `openspec/changes/archive/` (historical record, immutable). The
  rule also extends to git commit messages (subject and body).
  Live conversation with the operator MAY happen in French; that
  is out of scope because it is not a committed artefact.
- **NEW** — `Repository ships under the MIT license`: a `LICENSE`
  file SHALL exist at the repository root containing the standard
  MIT licence text; the copyright line SHALL read
  `Copyright (c) 2026 Supergeoff`.
- **NEW** — `Developer tooling is pinned via mise`: `mise.toml`
  becomes a minimal normative contract that pins the Rust version
  under `[tools].rust` to the same byte-identical value as
  `rust-toolchain.toml`'s `[toolchain].channel` and `Cargo.toml`'s
  `[package].rust-version`. Non-build-critical tools
  (e.g. `"npm:@anthropic-ai/claude-code"`) and entries under
  `[env]` remain out of scope of the contract.
- **MODIFIED (in place, non-delta)** — `openspec/project.md` gains a
  `## Tooling` section describing `mise` for toolchain and CLI
  pinning, the cargo workflow, and the container-CLI alternatives
  (podman primary, docker/buildah compatible). The `## Tech stack`
  and `## Deployment` sections are adjusted to match the new
  container-tool wording. The `## Purpose` paragraph of the promoted
  `project-foundations` spec is edited in place to list the new
  invariants, following the same pattern as P1 decision D4b.
- **HOUSEKEEPING** — `openspec/config.yaml` reduced to the minimal
  valid YAML header (`schema: spec-driven`), with any `Example:`
  block removed; this eliminates the existing non-fatal YAML parse
  warning. `.dockerignore` reviewed and adjusted only if its content
  is tool-specific (expected outcome: no change needed). Honcho peer
  card updated with the podman-first preference as an operational
  step.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `project-foundations`: one requirement RENAMED + MODIFIED
  (deployment target becomes container-tool-agnostic), and five
  requirements ADDED (asset CDN sourcing with bump-via-proposal,
  log format, language rule covering commit messages too, MIT
  licensing, and `mise.toml` Rust pin cross-check). Spec
  `## Purpose` updated in place to enumerate the new invariants.

## Impact

- **Code**: one localized change in
  `src/cli/gateway.rs::init_tracing` (~10 lines) to tighten
  `POIS_LOG_FORMAT` handling: accept `json` | `pretty`
  case-insensitively, default to `pretty` when unset/empty, and
  reject any other value with a non-zero exit. No other runtime
  code changes. Templates are inspected to confirm the CDN-only
  policy already holds (current `templates/base.html` already
  references htmx and pico.css by CDN URL).
- **Infrastructure**: contributors may build and run with
  `podman build -t pois .` and `podman run -e PORT=8080 -e
  POIS_ADMIN_USER=u -e POIS_ADMIN_PASS=p -v pois-data:/data pois`.
  Docker users keep the same commands.
- **Docs**: `openspec/project.md` gains a `## Tooling` section and
  adjusted container-CLI mentions. Promoted spec Purpose updated.
- **Tooling**: `openspec validate` stops emitting the YAML parse
  warning because `openspec/config.yaml` becomes strictly valid.
- **Licensing**: a `LICENSE` file is added at the repository root.
  No third-party contribution policy is introduced (out of scope).
- **Single operator**: Supergeoff (the operator) reviews and merges.
  No external stakeholder.
