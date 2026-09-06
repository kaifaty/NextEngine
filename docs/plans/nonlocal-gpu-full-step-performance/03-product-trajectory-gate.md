# NCGP6 product-oriented trajectory and performance contract

| Field | Value |
| --- | --- |
| Research ID | `NCGP6` revision 1 |
| Status | `FROZEN / CORRECTNESS_FIRST / REPORT_ONLY` |
| User decision | Explicitly authorized on 2026-08-31 after NCGP5 |
| Frozen parent | NCGP5 `SUPPORTED_BOUNDED / NO IMPLEMENTATION REPAIR SELECTED`, source `c134db6279fbb708e3ee4c82bcde87eafd71301d` |
| Architecture status | SPEC-38/ADR-076 remain Proposed; ADR-081 guardrails remain Accepted |
| Engineering consumer | decide whether the corrected standalone Nonlocal GPU solver may proceed to exact 50k timing on RTX 3080 |
| Claim class | finite/profile-bound correspondence, physical adequacy and empirical performance |

## Decision and near-miss firewall

NCGP4 remains honestly failed under its frozen per-particle maximum trajectory
gate: hydrostatic step 92 measured `5.211209258 mm > 5 mm`. NCGP5 found no
same-state formula, HVP, solver-publication or local boundary mismatch on that
witness. It observed one Lagrangian sample reaching the lower wall on adjacent
CPU/GPU steps while the 4,000-sample RMSE and p99 remained small.

The user has authorized a new product-oriented long-horizon gate. NCGP6 does
not change or reinterpret NCGP4/NCGP5. It retains strict same-state tests for
implementation correspondence, but evaluates independently evolved 240-step
fluid trajectories by their error distribution and physical invariants rather
than the single worst stable particle identity after nonlinear contact.

An NCGP6 PASS means only that this standalone Proposed solver is adequate to
enter the declared 50k laboratory timing. It does not make GPU state
authoritative, replace CPU DFSPH, pass an integrated game frame budget or
change any Rust/public/runtime/PhysX/renderer contract.

## Retained model and arithmetic

Retain NCGP4 revision 2 without tuning:

```text
profile_id        = nonlocal-water-50k-v1
dt                = 1/240 s
spacing           = 0.05 m
horizon           = 0.15 m
mass              = 0.125 kg
ghost_layers      = 3
maximum_neighbors = 256
kernel_scale      = 7.985668078772472
kappa             = 1226.25 J
lambda            = 1.4138231728735551e-5 kg m/s
mu                = 0
gamma             = 0.010664424039285813 m/(kg s^2)
solver             = unpreconditioned Steihaug--Toint
total_hvp_budget   = 128
```

Corrected FCR2 pressure, viscosity and surface formulas, canonical binary32
`(hi,lo)` state, pair-aware graph, swept analytical boundaries, fixed
reduction order, binary64 scalar reductions and trust-region rules remain
unchanged. NCGP4 already established that budgets 32 and 64 fail the unchanged
hydrostatic route; NCGP6 therefore evaluates only the retained maximum budget
128 and does not rerun smaller budgets.

## Strict implementation-correspondence gates

Before any long trajectory, the exact NCGP1--NCGP5 profile, graph, boundary,
transaction, corrected energy/gradient/HVP, free-fall, invariance, viscosity
and surface controls must pass on the NCGP6 source. In addition:

- the retained tiny pair/tetrahedron and same-state position maximum remains
  `<= 5 um`;
- HVP relative L2 remains `<= 1e-3`, cosine loss `<= 1e-6`, with identical
  pressure-active IDs on identical state bytes;
- the NCGP5 synchronized pre-step-92 discriminator remains valid, with one
  complete CPU/GPU step maximum `<= 5 um`;
- the first step of each 4k scenario starts from identical canonical binary32
  bytes and must have CPU/GPU position maximum `<= 5 um` plus an identical
  pressure-active signature;
- corrected and stable-ID-permuted GPU state/work roots remain exact at every
  trajectory step.

These gates continue to detect a wrong formula, operator, state update,
contact implementation or solver route. The new distributional rule cannot
waive one of them.

## Product long-trajectory position gate

For every accepted step of each independently evolved 4k CPU/GPU trajectory,
sort the 4,000 Euclidean position errors in nondecreasing order. Define:

```text
RMSE = sqrt(sum(error_i^2) / 4000)
p99  = sorted_errors[ceil(0.99 * 4000) - 1]
     = sorted_errors[3959]                 // zero-based
max  = sorted_errors[3999]
```

The trajectory passes position correspondence only when, at every step:

```text
RMSE <= 2.5 mm
p99  <= 2.5 mm
```

