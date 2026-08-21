# Nonlocal nonlinear solver research -- current task state

| Field | Value |
|---|---|
| Status | `ACTIVE / NSR3B4C3MAG_PASS / FULL_ADAPTIVE_MACRO_DESIGN` |
| Updated | `2026-08-21` |
| Task key | `nonlocal-nonlinear-solver-research` |
| Scope | Fundamental solver research over the verified Nonlocal variational objective, isolated from runtime and the stopped SISSM lineage |
| Definition of done | NSR0--NSR6 select a production-roadmap candidate or stop at an exact reproducible boundary |
| Authority | Working context only; Accepted architecture, SPEC-38/ADR-076 and the frozen stage contracts outrank this file |

## Resume in 60 seconds

- **Current conclusion:** B4C3TAR2 passes twice byte-identically at raw
  `911f4ee0...81c6`. It completes all 8/16 P1/P2 macro frames, recovers only
  the two exact reject-limit failures and preserves canonical, legacy-ledger
  and KKT-policy roots, schedules, physical envelopes and energy budgets.
- **Current decision:** select
  `CANONICAL_BALANCED_ADAPTIVE_RECOVERY_KKT_LEDGER_CANDIDATE`. B4C3TA and
  B4C3TAR remain preserved FAIL evidence; B4C3A2 remains the selected
  one-frame ledger transaction.
- **Current conclusion:** B4C3TR isolated lanes fail twice identically at raw
  `b04a7c93...51be`. Every solver/transaction/ledger lane finishes, but
  per-substep microunit continuation makes representation perturbations scale
  with substep count: P2 contact error grows `0.260 -> 0.521 -> 1.063 ms` and
  P1 fixed-192 changes the terminal contact set.
- **Current decision:** preserve B4C3TR FAIL and keep B4C3TC blocked. Do not
  widen tubes/event tolerances; the fixed-192 trajectory is not a valid
  refinement reference under current publication cadence.
- **Current conclusion:** B4C3P fails twice at raw `d7603a66...9d5f`, but
  confirms the cadence hypothesis: all four P1/P2 canonical ratios are
  `1.99--2.17`, contact timing/sets are exact and every macro transaction/
  ledger passes. Only P1/192 velocity exceeds `32*P*q` by `10.6%`.
- **Current decision:** preserve B4C3P FAIL. Do not increase coefficient 32;
  the reused envelope omits pressure/contact propagation of prior published
  position error.
- **Current conclusion:** B4C3PE passes twice at full raw `a2b55ae6...b986`.
  Decomposition is exact; scalar gain is ill-conditioned (`75--107x` in P1,
  one near-zero P2 ratio `7.74e9`). Fine contamination is resolved where
  meaningful, while absolute physical utilization stays below `0.00286`.
- **Current decision:** B4C3PE1 selects the macro-boundary canonical fixed
  reference candidate with 89 temporal, 55 absolute and zero rejected field/
  frame admissions. B4C3P remains the preserved legacy-tube FAIL.
- **Current action:** design the complete 8/16-frame adaptive macro recovery
  replay around the selected topology/transaction boundary. B4C3TC/B4C4/B4D
  remain blocked.
- **Current contract:** level-to-temporal-pair mapping is `{0,0,1}` for
  `48/96/192`; branch order is temporal `<=0.5D`, then absolute `<=1%` of
  `0.05dx/0.001c`. Both canonical fields must retain observed first order.
- **Performance finding:** six independent lanes use `311--312%` CPU and turn
  `~60.4` CPU-seconds into `~19.7` wall-seconds (`3.06x`) with byte-identical
  output. This validates harness parallelism, not solver/runtime performance.
- **Current evidence:** full raw-with-LF `911f4ee0...81c6`, no-LF
  `5862a1c9...6c3d`, semantic `ae52a97a...77f7`; wall time
  `107.72/107.50 s`. P1 accepts/attempts `364/563` substeps and has two strict
  diagnostic excursions while KKT residual stays `<=1e-9`.

- **Current conclusion:** B4C3Q exact aggregate-balanced apportionment passes
  twice at raw `ae44e39f...0731`. Biased aggregate error and 1,024-step center
  drift improve `49x`; P1/P2 physics, exact covariance and six atomic failure
  controls pass. Local error trades `<=0.5` nearest unit for `<1` balanced unit.
- **Current decision:** select `CANONICAL_AGGREGATE_BALANCED_CANDIDATE`. Bind a
  new profile and revalidate one-frame transaction/ledger as B4C3A1; do not
  reuse B4C3A nearest-even roots.
- **Current action:** freeze B4C3A1 with fine-only atomic commit plus explicit
  publication impulse, center and kinetic ledger terms. B4C3T remains blocked.
- **Current design:** B4C3A1 records raw published momentum closure separately
  from `L_raw-I_q`, which must reproduce the KKT ledger. It also decomposes the
  publication energy jump into kinetic, decoded-pressure and gravitational
  terms without fitting a pressure-energy cap before long-horizon evidence.

- **Current finding:** independent nearest-even continuation introduces a
  post-KKT aggregate position and velocity perturbation up to `N/2` canonical
  units per component and substep. The existing ledger is evaluated before
  that perturbation and therefore cannot claim conservation of published state.
- **Current decision:** insert B4C3Q before B4C3T. Compare nearest-even with
  exact deterministic aggregate-balanced apportionment; if selected, revalidate
  B4C3A1 under a new profile before any complete trajectory.
- **Current action:** implement the frozen B4C3Q algebraic, covariance,
  one-frame physical and 1,024-step temporal discriminator. B4C3T is blocked.

- **Current conclusion:** B4C3A passes P1 `21/42` and P2 `1/2` one-frame
  canonical transactions. Fine-only commit, decoded continuation, sample/step
  identity, repeat/reverse/affine roots, binary physical bounds and four atomic
  failure controls all pass twice byte-identically at raw `80920420...8ee3`.
- **Current decision:** select `JOINT_PRESSURE_CANONICAL_STAGE_CANDIDATE` and
  authorize only B4C3T full canonical-controller physical-bound design. The
  first false rejection at a half-microunit tie is preserved as a diagnostic
  measurement defect; quantizer, solver and frozen bound did not change.
- **Current action:** audit whole-controller canonical state ownership, derive
  independent multi-frame physical/drift gates and freeze B4C3T before any
  implementation. B4C4 packaging and B4D nominal execution remain blocked.

- **Current conclusion:** B4B2 passes P1 supported startup and P2 detached
  release/floor impact under the unchanged KKT, fixed `48/96/192` references,
  physical, ledger and work gates. Three raw reports are byte-identical at
  `43477c6d...a74f`.
- **Current decision:** select `TINY_PRESSURE_CONTACT_FORECAST_CANDIDATE`.
  The feasible predictor is used only when an inactive committed start becomes
  pressure-active after clamped macro prediction; otherwise the exact inactive
  or current-active path is retained.
- **Current cost:** P1 executes `444` substeps and `3391` total spectral plus
  nonlinear HVPs; P2 executes `124` substeps and `314` HVPs. This proves tiny
  correctness, not production performance.
- **Current decision:** decompose B4C into C0 membership/order/capacity, C1
  pressure tape, C2 complete solver substitution and C3 canonical
  publish/decode continuation. Equal pair sets alone are insufficient because
  binary64 reduction order is part of the selected result.
- **Current action:** implement the frozen B4C0 exact all-pairs-versus-joint-
  cell discriminator. Do not run a nominal water corpus before B4C0--B4C3
  pass independently.
