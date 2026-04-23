## Context

P1 (`translate-openspec-to-english`) left the `pois` repository with a
fully English OpenSpec baseline and no semantic changes. Several
policies were discussed during P1 but deliberately deferred here so
the translation pass could stay mechanical. Those policies now land
together under `project-foundations`:

- Container tooling (podman as primary local CLI, docker/buildah as
  compatible alternatives, `Dockerfile` kept as file name).
- Third-party web assets (htmx, pico.css) loaded from public CDNs
  rather than self-hosted.
- `POIS_LOG_FORMAT` elevated from narrative mention
  (`openspec/project.md:23`) to a spec-level requirement.
- Repository language invariant: every committed file in English;
  archived change folders exempt as historical record; live
  conversation with the operator stays free-form.
- MIT licensing via a `LICENSE` file at the repository root.
- `openspec/project.md` gains a `## Tooling` narrative section
  describing `mise`, cargo workflow, and container-CLI alternatives.
- `openspec/config.yaml` cleaned up to silence the non-fatal YAML
  parse warning (`Example:` block removed).

Stakeholder: the single operator (Supergeoff). No external audience.

## Goals / Non-Goals

**Goals:**

- Codify the four pending policies (container tooling, asset CDN,
  log format, language rule) as first-class requirements of
  `project-foundations`, each with at least one WHEN/THEN scenario.
- Rename the deployment requirement title to a container-tool-agnostic
  wording so future CLI alternatives do not require another RENAMED.
- Add the `LICENSE` file at the repository root so any third-party
  reader has an unambiguous legal reference.
- Produce a narrative `## Tooling` section under
  `openspec/project.md` that names `mise` as the toolchain/CLI
  manager and mentions podman/docker/buildah as compatible
  container CLIs.
- Leave `openspec/config.yaml` in a strictly valid YAML shape so
  `openspec` commands stop emitting a parse warning.
- Leave runtime code untouched. `POIS_LOG_FORMAT` already works; the
  spec only documents the invariant.

**Non-Goals:**

- No runtime behaviour change. This is a documentation + policy pass
  with two file additions (`LICENSE`, optional template cleanup) and
  two file edits (`openspec/project.md`, `openspec/config.yaml`).
- No Dockerfile rename to `Containerfile`. Podman consumes
  `Dockerfile` natively; renaming would force docker users to know an
  alias, for zero functional gain.
- No introduction of a contribution licence policy (CLA, DCO, etc.)
  beyond the MIT file itself. Single-operator repo; not needed.
- No test harness for the language rule. Enforcement is by review.
  Automating it would require a spell/language detector that is
  heavier than the policy itself.
- No attempt to retroactively translate archived change folders;
  they are immutable historical record (decision carried from P1).
- No observability framework changes. `POIS_LOG_FORMAT` keeps its
  existing semantics (`json` / `pretty` selector).
- No mise task/pipeline file added in this pass. The narrative
  explains the role; tooling wiring is out of scope.

## Decisions

### D1. Everything lands in the `project-foundations` capability

**Choice:** asset CDN, log format, language rule, licensing, and the
container-tool rewording all land as requirements under the existing
`project-foundations` capability, not under a new
`project-policies` capability.

**Rationale:** the audience, review path, and lifecycle are identical
for every invariant here. Splitting capabilities to group by theme
would fragment discovery (reviewers would have to look in two specs
for "how does this repo ship") and would duplicate the existing
"invariants evolve via OpenSpec" meta-requirement.

**Alternatives considered:**

- *Create `project-policies` for the non-stack invariants.* Rejected:
  stakeholder count is one; organisational overhead is zero-value.
- *Create `project-licensing` dedicated to MIT.* Rejected: a single
  requirement does not justify a capability.

### D2. Container tool: RENAMED + MODIFIED for the deployment requirement

**Choice:** the existing requirement
`### Requirement: Deployment target is Docker / Railway` is renamed
to `### Requirement: Deployment target is an OCI container image`
via a `## RENAMED Requirements` block, and then fully re-expressed
via `## MODIFIED Requirements` with scenarios that exercise `podman
build` / `podman run` as the primary path, with a scenario that
confirms `docker build` produces the same image from the same
`Dockerfile`.

