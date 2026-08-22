# Nonlocal nonlinear solver research roadmap

Status: `ACTIVE / NSR0_PASS / NSR1_PASS / NSR2B_PASS / NSR2C_FAIL / NSR2C1_PASS / NSR2C2_PASS / NSR3A_PASS / NSR3A1_PASS / NSR3A2_PASS / NSR3B0_PASS / NSR3B0R_PASS / NSR3B1_FAIL / NSR3B1D_INVALID / NSR3B1D1_PASS / NSR3B1S_FAIL / NSR3B1S1_PASS / NSR3B1S2_FAIL / NSR3B1S3_PASS / NSR3B1R_FAIL / NSR3B1R1_PASS / NSR3B2_PASS / NSR3B3_FAIL / NSR3B3D_PASS_CERT_REJECT / NSR3B3D1_FAIL / NSR3B3D2_PASS / NSR3B3D3_FAIL / NSR3B3D4_PASS / NSR3B3D5_PASS / NSR3B3R_PASS / NSR3B4A_PASS / NSR3B4B_FAIL / NSR3B4BK_FAIL / NSR3B4BK1_PASS / NSR3B4B1_FAIL / NSR3B4BF_PASS / NSR3B4B2_PASS / NSR3B4C3MC1_TINY_ACCURACY_PASS / NSR3B4C4M0_PASS / NSR3B4C4A_PASS / NSR3B4C4A1_PASS / NSR3B4C4B_PASS / NSR3B4C4BM_PASS / NSR3B4C4B1_PASS / NSR3B4C4C_PASS / NSR3B4C4CM_PASS / NSR3B4C4C1_PASS / NSR3B4D_MISSING_ARTIFACT / NSR3B4DR1E_PASS / NSR3B4EP10I_PASS / NSR3B4EP10PCD_PASS / NSR3B4EP10PCI_FAIL / NSR3B4EP10CTD_FAIL / NSR3B4EP10SID_PASS / NSR3B4EP10SICD_PASS / NSR3B4EP10SII_PASS / NSR3B4EP10SIR_PASS / NSR3B4EP10SIRD_PASS / NSR3B4EP10SIRDA_PASS / NSR3B4EP10SIRDI_PASS / NSR3B4EP10SIRDIR_PASS / NSR3B4EP10SIRDIRE_PASS / NSR3B4EP10SIRDIREA_PASS / NSR3B4EP10SIRDIREI_FAIL / NSR3B4EP10SIRDIREP_FAIL / NSR3B4EP10SIRDIREQ_HOST_UNQUALIFIED / NSR3B4EP10SIRDIREQ1_PASS / NSR3B4EP10SIRDIREQ2_PASS / NSR3B4EP10SIRDIREQ3_FAIL / NSR3B4EP10SIRDIREQ4_FAIL / NSR3B4E2R_PASS / NSR3B4E2D_FAIL / NSR3B4E2D0_PASS / NSR3B4E2D1_PASS / NSR3B4E2D2_PASS / NSR3B4E2D3_FAIL_STEP2_STRAIN / NSR3B4E2D4_PASS_FINITE_PENALTY / NSR3B4E2D5_PASS_AL_SELECTED / NSR3B4E2D6_PASS_AL_PATH / NSR3B4E2D7_FAIL_PRESSURE_COMMIT / NSR3B4E2D7R_FAIL_INNER_FLOOR / NSR3B4E2D7R1_FAIL_TOPOLOGY_GATE / NSR3B4E2D7R2_RESEARCH / REPORT_ONLY`

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
D4 passes: the owned identity error is `5e-21--1.3e-18`, while the legacy
error exceeds the whole fine residual. Owned-gradient trials remove both fine
overshoots and improve all six states; four still require another correction.
This selects `BOUNDED_OWNED_RESIDUAL_ITERATION_REQUIRED`; see the
[dated evidence](../../development/nonlocal-nsr3b3d4-owned-gradient-evidence-2026-08-21.md).
Freeze a capped full-trajectory D5 candidate next.
The fully owned, four-accept maximum candidate is frozen in
[NSR3-B3D5](03b3d5-owned-residual-trajectory-contract.md). It has no residual
line search and fails closed on non-decrease or a fifth required merit step.
D5 passes all six trajectories with at most two floor accepts per solve, exact
topology and all reaction/arithmetic/correspondence gates; see the
[dated evidence](../../development/nonlocal-nsr3b3d5-owned-residual-trajectory-evidence-2026-08-21.md).
This authorizes a separately frozen B3R composition retry only.
That retry is frozen in
[NSR3-B3R](03b3r-owned-boundary-composition-contract.md). It reuses every B3
fixture, adaptive/reference schedule and strict ledger, changing only the
selected D5 numerical state/globalization in all executed substeps.
B3R passes both adaptive compositions, fixed reference convergence, strict
ledger and final accuracy gates; see the
[dated evidence](../../development/nonlocal-nsr3b3r-owned-boundary-composition-evidence-2026-08-21.md).
This selects `STATIC_BOUNDARY_SMOKE_CANDIDATE` and authorizes B4 physical-
corpus contract design only.
The [B4 research](../../development/nonlocal-nsr3b4-physical-corpus-research-2026-08-21.md)
separates pressure water, scalable/canonical execution, internal aperture,
viscous walls and surface calibration. Its first
[B4A eligibility gate](03b4a-closed-box-eligibility-contract.md) passes exact
two/three-layer closed-box topology, free-surface separation and all six
analytical contact faces; see the
[dated evidence](../../development/nonlocal-nsr3b4a-closed-box-eligibility-evidence-2026-08-21.md).
The nominal all-pairs projection reaches `52.9M--2.338B` candidate checks per
objective evaluation, so joint fluid/support cell neighborhoods are mandatory
before nominal execution. The bounded pressure-only corpus is now frozen in
[B4B](03b4b-tiny-pressure-corpus-contract.md): supported-column startup and a
separate released-block/floor-impact phase, each with fixed `48/96/192`
references and no viscosity/surface term. Viscosity, surface tension, internal
aperture, nominal execution, CUDA and runtime remain blocked.
B4B fails reproducibly at the first P1 fixed-96 substep; see the
[dated evidence](../../development/nonlocal-nsr3b4b-tiny-pressure-corpus-evidence-2026-08-21.md).
The adaptive trajectory alone is not accepted. Its 96/192 references expose a
unilateral pressure-topology transition below the inherited energy floor when
ghost support and a post-solve hard wall both represent the same boundary.
Preserve all thresholds and research constrained contact KKT stationarity
before assigning a new corpus identity.
The KKT derivation and alternatives are recorded in the
[B4BK research note](../../development/nonlocal-nsr3b4bk-contact-kkt-research-2026-08-21.md).
Its [frozen discriminator](03b4bk-contact-kkt-discriminator-contract.md)
replays the exact failed P1 first steps, tests a feasible bound-constrained
stationarity/impulse ledger and retains detached P2 free flight as a negative.
A full B4B retry remains blocked.
B4BK r0 preserves exact split failures and passes its constrained KKT/ledger
states, but the report fails on an incorrect no-lateral-multiplier hypothesis;
see the [dated evidence](../../development/nonlocal-nsr3b4bk-contact-kkt-evidence-2026-08-21.md).
The full cross-section physically touches both x/z walls. Freeze the sole
[B4BK1 face-symmetry repair](03b4bk1-contact-face-symmetry-contract.md) with
exact face counts and unchanged numerical gates, then execute it before any
trajectory retry.
B4BK1 passes exact face populations, signed lateral symmetry, all inherited
KKT/ledger gates and detached P2; see the
[dated evidence](../../development/nonlocal-nsr3b4bk1-contact-face-evidence-2026-08-21.md).
The full [B4B1 contract](03b4b1-tiny-pressure-contact-kkt-contract.md) is now
frozen: it changes only post-solve sweep to constrained contact inside every
substep and inherits the complete B4B corpus without looser thresholds.
Nominal/scalable work remains blocked.
B4B1 closes KKT physics and all fixed references but fails the first adaptive
frame's fixed-192 kinetic comparison; see the
[dated evidence](../../development/nonlocal-nsr3b4b1-tiny-pressure-contact-kkt-evidence-2026-08-21.md).
The pressure-inactive start chooses `n=1`, and its passing `1/2` pair misses
contact-created stiffness. The selected
[B4BF research](../../development/nonlocal-nsr3b4bf-contact-forecast-research-2026-08-21.md)
freezes a [feasible-predictor spectrum discriminator](03b4bf-contact-forecast-controller-contract.md)
before any full retry. The kinetic gate remains unchanged.
B4BF passes: the feasible predictor selects the derived `21/42` pair, whose
fine member is `2.05%` from fixed-192 kinetic energy, while detached P2 keeps
the exact zero-HVP path; see the
[dated evidence](../../development/nonlocal-nsr3b4bf-contact-forecast-evidence-2026-08-21.md).
The full [B4B2 contract](03b4b2-tiny-pressure-contact-forecast-contract.md)
is frozen with this sole initial-substep repair. B4B2 passes both complete
pressure/contact trajectories, every fixed-reference, KKT, ledger, physical
and work gate, and three byte-identical reports; see the
[dated evidence](../../development/nonlocal-nsr3b4b2-tiny-pressure-contact-forecast-evidence-2026-08-21.md).
This selects `TINY_PRESSURE_CONTACT_FORECAST_CANDIDATE` and authorizes B4C
joint fluid/support neighborhood plus canonical-runner design only. Nominal
execution remains blocked until B4C freezes and passes correspondence,
capacity and publication-identity gates.
The [B4C research](../../development/nonlocal-nsr3b4c-scalable-runner-research-2026-08-21.md)
shows that membership, pressure tape, solver substitution and canonical
publish/decode continuation have independent failure modes. The first
[B4C0 contract](03b4c0-joint-neighborhood-contract.md) is frozen for exact
joint fluid/support membership, reduction order and capacity negatives.
B4C0 preserves exact pair/evaluation/HVP/permutation results and every typed
capacity negative, but fails the P1 work gate: its exact-count plus fill scans
perform `36,240` distance tests versus `27,240` all-pairs checks; see the
[dated evidence](../../development/nonlocal-nsr3b4c0-joint-neighborhood-evidence-2026-08-21.md).
Preserve this failure and freeze a one-pass pre-admitted-workspace revision.
B4C1--B4C3 and all nominal execution remain blocked.
The [B4C0R research](../../development/nonlocal-nsr3b4c0r-one-pass-workspace-research-2026-08-21.md)
selects a [single-pass revision](03b4c0r-one-pass-neighborhood-contract.md):
reserve the declared maximum pair payload before the query, emit once and
clear all private output on overflow. It changes no membership or physics.
B4C0R passes every inherited exact gate and reduces executed distance checks
to `0.665x` all-pairs for P1 and `0.237x` for P2; see the
[dated evidence](../../development/nonlocal-nsr3b4c0r-one-pass-neighborhood-evidence-2026-08-21.md).
This selects `JOINT_PRESSURE_NEIGHBORHOOD_CANDIDATE` and authorizes only B4C1
compact CSR/pressure-tape design. No solver substitution or nominal run is
authorized.
The [B4C1 research](../../development/nonlocal-nsr3b4c1-pressure-tape-research-2026-08-21.md)
rejects copying the full multi-term A2 record into pressure-only support
states. The [B4C1 contract](03b4c1-compact-pressure-tape-contract.md) instead
freezes a compact CSR of pair indices, one exact radius per unique pair and
one compression per fluid centre. B4C1 passes exact CSR/radius/compression,
four-direction full reaction HVP, permutation, inactive and typed-capacity
gates. Across the active controls, three HVPs reduce exact norm/sqrt work from
`24,846` to `3,626` and from `22,494` to `3,682`; see the
[dated evidence](../../development/nonlocal-nsr3b4c1-pressure-tape-evidence-2026-08-21.md).
This selects `JOINT_PRESSURE_RADIUS_TAPE_CANDIDATE` and authorizes only B4C2
one-substep current/trial/forecast solver-query substitution design. Full
trajectory and nominal execution remain blocked.
The [B4C2Q audit](../../development/nonlocal-nsr3b4c2q-query-substitution-research-2026-08-21.md)
separates private query lifecycle from adaptive continuation. Its
[frozen contract](03b4c2q-query-substitution-contract.md) requires exact
forecast/current/trial identities, full KKT one-substep results, atomic
accepted/rejected workspace ownership and zero all-pairs calls in the
candidate path. B4C2Q passes every gate: four complete one-substep results and
all KKT/reaction/ledger/counter fields are bit-exact, the active forecast keeps
the exact 48-call spectrum, the detached forecast remains zero-HVP and forced
rejection leaves current state exact; see the
[dated evidence](../../development/nonlocal-nsr3b4c2q-query-substitution-evidence-2026-08-21.md).
This selects `JOINT_PRESSURE_KKT_QUERY_CANDIDATE` and authorizes only B4C2T
full-controller substitution design.
The [B4C2T research](../../development/nonlocal-nsr3b4c2t-controller-substitution-research-2026-08-21.md)
freezes the [complete controller contract](03b4c2t-controller-substitution-contract.md):
run the entire adaptive and fixed-reference B4B2 corpus with audit disabled,
compare its exact physical/report state to an independent all-pairs oracle and
reduce private query identities into a bounded chain digest. B4C3 remains
blocked until this full binary64 continuation gate passes. B4C2T passes: both
complete cases retain byte-identical physical/report state, exact adaptive and
fixed trajectories and zero candidate/audit all-pairs calls; see the
[dated evidence](../../development/nonlocal-nsr3b4c2t-controller-substitution-evidence-2026-08-21.md).
This selects `JOINT_PRESSURE_B4B2_CONTROLLER_CANDIDATE` and authorizes B4C3
canonical transaction design only. Its `14,149/11,860` workspace counts also
freeze committed-workspace/static-index optimization as mandatory B4C4 work
before B4D nominal execution.
The [B4C3 audit](../../development/nonlocal-nsr3b4c3-canonical-transaction-research-2026-08-21.md)
shows that canonical decode inside an adaptive level and global publication of
that level are separate transactions. The first
[B4C3A contract](03b4c3a-canonical-stage-contract.md) freezes one-frame
P1/P2 level-local staging, exact fine-only commit, roundtrip/order roots and
failure atomicity before any full canonical-controller physical bounds.
B4C3A passes both selected one-frame controls: every continuation input equals
the prior decoded canonical frame, only the fine level commits, repeat and
publication-order roots are exact, physical differences remain within the
frozen bounds and all four failure paths are atomic; see the
[dated evidence](../../development/nonlocal-nsr3b4c3a-canonical-stage-evidence-2026-08-21.md).
This selects `JOINT_PRESSURE_CANONICAL_STAGE_CANDIDATE` and authorizes only
B4C3T full canonical-controller physical-bound design. B4C4 and B4D remain
blocked.
The subsequent [B4C3Q audit](../../development/nonlocal-nsr3b4c3q-conservative-quantization-research-2026-08-21.md)
finds that independent per-sample rounding adds an unreported aggregate
position/momentum perturbation after the KKT ledger is closed. The
[frozen discriminator](03b4c3q-balanced-quantization-contract.md) compares it
with exact deterministic aggregate-balanced apportionment, including
adversarial algebra, one-frame P1/P2 physics and long free-flight drift. B4C3T
is now blocked until B4C3Q selects a publication policy and B4C3A1 revalidates
the transaction under the resulting new profile identity.
B4C3Q passes its exact algebra, physical, temporal and failure gates twice
byte-identically; see the
[dated evidence](../../development/nonlocal-nsr3b4c3q-balanced-quantization-evidence-2026-08-21.md).
It selects `CANONICAL_AGGREGATE_BALANCED_CANDIDATE`: aggregate error and the
residual-stress center drift improve `49x`, at the explicit cost of increasing
the local bound from half a unit to below one unit. Only B4C3A1 selected-policy
transaction design is authorized next.
The [B4C3A1 audit](../../development/nonlocal-nsr3b4c3a1-publication-ledger-research-2026-08-21.md)
separates the physical KKT transition from the subsequent deterministic
representation transition. Its
[frozen contract](03b4c3a1-balanced-stage-ledger-contract.md) requires
fine-only atomic frame/ledger commit, explicit quantization impulse and center
shift, compensated momentum closure and a decomposed kinetic/pressure/gravity
publication-energy record. It intentionally leaves the pressure-energy
long-horizon cap to B4C3T evidence rather than fitting one before measurement.
B4C3A1 passes twice byte-identically; see the
[dated evidence](../../development/nonlocal-nsr3b4c3a1-publication-ledger-evidence-2026-08-21.md).
The raw decoded-state residual reaches `9.98e-6`, while subtracting the explicit
publication impulse reproduces the KKT ledger at `4.67e-10` or below. Fine-only
frame/ledger commit, rollback, order, energy decomposition and P1/P2 physical
gates pass. This selects `CANONICAL_BALANCED_STAGE_LEDGER_CANDIDATE` and
authorizes B4C3T design only.
The [full-controller split](../../development/nonlocal-nsr3b4c3t-full-controller-split-research-2026-08-21.md)
decomposes the long run into adaptive ownership (B4C3TA), fixed canonical
convergence (B4C3TR) and their final physical comparison (B4C3TC). The first
[B4C3TA contract](03b4c3ta-adaptive-canonical-controller-contract.md) freezes
complete P1/P2 adaptive lanes, global step/root continuity, post-commit
rollback, exact P2 pressure-onset schedule, a binary envelope derived from
B4C3A1 with `8/32` position/velocity safety factors, and a `1%` independent
publication-energy budget.
Its first execution is a preserved FAIL; see the
[dated evidence](../../development/nonlocal-nsr3b4c3ta-adaptive-canonical-evidence-2026-08-21.md).
P1's failed 16-substep candidate recovers at 32 and 64 substeps, and that
adjacent pair passes the unchanged embedded gate. P2 independently revealed
that the harness used an exact-zero precontact velocity predicate despite the
frozen local canonical allowance. B4C3TR remains blocked; only a separately
frozen B4C3TAR refinement-recovery repair is authorized.
The
[B4C3TAR contract](03b4c3tar-adaptive-refinement-recovery-contract.md)
now freezes exact `REJECT_LIMIT` classification, adjacent-pass selection,
attempted-work accounting, rollback and the pre-existing local canonical
free-flight bounds. Its implementation is the only authorized next step.
That implementation is now a preserved FAIL; see the
[dated evidence](../../development/nonlocal-nsr3b4c3tar-refinement-recovery-evidence-2026-08-21.md).
It successfully recovers two exact nonlinear failures and completes P2, then
finds a distinct non-monotonic ledger-normalization conflict at P1 frame seven.
The publication-compensated vector reproduces the KKT ledger, but a stricter
normalizer rejects a residual already accepted by the KKT gate. The next step
is a separately frozen ledger reclosure, not broader failure recovery or a
larger tolerance.
The
[B4C3L contract](03b4c3l-ledger-normalization-contract.md)
now freezes the isolated normalization discriminator. It keeps raw and
max-scaled residuals as mandatory diagnostics, gates the compensated physical
ledger only on the source KKT sum scale, and requires synthetic factor-two,
one-frame P1/P2 and exact frame-seven four-level controls before any B4C3A2
revalidation.
B4C3L passes twice byte-identically and selects
`CANONICAL_KKT_SCALE_LEDGER_CANDIDATE`; see the
[dated evidence](../../development/nonlocal-nsr3b4c3l-ledger-normalization-evidence-2026-08-21.md).
The strict legacy frame-seven pattern remains exact, all four KKT-scale stages
pass, and the unchanged 16/32 embedded gate passes. Only B4C3A2 one-frame
selected-policy ledger design is authorized next.
The
[B4C3A2 contract](03b4c3a2-kkt-scale-stage-ledger-contract.md)
freezes that one-frame revalidation. Canonical trajectory and legacy ledger
roots must remain B4C3A1-exact, while a new policy-ledger root binds both
normalizers/residuals, correspondence bounds and the B4C3L policy identity.
B4C3A2 passes twice byte-identically and selects
`CANONICAL_KKT_SCALE_STAGE_LEDGER_CANDIDATE`; see the
[dated evidence](../../development/nonlocal-nsr3b4c3a2-kkt-stage-ledger-evidence-2026-08-21.md).
It preserves canonical and legacy ledger roots, adds deterministic policy
roots, and passes atomicity/physical/negative gates. Only a new complete
adaptive recovery replay design is authorized.
The
[B4C3TAR2 contract](03b4c3tar2-combined-adaptive-replay-contract.md)
freezes that composition: exact reject-limit refinement plus KKT-scale ledger
admission, with unchanged P1/P2 horizons, binary/energy/schedule gates,
attempted-work accounting and atomic canonical/legacy/policy roots.
B4C3TAR2 passes twice byte-identically; see the
[dated evidence](../../development/nonlocal-nsr3b4c3tar2-combined-replay-evidence-2026-08-21.md).
It completes all eight P1 and sixteen P2 macro frames, recovers only the two
known exact reject-limit failures, preserves atomic roots and schedules, and
passes every pre-frozen physical and energy bound. This selects
`CANONICAL_BALANCED_ADAPTIVE_RECOVERY_KKT_LEDGER_CANDIDATE` and authorizes only
B4C3TR complete fixed canonical reference design.
The
[B4C3TR audit](../../development/nonlocal-nsr3b4c3tr-fixed-reference-research-2026-08-21.md)
separates physical time-discretization convergence from the deterministic
canonical representation floor. Its
[frozen contract](03b4c3tr-fixed-canonical-reference-contract.md) requires
independent canonical `48/96/192` lanes, same-level binary tubes, exact
canonical/legacy/policy roots, KKT-ledger and energy gates, and an explicit
order-or-forward-floor convergence classification. The six independent lanes
may execute concurrently with fixed report order; this changes harness
resource utilization, not solver semantics. B4C3TC remains blocked.
B4C3TR's isolated execution is a preserved FAIL; see the
[dated evidence](../../development/nonlocal-nsr3b4c3tr-fixed-reference-evidence-2026-08-21.md).
All six lanes pass solver, transaction and KKT-policy gates, and parallel lane
execution gives `3.06x` wall-time speedup. The blocking result is instead
fundamental: fixed microunit publication after every substep injects the
representation perturbation more frequently as `h` shrinks. P2 contact phase
error grows from `0.260 ms` at 48 to `1.063 ms` at 192, and canonical final
differences do not converge. B4C3TC remains blocked; only a publication-cadence
reclosure may follow.
The
[B4C3P cadence audit](../../development/nonlocal-nsr3b4c3p-publication-cadence-research-2026-08-21.md)
identifies the durable macro boundary, rather than private nonlinear substep,
as the candidate canonical transaction. Its
[frozen contract](03b4c3p-publication-cadence-contract.md) compares the exact
per-substep FAIL control with private binary64 `48/96/192` intervals followed
by one balanced macro publication. A new profile and macro-ledger policy bind
that changed state ownership. A PASS can authorize only adaptive transaction
redesign, not the previously planned B4C3TC comparison.
The first B4C3P execution is a preserved FAIL; see the
[dated evidence](../../development/nonlocal-nsr3b4c3p-publication-cadence-evidence-2026-08-21.md).
Macro-only publication restores observed first-order convergence and exact
contact timing/sets in both scenarios, and every private KKT/macro-ledger
transaction passes. Only P1 fixed-192 exceeds the pre-frozen `32*P*q` velocity
tube by `10.6%`. The formula omits propagation of a published position
perturbation through later pressure/contact dynamics. A separate stability
reclosure is required; the coefficient is not widened from this result.
The
[B4C3PE audit](../../development/nonlocal-nsr3b4c3pe-stability-diagnostic-research-2026-08-21.md)
freezes a threshold-free decomposition of direct publication error, propagated
macro-map error and fine-192 contamination relative to independent binary
96/192 temporal error. Its
[measurement contract](03b4c3pe-stability-diagnostic-contract.md) changes no
gate and cannot reclassify B4C3P; it may only authorize a separately frozen
stability-budget design.
B4C3PE passes twice with exact decomposition and parent replay; see the
[dated evidence](../../development/nonlocal-nsr3b4c3pe-stability-diagnostic-evidence-2026-08-21.md).
It rejects scalar macro-map gain because contact and near-zero starts make the
ratio ill-conditioned. Resolved temporal contamination and absolute physical
utilization instead support a mixed budget: at most half of resolved binary
temporal error, or 1% of the existing physical comparison scale. Only B4C3PE1
design is authorized; B4C3P remains FAIL.
The
[B4C3PE1 design](../../development/nonlocal-nsr3b4c3pe1-mixed-budget-research-2026-08-21.md)
freezes a [mixed stability contract](03b4c3pe1-mixed-stability-budget-contract.md):
each lane/frame/field receives at most half its resolved adjacent binary
temporal difference, or 1% of the existing physical accuracy scale when that
relative budget is unavailable or smaller than material representation noise.
Both canonical fields must still show observed first-order convergence; all
events, physical, ledger and transaction gates remain unchanged.
B4C3PE1 passes twice byte-identically; see the
[dated evidence](../../development/nonlocal-nsr3b4c3pe1-mixed-stability-budget-evidence-2026-08-21.md).
All 144 field/frame admissions are classified: 89 temporal, 55 absolute and
zero rejected. Both scenarios retain observed first-order convergence, exact
events, non-tube physics, roots and rollback while the legacy B4C3P tube FAIL
remains visible. The macro-boundary fixed reference candidate is selected and
only adaptive macro-transaction design is authorized.
The
[B4C3MA audit](../../development/nonlocal-nsr3b4c3ma-macro-adaptive-transaction-research-2026-08-21.md)
separates one-frame transaction ownership from long-horizon adaptive behavior.
Its [frozen contract](03b4c3ma-macro-adaptive-transaction-contract.md) keeps
spectrum, binary64 candidate levels, exact reject-limit recovery and adjacent
fine selection private; only the accepted endpoint is published once and owns
a macro ledger/root. B4C3MA must pass before any complete adaptive replay.
B4C3MA's isolated execution is a preserved FAIL; see the
[dated evidence](../../development/nonlocal-nsr3b4c3ma-macro-adaptive-transaction-evidence-2026-08-21.md).
All solver/selection/ledger/mixed-budget gates pass, but raw binary equality
loses 24 P1 upper-face memberships after canonical decode at only `2.78e-17 m`
coordinate difference and zero penetration. No epsilon is authorized; a new
canonical-integer topology discriminator must run first.
The
[B4C3MAG audit](../../development/nonlocal-nsr3b4c3mag-canonical-topology-research-2026-08-21.md)
selects exact canonical integer equality instead of an epsilon. Its
[frozen contract](03b4c3mag-canonical-topology-contract.md) requires KKT
terminal features to equal both private and decoded feature sets after exact
`quantize_position`, while raw equality, feature differences and penetration
remain diagnostics. Solver state and publication are unchanged.
B4C3MAG passes twice byte-identically; see the
[dated evidence](../../development/nonlocal-nsr3b4c3mag-canonical-topology-evidence-2026-08-21.md).
P1 retains all 64 KKT features exactly in canonical integer coordinates while
the 24-feature raw mismatch remains visible as the intended discriminator.
Both one-frame adaptive transactions, all negatives and rollback pass. Only
complete adaptive macro replay design is authorized next.
The
[B4C3MAR audit](../../development/nonlocal-nsr3b4c3mar-complete-adaptive-macro-research-2026-08-21.md)
keeps long-horizon composition separate from fixed-reference accuracy. Its
[frozen contract](03b4c3mar-complete-adaptive-macro-replay-contract.md) runs
all 8/16 frames with private recovery, one accepted macro publication, mixed
admission, canonical topology and macro ledger per frame. A PASS may authorize
only a later adaptive-versus-fixed comparison design.
B4C3MAR passes twice byte-identically; see the
[dated evidence](../../development/nonlocal-nsr3b4c3mar-complete-adaptive-macro-evidence-2026-08-21.md).
P1 completes at `296/444` accepted/attempted substeps with no recovery, versus
the old cadence's `364/563` and two recovery frames; P2 preserves `82/124` and
its exact onset schedule. All topology, mixed-budget, physical, energy, root,
work and post-first-commit rollback gates pass. Fixed comparison design is now
authorized, but its execution is not.
The
[B4C3MC0 audit](../../development/nonlocal-nsr3b4c3mc0-adaptive-fixed-diagnostic-research-2026-08-21.md)
introduces a threshold-free adaptive-versus-fixed measurement before any
accuracy budget is chosen. Its
[frozen contract](03b4c3mc0-adaptive-fixed-diagnostic-contract.md) aligns every
macro frame against fixed `48/96/192`, reports fine `96/192` temporal ratios,
physical-scale utilization, aggregates and contacts, and cannot select
accuracy. Only a later frozen budget may do so.
B4C3MC0 passes twice byte-identically; see the
[dated evidence](../../development/nonlocal-nsr3b4c3mc0-adaptive-fixed-diagnostic-evidence-2026-08-21.md).
P1's maximum position/velocity physical utilization is `0.00647/0.07028`.
P2 remains within `0.18631/0.79151`, but its resolved adaptive/fine ratios are
`60--100x` for position and `94x` for contact velocity. This authorizes only a
two-axis B4C3MC1 design: unchanged physical accuracy budgets plus an independent
temporal-reference classification, never an observed-ratio fit.
The
[B4C3MC1 design](../../development/nonlocal-nsr3b4c3mc1-adaptive-accuracy-budget-research-2026-08-21.md)
freezes that separation in an
[accuracy contract](03b4c3mc1-adaptive-accuracy-budget-contract.md). It reuses
the B4B state, aggregate, kinetic and one-adaptive-substep onset limits exactly,
while temporal evidence is classified as resolved ratio, floor coincidence or
stable-reference separation. A PASS may select accuracy only for the two tiny
research fixtures and authorize nominal-corpus design.
B4C3MC1 passes twice byte-identically; see the
[dated evidence](../../development/nonlocal-nsr3b4c3mc1-adaptive-accuracy-budget-evidence-2026-08-21.md).
All unchanged B4B budgets pass. Maximum utilization is `0.78125` in P1 onset
timing and `0.79151` in P2 velocity. Temporal evidence remains separate: P2
velocity contains 14 floor coincidences, one stable-reference separation and
one resolved ratio. The adaptive macro controller is selected only for P1/P2,
and only nominal-corpus design is authorized.
Roadmap re-audit keeps the mandatory B4C4 packaging gate before B4D reference
rehydration and any nominal trajectory. The
[B4C4M0 audit](../../development/nonlocal-nsr3b4c4m0-workspace-reuse-diagnostic-research-2026-08-21.md)
freezes a
[threshold-free diagnostic](03b4c4m0-workspace-reuse-diagnostic-contract.md)
over recorded one-macro P1/P2 query lifecycles. It will distinguish repeated
committed states, immutable support-index rebuilds, duplicate nested rows and
irreducible nonlinear trial states before choosing the B4C4 optimization.
The three DFSPH files are present and hash-exact again, but this is only an
availability preflight; B4D remains blocked.
B4C4M0 passes twice byte-identically; see the
[dated evidence](../../development/nonlocal-nsr3b4c4m0-workspace-reuse-diagnostic-evidence-2026-08-21.md).
It measures `327/12` P1/P2 one-macro builds. P1 has 195 distinct trial builds,
but every completed private substep causes an immediate equal-state diagnostic
rebuild (`63/3`). B4C4A will isolate retained accepted-workspace ownership;
immutable support indexing and flat-only CSR remain separate later stages.
The
[B4C4A design](../../development/nonlocal-nsr3b4c4a-retained-workspace-research-2026-08-21.md)
freezes a single-owner
[retention contract](03b4c4a-retained-workspace-contract.md). A successful KKT
solve lends its accepted workspace to exactly one same-state physical
diagnostic and then releases it. The one-macro work result is predeclared as
`327->264` builds for P1 and `12->9` for P2; all physical and durable roots
must remain exact.
B4C4A passes twice byte-identically; see the
[dated evidence](../../development/nonlocal-nsr3b4c4a-retained-workspace-evidence-2026-08-21.md).
It removes exactly `63/3` P1/P2 diagnostic rebuilds while every adaptive,
physical, topology, canonical and ledger value stays bit-exact. Ownership and
failure controls end with zero live workspaces. Only a separately frozen
complete adaptive/macro-fixed lane application is authorized; B4C4B/B4C4C and
B4D remain blocked.
The
[B4C4A1 design](../../development/nonlocal-nsr3b4c4a1-complete-retention-research-2026-08-21.md)
freezes a
[complete-lane contract](03b4c4a1-complete-retention-contract.md) before the
local optimization is composed over full adaptive and macro-fixed P1/P2
trajectories. Expected removed builds are derived from existing executed
substep schedules, not from candidate measurements. B4C4B/B4C4C remain
blocked until this correspondence passes.
B4C4A1 passes twice byte-identically; see the
[dated evidence](../../development/nonlocal-nsr3b4c4a1-complete-retention-evidence-2026-08-21.md).
Across all eight complete adaptive and macro-fixed lanes it removes exactly
the predeclared `444/124`, P1 `384/768/1536` and P2 `768/1536/3072` builds.
Every physical, schedule, topology, canonical, ledger and trajectory result
remains bit-exact; retention and forced rollback end with zero live ownership.
Only B4C4B immutable static-support-index design is authorized next.
The
[B4C4B audit](../../development/nonlocal-nsr3b4c4b-static-support-index-research-2026-08-21.md)
proves that the legacy combined-cell order factorizes exactly into canonical
fluid then canonical support ranges. Its
[frozen contract](03b4c4b-static-support-index-contract.md) keeps workspace
ownership and nested adjacency unchanged while predeclaring removal of
`143,072/9,728` P1/P2 support sort records. The one-macro discriminator is
threshold-free; B4C4C and complete-lane application remain blocked until it
passes.
B4C4B passes twice byte-identically; see the
[dated evidence](../../development/nonlocal-nsr3b4c4b-static-support-index-evidence-2026-08-21.md).
It preserves every one-macro physical/durable value and invalidation failure,
while reducing P1/P2 records admitted to sorting by `91.54%/86.96%`. Split
traversal also doubles ordered cell-range lookups, so complete-lane rollout is
deferred until a separately frozen interleaved timing discriminator measures
the net effect. B4C4C remains blocked.
The
[B4C4BM design](../../development/nonlocal-nsr3b4c4bm-static-support-timing-research-2026-08-21.md)
freezes a
[threshold-free timing contract](03b4c4bm-static-support-timing-contract.md)
over all `264/9` recorded one-macro query states. It uses three warmups and 21
alternating AB/BA paired rounds per fixture, validates exact outputs outside
timing and reports robust statistics without a fitted speed gate.
B4C4BM passes in three independent sequential processes; see the
[dated evidence](../../development/nonlocal-nsr3b4c4bm-static-support-timing-evidence-2026-08-21.md).
The deterministic corpus/checksum result repeats exactly. Candidate wins all
`63/63` paired rounds per fixture, with median process-level construction
speedup `1.1399x` P1 and `2.5952x` P2. Freeze complete-lane static-index
rollout next; this is not yet a whole-solver speed claim.
The
[B4C4B1 design](../../development/nonlocal-nsr3b4c4b1-complete-static-index-research-2026-08-21.md)
freezes one immutable index per complete adaptive or macro-fixed lane. Its
[contract](03b4c4b1-complete-static-index-contract.md) derives exact support
record removals from B4C4A1 workspace counts, retains every physical/root/
rollback gate and keeps B4C4C separate.
B4C4B1 passes twice byte-identically; see the
[dated evidence](../../development/nonlocal-nsr3b4c4b1-complete-static-index-evidence-2026-08-21.md).
All eight lanes build exactly one immutable index and preserve every B4C4A1
physical, query, retention and durable result. Predeclared removals through
`4,192,768` support records per lane are exact. B4C4C flat-only CSR design is
authorized next; B4D remains blocked.
The
[B4C4C audit](../../development/nonlocal-nsr3b4c4c-flat-adjacency-research-2026-08-21.md)
proves that the existing pressure tape contains every ordered adjacency datum
needed after initial evaluation. Its
[frozen contract](03b4c4c-flat-adjacency-contract.md) constructs canonical
CSR pair indices once, evaluates through them and transfers their ownership
to the tape. The isolated gate predeclares removal of `12,672/243` nested row
objects and both sets of row sorts; B4D remains blocked until execution.
B4C4C passes twice byte-identically in both isolated and full-parent modes;
see the
[dated evidence](../../development/nonlocal-nsr3b4c4c-flat-adjacency-evidence-2026-08-21.md).
It removes duplicate construction of `1,223,663/9,623` directed records and
both `12,672/243` row-sort passes while preserving the final tape, physics and
all durable roots exactly. Freeze timing and complete-lane rollout separately;
no whole-solver or production claim is authorized.
The
[B4C4CM design](../../development/nonlocal-nsr3b4c4cm-flat-adjacency-timing-research-2026-08-21.md)
freezes a
[threshold-free timing contract](03b4c4cm-flat-adjacency-timing-contract.md)
over the same recorded `264/9` states. It times neighborhood, evaluation and
tape creation together with exact preflight outside timing, three warmups and
21 alternating paired rounds. Complete-lane rollout waits for this evidence.
B4C4CM passes in three independent processes; see the
[dated evidence](../../development/nonlocal-nsr3b4c4cm-flat-adjacency-timing-evidence-2026-08-21.md).
The deterministic corpus/checksum result repeats exactly, and the candidate
wins all `63/63` paired rounds per fixture. Median process-level construction
speedup is `1.2274x/1.2055x` P1/P2. Freeze complete-lane ownership next; this
is not a whole-solver performance claim.
The
[B4C4C1 design](../../development/nonlocal-nsr3b4c4c1-complete-flat-adjacency-research-2026-08-21.md)
freezes complete adaptive/fixed ownership over the
[B4C4C1 contract](03b4c4c1-complete-flat-adjacency-contract.md).
Workspace counts predeclare `8,721--222,288` removed nested rows per lane and
one transfer per workspace; directed volume remains an exact measured
correspondence. B4D waits for this final packaging gate.
B4C4C1 passes twice byte-identically in probe and full-parent modes; see the
[dated evidence](../../development/nonlocal-nsr3b4c4c1-complete-flat-adjacency-evidence-2026-08-21.md).
All eight lanes and forced rollback preserve physical/durable results with one
transfer per workspace and zero final ownership. B4C4 packaging is complete;
re-attest B4D's frozen reference inputs next.
The [B4D audit](../../development/nonlocal-nsr3b4d-reference-reattestation-research-2026-08-21.md)
confirms that the formula, selected solver, macro-publication, tiny-accuracy,
packaging and W0I source identities remain exact, while all three external
`CWREFV1` payloads are currently absent. Its
[frozen contract](03b4d-reference-reattestation-contract.md) requires complete
file/header/profile hashes and one in-memory mutation rejection before any
trajectory. The implemented reader fails twice deterministically at
`CW-HYDRO-001:MISSING_ARTIFACT` without starting a trajectory; see the
[dated evidence](../../development/nonlocal-nsr3b4d-reference-reattestation-evidence-2026-08-21.md).
Preserve B4D FAIL and audit reproducible recovery of the exact external
generator lineage next. B4E remains blocked until all three exact files are
restored and B4D passes, or a new independently frozen reference profile
receives new roots and comparator evidence.
The read-only
[B4DR0 audit](03b4dr0-reference-recovery-audit-contract.md)
finds no exact payload, adapter diff, comparator source or binary in the
retained filesystem or shared Git object store; public exact-hash lookup is
also empty but remains diagnostic only. The
[dated evidence](../../development/nonlocal-nsr3b4dr0-reference-recovery-audit-evidence-2026-08-21.md)
selects `NEW_REFERENCE_PROFILE_REQUIRED`: historical W0I/W1 remains valid
evidence, but a fresh comparator cannot inherit its roots. Freeze a
reproducible B4DR1 external generator/profile before cloning or building
upstream code. B4E remains blocked.
The [B4DR1 design](../../development/nonlocal-nsr3b4dr1-reference-generator-research-2026-08-21.md)
selects a new-root external comparator with a tracked standalone adapter,
strict build/float manifest, content-addressed external outputs and a
cost-aware bootstrap/contact/24-step/full/attestation ladder. Its
[frozen contract](03b4dr1-reference-generator-contract.md) authorizes only the
R1A recursive pinned-upstream bootstrap next. No adapter trajectory or full
generation is authorized yet; B4E remains blocked.
R1A passes with byte-identical strict artifacts and a later verified
true-full-clone revalidation; see the
[bootstrap evidence](../../development/nonlocal-nsr3b4dr1a-external-bootstrap-evidence-2026-08-21.md)
and its
[provenance correction](../../development/nonlocal-nsr3b4dr1a-full-clone-provenance-correction-evidence-2026-08-21.md).
The strict binary64/no-AVX/no-FMA flags reach the main and nested dependency
commands, and all eight static artifacts match. One originally retained source
copy lacked a complete Git object database; that narrower claim is withdrawn
without changing the reproduced artifact or R1C1 report identities. Linked Git
worktrees are rejected because upstream's old revision probe reports
`HEAD-HASH-NOTFOUND`. Freeze the standalone R1B contact adapter and six-vector
contract next; no particle trajectory is authorized.
The [R1B design](../../development/nonlocal-nsr3b4dr1b-contact-adapter-research-2026-08-21.md)
freezes a standalone independent C++ contact/validation tool over the
[R1B contract](03b4dr1b-contact-adapter-contract.md). Six parent vectors plus
internal-face and one-ulp `t=0` sentinels close all outer/plane/edge/corner
branches, strict process/ABI preflight and structural root mutations before
any particle world exists. Implement only this self-test next; R1C and B4E
remain blocked.
Before implementation, the R1B contract was reclosed from rejected draft v1
to v2: outer controls use `[0,1]^3`, while aperture controls use the real
`[0,2] x [0,1] x [0,1]` orifice box. This prevents a legal pass beyond
internal wall `x=1` from being misclassified as an outer escape. The v1 root
has no implementation authority.
R1B passes; see the
[dated evidence](../../development/nonlocal-nsr3b4dr1b-contact-adapter-evidence-2026-08-21.md).
Two builds produce the same adapter ELF and normalized command root, two fresh
processes produce the same 1,730-byte report, all eight contact cases pass and
all environment mutations fail before contact. Freeze R1C manifests next;
no 24-step trajectory or B4E design is authorized yet.
The [R1C research](../../development/nonlocal-nsr3b4dr1c-trajectory-preflight-research-2026-08-21.md)
finds that the pinned upstream enables warm starts and hides divergence/last-
error convergence state. The
[frozen R1C contract](03b4dr1c-trajectory-preflight-contract.md) therefore
binds a minimal equation-preserving cold-start/diagnostics patch, exact
fluid/boundary/scenario projections and `CWREFV2` layout. Implement and attest
the manifest-only preflight first. A trajectory is conditionally authorized
only after that gate passes; R1D and B4E remain blocked.
The first R1C manifest implementation fails closed before Simulation creation:
the contract shortened `CW-DAMBREAK-001` to `CW-DAM-001` but retained roots
computed from the normative ID. Preserve the
[negative evidence](../../development/nonlocal-nsr3b4dr1c-manifest-preflight-negative-evidence-2026-08-21.md)
and use only the
[R1C1 reclosure](03b4dr1c1-manifest-identity-reclosure-contract.md). Repeat the
manifest-only gate twice; no trajectory is authorized by the failed run.
R1C1 manifest preflight passes; see the
[dated evidence](../../development/nonlocal-nsr3b4dr1c1-manifest-preflight-evidence-2026-08-21.md).
Two builds and reports are byte-identical, all exact geometry roots pass, the
R1B report is unchanged, and a forced manifest mismatch rejects before
Simulation creation. This authorizes only implementation of the frozen
24-step path using a fresh patched upstream clone. R1D and B4E remain blocked.
The first physical Hydro process then fails at `PRESSURE_NOT_CONVERGED` and
publishes no payload; see the
[negative evidence](../../development/nonlocal-nsr3b4dr1c-trajectory-negative-evidence-2026-08-21.md).
The cost-aware ladder stops before a repeat, Dam or Orifice. The failure report
omits the already available iteration/residual fields, so the
[R1C2 research](../../development/nonlocal-nsr3b4dr1c2-failure-observability-research-2026-08-21.md)
and [frozen contract](03b4dr1c2-failure-observability-contract.md) authorize
only canonical failure observability and one Hydro diagnostic process. No
solver tuning or R1C/R1D credit is authorized.
R1C2 confirms the failure is pressure-only on step 1: cap `100`, residual
`0x3fea7a64ac09a4ac` (`0.8274405823`) against threshold `0.1`; divergence
converges in one iteration with zero residual and timestep bits remain exact.
See the
[dated evidence](../../development/nonlocal-nsr3b4dr1c2-failure-observability-evidence-2026-08-21.md).
The [R1C3 research](../../development/nonlocal-nsr3b4dr1c3-pressure-cap-research-2026-08-21.md)
and [frozen sweep contract](03b4dr1c3-pressure-cap-sweep-contract.md) select a
one-step ascending cap discriminator next. It changes no other profile value,
writes no payload and grants no R1C/R1D authority.
R1C3 finds a monotone pressure curve and first convergence at iteration 220
when cap 300 is allowed; see the
[sweep evidence](../../development/nonlocal-nsr3b4dr1c3-pressure-cap-evidence-2026-08-21.md).
The accompanying lattice check shows the selected `0.000125` volume is nearly
unit-normalized, while upstream's 0.8 startup heuristic is intentionally about
20% underdense. The
[R1C4 research](../../development/nonlocal-nsr3b4dr1c4-pressure-cap-reclosure-research-2026-08-21.md)
and [frozen contract](03b4dr1c4-pressure-cap-reclosure-contract.md) therefore
change only pressure cap 100 to 300 under a new identity. Implement and run
paired short scenarios next; R1D remains blocked until all pairs pass.
R1C4 Hydro and Dam pairs pass byte-identically, but the first Orifice process
rejects after a converged solver step because the adapter derives analytical
`x_max=1.0` from the intentionally one-metre source-support boundary. See the
[negative/partial evidence](../../development/nonlocal-nsr3b4dr1c4-trajectory-evidence-2026-08-21.md).
The [R1C5 research](../../development/nonlocal-nsr3b4dr1c5-orifice-domain-reclosure-research-2026-08-21.md)
and [frozen contract](03b4dr1c5-orifice-domain-reclosure-contract.md) separate
analytical domain extent from boundary lattice width, keep all boundary roots
unchanged and require every pair to rerun under a new global identity. R1D is
still blocked.
R1C5 now passes all three corrected pairs byte-identically; see the
[dated evidence](../../development/nonlocal-nsr3b4dr1c5-trajectory-evidence-2026-08-21.md).
Orifice completes 24 steps with 28 final receiver samples while its source-
support boundary remains unchanged. This opens only R1D full external
generation; R1E, B4E, runtime and production authority remain blocked.
The [R1D research](../../development/nonlocal-nsr3b4dr1d-full-generation-research-2026-08-21.md)
shows that the short manifests cannot truthfully describe full schedules and
selects schedule-only scenario reclosure. The
[frozen R1D contract](03b4dr1d-full-generation-contract.md) binds exact
51/181/181-frame sizes, domain-separated q99/receiver roots, two waves of at
most three independent one-thread processes and verified content-addressed
publication. Implement its manifest-only gate and generator next.
R1D now passes all three full pairs; see the
[dated evidence](../../development/nonlocal-nsr3b4dr1d-full-generation-evidence-2026-08-21.md).
The six processes are byte-identical per scenario, Orifice ends with 1,172
receiver samples and three verified regular files live under the explicit
content-addressed external profile. Only R1E reader/profile contract design is
authorized next; B4E and every runtime/production claim remain blocked.
The [R1E research](../../development/nonlocal-nsr3b4dr1e-reference-attestation-research-2026-08-21.md)
selects a standalone reader rather than generator reuse. The
[frozen R1E contract](03b4dr1e-reference-attestation-contract.md) binds actual
generator source/build, payload, semantic and aggregate roots; descriptor-safe
path admission; full independent parse/re-encoding; and external plus decoded
mutation controls. Implement only that reader next. B4E execution remains
blocked.
R1E now passes; see the
[dated evidence](../../development/nonlocal-nsr3b4dr1e-reference-attestation-evidence-2026-08-21.md).
Two independent builds and two fresh attestation processes agree exactly; all
three complete references pass independent parse/reconstruction and all four
external negative fixtures reject. This selects only the new external DFSPH
reference candidate and opens B4E nominal-corpus contract design. B4E
execution, runtime integration, CUDA and production claims remain blocked.
The [B4E research](../../development/nonlocal-nsr3b4e-nominal-corpus-research-2026-08-21.md)
finds that the packaged solver has nominal capacities but only tiny P1/P2
entry points, and that an immediate full run would mix alignment, physics and
cost failures. The
[frozen B4E0 contract](03b4e0-nominal-alignment-contract.md) therefore admits
only Hydro/Dam zero-trajectory geometry, stable-ID, canonical aggregate and
flat-neighborhood preflight. Orifice remains B4O. Implement B4E0 next; no
reference curve or candidate trajectory is authorized yet.
B4E0 passes; see the
[dated evidence](../../development/nonlocal-nsr3b4e0-nominal-alignment-evidence-2026-08-21.md).
Hydro/Dam exact roots, initial aggregates and nominal flat neighborhoods agree
twice and remain within 118/117 neighbors. The zero-step probe takes about
0.4 s and 47 MiB, but this is not solver throughput. Hydro's nine active
centres are only `6.66e-16` positive strain and remain an explicit B4E1 cost
diagnostic. Design only the one-macro resource probe next.
The [B4E1S audit](../../development/nonlocal-nsr3b4e1s-spectrum-research-2026-08-21.md)
splits that probe again because Hydro's nine epsilon-active centres force 48
HVPs before KKT work. Its
[frozen contract](03b4e1s-hydro-spectrum-contract.md) measures the exact
nominal Lanczos path and derives whether the existing adjacent fine-level cap
can admit a macro solve. Implement and run only B4E1S next.
B4E1S passes; see the
[dated evidence](../../development/nonlocal-nsr3b4e1s-hydro-spectrum-evidence-2026-08-21.md).
The bit-exact 48-HVP estimate gives maximum eigenfrequency `499.4373 s^-1`
and 14 initial substeps, safely below the frozen capacity boundary of 96. Two
independent builds/processes agree exactly and use no all-pairs fallback. This
authorizes only B4E1M one-macro research and contract design; no KKT trajectory
or external-reference comparison has run.
The [B4E1M research](../../development/nonlocal-nsr3b4e1m-hydro-macro-research-2026-08-21.md)
maps the existing complete adaptive transaction onto exact nominal Hydro. Its
[frozen contract](03b4e1m-hydro-macro-contract.md) admits one step-1
transaction per fresh process over levels `14,28,56,112`, with retained flat
workspaces, fine-only publication and an external 900-second watchdog. No
reference file or second macro may be opened. Implement only B4E1M next.
B4E1M passes physically and deterministically; see the
[dated evidence](../../development/nonlocal-nsr3b4e1m-hydro-macro-evidence-2026-08-21.md).
It selects the 28-substep fine level with zero energy creation, strain
`4.55e-4` and exact ownership/root gates. Cost is not admissible: 48.8 s and
99% of one CPU core per first macro project one unrepeated Hydro+Dam pair to
about 26 machine-hours. B4E2 execution is therefore held. Profile B4EP before
selecting or implementing an optimization.
The [B4EP0 research](../../development/nonlocal-nsr3b4ep0-attribution-research-2026-08-21.md)
finds four live hypotheses: unconditional inner-state SHA/streaming, complete
cell/CSR/tape rebuilds, serial HVP traversal and 221 outer trials. Process
`perf` is unavailable under `perf_event_paranoid=4`; host policy remains
unchanged. The [frozen B4EP0 contract](03b4ep0-attribution-contract.md)
selects one external `-pg`/gprof run whose stdout must remain byte-identical to
B4E1M. Run attribution only; do not optimize yet.
B4EP0 passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep0-attribution-evidence-2026-08-21.md).
The exact-output profile assigns 41.36% self time to SHA-256, about 23.4% total
to HVP, 22.7% to neighborhood construction and 8.3% to evaluation. Hashing is
the largest safe first ablation, but its `~1.70x` Amdahl ceiling cannot close
the full gap. Design B4EP1 query-evidence separation next; retain topology and
HVP as required later stages.
The [B4EP1 research](../../development/nonlocal-nsr3b4ep1-query-evidence-research-2026-08-21.md)
separates transient proof hashing from physical computation without deleting
final publication/ledger roots. Its
[frozen contract](03b4ep1-query-evidence-contract.md) keeps full-state hashing
as the byte-exact default, introduces a work-only transaction policy and
requires exact B4E1M roots/counters plus three balanced Release timing wins.
Implement and A/B only B4EP1 next.
B4EP1 passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep1-query-evidence-evidence-2026-08-21.md).
The full-state oracle remains byte-exact, the work-only candidate repeats
across both builds and all three alternating pairs win with median `3.0168x`
speedup. Median one-macro wall is now 16.15 s and RSS is 62,016 KiB, but this
still leaves material HVP/topology cost. Only B4EP2 residual profiling and
design are authorized; B4E2, runtime, CUDA and production remain blocked.
The [B4EP2 research](../../development/nonlocal-nsr3b4ep2-residual-attribution-research-2026-08-21.md)
keeps four residual hypotheses live and selects an exact-output work-only
gprof run. The
[frozen B4EP2 contract](03b4ep2-residual-attribution-contract.md) binds the
B4EP1 implementation/source/output bytes and permits only one external
profile. Run attribution only; do not optimize or start B4E2 yet.
B4EP2 passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep2-residual-attribution-evidence-2026-08-21.md).
Workspace construction owns 58.33% inclusive sampled time, HVP 39.64% and
hashing only 0.46%. Topology alone and HVP are nearly tied, so no final
bottleneck is claimed. Design a bounded B4EP3 canonical-superset feasibility
audit that preserves the earlier P4 order-mismatch negative and proves the
current lexicographic-pair invariant before any timing candidate.
The [B4EP3 research](../../development/nonlocal-nsr3b4ep3-canonical-superset-research-2026-08-21.md)
preserves the P4 negative and freezes one untuned `0.04h` discriminator over
all 227 nominal query states. Its
[frozen contract](03b4ep3-canonical-superset-audit-contract.md) requires exact
filtered pair order/CSR, evaluation and tape, plus capacity, overhead, reuse
and negative-control gates. Implement and run this audit only; it is not yet a
cache optimization or timing claim.
B4EP3 passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep3-canonical-superset-evidence-2026-08-21.md).
All 227 states reproduce exact pairs/CSR/evaluation/tape. One initial list
covers 226 certified reuses with maximum candidate degree 122, 6.89% extra
pair visits and a 0.1998 construction-work ratio. This authorizes only B4EP3I
hot-path cache design/A-B; it is not yet a measured solver speedup.
The [B4EP3I research](../../development/nonlocal-nsr3b4ep3i-hotpath-cache-research-2026-08-21.md)
selects an optional transaction-local cache carried by the internal query
trace. Its [frozen contract](03b4ep3i-hotpath-cache-contract.md) preserves the
full parent and all default bytes, requires exact B4EP1 physics/cache work and
three balanced Release wins. Implement and A/B only this dedicated command.
B4EP3I passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep3i-hotpath-cache-evidence-2026-08-22.md).
The dedicated cached transaction performs one superset build and 225 certified
reuses with exact B4EP1 physics and unchanged B4EP1/B4EP3 report bytes. All
three Release pairs win; median paired speedup is `1.5899x` and median wall
falls from 16.71 s to 10.40 s. This selects
`HOTPATH_CANONICAL_SUPERSET_CANDIDATE` and authorizes only B4EP4 residual
profiling/design. B4E2, CUDA, runtime and production remain blocked.
The [B4EP4 research](../../development/nonlocal-nsr3b4ep4-cached-residual-attribution-research-2026-08-22.md)
keeps HVP, filtered workspace refresh and nonlinear bookkeeping as competing
residual hypotheses. Its
[frozen contract](03b4ep4-cached-residual-attribution-contract.md) selects one
exact-output GCC/gprof run and a predeclared `1.20x` leader rule. Execute that
profile only; do not implement the next optimization yet.
B4EP4 passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep4-cached-residual-attribution-evidence-2026-08-22.md).
The exact-output profile assigns 62.14% inclusive time to HVP and 35.52% to
complete cached workspace. HVP leads by `1.749x`, clearing the frozen rule;
only B4EP5 HVP research/design is authorized. No second workspace change,
B4E2, CUDA, runtime or production work is authorized.
The [B4EP5 research](../../development/nonlocal-nsr3b4ep5-hvp-coefficient-tape-research-2026-08-22.md)
finds 971,831,424 repeated invariant kernel-coefficient evaluations in the HVP
path, while allocator work receives no samples. Its
[frozen contract](03b4ep5-hvp-coefficient-tape-contract.md) selects only an
optional two-scalar pressure-tape extension with exact B4EP3I correspondence
and balanced Release timing. Implement and A/B that command only.
B4EP5 passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep5-hvp-coefficient-tape-evidence-2026-08-22.md).
The candidate preserves every frozen physics/root/counter fact and all three
old command bytes. All three timing pairs win; median wall falls from 10.61 s
to 8.50 s for `1.2482x` paired speedup. This selects the internal invariant
coefficient tape and authorizes only B4EP6 exact residual profiling/design.
B4E2, CUDA, runtime and production remain blocked.
The [B4EP6 research](../../development/nonlocal-nsr3b4ep6-coefficient-residual-attribution-research-2026-08-22.md)
resets attribution after coefficient caching. Its
[frozen contract](03b4ep6-coefficient-residual-attribution-contract.md)
requires one exact-output gprof run and a `1.20x` top-level leader. Execute
that profile only; if no leader clears the rule, instrument scoped internal
phases before any further optimization.
B4EP6 passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep6-coefficient-residual-attribution-evidence-2026-08-22.md).
Complete workspace leads HVP `1.3507x`; evaluation plus base pressure tape
then leads topology/CSR `2.5046x`. This selects only B4EP7 research/design of
an exact evaluation/base-tape mechanical discriminator. No implementation,
B4E2, CUDA, runtime or production authority is created.
The [B4EP7D research](../../development/nonlocal-nsr3b4ep7d-evaluation-tape-dataflow-research-2026-08-22.md)
finds duplicate pair radius, gradient-kernel and compression work across
evaluation and tape construction. Its
[frozen contract](03b4ep7d-evaluation-tape-dataflow-contract.md) adds only
post-tape derived counters and a dedicated audit command. Run that audit twice;
do not implement fusion or time it yet.
B4EP7D passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep7d-evaluation-tape-dataflow-evidence-2026-08-22.md).
The audit freezes 131,987,230 active directed records and shows that exact
fusion can remove 71.75% of radius evaluations and 60.63% of gradient-kernel
evaluations. This authorizes only a frozen B4EP7I exact fusion A/B contract.
The [B4EP7I research](../../development/nonlocal-nsr3b4ep7i-fused-evaluation-tape-research-2026-08-22.md)
selects one transaction-only flat-workspace fusion that preserves density and
gradient accumulation order. Its
[frozen contract](03b4ep7i-fused-evaluation-tape-contract.md) requires exact
old/candidate bytes and three balanced Release wins before retaining it.
B4EP7I passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep7i-fused-evaluation-tape-evidence-2026-08-22.md).
The fused transaction is bit-exact and wins all three pairs with median
`1.1111x` speedup, reducing median wall from 8.90 s to 8.00 s. The modest
margin selects only B4EP8 exact residual profiling/design, not production.
The [B4EP8 research](../../development/nonlocal-nsr3b4ep8-fused-residual-attribution-research-2026-08-22.md)
resets attribution after fusion. Its
[frozen contract](03b4ep8-fused-residual-attribution-contract.md) authorizes
one exact-output gprof run and no optimization. A missing `1.20x` leader routes
to scoped internal phase timing.
B4EP8 passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep8-fused-residual-attribution-evidence-2026-08-22.md).
Fused workspace and HVP are 3.61 s and 3.59 s (`1.0056x`), and gprof cannot
separate the inlined pair/center loops. This selects B4EP9 opt-in internal
phase timing and no optimization.
The [B4EP9 research](../../development/nonlocal-nsr3b4ep9-fused-phase-timing-research-2026-08-22.md)
defines a conservative Amdahl-ready partition. Its
[frozen contract](03b4ep9-fused-phase-timing-contract.md) adds opt-in
transaction-only timers and requires three semantically exact fresh processes.
Only a stable parallelizable fraction meeting the frozen `0.80/0.75/0.05`
median/minimum/range gate may authorize B4EP10 CPU-parallel architecture
research; no parallel implementation is authorized by B4EP9.
B4EP9 passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep9-fused-phase-timing-evidence-2026-08-22.md).
The conservative parallelizable fraction is 92.14--92.22%, with median
92.19% and range 0.08 percentage point. Exact old-command regressions remain
unchanged. This selects B4EP10 deterministic CPU-parallel architecture
research only; no thread implementation or speedup claim exists yet.
The [B4EP10 research](../../development/nonlocal-nsr3b4ep10-cpu-parallel-architecture-research-2026-08-22.md)
selects fixed logical partitions plus owner-computes gathers; atomics and
partial floating reductions are rejected. The first
[frozen contract](03b4ep10d-owner-computes-dataflow-contract.md) is a serial
dataflow audit over all 226 topology/evaluation and 459 HVP calls. Threads and
OpenMP linkage remain blocked until it passes.
B4EP10D passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10d-owner-computes-dataflow-evidence-2026-08-22.md).
All alternative topology/evaluation/HVP values are bit-exact with zero
mismatch, and the maximum added nominal payload is 28.19 MiB. This authorizes
only B4EP10I OpenMP contract research; no parallel result exists yet.
The [B4EP10I research](../../development/nonlocal-nsr3b4ep10i-openmp-implementation-research-2026-08-22.md)
selects a research-only OpenMP backend with 64 fixed logical partitions and
explicit `1/2/4/8/16` worker commands. Its
[frozen contract](03b4ep10i-owner-parallel-contract.md) requires exact
cross-count correspondence and fail-closed worker negatives before a separate
B4EP10S timing stage.
B4EP10I passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10i-owner-parallel-evidence-2026-08-22.md).
All `1/2/4/8/16` commands reproduce one common correspondence hash, all
executor mismatch counts are zero and the 16-worker repeat is byte-identical.
This authorizes only B4EP10S serialized scaling design; no speedup has yet
been measured.
The [B4EP10S design research](../../development/nonlocal-nsr3b4ep10s-scaling-design-research-2026-08-22.md)
selects three serialized balanced rounds on distinct physical cores. Its
[frozen contract](03b4ep10s-owner-parallel-scaling-contract.md) chooses the
smallest count within 3% of the fastest median only after exactness, 3/3 wins,
10% speedup, utilization and stability gates. B4EP10S is host-specific and
authorizes at most a selected-count residual profile.
B4EP10S passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10s-owner-parallel-scaling-evidence-2026-08-22.md).
Eight workers are selected at `1.237020x` median same-round speedup and 6.421
effective cores. Sixteen workers are only 1.70% faster while using nearly
twice the CPU. This authorizes only B4EP10R selected-count residual profile
research; one macro still takes 5.91 s.
The [B4EP10R research](../../development/nonlocal-nsr3b4ep10r-selected8-profile-research-2026-08-22.md)
selects one unmodified gprofng clock/synchronization profile because Linux
perf remains policy-blocked and classic gprof cannot reliably own worker
samples. Its [frozen contract](03b4ep10r-selected8-profile-contract.md)
routes only to persistent-region research, one `1.20x` CPU-category leader or
scoped internal timing.
B4EP10R fails; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10r-selected8-profile-evidence-2026-08-22.md).
The target remains exact, but the collector reports that its sampling interval
changed and the profile may be unreliable. Raw libgomp/kernel percentages
receive no routing credit; B4EP10S remains valid and a separately frozen
internal timing stage is required.
The [B4EP10R1 research](../../development/nonlocal-nsr3b4ep10r1-internal-parallel-timing-research-2026-08-22.md)
selects opt-in hierarchical steady-clock timing plus per-worker active
intervals. Its
[frozen contract](03b4ep10r1-internal-parallel-timing-contract.md) separates
active, imbalance and orchestration capacity across the exact 3,411 regions
and keeps all old commands on the uninstrumented branch.
B4EP10R1 passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10r1-internal-parallel-timing-evidence-2026-08-22.md).
Executor orchestration and imbalance stay far below their routing thresholds.
Evaluation is the stable 54.63% leader and exceeds HVP by `1.916851x`; its
serial owner-plan/energy-fold subphase takes 42.33% of evaluation. Only
B4EP10P plan-architecture research is authorized next.
The [B4EP10P research](../../development/nonlocal-nsr3b4ep10p-masked-superset-plan-research-2026-08-22.md)
selects a fixed masked target CSR over repeated parallel rebuild. Its
[frozen B4EP10PD contract](03b4ep10pd-masked-superset-plan-audit-contract.md)
first requires exact stable-subsequence rows for all 226 plans, bounded full
scan expansion and a 64 MiB capacity gate. The returned solver path and
OpenMP implementation remain unchanged during this audit.
B4EP10PD passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10pd-masked-superset-plan-evidence-2026-08-22.md).
One fixed 705,284-slot plan reproduces every active target row. Its projected
full-scan ratio is `1.288507x` and conservative combined payload is 38.0 MB.
This authorizes only B4EP10PI opt-in implementation/A-B contract research;
no speed improvement has been measured.
The [B4EP10PI research](../../development/nonlocal-nsr3b4ep10pi-masked-plan-implementation-research-2026-08-22.md)
selects cache-owned fixed plan plus workspace-owned mapping with the canonical
energy fold unchanged. Its
[frozen contract](03b4ep10pi-masked-superset-plan-implementation-contract.md)
requires exact candidate work and three balanced external A/B pairs before
any speed credit.
B4EP10PI fails the speed gate; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10pi-masked-plan-evidence-2026-08-22.md).
It is physically exact, wins `3/3`, lowers RSS and has stable wall, but median
paired speedup is only `1.030796x` versus the frozen `1.05x`. The extra masked
scan raises CPU work; B4EP10I's active plan remains selected and parallel
active-plan construction is the next research direction.
The [B4EP10PC research](../../development/nonlocal-nsr3b4ep10pc-partitioned-active-plan-research-2026-08-22.md)
selects a stable counting-sort/CSR transpose with 64 fixed logical source
partitions. Its
[frozen B4EP10PCD contract](03b4ep10pcd-partitioned-active-plan-audit-contract.md)
compares all three plan arrays across all 226 states before the parallel
builder may enter a candidate solver path.
B4EP10PCD passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10pcd-partitioned-active-plan-evidence-2026-08-22.md).
All 226 stable partitioned transposes reproduce the serial active plans
exactly, both corrupt fixtures reject, and conservative combined payload is
37.3 MB. This authorizes only B4EP10PCI candidate-path/A-B contract research;
the audit itself carries no timing credit.
The [B4EP10PCI research](../../development/nonlocal-nsr3b4ep10pci-partitioned-plan-implementation-research-2026-08-22.md)
selects one opt-in replacement of serial active-plan construction while
retaining compact rows and canonical floating folds. Its
[frozen contract](03b4ep10pci-partitioned-active-plan-implementation-contract.md)
requires exact candidate work and three balanced external A/B pairs before
any speed credit. Implement and measure only this command.
B4EP10PCI fails the speed gate; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10pci-partitioned-plan-evidence-2026-08-22.md).
It is exact, wins `3/3` and is stable, but median paired speedup is only
`1.033650x` while user and system CPU increase. Keep it as negative evidence,
retain B4EP10I, and research a compression-independent current-topology
reverse plan before another implementation.
The [B4EP10CT research](../../development/nonlocal-nsr3b4ep10ct-current-topology-plan-research-2026-08-22.md)
selects a reverse plan over the already filtered topology, with compression
applied only as a stable gather mask. Its
[frozen B4EP10CTD contract](03b4ep10ctd-current-topology-plan-audit-contract.md)
first measures exact evaluation/HVP scan expansion and capacity while the
returned path remains unchanged. Implement only this audit.
B4EP10CTD fails the scan gate; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10ctd-current-topology-plan-evidence-2026-08-22.md).
Its exact `1.213341x` full-plan ratio exceeds `1.20x`, so topology construction
is not authorized. The evidence exposes a smaller split self/incoming view
with derived `1.106670x` visits; audit its exact fold order next.
The [B4EP10SI research](../../development/nonlocal-nsr3b4ep10si-split-incoming-plan-research-2026-08-22.md)
uses source rows for self contributions and retains only participant/incoming
reverse entries. Its
[frozen B4EP10SID contract](03b4ep10sid-split-incoming-plan-audit-contract.md)
requires exact three-part target-row reconstruction and the derived
`1.106670x` work before any topology builder is designed. Implement only the
audit.
B4EP10SID passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sid-split-incoming-plan-evidence-2026-08-22.md).
All 226 three-part row reconstructions are exact, both corrupt fixtures reject,
and projected visits are `1.106670x` the selected target entries with 32.1 MB
conservative payload. This authorizes only B4EP10SIC topology-construction
research, not a floating path or timing.
The [B4EP10SIC research](../../development/nonlocal-nsr3b4ep10sic-incoming-construction-research-2026-08-22.md)
selects pair-endpoint dual indexing plus support-target CSR to construct the
incoming view without sort or atomics. Its
[frozen B4EP10SICD contract](03b4ep10sicd-incoming-construction-audit-contract.md)
requires exact arrays across all 226 topologies before floating integration or
timing may be designed. Implement only this construction audit.
B4EP10SICD passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sicd-incoming-construction-evidence-2026-08-22.md).
All candidate incoming arrays match across 226 topologies, both corrupt
fixtures reject and conservative combined payload is 35.6 MB. This authorizes
only B4EP10SII floating integration/A-B contract research; no speed result
exists yet.
The [B4EP10SII research](../../development/nonlocal-nsr3b4ep10sii-split-incoming-integration-research-2026-08-22.md)
selects one opt-in floating split-fold path with topology-to-tape ownership and
the three audited construction regions left unfused. Its
[frozen contract](03b4ep10sii-split-incoming-integration-contract.md) requires
exact candidate work and three balanced A/B pairs before any speed credit.
Implement and measure only this command.
B4EP10SII passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sii-split-incoming-plan-evidence-2026-08-22.md).
The candidate is bit-exact, wins `3/3` pairs and clears the frozen gate at
`1.052521x` median paired speedup, reducing median wall from 5.820 s to
5.537 s. RSS falls by 4,852 KiB, but total user/system CPU rises. Retain
B4EP10I as rollback and freeze candidate-specific residual timing before
another construction or floating-work change. B4E2, broad corpus, runtime,
GPU, schema and production remain blocked.
The [B4EP10SIR research](../../development/nonlocal-nsr3b4ep10sir-split-incoming-residual-research-2026-08-22.md)
selects the existing hierarchical stage/subphase and worker-active timers over
the exact split-incoming command. Its
[frozen contract](03b4ep10sir-split-incoming-residual-timing-contract.md)
requires three exact fresh processes, stable disjoint category shares and a
predeclared leader rule before one next mechanical discriminator may be
selected. Implement and measure only this timing command.
B4EP10SIR passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sir-split-incoming-residual-timing-evidence-2026-08-22.md).
Source-local work has stable median share `0.587730` and leads topology by
`2.918189x`. Median executor orchestration and imbalance are only
`0.011919/0.071954`, rejecting persistent-region and partition-balance work.
Research one narrower source-local discriminator before changing code.
The [B4EP10SIRD research](../../development/nonlocal-nsr3b4ep10sird-source-local-discriminator-research-2026-08-22.md)
reduces the unchanged source-local subphases into directed, setup, compression
and local-scalar groups. Its
[frozen contract](03b4ep10sird-source-local-discriminator-contract.md)
requires three fresh exact processes and a stable `1.20x` leader before one
structural audit can be selected. Execute only this reduction; change no code.
B4EP10SIRD passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sird-source-local-discriminator-evidence-2026-08-22.md).
Directed evaluation/HVP work has median transaction share `0.389073`, range
`0.000870`, and leads setup by `4.094282x`. Because the phase also includes
full-buffer allocation/value-initialization, research one exact scratch-
liveness audit before selecting arithmetic changes or buffer reuse.
The [B4EP10SIRDA research](../../development/nonlocal-nsr3b4ep10sirda-directed-scratch-audit-research-2026-08-22.md)
defines a write/read certificate and high-water initialization projection for
the selected split incoming path. Its
[frozen contract](03b4ep10sirda-directed-scratch-audit-contract.md) requires
two exact byte-identical audit processes and both corrupt shadow negatives
before scratch-reuse implementation research. Implement only the audit.
B4EP10SIRDA passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sirda-directed-scratch-audit-evidence-2026-08-22.md).
All active-slot writes/reads are exact and a transaction-local high-water
buffer projects only 670,229 growth slots versus 454,936,226 repeated full
initializations. Research one opt-in directed-buffer reuse implementation/A-B
contract; do not fuse arithmetic or extend reuse to other phases.
The [B4EP10SIRDI research](../../development/nonlocal-nsr3b4ep10sirdi-directed-scratch-reuse-research-2026-08-22.md)
selects one transaction-local, directed-only high-water buffer with mandatory
release on every exit. Its
[frozen contract](03b4ep10sirdi-directed-scratch-reuse-contract.md) requires
exact work/lifetime counters and three balanced external A/B pairs before any
speed credit. Implement and measure only this command.
B4EP10SIRDI passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sirdi-directed-scratch-reuse-evidence-2026-08-22.md).
The candidate is exact, wins all three pairs and reduces median wall from
5.514 to 4.290 s (`1.284223x`) while median total CPU falls to `0.784012x`
baseline and RSS rises only 496 KiB. Select it for nominal research, keep both
rollbacks, and reprofile the exact candidate before another optimization.
The [B4EP10SIRDIR research](../../development/nonlocal-nsr3b4ep10sirdir-directed-scratch-residual-research-2026-08-22.md)
selects the existing hierarchical stage/subphase and worker-active timers over
the exact scratch-reuse command. Its
[frozen contract](03b4ep10sirdir-directed-scratch-residual-timing-contract.md)
requires three fresh exact processes, stable disjoint category shares and the
same predeclared leader rule before another change. Implement and measure only
this timing command.
B4EP10SIRDIR passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sirdir-directed-scratch-residual-timing-evidence-2026-08-22.md).
Source-local remains the stable leader at 44.72%, but the old directed group
no longer dominates after scratch reuse. `evaluation_setup` is the largest
individual source-local subphase at 14.58%; research a timing-only split of
validation, capacity/control and buffer preparation before changing any of
them.
The [B4EP10SIRDIRE research](../../development/nonlocal-nsr3b4ep10sirdire-evaluation-setup-research-2026-08-22.md)
identifies three distinct setup hypotheses. Its
[frozen contract](03b4ep10sirdire-evaluation-setup-timing-contract.md) adds
only nested validation/capacity/buffer timers, requires three exact fresh
processes and routes at most one later structural audit. Implement and measure
only this command.
B4EP10SIRDIRE passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sirdire-evaluation-setup-timing-evidence-2026-08-22.md).
Buffer preparation is the stable leader at 89.50% of `evaluation_setup`,
leading validation by `8.531659x`; all setup-share ranges remain below 0.009.
Research one timing-free write-before-read, ownership-lifetime and high-water
audit before designing any evaluation-buffer reuse.
The [B4EP10SIRDIREA research](../../development/nonlocal-nsr3b4ep10sirdirea-evaluation-buffer-liveness-research-2026-08-22.md)
separates six returned workspace buffers from one builder-local ephemeral
buffer. Its
[frozen contract](03b4ep10sirdirea-evaluation-buffer-liveness-audit-contract.md)
requires exact per-index write coverage, two workspace-lane receipts, one
ephemeral lane and bounded high-water projection. Implement and run only the
timing-free audit; no reuse path is authorized yet.
B4EP10SIRDIREA passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sirdirea-evaluation-buffer-liveness-evidence-2026-08-22.md).
All seven roles are fully overwritten; exact lifetime requires two returned
workspace lanes and one ephemeral lane. Projected growth is 22.07 MB versus
2.829 GB repeated initialization (`0.007801x`). Research and freeze one opt-in
reuse implementation/A-B contract; retain SIRDI as rollback.
The [B4EP10SIRDIREI research](../../development/nonlocal-nsr3b4ep10sirdirei-evaluation-buffer-overwrite-research-2026-08-22.md)
finds that an ordinary vector pool cannot realize the high-water projection
without repeated resize initialization or a wider logical-size redesign. Its
[frozen contract](03b4ep10sirdirei-evaluation-buffer-overwrite-contract.md)
instead selects candidate-only overwrite construction for the six fully
covered `double` roles, removing 97.73% of measured initialization bytes while
leaving sizes, ownership and `Vec3` unchanged. Implement and run only this
opt-in candidate and its frozen balanced A/B.
B4EP10SIRDIREI fails before A/B; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sirdirei-evaluation-buffer-overwrite-evidence-2026-08-22.md).
The user-allocator type makes unchanged SIRDI take 10.43 s versus its accepted
4.290 s median (`2.431x` regression); the candidate's relative 9.63 s result is
therefore invalid. The implementation was reverted. Retain SIRDI and research
only the builder-local density-contribution scratch lane, with no returned-
workspace representation change.
The [B4EP10SIRDIREP research](../../development/nonlocal-nsr3b4ep10sirdirep-density-contribution-scratch-research-2026-08-22.md)
selects the one builder-local pair buffer proven safe by SIRDIREA. Its
[frozen contract](03b4ep10sirdirep-density-contribution-scratch-contract.md)
keeps a transaction-local standard vector at high-water, changes no returned
storage and adds explicit baseline-health gates before relative A/B. Implement
and measure only this opt-in candidate.
B4EP10SIRDIREP fails; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sirdirep-density-contribution-scratch-evidence-2026-08-22.md).
The exact candidate wins `3/3` at median `1.194152x`, but baseline health and
candidate stability violate their frozen gates. The implementation was
reverted, SIRDI remains selected and the buffer-initialization branch is
stopped. Continue only from unchanged residual attribution.
The [B4EP10SIRDIREQ research](../../development/nonlocal-nsr3b4ep10sirdireq-default-path-drift-research-2026-08-22.md)
observes that exact current SIRDI uses materially more total CPU than its
accepted checkpoint after three dormant instrumentation/audit additions. Its
[frozen contract](03b4ep10sirdireq-default-path-drift-contract.md) compares
independent accepted/current source builds in balanced pairs. Execute this
qualification before any new residual optimization.
B4EP10SIRDIREQ closes `HOST_UNQUALIFIED`; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sirdireq-default-path-drift-evidence-2026-08-22.md).
Both source checkpoints remain exact and current/accepted ratios do not show a
regression, but accepted median `4.801273 s` misses its absolute health gate.
Do not bisect source or run short-margin wall A/B; research preemption-resistant
CPU-time attribution or use structural evidence until wall timing is qualified.
The [B4EP10SIRDIREQ1 research](../../development/nonlocal-nsr3b4ep10sirdireq1-cpu-time-residual-research-2026-08-22.md)
selects process CPU clocks for disjoint phase work and per-worker thread CPU
clocks for active intervals. Its
[frozen contract](03b4ep10sirdireq1-cpu-time-residual-contract.md) requires
exact clock/call/category accounting across three processes. Implement and run
only this attribution command; wall time grants no speed credit.
B4EP10SIRDIREQ1 passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sirdireq1-cpu-time-residual-evidence-2026-08-22.md).
All three exact reports pass, every category-share range is at most `0.024734`
and the GNU/internal CPU ratio median is `1.007535`. Topology owns median
process-CPU share `0.301239` and leads target fold by `1.629380x`. Research
and freeze exactly one timing-free topology structural audit next; this grants
no implementation, wall-speed, corpus or production authority.
The [B4EP10SIRDIREQ2 research](../../development/nonlocal-nsr3b4ep10sirdireq2-topology-incoming-fusion-research-2026-08-22.md)
selects fusion of split-incoming construction into existing topology metadata
and row-fill passes. Its
[frozen contract](03b4ep10sirdireq2-topology-incoming-fusion-audit-contract.md)
requires a byte-exact shadow plan while reducing standalone structural scans
from 665,142,896 entries to one 85,716,150-pair pass. Implement and run only
this timing-free audit.
B4EP10SIRDIREQ2 passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sirdireq2-topology-incoming-fusion-evidence-2026-08-22.md).
Both reports are byte-identical, all 226 plans match exactly, standalone work
falls to `0.128869x`, no parallel region is added and the combined payload is
21,758,020 bytes. Research and freeze one separate opt-in fused-plan consumer
contract; do not enable it or claim wall speed on the unqualified host.
The [B4EP10SIRDIREQ3 research](../../development/nonlocal-nsr3b4ep10sirdireq3-topology-incoming-fusion-candidate-research-2026-08-22.md)
selects direct publication through the existing neighborhood-to-tape ownership
path, not the Q2 audit trace. Its
[frozen contract](03b4ep10sirdireq3-topology-incoming-fusion-candidate-contract.md)
requires two exact candidate processes before one balanced process-CPU A/B.
Implement only these opt-in commands; CPU evidence cannot claim wall speed.
B4EP10SIRDIREQ3 fails and is reverted; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sirdireq3-topology-incoming-fusion-candidate-evidence-2026-08-22.md).
The fast path is exact and removes 678 regions, but wins only `1/3` CPU pairs;
median paired speedup is `0.983844x` and range ratio is `1.565388`. Retain
SIRDI. Qualify a less interference-sensitive measurement lane before another
performance implementation; do not rerun Q3.
The [B4EP10SIRDIREQ4 research](../../development/nonlocal-nsr3b4ep10sirdireq4-executor-adjusted-cpu-research-2026-08-22.md)
finds that Q1's whole-transaction instability is concentrated in process CPU
inside OpenMP regions but outside measured worker-active intervals. Its
[frozen contract](03b4ep10sirdireq4-executor-adjusted-cpu-qualification-contract.md)
derives `E = transaction - region + active-worker` from the unchanged Q1
command and requires three fresh stable reports. Execute only this
qualification; it grants no implementation or wall credit.
B4EP10SIRDIREQ4 fails; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10sirdireq4-executor-adjusted-cpu-evidence-2026-08-22.md).
Exact accounting passes, but adjusted CPU range is `1.031236x` versus the
frozen `1.03` gate. Do not repeat or round it into PASS. A dedicated/quiescent
performance lane is required before another CPU/wall candidate A/B.
The [B4E2R research](../../development/nonlocal-nsr3b4e2r-first-output-reference-slice-research-2026-08-22.md)
separates external reference slicing from multi-macro physics. Its
[frozen contract](03b4e2r-first-output-reference-slice-contract.md) selects a
new standalone extractor for Dam step 4 and Hydro step 24 with complete-file
hash admission, canonical micrometre aggregates and mutation controls.
Implement and execute only B4E2R next. It runs no trajectory and makes no
performance claim; B4E2D/H, broad corpus, runtime/GPU and production remain
blocked.
B4E2R passes; see the
[dated evidence](../../development/nonlocal-nsr3b4e2r-first-output-reference-slice-evidence-2026-08-22.md).
Two independent builds emit the same 1,977-byte report and executable. The
immutable Dam-step-4 and Hydro-step-24 canonical slice roots now exist.
Research and freeze a Dam-first B4E2D physical-pilot contract next; do not
start either trajectory yet.
The B4E2D pilot and D0--D3 diagnostics preserve exact decoded-reference
topology and stop at Dam step two: converged density strain
`0.0011747197409319732` exceeds the frozen `0.001` material limit. B4E2D4
shows finite-penalty compressibility, not premature temporal admission.
B4E2D5 selects unilateral PHR augmented pressure state, and B4E2D6 proves
multiplier convergence on the true one-DOF density path.
The full 24-coordinate
[B4E2D7 contract](03b4e2d7-al-dense-vector-contract.md) closes every
derivative, trust-inner and monotone cold control but fails pressure-state
commit stability; see the
[dated evidence](../../development/nonlocal-nsr3b4e2d7-al-dense-vector-evidence-2026-08-22.md).
Its dimensionless cold dual gate admits a state whose next absolute multiplier
update is `3.4975e-7 J`, above `1e-8 J`. Preserve the hard FAIL and research a
new dimensionally consistent commit/confirmation discriminator. Do not tune
`beta`, select semismooth, start Dam/Hydro, or claim performance/production.
The [B4E2D7R research](../../development/nonlocal-nsr3b4e2d7r-al-stable-commit-research-2026-08-22.md)
derives the dimensional mismatch and selects a two-update private commit
protocol. Its
[frozen contract](03b4e2d7r-al-stable-commit-contract.md) retains all D7
mathematics and exact first-eight roots, then requires absolute
`delta_lambda <= 1e-8 J`, position update `<=1e-8 dx` and one immediately
following confirmation before a single state commit. Implement/run only this
standalone oracle next; the 14-update cap has no performance credit.
B4E2D7R fails at a new, narrower boundary; see the
[dated evidence](../../development/nonlocal-nsr3b4e2d7r-al-stable-commit-evidence-2026-08-22.md).
The exact D7 prefix remains intact, but outer 9 accepts zero primal motion at
the fixed `1e-8` inner stationarity floor, applies another material dual
update, and the next inner solve reaches `REJECT_LIMIT`. No state commits.
Preserve this as `INNER_ACCURACY_FLOOR`; freeze failed-trial observability
before changing inner tolerance, merit arithmetic, beta or solver family.
The [B4E2D7R1 research](../../development/nonlocal-nsr3b4e2d7r1-inner-floor-diagnostic-research-2026-08-22.md)
separates inexact-inner scheduling from raw total-energy subtraction. Its
[frozen contract](03b4e2d7r1-inner-floor-diagnostic-contract.md) replays the
exact post-outer-9 failure and compares raw actual reduction with the same
per-term difference in factored form. Implement/run only this observability
command next; it cannot accept or commit a trial.
B4E2D7R1 fails closed at its frozen topology gate; see the
[dated evidence](../../development/nonlocal-nsr3b4e2d7r1-inner-floor-diagnostic-evidence-2026-08-22.md).
Raw and factored differences agree that the repeated Newton proposal raises
the objective, while all nine trials change active/pair topology. The trust
radius remains about 192 times larger than the physical step after the final
reject, so the reject cap has not yet tested a radius-binding proposal.
Preserve the FAIL and research/freeze one replay-only B4E2D7R2 discriminator
that identifies the exact topology delta and scans step scale under both live
and fixed-current topology. No formula, beta, tolerance, policy or state
change is authorized.
The [B4E2D7R2 research](../../development/nonlocal-nsr3b4e2d7r2-topology-step-research-2026-08-22.md)
shows why pair membership alone is insufficient for the C2 compact-support
kernel. Its
[frozen contract](03b4e2d7r2-topology-step-discriminator-contract.md)
adds exact set/horizon detail, a `2^0..2^-20` scale ladder and continuation of
the current radial/PHR branches. Implement/run only this replay command next;
route precedence is frozen and no trial may be accepted.
B4E2D7R2 fails only its positive-zero horizon bit gate; see the
[dated evidence](../../development/nonlocal-nsr3b4e2d7r2-topology-step-evidence-2026-08-22.md).
The implementation yields `W'(h)=-0.0`, which is numerically zero but not the
required positive-zero bit pattern. Preserve the FAIL. Research/freeze one
narrow signed-zero reclosure that changes no kernel or route precedence and
keeps the complete D7R2 report byte-exact.
The [B4E2D7R2R research](../../development/nonlocal-nsr3b4e2d7r2r-signed-zero-reclosure-research-2026-08-22.md)
selects numerical equality to zero plus the exact observed signed-zero bits.
Its [frozen contract](03b4e2d7r2r-signed-zero-reclosure-contract.md) requires
the unchanged complete D7R2 report and half-step facts before route selection.
Implement/run only this narrow reclosure next; do not canonicalize the kernel
or implement the anticipated trust-policy route.
B4E2D7R2R passes; see the
[dated evidence](../../development/nonlocal-nsr3b4e2d7r2r-signed-zero-reclosure-evidence-2026-08-22.md).
The unchanged signed-zero kernel is valid, current-branch continuation rules
out horizon crossing, and the topology-stable half step descends with ratio
`1.3285`. Research/freeze one B4E2D7R3 globalization discriminator comparing
backtrack reuse, step-norm-aware trust recomputation and the legacy first-bind
continuation. No solver-policy implementation is authorized yet.
The [B4E2D7R3 research](../../development/nonlocal-nsr3b4e2d7r3-globalization-policy-research-2026-08-22.md)
grounds the candidates in step-norm-aware trust updates and hybrid rejected-
direction line search. Its
[frozen contract](03b4e2d7r3-globalization-policy-contract.md) compares three
exact replay lanes with fixed precedence. Implement/run only this discriminator
next; no candidate may update state.