- **Current conclusion:** B4C0 proves exact membership, reduction order,
  pressure evaluation/HVP and permutation identity, but its two exact cell
  scans cost `36,240` distance tests versus P1's `27,240` all-pairs checks.
  P2 still improves to `15,726/33,183`.
- **Current decision:** preserve B4C0 FAIL. The blocker is the exact-count
  allocation policy, not the cell broad phase or pressure formula. Test a
  separately frozen one-pass builder over a pre-admitted bounded workspace;
  do not weaken the original executed-work predicate.
- **Current conclusion:** B4C0R passes all exact pair/math/permutation and
  typed-failure gates. One-pass distance work is `0.665x` all-pairs on P1 and
  `0.237x` on P2; maximum admitted pair/adjacency payload is `64/32 MB`.
- **Current decision:** select `JOINT_PRESSURE_NEIGHBORHOOD_CANDIDATE` and
  authorize only B4C1 compact CSR/pressure-tape design. Nested-vector row
  overhead remains diagnostic and carries no nominal memory credit.
- **Current conclusion:** B4C1 passes bit-exact CSR, radius, compression,
  full fluid/support HVP, permutation, inactive and failure-atomicity gates.
  On its two active controls, three HVPs reduce modeled norm/sqrt work from
  `24,846` to `3,626` and from `22,494` to `3,682`.
- **Current decision:** select `JOINT_PRESSURE_RADIUS_TAPE_CANDIDATE` and
  authorize only B4C2 one-substep current/trial/forecast query-substitution
  design. This is algorithmic-work evidence, not wall-clock or trajectory
  authority.
- **Current conclusion:** B4C2Q substitutes joint/taped forecast, current and
  projected-trial queries in four one-substep solves. Complete KKT states and
  all counters are bit-exact; the active forecast spectrum is exact over 48
  taped HVPs and forced rejection preserves current workspace identity.
- **Current decision:** select `JOINT_PRESSURE_KKT_QUERY_CANDIDATE` and
  authorize only B4C2T full B4B2 controller-substitution design. Candidate
  all-pairs query counters are zero; audit-oracle calls are separate.
- **Current conclusion:** B4C2T reproduces both complete adaptive and fixed
  B4B2 cases bit-for-bit with zero candidate/audit all-pairs calls. P1/P2 use
  `14,149/11,860` joint workspaces and `16,153/1,474` taped HVPs.
- **Current decision:** select `JOINT_PRESSURE_B4B2_CONTROLLER_CANDIDATE` and
  authorize only B4C3 canonical transaction design. Record accepted-workspace
  retention, static support indexing and nested-row elimination as mandatory
  B4C4 packaging before nominal B4D.

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
- **Historical transition:** B1R was executed and rejected; B1R1 fine-state
  ownership supersedes that next action for this research lineage.
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
- **Current conclusion:** D4 certifies `R=h*g_owned` at `5e-21--1.3e-18`
  error. At fine levels legacy identity error exceeds the residual itself.
- **Current conclusion:** one owned-gradient HVP removes both fine overshoots
  and strictly improves all six states; four coarse/mid states remain above
  the unchanged reaction limit after their first correction.
- **Current decision:** select `BOUNDED_OWNED_RESIDUAL_ITERATION_REQUIRED`.
  The full inertia gradient, reaction and velocity must share `delta`.
- **Current conclusion:** D5 completes all six fixed boundary traces; no solve
  needs more than two floor accepts, all active/inactive/momentum/contact and
  `1e-5 dx/c` correspondence gates pass.
- **Current cost:** candidate HVP work is `2.01x--2.87x` the ordinary trace;
  this is bounded correctness evidence, not a performance result.
- **Current decision:** select `OWNED_RESIDUAL_TRAJECTORY_CANDIDATE` and
  authorize a separately frozen B3R retry only.
- **Current conclusion:** B3R passes original adaptive composition, fixed
  convergence, strict ledger, terminal-contact and final accuracy gates for
  both static fixtures.
- **Current cost:** adaptive face/corner execute `597/90` nonlinear HVPs plus
  `96/0` spectral HVPs; speculative substeps remain `36/6`.
- **Current decision:** select `STATIC_BOUNDARY_SMOKE_CANDIDATE` and authorize
  B4 physical-corpus contract design only.
- **Current conclusion:** the paper's Poiseuille control uses fixed ghost
  particles in the viscous neighbor solve and a constant inlet. The current
  split wall is pressure support plus frictionless contact, so viscosity needs
  a separate wall/inlet formula contract before that analytical comparison.
- **Current conclusion:** Pairwise Descent paper/code remains `to appear` on
  the authors' official page; no algorithm is inferred from its title.
- **Current conclusion:** B4A passes exact closed-box two/three-layer
  correspondence, all six face/corner sweeps and free-surface separation.
  The 108 missing air cells contribute zero wall support; top density reaches
  `0.7337rho0` while the bottom reconstructs rest density to `3.33e-15`.
- **Current cost:** the existing boundary all-pairs path would perform
  `52.9M/116.3M/74.1M/2.338B` candidate checks per objective evaluation for
  hydro/dam/orifice-outer/sealed, before trust HVPs or embedded refinement.
- **Current decision:** select `CLOSED_BOX_FREE_SURFACE_ELIGIBLE`; pressure-only
  tiny controls may proceed, but nominal execution requires a joint
  fluid/support cell neighborhood.
- **Current conclusion:** B4B separates a `48`-particle supported-column
  startup from a `27`-particle released-block impact. It does not mislabel the
  short, non-viscous startup as hydrostatic equilibrium.
- **Current decision:** freeze exact `8/16` macro-frame horizons, physical and
  energy gates, fine-owned controller, fixed `48/96/192` reference ladder and
  aggregate differences before execution.
- **Current result:** B4B failed, the derived KKT and contact-onset controller
  repairs were isolated, and B4B2 now passes without changing coefficients or
  thresholds.
- **Next gate:** PASS may authorize B4C joint neighborhood/canonical design
  only; do not add viscosity, surface tension or internal aperture.
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

### D-027 -- Make displacement ownership cover the optimizer gradient

- **Observation:** owned gradient restores `R=h*grad F` within the computed
  binary64 bound and eliminates both fine-level overshoots without changing
  the HVP. Coarse states need additional improving corrections.
- **Decision:** use `delta-delta*` for inertia energy/gradient throughout the
  owned candidate and test a capped residual phase at the existing floor.
- **Consequence:** D5 may iterate only while topology is unchanged and
  residual strictly decreases. It cannot change pressure/contact physics or
  inherit B3 authority before a later retry passes.

### D-028 -- Select bounded owned-residual boundary trajectories

- **Observation:** D5 completes every fixed trace with at most two merit
  accepts per solve, zero rejected trials, certified inactive arithmetic and
  active reaction ratios below one. Final state changes stay below `1e-5`.
- **Decision:** select `OWNED_RESIDUAL_TRAJECTORY_CANDIDATE` and freeze B3R
  against the untouched B3 composition/ledger/reference contract.
- **Consequence:** numerical boundary trajectories are now credible enough for
  one adaptive composition retry. Physical corpus, CUDA and runtime remain
  blocked until B3R independently passes.

### D-029 -- Select repaired static-boundary composition

- **Observation:** B3R passes the untouched strict ledger and all adaptive /
  fixed-reference gates with the D5 state boundary. Reference ratios remain
  near two and adaptive final errors are below `0.006dx/0.00042c`.
- **Decision:** select `STATIC_BOUNDARY_SMOKE_CANDIDATE` and proceed only to
  B4 physical-corpus design.