**Rationale:** RENAMED exists precisely for title changes; using it
keeps traceability explicit and keeps the MODIFIED block focused on
body/scenario content. Combining both primitives is idiomatic
OpenSpec usage.

**Alternatives considered:**

- *Plain MODIFIED with a changed title.* Rejected: MODIFIED is
  whitespace-insensitive on header match, so using MODIFIED with a
  new title would fail to pair with the old requirement on archive.
- *RENAMED only, body unchanged.* Rejected: the body still says
  "Docker image" and the scenarios still name `docker build`;
  leaving them would create a mismatch between title and body.

### D3. `Dockerfile` name is preserved

**Choice:** the file at the repository root stays named `Dockerfile`.
No `Containerfile` alias, no rename.

**Rationale:** `podman build` reads `Dockerfile` by default; there is
no functional reason to change. Docker users would need to learn the
alias, and most CI/CD integrations hardcode `Dockerfile`. The cost
of the rename exceeds any perceived cleanliness benefit.

### D4. Asset CDN policy is normative-by-review, with explicit bump-via-proposal rule

**Choice:** the requirement states the invariant (htmx and pico.css
come from public CDN URLs; no vendored copy in `templates/`,
`assets/`, or `static/`) with scenarios that reference template
inspection and directory absence. In addition, a third scenario
codifies that any version bump of a CDN-pinned asset
(e.g. `htmx 2.0.4` → `htmx 2.0.5`) SHALL go through an OpenSpec
proposal. No automated check is wired in for either rule.

**Rationale:** enforcement cost (adding a build step that greps
templates) exceeds the risk level (single operator; each PR touches
templates rarely). The CDN-version bump rule aligns with the
existing in-file comment in `templates/base.html`
(`"bump via OpenSpec proposal"`), turning a tacit convention into a
tested scenario. An automation pass can be added later via a
dedicated proposal if the repo ever scales beyond one operator.

**Alternative considered:**

- *Add a `cargo xtask check-assets` CLI.* Rejected for complexity; may
  resurface if a future proposal introduces a wider cargo-xtask layer.
- *Leave the bump rule as the in-template comment only.* Rejected
  because the comment is not normative — a contributor could bump
  without noticing the comment, whereas a scenario surfaces the
  rule during spec review.

### D5. `POIS_LOG_FORMAT` accepts `json` | `pretty`, default `pretty`, unknown rejected

**Choice:** the binary SHALL default to the `pretty` human-readable
output when `POIS_LOG_FORMAT` is unset or empty. Accepted values are
`json` and `pretty` (case-insensitive). Any other value causes the
binary to exit with a non-zero status. The production `Dockerfile`
SHALL set `ENV POIS_LOG_FORMAT=json` so deployed images emit
structured logs by default.

**Rationale:** `pretty` is the dev-friendly choice on a laptop; it is
what the operator sees on first `cargo run`. Production deployments
need machine-parseable logs, so the Dockerfile flips the default.
Rejecting unknown values catches typos (e.g. `JSON` vs `json` works
thanks to case-insensitivity, but `jsno` would surface as a hard
error rather than silently falling through).

**Code touched by this decision:** `src/cli/gateway.rs::init_tracing`
changes from a binary `is_ok_and(json)` check to an explicit match
on the three outcomes (unset/empty → pretty, `json` → json,
`pretty` → pretty, other → error + exit). Scope is one function,
roughly ten lines. The narrative in `openspec/project.md:23` is
also adjusted from "compact in dev" to "pretty in dev" so the
project-level doc matches the normative requirement.

**Alternative considered:**

- *Default `json` everywhere, require developers to opt out.*
  Rejected: first-run UX matters; a developer running `cargo run` on
  a fresh clone should see readable logs.
- *Keep the `compact` formatter name in the spec.* Rejected:
  `compact` is a `tracing-subscriber` jargon term; `pretty` as the
  user-facing value is more portable and allows future changes to
  the internal formatter without a spec update.

### D6. Language rule exempts only `openspec/changes/archive/`, and covers commit messages

