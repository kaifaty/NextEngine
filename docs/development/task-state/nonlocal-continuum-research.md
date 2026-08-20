# Nonlocal continuum research — current task state

| Field | Value |
|---|---|
| Status | `NR2_O2_RETAINED / O3_SPECIFIED / IMPLEMENTATION_NEXT / REPORT_ONLY` |
| Updated | `2026-08-20` |
| Task key | `nonlocal-continuum-research` |
| Scope | Source-faithful Nonlocal/SISSM baseline and bounded GPU optimization research |
| Definition of done | One NR4 decision state selected from reproducible correctness and performance evidence; no runtime/public-contract claim |
| Authority | Working context only; Accepted SPEC/ADR and current DFSPH roots outrank this file |

## Resume in 60 seconds

- Branch/worktree: `codex/nonlocal-continuum-n0` at
  `/home/kaifaty/Documents/NextEngine-nonlocal-continuum-n0`, based on merged
  continuum checkpoint `fe223f9`.
- The original NR1 source-atomic record at `107839b` remains
  `BASELINE_MISMATCH`. Its tiny/water/viscous controls pass, while stiff
  surface varies from iteration two despite byte-identical CSR.
- NR1-RC1 is implemented at `c3ba007` as the separate
  `nuv-gather-directed-r0` identity. CPU scatter/gather and CUDA tiny pass
  11/11; surface-16k produces one exact output digest across ten cold repeats
  at both two and twenty iterations; all full controls pass.
- O1 is implemented at `b192f0a` and finalized at `d7ef06a` as the explicit
  `pointer-swap-o1` handoff with the legacy/self-test gate correctly scoped.
  It is exact against `copy-v0` across tiny, stiff-surface, odd/even reused
  instances and all full controls; memory remains `10,914,054 B` at 16k and
  `32,725,766 B` at 48k.
- Adjacent O1 total p95 is `2.503 ms` water-16k, `4.626 ms` water-48k and
  `8.313 ms` viscous-16k. Handoff p95 decreases on all profiles; total p95
  decreases `12.105%` water-48k and `7.505%` viscous-16k, retaining O1 with a
  `1.1091x` HN-3 geometric-mean speedup.
- O2 is specified at `e3f6af5` and implemented at `053e1e7` as
  `nuv-terms-specialized-o2`. It is exact against runtime term dispatch across
  tiny masks, stiff surface and all full controls, with unchanged memory.
  Required adjacent total p95 is `1.903 ms` water-16k, `3.595 ms` water-48k
  and `6.608 ms` viscous-16k; O2-only HN-3 geometric-mean speedup is `1.0635x`.
- Same-process Nsight Systems attribution reports `5.886%` lower water-48k
  and `1.539%` lower viscous-16k specialised viscosity-launch mean. Nsight
  Compute counters are explicitly unavailable under `ERR_NVGPUCTRPERM`; no
  occupancy or throughput is inferred.
- The experiment does not replace DFSPH, change SPEC-38/ADR-076, add GPU
  authority or inherit `CONTINUUM-WATER-REF-P1` credit.
- The standalone CPU `f64` oracle and source-shaped CUDA `f32` baseline live
  under `crates/continuum-water/tools/nonlocal-feasibility`.
- Primary fixed-iteration discriminator: 48k water, five iterations, total p95
  including neighbor construction. `<= 8 ms` is a research cutoff, not the
  existing `4/6 ms` production gate.
- Pairwise Descent remains unavailable as a public paper/code input and is not
  implemented.
- O2 is retained. O3 is specified as a three-layout observation with only
  `nuv-unique-pair-segmented-o3 + pointer-swap-o1 +
  nuv-terms-specialized-o2` retainable against gather. It uses one `48 B`
  endpoint fragment plus one `4 B` reverse index per capacity slot, caps the
  48k candidate at `339,733,770 B`, and requires same-process rotating timing.

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
- **Closure:** RC1 satisfies the reconsideration condition without changing
  the old record. `source-atomic-v0` stays failed for stiff surface and remains
  an HN-3 denominator only on its correctness-passing profiles.

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
- **Closure:** none occurred. All ordered gates pass, no pair-fragment/device
  array was added, and adjacent timing improves rather than rejects the route.

### D-NR-006 — Admit gather as the NR2 correctness baseline

- **Decision:** exit `NR1_RECLOSED_GATHER_DIRECTED`, unblock NR2 and start at
  O1 with `nuv-gather-directed-r0` as the correctness-valid baseline.
- **Reason:** independent CPU algebra, 11/11 CUDA tiny fixtures, exact stiff
  surface repeats and the three full controls all pass unchanged gates.
- **Constraint:** RC1 speedups do not become retained NR2 or NR4 evidence until
  the ordered ladder and required profiler captures are complete.

### D-NR-007 — Retain pointer-swap handoff

- **Decision:** retain `pointer-swap-o1` as the O2 input while keeping
  `copy-v0` explicitly selectable as the adjacent rollback baseline.
- **Reason:** all stale-state and numeric gates pass exactly, memory is
  unchanged, handoff p95 decreases on every profile and total p95 improves on
  both HN-3 denominators.
- **Constraint:** the `1.1091x` adjacent geometric-mean speedup is O1-only. It
  grants no aggregate NR2, profiler, NR4, runtime, W2 or production credit.

### D-NR-008 — Retain compile-time viscosity specialization