- **Consequence:** split static support/contact ordering is validated for the
  bounded face/corner smoke fixtures. Hydrostatic equilibrium, free-surface
  release, larger topology and long-horizon drift remain untested.

### D-030 -- Decompose physical reclosure and select closed-box eligibility

- **Observation:** pressure trajectories, viscous wall conditions, surface
  calibration, internal aperture, canonical publication and scalable
  neighborhoods have different missing prerequisites. A nominal monolithic
  run could not attribute failure. The current brute-force boundary path also
  reaches billions of candidate checks per objective evaluation.
- **Decision:** split B4 into pressure-water, scalable/canonical, aperture,
  viscosity and surface lineages. Select the passing B4A box-owned support /
  free-surface topology and proceed to a tiny pressure-only B4B contract.
- **Consequence:** B4B may use the bounded brute-force oracle. Nominal runs
  require joint cell neighborhoods and restored exact references; Poiseuille
  requires a new fixed-wall/inlet viscous contract.

### D-031 -- Test support startup and release/impact as separate phases

- **Observation:** a uniform column under gravity is not the discrete
  hydrostatic equilibrium of the compressible unilateral penalty, and a short
  no-viscosity run cannot prove long-window settling.
- **Decision:** B4B names the first fixture supported-column startup and uses
  a separate initially pressure-inactive released block to expose the exact
  free-flight-to-contact transition. Freeze aggregate and fixed-refinement
  gates before either run.
- **Consequence:** B4B can falsify the tiny pressure trajectory and split
  composition, but even PASS grants no equilibrium or nominal-water credit.

### D-032 -- Reject post-solve wall splitting at the pressure active-set limit

- **Observation:** adaptive P1 completes within its local physical scales,
  but fixed 96/192 stop on their first substep. Their proposed residual steps
  reduce the reaction defect by `7.8e3--3.1e4`, yet cross the unilateral
  pressure active set while the predicted energy decrease is at or below the
  inherited binary64 floor.
- **Decision:** preserve B4B FAIL, the reference ladder and all thresholds.
  Treat this as a boundary-composition defect: ghost pressure support and the
  post-solve hard sweep cannot independently own stationarity in this limit.
- **Consequence:** derive and test a bound-constrained KKT/contact-multiplier
  formulation before any B4B retry. Do not relabel the unresolved floor,
  remove ghost support, execute P2, or begin neighborhood/nominal work.

### D-033 -- Select exact box KKT before a contact-potential lineage

- **Observation:** NUV's published objective is unconstrained and leaves
  unified fluid-solid collision as future work. SAM and IPC couple contact
  through new potentials, parameters and globalization, while the B4B static
  box admits parameter-free exact displacement bounds.
- **Decision:** first test a feasible bound-constrained form of the unchanged
  pressure objective. Derive contact impulse from the KKT multiplier and keep
  ghost-pressure and contact reactions separate in the complete ledger.
- **Consequence:** B4BK is a one-substep discriminator at exact P1 failures,
  not a trajectory retry. PASS only permits freezing B4B r1; FAIL opens a new
  SAM/IPC-style formula lineage.

### D-034 -- Preserve KKT r0 and repair only the filled-box face assertion

- **Observation:** all three constrained solves and detached P2 pass their
  numerical/physical gates. r0 fails solely because it forbids x/z
  multipliers, although the P1 fluid fills and touches both lateral face
  pairs. Their signed aggregate impulses cancel to roundoff.
- **Decision:** preserve B4BK r0 FAIL. Freeze r1 with exact face counts
  `[12,12,16,0,12,12]`, zero upper-y contact and the inherited absolute
  contact-impulse symmetry scale.
- **Consequence:** r1 changes no solver action, threshold or state. A second
  failure ends the hard-box lineage; PASS only permits B4B r1 contract design.

### D-035 -- Select contact KKT for one full pressure-corpus retry

- **Observation:** B4BK1 passes exact face counts at all three step sizes;
  opposite lateral impulses cancel at `5.4e-17--1.6e-16 N s`, all KKT/ledger
  gates remain closed, and detached P2 is bit-exact.
- **Decision:** select `BOX_CONTACT_KKT_CANDIDATE` and freeze B4B1 with the
  original P1/P2 fixtures, references and thresholds. Replace only post-solve
  sweep with the constrained solve in every substep.
- **Consequence:** B4B1 may test temporal/reference behavior. One-step PASS
  does not authorize general collision, nominal water or runtime work.

### D-036 -- Preserve KKT trajectory physics and reject its onset controller

- **Observation:** B4B1 adaptive/fixed paths close KKT, ledger, energy and
  reference-order gates. Frame zero starts pressure-inactive, accepts `1/2`
  at `8.58%` embedded kinetic error, but the committed two-step state is
  `24.46%` from fixed-192 and fails the frozen `15%` gate.
- **Decision:** preserve B4B1 FAIL and the kinetic threshold. Test a feasible
  clamped macro-predictor spectrum that detects contact-created pressure
  activity before selecting `n`.
- **Consequence:** B4BF is a one-frame controller discriminator with a full
  level curve and detached P2 negative. No full B4B2 retry is authorized yet.

### D-037 -- Select feasible-predictor spectrum for the full retry

- **Observation:** every adjacent small-count pair passes despite fixed-192
  kinetic failures through `n=4`; a generic consecutive-pair rule is not
  sufficient. The feasible predictor activates 16 pressure centres, derives
  `n=21`, and its fine-42 member is only `2.05%` from fixed-192. Detached P2
  remains exact at zero forecast HVPs.
- **Decision:** select `CONTACT_ONSET_SPECTRAL_FORECAST_CANDIDATE` and freeze
  B4B2. Use start spectrum when already active, forecast spectrum only for an
  inactive start that becomes active under feasible macro prediction.
- **Consequence:** the full corpus may be retried without changing KKT,
  thresholds or references. PASS is still only tiny pressure authority.

### D-038 -- Select the complete tiny pressure/contact trajectory

- **Observation:** B4B2 passes both frozen fixtures and every inherited
  comparison, convergence, physical, KKT, ledger and work gate. P1 frame zero
  uses `FORECAST_ACTIVE 21/42`; P2 stays `INACTIVE_EXACT` through frame 13,
  predicts the impact on frame 14 and becomes start-active on frame 15. Three
  reports are byte-identical and all ten historical raw reports remain exact.
- **Decision:** select `TINY_PRESSURE_CONTACT_FORECAST_CANDIDATE`. The
  feasible predictor is a charged initial-step estimator only; it cannot own
  state, impulse or accuracy acceptance.
- **Rejected alternatives:** do not restore post-solve wall splitting, accept
  the failed B4B1 `1/2` onset pair, hard-code a minimum count, use fixed-192 as
  an online oracle, or treat tiny elapsed time as a performance benchmark.
- **Consequence:** authorize only B4C joint fluid/support neighborhood and
  canonical-runner design. Nominal water, general collision, viscosity,
  surface tension, aperture, CUDA, runtime and production remain blocked.
- **Remaining uncertainty:** exact neighborhood correspondence under moving
  candidate positions, deterministic pair ordering, bounded capacity failure
  and canonical report identity are not yet demonstrated together.
- **Smallest next action:** derive the B4C identity and freeze a bounded
  all-pairs-versus-joint-cell discriminator before implementing or executing
  any nominal corpus.

### D-039 -- Reject two-pass exact-count neighborhood construction

- **Observation:** B4C0 is bit-exact in every mathematical and ordering gate,
  but P1's dense tiny box reduces candidates only from `27,240` to `18,120`
  per cell pass. Repeating the scan to count then fill raises actual work to
  `36,240`; P2's sparser box still improves to `15,726/33,183`.
