# Nonlocal corrected CUDA full-static-solve correspondence contract

| Field | Value |
| --- | --- |
| Research ID | `NCGA5` revision 1 |
| Status | `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY` |
| Parent checkpoint | commit `7f9ab3c6412a3ce1c5cfc25b0ce7353c196047a9`, tree `ab8e054718937b5ed9edef55c6aaff53e8681789` |
| Mathematical parent | FCR objective and reviewed NSR1 Steihaug--Toint policy |
| Engineering parent | NCGA4 strict-f32 CUDA assembly/host-controller solve prefix |
| Engineering consumer | Decide whether the corrected strict-f32 CUDA evaluator can finish the two retained static nonlinear solves before any trajectory or performance claim |
| Claim class | Two tiny fixed-graph static solves; no trajectory, dynamic-neighborhood, boundary, timing, runtime or product-water authority |

## Frozen parent identities

| Input | SHA-256 |
| --- | --- |
| assembly public header | `51091a1c3d1d86d13e7fdca9106e1b2352338dbd2c9b0dc37fec9e51d4d7c809` |
| strict CUDA assembly | `1d99f003e09f1211566efc77279f1bb7f4ef99af19d9be183576f3658cf8b41c` |
| independent host assembly | `7bd0462b36b5f83cdae88872551e067fef17faf9a98325716646d7d29cde40da` |
| NCGA4 controller/driver | `e6aba1f91a4b52afd5bedfe31decea7be9b94419d6e42aa4c5db3dff69b5616c` |
| NSR1 contract | `321128c11c5270f8ed1f0d4f4c3af884291b1e900ced679a4d7de47fa38f1096` |
| NSR1 evidence | `38c7d16ce23a6e4c9243699be7e5244c51dc5845477797efe46a7bf61f70bfd1` |
| NCGA4 contract | `9eb63f2a7cf6728062b0989c5b1da51435444a2a525d770dda865e598bc3ce7c` |

The parent evidence remains `AUTHOR_ONLY / REVIEW_NOT_RUN` unless an
independent reviewer is explicitly requested. NCGA2 remains refuted at its
elementwise Hessian gate and is not relabelled by this experiment.

## Question and competing hypotheses

| ID | Hypothesis | Prediction | Falsifier |
| --- | --- | --- | --- |
| H1 | strict-f32 assembly is sufficient for a complete static solve | both CUDA cases finish finite, monotone and physically coincident with the independent host solve | material final-state/objective drift, wrong active set, non-descent or a non-numerical solver failure |
| H2 | binary32 energy resolution is the first remaining boundary | CUDA approaches the host solution inside the frozen state/residual bands, then loses positive actual reduction and contracts the radius | CUDA reaches the raw NSR1 gradient stop or diverges materially before the reduction floor |
| H3 | the NCGA4 one-HVP prefix hid a Krylov boundary | a retained positive-curvature case changes residual/boundary/iteration behavior before convergence | both paths execute multi-HVP residual termination without material drift |
| H4 | micrometre reference/predicted quantization would contaminate the comparison | full continuous reference, predicted and current coordinates reproduce the host assembly; an intentionally changed reference/predicted state changes the expected result | the new state fields are ignored or old entry points change |
| H5 | shared controller logic can hide an invalid operator | a first-HVP sign inversion changes the route/result and is rejected | the negative control is classified like the strict candidate |

## Frozen cases and continuous-state boundary

Run exactly the original NSR1 cases:

1. `compressed_pair`: two particles at `(-0.025,0,0)` and `(0.025,0,0)`,
   zero velocity, `kappa=500`, and rest density equal to the original pair
   density divided by `1.1`;
2. `combined_tetrahedron`: the exact four binary64 positions, velocities and
   `kappa=200`, `lambda=20`, `mu=10`, `gamma=100` values in
   `combined_fixture()`; rest density is the original particle-zero density
   divided by `1.1`.

All other profile values remain `horizon=0.15 m`, `spacing=0.05 m`,
`mass=0.125 kg` and `dt=1/240 s`; gravity is zero. The initial current state
is the exact binary64 prediction `y_star=x+dt*v`.

Add one state overload carrying canonical ascending-ID binary64 reference,
predicted and current positions. The host path consumes them as binary64
promoted to its existing `long double` arithmetic. The CUDA path rounds every
component once to binary32 during upload. Integer micrometre coordinates remain
the immutable graph scaffold; every pair in these tiny cases lies safely
inside support, so rounding the scaffold cannot change graph membership.

The existing integer and current-only APIs are immutable regressions. Their
stdout hashes must remain exact. A nonfinite or wrong-sized continuous state
must be rejected before device work. Controls that perturb only predicted
state and only reference state must alter, respectively, inertia and the
reference-bound viscosity path on a coefficient-bearing case.

## Solver and arithmetic

Use the unchanged NSR1 unpreconditioned Steihaug--Toint policy:

