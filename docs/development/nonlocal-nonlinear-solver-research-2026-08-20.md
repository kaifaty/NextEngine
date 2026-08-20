# Nonlocal nonlinear solver research -- 2026-08-20

Status: `ACTIVE / NEW_SOLVER_LINEAGE / NSR2C2_PASS / NSR3_DESIGN / REPORT_ONLY`

## Outcome

The corrected Nonlocal energy remains worth researching, but the next step is
not another SISSM coefficient or acceleration sweep. The stopped evidence is
consistent with a solver-model mismatch: local fixed-point maps and a fixed
Chebyshev recurrence do not see the full, state-dependent pressure curvature.

The selected first research path is:

```text
exact analytic HVP
  -> spectral/active-set atlas on tiny controls
  -> Steihaug--Toint trust-region Newton-CG
  -> measured branch decision
```

This keeps the already verified objective as the source of truth while testing
a globalization strategy that reacts to actual model agreement and negative
curvature. It does not assume a stationary spectral radius.

## Why full curvature is a distinct hypothesis

For active compression constraints, `Phi = kappa/2 * sum(s_i^2)` has

```text
H_Phi = kappa * (J^T J + sum_i s_i H_i).
```

`J^T J` couples all particles that influence the same density sample; it is
not block diagonal. `sum_i s_i H_i` can contain negative curvature because the
SPH kernel is radial and nonlinear. FCR3-A tested local block/Gauss--Newton
approximations, not this full operator. Its negative result therefore does not
reject Newton--Krylov.

Chebyshev acceleration has a different failure mode. Its recurrence assumes a
usable spectral interval for a sufficiently stable iteration map. In the
observed pressure cases the density active set and local linearization change,
and the recurrence becomes non-descent after 2--7 accepted iterations. A trust
region instead truncates on negative curvature/boundary, evaluates the actual
to predicted reduction ratio, rejects an inaccurate model step without state
mutation and changes only the radius under a fixed policy.

## Candidate families considered

| Family | Decision | Reason |
|---|---|---|
| Fixed SISSM/Chebyshev retuning | Rejected | Already closed by FCR3-B2; a radius or iteration sweep cannot repair non-descent evidence |
| L-BFGS/nonlinear CG | Deferred comparator | Lower implementation cost, but does not answer whether omitted full curvature is the root cause |
| Projected/local PSD Newton | Deferred | Graphics evidence shows unconditional projection may reduce convergence and can hide useful negative curvature |
| Trust-region Newton-CG | Selected for first discriminator | Matrix-free, globally safeguarded for non-convex smooth regions and explicitly detects negative curvature |
| Pairwise Descent | Monitor only | The authors list paper and code as `to appear`; no method may be inferred from the title |
| Primal-dual/active-set incompressibility | Conditional NSR2 branch | Removes penalty stiffness and represents compression as an inequality, but creates a new model/KKT identity that needs its own oracle |

## Primary-source basis

- Steihaug, *The Conjugate Gradient Method and Trust Regions in Large Scale
  Optimization*, SIAM J. Numer. Anal. 20(3), DOI `10.1137/0720042`: truncated
  preconditioned CG obtains a trust-region step and handles negative curvature
  without a global factorization.
- Curtis, Robinson, Royer and Wright, *Trust-Region Newton-CG with Strong
  Second-Order Complexity Guarantees for Nonconvex Optimization*, DOI
  `10.1137/19M130563X`: modern non-convex Newton-CG globalization and
  Hessian-vector-product complexity basis.
- Longva et al., *Pitfalls of Projection: A study of Newton-type solvers for
  incremental potentials*, arXiv `2311.14526`: unconditional Hessian PSD
  projection can slow incremental-potential solves; conditional/regularized
  alternatives and robust globalization matter.
- Probst and Teschner, *Monolithic Friction and Contact Handling for Rigid
  Bodies and Fluids Using SPH*, DOI `10.1111/cgf.14727`: fluid
  incompressibility can be expressed as a pressure complementarity/LCP system,
  supporting the conditional constrained branch.
- He et al., *A Semi-Implicit SPH Method for Compressible and Incompressible
  Flows with Improved Convergence*, DOI `10.1111/cgf.70043`: source lineage for
  the nonlinear position objective and SISSM acceleration already tested.
- He research-group publication page, verified `2026-08-20`: *Semi-Implicit
  Pairwise Descent for Nonlocal Continuum Mechanics* and its code remain
  `to appear`.

## Falsifiable hypotheses

| ID | Hypothesis | Discriminator |
|---|---|---|
| H1 | Omitted off-diagonal pressure curvature explains the local solver stall | NSR0 full Hessian spectrum/coupling plus NSR1 full-HVP versus FCR2 cost |
| H2 | Fixed-spectrum acceleration failed because curvature/active set changes materially | report active margins, eigenvalue range and trust-ratio rejection/resize events along NSR1 |
| H3 | Full HVP plus trust-region globalization reaches FCR2 quality with at least 4x fewer expensive evaluations | frozen NSR1 tiny corpus and call counters |
| H4 | The quadratic compression penalty is intrinsically too stiff even for full Newton | NSR1 shows correct model agreement but unacceptable condition/cost; branch to constrained NSR2-C |
| H5 | Pairwise descent is a better local parallel solver | no test until primary formulas or reproducible code are public |

## Decision

Execute the [new roadmap](../plans/nonlocal-nonlinear-solver-research/README.md)
starting with its frozen NSR0 Hessian/HVP contract. Preserve every old report
byte-for-byte. No product-scale, CUDA or production work is authorized yet.