- **Decision:** retain `nuv-terms-specialized-o2` as the O3 input while keeping
  `nuv-terms-runtime-v0` explicitly selectable as the rollback comparator.
- **Reason:** the complete correctness matrix is exact, memory is unchanged,
  the frozen adjacent p95 rules pass and same-process profiler attribution
  confirms lower viscosity-launch mean on water-48k and viscous-16k.
- **Constraint:** short-process timing is sensitive to GPU clock state; only
  the viscosity-kernel attribution is credited. The `1.0635x` adjacent HN-3
  result is O2-only and grants no aggregate NR2, NR4, W2 or production credit.

## Evidence and sources

| Evidence | Status | Consequence |
|---|---|---|
| [Source audit](../nonlocal-unified-continuum-source-audit-2026-08-19.md) | `PRIMARY_SOURCES_INSPECTED` | formulas/code are sufficient for a bounded experiment; universal-solver claim rejected |
| [Research roadmap](../../plans/nonlocal-continuum/README.md) | `NR2_O2_RETAINED_TERM_SPECIALIZATION / O3_NEXT` | defines NR0–NR4 and preserves the no-credit relationship to W2 |
| [Research contract](../../plans/nonlocal-continuum/00-research-contract.md) | `SPECIFIED` | freezes hypotheses, workloads, measurement scope and terminal states |
| [Baseline/oracle specification](../../plans/nonlocal-continuum/01-source-faithful-baseline-and-oracle.md) | `EXECUTED / BASELINE_MISMATCH` | source-shaped NR1 may not enter NR2 |
| [NR1 evidence](../nonlocal-continuum-nr1-baseline-evidence-2026-08-19.md) | `BASELINE_MISMATCH` | tiny/water/viscous reproduce; surface atomics amplify repeated `f32` order noise beyond state tolerances |
| [Accumulation reclosure research](../nonlocal-continuum-accumulation-reclosure-research-2026-08-19.md) | `GATHER_DIRECTED_SELECTED` | formula closure, CUDA determinism limits and fragment memory select owner-only gather first |
| [NR1-RC1 specification](../../plans/nonlocal-continuum/03-nr1-deterministic-accumulation-reclosure.md) | `EXECUTED / NR1_RECLOSED_GATHER_DIRECTED` | freezes and closes candidate identity, unchanged dimensions and ordered gates |
| [NR1-RC1 evidence](../nonlocal-continuum-nr1-rc1-evidence-2026-08-19.md) | `NR1_RECLOSED_GATHER_DIRECTED / NR2_UNBLOCKED` | exact surface repeats support the atomic-order diagnosis; full controls and bounded adjacent costs pass |
| [Optimization discriminators](../../plans/nonlocal-continuum/02-gpu-optimization-discriminators.md) | `O2_RETAINED_TERM_SPECIALIZATION / O3_NEXT` | continue in order from the exact gather/swap/specialized candidate |
| [O1 specification](../../plans/nonlocal-continuum/04-nr2-o1-pointer-swap.md) and [evidence](../nonlocal-continuum-nr2-o1-evidence-2026-08-20.md) | `EXECUTED / O1_RETAINED_POINTER_SWAP` | exact reused/copy correspondence, unchanged memory and adjacent retention gates pass |
| [O2 specification](../../plans/nonlocal-continuum/05-nr2-o2-term-specialization.md) and [evidence](../nonlocal-continuum-nr2-o2-evidence-2026-08-20.md) | `EXECUTED / O2_RETAINED_TERM_SPECIALIZATION` | exact runtime/specialized correspondence, unchanged memory, adjacent gates and same-process profiler attribution pass; counters unavailable explicitly |
| [O3 specification](../../plans/nonlocal-continuum/06-nr2-o3-accumulation-layout-tournament.md) | `SPECIFIED / IMPLEMENTATION_NEXT` | freezes unique-pair endpoint fragments, reverse-map/work proofs, byte ceilings, rotating three-layout timing and a `1.10x` memory-cost retention threshold |
| Pairwise Descent paper/code | `TO_APPEAR / NOT_AUDITABLE` | do not implement or infer formulas |

## Next action

1. Implement only the specified segmented identity, reverse-map validation,
   capacity/work reporting and same-process tournament command.
2. Run tiny, stiff-surface and full correspondence before any tournament
   timing; on numeric mismatch preserve gather and stop changing association.
3. Preserve runtime term dispatch, copy handoff, source-atomic, RC1, O1 and O2
   evidence boundaries. Do not add their non-adjacent percentages.
4. Keep O3 report-only; do not infer NR2-SPEEDUP, NR4, W2 or production
   authority from an implementation or timing result.

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

- **Current change:** O3 contract freezes one unique-pair/CSR-segmented
  candidate, exact memory/work gates and rotating same-process timing; O2
  binary `ae444274...c389` remains the implementation baseline.
- **Executable checks:** build, both legacy tiny identities and specialised
  tiny 11/11 PASS; invalid identity combinations reject; surface i2/i20
  ten-repeat, reused-instance, runtime correspondence and all full controls
  are exact; adjacent 5+50 timing gates PASS.
- **Profiler:** Nsight Systems same-process viscosity attribution is complete;
  Nsight Compute counter collection is explicitly unavailable under
  `ERR_NVGPUCTRPERM` and recorded by two hashed command logs.
- **Remaining uncertainty:** O3 implementation/correctness/cost, O4–O6 retained attribution, final fixed-work
  speedup after ordered optimization, privileged counter evidence and NR4
  product interpretation.
