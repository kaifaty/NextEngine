# NSR3-B4E2D7R19R35 equal-work generalized-Hessian research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

## Question left by R34

R34 provides a deterministic first-order continuation from exact `v8` to
`v32` using 51 new pair passes including prefix and terminal checks. It reduces
the objective strongly but remains nonstationary. A curvature method is now
justified only as an equal-work discriminator from the same `v8` state.

## Selected curvature model

For residual `r=c+A v` and active mask `D=diag(r>0)`, squared hinge has

```text
g = A^T [r]+
H = A^T D A
```

The L2-loss SVM work of Lin, Weng and Keerthi uses this active-set generalized
Hessian and matrix-free Hessian-vector products inside trust-region Newton:

- https://www.jmlr.org/papers/volume9/lin08b/lin08b.pdf

Lin and Moré's TRON combines projected-gradient/Cauchy geometry, conjugate
gradient curvature directions and projected search for large problems:

- https://www.csie.ntu.edu.tw/~cjlin/papers/tron.pdf
- https://www.mcs.anl.gov/~more/tron/

Steihaug establishes matrix-vector-only CG approximation of large trust-region
subproblems, including curvature/boundary termination:

- https://doi.org/10.1137/0720042

R35 is a separately derived discriminator, not a claim of exact equivalence to
those full algorithms.

## Singularity decision

`H` is positive semidefinite and may be singular. No damping is added in R35.
For the current active rows, `g=A_active^T r_active` and

```text
range(A_active^T A_active) = range(A_active^T),
```

so `H p=-g` is consistent in exact arithmetic even with a nullspace. CG starts
from `p=0`, returns the Krylov minimal-norm direction, stops on exact residual
zero or nonpositive computed curvature, and performs at most five HVPs. This
isolates curvature benefit without fitting regularization from nominal data.

## Equal-work recurrence

Start from the exact R33 `v8` and maintained response, not `v0` or R34 `v32`.
A fresh prefix JVP must reproduce the frozen `v8` response. Execute four
generalized-Newton outer iterations:

```text
gradient VJP                         1 pass
five (JVP -> active mask -> VJP)    10 passes maximum
globalization-direction JVP          1 pass
per outer maximum                   12 passes
four outers                         48 passes
prefix + fresh terminal checks       3 passes
total maximum                       51 passes
```

After CG, project `v+p` into the fixed radius-`0.25` ball and run the exact
R32 all-row piecewise hinge line minimization along that feasible chord. This
allows the active set to change and rejects a raw active-only Newton update.

## Classification

Fresh terminal JVP/VJP evidence is compared with the frozen R34 terminal at
the same 51-pass continuation budget. Curvature wins only if objective,
violation norm and projected-mapping norm are all strictly smaller. Otherwise
the first-order reference is retained. Projected stationarity is a separate
classification.

No magnitude threshold, wall time or post-result work extension participates
in the decision.

## Controls and authority

Independent dense controls cover SPD Newton convergence, a singular but
consistent generalized Hessian, active-set switching under all-row
globalization and trust projection. Exact parent/source/prefix, CG recurrence,
line KKT, terminal operators, 51-pass ledger and rollback are mandatory.

R35 cannot apply either iterate, evaluate a nonlinear moved state, change
penalty/trust policy, update duals or claim floor, timing, runtime or production
authority.