- **Decision:** preserve B4C0 FAIL and its two-pass report. Investigate one
  pass into capacity admitted before execution, clearing private output on any
  overflow. Capacity admission, not exact-size allocation, is the normative
  requirement.
- **Rejected alternatives:** do not report only the cheaper of two executed
  passes, remove the small dense control, loosen the work gate, or select a
  finer cell solely from unmeasured distance-test counts.
- **Consequence:** B4C1 pressure tape remains blocked. A repair must retain all
  exact correspondence, storage-order and typed negative controls and publish
  its reserved memory bound.
- **Smallest next action:** freeze B4C0R with one-pass membership and an exact
  pre-admitted workspace byte ceiling, then rerun the unchanged controls.

### D-040 -- Select one-pass pre-admitted joint membership

- **Observation:** B4C0R preserves all four pair roots and every exact
  evaluation/HVP/permutation result while removing the duplicated cell scan.
  P1/P2 work becomes `18,120/27,240` and `7,863/33,183`; all capacity failures
  clear pair and adjacency output.
- **Decision:** select `JOINT_PRESSURE_NEIGHBORHOOD_CANDIDATE`. Use `u32`
  indices and reserve the frozen maximum pair payload before the query.
- **Consequence:** B4C1 may design compact CSR and a pressure coefficient tape
  over this exact pair order. Solver substitution, canonical continuation and
  nominal execution remain blocked.
- **Remaining uncertainty:** the current nested rows add ABI-dependent header
  overhead; pressure density/Jacobian/radial coefficients are still recomputed
  per HVP; no current/trial/forecast solver query uses this operator yet.
- **Smallest next action:** freeze a B4C1 compact-CSR/tape discriminator with
  exact untaped HVP/reaction correspondence and explicit byte formulas.

### D-041 -- Select compact CSR pressure-radius tape

- **Observation:** B4C1 reproduces every nested adjacency row, stored radius,
  outer-state compression and complete fluid/support HVP bit-for-bit for five
  controls, four directions and two input permutations. All three failure
  controls publish empty output. Three reports are byte-identical; B4C0R and
  B4C0 raw hashes remain exact.
- **Decision:** select `JOINT_PRESSURE_RADIUS_TAPE_CANDIDATE`. Reuse the
  canonical pair list plus `u32` CSR pair indices, binary64 radius and one
  binary64 compression per fluid centre.
- **Rejected alternatives:** do not copy the full multi-term A2 record, store
  binary32 coefficients, search the pair list from participant-only rows, or
  recompute density and gradient in every HVP.
- **Consequence:** B4C2 may design a one-substep substitution of current,
  trial and feasible-forecast pressure queries. Full trajectory, canonical
  publish/decode continuation and nominal execution remain blocked.
- **Remaining uncertainty:** a correct standalone HVP tape does not prove that
  trust trials rebuild membership at the right candidate state, that forecast
  queries remain read-only, or that every all-pairs solver call was removed.
- **Smallest next action:** audit the B4B2 call graph and freeze a paired
  all-pairs-versus-joint one-substep trace with exact query identities,
  trial-state rebuilds, result and work counters.

### D-042 -- Select one-substep joint pressure query lifecycle

- **Observation:** B4C2Q produces bit-exact legacy/candidate solves for P1
  initial, P1 forecast-active, P2 detached and P1 compressed states. Active
  cases execute `7/6/6` taped HVPs and three atomic promotions; P2 executes
  zero HVPs. Forecast and forced-reject ownership gates pass. Three reports
  are byte-identical and B4C1/B4C0R/B4C0 raw hashes remain exact.
- **Decision:** select `JOINT_PRESSURE_KKT_QUERY_CANDIDATE`. A private query
  owns position, joint neighborhood, Evaluation, tape and state digest;
  acceptance moves them together and rejection publishes none.
- **Rejected alternatives:** do not reuse current membership at projected
  trial positions, retain a stale tape after acceptance, let forecast own
  committed state, or combine the first substitution diagnosis with the full
  adaptive trajectory gate.
- **Consequence:** B4C2T may design full B4B2 adaptive and fixed-reference
  controller substitution. Canonical continuation and nominal execution stay
  blocked.
- **Remaining uncertainty:** per-substep exactness has not yet proven adaptive
  level selection, discarded-level isolation, multi-frame contact timing or
  fixed `48/96/192` trajectory identity under the joint operator.
- **Smallest next action:** freeze B4C2T with the complete B4B2 corpus and old
  report aggregates, plus joint query/workspace counters that do not alter the
  physical comparison gates.

### D-043 -- Select the complete binary64 joint-pressure controller

- **Observation:** B4C2T matches complete P1/P2 adaptive controllers, every
  discarded level, fixed `48/96/192` references and all report/physical/KKT
  state bit-for-bit. Candidate and audit all-pairs counters are zero. Two
  reports are byte-identical.
- **Decision:** select `JOINT_PRESSURE_B4B2_CONTROLLER_CANDIDATE` and proceed
  to a separately frozen canonical publish/decode transaction.
- **Rejected alternatives:** do not infer full continuation from B4C2Q, reuse
  a discarded adaptive level, keep diagnostic all-pairs calls, or claim the
  whole-harness wall time as solver performance.
- **Consequence:** B4C3 canonical transaction design is authorized. Nominal
  execution remains blocked through B4C3 and B4C4.
- **Remaining uncertainty:** publishing micrometre integers changes every next
  substep input and cannot retain the B4B2 binary64 trajectory root. The trace
  also exposes excessive accepted-state diagnostic rebuilds.
- **Smallest next action:** audit the existing canonical publisher/decode
  contract, freeze transactional substep ownership and derive new bounded
  physical comparisons without reusing invalid binary64 equality gates.

### D-044 -- Select level-local canonical staging

- **Observation:** B4C3A passes P1 `21/42` and P2 `1/2`. Every staged successor
  consumes the exact decoded prior frame; fine-only commit, step/sample
  identity, repeat/reverse/affine roots, binary physical bounds and all four
  failure-atomicity controls pass. Two reports are byte-identical and NPR1-A,
  B4C1, B4C2Q and B4C2T raw hashes remain exact.
- **Decision:** select `JOINT_PRESSURE_CANONICAL_STAGE_CANDIDATE` and permit
  B4C3T physical-bound design only.
- **Rejected alternatives:** do not publish coarse or provisional frames,
  continue from unquantized solver state, import the stopped NPR1 solver, or
  loosen `0.5e-6` because a binary64 decode/subtract diagnostic crossed the
  half-unit boundary.
- **Consequence:** a complete canonical adaptive controller may now be designed
  around private level batches and atomic selected-fine commit. B4C4/B4D and
  runtime/production remain blocked.
- **Remaining uncertainty:** one frame does not bound accumulation across all
  P1/P2 macro frames, contact-onset drift, adaptive schedule changes, discarded
  levels, trajectory root continuity or canonical failure after earlier
  committed macro frames.
- **Smallest next action:** freeze B4C3T with complete P1/P2 adaptive and fixed
  canonical trajectories, explicit checkpoint rollback and independent
  canonical-versus-binary physical envelopes.

### D-045 -- Expose the post-KKT quantization ledger gap

- **Observation:** B4C3A rounds every sample independently after the accepted
  KKT solve. The local half-unit bound permits an aggregate velocity error of
  `N/2` units per component per substep; its momentum impulse is absent from
  the already-closed solver ledger. Position rounding likewise perturbs center
  and pressure geometry before the next solve.
