# Nonlocal corrected CUDA binary64-energy globalization contract

| Field | Value |
| --- | --- |
| Research ID | `NCGA6` revision 1 |
| Status | `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY` |
| Parent checkpoint | commit `3e3e4107cc1936c78246c490c4d246163b9584b1`, tree `1ddf62cd6407d3420e8213b012d3774089930462` |
| Parent result | NCGA5 `AUTHOR_REFUTED / F32_STATIC_SOLVE_MATERIAL` |
| Parent evidence | SHA-256 `52f716f952270b9b3464ff7b7088861c8fedb548be55166076d6f57a9f43bb49` |
| Engineering consumer | Decide whether binary64 objective evaluation alone closes the demonstrated f32 trust-globalization floor before pressure-operator promotion |
| Claim class | Two tiny fixed-graph mixed-energy static solves; no trajectory, timing, runtime or product-water authority |

## Frozen inputs

| Input | SHA-256 |
| --- | --- |
| NCGA5 driver/controller | `6e079c3326cb0b4d97e86f09461729a9b7c12c800250c820a577586f19a06299` |
| strict CUDA assembly | `961bd9fdb0759fa188a4e88531abd544b23cc19ac256d3a276be2bb02ecce72a` |
| assembly public header | `919c011b8e9565d7ed0ffdec4fcf2d39b1e6a58d0c7305896ef09bab276cfdcc` |
| NCGA5 accepted raw report | `fa1e5f059fd60aeed6e5ad67094c9c5aa9c17dc1b7444c23491bf990068bc03c` |

NCGA5 remains refuted. This experiment neither widens its gates nor relabels
its strict-f32 result.

## Question and hypotheses

| ID | Hypothesis | Prediction | Falsifier |
| --- | --- | --- | --- |
| H1 | f32 objective resolution is the first full-solve boundary | recomputing only the four objective scalars in device binary64 restores positive actual reduction and both cases reach the frozen scale-aware stop | mixed energy still contracts to minimum radius or misses state/objective/residual bands |
| H2 | pressure gradient/Hessian precision is also required | f64 energy fixes reduction sign but the unchanged f32 operator stalls above the residual band or drifts materially | energy-only closes both cases |
| H3 | a changed stopping rule could masquerade as an arithmetic fix | strict-f32 under the same scale-aware controller remains outside the success gate, and at least one mixed trial is accepted where strict f32 rejects on actual-reduction sign | strict f32 passes identically or no decision is attributable to the f64 energy |
| H4 | a host oracle could hide the cost/identity | the selected objective is computed by an independent CUDA kernel from device-rounded inputs with sealed f64 work | host energy is substituted or f64 work is unowned |

## Only authorized arithmetic change

Add `AssemblyVariant::F64Energy`. The existing strict kernel still computes
density, pressure activity, gradient, exact dense Hessian, diagonal blocks and
HVP in compensated strict f32. State storage, profile coefficients and all
reference/predicted/current coordinates are exactly the binary32 values already
uploaded by NCGA5.

After that strict assembly, one deterministic CUDA thread recomputes only the
four total objective components in binary64, in this fixed order:

1. particles by canonical ascending ID for inertia and pressure;
2. unique current pairs `(i,j), i<j` for surface;
3. unique reference pairs `(i,j), i<j` for viscosity;
4. objective components in inertia, pressure, viscosity, surface order.

The kernel uses the device-rounded binary32 profile and positions promoted
exactly to binary64. Its cubic-kernel `pi` is the existing binary32 constant
promoted to binary64, so the experiment changes operation precision rather
than constants. Compile with FMA disabled, precise divide/sqrt and FTZ off. No
floating atomic, host objective, compensated f32 subtotal, long-double value,
pressure-gradient promotion or Hessian replacement is allowed.

The old variants and entry points must emit their exact retained stdout hashes.
Report and seal f64 density terms, unique energy-pair visits, scalar products,
component reductions and output replacements separately from inherited strict
assembly work.

## Controller and convergence

Use the unchanged NCGA5 Steihaug--Toint controller and two exact cases. Run
three paths in order: independent reference, strict-f32 negative, then
`F64Energy`. Both candidate paths use the same success rule:

```text
raw || scale-aware success := ||g||_2 <= 1e-10 || R_x <= 1e-7.
```

`R_x<=1e-7` is the already frozen NCGA5 positive candidate band; it is not
derived from the failed values. The stop is checked only at the start of an
outer transaction. It does not accept a state that is nonfinite, nonmonotone,
outside the state/objective bands or reached by an invalid trial.

All other radius, residual, acceptance, f32-state rounding, maximum-work and
active-set rules remain NCGA5 exact. No numerical-floor success, line search,
retry, regularization, warm state or pressure-product promotion is allowed.

## Gates and classifications

`GPU_F64_ENERGY_STATIC_SOLVE_SUPPORTED` requires:

1. the repaired NCGA5 independent references remain valid;
2. both mixed-energy cases reach raw or `R_x<=1e-7` success before `64` trials
   and before the minimum radius, with finite strictly decreasing accepted
   objectives and no active-set change;
3. final maximum particle drift from reference is `<=5 um`, objective
   difference is `<=1e-6`, and final active roots match;
4. at least one multi-HVP `RESIDUAL` path remains exercised;
5. at least one mixed candidate trial is accepted at an ordinal where the
   same-state strict path rejects because its actual reduction is non-positive;
6. strict f32 under the common success rule does not pass both cases;
7. canonical/permuted mixed inputs are bit-identical; the first-HVP sign
   inversion, f64-energy-rounded-back-to-f32, invalid-state and state-ownership
   controls are rejected; and
8. work, two clean builds/runs, sanitizers and retained NCGA0--5/NSR1
   regressions close exactly.

`PRESSURE_OPERATOR_PRECISION_REQUIRED` is selected when apparatus and controls
pass, f64 energy removes or delays the wrong-sign reduction boundary, but a
case still misses convergence/state/objective/active gates with the unchanged
f32 gradient/Hessian. `F64_ENERGY_NOT_CAUSAL` is selected when the mixed path
does not change the first failing transaction. `INCONCLUSIVE` covers identity,
independence, work, control, repeatability or apparatus failure.

## Ordered execution and next branch

1. Reproduce the NCGA5 strict report and retained reference shapes.
2. Run strict and mixed candidates with the common scale-aware controller.
3. Run permutation and negative controls.
4. Build/run twice clean, then sanitizers and retained regressions.

Stop before timing. A positive result authorizes a separately frozen short
boundary-free trajectory using the selected mixed-energy arithmetic. A
`PRESSURE_OPERATOR_PRECISION_REQUIRED` result authorizes one f64 pressure
coefficient/product discriminator. No other result authorizes trajectory work.

## Claim ceiling

Even a positive result proves only that a deterministic GPU binary64 objective
can globalize two tiny static f32-operator solves. It does not establish
physical trajectories, dynamic neighborhoods, boundaries/contacts, a
GPU-resident Krylov solve, matrix-free scaling, 50k cost, frame time, runtime
integration or game-ready water.
