# Nonlocal continuum research — current task state

| Field | Value |
|---|---|
| Status | `NR0_SPECIFIED / NR1_IMPLEMENTATION_NEXT / REPORT_ONLY` |
| Updated | `2026-08-19` |
| Task key | `nonlocal-continuum-research` |
| Scope | Source-faithful Nonlocal/SISSM baseline and bounded GPU optimization research |
| Definition of done | One NR4 decision state selected from reproducible correctness and performance evidence; no runtime/public-contract claim |
| Authority | Working context only; Accepted SPEC/ADR and current DFSPH roots outrank this file |

## Resume in 60 seconds

- Branch/worktree: `codex/nonlocal-continuum-n0` at
  `/home/kaifaty/Documents/NextEngine-nonlocal-continuum-n0`, based on merged
  continuum checkpoint `fe223f9`.
- NR0 is documentation-only. The research contract, source audit, baseline
  specification and optimization ladder are frozen for implementation review.
- The experiment does not replace DFSPH, change SPEC-38/ADR-076, add GPU
  authority or inherit `CONTINUUM-WATER-REF-P1` credit.
- NR1 implements a standalone CPU `f64` oracle plus source-shaped CUDA `f32`
  baseline under `crates/continuum-water/tools/nonlocal-feasibility`.
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

## Evidence and sources

| Evidence | Status | Consequence |
|---|---|---|
| [Source audit](../nonlocal-unified-continuum-source-audit-2026-08-19.md) | `PRIMARY_SOURCES_INSPECTED` | formulas/code are sufficient for a bounded experiment; universal-solver claim rejected |
| [Research roadmap](../../plans/nonlocal-continuum/README.md) | `BOUNDED_SPIKE_SELECTED` | defines NR0–NR4 and no-credit relationship to W2 |
| [Research contract](../../plans/nonlocal-continuum/00-research-contract.md) | `SPECIFIED` | freezes hypotheses, workloads, measurement scope and terminal states |
| [Baseline/oracle specification](../../plans/nonlocal-continuum/01-source-faithful-baseline-and-oracle.md) | `NOT_STARTED` | next implementation boundary |
| [Optimization discriminators](../../plans/nonlocal-continuum/02-gpu-optimization-discriminators.md) | `NOT_STARTED` | begins only after NR1 baseline reproduction |
| Pairwise Descent paper/code | `TO_APPEAR / NOT_AUDITABLE` | do not implement or infer formulas |

## Next action

1. Create the standalone tool skeleton and exact machine-readable profile
   record without importing PeriDyno.
2. Implement the CPU `f64` pair/one-iteration oracle and make every tiny case
   pass before CUDA timing work.
3. Implement the source-shaped fixed-iteration CUDA baseline and bind its
   report schema/self-test.
4. Run only bounded NR1 controls; do not begin NR2 until the checked-in NR1
   evidence summary records `BASELINE_REPRODUCED`.

## Do not retry or infer

- direct APG graph port or scheduler-only tuning as a 4 ms solution;
- full PeriDyno integration;
- lower precision without the oracle matrix;
- position-delta-only convergence;
- hidden warm-start state;
- Pairwise Descent without a public primary source;
- plasticity, temperature, phase or solid claims from the fluid paper;
- W3 coupling, persistence, public contracts or ProductCheck promotion.

## Handoff

- **Current change:** documentation/specification only.
- **Executable checks:** `NotRun(NoExecutableChange)`.
- **Remaining uncertainty:** formula reproduction, numerical tolerances,
  baseline bottleneck attribution, fixed-iteration speedup and both scale gates
  have no local evidence yet.
