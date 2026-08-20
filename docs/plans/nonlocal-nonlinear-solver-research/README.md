# Nonlocal nonlinear solver research roadmap

Status: `ACTIVE / NSR0_PASS / NSR1_PASS / NSR2B_PASS / NSR2C_FAIL / NSR2C1_PASS / NSR2C2_PASS / NSR3A_PASS / NSR3A1_PASS / NSR3A2_PASS / NSR3B0_PASS / NSR3B0R_PASS / NSR3B1_FAIL / NSR3B1D_INVALID / NSR3B1D1_PASS / NSR3B1S_FAIL / NSR3B1S1_PASS / NSR3B1S2_FAIL / NSR3B1S3_PASS / NSR3B1R_FAIL / NSR3B1R1_PASS / NSR3B2_PASS / NSR3B3_FAIL / NSR3B3D_PASS_CERT_REJECT / NSR3B3D1_FAIL / NSR3B3D2_PASS / NSR3B3D3_FAIL / NSR3B3D4_FROZEN / REPORT_ONLY`

Candidate identity:

```text
nuv-newton-krylov-r0
```

This is a new solver lineage over the verified
`nuv-variational-fcr2` objective. It does not reopen, repair or relabel the
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
NSR3-A1 then selects exact allocation-free HVP workspaces; NSR3-A2 selects an
exact per-outer-state Hessian coefficient tape. Including its construction,
the tape improves build+HVP by `2.05x--2.43x` and total solve by
`1.36x--1.55x` through 4096 particles while remaining under its linear memory
cap; see the [dated evidence](../../development/nonlocal-nsr3a2-hessian-tape-evidence-2026-08-20.md).
NSR3-B0 then proves that the raw FCR cubic integrates to `1/8` and yields
`0.125224338*rho0` on the canonical reference lattice. The authors' fixed
lattice normalization reconstructs density correctly, so B0 selects
`FORMULA_RECLOSURE_REQUIRED`, not a tuned profile; see the
[dated evidence](../../development/nonlocal-nsr3b0-dimensional-profile-evidence-2026-08-20.md).
B0R passes the common-scale density, gradient, HVP, dense-Hessian, trust and
Hessian-tape gates; see the
[dated evidence](../../development/nonlocal-nsr3b0r-kernel-normalization-evidence-2026-08-20.md).
This selects only `FCR2_NORMALIZED_OBJECTIVE_CANDIDATE`. The separately frozen
NSR3-B1 manufactured multi-step run passes four invariance/objectivity cases
but fails compression step doubling at ratio `0.8865`; see the
[dated evidence](../../development/nonlocal-nsr3b1-multistep-evidence-2026-08-20.md).
Preserve that failure and design an acoustic-Courant diagnostic next. Static
boundaries and physical trajectories remain blocked.
The diagnostic is now frozen in
[NSR3-B1D](03b1d-temporal-stiffness-contract.md); it extends the same fixture
to Courant `0.129` and cannot retroactively change the B1 result.
Its main ladder shows a first-order-like trend, but the floor-disabled strict
oracle fails at minimum trust radius; see the
[dated evidence](../../development/nonlocal-nsr3b1d-temporal-stiffness-evidence-2026-08-20.md).
Preserve B1D as invalid and design a floor-limited B1D1 oracle next.
That oracle is frozen in
[NSR3-B1D1](03b1d1-floor-limited-oracle-contract.md); it retains the arithmetic
floor and removes only the ordinary early scale stop.
B1D1 passes and confirms temporal stiffness with negligible nonlinear-solver
sensitivity; see the
[dated evidence](../../development/nonlocal-nsr3b1d1-floor-oracle-evidence-2026-08-20.md).
Freeze a multi-fixture acoustic substep-policy gate next; B2 remains blocked.
The six-case policy gate is frozen in
[NSR3-B1S](03b1s-acoustic-substep-policy-contract.md) at target Courant
`0.25`, with explicit `34/67` base/high-stiffness substep costs.
B1S rejects that linear policy: all 2% compression cases exceed the velocity
accuracy limit despite clean self-convergence; see the
[dated evidence](../../development/nonlocal-nsr3b1s-acoustic-policy-evidence-2026-08-20.md).
Diagnose the finite-state pressure tangent spectrum before another policy.
The bounded matrix-free/dense diagnostic is frozen in
[NSR3-B1S1](03b1s1-pressure-tangent-spectrum-contract.md); it selects no
trajectory target and must publish its operator cost.
B1S1 passes: dense and Lanczos agree, stiffness scaling is exact, and spectral
amplification captures the amplitude increase; see the
[dated evidence](../../development/nonlocal-nsr3b1s1-pressure-spectrum-evidence-2026-08-20.md).
Freeze a spectral trajectory policy next, with 48-HVP estimate cost explicit.
That policy gate is frozen in
[NSR3-B1S2](03b1s2-spectral-substep-policy-contract.md) at spectral target
`0.15`; expected base/high costs are `39/43` and `78/86` substeps per frame.
B1S2 rejects spectrum as a standalone error policy: its 2% row remains just
over the velocity threshold; see the
[dated evidence](../../development/nonlocal-nsr3b1s2-spectral-policy-evidence-2026-08-20.md).
Design an embedded error controller with a new amplitude holdout next.
The controller is frozen in
[NSR3-B1S3](03b1s3-embedded-error-controller-contract.md), including accepted
state ownership, discarded comparator work and a new `0.97dx` holdout.
B1S3 passes all parent cases and the holdout without fitting a new safety
factor; see the
[dated evidence](../../development/nonlocal-nsr3b1s3-embedded-controller-evidence-2026-08-21.md).
It selects only a report-only error controller. Freeze B1R next to test its
transactional composition over the original full multi-step horizon and to
expose the recurring comparator cost before B2 boundary design.
That discriminator is now frozen in
[NSR3-B1R](03b1r-transactional-controller-contract.md): 12 transactional
macro frames, current-state spectra, explicit rollback/discarded work and an
independent fixed `96/192/384` reference ladder.
B1R rejects coarse-state composition: all local transactions pass, but final
position error reaches `0.072--0.134dx` as first-frame velocity error
propagates; see the
[dated evidence](../../development/nonlocal-nsr3b1r-transactional-composition-evidence-2026-08-21.md).
Preserve that result and test fine-state ownership in B1R1. B2 remains
blocked.
The isolated ownership discriminator is frozen in
[NSR3-B1R1](03b1r1-fine-state-ownership-contract.md). It commits the already
computed fine member, retains every B1R reference/gate and must account for
the coarser probe as discarded work.
B1R1 passes all three full-horizon cases without executing additional work;
see the
[dated evidence](../../development/nonlocal-nsr3b1r1-fine-state-evidence-2026-08-21.md).
It selects `NSR_MULTISTEP_CANDIDATE` and authorizes B2 static-boundary formula
design only. Boundary execution remains blocked until that contract is frozen
and its derivative oracles pass.
The primary-source audit selects a split support/contact model; see the
[B2 research note](../../development/nonlocal-nsr3b2-boundary-formula-research-2026-08-21.md).
The resulting
[NSR3-B2 contract](03b2-split-static-boundary-contract.md) is frozen with
fluid-only pressure centers, virtual support reaction, two-versus-three-layer
correspondence and separate hard-contact oracles.
B2 passes all derivative, reaction, layer and contact gates; see the
[dated evidence](../../development/nonlocal-nsr3b2-split-boundary-evidence-2026-08-21.md).
It selects `SPLIT_STATIC_BOUNDARY_FORMULA_CANDIDATE`: two layers remain exact
for this `H=3dx` cubic identity, while the required ghost-only negative proves
that support still cannot provide nonpenetration. The boundary pressure
Hessian is symmetric but indefinite, so B3 must retain safeguarded trust-region
handling. Freeze a tiny split-composition smoke trajectory next; hydrostatic,
product-scale, CUDA and performance execution remain blocked.
The ordering study selects a post-solve analytical sweep as the smallest
falsifiable composition; see the
[B3 research note](../../development/nonlocal-nsr3b3-boundary-composition-research-2026-08-21.md).
The bounded [B3 contract](03b3-boundary-composition-smoke-contract.md) freezes
face/corner impact fixtures, fine-state ownership, fixed references, separate
support/contact impulse ledgers and mandatory cache invalidation before any
trajectory code.
B3 stops at its first momentum-ledger failure before face contact; see the
[dated evidence](../../development/nonlocal-nsr3b3-boundary-smoke-evidence-2026-08-21.md).
The selected scale-aware trajectory stop does not certify virtual reaction at
the stricter local tolerance, and very fine inactive steps expose a separate
relative-roundoff floor. Preserve B3 FAIL and design B3D to distinguish a
reaction-aware solve from a mixed absolute/relative certificate before any
second composition attempt.
The [B3D research](../../development/nonlocal-nsr3b3d-reaction-accuracy-research-2026-08-21.md)
separates active stationarity, translation identity, inactive reconstruction
roundoff and contact closure. Its
[frozen contract](03b3d-reaction-accuracy-contract.md) requires a computed
binary64 forward-error bound and charges a reaction-aware-stop counterfactual;
neither mechanism may weaken the other.
B3D completes and validates the active reaction-aware stop, but rejects its
inactive cumulative certificate because the conservative `|x|/h` bound grows
under refinement; see the
[dated evidence](../../development/nonlocal-nsr3b3d-reaction-accuracy-evidence-2026-08-21.md).
The follow-up [D1 research](../../development/nonlocal-nsr3b3d1-displacement-ownership-research-2026-08-21.md)
selects transient substep displacement as the next falsifiable representation.
Its [frozen contract](03b3d1-displacement-ownership-contract.md) must remove
world-position cancellation and retain the reaction-aware result before B3R.
D1 removes the diagnosed cancellation on every completed prefix, but all six
fixed trajectories stop at `REACTION_BELOW_ENERGY_RESOLUTION`; see the
[dated evidence](../../development/nonlocal-nsr3b3d1-displacement-ownership-evidence-2026-08-21.md).
Preserve D1 FAIL. Research a direct per-term objective-difference evaluator
with a derived roundoff certificate before attempting B3R; do not weaken the
reaction gate or reinterpret the inherited absolute energy floor as success.
The competing finite-precision explanations and decision tree are now frozen
in [NSR3-B3D2](03b3d2-finite-precision-merit-contract.md). D2 replays only the
six first-floor states and may classify a later candidate; it cannot continue
their trajectories or authorize B3R.
D2 passes and rejects factored endpoint energy as a common solution: three
states have a certified negative endpoint change. All six unchanged-topology
trials nevertheless reach the existing reaction gate, selecting only
`FLOOR_STATIONARITY_MERIT_CANDIDATE`; see the
[dated evidence](../../development/nonlocal-nsr3b3d2-finite-precision-merit-evidence-2026-08-21.md).
Freeze a full-trajectory D3 candidate before any B3 retry.
The bounded candidate is now frozen in
[NSR3-B3D3](03b3d3-floor-stationarity-trajectory-contract.md). It permits one
charged residual-merit acceptance only at the inherited active energy floor
and only when that same trial already closes the unchanged reaction gate.
D3 rejects the one-trial policy: it accepts `1--121` exact floor trials per
prefix, then every row reaches a new failure; see the
[dated evidence](../../development/nonlocal-nsr3b3d3-floor-stationarity-trajectory-evidence-2026-08-21.md).
Diagnose legacy `(y-y*)` inertia gradient versus owned
`(delta-delta*)` stationarity before designing any residual iteration.
That six-state discriminator is now frozen in
[NSR3-B3D4](03b3d4-owned-gradient-contract.md). It changes only the inertia
gradient input for one counterfactual trust step; the HVP and physics remain
unchanged.