- **Decision:** block B4C3T and insert B4C3Q. Test exact aggregate-balanced
  apportionment that minimizes incremental squared error, has no hidden carry,
  keeps local error below one unit and aggregate error within half a unit.
- **Rejected alternatives:** do not ignore publication impulse, weaken the
  ledger, use an unordered/binary64 aggregate sum, or silently retain binary64
  continuation while claiming replay from canonical frames.
- **Consequence:** balanced PASS requires a new canonical profile and B4C3A1
  transaction revalidation. Balanced FAIL requires a new decision between
  snapshot-only publication and another frozen state representation.
- **Remaining uncertainty:** aggregate balancing may double the worst local
  error, perturb local pair geometry or introduce tie-break symmetry artifacts;
  its long-horizon physical benefit is not yet established.
- **Smallest next action:** execute the frozen algebraic, symmetry, physical and
  temporal discriminator before revisiting full-controller bounds.

### D-046 -- Select aggregate-balanced canonical publication

- **Observation:** exact apportionment reduces the 48-sample biased aggregate
  error from `23.52` to `0.48` units and the 1,024-step center drift from
  `5.0176e-4` to `1.024e-5`, both `49x`. P1/P2 remain within frozen binary and
  nearest physical bounds; contact, order, kinetic inequality and failures pass.
- **Decision:** select `CANONICAL_AGGREGATE_BALANCED_CANDIDATE`, accepting a
  local `<1`-unit bound in exchange for an aggregate `<=0.5`-unit bound.
- **Rejected alternatives:** do not retain independent nearest as authoritative
  continuation, hide a temporal carry, or interpret the residual-stress test as
  fluid accuracy. Snapshot-only publication remains a fallback if full
  canonical continuation later fails.
- **Consequence:** the canonical profile changes. B4C3A1 must revalidate atomic
  staging and add publication perturbations to the ledger before B4C3T.
- **Remaining uncertainty:** one-frame success does not prove long-horizon
  nonlinear/contact behavior; tie allocation may still affect local topology,
  and exact superaccumulator cost has no runtime acceptance.
- **Smallest next action:** freeze B4C3A1 selected-policy roots, ledger equations
  and atomic rollback; then implement it independently of full-controller gates.

### D-047 -- Freeze the quantization-aware publication ledger

- **Observation:** published momentum closure equals the solver ledger plus a
  deterministic numerical impulse `I_q`; raw published residual is therefore
  not expected to retain the physical `1e-9` ratio unless `I_q` is modeled.
  Position rounding also changes pressure and gravitational energy.
- **Decision:** B4C3A1 records `I_q`, raw and compensated momentum ledgers,
  center shift, and kinetic/pressure/gravity/mechanical publication deltas per
  staged frame. Only fine entries commit with fine frames.
- **Rejected alternatives:** do not label quantization impulse as a boundary
  reaction, silently subtract an unreported correction, or fit a universal
  pressure-energy threshold from the first observed result.
- **Consequence:** one-frame ledger mechanics can be tested independently;
  full-horizon pressure-energy accumulation remains a B4C3T design input.
- **Remaining uncertainty:** decoded pressure-energy perturbations and local
  topology changes may accumulate nonlinearly even when momentum aggregation is
  bounded.
- **Smallest next action:** implement the frozen B4C3A1 transaction and ledger,
  preserve all parent hashes and execute two byte-identical reports.

### D-048 -- Select balanced stage ledger semantics

- **Observation:** B4C3A1 commits P1/P2 fine frames and ledger entries exactly,
  preserves order/repeat/rollback and keeps physical correspondence. Raw
  published residual is up to `9.98e-6`; subtracting the recorded numerical
  impulse restores at most `4.67e-10`. P1 pressure publication deltas dominate
  its one-frame representation-energy record at `6.55e-5 J` absolute sum.
- **Decision:** select `CANONICAL_BALANCED_STAGE_LEDGER_CANDIDATE`. Raw residual
  remains evidence; compensated residual is the KKT gate. Pressure-energy is a
  recorded numerical term pending a long-horizon discriminator.
- **Rejected alternatives:** do not apply `1e-9` directly to raw decoded state,
  hide `I_q`, classify it as physical reaction, or derive a global energy cap
  from one frame.
- **Consequence:** B4C3T full canonical-controller design is now authorized.
  Nominal/runtime/production remain blocked.
- **Remaining uncertainty:** aggregate pressure/mechanical deltas across 8/16
  macro frames, adaptive schedule changes, contact onset and fixed-reference
  convergence are unknown.
- **Smallest next action:** audit full lane ownership and freeze B4C3T physical,
  root, rollback and cumulative publication-ledger bounds before implementation.

### D-049 -- Split and freeze the full canonical controller

- **Observation:** adaptive transaction identity, fixed canonical convergence
  and adaptive-versus-fixed comparison have independent failure modes and would
  make one long B4C3T report diagnostically ambiguous.
- **Decision:** execute B4C3TA adaptive lanes first, then B4C3TR fixed
  `48/96/192`, then B4C3TC comparison. Freeze B4C3TA with global steps, exact P2
  onset schedule, post-commit rollback and binary/energy envelopes derived
  before its run.
- **Rejected alternatives:** do not combine all long lanes, loosen schedule
  activation after observing canonical noise, or derive trajectory tolerances
  from the B4C3TA result itself.
- **Consequence:** B4C3TR remains blocked until adaptive canonical ownership and
  ledger pass independently.
- **Remaining uncertainty:** aggregate balancing may seed precontact local
  pressure or change accepted refinement; cumulative pressure-energy may exceed
  the independent `1%` budget.
- **Smallest next action:** implement global-offset canonical intervals and the
  complete transactional adaptive controller, then run B4C3TA twice.

### D-050 -- Preserve B4C3TA and isolate refinement recovery

- **Observation:** the first B4C3TA run aborts after four P1 frames when the
  16-substep level reaches `KKT_SOLVE:REJECT_LIMIT`. Exact-state replay passes
  at 32, 64 and 128 substeps; both adjacent passing gates pass. P2 completes
  with schedule, contacts, binary envelope and energy budgets intact, but its
  case gate accidentally demands exact-zero precontact velocity error instead
  of the frozen local canonical bound.
- **Decision:** preserve B4C3TA FAIL and authorize a separate B4C3TAR repair.
  Only `KKT_SOLVE:REJECT_LIMIT` is refinable. Failed levels remain private and
  counted, while selection still needs an adjacent passing pair. Restore the
  `<1e-6` local position/velocity comparisons already frozen by B4C3TA.
- **Rejected alternatives:** do not loosen KKT, embedded, contact, activation,
  ledger or energy gates; do not commit a failed/isolated passing level; do not
  reinterpret B4C3TA as PASS.
- **Consequence:** the evidence distinguishes a controller-policy defect from a
  formula failure. B4C3TR and all later authority remain blocked until B4C3TAR
  passes twice and preserves parent reports.
- **Remaining uncertainty:** later P1 frames may require repeated recovery or
  exhaust four levels, and accumulated canonical energy may still exceed its
  frozen budget.
- **Smallest next action:** implement the frozen B4C3TAR exact failure
  classification, attempted-work accounting and local precontact bounds, then
  execute two byte-identical complete reports.

### D-051 -- Preserve B4C3TAR and reclose ledger normalization

- **Observation:** B4C3TAR recovers P1 frames four and five, accounts 563
  attempted substeps exactly and passes P2. At P1 frame seven the compensated
  publication ledger has zero closure to the KKT ledger, yet 32/128 levels fail
  at `1.1268e-9/1.0709e-9` while their KKT residuals remain below `1e-9`.
  Levels 16/64 pass, so refinement is non-monotonic.