**Choice:** the requirement body states that every committed file
under the repository root SHALL be written in English. The one
exception is the archived change tree at
`openspec/changes/archive/`, which preserves the original language
of each change as historical record (decision inherited from P1 D5).
A dedicated scenario extends the rule to git commit messages:
subject and body are in English, enforced at review (amend or
squash before merge). Live operator ↔ agent conversation remains
out of scope because it produces no committed artefact.

**Rationale:** the archive is immutable; retroactive translation
would rewrite history and make `git blame` harder. Any other file
(code comments, narrative documentation, active proposals, future
specs) is English. Commit messages are part of the permanent
versioned history — the same logic that applies to files applies to
them. Covering commit messages in the same requirement keeps the
language policy in one place.

**Alternative considered:**

- *Allow French in narrative documentation (`docs/`) and English
  only in normative specs.* Rejected: dual-language corpora force
  contributors (or future contributors) to translate on reading;
  single-language keeps artefacts comparable.
- *Leave commit messages free-form.* Rejected: versioned history is
  an artefact of the repository; mixing languages there is the same
  kind of drift the file-level rule prevents.

### D7. MIT licence, `Copyright (c) 2026 Supergeoff`

**Choice:** the repository ships a `LICENSE` file at the root with
the standard MIT text (as published on `spdx.org/licenses/MIT`), and
a single copyright line reading `Copyright (c) 2026 Supergeoff`.

**Rationale:** MIT is the minimum-friction permissive licence; it
matches a single-operator, small-surface project. The copyright
holder is the operator's preferred name. The year `2026` reflects
the current session.

**Alternatives considered:**

- *Apache-2.0 for explicit patent grant.* Rejected: no patent
  situation anticipated; MIT is simpler.
- *Dual MIT / Apache-2.0 (Rust ecosystem convention).* Rejected for
  this stage: single operator, no downstream consumer yet; MIT-only
  is enough and the dual option can be revisited via a follow-up
  proposal if the licence audience expands.

### D8. `Product shape` and `Foundation invariants` requirements get MODIFIED too

**Choice:** the existing requirement
`### Requirement: Product shape is documented` is updated (MODIFIED)
to say "OCI container image" instead of "Docker image" for
terminology consistency. The requirement
`### Requirement: Foundation invariants evolve via OpenSpec` is
updated (MODIFIED) to enumerate the new invariants (asset sourcing,
log format, language rule, licensing) in its list of tracked
invariants.

**Rationale:** without these two updates the spec would be internally
inconsistent — one requirement would name the container tool as
"OCI" while another keeps "Docker"; and the meta-requirement
enumerating what is tracked would be out of date relative to what
this change adds.

### D9. `## Purpose` edited in place, same pattern as P1 D4b

**Choice:** the top-level `## Purpose` paragraph of
`openspec/specs/project-foundations/spec.md` is edited directly
during the apply phase of this change, before archive. The delta
file carries only requirement-level blocks.

**Rationale:** OpenSpec deltas have no primitive for the spec Purpose
section. Inheriting P1's D4b pattern keeps the approach consistent
and leaves the promoted spec internally coherent after archive.

### D11. `mise.toml` is a minimal contract: Rust pin cross-check only

**Choice:** `mise.toml` becomes a normative artefact, but with a
deliberately narrow scope. It MUST pin the Rust version under
`[tools].rust` to a value byte-identical to `rust-toolchain.toml`'s
`[toolchain].channel` and `Cargo.toml`'s `[package].rust-version`.
Everything else in `mise.toml` — other pinned tools (such as
`"npm:@anthropic-ai/claude-code"`), floating versions (`"latest"`),
and entries under `[env]` (such as `HONCHO_WORKSPACE_ID`) — is
explicitly out of the contract scope.

**Rationale:** the three-way Rust pin is a real foot-gun: a
contributor who bumps only one location causes the other two to
silently drift. A single cross-check scenario attaches a review
trigger to this. Pinning anything else in `mise.toml`
(cargo-nextest, cargo-audit, python for openspec) would creep the
contract surface without real benefit — those tools are not on the
build-critical path for `pois`. Allowing `"latest"` for agent CLIs
keeps the file's current shape intact.

**Alternatives considered:**

- *Require `mise.toml` to enumerate every cargo tool used in the
  workflow (nextest, audit, etc.).* Rejected: creates friction on
  every workflow tweak for zero downstream benefit while the
  operator is the sole contributor.
