# Nonlocal continuum research — current task state

| Field | Value |
|---|---|
| Status | `NR1_BASELINE_MISMATCH / NR2_BLOCKED / REPORT_ONLY` |
| Updated | `2026-08-19` |
| Task key | `nonlocal-continuum-research` |
| Scope | Source-faithful Nonlocal/SISSM baseline and bounded GPU optimization research |
| Definition of done | One NR4 decision state selected from reproducible correctness and performance evidence; no runtime/public-contract claim |
| Authority | Working context only; Accepted SPEC/ADR and current DFSPH roots outrank this file |

## Resume in 60 seconds

- Branch/worktree: `codex/nonlocal-continuum-n0` at
  `/home/kaifaty/Documents/NextEngine-nonlocal-continuum-n0`, based on merged
  continuum checkpoint `fe223f9`.
- NR1 is implemented at `107839b`; the independent CPU and CUDA tiny oracle
  passes all 11 cases. Water 16k/48k and viscous 16k fixed controls pass.
- The stiff surface 16k control is finite but fails repeated-output gates from
  iteration two despite byte-identical neighbor CSR. NR1 therefore records
  `BASELINE_MISMATCH`, not `BASELINE_REPRODUCED`.
- The experiment does not replace DFSPH, change SPEC-38/ADR-076, add GPU
  authority or inherit `CONTINUUM-WATER-REF-P1` credit.
- The standalone CPU `f64` oracle and source-shaped CUDA `f32` baseline live
  under `crates/continuum-water/tools/nonlocal-feasibility`.
- Primary fixed-iteration discriminator: 48k water, five iterations, total p95
  including neighbor construction. `<= 8 ms` is a research cutoff, not the
  existing `4/6 ms` production gate.
- Pairwise Descent remains unavailable as a public paper/code input and is not
  implemented.

## Decisions

### D-NR-001 — Quarantined standalone tool

- **Decision:** no PeriDyno dependency or engine integration; reimplement the
  published equations in a report-only standalone tool.
- **Reason:** isolate numerical feasibility from framework/plugin/runtime cost
  and preserve existing ownership.
- **Reconsider when:** NR4 selects a reclosure candidate and a consumer-backed
  architecture decision is drafted.

### D-NR-002 — Fixed work before adaptive work

- **Decision:** NR1/NR2 performance uses fixed term sets and iteration counts.
  Adaptive exit, warm starts and alternative optimizers have separate profile
  IDs and cannot improve the fixed-work claim by relabelling it.
- **Reason:** distinguish implementation speedup from doing less/different work.
- **Reconsider when:** the fixed baseline is correct and a residual/failure
  contract is independently frozen.

### D-NR-003 — Three terminal product interpretations

- **Decision:** evidence may recommend a new 48k reclosure, a smaller local
  high-fidelity consumer, or a research stop. None is automatic promotion.
- **Reason:** preserve the 50k gate while still testing the formulation's
  strongest material-coupling use case.

### D-NR-004 — Preserve the failed atomic baseline

- **Decision:** record `source-atomic-v0` as `BASELINE_MISMATCH` and block NR2;
  do not relabel deterministic gather/segmented accumulation as the same NR1
  implementation.
- **Reason:** the 20-iteration surface profile exceeds repeated position and
  velocity bounds from iteration two while fixtures and CSR remain identical.
- **Reconsider when:** a committed reclosure gives the remediation a separate
  identity and requires surface correctness before timing.

## Evidence and sources

| Evidence | Status | Consequence |
|---|---|---|
| [Source audit](../nonlocal-unified-continuum-source-audit-2026-08-19.md) | `PRIMARY_SOURCES_INSPECTED` | formulas/code are sufficient for a bounded experiment; universal-solver claim rejected |
| [Research roadmap](../../plans/nonlocal-continuum/README.md) | `NR1_BASELINE_MISMATCH / NR2_BLOCKED` | defines NR0–NR4 and preserves the no-credit relationship to W2 |
| [Research contract](../../plans/nonlocal-continuum/00-research-contract.md) | `SPECIFIED` | freezes hypotheses, workloads, measurement scope and terminal states |
| [Baseline/oracle specification](../../plans/nonlocal-continuum/01-source-faithful-baseline-and-oracle.md) | `EXECUTED / BASELINE_MISMATCH` | source-shaped NR1 may not enter NR2 |
| [NR1 evidence](../nonlocal-continuum-nr1-baseline-evidence-2026-08-19.md) | `BASELINE_MISMATCH` | tiny/water/viscous reproduce; surface atomics amplify repeated `f32` order noise beyond state tolerances |
| [Optimization discriminators](../../plans/nonlocal-continuum/02-gpu-optimization-discriminators.md) | `BLOCKED_BY_NR1` | cannot begin under the current baseline gate |
| Pairwise Descent paper/code | `TO_APPEAR / NOT_AUDITABLE` | do not implement or infer formulas |

## Next action

1. Do not begin NR2 under the current contract or widen the frozen tolerances.
2. If research continues, reclose a separate deterministic gather/segmented
   accumulation remediation identity while retaining `source-atomic-v0` as the
   failed denominator.
3. Require that candidate to pass the existing tiny matrix and the
   two-iteration surface discriminator before any performance run.

## Do not retry or infer

- direct APG graph port or scheduler-only tuning as a 4 ms solution;
- full PeriDyno integration;
- lower precision without the oracle matrix;
- another source-atomic surface run as a correctness remedy;
- position-delta-only convergence;
- hidden warm-start state;
- Pairwise Descent without a public primary source;
- plasticity, temperature, phase or solid claims from the fluid paper;
- W3 coupling, persistence, public contracts or ProductCheck promotion.

## Handoff

- **Current change:** CPU oracle `5e77bbc`, CUDA baseline `107839b`, bounded NR1
  evidence and task/roadmap transition.
- **Executable checks:** build PASS; CPU/CUDA self-test PASS; water 16k/48k and
  viscous 16k 5+50 benchmarks PASS; surface 16k benchmark FAIL as recorded;
  Nsight attribution complete.
- **Remaining uncertainty:** deterministic accumulation correctness and cost,
  retained NR2 speedup, optimized 48k cutoff and any final NR4 interpretation.
