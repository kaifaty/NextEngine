# R7a Linux release authority — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE / LINUX_SCHEMA_V2_PASS` |
| Updated | 2026-08-20 |
| Task key | `r7a-linux-release-authority` |
| Scope | Version `v1-closure` and native-gate so one complete native Linux bundle can publish the current release verdict without a Windows slot |
| Definition of done | Schema-v2 closure and Linux bundle pass strict/fail-closed tests, reject schema v1 as current evidence, and one clean exact Linux commit publishes a complete `PASS / release_ready` bundle |
| Authority | Working context only; ADR-090, SPEC-12 and `docs/roadmap.md` remain normative |

## Resume in 60 seconds

- **Current conclusion:** `v1-closure` and `native-gate-run` emit strict
  Linux-only schema v2, and exact clean commit `1e88934…` published a complete
  `PASS / release_ready=true` bundle. The schema-v1 Windows/Linux report and
  comparator remain a dormant historical decoder and cannot set the current
  verdict.
- **Why:** The source audit falsified the initial assumption that the native
  target report was already target-local: schema v1 embedded both closure slots
  and both descriptor roots.
- **Next action:** Start R7b release-package and clean-install closure using the
  schema-v2 Linux report as its release authority.
- **Current blocker:** None for R7a. R7b package/install closure and R7c Linux
  hard-performance authority remain later, separate work packages.
- **Do not retry:** Do not restore a synthetic Windows slot, relabel schema v1,
  or put Linux-v2 validation back into the oversized monolithic module.
- **Reconsider when:** A future Accepted ADR reintroduces Windows, or a newer
  report schema is required by a concrete release consumer.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| Executable commit `1e88934e0d811d90dcf46f6b6f0f27a348a0d9b5` | Full native Linux gate published strict schema-v2 `PASS / release_ready=true`; all eight checks, runtime smoke, desktop smoke and package passed. Target-report SHA-256: `0789954c684455aa119b5962f16f1081328e9acdfdaef29994811915dc8e9204`; closure: `4d14d75bd53acdd4ab1880fa608029a5f8978f0cea121982e87bc84a67b860be`; package descriptor: `da1cc1bfbf8b238a6995b737c140bb78b65fccf5379226905ca33045dddc0685` | Completes R7a and establishes current Linux-only release authority; R7b is next |
| Executable commit `f4833a2eb5264a336fdfcf33afc5743bc8d22949` | Full native gate published strict schema-v2 `FAIL / release_ready=false`; `host-check` reported `SOURCE_FILE_TOO_LARGE` for the 1,257-line `native_gate.rs`; all later checks were typed `NOT_RUN(PRIOR_CHECK_FAILED)` | Fail-closed publication works; this commit is not release-ready and must not be retried unchanged |
| Executable commit `4977bc8780d069c533fc9d7505c438eb44b608c2` | Full native gate published strict schema-v2 `FAIL / release_ready=false`; after the first split, `host-check` found the next `SOURCE_FILE_TOO_LARGE` at the 1,433-line `native_gate/check_reports.rs`; the tail again remained typed `NOT_RUN(PRIOR_CHECK_FAILED)` | This commit is also negative evidence and must not be retried unchanged; focused source-size unit routing was insufficient |
| `cargo test -p xtask` on `f4833a2…` | `108 + 44 PASS` | Legacy comparator and new Linux schema behavior pass their complete crate tests |
| Focused post-split checks | Source-size guard, four Linux schema tests, failure bundle publication and clippy all `PASS` | The exact rerun may proceed after committing the split |
| Standalone `boundary-scan` after the second failure | First found the remaining 1,005-line legacy test fixture before commit; after moving one package diagnostic test, returned `PASS` with all affected files at 989 lines or fewer | Source-layout validation is now complete before the next expensive run |

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

### D-003 — Run the full boundary command before another exact gate

- **Observation:** The focused source-size unit passed while the complete
  `boundary-scan` invoked by host-check still rejected another oversized file.
- **Decision:** A standalone `cargo run --locked -p xtask -- boundary-scan`
  must pass before committing the next exact-run candidate.
- **Consequences:** `check_reports.rs` now retains legacy/common parsing in 987
  lines while Linux-v2 binding lives in a 454-line child module; the expensive
  gate will not be used as the first discriminator for source layout again.

## Resolved hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: one complete clean Linux schema-v2 bundle validates and publishes `PASS / release_ready=true` | Exact commit `1e88934…` published all eight checks, package/runtime/desktop smoke and the aggregate verdict as `PASS` | None | Resolved positively; target-report hash is recorded above |

## Required context

1. ADR-090 and SPEC-12 release policy
2. `tools/xtask/src/native_gate.rs`, `native_gate/linux.rs` and strict check-report validation
3. Executable commit `1e88934…` as positive release-authority evidence;
   `f4833a2…` and `4977bc8…` only as negative fail-closed evidence

## Handoff

- **Workspace state:** Executable R7a candidate is committed at `1e88934…`;
  this state and the roadmap record its exact-run evidence; generated artifacts
  remain outside version control.
- **Checks:** Full exact `native-gate-run` with `desktop-sdl-ash` is `PASS`;
  all eight typed checks, package/runtime/desktop smoke and strict schema-v2
  publication passed on the exact clean commit.
- **Remaining risk:** None within R7a. Reproducible copied install and runtime
  dependency audit belong to R7b; numeric Linux release budgets belong to R7c.
- **Promotion needed:** None for R7a. Continue with R7b.
