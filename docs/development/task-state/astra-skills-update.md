# Astra skill adaptation — current task state

| Field | Value |
| --- | --- |
| Status | COMPLETE |
| Updated | 2026-09-05 |
| Task key | astra-skills-update |
| Scope | Adapt project and available user-space skill instructions to Astra guidance |
| Definition of done | All inventoried skills reviewed, conflicting process defaults corrected, links/frontmatter checked, project change committed |
| Authority | User request and AGENTS.md; skills do not change Accepted product contracts |

## Resume in 60 seconds

- **Current conclusion:** All 34 inventoried skills now share scoped execution guidance; conflicting workflow defaults were corrected in the relevant entrypoints.
- **Evidence:** Official [Astra guide](https://developers.openai.com/api/docs/guides/latest-model); local skill entrypoints and their references.
- **Next action:** Use the adapted skills. Re-audit managed files after an upstream update or a concrete behavioral failure.
- **Current blocker:** None.
- **Do not retry:** Do not replace skills wholesale from upstream or edit inactive cached plugins; that would lose local policy or change unavailable workflows.
- **Reconsider when:** New plugin versions replace the local adaptations, or observed behavior shows a remaining conflict.

## Decisions that still constrain the work

### D-001 — Scope behavior changes without changing product authority

- **Observation:** Rust/RL routers mandate questions and generic documentation; CLI/CAD/artifact workflows contain unconditional process gates.
- **Evidence:** Pre-edit files and SHA-256 inventory in `/home/kaifaty/.codex/backups/astra-skills-20260905-001348/inventory.json`.
- **Decision:** Share concise execution guidance within each scope and correct conflicting instructions at their source. Preserve technical references, licenses, names, invocation policy and scripts.
- **Rejected alternatives:** Upstream reinstall would discard project adaptations; adding model/API configuration would exceed this instruction-only request.
- **Consequences:** Local plugin/system edits require re-audit after managed updates; project changes travel through Git.
- **Uncertainty:** Static checks and six read-only scenario evaluations do not measure live behavior for every skill. Unchanged technical references are not newly certified; the scenario review found pre-existing offline-RL example defects recorded in the external backup report.
- **Reconsider when:** A real task exposes a conflict or an upstream update changes a patched file.

## Required context

1. [Repository instructions](../../../AGENTS.md).
2. [Shared skill guidance](../../../.agents/skills/astra-guidance.md).
3. [Skill provenance](../../../.agents/skills/README.md).
4. External backup inventory above for exact original files.

## Handoff

- **Coverage:** 16 project, 7 user/system, 11 active plugin skills; inactive plugin caches excluded. Two shared guidance files, one per independently portable scope.
- **Checks passed:** YAML/identity 34/34; all 122 entrypoint Markdown local links outside code examples; all 51 shortened-router topic references; documentation diff whitespace checks. Code examples in other entrypoints remain unchanged.
- **Validator exceptions:** `quick_validate.py` passes 32/34. Upstream plugin names `Presentations` and `Spreadsheets` already fail its lowercase-name rule; preserved to avoid changing plugin identities. No new validator regressions.
- **Scenario evidence:** Independent read-only forward pass across six tasks; DOCX process conflict corrected and rechecked with no remaining finding in scope. These scenarios did not execute real training, publishing, document rendering or CAD operations.
- **Not run:** Cargo, host-check and runtime ProductChecks; this change contains only documentation and agent guidance.
- **Recovery:** [External backup report](/home/kaifaty/.codex/backups/astra-skills-20260905-001348/README.md), before/after snapshots, hashes and patch. Managed plugin/system updates may replace local adaptations.
- **Workspace:** Unrelated water changes were committed separately during this task and are excluded from this skill change. Runtime, product-contract, roadmap, model/API configuration, scripts and invocation metadata were not changed.
- **Promotion:** Reusable policy is in the shared guidance and skill entrypoints; provenance changes are in the skills README. No architecture or roadmap promotion is needed.
