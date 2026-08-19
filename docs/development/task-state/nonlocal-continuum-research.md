# Nonlocal continuum research — current task state

| Field | Value |
|---|---|
| Status | `NR1_BASELINE_MISMATCH / NR1-RC1_SPECIFIED / NR2_BLOCKED / REPORT_ONLY` |
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
- The bounded accumulation research selects `nuv-gather-directed-r0`: one
  owner thread reconstructs outgoing plus incoming directed contributions in
  frozen CSR order without floating atomics or pair-fragment storage.

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

### D-NR-005 — Reclose with owner-only directed gather first

- **Decision:** specify `nuv-gather-directed-r0` as NR1-RC1; keep stable
  segmented reduction only as a separately bounded fallback.
- **Reason:** gather is real-arithmetic equivalent over the frozen symmetric
  CSR, removes the shared floating write implicated by NR1 and needs no
  endpoint-fragment capacity. At 48k, a segmented full-matrix fragment layout
  would consume `235.409 MiB` of values plus at least `19.617 MiB` of owner
  keys before sort/reduction scratch, versus `31.209 MiB` for all NR1 buffers.
- **Reconsider when:** CPU scatter/gather algebra mismatches, owner-only output
  still varies, or the correctness-valid adjacent cost rules out a credible
  NR2 route.

## Evidence and sources

| Evidence | Status | Consequence |
|---|---|---|
| [Source audit](../nonlocal-unified-continuum-source-audit-2026-08-19.md) | `PRIMARY_SOURCES_INSPECTED` | formulas/code are sufficient for a bounded experiment; universal-solver claim rejected |
| [Research roadmap](../../plans/nonlocal-continuum/README.md) | `NR1_BASELINE_MISMATCH / NR1-RC1_SPECIFIED / NR2_BLOCKED` | defines NR0–NR4 and preserves the no-credit relationship to W2 |
| [Research contract](../../plans/nonlocal-continuum/00-research-contract.md) | `SPECIFIED` | freezes hypotheses, workloads, measurement scope and terminal states |
| [Baseline/oracle specification](../../plans/nonlocal-continuum/01-source-faithful-baseline-and-oracle.md) | `EXECUTED / BASELINE_MISMATCH` | source-shaped NR1 may not enter NR2 |
| [NR1 evidence](../nonlocal-continuum-nr1-baseline-evidence-2026-08-19.md) | `BASELINE_MISMATCH` | tiny/water/viscous reproduce; surface atomics amplify repeated `f32` order noise beyond state tolerances |
| [Accumulation reclosure research](../nonlocal-continuum-accumulation-reclosure-research-2026-08-19.md) | `GATHER_DIRECTED_SELECTED` | formula closure, CUDA determinism limits and fragment memory select owner-only gather first |
| [NR1-RC1 specification](../../plans/nonlocal-continuum/03-nr1-deterministic-accumulation-reclosure.md) | `SPECIFIED / NOT_STARTED` | freezes candidate identity, exact unchanged dimensions, ordered correctness gates and stop states |
| [Optimization discriminators](../../plans/nonlocal-continuum/02-gpu-optimization-discriminators.md) | `BLOCKED_BY_NR1-RC1` | cannot begin until owner-only gather reclosure passes |
| Pairwise Descent paper/code | `TO_APPEAR / NOT_AUDITABLE` | do not implement or infer formulas |

## Next action

1. Implement only NR1-RC1 `nuv-gather-directed-r0`; do not begin NR2 or widen
   the frozen tolerances.
2. First prove CPU scatter/gather algebra on the eleven tiny cases, then pass
   the CUDA tiny matrix.
3. Require ten byte-identical repeats of the known two-iteration surface-16k
   discriminator before the 20-iteration control or any performance run.
4. Preserve `source-atomic-v0` and all NR1 hashes as the failed source-shaped
   record; do not use its surface timing as evidence.

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
- **Remaining uncertainty:** directed-gather correctness and cost,
  retained NR2 speedup, optimized 48k cutoff and any final NR4 interpretation.
