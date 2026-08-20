# Nonlocal nonlinear solver research roadmap

Status: `ACTIVE / NSR0_PASS / NSR1_PASS / NSR2B_PASS / NSR2C_FAIL / NSR2C1_PASS / NSR2C2_PASS / NSR3A_PASS / NSR3A1_PASS / NSR3A2_ACTIVE / REPORT_ONLY`

Candidate identity:

```text
nuv-newton-krylov-r0
```

This is a new solver lineage over the verified
`nuv-variational-fcr1` objective. It does not reopen, repair or relabel the
stopped SISSM/Chebyshev lineage. FCR0--FCR3-B2 reports and roots remain exact
historical evidence. The FCR2 binary64 objective/gradient is the comparison
oracle; DFSPH remains the water correctness reference.

## Research question

> Is the corrected Nonlocal objective practically solvable when the optimizer
> sees its full coupled curvature, including negative curvature and active-set
> changes, or does incompressibility need a constrained reformulation rather
> than a stiff compression penalty?

The first result is diagnostic, not a solver benchmark. It must distinguish:

1. missing global/off-diagonal Hessian coupling;
2. non-convex or rapidly changing curvature;
3. compression active-set discontinuity;
4. intrinsic penalty stiffness that remains after exact curvature is used.

## Stage graph

```text
NSR0 exact Hessian/HVP oracle + spectral atlas
  -> NSR1 safeguarded trust-region Newton-CG discriminator
      -> if penalty form is viable: NSR2 matrix-free analytic HVP/preconditioner
      -> if penalty stiffness dominates: NSR2-C constrained primal-dual study
          -> NSR3 scalable CPU neighborhood implementation
              -> NSR4 physical corpus and convergence
                  -> NSR5 CUDA correspondence
                      -> NSR6 performance and production-roadmap handoff
```

| Stage | Required result | Exit gate |
|---|---|---|
| NSR0 | Dense strict-f64 Hessian on tiny controls, analytic matrix-free HVP, symmetry/eigenvalue/active-margin atlas | HVP agrees with central gradient differences and dense multiplication under the frozen tolerances; old FCR reports remain byte-identical |
| NSR1 | Unpreconditioned Steihaug--Toint trust-region Newton-CG over the unchanged objective | Every accepted step has positive actual reduction and valid model reduction; pressure and combined controls reach FCR2-or-better objective/gradient with at least a predeclared 4x reduction in objective/gradient evaluations, or emit the exact first failure |
| NSR2 | NSR2-A separately freezes/tests a full-HVP preconditioner; NSR2-B replaces quadratic pair scans with a deterministic neighborhood operator; constrained pressure remains conditional | Same objective and trust acceptance; no coefficient/iteration sweep; correspondence precedes scale claims |
| NSR3 | Deterministic CPU pair/neighborhood implementation with bounded work and stable reductions | Matrix-free result corresponds to the tiny oracle and storage-order controls; complexity and memory scale with admitted particles/neighbors rather than dense dimension |
| NSR4 | Hydrostatic, dam-break, orifice, viscosity and surface-tension corpus | Quality, conservation, convergence and failure gates are frozen before execution; DFSPH/analytic comparisons pass |
| NSR5 | CUDA report-only mirror | CPU/GPU aggregate correspondence, repeatability, capacity and failure-before-publication gates pass; no authority claim |
| NSR6 | Measured optimization and disposition | Either a bounded production roadmap is justified or the lineage stops with its first reproducible boundary |

NSR3 is now decomposed into [serial CPU baseline](03a-serial-cpu-baseline-contract.md)
and [physical corpus design](03b-physical-corpus-design.md). Measurement runs
first; physical execution remains blocked until dimensional profile and
corrected boundary contracts exist.

## Frozen branch rule after NSR1

- If full curvature gives reliable accepted steps and the required evaluation
  reduction, continue the penalty objective and optimize HVP/preconditioning.
- If Newton steps are dominated by extreme positive curvature but otherwise
  model the objective correctly, investigate replacing
  `kappa/2 * max(rho/rho0-1, 0)^2` with the explicit inequality
  `rho/rho0 - 1 <= 0`, a non-negative multiplier and a primal-dual merit
  function. This is a new mathematical identity and cannot inherit FCR roots.
- If frequent active-set crossings invalidate both local models within the
  frozen trust policy, stop and await/compare the authors' Pairwise Descent
  publication; do not infer that method from its title.

## Execution rules

1. Freeze each discriminator, corpus, budget and tolerances before its code.
2. Do not tune physical coefficients to help an optimizer pass.
3. Count objective, gradient and HVP calls separately; an iteration count is
   not a cost claim.
4. A trial rejected by the trust ratio cannot update state or become evidence.
5. Negative curvature is a measured solver event, not an error by itself.
6. The compression active set and minimum distance to `rho=rho0` are reported.
7. Product-scale, CUDA and performance runs stay blocked until the preceding
   tiny gate passes.
8. Windows remains out of scope for this research branch.
9. No public schema, save state, PhysX coupling, gameplay mutation or runtime
   authority is authorized.

## Stop states

- `NSR_HVP_CANDIDATE`: NSR0 proves the curvature oracle only.
- `NSR_TRUST_REGION_CANDIDATE`: NSR1 proves a bounded fast-solver direction on
  tiny controls only.
- `NSR_CPU_PHYSICS_CANDIDATE`: NSR3--NSR4 pass.
- `NSR_CUDA_CANDIDATE`: NSR5 passes correspondence only.
- `NSR_PRODUCTION_ROADMAP_CANDIDATE`: NSR6 justifies a separate integration
  proposal.
- `NSR_STOP`: the frozen discriminator/remediation cycle fails.

## Current result and next action

NSR0 passes and selects `NSR_HVP_CANDIDATE`; see the
[dated evidence](../../development/nonlocal-nsr0-spectral-hvp-evidence-2026-08-20.md).
NSR1 then passes and selects `NSR_TRUST_REGION_CANDIDATE`; see its
[dated evidence](../../development/nonlocal-nsr1-trust-region-evidence-2026-08-20.md).
NSR2-A/A1 rejects the local block metric, while NSR2-B selects the exact
canonical neighborhood operator. NSR2-C then preserves exact correspondence
and converges at every scale, but fails the frozen work gate at 512 particles
with 13 rejects and 153 HVP calls. NSR2-C1 proves those rejects repeatedly
evaluate one step below the binary64 energy-difference floor, with no support
or active-set change. NSR2-C2 passes with a guarded numerical-floor stop,
reducing the 512-particle case to 12 outer trials, zero rejects and 46 HVPs.
Freeze and execute NSR3 baseline/performance and physical-corpus contracts.
