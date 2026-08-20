# Nonlocal nonlinear solver research -- current task state

| Field | Value |
|---|---|
| Status | `ACTIVE / NSR3B3D3_FAIL_ONE_TRIAL / NSR3B3D4_FROZEN_IMPLEMENTATION` |
| Updated | `2026-08-21` |
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
- **Current conclusion:** dense/Lanczos `lambda_max` agrees to `7.6e-16`;
  eigenvalues scale exactly with `kappa`, while finite-state amplification
  increases from `0.7041` to `0.7790` between the two amplitudes.
- **Current conclusion:** spectral target `0.15` passes all 1% cases but the
  2% row remains `0.05%--2.12%` above the normalized velocity threshold;
  spectra, convergence and every other physical gate pass.
- **Current decision:** spectrum is retained as an initial-step estimator, not
  selected as a standalone error policy.
- **Current conclusion:** B1S3 accepts the initial spectral count for every 1%
  case, doubles it for every 2% case, and also passes the unseen 3% holdout at
  92 accepted substeps with `0.000903905c` velocity difference.
- **Current cost:** the online step-doubling controller executes `3x` accepted
  substeps at depth zero and `3.5x` at depth one. The holdout discards 230 of
  322 controller substeps and 242 of 348 nonlinear HVP calls.
- **Current decision:** select `EMBEDDED_SPECTRAL_ERROR_CONTROLLER_R0` only for
  report-only CPU research. It is an accuracy mechanism, not yet an efficient
  production policy.
- **Current action:** implement the frozen B1R transactional composition over
  the original `0.05 s` horizon and fixed `96/192/384` reference ladder.
- **Current constraint:** inactive pressure is not an inertia-only state while
  bulk viscosity remains selected. The B1R one-step fast path additionally
  requires zero relative bond velocity; relaxation frames use measured `1/2`
  refinement instead.
- **Current conclusion:** all B1R local transactions and independent
  references pass, but coarse-state composition ends at `0.072--0.134dx`
  position error; the first active-frame velocity error propagates through the
  remaining horizon.
- **Current decision:** reject `TRANSACTIONAL_COARSE_STATE_R0`. A local error
  gate is not a global trajectory bound when the coarse state owns commit.
- **Current conclusion:** B1R1 passes all full-horizon cases at
  `0.0296--0.0468dx` position and `0.000305--0.000483c` velocity error while
  executing exactly the same work as failed B1R.
- **Current cost:** fine ownership reduces discarded substeps from
  `100/237/252` to `50/140/149`; depth-zero/one execution multipliers are
  `1.5x/1.75x` accepted work.
- **Current decision:** select `NSR_MULTISTEP_CANDIDATE`. The coarse trajectory
  is an error probe; the fine trajectory owns the committed transition.
- **Current conclusion:** the Nonlocal paper leaves fluid-solid collision to
  future work. The 2026 semi-analytical boundary energy belongs to a separate
  SISPH identity and explicitly lacks feedback-force modeling.
- **Current decision:** B2 uses split static support/contact. Fixed samples
  enter only fluid-centered pressure density; hard swept contact and its
  reaction remain a separate nonsmooth operation.
- **Current conclusion:** B2 passes gradient, HVP, dense-Hessian, translation,
  virtual-reaction, capacity and independent contact gates. Its semantic/raw
  hashes are `80a01b2e...f80` / `d6ba5f8e...9d9`.
- **Current conclusion:** two and three layers agree exactly on the compressed
  corner and face-slab controls. The third shell contributes no pair because
  `W(H)=W'(H)=W''(H)=0`; the two-layer candidate remains bounded at 454
  samples in the largest derivative fixture.
- **Current conclusion:** the support pressure Hessian remains indefinite
  (`lambda_min=-395/-3347` on corner/face controls), so boundary support does
  not authorize an SPD-only solve or removal of trust-region safeguards.
- **Current decision:** select `SPLIT_STATIC_BOUNDARY_FORMULA_CANDIDATE`.
  Support/reaction and nonpenetration/contact remain different operations.
- **Current conclusion:** primary algorithms do not supply an inherited split
  order. B3 selects `smooth solve -> swept contact -> velocity reconstruction
  -> rebuild` as a falsifiable engine-side hypothesis; SAM's CCD-initialized
  contact energy remains a different solver identity.
- **Current conclusion:** the apparent wallward third-layer counterexample is
  a geometry-ownership bug: a wall-owned third layer is at `a+R-3dx`, so every
  nonpenetrating centre remains at `r>=H`. Fluid-relative ghost anchoring would
  contribute about `5.6e-5 rho0` at contact and is explicitly rejected.
