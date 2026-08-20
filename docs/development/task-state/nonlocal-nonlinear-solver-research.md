# Nonlocal nonlinear solver research -- current task state

| Field | Value |
|---|---|
| Status | `ACTIVE / NSR3B1S_FAIL / NSR3B1S1_TANGENT_SPECTRUM_DESIGN` |
| Updated | `2026-08-20` |
| Task key | `nonlocal-nonlinear-solver-research` |
| Scope | Fundamental solver research over the verified Nonlocal variational objective, isolated from runtime and the stopped SISSM lineage |
| Definition of done | NSR0--NSR6 select a production-roadmap candidate or stop at an exact reproducible boundary |
| Authority | Working context only; Accepted architecture, SPEC-38/ADR-076 and the frozen stage contracts outrank this file |

## Resume in 60 seconds

- **Current conclusion:** NSR2-C2 passes all gates. At 512 particles the guarded
  floor stop reduces work from `26/13/153` outer/reject/HVP to `12/0/46` while
  retaining nanometric physical residual and exact oracle correspondence.
- **Current conclusion:** `outer-state-hessian-tape-v1` is bit-exact, remains
  inside its linear memory cap, improves build+HVP by `2.05x--2.43x` and total
  solve by `1.36x--1.55x` across 512--4096 particles.
- **Current conclusion:** the unscaled FCR cubic integrates to `1/8`; at
  `H=3dx` its infinite-lattice density is `0.125224*rho0`. The authors' code
  applies a missing fixed lattice normalization near `7.985668`.
- **Current conclusion:** the independent B0 diagnostic passes twice
  byte-identically and selects `FORMULA_RECLOSURE_REQUIRED`; all historical
  raw hashes remain unchanged.
- **Current conclusion:** `lattice-normalized-cubic-v1` passes density,
  gradient, HVP, dense, trust and A1/A2 tape correspondence under the new
  `nuv-variational-fcr2` identity; FCR1 hashes remain byte-identical.
- **Current conclusion:** B1 passes free flight, rigid translation, Galilean
  covariance and rotation objectivity, but the `7x7x7` relaxation has
  step-doubling ratio `0.8865` and therefore fails exactly as frozen.
- **Current diagnosis:** the tested acoustic Courant numbers are
  `8.25 / 4.13 / 2.06`; stability and nonlinear convergence did not establish
  trajectory accuracy.
- **Current conclusion:** the B1D main ladder reaches final ratios
  `q_x=1.559/1.744` and `q_v=1.554/1.741`, but the strict replicas that disable
  the energy-floor stop fail at `MINIMUM_TRUST_RADIUS`.
- **Current conclusion:** B1D1 exact-overlaps all five main levels; its D3
  floor-oracle differences are only `5.53e-5 / 5.35e-5` of temporal position /
  velocity differences, and D4 is bit-exact.
- **Current decision:** temporal stiffness is confirmed. The first-order-like
  regime appears below acoustic Courant about one and is clear below `0.516`.
- **Current conclusion:** linear `C<=0.25` passes every `0.99dx` case but all
  `0.98dx` cases exceed the normalized velocity limit; the high-stiffness case
  also exceeds the position limit. All cases remain self-convergent.
- **Current diagnosis:** normalized error is amplitude-dependent but nearly
  invariant across `kappa` scale, pointing to missing finite-state tangent
  stiffness rather than the `sqrt(kappa)` scaling itself.
- **Current action:** freeze a pressure-only maximum-eigenfrequency diagnostic
  with dense tiny and deterministic matrix-free correspondence.
- **Next gate:** B1S1 must decide whether a spectral local-frequency policy is
  well-defined and bounded before any new substep candidate. B2 stays blocked.
- **Do not retry:** old profile tuning, block/hybrid maps, Chebyshev radius or
  iteration sweeps, product-scale/CUDA work.
- **Runtime authority:** none.

## Decisions

### D-001 -- Separate identity

- **Observation:** FCR2 objective passes, while all admitted fast-SISSM paths
  stopped.
- **Decision:** Use `nuv-newton-krylov-r0`; preserve `nuv-variational-fcr1` as
  objective parent and historical report identity.
- **Consequence:** Solver reports and future performance roots are new; old
  timing grants no credit.

### D-002 -- Diagnose curvature before selecting a fast solver

- **Observation:** Pressure Hessian contains global `J^T J` coupling and a
  possibly indefinite geometric term not represented by FCR3-A blocks.
- **Decision:** NSR0 must prove an analytic matrix-free HVP against the verified
  gradient and a dense tiny oracle before NSR1.
- **Rejected:** implementing trust-region logic on an unverified Hessian;
  treating an approximate block as ground truth.

### D-003 -- Trust-region first, constrained reformulation conditional

- **Observation:** fixed Chebyshev becomes non-descent under changing
  pressure-bearing states.
- **Decision:** use full-HVP Steihaug--Toint trust-region Newton-CG as the first
  bounded optimizer discriminator. Consider primal-dual inequality pressure
  only if NSR1 evidence isolates penalty stiffness.
- **Rejected:** simultaneous solver/model changes, which would make a result
  uninterpretable.

