## Why

A new repository-wide language rule is landing: every textual artefact in
the repository (code, comments, Markdown under `openspec/`, README,
LICENSE, commit messages, PR descriptions) is written in English. The
only language exception is real-time conversation with the operator,
which is hors-repo. The current `openspec/` tree is entirely in French,
which violates the upcoming rule before it is even codified.

Rather than mix pure translation with new normative content, this
proposal performs the translation as a standalone, mechanical pass — no
requirement is strengthened, weakened, re-scoped, or re-worded beyond
the change of language. Keeping the translation in its own change makes
the diff trivial to review and leaves a clean English baseline for the
follow-up proposal (`align-stack-preferences`) that introduces new
requirements.

## What Changes

- Rewrite `openspec/project.md` in English, preserving every paragraph,
  section, bullet, code fence, diagram and hyperlink. No semantic
  change, no added or removed content.
- Translate every requirement of the promoted `project-foundations`
  specification from French to English via a delta that lists each
  existing requirement as **MODIFIED**. Scenario names, trigger
  conditions, environment variables, exit codes, file paths, URLs and
  any other literals stay identical to the source.
- Translate the `## Purpose` section of
  `openspec/specs/project-foundations/spec.md` directly in place.
  OpenSpec deltas only carry Requirement changes, so the Purpose
  section cannot ride through the delta mechanism and must be
  translated as a direct edit of the promoted spec, tracked by this
  change's task list.
- Explicitly leave out of scope: `openspec/config.yaml` cleanup,
  addition of any new requirement, edits to archived change folders
  under `openspec/changes/archive/`, code and template changes.

## Capabilities

### New Capabilities

<!-- None. This change is a mechanical translation. -->

### Modified Capabilities

- `project-foundations`: every existing requirement is re-expressed in
  English with identical semantics.

## Impact

- **Documentation artefacts**: `openspec/project.md` rewritten in place.
- **Specification**: `openspec/specs/project-foundations/spec.md` fully
  replaced in English once this change is archived (the delta lists all
  10 existing requirements as MODIFIED).
- **Archived changes** under `openspec/changes/archive/`: untouched,
  left in their original French as historical record.
- **Code**: no Rust source, no template, no `Cargo.toml` change.
- **Runtime behaviour**: none.
- **Review surface**: a reviewer can confirm correctness by comparing
  the French source and the English target paragraph by paragraph and
  requirement by requirement.