- **Current conclusion:** B3 reproducibly fails its first face candidate at
  the per-substep momentum ledger before contact: absolute `5.67e-8 kg m/s`,
  normalized `1.48e-7` versus `1e-9` required.
- **Current conclusion:** the corrected C2 scale-aware/floor transcription
  removes the preliminary reject-limit defect. The remaining failure is a
  mismatch between trajectory displacement stopping and reaction accuracy,
  not a pressure/contact formula mismatch.
- **Current conclusion:** the fixed ladder exposes two effects: active support
  can pass the `1e-8` displacement stop with `5.52e-3` relative ledger error,
  while inactive very-fine steps reach a relative binary64 reconstruction
  floor despite piconewton-second absolute defects.
- **Current decision:** preserve B3 FAIL and do not loosen the ledger or
  globally tighten the solver without a cost/accuracy discriminator.
- **Current conclusion:** B3D separates `D_stationarity`, `D_translation`,
  `D_reconstruct` and `D_contact`. Only inactive reconstruction may use a
  computed binary64 forward-error bound; active support requires a stricter
  reaction-aware stop.
- **Current conclusion:** B3D validates the active reaction-aware stop. The
  corner defect falls from `1.328e-5` to `7.12e-13 kg m/s` for two HVPs with
  negligible state change.
- **Current conclusion:** inactive actual defects are only `0.8%--1.4%` of
  the computed forward bound, but cumulative `|x|/h` bounds exceed the
  `1e-10*M*c` budget at 192/384 substeps. The certificate is correctly
  rejected; B3 retry remains blocked.
- **Current conclusion:** D1 displacement ownership removes the diagnosed
  `|x|/h` cancellation on every completed prefix; its cumulative bound is
  only `2.90e-12--1.36e-11`, versus up to `4.03e-7` previously.
- **Current conclusion:** D1 nevertheless fails all six full traces at
  `REACTION_BELOW_ENERGY_RESOLUTION`. Required predicted decreases are
  `4.55e-24--1.25e-20`, while the inherited absolute floor is `2.27e-13` and
  reaction residual remains `1.10x--23.6x` above its mixed limit.
- **Current decision:** preserve D1 FAIL and retain displacement ownership as
  a representation candidate only. Do not accept an unresolved energy step.
- **Current conclusion:** D2's factored endpoint arithmetic matches extended
  precision, but three of six endpoint changes are certifiably negative;
  density/compression quantization dominates the `1e-24--1e-20` model signal.
- **Current conclusion:** all six unchanged-topology trial steps reduce the
  impulse stationarity residual to `0.0030--0.6279` of its prior value and
  finish below the unchanged reaction limit.
- **Current decision:** select `FLOOR_STATIONARITY_MERIT_CANDIDATE` for one
  separately frozen full-trajectory experiment. Do not relabel D1 or energy
  ascent as descent.
- **Current conclusion:** D3 accepts `1--121` valid floor-merit trials before
  every fixed row reaches a later failure. Four trials reduce residual by
  about `1e4` but need another iteration; both `/384` trials increase it.
- **Current decision:** preserve D3 FAIL. Do not generalize the D2 one-step
  result into an unbounded residual solver or hide the fine-level overshoot.
- **Current action:** implement frozen D4 legacy/owned gradient identity and
  one-step counterfactual at the six D3 failure states.
- **Next gate:** compare legacy/owned gradient residuals and one owned-gradient
  trust trial before selecting iteration, line search or local geometry work.
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

### D-011 -- Validate the pressure spectral-policy premise

- **Observation:** pressure `lambda_max` has dense/matrix-free correspondence,
  exact `kappa` scaling, null translation modes and a reproducible 10.6%
  amplitude increase in nondimensional amplification.
- **Decision:** authorize B1S2 design around `dt*omega_max`, not around linear
  `dt*c/dx`; retain the 48-HVP estimate as explicit policy overhead.
- **Consequence:** no policy is selected yet, and a cheaper estimator cannot
  inherit correctness without a separate correspondence gate.

### D-012 -- Reject spectrum as a standalone error estimator

- **Observation:** `dt*omega<=0.15` reproduces every spectrum/count and all
  convergence gates, but finite-amplitude velocity error remains slightly over
  limit for the 2% row.
- **Decision:** retain spectrum only to seed an embedded step-doubling
  controller; do not fit another global target to the observed matrix.
- **Consequence:** B1S3 must publish discarded comparator work and pass a new
  amplitude holdout before any multi-step selection.

### D-013 -- Select measured refinement, retain its cost as a blocker

- **Observation:** B1S3 passes all six parent cases and the unseen 3%
  compression holdout. It automatically refines the amplitude-dependent row,
  preserves every parent phase hash and is byte-repeatable.
