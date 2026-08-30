# Nonlocal corrected CUDA trust-region solve-prefix contract

| Field | Value |
| --- | --- |
| Research ID | `NCGA4` revision 1 |
| Status | `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY` |
| Parent checkpoint | commit `bfa9ec009fb31dc906cdeee7c82ba15c1aba31f2`, tree `cc7ec8de6c66027c7030009385ee64f861d83d5e` |
| Mathematical parent | FCR objective, reviewed NSR1 Steihaug--Toint policy and NCGA3 strict-f32 consequence result |
| Engineering consumer | Decide whether the verified strict-f32 CUDA assembly can drive a bounded globalized nonlinear solve prefix before any trajectory or performance work |
| Claim class | One tiny fixed-graph GPU-assembled/host-controlled solve prefix; no full solve, trajectory, timing, runtime or product-water authority |

## Frozen parent identities

The following SHA-256 identities are immutable inputs:

| Input | SHA-256 |
| --- | --- |
| assembly public header | `c461461061e99ecb215f2854a861115d54fdb52c8e5275394cc20f1af85b7ee8` |
| strict CUDA assembly | `6e0b3606cf655beeba73495f921329b0c600c5551bb181caf0291ff575af8493` |
| independent host assembly | `c91e93e98e4f681aad41d28cddabc5586cdfa9d66e8df9422099f3a0fdd9652d` |
| NCGA3 fixture/consequence driver | `a9a785aa39ffee9126399d5b19881a864a6490b49e6bcada3b60ee0176d6b10c` |
| NSR1 trust-region contract | `321128c11c5270f8ed1f0d4f4c3af884291b1e900ced679a4d7de47fa38f1096` |
| NCGA3 evidence | `22c61ef348ba361ce0a951c3b55f52205abc59dfca9b222bda90bee2262fceec` |

NCGA2 remains refuted at its `2e-4` elementwise Hessian gate. NCGA4 neither
changes that result nor treats NCGA3 as independently reviewed evidence.

## Question and competing hypotheses

NCGA4 asks whether the same small strict-f32 error that was negligible under
fixed local responses remains negligible when a real trust-region transaction
chooses directions, accepts/rejects trials and updates its radius.

| ID | Hypothesis | Prediction | Falsifier |
| --- | --- | --- | --- |
| H1 | strict-f32 assembly is solver-benign on the frozen local state | reference and CUDA paths make the same eight outer decisions, descend, and remain within `5 um` | route/reason divergence, non-descent, invalid model or excessive drift |
| H2 | energy rounding, not Hessian error, is the first solver boundary | CUDA and reference directions remain close but CUDA actual reduction becomes zero/wrong-sign or changes acceptance | both paths preserve reduction sign and acceptance |
| H3 | the isolated Hessian scalar can alter Krylov control flow | inner stop reason/iteration or trust radius diverges before a material state error appears | complete controller signatures match |
| H4 | a shared controller could hide a broken comparison harness | a deliberate first-HVP sign inversion is rejected by route/signature/state gates | the negative control is classified like the strict candidate |

## Frozen fixture and continuous-state boundary

Use only NCGA3 `combined_cluster` and its exact profile, terms, IDs,
reference/predicted/current integer coordinates and canonical ascending-ID
order. `combined_cluster_permuted` is the order-invariance control.

The starting current state is decoded exactly as integer micrometres to
binary64 metres. Subsequent accepted reference states are stored in binary64;
accepted CUDA states are rounded once componentwise to binary32 and then
stored in binary64. Reference/predicted coordinates and both initial CSR graphs
remain frozen. Every continuous trial uses those graphs and recomputes density,
energy, gradient and exact Hessian at its trial coordinates.

This frozen-graph choice is a local discriminator, not a dynamic-neighborhood
solver. Before every evaluation, recompute all current pair distances and fail
if a trial changes any support predicate or crosses either surface branch at
`spacing` or `3*spacing`. Also require the pressure-active signature to be
reported at every evaluated state; an active-set change is allowed but must be
identical between reference and CUDA at each common outer ordinal.

The continuous-state APIs are additive overloads. Existing integer NCGA2/3
entry points, fixture bytes, outputs and hashes must not change.

## Evaluators and controller ownership

The reference evaluator is the existing independent host translation using
`long double` arithmetic and continuous binary64 current positions. The
candidate evaluator is the existing compensated strict-f32 CUDA formula with
continuous positions rounded to binary32 at upload. It must not call the host
analytical or energy-only oracle.

Both paths use one host binary64 controller implementation so this experiment
isolates evaluator consequences:

```text
m(p) = E(y) + g^T p + 1/2 p^T H p,  ||p||_2 <= Delta.
```

The controller is the NSR1 unpreconditioned Steihaug--Toint truncated-CG
policy, with these predeclared local-prefix bounds:

- `Delta_0 = 50 um`, inherited from the NCGA3 maximum local step;
- `Delta_min = 2^-20 * Delta_0`, `Delta_max = 4 * Delta_0`;
- residual forcing `eta=min(0.5,sqrt(||g||_2))`;
- at most `3*N_dof` inner iterations;
- finite non-positive curvature takes the positive trust-boundary intersection
  and records `NEGATIVE_CURVATURE`; nonfinite curvature fails;
