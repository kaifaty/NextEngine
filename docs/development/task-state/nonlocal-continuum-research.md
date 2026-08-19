# Nonlocal continuum research — current task state

| Field | Value |
|---|---|
| Status | `NR1_RECLOSED_GATHER_DIRECTED / NR2_UNBLOCKED / O1_NEXT / REPORT_ONLY` |
| Updated | `2026-08-19` |
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
- RC1 adjacent p95 is `2.632 ms` water-16k, `4.630 ms` water-48k and
  `8.693 ms` viscous-16k. Versus the same-binary atomic denominators, the
  water-48k/viscous p95 geometric-mean speedup is `3.116x`, with no additional
  device memory. These are observations, not retained NR2 credit.
- The experiment does not replace DFSPH, change SPEC-38/ADR-076, add GPU
  authority or inherit `CONTINUUM-WATER-REF-P1` credit.
- The standalone CPU `f64` oracle and source-shaped CUDA `f32` baseline live
  under `crates/continuum-water/tools/nonlocal-feasibility`.
- Primary fixed-iteration discriminator: 48k water, five iterations, total p95
  including neighbor construction. `<= 8 ms` is a research cutoff, not the
  existing `4/6 ms` production gate.
- Pairwise Descent remains unavailable as a public paper/code input and is not
  implemented.
- NR2 is unblocked. O1 is next: retain gather correctness while replacing the
  per-iteration device-to-device position handoff with an explicitly audited
  pointer swap; do not skip to O2/O3 or claim NR4 from RC1 timing alone.

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

## Evidence and sources

| Evidence | Status | Consequence |
|---|---|---|
| [Source audit](../nonlocal-unified-continuum-source-audit-2026-08-19.md) | `PRIMARY_SOURCES_INSPECTED` | formulas/code are sufficient for a bounded experiment; universal-solver claim rejected |
| [Research roadmap](../../plans/nonlocal-continuum/README.md) | `NR1_RECLOSED_GATHER_DIRECTED / NR2_UNBLOCKED / O1_NEXT` | defines NR0–NR4 and preserves the no-credit relationship to W2 |
| [Research contract](../../plans/nonlocal-continuum/00-research-contract.md) | `SPECIFIED` | freezes hypotheses, workloads, measurement scope and terminal states |
| [Baseline/oracle specification](../../plans/nonlocal-continuum/01-source-faithful-baseline-and-oracle.md) | `EXECUTED / BASELINE_MISMATCH` | source-shaped NR1 may not enter NR2 |
| [NR1 evidence](../nonlocal-continuum-nr1-baseline-evidence-2026-08-19.md) | `BASELINE_MISMATCH` | tiny/water/viscous reproduce; surface atomics amplify repeated `f32` order noise beyond state tolerances |
| [Accumulation reclosure research](../nonlocal-continuum-accumulation-reclosure-research-2026-08-19.md) | `GATHER_DIRECTED_SELECTED` | formula closure, CUDA determinism limits and fragment memory select owner-only gather first |
| [NR1-RC1 specification](../../plans/nonlocal-continuum/03-nr1-deterministic-accumulation-reclosure.md) | `EXECUTED / NR1_RECLOSED_GATHER_DIRECTED` | freezes and closes candidate identity, unchanged dimensions and ordered gates |
| [NR1-RC1 evidence](../nonlocal-continuum-nr1-rc1-evidence-2026-08-19.md) | `NR1_RECLOSED_GATHER_DIRECTED / NR2_UNBLOCKED` | exact surface repeats support the atomic-order diagnosis; full controls and bounded adjacent costs pass |
| [Optimization discriminators](../../plans/nonlocal-continuum/02-gpu-optimization-discriminators.md) | `UNBLOCKED / O1_NEXT` | begin with pointer-swap/persistent-state O1 from the gather correctness baseline |
| Pairwise Descent paper/code | `TO_APPEAR / NOT_AUDITABLE` | do not implement or infer formulas |

## Next action

1. Implement only NR2 O1 from `nuv-gather-directed-r0`: replace the current
   per-iteration device-to-device `current -> next` handoff with a bounded
   pointer swap while retaining explicit reset/fixture semantics.
2. Run the full O1 adjacent correctness/timing protocol before O2; retain O1
   only if it improves the declared stage/total and introduces no stale state.
3. Preserve `source-atomic-v0`, all original NR1 hashes and the RC1 binary/hash
   boundary. Do not use the invalid atomic surface timing as evidence.
4. Do not infer NR2-SPEEDUP, an NR4 reclosure candidate or production
   authority from the promising RC1 adjacent observation alone.

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

- **Current change:** CPU gather `23a7f73`, CUDA/report reclosure `c3ba007`,
  binary `adb663d7...dc1a`; original source-atomic evidence is unchanged.
- **Executable checks:** build PASS; CPU scatter/gather and CUDA gather tiny
  11/11 PASS; surface two-/twenty-iteration ten-repeat digests exact; water
  16k/48k and viscous 16k controls PASS; adjacent 5+50 timings PASS.
- **Remaining uncertainty:** O1–O6 retained attribution, final fixed-work
  speedup after ordered optimization, profiler counter evidence and NR4
  product interpretation.