### D-004 -- Reuse immutable outer-state curvature

- **Observation:** positions, active pressure centers and pair support do not
  change inside one trust-region outer state, but A1 recomputed their Hessian
  coefficients for every Krylov product.
- **Decision:** select `outer-state-hessian-tape-v1`; its construction is
  charged to total work and storage is bounded linearly in particles/pairs.
- **Evidence:** exact A1 correspondence, `2.05x--2.43x` combined build+HVP and
  `1.36x--1.55x` total speedup in two clean pinned campaigns.
- **Consequence:** use A2 for later report-only CPU research; retain A1 as the
  exact oracle. This grants no physical, GPU or runtime authority.

### D-005 -- Normalize the material kernel before selecting profiles

- **Observation:** the FCR1 raw cubic integrates to `1/8` and produces
  `0.125224338*rho0` on the `H=3dx` lattice; the authors' implementation uses
  a separate fixed lattice scale not present in FCR1.
- **Decision:** do not hide the deficit in `rho0`, mass or material strengths.
  Reclose one explicit common scale for `W`, `dW/dr` and `d2W/dr2` under the
  new `nuv-variational-fcr2` identity.
- **Consequence:** all FCR1 solver evidence remains valid for its synthetic
  objective but gives no physical-profile authority. NSR3-B1 stays blocked.

### D-006 -- Select the normalized FCR2 objective

- **Observation:** one common factor restores reference density while all six
  gradient/HVP/dense controls, both trust solves and A1/A2 correspondence pass.
- **Decision:** select `nuv-variational-fcr2` for later report-only CPU
  research; keep FCR1 available only under its old explicit commands.
- **Consequence:** coefficient anchors may now enter manufactured controls,
  but no hydro, boundary, CUDA or runtime claim is unlocked.

### D-007 -- Preserve the B1 temporal-convergence failure

- **Observation:** four invariance/objectivity controls pass, and all three
  compression runs are finite, conservative and solver-converged, but their
  final-position step-doubling ratio is `0.8865` rather than `>=1.5`.
- **Decision:** keep B1 FAIL and do not alter its time steps or threshold.
  Diagnose acoustic stiffness under a separately rooted B1D contract.
- **Consequence:** static-boundary and physical-trajectory work stays blocked;
  implicit stability cannot be used as evidence of temporal accuracy.

### D-008 -- Reject the floor-disabled sensitivity oracle

- **Observation:** the main acoustic ladder exhibits plausible first-order
  convergence, but both strict replicas hit `MINIMUM_TRUST_RADIUS` after the
  numerical-floor stop is disabled.
- **Decision:** B1D is invalid as a diagnostic; its main-ladder trend is not a
  selection result. Preserve it and test a floor-limited oracle under B1D1.
- **Consequence:** do not weaken the selected arithmetic stop or claim that
  smaller trust radii increase binary64 accuracy.

### D-009 -- Confirm temporal stiffness with the arithmetic floor retained

- **Observation:** floor-limited D3 differs from ordinary D3 by about `5e-5`
  of the D3--D4 temporal difference; D4 is bit-exact. Both oracle runs finish.
- **Decision:** select `TEMPORAL_STIFFNESS_CONFIRMED` as the B1D1 disposition
  and use acoustic Courant as the B1S policy variable.
- **Consequence:** do not treat implicit stability as accuracy. B1S must test
  the policy beyond the single amplitude/coefficient point before B2.

### D-010 -- Reject the linear acoustic policy at finite compression

- **Observation:** all six cases converge under refinement, but the 2%
  compression row exceeds the frozen velocity-error limit at every stiffness;
  its normalized error is nearly stiffness-scale invariant.
- **Decision:** reject `acoustic-courant-substeps-r0`. Measure the actual
  pressure tangent spectrum before proposing another policy.
- **Consequence:** do not hide amplitude dependence in a tuned global safety
  factor; B1S1 must expose its source and computational cost.

## Required context

1. `docs/architecture/agent-routing.md`, SPEC-38, ADR-076 and ADR-081.
2. `docs/plans/nonlocal-continuum-formula-reclosure/README.md` and
   `00-formula-contract.md`.
3. The stopped formula-reclosure task state and FCR3-B2 evidence.
4. `docs/development/nonlocal-nonlinear-solver-research-2026-08-20.md`.
5. `docs/plans/nonlocal-nonlinear-solver-research/README.md` and the current
   frozen stage contract.

## Exact next action

1. Freeze B1S1 pressure-only HVP and eigenvalue conventions.
2. Prove deterministic Lanczos/power estimates against a dense tiny Hessian.
3. Measure amplitude and `kappa` scaling on the six B1S initial states plus
   inactive/translation null controls.
4. Select/stop a spectral-policy premise; do not execute another trajectory
   policy until the spectrum result is fixed.

## Reconsideration triggers

- Pairwise Descent paper/code becomes public: compare only after its exact
  formula and identity are reviewable.
- NSR0 HVP mismatch: fix one derivation/transcription defect under the same
  contract; a second mismatch stops the branch.
- NSR1 correct-model but penalty-dominated cost: draft an independent
  constrained primal-dual formula contract.