- **Decision:** select `EMBEDDED_SPECTRAL_ERROR_CONTROLLER_R0` for report-only
  research and use it to design B1R; do not call it a production controller.
- **Consequence:** accepted state ownership is now defined, but repeated
  macro-frame composition and the `3x--3.5x` speculative-work multiplier must
  be tested before B2. Later performance research must reduce or amortize the
  comparator, not omit it.

### D-014 -- Keep viscosity inside the inactive-state decision

- **Observation:** a pressure-inactive expanding lattice can still have
  nonzero bulk-viscosity energy; pressure count alone cannot authorize exact
  free flight.
- **Decision:** B1R's zero-comparator fast path also requires zero relative
  velocity on every supported bond and an inactive free-flight endpoint.
- **Consequence:** single-particle flight and rigid translation stay cheap;
  relaxing material states enter the embedded `n=1` path even after pressure
  switches off.

### D-015 -- Reject coarse-state ownership under composition

- **Observation:** 36 local frame transactions pass and fixed references
  converge at order about 1.95, yet all three composed coarse-state paths miss
  the global position gate. Degenerate controls and solver work remain valid.
- **Decision:** preserve B1R FAIL and test ownership of the already computed
  fine member under B1R1; do not loosen the global error budget.
- **Consequence:** B2 stays blocked. If fine ownership passes, comparator work
  becomes accepted work and speculative overhead falls without extra solves;
  if it fails, the next design must allocate horizon-aware error or use a
  higher-order accepted state.

### D-016 -- Commit the fine member of a passing error pair

- **Observation:** B1R1 preserves every reference and first-frame observable,
  executes no additional work, and passes all global accuracy gates. The 3%
  case improves from `0.13446dx` to `0.04684dx`.
- **Decision:** select fine-state ownership and `NSR_MULTISTEP_CANDIDATE` for
  report-only CPU research.
- **Consequence:** B2 static-boundary formula design is authorized. The coarse
  probe remains charged as discarded work; no boundary, CUDA, runtime or
  production execution is authorized.

### D-017 -- Split boundary support, contact and reaction semantics

- **Observation:** selected Nonlocal provides no validated collision formula;
  the semi-analytical SISPH boundary paper changes solver/globalization and is
  one-way without solid feedback forces.
- **Decision:** use fixed samples only as fluid-centered pressure support;
  derive their virtual reaction, and retain exact swept contact as a separate
  operation with its own reaction.
- **Consequence:** B2 pressure support receives gradient/HVP oracles; hard
  contact deliberately does not. A unified boundary energy is deferred to a
  new identity if split composition later fails.

### D-018 -- Prove two layers from kernel closure, do not import three

- **Observation:** FCR2 has `H=3dx`, but `W(H)=W'(H)=W''(H)=0`; the third axis
  shell should carry zero density and curvature while exceeding the current
  product capacity if retained mechanically.
- **Decision:** make two-versus-three-layer face/edge/corner correspondence a
  B2 gate before changing any profile or capacity.
- **Consequence:** PASS preserves the bounded two-layer candidate; mismatch
  stops for profile/capacity reclosure rather than silently raising limits.

### D-019 -- Select split static-boundary formula, retain trust safeguards

- **Observation:** B2's two/three-layer values agree exactly; independent
  gradient/HVP/dense checks and support/contact reaction closure pass. The
  pressure Hessian nevertheless contains reproducible negative eigenvalues.
- **Decision:** select `SPLIT_STATIC_BOUNDARY_FORMULA_CANDIDATE` and proceed
  only to a tiny B3 composition smoke with the existing trust-region solver.
- **Consequence:** fixed support is not contact, hard contact has no pressure
  HVP, and neither hydrostatic execution nor an SPD-only shortcut is
  authorized.

### D-020 -- Test post-solve contact as an explicit operator split

- **Observation:** NUV has no collision stage; SAM uses CCD to initialize a
  separate bulk-plus-contact SISPH solve. Pre-sweeping FCR2's inertia target
  would hide contact impulse and still not prevent a later crossing.
- **Decision:** B3 solves the unchanged smooth objective first, then sweeps
  the accepted segment, reconstructs velocity and invalidates every smooth
  cache before the next substep.
- **Consequence:** impact-order reduction and post-contact nonstationarity are
  measured rather than denied. Failure may justify one newly identified
  ordering remediation or a separately rooted unified-contact objective.

### D-021 -- Separate trajectory convergence from reaction authority

