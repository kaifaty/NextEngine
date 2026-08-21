# Nonlocal nonlinear solver research roadmap

Status: `ACTIVE / NSR0_PASS / NSR1_PASS / NSR2B_PASS / NSR2C_FAIL / NSR2C1_PASS / NSR2C2_PASS / NSR3A_PASS / NSR3A1_PASS / NSR3A2_PASS / NSR3B0_PASS / NSR3B0R_PASS / NSR3B1_FAIL / NSR3B1D_INVALID / NSR3B1D1_PASS / NSR3B1S_FAIL / NSR3B1S1_PASS / NSR3B1S2_FAIL / NSR3B1S3_PASS / NSR3B1R_FAIL / NSR3B1R1_PASS / NSR3B2_PASS / NSR3B3_FAIL / NSR3B3D_PASS_CERT_REJECT / NSR3B3D1_FAIL / NSR3B3D2_PASS / NSR3B3D3_FAIL / NSR3B3D4_PASS / NSR3B3D5_PASS / NSR3B3R_PASS / NSR3B4A_PASS / NSR3B4B_FAIL / NSR3B4BK_FAIL / NSR3B4BK1_PASS / NSR3B4B1_FAIL / NSR3B4BF_PASS / NSR3B4B2_PASS / NSR3B4C3MC1_TINY_ACCURACY_PASS / NSR3B4C4M0_PASS / NSR3B4C4A_PASS / NSR3B4C4A1_PASS / NSR3B4C4B_PASS / NSR3B4C4BM_PASS / NSR3B4C4B1_PASS / NSR3B4C4C_PASS / NSR3B4C4CM_PASS / NSR3B4C4C1_PASS / NSR3B4D_MISSING_ARTIFACT / REPORT_ONLY`

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
R1A passes with two byte-identical fresh full-clone builds; see the
[bootstrap evidence](../../development/nonlocal-nsr3b4dr1a-external-bootstrap-evidence-2026-08-21.md).
The strict binary64/no-AVX/no-FMA flags reach the main and nested dependency
commands, and all eight static artifacts match across paths. Linked Git
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
