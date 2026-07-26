# Autonomous Quest and Divine Architecture Consistency Remediation

| Field | Value |
|---|---|
| Status | Implemented |
| Date | 2026-07-26 |
| Scope | Architecture contracts and documentation only |
| Authority | SPEC-31, ADR-029 and ADR-031 after synchronization |

## Objective

Make the autonomous-quest and divine-judgment architecture decision-complete
without selecting an LLM/provider or claiming runtime implementation.

The implemented contract guarantees:

- a fixed world-time `NarrativeDecisionBoundaryV1`; provider/worker timing can
  affect only the recorded completion assignment, never the commit boundary or
  merge order;
- one bounded Runtime-owned `CrossContextTransactionPlanV1` for
  `AdmitQuestGraphRevision` and `AdmitDivineJudgmentBatch`;
- a closed `QuestSponsorV1`, RPG-owned `DivineStandingV1`, complete
  boon/covenant offer state machines and exact player/internal transition
  commands;
- whole-candidate per-god fallback, epistemically authorized directed
  spillover and bounded continuous reaction lineage;
- a world-locked ordered patron set and pantheon graph hash with fail-closed
  load compatibility;
- save/replay from recorded candidates, assignments and boundary closures with
  zero model or network regeneration.

## Work units completed

1. Normalized narrative/divine decisions around one fixed world-time boundary
   and SPEC-21 closed ingress assignment.
2. Defined owner-built RPG, Mechanics and quest-graph subplans with Runtime
   atomic composition and permanent cross-owner event ordering.
3. Completed quest sponsorship, divine standing, offer, covenant and player
   action schemas.
4. Defined whole-candidate fallback, epistemic edge eligibility, additive
   canonical resolution and reaction-root/depth lineage.
5. Locked the V1 pantheon per world and defined
   `DIVINE_PANTHEON_WORLD_MISMATCH`.
6. Synchronized affected Accepted SPECs, README, glossary and traceability;
   accepted SPEC-31, ADR-029 and ADR-031; recorded ADR-031's narrow partial
   supersession of ADR-020.

## Dependency order

The accepted dependency order is:

```text
Accepted baseline
  → ADR-029
  → ADR-031
  → SPEC-19
  → SPEC-31
```

Companion references from older subsystem SPECs do not create reverse
normative dependencies.

## Verification

Completed on 2026-07-26:

- relative Markdown link validation: `PASS`;
- focused dependency-DAG validation: `PASS`;
- `git diff --check`: `PASS`;
- `cargo run -p xtask -- host-check`: `PASS`;
- included `cargo fmt`, `cargo clippy`, workspace tests, doc-tests and
  `boundary-scan`: `PASS`.

`NARRATIVE-ASYNC-P1`, `DIVINE-JUDGMENT-P1`,
`PANTHEON-CONFLICT-P1`, `DIVINE-ASYNC-P1`, `DIVINE-REPLAY-P1` and the
pantheon-mismatch persistence scenario are Accepted future runtime acceptance
contracts. They remain `NotRun` until the corresponding runtime implementation
exists.

## Rollback

SPEC-31, ADR-029 and ADR-031 are now Accepted. Reversing their semantics
requires a new superseding ADR and synchronized schema/save changes. Optional
LLM routing may be disabled independently; the deterministic template path
remains mandatory.
