## Context

The `pois` repository carries every OpenSpec artefact (the promoted
specification `project-foundations/spec.md` and the repository-wide
`openspec/project.md`) in French. A broader language rule is about to
land in a separate follow-up change (`align-stack-preferences`):
everything in the repository, without distinction between specs,
narrative documentation, and proposals, must be written in English;
only live conversation with the operator stays in French.

If the language change and new normative requirements landed in the
same proposal, the diff would mix mechanical re-expression with real
policy shifts, making review harder and the archived delta confusing
to re-read. This change splits the two concerns and delivers only the
mechanical pass: translate every existing artefact under `openspec/`
outside of `openspec/changes/archive/`, with zero semantic change.

Stakeholders: the single operator of `pois`. No external consumer.

## Goals / Non-Goals

**Goals:**

- Produce an English `openspec/project.md` that is paragraph-for-
  paragraph equivalent to the current French version.
- Produce an English promoted `project-foundations/spec.md` (after
  archival) that is requirement-for-requirement equivalent to the
  current French version, with identical scenarios, literals and
  structure.
- Make the review so boring that a human can confirm it with a
  side-by-side read in a single sitting.
- Leave the repository in a state where `align-stack-preferences` can
  be proposed against a purely English baseline.

**Non-Goals:**

- Any new, removed, strengthened or weakened requirement. That work
  belongs to `align-stack-preferences`.
- Editing any file under `openspec/changes/archive/`. Archived change
  folders are immutable historical record; they stay in their original
  French.
- Cleaning up `openspec/config.yaml`. The file currently contains an
  invalid YAML block that triggers a non-fatal parse warning; fixing
  it is scoped to `align-stack-preferences` because the fix is part of
  the broader context-source consolidation decided there.
- Introducing a testable requirement about the English-only rule. The
  rule itself is codified in `align-stack-preferences`; this change
  only makes the existing content compliant in advance.
- Translating commit messages, code comments, or other non-`openspec/`
  artefacts. The repository already uses English outside `openspec/`.

## Decisions

### D1. Translate, do not rewrite

**Choice:** every French sentence maps to its closest English
equivalent. Paragraph order is preserved. Section headings are
translated (`## Outillage` → `## Tooling`, etc.) but not renamed to
reflect new taxonomies. Idioms are carried over to idiomatic English,
not transliterated.

**Rationale:** mixing language change with stylistic improvement
defeats the purpose of a mechanical pass and makes the diff harder to
audit. Improvements belong to follow-up proposals that can justify
each change on its own merits.

**Alternatives considered:**
- *Translate and tighten prose in the same pass.* Rejected for the
  above reason.
- *Translate only the sections modified by the follow-up proposal.*
  Rejected: it would leave the promoted spec hybrid FR/EN after both
  changes land, which violates the English-only rule that
  `align-stack-preferences` itself establishes.

### D2. Preserve all literals verbatim

**Choice:** environment variable names (`POIS_ADMIN_USER`,
`POIS_DATA_DIR`, `PORT`), file paths (`/data`, `Cargo.toml`,
`rust-toolchain.toml`), commands (`cargo fmt --check`,
`cargo tree -i async-std`), version strings (`1.95.0`, `edition = "2024"`),
URLs, exit-code expectations, and scenario values remain bit-identical
to the French source.

**Rationale:** these are the testable parts of the spec. Touching them
would slip semantic change into a mechanical pass.

### D3. Delta spec uses MODIFIED for every existing requirement

**Choice:** the delta under
`changes/translate-openspec-to-english/specs/project-foundations/spec.md`
contains one `MODIFIED Requirement` block per existing requirement of
the promoted spec, each with the full English re-expression. No ADDED,
no REMOVED.

**Rationale:** OpenSpec deltas cannot carry a pure "translate without
changing requirements" operation; the closest primitive is MODIFIED
with semantically equivalent content. Listing all 10 requirements
keeps the archival step well-defined and the resulting promoted spec
fully in English.

Note: requirement header names are already in English in the source
spec (for example `### Requirement: Rust toolchain is pinned`). Only
requirement bodies, scenario names, and scenario WHEN/THEN clauses
are translated.

**Alternatives considered:**
- *Edit the promoted spec in place without a delta.* Rejected: it
  would silently mutate a normative artefact outside the OpenSpec
  flow, and the language rule itself (landing in the follow-up) will
  forbid exactly that kind of silent change on invariant files.

### D4. Project-level narrative translated without a delta

**Choice:** `openspec/project.md` is narrative documentation, not a
normative spec. It is translated directly as part of the task list,
outside the delta mechanism.

**Rationale:** `project.md` has no "requirement" structure for OpenSpec
to track. The OpenSpec CLI treats it as context; rewriting it in place
is the natural operation.

### D4b. Spec Purpose translated in place, not through the delta

**Choice:** the top-level `## Purpose` paragraph of
`openspec/specs/project-foundations/spec.md` is edited directly in
place during this change's apply phase, before the archive step. The
delta file only carries `## MODIFIED Requirements` blocks.

**Rationale:** OpenSpec's delta schema (`## ADDED | MODIFIED | REMOVED |
RENAMED Requirements`) has no primitive for the spec-level Purpose
section. A delta-only translation would leave a residual French
paragraph at the top of the promoted spec after archival. Editing
Purpose directly, as a tracked task of this change, is the only
sound way to deliver a fully-English promoted spec without
over-engineering the delta format.

**Alternative considered:**
- *Leave Purpose in French, translate it in a later change.* Rejected:
  P1's contract says the promoted spec is fully English after
  archival. Leaving Purpose FR breaks that promise and would require
  a follow-up change that only fixes a paragraph.

### D5. Archived change folders are immutable

**Choice:** no file under `openspec/changes/archive/` is touched. The
archived bootstrap change stays in French.

**Rationale:** archives are historical record of what was decided at
the time, in the language it was decided. Retroactive translation
would rewrite history and make old reviews non-locatable via git
blame.

## Risks / Trade-offs

- **[Translation drift between source and target]** some French
  sentence could be rendered in a way that silently changes its
  meaning (e.g. "MUST" vs "SHOULD" shades). → *Mitigation:* reviewer
  performs a side-by-side read; any deviation from strict RFC 2119
  keyword mapping is flagged.
- **[Hybrid state during the transition]** between archival of this
  change and start of `align-stack-preferences`, the promoted spec is
  fully English while archived change artefacts remain French. → *
  Mitigation:* acceptable and expected. The hybrid is between live
  content (now English) and archive (always historical), not within a
  single artefact.
- **[Config.yaml parse warning persists]** `openspec` commands emit a
  YAML parse warning because of the malformed `Example:` block. → *
  Mitigation:* non-fatal; all commands proceed. Fix deferred to
  `align-stack-preferences`.
- **[Review fatigue on 11 MODIFIED blocks]** the delta is long even
  though the semantics are identical. → *Mitigation:* reviewer can
  diff each MODIFIED block against the corresponding French
  requirement in the current promoted spec, and the task list
  enumerates which requirement maps to which MODIFIED block.

## Migration Plan

This change is a documentation pass; no code or runtime migration is
required. Rollback on a merged change is a plain `git revert`.

## Open Questions

- None blocking. Copyright holder for the future `LICENSE` file and
  further tooling-narrative decisions live in `align-stack-preferences`;
  they are not required to complete this translation pass.
