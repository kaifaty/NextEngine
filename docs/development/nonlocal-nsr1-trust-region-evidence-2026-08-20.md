# Nonlocal NSR1 trust-region evidence -- 2026-08-20

Status: `PASS / NSR_TRUST_REGION_CANDIDATE / NSR2_AUTHORIZED / REPORT_ONLY`

## Outcome

Full-HVP unpreconditioned Steihaug--Toint trust-region Newton-CG closes the two
frozen stiff controls without one rejected trial, radius contraction, active-set
change or objective regression. It reaches the FCR2 optimum to binary64
agreement with much smaller gradient residual and radically fewer
objective/gradient evaluations.

| Case | FCR2 evaluations | NSR1 evaluations | HVP calls | Outer trials | Final gradient FCR2 -> NSR1 |
|---|---:|---:|---:|---:|---:|
| compressed pair | `1816` | `5` | `8` | `4` | `5.86e-6 -> 2.33e-12` |
| combined tetrahedron | `896` | `14` | `33` | `13` | `3.31e-6 -> 5.16e-13` |

The objective differences from FCR2 are `5.2e-16` and `6.2e-15`, respectively,
inside the frozen `1e-10` relative allowance. Accepted trust ratios are at
least `0.9987` and `0.9684`. Both cases retain the initial `0.05 m` radius.
Internal momentum residual is `0` and `2.06e-16`.

No inner Krylov path encountered negative curvature in these trajectories. NSR0
still proves that negative modes exist in the compressed-pair Hessian; the
gradient and generated Krylov subspace were radial and did not excite its
transverse modes. Keeping negative-curvature handling is therefore required,
but its runtime frequency remains an NSR4 corpus question.

The very large maximum trust ratios occur only at the last near-stationary
steps where predicted and actual reductions are both close to binary64 noise.
They do not select a radius change because those inner solves finish by
residual rather than at the boundary. NSR2 must retain separate absolute
progress diagnostics; this does not invalidate any frozen acceptance gate.

## What this proves

- the corrected variational energy is solvable on the bounded stiff controls;
- omitted full curvature, not a wrong FCR0 formula, explains the observed
  fast-solver failure on these controls;
- fixed-spectrum Chebyshev failure does not generalize to safeguarded
  Newton--Krylov;
- the penalty formulation does not yet need replacement by a constrained KKT
  identity.

It does not prove scalable cost. The current reference evaluates pairs with
quadratic loops, and `8/33` HVP calls on 2/4 particles say nothing about a
50k-neighbor workload or preconditioning quality.

## Exact artifacts

| Artifact | SHA-256 |
|---|---|
| accepted raw report, run 1 | `520258ef7f711d90bb4e1ab1e7c0fc8c9ca12002aff0287223901626cc50678f` |
| accepted raw report, run 2 | `520258ef7f711d90bb4e1ab1e7c0fc8c9ca12002aff0287223901626cc50678f` |
| semantic result | `da651a423d116f2e9c0d08205f450f8aaca392672d7e58452e4ea6b219ef4046` |

Frozen NSR0 and FCR0/FCR1/FCR2/FCR3-B2 report hashes remain exactly
`ab578e...`, `996eff...`, `ead18d...`, `10b98c...` and `95c51f...` as recorded
in the stage evidence.

## Decision

Select `NSR_TRUST_REGION_CANDIDATE`. Keep the penalty objective for NSR2 and
test scalable neighborhood HVP plus a local block-Gauss--Newton preconditioner
inside the full trust-region method. Do not start the constrained primal-dual
branch unless later evidence isolates penalty-dominated cost.

