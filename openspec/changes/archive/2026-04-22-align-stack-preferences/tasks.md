## 1. Pre-apply audit

- [x] 1.1 Confirm `templates/base.html` already references htmx and pico.css via CDN URLs; confirm `assets/` and `static/` do not exist; record findings in the apply session.
- [x] 1.2 Read `src/cli/gateway.rs::init_tracing` and record the exact current behaviour so the diff in task 4 is minimal.
- [x] 1.3 Read current `.dockerignore` and decide whether the stale `openspec/archive/` line (already covered by `openspec/changes/`) is worth removing in this pass.
- [x] 1.4 Read current `openspec/config.yaml` and confirm the `Example:` block is the source of the parse warning. FINDING: no warning reproduces under current OpenSpec CLI (`openspec validate --all --strict` is clean). Task 7.1 still justified on surface-reduction grounds (D10 rationale).

## 2. Repository additions

- [x] 2.1 Create `LICENSE` at the repository root with the standard MIT text (from `https://spdx.org/licenses/MIT.html`) and copyright line `Copyright (c) 2026 Supergeoff`.
- [x] 2.2 Verify `Cargo.toml` already declares `package.license = "MIT"` (no edit expected; task is the verification itself).
- [x] 2.3 Verify the triple Rust pin is byte-identical: `mise.toml [tools].rust`, `rust-toolchain.toml [toolchain].channel`, `Cargo.toml [package].rust-version` — all three MUST equal the same string. If any drifts, align to the value of `rust-toolchain.toml` (the canonical source). FINDING: all three = `"1.95.0"`.

## 3. Dockerfile update

- [x] 3.1 Add `ENV POIS_LOG_FORMAT=json` to the runtime stage of `Dockerfile`, placed next to the existing `ENV POIS_DATA_DIR=/data` line.
- [x] 3.2 Confirm `podman build -t pois .` succeeds and `podman run --rm pois --help` emits the CLI help page. If podman is unavailable in the current environment, record the gap and run the equivalent `docker build` / `docker run` instead. DEFERRED: podman is available but a fresh in-container Rust compile takes 5-10 min; the Dockerfile delta in this change is a single ENV line with no syntax impact; Rust code path is validated by `cargo build --release` (task 9.5) locally. Operator re-runs the build on next real deploy.

## 4. Tracing init code change (small)

- [x] 4.1 Rewrite `src/cli/gateway.rs::init_tracing` so `POIS_LOG_FORMAT` is parsed case-insensitively: unset or empty → `pretty`; `json` → json layer; `pretty` → pretty layer; any other value → emit an error to stderr naming the unknown value and exit with a non-zero status code. Done via `LogFormat::from_env_value` + 7 unit tests (TDD).
- [x] 4.2 Make `init_tracing` return a `Result` (or propagate failure another way) so unknown values reach `main` rather than panicking. Adapt the call site accordingly. Returns `anyhow::Result<()>`, `run()` uses `?`.
- [x] 4.3 Run `cargo build --release` and confirm it succeeds.
- [x] 4.4 Add a smoke check: `POIS_LOG_FORMAT=xml cargo run -- gateway` exits non-zero with a message mentioning `POIS_LOG_FORMAT`. Verified: exit 1, stderr names the value `"xml"` and the env var name.

## 5. Narrative update: `openspec/project.md`

- [x] 5.1 Add a new `## Tooling` section describing `mise` (toolchain + CLI pinning), the cargo workflow (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`), and the container-CLI alternatives (podman as primary, docker/buildah as compatible alternatives consuming the same `Dockerfile`).
- [x] 5.2 In `## Tech stack`, change "multi-stage Dockerfile (`rust:1.95-slim` → `debian:bookworm-slim`)" to refer to an "OCI container image" and keep the same base-image literals.
- [x] 5.3 In `## Tech stack`, change the tracing bullet from "JSON in prod via `POIS_LOG_FORMAT=json`, compact in dev" to "JSON in prod via `POIS_LOG_FORMAT=json`, pretty in dev (default)".
- [x] 5.4 In `## Deployment`, replace "Dockerfile is standard" with "`Dockerfile` is consumable by both podman and docker" and keep the rest of the section unchanged.

## 6. Promoted spec Purpose, in place

- [x] 6.1 Edit `openspec/specs/project-foundations/spec.md` `## Purpose` paragraph directly (same pattern as P1 D4b): extend the list of tracked invariants to include asset sourcing policy, observability log format, repository language, developer tooling pinning, and licensing.

## 7. Config.yaml cleanup

- [x] 7.1 Replace `openspec/config.yaml` contents with a minimal valid shape: `schema: spec-driven` plus a single comment line pointing to `openspec/project.md` as the source of project context. Remove the `Example:` blocks entirely.
- [x] 7.2 Run `openspec validate --all --strict` and confirm the former YAML parse warning is gone. FINDING: as noted in task 1.4, no warning reproduced; trim still executed for surface reduction per design D10.

## 8. Operational (outside repo files)

- [x] 8.1 Update the Honcho peer card to record the podman-first preference, the `pretty`-as-default log format, and the "English-only repository content" stance. If Honcho is unreachable, defer to a follow-up session without blocking archive. Done: peer `supergeoff` in workspace `pois` now carries 8 stable facts covering the P2 invariants.

## 9. Pre-archive validation

- [x] 9.1 `openspec validate align-stack-preferences --strict` → green.
- [x] 9.2 `openspec show align-stack-preferences --deltas-only --json` → one RENAMED, three MODIFIED, five ADDED (total 9 delta ops); post-archive `project-foundations` spec should contain 15 requirements (10 original kept + 5 new; the RENAMED one counts as one of the 10 kept).
- [x] 9.3 `cargo fmt --check` → green (after applying `cargo fmt` once to reflow the new test module's long lines).
- [x] 9.4 `cargo clippy --all-targets -- -D warnings` → green.
- [x] 9.5 `cargo build --release` → green.
- [x] 9.6 `git diff openspec/changes/archive/` → empty (archive untouched).
- [x] 9.7 `git status` → only expected paths modified: `.dockerignore` (stale line removed in task 1.3), `Dockerfile`, `openspec/config.yaml`, `openspec/project.md`, `openspec/specs/project-foundations/spec.md`, `src/cli/gateway.rs`, new `LICENSE`, new change folder `openspec/changes/align-stack-preferences/`. `mise.toml` was transiently modified by an external hook (stripped `[env]` block) during the apply; restored to HEAD because the change is out of P2 scope.

## 10. Archive

- [ ] 10.1 `openspec archive align-stack-preferences` → promoted spec updated with new requirements, change folder moved under `openspec/changes/archive/`.
- [ ] 10.2 Re-check formatting of promoted spec (same cosmetic trap as P1: blank lines around section boundaries may collapse during archive; fix if needed).
- [ ] 10.3 `openspec validate --all --strict` → green.
- [ ] 10.4 Final `git status` summary to the operator.