- a candidate crossing the radius takes the positive boundary intersection
  and records `BOUNDARY`;
- otherwise the residual rule records `RESIDUAL`, and exhaustion records
  `DIMENSION_LIMIT`;
- `ared=E(y)-E(y+p)`,
  `pred=-(g^T p + 0.5 p^T H p)` and `rho=ared/pred`;
- accept only finite trials with `ared>0`, `pred>0` and `rho>=0.1`;
- shrink by `0.25` when invalid or `rho<0.25`; expand by `2` up to the maximum
  only when `rho>0.75` and the inner step touched the boundary;
- rejected trials mutate only the radius and diagnostic counters.

All dot products and dense matvec reductions use ascending scalar index and
host binary64 without FMA contraction. The CUDA path assembles on the GPU but
copies its dense Hessian to the host for inner CG. No GPU-resident solver or
performance claim is allowed.

Execute exactly eight outer trials unless an invalid/nonfinite/topology gate
stops the run. Do not add a convergence tolerance, numerical-floor success,
preconditioner, regularization, line search, retry or warm state. At least one
trial must be accepted for the result to be informative.

## Frozen gates

The positive strict-f32 candidate is `GPU_ASSEMBLED_TRUST_PREFIX_SUPPORTED`
only when all gates pass:

1. reference and CUDA initial states are finite and have identical frozen
   graph and pressure-active identities;
2. every accepted state strictly decreases the objective evaluated by its own
   evaluator; every accepted trial has positive finite `ared`, `pred` and
   `rho>=0.1`;
3. the eight-entry accepted/rejected, inner-stop-reason, inner-iteration,
   boundary flag and radius-update signatures match exactly;
4. pressure-active signatures match at every common outer ordinal;
5. maximum particle-position difference at every common accepted ordinal and
   at the final state is `<=5 um`;
6. final objective reduction fractions differ by at most `1e-3`, using
   `abs((E0-Ef)_cuda-(E0-Ef)_ref)/max(abs(E0-Ef)_ref,1)`;
7. graph/support/surface predicates never change and no minimum radius is
   reached;
8. the sign-inverted first-HVP control is detected by a different inner route,
   acceptance signature or final state outside the positive correspondence
   band; and
9. canonical and permuted CUDA inputs produce bit-identical canonical state,
   controller signature, work receipt and semantic result.

Report each outer trial's objective bits, gradient norm, step norm, `ared`,
`pred`, `rho`, radius before/after, inner reason/iterations/HVP count, accepted
flag, active count and canonical state root. Seal aggregate evaluator counts,
CG matvecs, scalar products, boundary intersections, accepted/rejected trials,
radius changes and all inherited assembly work.

## Ordered execution and evidence

1. Reproduce the exact NCGA2 revision-2 failure and NCGA3 positive report
   before interpreting NCGA4.
2. Run the independent reference prefix, strict CUDA prefix, sign-inverted
   control and permuted strict prefix in that order.
3. Build in two fresh Release directories; stripped binaries and stdout must
   be byte-identical.
4. Only after numerical and negative-control gates complete, run CUDA
   `memcheck`, `initcheck` and `synccheck` on the exact Release executable.
5. Re-run NCGA0, NCGA1, NCGA2 and NCGA3 focused regressions.

Raw matrices and state arrays remain outside Git. Checked-in evidence contains
bounded metrics, semantic roots, work receipts, source/binary/report hashes and
the exact build commands. Independent mathematical review is required before
the result may be called reviewed; without an explicitly requested reviewer it
must remain `AUTHOR_ONLY / REVIEW_NOT_RUN`.

## Resolution and claim ceiling

- `GPU_ASSEMBLED_TRUST_PREFIX_SUPPORTED`: every positive and negative gate
  passes.
- `ENERGY_ROUNDING_GLOBALIZATION_BOUNDARY`: direction/state proximity holds
  before an f32 actual-reduction sign/zero changes acceptance.
- `KRYLOV_CONTROL_FLOW_BOUNDARY`: strict f32 changes an inner reason,
  iteration, radius or acceptance before the state-drift gate fails.
- `F32_SOLVER_CONSEQUENCE_MATERIAL`: a valid path loses descent, exceeds
  `5 um`, changes pressure activity or crosses the objective-reduction band.
- `INCONCLUSIVE`: identity, independence, topology, work, control,
  repeatability or apparatus closure fails.

Even a positive result proves only that one eight-trial, 100-particle,
fixed-graph solve prefix can consume strict-f32 CUDA assembly with a CPU
controller. It does not establish convergence, physical trajectory accuracy,
dynamic neighborhoods, contacts/boundaries, 50k cost, full frame time,
GPU-resident Newton--CG, runtime integration or game-ready water. Mixed
pressure products may be studied only in a separately frozen revision after a
strict-f32 solver-level failure; no arithmetic fallback runs in revision 1.
