# NCGP5 hydrostatic step-92 outlier diagnosis

| Field | Value |
| --- | --- |
| Research ID | `NCGP5` revision 1 |
| Status | `FROZEN / DIAGNOSIS_ONLY / REPORT_ONLY` |
| Parent result | NCGP4 `REFUTED_BOUNDED`, source `08036770046f3f937221712f7ba554be38b2b454` |
| Exact witness | hydrostatic hold, unpreconditioned, budget 128, first `position_max` failure on step 92 |
| Engineering consumer | decide whether one implementation/semantic repair is justified before any new correctness or performance attempt |

## Exact question and claim ceiling

Why does one GPU sample exceed the independent CPU trajectory by
`5.211209258 mm` on hydrostatic step 92 when the same run has position RMSE
`0.238779 mm`, density correspondence max `1.01934%`, momentum residual
`0.238287%`, no penetration, no solver failure and exact coherent/permuted GPU
state identity?

NCGP5 may identify a concrete mismatch and authorize one smallest repair. It
does not reopen NCGP4, loosen the 5 mm gate, change physics, tune tolerances,
increase 128 HVP or run 50k performance. If the witness is the expected
sensitivity of two admitted arithmetic trajectories rather than a candidate or
oracle defect, the result remains `REFUTED_BOUNDED` and a later product-level
contract decision is required before any different trajectory gate.

## Retained identity

Retain exactly the NCGP4 revision-2 physical profile, compensated GPU state,
unpreconditioned Steihaug--Toint rules, CPU long-double oracle, basin ghosts,
hydrostatic input, 128-HVP ceiling and all numerical gates. Replay from the
initial root; a checkpoint synthesized by a different route is not evidence.

The NCGP4 witness has raw stdout SHA-256
`a3aca468e8eb5bef283db65e4891dc280970f50fa35596e57da6afd2ee6b4d05`,
result root
`e9f888f0a611e5985b8d3d2d79323d1699186af7e2ff8362c42f5eb6f963cc16`
and coherent/permuted compensated state root
`0a308959b3f5fda9e9c6146e597dbd72e26256d8bf1f7da73032ed8e5ffb011f`.

## Competing hypotheses

| ID | Hypothesis | Prediction | Falsifier |
| --- | --- | --- | --- |
| H5A | a pressure active-set bifurcation accumulates from the first CPU/GPU signature difference on step 8 | the eventual outlier or its neighbors repeatedly straddle `rho=rho0`; the error growth changes slope at an active transition, while same-state operators remain within gates | the outlier and neighborhood stay far from the threshold, or same-state operator/one-step correspondence fails independently |
| H5B | analytical contact or boundary semantics diverge locally | the outlier approaches an inset face and CPU/GPU face/contact histories or impulse differ before the position jump | the outlier remains interior with no face hit and matching boundary records |
| H5C | compensated predictor or GPU operator has a local semantic defect | reconstructing the exact hi+lo pre-step state exposes an above-gate gradient/HVP or one-step error, with one pressure/surface/inertia component dominating | synchronized-state operator and one-step outputs remain within frozen gates |
| H5D | CPU oracle and GPU trust-region routes implement different stopping/publication semantics | accepted/rejected/radius/HVP history first differs before the state error jump even when fed the same semantic state | synchronized solver records and published state agree within the tiny gate |
| H5E | the two valid arithmetic trajectories are simply sensitive over 92 nonlinear steps | error grows smoothly, no local branch/semantic discrepancy is found, same-state operator and one-step comparisons pass, and the max is a tail outlier while RMSE remains small | a reproducible local code/semantic mismatch explains the outlier |

## Frozen diagnostic

Add one NCGP5-only diagnostic route. Replay corrected and ID-permuted GPU plus
the independent CPU oracle from the exact initial input through step 92. Seal
every per-step work/result and complete compensated checkpoint root. For steps
80--92 record, in ascending SampleId order before reduction:

- per-sample position and velocity error, density difference, CPU/GPU active
  flags, inset-face distance and GPU contact face mask/impulse ownership;
- exact maximum-error SampleId/component, its reference/current/predicted/
  velocity hi+lo parts and CPU state;
- p50/p95/p99/max position error by nearest rank, not only RMSE;
- GPU/CPU HVP used, outer/accepted/rejected trials, radius changes, active
  counts and graph degree/pair counts;
- the outlier's current-neighbor ID root and which neighbor centers are active.

At the last pre-failure step and at the first step where the max-error slope
changes by more than `2x`, run two discriminators:

1. reconstruct a common long-double state from the captured GPU hi+lo pairs
   and compare the GPU gradient/HVP and active signature against an independent
   evaluation on those exact values;
2. execute one CPU and GPU step from one canonical synchronized public state,
   recording trust-region decisions and the published outlier error.

Diagnostic full-vector transfers are outside any hot window. Event order,
sample records, work, input/source/binary/environment identities and aggregate
roots are mandatory. A sample-record mutation, neighbor-root mutation and work
mutation must change or invalidate the result.

## Decision rule

- Select one smallest repair only if H5B, H5C or H5D identifies a concrete
  candidate/oracle mismatch and its negative control fails before the repair.
- If H5A is the only cause, a repair must correct a shared threshold semantic;
  smoothing, hysteresis or tolerance changes are new physics and not authorized.
- If H5E remains after the discriminators, close NCGP5 with no code repair and
  keep performance `NOT_RUN`. Any proposal to replace the per-particle 5 mm
  gate with a percentile/product metric requires a separately frozen decision
  requested from the user.
- No more than one diagnostic implementation and one selected repair/recheck.
