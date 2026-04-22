## 1. Translate narrative artefacts in openspec/

- [x] 1.1 Rewrite `openspec/project.md` in English: translate every
      paragraph, section heading, and bullet. Preserve code fences,
      ASCII tree diagrams, inline file/path literals, URLs and
      inspiration citations bit-identical. Do not add, remove or
      reorder content.
- [x] 1.2 Review the translated `openspec/project.md` against the
      French source section by section to confirm no semantic drift.
- [x] 1.3 Translate the `## Purpose` section (top of
      `openspec/specs/project-foundations/spec.md`, currently lines
      3-5 in French) directly in place. Preserve every literal,
      requirement reference, and enumerated invariant list. OpenSpec
      deltas do not carry the Purpose section, so this edit is the
      only way to land a fully-English promoted spec after archival.

## 2. Review delta spec correctness

- [x] 2.1 Read
      `openspec/specs/project-foundations/spec.md` (French) and
      compare each requirement body and scenario to the English
      version now drafted in
      `openspec/changes/translate-openspec-to-english/specs/project-foundations/spec.md`.
- [x] 2.2 For each of the 10 `MODIFIED Requirement` blocks, verify:
      header text matches the source exactly, every RFC 2119 keyword
      (SHALL / MUST / SHOULD / MAY) maps to the correct French
      original (DOIT, MUST, SHALL, PEUT, MAY), and every literal
      (env var names, file paths, commands, exit codes, URLs,
      versions) is bit-identical to the source.
- [x] 2.3 Run `openspec validate translate-openspec-to-english --strict`
      and confirm success.
- [x] 2.4 Run `openspec show translate-openspec-to-english --type change --deltas-only`
      and eyeball the delta structure.

## 3. Pre-archive verification

- [x] 3.1 Ensure no file under `openspec/changes/archive/` has been
      modified by this change (`git diff openspec/changes/archive/`
      returns empty).
- [x] 3.2 Ensure no file outside the expected set has been modified by
      this change. Expected modifications: `openspec/project.md`
      (task 1.1), `openspec/specs/project-foundations/spec.md`
      (task 1.3 — Purpose paragraph only), and the new change folder
      `openspec/changes/translate-openspec-to-english/`. Any other
      modified tracked file is out of scope and must be reverted
      before archiving.
- [x] 3.3 Confirm that `cargo build`, `cargo fmt --check` and
      `cargo clippy --all-targets -- -D warnings` still succeed (no
      code was touched, this is a regression guard).

## 4. Archive the change

- [ ] 4.1 After user review, run
      `openspec archive translate-openspec-to-english` (or the
      equivalent `/opsx-archive-change` slash command) to promote the
      English delta into `openspec/specs/project-foundations/spec.md`.
- [ ] 4.2 Verify the archived promoted spec is fully in English and
      contains all 10 requirements by reading the file end to end.
- [ ] 4.3 Run `openspec validate --all --strict` and confirm success.