- **Decision:** preserve B4C3TAR FAIL. Treat this as a residual-normalization
  contract conflict, not a recoverable stage failure. Keep the max-scaled
  publication residual for diagnostics, but test a separately frozen
  KKT-scale compensated residual for physical admission.
- **Rejected alternatives:** do not raise the threshold, silently accept stage
  failures, classify all ledger failures as refinement signals, or erase the
  stricter residual.
- **Consequence:** B4C3A1 ledger admission must be revalidated under a new
  identity before complete-controller work resumes. Existing frame/root and
  quantization identities remain evidence, not automatically selected policy.
- **Remaining uncertainty:** scale alignment may remove the false rejection but
  could reveal a true ledger or energy failure later in the complete P1 lane.
- **Smallest next action:** freeze a ledger-normalization discriminator with a
  synthetic near-factor-two case, exact vector/closure identity, one-frame
  P1/P2 replay and the real 16/32/64/128 frame-seven checkpoint.

### D-052 -- Freeze compensated-ledger normalization discriminator

- **Observation:** for `d=|DeltaP|` and external impulse magnitude sum `e`, the
  existing KKT and publication scales are `d+e` and `max(d,e)`. Their residual
  ratio is bounded by `[1,2]`; therefore the latter is a conditioning diagnostic
  but not semantically equivalent physical admission.
- **Decision:** B4C3L records both scales/residuals, preserves raw/max evidence,
  and gates only compensated KKT-scale residual at the unchanged `1e-9`.
  Synthetic separation and real B4C3A1/frame-seven states are frozen before
  implementation.
- **Rejected alternatives:** do not copy the source PASS boolean without
  recomputation, erase the stricter diagnostic, tune a threshold from the
  observed residual or alter canonical frames/physical coefficients.
- **Consequence:** a PASS can authorize only B4C3A2 one-frame ledger replay.
  Complete adaptive recovery remains blocked.
- **Remaining uncertainty:** recomputed and source KKT residuals may differ by
  floating evaluation order; that difference must fit a derived forward bound.
- **Smallest next action:** implement B4C3L as a read-only discriminator and run
  twice before defining B4C3A2 identity or changing selected ledger admission.

### D-053 -- Select KKT-scale compensated-ledger admission

- **Observation:** B4C3L passes synthetic factor-two/separation, all one-frame
  B4C3A1 entries and the exact frame-seven four-level corpus twice
  byte-identically. All candidate residuals match their source KKT residual
  within the derived bound; the maximum bound utilization is `0.7538`.
- **Decision:** select `CANONICAL_KKT_SCALE_LEDGER_CANDIDATE`. Keep raw and
  strict-max residuals as diagnostics and gate compensated physical ledger at
  unchanged `1e-9` using the KKT sum scale.
- **Rejected alternatives:** do not raise the threshold, erase strict evidence,
  directly promote B4C3TAR, or mutate old B4C3A1 report identities.
- **Consequence:** only B4C3A2 one-frame selected-policy ledger design is now
  authorized. Complete adaptive replay remains blocked.
- **Remaining uncertainty:** new ledger admission must preserve fine-only
  atomic commit, rollback, energy decomposition and exact canonical roots under
  a new evidence identity before it can become controller input.
- **Smallest next action:** freeze B4C3A2 policy fields, selected roots,
  candidate/diagnostic residual serialization and one-frame negative controls.

### D-054 -- Freeze KKT-scale stage-ledger revalidation

- **Observation:** B4C3L changed evidence/admission semantics, not canonical
  samples. Reusing the representation profile is required, while reusing only
  the legacy ledger hash would omit the new policy fields.
- **Decision:** B4C3A2 preserves exact B4C3A1 canonical trajectory and legacy
  ledger roots, and adds a policy ledger root over both scales/residuals,
  correspondence, closure, energy and B4C3L policy identity.
- **Rejected alternatives:** do not change the canonical profile, mutate old
  B4C3A1 report/root definitions, omit strict diagnostics or directly resume
  the complete controller.
- **Consequence:** a PASS proves one-frame transactional ownership under the
  selected policy and can authorize only a new complete replay design.
- **Remaining uncertainty:** policy fields/order and atomic rollback must remain
  deterministic across repeat/permutation runs.
- **Smallest next action:** implement the frozen B4C3A2 P1/P2 transactions,
  policy hash, exact-root checks and negatives, then execute twice.

### D-055 -- Select KKT-scale stage-ledger transaction

- **Observation:** B4C3A2 preserves P1/P2 trajectory and legacy ledger roots,
  produces distinct deterministic policy roots, and passes all atomic,
  correspondence, order/repeat, energy and negative gates twice identically.
- **Decision:** select `CANONICAL_KKT_SCALE_STAGE_LEDGER_CANDIDATE` and
  authorize design only of a new complete adaptive recovery replay.
- **Rejected alternatives:** do not reinterpret B4C3TAR as PASS, mutate old
  evidence roots, omit strict diagnostics or proceed directly to fixed/nominal.
- **Consequence:** complete P1/P2 replay may now combine two independently
  proven repairs: exact reject-limit refinement and KKT-scale ledger admission.
- **Remaining uncertainty:** later P1 frames, accumulated energy budgets,
  schedules and full policy-ledger root continuity have not run together.
- **Smallest next action:** freeze the combined replay's parent hashes, policy
  identity, attempted-work/rollback semantics and unchanged long-horizon gates.

### D-056 -- Freeze combined complete adaptive replay

- **Observation:** reject-limit recovery and KKT-scale ledger admission now
  pass independently, with distinct historical and policy identities.
- **Decision:** B4C3TAR2 composes only those two policies over the unchanged
  B4C3TA long-horizon corpus/gates and commits canonical, legacy-ledger and
  policy-ledger roots atomically.
- **Rejected alternatives:** do not fold another repair into the replay,
  refit energy/binary gates, alter recovery classification or reinterpret r0/r1
  FAIL reports.
- **Consequence:** a PASS can reopen B4C3TR fixed-reference design; nothing
  later is currently authorized.
- **Remaining uncertainty:** the combined P1 lane may expose a later physical,
  energy, schedule or solver failure not present in either local discriminator.
- **Smallest next action:** implement the frozen r2 controller and execute its
  isolated lanes before the two complete parent-gated reports.

### D-057 -- Select combined adaptive recovery candidate

- **Observation:** B4C3TAR2 completes both long-horizon scenarios twice
  byte-identically while preserving all inherited physical, energy, schedule,
  root, rollback and negative gates. P1 recovers exactly two classified
  reject-limit failures; its two strict-normalizer excursions remain finite
  diagnostics and pass the KKT sum-scale gate.
- **Decision:** select
  `CANONICAL_BALANCED_ADAPTIVE_RECOVERY_KKT_LEDGER_CANDIDATE` and authorize
  B4C3TR fixed canonical reference design only.
- **Rejected alternatives:** do not erase the B4C3TA/B4C3TAR failures, treat
  strict residual excursions as unreported, change solver tolerances, compare
  adaptive versus fixed before validating the fixed canonical references, or
  begin nominal/runtime/GPU work.
- **Consequence:** complete fixed `48/96/192` canonical lanes may now be
  specified. B4C3TC and all downstream authority remain blocked.
- **Remaining uncertainty:** fixed lanes have not yet proven temporal
  convergence under balanced publication and KKT-policy ledgers, and their
  cost may materially exceed the already long adaptive replay.
