# Nonlocal corrected CUDA binary64-pressure-operator contract

| Field | Value |
| --- | --- |
| Research ID | `NCGA7` revision 1 |
| Status | `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY` |
| Parent checkpoint | commit `5b9884d93fbd66bac4ebea56c18e5d44c9f7f2c8`, tree `c932fdeeb957e51a7a1aec1879bc29a98d0760e9` |
| Parent result | NCGA6 `PRESSURE_OPERATOR_PRECISION_REQUIRED` |
| Parent evidence | SHA-256 `6d3a97802f5412a87fd5b7f547fa0925053f6d3da82d6bd27a66cd01355b3e27` |
| Engineering consumer | Decide whether f64 pressure products plus the retained f64 objective close the two static solves with f32 state storage |
| Claim class | Two tiny deterministic mixed-pressure static solves; no trajectory, timing, runtime or product-water authority |

## Frozen inputs and question

NCGA6 source identities are immutable: CUDA assembly
`d4ad4ab41a0cb28201cb2b56e9fb31d674db9b97f4cb8ee2d48e0b91e97020ab`
and driver
`3594e5bdafcfe13adb040d729d94f3507f8a6fb93ba44499b0477cccd19c91b7`.
Its f64-energy result remains insufficient and is the mandatory negative.

NCGA7 asks whether the remaining residual is caused by binary32 composition of
the pressure density, compression, gradient coefficients and exact pressure
Hessian products.

| Hypothesis | Prediction | Falsifier |
| --- | --- | --- |
| f64 pressure products are sufficient | both retained cases reach `R_x<=1e-7`, stay inside state/objective bands and avoid minimum radius | either case still stalls, drifts or loses descent |
| another term/controller is responsible | promoted pressure changes little or the first failure persists | both cases close with unchanged non-pressure arithmetic |
| host combination hides invalid work | independently computed strict-pressure and f64-pressure contributions plus full strict assembly have sealed additive work and permutation identity | unowned calls, host analytical pressure, or order-dependent result |

## Only authorized arithmetic change

Add `AssemblyVariant::F64PressureOperator`, admitted only for a pressure-only
fixture. From device-rounded f32 current positions and f32 profile values
promoted to binary64, one deterministic CUDA thread computes:

1. density and positive compression in canonical row/neighbor order;
2. pressure energy and the pairwise analytical pressure gradient;
3. every active-center `kappa J^T J` product; and
4. every exact radial/tangential pressure geometric-curvature block.

Use the existing f32 `pi` constant promoted to f64, no FMA, precise divide/sqrt,
FTZ off and no floating atomics. Store pressure gradient and dense Hessian as
binary64 outputs. State/reference/predicted coordinates, non-pressure
gradient/Hessian terms and all accepted states remain f32.

The mixed evaluator is an explicit additive composition:

```text
full mixed = full strict-f32 with f64 energy
             - strict-f32 pressure-only contribution
             + f64 pressure-only contribution.
```

All three CUDA evaluator calls and their work are sealed. No host analytical
pressure, f64 viscosity/surface/inertia derivative, regularization, projected
Hessian, tolerance change or further arithmetic variant is allowed.

## Controller and gates

Use the exact NCGA6 common controller and `raw || R_x<=1e-7` success rule.
Reference, strict, f64-energy and f64-energy+pressure paths run in that order.

`GPU_F64_PRESSURE_STATIC_SOLVE_SUPPORTED` requires both pressure-mixed cases:

- finite monotone success before `64` trials/minimum radius;
- final `R_x<=1e-7`, state drift `<=5 um`, objective difference `<=1e-6`
  and reference-identical pressure activity;
- at least one multi-HVP residual path across the corpus;
- progress beyond the NCGA6 energy-only path in acceptance or residual;
- exact canonical/permuted mixed roots;
- rejection of strict, energy-only, f64-pressure-rounded-to-f32, sign-inverted
  HVP and invalid-state controls; and
- complete three-evaluator work receipts, two clean builds/runs, all three
  sanitizers and exact NCGA0--6/NSR1 regressions.

`MIXED_PRESSURE_STATIC_SOLVE_FAILED` is selected when apparatus/controls close
but either case misses a numerical or physical gate. `INCONCLUSIVE` covers
identity, composition, work, control or repeatability failure.

Stop before timing. A positive result authorizes a separately frozen short
boundary-free trajectory using this arithmetic. Failure stops the current
precision ladder and returns to model/controller analysis; it does not
authorize a fourth arithmetic promotion.

## Claim ceiling

Even a positive result proves only two tiny fixed-graph host-controlled static
solves. The serial f64 diagnostic kernel is deliberately non-scalable and says
nothing about trajectory stability, dynamic neighbors, boundaries, GPU-
resident CG, 50k throughput, frame time, runtime integration or game-ready
water.