- `Delta_0=spacing`, `Delta_min=2^-40*spacing`, `Delta_max=4*spacing`;
- residual forcing `eta=min(0.5,sqrt(||g||_2))`;
- at most `3*N_dof` inner iterations;
- the first non-positive curvature takes the positive boundary intersection;
- a candidate reaching the radius takes the positive boundary intersection;
- otherwise stop at the residual rule or the dimension limit;
- `ared=E(y)-E(y+p)`, `pred=-(g^T p+0.5*p^T*H*p)`;
- accept only finite `ared>0`, `pred>0`, `rho=ared/pred>=0.1`;
- shrink by `0.25` when invalid or `rho<0.25`; expand by `2` only for
  `rho>0.75` at the boundary;
- at most `64` outer trials and the unchanged raw success stop
  `||g||_2<=1e-10`.

The host controller, dense matvecs and dot products use ascending scalar order,
binary64 and no FMA contraction. CUDA assembles energy, gradient and dense
Hessian, then transfers them to the controller. No convergence tolerance,
regularization, line search, retry, warm state or mixed arithmetic may be added
after observing the result.

Also report the scale-aware displacement residual

```text
R_x = dt^2 / mass * max_i ||g_i||_2 / spacing
```

only as a diagnostic and as part of the predeclared numerical-floor
classification below. It does not turn the strict raw NSR1 stop into success.

## Frozen gates and classifications

The independent host path is valid only when it reproduces the retained NSR1
shape: compressed pair `4` outer trials, `5` objective evaluations and `8`
total Hessian products; combined tetrahedron `13`, `14` and `33`; no rejected
trial, radius contraction, active-set change or negative-curvature stop; final
gradient `<=1e-10`; and at least one multi-iteration `RESIDUAL` inner stop.

`GPU_STATIC_SOLVE_SUPPORTED` requires, for both strict-f32 cases:

1. raw NSR1 success within `64` trials, finite monotone accepted states and no
   minimum-radius, graph, branch or active-set failure;
2. at least one positive-curvature multi-HVP `RESIDUAL` stop across the corpus;
3. final maximum particle drift from the host result `<=5 um`;
4. final objective difference
   `abs(E_gpu-E_host)/max(abs(E_host),1)<=1e-6`;
5. `R_x<=1e-7` and identical final pressure-active signature;
6. canonical/permuted CUDA fixtures produce bit-identical canonical state,
   route, work and semantic roots; and
7. the sign-inverted first-HVP and continuous-state controls are detected.

`STRICT_F32_CONVERGENCE_FLOOR` is selected, not counted as solve success, when
the host gates pass and every strict-f32 case stays finite/monotone with final
drift `<=5 um`, objective difference `<=1e-6`, identical active signature and
`R_x<=1e-5`, but at least one case fails the raw stop only after non-positive
or zero-resolution actual reduction causes rejection/radius contraction.

`F32_STATIC_SOLVE_MATERIAL` is selected when the apparatus is valid but a CUDA
case loses finiteness/descent, exceeds the state/objective/residual bands,
changes pressure activity, crosses a topology branch or fails for a reason not
covered by the numerical-floor definition. `INCONCLUSIVE` covers broken
identity, host reproduction, state ownership, negative controls,
repeatability, work accounting or apparatus closure.

## Work, ordering and evidence

Seal per path: evaluator calls, trial evaluations, inner and prediction
matvecs, dot products, boundary intersections, accepted/rejected trials,
radius changes, state uploads/rounds, stop-reason counts, active signatures and
all inherited assembly work. Report each outer record with objective,
gradient, `R_x`, step, reductions, ratio, radii, reason, inner iterations/HVPs,
acceptance and state root.

Execute in this order:

1. independent host compressed and combined solves;
2. strict-f32 CUDA compressed and combined solves;
3. permuted strict-f32 controls;
4. sign-inverted HVP, invalid-state and reference/predicted ownership controls;
5. two fresh Release builds/runs with byte-identical stripped binaries/stdout;
6. CUDA `memcheck`, `initcheck` and `synccheck` only after numerical gates;
7. exact NCGA0--4 and retained NSR1 focused regressions.

Raw matrices and state arrays stay outside Git. Checked-in evidence contains
bounded metrics, roots, work receipts and source/binary/report hashes. Stop at
the first failed correctness or apparatus gate; do not time a failed solve.

## Claim ceiling and next branch

A positive result authorizes a separately frozen boundary-free short physical
trajectory. `STRICT_F32_CONVERGENCE_FLOOR` authorizes only one separately
frozen mixed-precision discriminator that promotes energy/globalization
quantities while retaining f32 stored state and the corrected f32 operator.
`F32_STATIC_SOLVE_MATERIAL` stops trajectory work and requires causal repair.

No result here establishes dynamic neighborhoods, contacts, boundaries, a
GPU-resident Krylov solver, scalable matrix-free cost, 50k throughput, frame
time, runtime integration or game-ready water.