- **Smallest next action:** derive and freeze B4C3TR identity, fixed-lane root
  ownership, convergence/floor gates, exact work accounting and failure
  atomicity before implementation.

### D-058 -- Freeze fixed canonical reference validation

- **Observation:** fixed binary64 convergence and deterministic canonical
  publication are separate error sources. A raw canonical ratio alone can
  mislabel representation noise as lost temporal order, while a broad physical
  tolerance alone would not establish reference validity.
- **Decision:** B4C3TR requires both the inherited binary64 convergence result
  and per-level canonical-to-binary forward tubes. Canonical successive levels
  must show first-order ratio or an explicitly named, independently computed
  representation-floor overlap.
- **Rejected alternatives:** do not compare adaptive and fixed in the same
  stage, reuse adaptive roots, hide floor overlap as observed order, derive a
  tolerance from future fixed output, or change the KKT/quantization policy.
- **Consequence:** six `(case,level)` lanes may execute independently and in
  parallel, but each owns complete state, work and three evidence roots.
- **Remaining uncertainty:** canonical fixed lanes may expose nonlinear failure,
  exceed the forward tube/energy budget or be dominated by representation
  floor; their wall-time scaling is not yet measured.
- **Smallest next action:** implement isolated lane execution, convergence and
  rollback controls, then run the isolated report before parent-gated replay.

### D-059 -- Preserve physical versus published feasibility semantics

- **Observation:** the first B4C3TR draft copied binary64 penetration and COM
  limits onto decoded microunit state, contradicting the already selected
  B4C3TAR2 representation allowance. It also omitted the explicit publication
  term from the canonical energy-creation allowance.
- **Decision:** before any B4C3TR execution, restore the inherited canonical
  bounds: decoded penetration `<=1e-6+64epsilon`, aggregate-balanced COM bound
  `steps*0.5e-6/N+1e-10`, and energy creation `<=1% scale + cumulative
  absolute publication mechanical delta`. The constrained KKT/contact and
  ledger tolerances do not change.
- **Rejected alternatives:** do not run a knowingly contradictory contract,
  loosen KKT feasibility, hide quantization in a generic epsilon, or refit a
  bound from future fixed-lane output.
- **Consequence:** B4C3TR now tests the same two-stage physical/published state
  semantics already proven by B4C3TAR2.
- **Remaining uncertainty:** the much longer fixed lanes may accumulate enough
  quantization energy or drift to exhaust these pre-existing allowances.
- **Smallest next action:** implement the corrected frozen contract before any
  fixed-lane measurement.

### D-060 -- Reject per-substep canonical fixed reference

- **Observation:** all six fixed lanes pass KKT, neighborhood, canonical
  transaction, ledger and root gates, while the binary64 ladders retain
  first-order convergence. Nevertheless canonical contact phase and same-level
  velocity error grow with `48 -> 96 -> 192`; P2 final canonical differences
  have position/velocity ratios `0.523/0.204` rather than positive order.
- **Decision:** preserve B4C3TR FAIL. Fixed microunit publish/decode after every
  physical substep is timestep-dependent model perturbation and cannot define
  the higher-resolution reference.
- **Rejected alternatives:** do not widen the event/tube gates, call the broad
  representation floor convergence, discard terminal contact identity, reduce
  fixed levels or proceed to adaptive-versus-fixed comparison.
- **Consequence:** B4C3TC is blocked. B4C3P must separate internal integration
  cadence from externally durable canonical publication cadence.
- **Remaining uncertainty:** macro-boundary-only publication may restore
  temporal convergence while retaining deterministic durable checkpoints, but
  it changes substep replay semantics and must be tested under a new identity.
- **Smallest next action:** derive and freeze a cadence discriminator with
  current per-substep control, macro-boundary candidate and explicit
  transaction/ledger semantics before implementation.

### D-061 -- Freeze macro-boundary publication discriminator

- **Observation:** fixed integration level must control only private physical
  accuracy; durable representation frequency must remain fixed per simulated
  second. The macro frame is the existing stable transaction boundary shared
  by every adaptive/fixed lane.
- **Decision:** B4C3P runs unchanged private binary64 KKT intervals and one
  balanced publish/decode per macro frame. A new profile binds cadence and a
  new macro-ledger identity binds aggregate interval closure without pretending
  there is one macro KKT solve.
- **Rejected alternatives:** do not scale the durable quantum with timestep,
  remove canonical continuation entirely, reuse per-substep profile/root,
  compare only final state, or alter solver/contact tolerances.
- **Consequence:** fixed `48/96/192` lanes now have identical durable
  publication counts, so temporal refinement no longer changes representation
  injection frequency.
- **Remaining uncertainty:** private solver differences may be amplified by
  macro quantization, and aggregate impulse cancellation may challenge the new
  macro ledger even if every substep ledger passes.
- **Smallest next action:** implement candidate macro transaction/ledger,
  same-level tubes, convergence and pre-publication rollback; execute twice.

### D-062 -- Preserve macro cadence with unclosed stability envelope

- **Observation:** B4C3P restores observed first-order position/velocity
  convergence and exact event/contact identity in both cases. Five of six
  same-level lanes pass. P1/192 alone reaches `2.83069e-4 m/s` against the
  frozen `2.56e-4 m/s` velocity formula while every physical, energy and
  ledger gate passes.
- **Decision:** preserve B4C3P FAIL but retain macro-boundary cadence as the
  supported hypothesis. The next blocker is the representation-error model,
  not cadence, KKT or contact.
- **Rejected alternatives:** do not change `32` to a post-hoc larger constant,
  waive the P1/192 lane, treat first-order convergence alone as sufficient, or
  reopen per-substep publication.
- **Consequence:** derive a separate stability budget that accounts for
  propagation of published position error through later macro dynamics and
  quantifies contamination of the independently convergent fine reference.
- **Remaining uncertainty:** a useful bound may require local macro-map gain or
  a relative temporal-error budget rather than a closed-form multiple of
  publication count.
- **Smallest next action:** build a diagnostic exposing per-frame direct
  publication perturbation, propagated same-level error and binary 96/192
  temporal difference before selecting a bound.

### D-063 -- Freeze threshold-free macro stability measurement

- **Observation:** the B4C3P output exposes only published-versus-binary error;
  it cannot distinguish direct quantization from amplification of an earlier
  durable-state perturbation.
- **Decision:** B4C3PE retains each private prepublication boundary and reports
  start, propagated, direct and published RMS errors with forward triangle
  closure. It separately reports resolved fine-192 contamination against
  binary 96/192 temporal error.
- **Rejected alternatives:** do not choose a coefficient from the 1.10574
  exceedance, infer a stability gain from one final frame, combine position and
  velocity units, or count binary64-floor divisions as meaningful ratios.
- **Consequence:** the next acceptance rule, if any, will be based on measured
  error transport and independent temporal scale rather than a post-hoc
  multiple of `q`.
- **Remaining uncertainty:** active contact may make resolved macro gains
  discontinuous; a contamination budget may be more stable than a Lipschitz
  gain bound.
- **Smallest next action:** implement exact per-frame decomposition and run it
  twice before designing B4C3PE1.

### D-064 -- Select mixed stability-budget direction

- **Observation:** B4C3PE proves every error triangle exactly. Scalar gains are
  not a stable admission variable, and pure temporal ratios exceed one when
  the binary temporal difference is materially smaller than a direct durable
  quantum. Nevertheless maximum canonical error uses only `0.286%` of the
  existing velocity and `0.069%` of the existing position accuracy scales.