The maximum error, its SampleId/component, p50/p95/p99 and complete sample
record root remain mandatory diagnostics and are sealed into the result. The
maximum is not a long-horizon rejection criterion. There is no particle
removal, outlier deletion, reassignment, retry, alignment or identity change.

This rule is deliberately independent of the measured `5.211 mm` value: the
new p99 limit equals the pre-existing RMSE limit, not the old maximum or the
observed tail. A synthetic corpus with one large error must demonstrate that
the maximum is diagnostic-only; a corpus with 41 errors strictly above
`2.5 mm` must fail specifically through p99 while remaining below the RMSE
limit. RMSE, p99, sample-record and work mutations must change or invalidate
the result.

## Contact and physical adequacy

Stable particle IDs remain necessary for reproducibility and diagnostics, but
the game does not expose the identity of an individual water sample. CPU/GPU
contact with the same analytical face may occur on adjacent accepted steps
after their valid arithmetic trajectories have separated. Such event-time
offset is not itself a failure.

This does not weaken boundary safety. Retain and seal:

- exact corrected/permuted GPU contact masks, impulses and state roots;
- swept-face/edge/corner same-state controls;
- maximum centre penetration `<= 2.5 mm` and closed inset-basin bounds;
- exact particle count and mass;
- density correspondence RMSE/max `<= 5%/10%`;
- compression-density RMSE/max `<= 5%/10%`;
- normalized momentum residual `<= 1%`;
- positive mechanical-energy excess and reversible energy drift `<= 1%`;
- no nonfinite value, capacity overflow, hidden work, permutation loss or
  typed solver/device failure.

Maximum position remains a mandatory reported warning. Any boundary escape,
physical-invariant failure or same-state control failure stops before timing
regardless of RMSE/p99.

## Correctness and performance order

Stop at the first failure:

1. NCGP6 gate self-test and all retained profile/graph/boundary/transaction/
   physics controls;
2. 240-step 4k hydrostatic hold at budget 128;
3. 240-step 4k dam break at budget 128;
4. 240-step 4k orifice jet at budget 128;
5. 16k and exact 50k coherent/permuted/advected correctness and capacity;
6. 240-step exact 50k sealed-basin correctness;
7. two-process exact 50k performance.

Steps 2--4 use the NCGP6 position distribution plus every retained physical
gate. Steps 5--6 additionally require exact capacity/work admission and no
hot-step allocation or full-vector CPU transfer. A failure preserves its first
state and keeps all later stages `NOT_RUN`.

The performance window and target remain NCGP4 revision 1: dynamic graph,
density/active set, energy/gradient/all HVP, trust-region control, boundary and
integration are included; startup, initial allocation, CPU oracle, diagnostic
snapshots, JSON, rendering and PhysX are excluded. Use two fresh processes,
256 conditioning + 32 warmup + 128 measured steps, CUDA events and one
synchronization per sample. Publish nearest-rank p50/p95/p99 with no sample
removal or retry-to-green. Both processes must satisfy:

```text
p95 <= 4 ms
p99 <= 6 ms
```

If performance misses, retain the two already-authorized optimisation cycles:
(1) single-pass neighbor construction plus compatible owner-row fusion;
(2) launch/synchronization reduction, scratch reuse and compatible HVP fusion.
Physics, workload, gates and tolerances cannot change inside those cycles.

## Negative controls and evidence closure

Retain every prior formula, graph, boundary, transaction, permutation,
capacity, identity and work mutation. Add:

- old long-horizon maximum-gate comparator, which must still reject the frozen
  NCGP4 step-92 record;
- one-tail synthetic position set, which must pass RMSE/p99 while retaining a
  maximum warning;
- 41-tail synthetic position set, which must fail p99 before timing;
- changed quantile rank, omitted p99, sample-record mutation and gate-work
  mutation controls.

The versioned report seals contract/profile/input/source/tree/binary/compiler/
environment roots, exact command, all gate values, nearest-rank index, first
failure, work receipts, state roots, transfer/allocation counts and timing
status. Two clean Release builds/runs, Compute Sanitizer
memcheck/initcheck/synccheck and one independent read-only review are required
before a final positive result. One batched repair and one re-review are
allowed; a remaining load-bearing defect closes NCGP6 `INCONCLUSIVE`.

## Stop and promotion boundary

- A failed same-state or invariant gate is not rescued by RMSE/p99.
- A failed 4k/16k/50k correctness stage blocks performance.
- The NCGP4 max-gate failure remains a valid historical result.
- Do not raise 128 HVP, select pressure-f64, tune coefficients/tolerances,
  localize host coordinates or report neighbor-only time as full-solver time.
- Even a `4/6 ms` PASS is standalone RTX-3080 feasibility only. CPU DFSPH
  remains the product fallback and integrated game-frame work remains a later
  consumer-backed decision.