- **Observation:** B3's face control fails the support momentum ledger before
  any contact. The selected scale-aware stop can accept an active state whose
  virtual reaction does not close reconstructed momentum at `1e-9`; inactive
  tiny steps also make a purely relative ledger ill-conditioned.
- **Decision:** preserve B3 FAIL. Diagnose signed/absolute/cumulative defects
  and a charged reaction-aware stop before changing either solver or ledger.
- **Consequence:** B2 algebra remains selected, but no static-boundary
  trajectory, reaction authority or physical-corpus execution is authorized.

### D-022 -- Certify inactive arithmetic; solve active reaction accuracy

- **Observation:** a mixed tolerance without causal decomposition would hide
  active stationarity error behind inactive subtraction roundoff.
- **Decision:** compute a gamma-bound only for inactive reconstruction and
  require active steps to continue until an impulse-based stationarity stop;
  charge up to `2.5x + 2` HVPs before rejecting cost.
- **Consequence:** B3 retry remains blocked until both parts pass. An energy-
  floor exit before reaction closure explicitly rejects reaction authority.

### D-023 -- Preserve the substep displacement instead of recovering it

- **Observation:** active reaction-aware replays pass cheaply, but the
  inactive world-position forward bound grows as `|x|/h` and exceeds the
  cumulative budget under refinement despite covering actual error tightly.
- **Decision:** test one transient `delta` that accumulates prediction, trust
  and contact increments; derive velocity from `delta/h` and position from
  `x+delta`.
- **Consequence:** no public state changes. B3 remains failed until D1 proves
  the representation and a separately frozen B3R passes composition.

### D-024 -- Preserve D1 failure and remove total-energy cancellation

- **Observation:** displacement ownership lowers the completed-prefix
  reconstruction bound by roughly four orders, and both inherited active
  replays still pass. Full traces instead stop when the predicted objective
  decrement is `1e7--1e10` below the inherited `max(|E|,1)` energy floor while
  reaction closure is not yet satisfied.
- **Decision:** preserve D1 FAIL. Research a direct difference form for every
  unchanged objective term and require a derived roundoff interval to prove
  the sign used by the trust ratio.
- **Consequence:** no threshold relaxation or B3 retry is authorized. A direct
  difference implementation must replay D1 exactly until the first floor and
  must retain displacement ownership and all reaction/contact gates.

### D-025 -- Switch merit, not physics, at unresolved energy scale

- **Observation:** direct endpoint energy is certifiably negative in three of
  six D2 states because coordinate/density quantization exceeds the model
  decrement. The same trial retains topology and reaches the existing impulse
  stationarity limit in all six states.
- **Decision:** reject factored energy as a universal floor repair. Test one
  bounded acceptance of decreasing `0.5*||h*grad F||^2` only when the old
  energy-floor predicate fires and the trial already satisfies reaction
  closure.
- **Consequence:** this changes numerical globalization, not the objective or
  physical model. D3 must fail closed on any nonconverged residual trial and
  remains report-only until a later B3 retry independently passes.

### D-026 -- Reject one-shot residual closure; reclose the gradient identity

- **Observation:** D3's first-floor action remains valid many times, but later
  coarse states need more than one correction and fine states overshoot. The
  candidate measures reaction from owned displacement while its inherited
  inertia gradient still subtracts materialized positions.
- **Decision:** preserve D3 FAIL and compare both gradient identities at the
  exact failed states before authorizing residual iteration or line search.
- **Consequence:** the next result must distinguish a transcription/state-
  ownership inconsistency from an intrinsic residual-globalization problem.
  No B3 retry is authorized.

## Required context

1. `docs/architecture/agent-routing.md`, SPEC-38, ADR-076 and ADR-081.
2. `docs/plans/nonlocal-continuum-formula-reclosure/README.md` and
   `00-formula-contract.md`.
3. The stopped formula-reclosure task state and FCR3-B2 evidence.
4. `docs/development/nonlocal-nonlinear-solver-research-2026-08-20.md`.
5. `docs/plans/nonlocal-nonlinear-solver-research/README.md` and the current
   frozen stage contract.

## Exact next action

1. Derive legacy and displacement-owned inertia-gradient identities at the
   D3 first-failure captures.
2. Freeze a six-state gradient-consistency discriminator, including one
   owned-gradient trust step and residual outcome.
3. Select a bounded next candidate or stop; keep D3/D1/B3 exact.

## Reconsideration triggers

- Pairwise Descent paper/code becomes public: compare only after its exact
  formula and identity are reviewable.
- NSR0 HVP mismatch: fix one derivation/transcription defect under the same
  contract; a second mismatch stops the branch.
- NSR1 correct-model but penalty-dominated cost: draft an independent
  constrained primal-dual formula contract.