- **Decision:** design B4C3PE1 with two explicit classifications: temporal
  budget `<=0.5` of a resolved adjacent binary difference, otherwise absolute
  representation budget `<=1%` of the already frozen physical comparison
  scale. Neither branch is called a solver convergence order.
- **Rejected alternatives:** no scalar macro gain, post-hoc `32 -> 36`
  coefficient, pure relative division, full physical-tolerance waiver or
  unreported fallback.
- **Consequence:** every accepted frame must name the branch and utilization;
  canonical fixed levels must still show observed first-order convergence and
  exact events/transactions.
- **Remaining uncertainty:** the mixed rule has not yet been applied to every
  level/frame or combined with an exact B4C3PE parent replay.
- **Smallest next action:** freeze the B4C3PE1 policy identity, adjacent-level
  mapping, branch priority, rollback and unchanged gates before implementation.

### D-065 -- Freeze mixed stability admission

- **Observation:** a durable representation error should be subordinate to
  resolved time-discretization uncertainty, but exact/free-flight frames need
  a small absolute allowance independent of a near-zero denominator.
- **Decision:** map levels to binary temporal pairs `{0,0,1}`, admit temporal
  share `<=0.5`, otherwise absolute share `<=0.01` of `0.05dx/0.001c`, and
  require an explicit branch for every position/velocity frame.
- **Rejected alternatives:** no maximum-of-scales without classification,
  level-dependent fitted constants, reuse of `32*P*q` as a hidden fallback,
  or representation-floor-only final convergence.
- **Consequence:** B4C3PE1 can select a fixed macro canonical reference only if
  both fields retain observed first order and every non-stability gate remains
  exact.
- **Remaining uncertainty:** all-level branch coverage and exact complete
  replay have not yet been executed.
- **Smallest next action:** implement the policy report and two exact
  parent-gated replays.

### D-066 -- Select the fixed macro-boundary reference candidate

- **Observation:** two complete B4C3PE1 replays are byte-identical; all 144
  field/frame admissions select an explicit branch, both fields retain
  observed first order and all non-tube physical/transaction gates pass.
- **Decision:** select `MACRO_BOUNDARY_CANONICAL_FIXED_REFERENCE_CANDIDATE`
  for subsequent adaptive research. Keep B4C3P's old tube FAIL visible.
- **Rejected alternatives:** widening `32*P*q`, treating an unresolved temporal
  denominator as zero tolerance, publishing trial substeps, or immediately
  resuming B4C3TC against the rejected per-substep reference.
- **Consequence:** adaptive trials must compare private binary64 macro endpoints
  and publish only the accepted endpoint. Canonical error is then admitted by
  the frozen mixed policy, not by a per-substep accumulation formula.
- **Remaining uncertainty:** the adaptive selector, retry semantics, work
  accounting and accepted-frame ledger have not yet been reclosed at this
  transaction boundary.
- **Smallest next action:** freeze the adaptive macro transaction state machine,
  comparison estimator and rejection/rollback controls before implementation.

### D-067 -- Split adaptive macro transaction from full replay

- **Observation:** B4C3TAR2's recovery/selection policy is reusable, but its
  per-substep canonical staging is exactly the cadence rejected by B4C3TR.
- **Decision:** B4C3MA runs private binary64 levels, selects an adjacent fine
  endpoint and performs one atomic macro publication. Test one P1/P2 frame
  before any long-horizon replay.
- **Rejected alternatives:** porting the old canonical stage unchanged,
  publishing every passing trial, counting only accepted work, or accepting a
  `REJECT_LIMIT` substring without exact substep identity.
- **Consequence:** canonical frame/root/ledger count advances once per accepted
  macro frame regardless of its private substep count. Discarded trials have
  no durable representation or energy effects.
- **Remaining uncertainty:** the one-frame implementation and negative controls
  have not yet executed.
- **Smallest next action:** implement B4C3MA and replay its B4C3PE1 parent twice.

### D-068 -- Reject raw floating equality as canonical topology identity

- **Observation:** B4C3MA passes every numerical/transaction gate but raw
  binary equality loses 24 of 64 P1 boundary features after decode. The maximum
  coordinate difference is `2.78e-17 m`, no feature is gained and penetration
  is zero.
- **Decision:** preserve B4C3MA FAIL. Define durable topology by exact equality
  of `canonical::quantize_position` integer coordinates for both state and
  boundary geometry; retain raw equality and penetration as diagnostics.
- **Rejected alternatives:** epsilon contact tests, snapping decoded state away
  from its committed frame, changing fixture geometry inside the failed stage,
  or ignoring topology because the long fixed lanes happened to recover it.
- **Consequence:** the repair changes identity and must rerun the complete
  B4C3MA transaction plus parent. It does not change solver coordinates,
  publication samples or mixed budgets.
- **Remaining uncertainty:** canonical-integer topology identity has not yet
  been implemented or checked against KKT terminal features.
- **Smallest next action:** freeze and execute the B4C3MAG topology reclosure.

### D-069 -- Freeze canonical integer topology

- **Observation:** durable particle coordinates and geometry already have an
  exact micrometre identity through `canonical::quantize_position`.
- **Decision:** define macro-boundary geometric features by equality of these
  integers and require exact equality with selected fine KKT terminal features.
- **Rejected alternatives:** any epsilon, ulp-count heuristic, post-decode snap,
  or changing private solver geometry.
- **Consequence:** B4C3MA remains the raw-equality FAIL control; B4C3MAG changes
  only topology admission and policy identity.
- **Remaining uncertainty:** integer topology negatives and complete parent
  replay have not executed.
- **Smallest next action:** implement the exact feature map and discriminator.

### D-070 -- Select canonical-topology adaptive macro transaction

- **Observation:** B4C3MAG passes twice with exact positive/negative parents.
  P1 canonical features equal all 64 KKT terminal features before and after
  decode; P2 remains empty/exact; topology negatives and rollback pass.
- **Decision:** select
  `CANONICAL_TOPOLOGY_ADAPTIVE_MACRO_TRANSACTION_CANDIDATE` and authorize only
  complete adaptive macro replay design.
- **Rejected alternatives:** reviving raw equality, adding a contact epsilon or
  treating the one-frame result as long-horizon evidence.
- **Consequence:** complete adaptive state advances by one canonical frame and
  one macro-ledger entry per accepted macro interval, irrespective of private
  substeps.
- **Remaining uncertainty:** recovery schedule, accumulated publication effects,
  contacts, physical gates, roots and work have not run over 8/16 frames.
- **Smallest next action:** freeze the full adaptive macro recovery contract.

## Required context

1. `docs/architecture/agent-routing.md`, SPEC-38, ADR-076 and ADR-081.
2. `docs/plans/nonlocal-continuum-formula-reclosure/README.md` and
   `00-formula-contract.md`.
3. The stopped formula-reclosure task state and FCR3-B2 evidence.
4. `docs/development/nonlocal-nonlinear-solver-research-2026-08-20.md`.
5. `docs/plans/nonlocal-nonlinear-solver-research/README.md` and the current
   frozen stage contract.

## Exact next action

1. Freeze full adaptive macro recovery state, schedule and ledger ownership.
2. Define fixed-reference comparison and mixed admission without using the
   rejected per-substep trajectory.
3. Implement only after report identity and negative controls are immutable.

## Reconsideration triggers

- Pairwise Descent paper/code becomes public: compare only after its exact
  formula and identity are reviewable.
- NSR0 HVP mismatch: fix one derivation/transcription defect under the same
  contract; a second mismatch stops the branch.
- NSR1 correct-model but penalty-dominated cost: draft an independent
  constrained primal-dual formula contract.