- *Require `mise.toml` to set `HONCHO_WORKSPACE_ID`.* Rejected:
  `[env]` is operator-scoped; forcing a specific value would make
  the file personal rather than repository-level.
- *Remove the Rust entry from `mise.toml` and let only
  `rust-toolchain.toml` pin it.* Rejected earlier during
  exploration: `mise install` would then not know which Rust to
  install on a fresh clone; rustup is not a hard prerequisite.

### D10. `openspec/config.yaml` trimmed to a valid minimal shape

**Choice:** `openspec/config.yaml` is reduced to the single valid
line `schema: spec-driven`, with all commented `Example:` blocks
removed. A one-line comment links to `openspec/project.md` as the
source of project context.

**Rationale:** the current `Example:` block contains indentation that
`js-yaml` (used by the OpenSpec CLI) parses as an invalid top-level
key, which triggers the current parse warning. Removing the
commented examples makes the file strictly valid. Project context
already lives in `openspec/project.md`; duplicating it here would
drift.

**Alternative considered:**

- *Fix the indentation and keep the examples as a teaching block.*
  Rejected: the examples duplicate content that the OpenSpec docs
  (and `openspec/project.md`) already cover; keeping them risks
  drift on every future config tweak.

## Risks / Trade-offs

- **[Podman and Docker differ on rootless / socket path]** the
  scenarios promise "same `Dockerfile`, same image, same runtime
  flags"; rootless-podman is the realistic default on Linux, and
  some `docker run` flags have subtle behavioural differences. →
  *Mitigation:* scenarios use only the intersection of the two CLIs
  (`-e`, `-v`, `-p`, no `--userns`/`--security-opt` mentioned). A
  real divergence would surface during the apply phase's validation
  build and can trigger a dedicated proposal.
- **[Asset drift without automated check]** a future contributor may
  vendor an asset without noticing the requirement. → *Mitigation:*
  review catches it; `tasks.md` adds a directory-scan step during
  apply to confirm current state is clean.
- **[Language rule re-opens for non-English integrations]** if a
  future integration exposes a non-English artefact (e.g. a
  third-party SDK doc), an exception would be needed. → *Mitigation:*
  requirement body explicitly names the archive-only exception;
  adding a new exception requires a proposal. That is the intended
  cost.
- **[License year drift]** the `2026` year may become stale next
  January. → *Mitigation:* MIT text requires only the year of first
  publication; the standard convention is to keep the original year
  (or use a range `2026-YYYY` if a material contribution happens
  later). Left as-is; a future proposal can widen if needed.
- **[Config.yaml downstream consumer regresses]** trimming the
  commented examples removes pedagogical content from the file. →
  *Mitigation:* the OpenSpec docs and `openspec/project.md` cover
  the same ground; no operator workflow depends on inline examples.
- **[Promoted spec grows by four requirements]** future reviews of
  `project-foundations` become longer. → *Mitigation:* the
  requirements are independent and read in isolation; a single TOC
  pass in project.md's narrative remains sufficient.

## Migration Plan

No runtime migration. Steps during apply are:

1. Inspect `templates/`, `assets/`, `static/` directories: confirm
   no vendored htmx/pico.css copy exists. If any is found, remove
   and adjust templates to CDN.
2. Edit `openspec/project.md`: add `## Tooling` section; adjust
   `## Tech stack` and `## Deployment` sections to name podman
   first; cross-link `POIS_LOG_FORMAT` to the new requirement.
3. Edit `openspec/specs/project-foundations/spec.md` `## Purpose`
   paragraph in place to enumerate the four new invariants.
4. Create `LICENSE` at the repo root.
5. Replace `openspec/config.yaml` with the minimal valid form.
6. Update Honcho peer card with podman-first preference (operational
   step, no file change).
7. `openspec validate --strict` and `cargo build / fmt / clippy`
   must pass before archive.

Rollback on a merged change is a plain `git revert`. No data
migration, no binary rebuild required.

## Open Questions

- None blocking. The Honcho peer card update is an operational step
  that happens during apply; if Honcho is unreachable, the update
  is deferred to a follow-up session but does not block archive of
  this change (the policy is codified in the spec regardless).
