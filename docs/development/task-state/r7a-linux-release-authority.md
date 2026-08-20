# R7a Linux release authority — task state

| Field | Value |
| --- | --- |
| Status | `IN_PROGRESS / EXACT_LINUX_RERUN_READY` |
| Updated | 2026-08-20 |
| Task key | `r7a-linux-release-authority` |
| Scope | Version `v1-closure` and native-gate so one complete native Linux bundle can publish the current release verdict without a Windows slot |
| Definition of done | Schema-v2 closure and Linux bundle pass strict/fail-closed tests, reject schema v1 as current evidence, and one clean exact Linux commit publishes a complete `PASS / release_ready` bundle |
| Authority | Working context only; ADR-090, SPEC-12 and `docs/roadmap.md` remain normative |

## Resume in 60 seconds

- **Current conclusion:** `v1-closure` and `native-gate-run` now emit strict
  Linux-only schema v2. The schema-v1 Windows/Linux report and comparator remain
  a dormant historical decoder and cannot set the current verdict.
- **Why:** The source audit falsified the initial assumption that the native
  target report was already target-local: schema v1 embedded both closure slots
  and both descriptor roots.
- **Next action:** Commit the bounded module split, then rerun the full native
  Linux gate on that clean exact commit.
- **Current blocker:** None. The first exact run failed only the source-layout
  guard; the offending 1,257-line module is now split into 863- and 401-line
  files.
- **Do not retry:** Do not restore a synthetic Windows slot, relabel schema v1,
  or put Linux-v2 validation back into the oversized monolithic module.
- **Reconsider when:** A future Accepted ADR reintroduces Windows, or a newer
  report schema is required by a concrete release consumer.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| Executable commit `f4833a2eb5264a336fdfcf33afc5743bc8d22949` | Full native gate published strict schema-v2 `FAIL / release_ready=false`; `host-check` reported `SOURCE_FILE_TOO_LARGE` for the 1,257-line `native_gate.rs`; all later checks were typed `NOT_RUN(PRIOR_CHECK_FAILED)` | Fail-closed publication works; this commit is not release-ready and must not be retried unchanged |
| `cargo test -p xtask` on `f4833a2…` | `108 + 44 PASS` | Legacy comparator and new Linux schema behavior pass their complete crate tests |
| Focused post-split checks | Source-size guard, four Linux schema tests, failure bundle publication and clippy all `PASS` | The exact rerun may proceed after committing the split |

## Decisions

### D-001 — Advance both aggregate wire boundaries

- **Observation:** Schema-v1 native target reports contain `closure_targets`
  and `comparable_roots` with mandatory Windows and Linux members.
- **Decision:** Keep schema v1 and its comparator as an explicit legacy path;
  make current `native-gate-run` publish `NativeGateLinuxReportV2` with one
  `release_target`, `release_roots` and `release_ready` verdict.
- **Rejected alternatives:** Silently ignore the Windows member, fabricate a
  Windows result, or reinterpret a schema-v1 report under the new policy.
- **Consequences:** Current and legacy decoders reject each other's wire shape;
  only native Linux x86_64 GNU may produce schema-v2 release evidence.

### D-002 — Keep validation bounded by source layout

- **Observation:** The first exact run found a source-layout violation that
  focused semantic tests did not cover.
- **Decision:** Place Linux-v2 types and validation in `native_gate/linux.rs`;
  keep legacy comparator orchestration in `native_gate.rs`.
- **Consequences:** Both modules remain below the 1,000-line source limit and
  the split mirrors the current-versus-legacy authority boundary.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: one complete clean Linux schema-v2 bundle validates and publishes `PASS / release_ready=true` | Unit, strict-wire, derived-root and failure-publication tests pass; prior Linux platform/package paths are known good | No complete schema-v2 PASS bundle exists yet | Full `native-gate-run` on the post-split exact commit |

## Required context

1. ADR-090 and SPEC-12 release policy
2. `tools/xtask/src/native_gate.rs`, `native_gate/linux.rs` and strict check-report validation
3. Executable commit `f4833a2…` only as negative evidence

## Handoff

- **Workspace state:** Bounded module split plus this task-state update are not
  yet committed.
- **Checks:** Focused source-size/schema/failure tests and clippy `PASS`.
- **Remaining risk:** Complete hardware/package schema-v2 positive path is not
  proven until the next exact clean run.
- **Promotion needed:** After a positive run, update SPEC-12/roadmap and mark
  this state `COMPLETE`, with R7b as the next package.
