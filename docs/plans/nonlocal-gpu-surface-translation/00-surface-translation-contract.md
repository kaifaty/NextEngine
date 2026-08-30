# NCGP2 surface translation-floor discriminator

| Field | Value |
| --- | --- |
| Research ID | `NCGP2` revision 1 |
| Status | `FROZEN / DISCRIMINATOR_AUTHORIZED / REPORT_ONLY` |
| Frozen parent | `NCGP1 VERIFIED_PHYSICS_REFUTED` at commit `d0679986`; its contract, failure route and evidence are immutable |
| Mathematical parent | FCR0 corrected pressure, viscosity and surface objective |
| Engineering consumer | Select the smallest arithmetic/state representation that can reopen tiny CUDA correspondence without changing the physical model or its tolerances |
| Claim class | Finite translation-dependent numerical mechanism on the retained compressed-pair fixture |

## Question and claim ceiling

NCGP1 failed only after its repaired apparatus reproduced the translated
compressed pair around `x = 0.75 m`: the independent direct/all-pairs
long-double solve succeeded, while corrected and permuted CUDA executions
ended at the same `R_x = 1.50362650553e-5 > 1e-5`. Independent
counterfactuals already show that the same pair around `x = 0.25 m` passes and
that the `x = 0.75 m` pair passes when `gamma = 0`.

This package asks whether the remaining failure is caused primarily by:

1. loss of accepted sub-ULP displacement when global positions/trials are
   repeatedly stored in binary32;
2. binary32 evaluation of the surface distance/spline/gradient/HVP itself; or
3. a solver/globalization mechanism that survives higher-precision surface and
   state counterfactuals.

The package cannot relabel NCGP1, weaken `R_x <= 1e-5`, change the `5 um`
state gate, tune `gamma`, select product water, or authorize performance.
SPEC-38/ADR-076 remain Proposed and CPU DFSPH remains the product fallback.

## Immutable fixture and translation sweep

The retained pair profile is byte-for-byte the NCGP1 compressed-pair profile:

```text
dt      = 1/240 s
spacing = 0.05 m
horizon = 0.15 m
mass    = 0.125 kg
kappa   = 500
lambda  = 2.5
mu      = 1.7
gamma   = 3.5
gravity = 0
rho0    = m * (W(0) + W(0.05)) / 1.1
ids     = [101, 202]
```

For each center

```text
x_c in [0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 1.0, 1.5, 2.0] m
```

the mathematical reference/current coordinates are
`x_c - 0.025` and `x_c + 0.025`, with `y = z = 0.75 m`, then explicitly
round-tripped once through IEEE binary32 before both solvers. There are no
ghosts or active box contacts. Input and permutation roots are sealed per
center. The original `x_c = 0.75` input bytes remain the primary positive;
the other centers are discriminators and may not replace it.

Two controls are mandatory:

- the unchanged corrected profile at `x_c = 0.25`;
- the unchanged `x_c = 0.75` bytes with only `gamma = 0`, reported as a
  causal negative and never accepted as a repaired model.

## Phase A: representation and route observables

Before an arithmetic repair, one new report-only harness runs the unmodified
NCGP1 CPU and CUDA implementations for every center and records:

- exact binary32 coordinate bits and the binary32 ULP at each endpoint;
- encoded pair separation and its error from `spacing`;
- initial surface `q`, `c(q)` and radial gradient in independent long double;
- CPU final displacement from the shared binary32 initial state, per endpoint
  and normalized by that endpoint's ULP;
- the error from rounding the CPU final position once to binary32;
- CUDA failure, `R_x`, gradient norm, outer/accepted/rejected trials, HVP work,
  work root and result root;
- corrected/permuted route, work and result identity.

The harness must not call a CUDA expected-result helper and must not alter the
candidate path. Two fresh executions must be byte-identical after excluding no
field. A malformed center, nonfinite value, changed input root, permutation
loss or work mismatch is `INCONCLUSIVE`.

## Phase B: predeclared arithmetic counterfactuals

Only if Phase A reproduces the primary failure and both controls, exactly two
counterfactuals may be implemented in a separate NCGP2 target:

1. `surface-f64`: global position/trial storage and all solver decisions remain
   binary32/frozen; only surface pair subtraction, radius, normal, spline,
   gradient/HVP products and surface energy use binary64 before one declared
   narrowing to the existing vector output.
2. `compensated-state-f32`: position, predicted position and accepted trial are
   represented as a canonical `(hi, lo)` pair of binary32 vectors. Pair
   differences, inertia displacement and boundary-free trial updates consume
   both parts in a fixed error-free-transform order. Pressure, viscosity,
   surface formulas, trust decisions, reductions, work ceiling and final
   published binary32 position remain otherwise unchanged.

`compensated-state-f32` is a tiny, boundary-free discriminator first. It may
not claim scalable graph/boundary correctness or performance until a successor
contract freezes those semantics. A binary64 full-state implementation is an
optional diagnostic upper bound only and cannot be selected as the candidate
by this revision.

## Falsifiable hypotheses and decision rule

| Hypothesis | Prediction | Falsifier |
| --- | --- | --- |
| H1 global-state quantization floor | failure clusters with absolute-coordinate ULP and CPU accepted corrections that are lost by one binary32 state update; `surface-f64` still fails; `compensated-state-f32` restores the original primary and translation sweep | `surface-f64` alone closes the primary, or compensated state still fails with exact apparatus |
| H2 surface-operator arithmetic floor | CPU corrections are representable but surface force/HVP error grows with translation; `surface-f64` closes the primary and sweep without changing update storage | primary still fails under `surface-f64` while compensated state passes |
| H3 trust/globalization floor | neither predeclared counterfactual closes the primary under the frozen solver route | either counterfactual closes it |
| H4 apparatus/shared-oracle defect | CPU/permutation/input/work identity or controls fail before a physical comparison | exact apparatus and controls reproduce |

Selection is first-specific: H4 gives `INCONCLUSIVE`; otherwise H2 is selected
only if `surface-f64` passes; H1 is selected only if `surface-f64` fails and
`compensated-state-f32` passes; two exact failures select H3 and close this
package `REFUTED / NO_ARITHMETIC_REPAIR_SELECTED`. No blend of both repairs is
allowed in revision 1.

## Numerical gates

A counterfactual passes only when all of the following hold:

- the original `x_c = 0.75` pair reaches `R_x <= 1e-5` after at least one
  accepted trial and differs from the independent CPU final state by at most
  `5 um`;
- every sweep center reaches the same success class and active-pressure
  signature, with corrected/permuted states exact at published binary32;
- the retained NCGP1 graph/operator/solver controls and the `gamma = 0`
  negative remain unchanged;
- FCR0 surface directional derivatives and equal/opposite closure pass;
- no nonfinite, capacity, hidden work, changed HVP ceiling, tolerance change or
  route-specific expected path appears.

If one counterfactual passes, the next package must freeze its scalable storage,
graph, boundary, transaction and cost semantics before any 4k/50k trajectory
or timing run. Performance remains `NOT_RUN` in NCGP2.

## External evidence boundary

NVIDIA's CUDA floating-point documentation is used only for the IEEE binary32,
round-to-nearest, contraction and cancellation mechanism:

- <https://docs.nvidia.com/cuda/cuda-programming-guide/05-appendices/mathematical-functions.html>
- <https://docs.nvidia.com/cuda/pdf/Floating_Point_on_NVIDIA_GPU.pdf>

Those sources do not establish the cause in this solver. Only the frozen local
discriminators above may select H1, H2 or H3.

