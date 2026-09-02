# Nonlocal GPU full-step performance — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / LIVE WATER WATCHABLE / WALL STALL RESOLVED (NGQ7 REV 2, REV 3 SETTLED FLOOR PASS) / 48K COST AND MATERIAL NEXT` |
| Updated | `2026-09-02` |
| Task key | `nonlocal-gpu-full-step-performance` |
| Scope | Qualify the original compact/fused Nonlocal GPU path for game-quality water, selectively adding only observed necessary semantics |
| Definition of done | bounded dynamic visual/invariant acceptance plus two-process near-50k p95/p99 evidence, or the first honest bounded refutation |
| Authority | Working context only; SPEC-38, ADR-076/081, frozen NCGP1--NCGP4 contracts and exact evidence outrank this file |

## Resume in 60 seconds

- **Goal:** qualify a plausible game-water Nonlocal GPU step near 50,000
  particles against `p95 <= 4 ms`, `p99 <= 6 ms` on RTX 3080; laboratory
  fidelity to the later research solver is not required.
- **Current boundary:** the live Nonlocal GPU solver now drives the
  production SDL3/Ash renderer as a separate process:
  `nonlocal-feasibility --game-surface-stream` steps one persistent advected
  device state and streams the NGQ5 surface of every K-th step;
  `xtask water-preview --stream-binary` validates, converts and paces the
  frames into the D-039 dynamic surface ring. With ordered extraction
  workers both lanes are real time at a 60 Hz surface (4k `0.999` with two
  workers, 16k `0.997` with four; 16k at 30 Hz `0.996` with three). One
  catalog/snapshot/frame plan per run, `0` dropped samples.
- **First current risk:** 48k is solver-bound below real time on this host
  (`3.87 ms` GPU per `4.17 ms` step plus `~0.9 ms` research-wrapper
  overhead: `0.74x` unpaced, continuous at `--stream-rate 0.7`); no bridge
  change closes that gap. The stream lanes keep float state on the device
  and audit diagnostics every 60 frames, so they do not reproduce the
  accepted corpus roots and must not be cited as such.
- **Current action:** the user authorized the solver-side discriminator.
  NGQ7 revision 2 (two fixed lattice layers on the basin faces that support
  density only; viscosity and surface terms skip them) removes the wall
  stall on 4k and 16k: crest at the wall, `0` stall frames, front `1.2x`
  faster than the control, 16k physics cost unchanged. Revision 1 (fixed
  samples in every term) was refuted by a `0.89 m/s` no-slip front. The
  live bridge now defaults to density-only support (one layer on 48k, u16
  bound). Revision 3 split the compression observable: the floor layer
  peaks at `1.32 / 1.29` during impact and settles at `1.13 / 1.17` inside
  the `1.2` gate (control `2.25 / 2.27`). Revision 4 makes fixed owners
  skip the term kernel (bit-identical dumps) so 48k costs `4.98 ms` physics
  per step with one layer (`3.86` without support); 48k stays paced.
- **Performance baseline:** the exact historical fixed-work GPU source at
  `e2b533b49102bdff6684a7b68aa917ca635cc9e6` was rebuilt with CUDA `13.3.73`
  and rerun twice on the RTX 3080. Its old coherent/advected 50k corpus remains
  root-exact and inside `4/6 ms`; this is a reusable speed baseline, not
  corrected-water evidence.
- **Latest exact result:** two capacity-160 48k processes with analytic contact
  included in primary timing give p95 `3.225760 / 3.231616 ms` and p99
  `3.269312 / 3.334560 ms`; all traces/capacity checks pass. Combined with the
  exact 4k/16k visual PASS, this is bounded quality-and-budget evidence, not a
  shipping or corrected-research claim.
- **Product ceiling:** developer presentation tool under Proposed SPEC-38.
  CPU DFSPH remains fallback; no public gameplay/runtime authority, PhysX or
  renderer semantic contract changed.

## Required context

- `docs/plans/nonlocal-gpu-full-step-performance/00-ncgp4-solver-diagnosis-contract.md`
- `docs/plans/nonlocal-gpu-full-step-performance/01-unpreconditioned-selection.md`
- `docs/plans/nonlocal-gpu-full-step-performance/02-step92-outlier-diagnosis.md`
- `docs/plans/nonlocal-gpu-full-step-performance/03-product-trajectory-gate.md`
- `docs/plans/nonlocal-gpu-full-step-performance/04-eulerian-step112-diagnostic.md`
- `docs/plans/nonlocal-gpu-full-step-performance/05-visible-surface-observer.md`
- `docs/plans/nonlocal-gpu-full-step-performance/06-visible-surface-control-corrigendum.md`
- `docs/plans/nonlocal-gpu-full-step-performance/07-complete-4k-corpus.md`
- `docs/plans/nonlocal-gpu-full-step-performance/08-pressure-f64-complete-4k.md`
- `docs/plans/nonlocal-gpu-full-step-performance/09-invalid-physics-cost-only.md`
- `docs/plans/nonlocal-gpu-full-step-performance/10-pressure-state-equilibrium-discriminator.md`
- `docs/plans/nonlocal-gpu-full-step-performance/11-pressure-contact-tiny-trajectory.md`
- `docs/plans/nonlocal-gpu-full-step-performance/12-confined-pressure-contact-discriminator.md`
- `docs/plans/nonlocal-gpu-full-step-performance/13-unified-constrained-surface-viscosity-step.md`
- `docs/plans/nonlocal-gpu-full-step-performance/14-pressure-qp-preconditioned-unified-step.md`
- `docs/plans/nonlocal-gpu-full-step-performance/15-original-gpu-dynamic-visual-corpus.md`
- `docs/plans/nonlocal-gpu-full-step-performance/16-original-gpu-dynamic-visual-capacity-corrigendum.md`
- `docs/plans/nonlocal-gpu-full-step-performance/17-analytic-contact-cap160-performance.md`
- `docs/plans/nonlocal-gpu-full-step-performance/18-presentation-surface-prototype.md`
- `docs/plans/nonlocal-gpu-full-step-performance/19-presentation-surface-area-corrigendum.md`
- `docs/plans/nonlocal-gpu-full-step-performance/20-edge-aware-presentation-surface.md`
- `docs/development/nonlocal-gpu-complete-4k-evidence-2026-08-31.md`
- `docs/development/nonlocal-gpu-pressure-f64-corpus-evidence-2026-08-31.md`
- `docs/development/nonlocal-gpu-invalid-physics-cost-evidence-2026-08-31.md`
- `docs/development/nonlocal-gpu-pressure-state-discriminator-evidence-2026-08-31.md`
- `docs/development/nonlocal-gpu-pressure-contact-trajectory-evidence-2026-09-01.md`
- `docs/development/nonlocal-gpu-confined-pressure-contact-evidence-2026-09-01.md`
- `docs/development/nonlocal-gpu-unified-constrained-evidence-2026-09-01.md`
- `docs/development/nonlocal-gpu-unified-mutation-fixture-research-2026-09-01.md`
- `docs/development/nonlocal-gpu-unified-solver-diagnosis-2026-09-01.md`
- `docs/development/nonlocal-gpu-game-quality-evidence-2026-09-01.md`
- `docs/development/nonlocal-gpu-dynamic-visual-evidence-2026-09-01.md`
- `docs/development/nonlocal-gpu-cap160-performance-evidence-2026-09-01.md`
- `docs/development/nonlocal-gpu-presentation-surface-evidence-2026-09-01.md`
- `docs/development/nonlocal-gpu-engine-water-preview-evidence-2026-09-01.md`
- `docs/development/nonlocal-gpu-engine-dynamic-surface-evidence-2026-09-01.md`
- `docs/development/nonlocal-gpu-engine-live-stream-evidence-2026-09-01.md`
- `docs/development/nonlocal-gpu-engine-gpu-extraction-evidence-2026-09-02.md`
- `docs/development/nonlocal-gpu-engine-48k-live-lane-evidence-2026-09-02.md`
- `docs/development/nonlocal-gpu-engine-visual-surface-evidence-2026-09-02.md`
- `docs/plans/nonlocal-gpu-full-step-performance/21-wall-monolayer-boundary-support.md`
- `docs/development/nonlocal-gpu-wall-monolayer-evidence-2026-09-02.md`
- `docs/development/nonlocal-gpu-step92-diagnosis-evidence-2026-08-31.md`
- `docs/development/nonlocal-gpu-product-gate-evidence-2026-08-31.md`
- `docs/development/nonlocal-gpu-eulerian-step112-evidence-2026-08-31.md`
- `docs/development/nonlocal-gpu-visible-surface-evidence-2026-08-31.md`
- `docs/development/task-state/nonlocal-gpu-compensated-scale.md`
- `docs/development/nonlocal-gpu-compensated-scale-evidence-2026-08-30.md`
- `docs/plans/nonlocal-gpu-compensated-scale/00-compensated-scale-contract.md`
- `docs/plans/nonlocal-gpu-compensated-scale/01-graph-control-corrigendum.md`
- `docs/plans/nonlocal-gpu-compensated-scale/02-boundary-transaction-fixtures.md`
- `docs/plans/nonlocal-gpu-compensated-scale/03-corrected-fcr-profile-corrigendum.md`
- `docs/plans/nonlocal-gpu-compensated-scale/04-review-repair-contract.md`

## Material evidence and decisions

### D-001 — Diagnose before changing solver work

- **Observation:** the deterministic work ceiling gives only aggregate HVP and
  outer counts; it does not show whether inner conditioning, outer rejection,
  numerical cancellation or accounting caused the failure.
- **Evidence:** reviewer hydro240 stdout SHA-256
  `313d5eb78babfd34b407876248c90881dca0a5c75f63308e707e781e8184c834`;
  work root `0687e23dc524a83858a04d58b9cfd524a0e639c3ba282bf4cb7429a383c2d128`.
- **Decision:** freeze NCGP4 and instrument the exact pre-step-39 state before
  any preconditioner, precision, budget or tolerance change.
- **Rejected:** increasing 128 HVP, timing the failed route, treating dam step 6
  as first failure or guessing that scalar Jacobi is the cause.
- **Reconsider when:** a root-closed trace and independent same-state operator
  comparison select one frozen hypothesis.

### D-002 — Remove harmful scalar Jacobi from the scalable path

- **Observation:** on the exact compensated pre-step-39 state, scalar Jacobi
  fails after 126 HVP and 28 outer trials; the retained unpreconditioned path
  succeeds after 69 HVP and 19 outer trials.
- **Evidence:** diagnostic stdout SHA-256
  `78b40392e0981eec77e37c514ce39cae78172c642edfa9ead3de0d737e342f47`,
  result root `2f376da73d4edb8a09d5a223b43870415cfdd3e685601cb5e639b4fff52e4cce`;
  HVP relative L2 against long double `1.4262815435566996e-6`, cosine loss
  `9.8629643948550116e-13` and exact active signature.
- **Conclusion:** scalar diagonal construction is correctly counted but harms
  this coupled operator. The failure is not evidence for pressure-f64 or a
  changed trust-region tolerance.
- **Decision:** freeze revision 2 with the existing unpreconditioned
  Steihaug--Toint profile as the one selected solver repair. Keep physics,
  tolerances and the 128-HVP ceiling unchanged.
- **Rejected:** block-Jacobi, pressure-f64, higher HVP ceiling, radius or
  acceptance tuning.
- **Reconsider when:** only if the complete 4k corpus exposes a different
  first failure with root-closed evidence; no second solver repair is allowed
  by NCGP4.

### D-003 — Close NCGP4 at the first 240-step physical gate

- **Observation:** budgets 32 and 64 exhaust work after 0 and 11 complete
  hydrostatic steps. Budget 128 completes 92 steps, then maximum CPU/GPU
  position error reaches `5.211209258 mm` against the frozen `5 mm` limit.
- **Evidence:** exact final stdout SHA-256
  `a3aca468e8eb5bef283db65e4891dc280970f50fa35596e57da6afd2ee6b4d05`,
  result root `e9f888f0a611e5985b8d3d2d79323d1699186af7e2ff8362c42f5eb6f963cc16`,
  binary `55db74591b893b71fd8d329d5a28505ae890c676143181896e0c4248c7344f6c`.
- **Conclusion:** the solver-work repair is real, but no allowed HVP budget
  passes the complete 4k corpus. The first remaining blocker is a localized
  trajectory-correspondence outlier, not work exhaustion, NaN, capacity,
  permutation, density, momentum, energy or containment.
- **Decision:** close NCGP4 `REFUTED_BOUNDED`; keep dam/orifice, 16k/50k and
  timing `NOT_RUN`. Begin a new bounded diagnosis rather than changing the
  5 mm tolerance or timing a failed candidate.
- **Rejected:** declaring `0.211 mm` noncritical after the result, timing only
  the passing prefix, or treating RMSE as permission to ignore the max gate.
- **Reconsider when:** a root-closed synchronized-state experiment identifies
  a concrete implementation/semantic mismatch and one smallest repair.

### D-004 — Close NCGP5 without a code repair

- **Observation:** the step-92 outlier reaches the lower basin plane one GPU
  step after the independent CPU route. The actual GPU contact mask is `0x04`.
  From one synchronized pre-step-92 state, GPU/long-double gradient and HVP
  remain within gate and a complete CPU/GPU step differs by at most
  `0.336541 um`.
- **Evidence:** NCGP5 raw stdout SHA-256
  `8a4b0eeedaaca7e3f9daf9455f2b05c4a886bdddc78a470b1970b05cf9c3a40b`,
  result root
  `a8e7f138089874aca3e413445e7eac06ac6aa32ed2d449d2539386b0bf53e494`;
  detailed evidence is linked under Required context.
- **Conclusion:** no concrete candidate/oracle semantic defect explains the
  frozen witness. Smooth tail growth plus adjacent-step contact supports H5E
  for this bounded trajectory. The old 5 mm maximum gate still fails.
- **Decision:** select no physics, solver or tolerance repair. Ask for an
  explicit product-level decision before replacing the long-horizon
  per-particle CPU maximum with a distributional/contact-aware gate.
- **Rejected:** tuning contact/pressure/surface, accepting the old gate after
  the fact, or proceeding to 50k timing under NCGP4.
- **Reconsider when:** a new frozen product contract defines the admissible
  long-trajectory metric while retaining same-state operator and invariant
  controls.

### D-005 — Freeze the authorized product trajectory gate

- **Observation:** individual water-sample identity is not a visible product
  property after nonlinear contact, while same-state formula correctness and
  mass/density/momentum/energy/containment remain load-bearing.
- **Decision:** the user explicitly authorized NCGP6. At every 4k trajectory
  step require `RMSE <= 2.5 mm` and nearest-rank `p99 <= 2.5 mm`; report and
  seal maximum error without using it as the long-horizon rejection gate.
- **Guardrail:** retain `<= 5 um` same-state/first-step maxima, exact active
  signatures on identical state, exact GPU permutation and all physical,
  failure, work and identity controls. NCGP4 remains failed under its old gate.
- **Consequence:** implement the frozen gate, then restart the complete order;
  performance is still `NOT_RUN` until 4k, 16k/50k and sealed-basin
  correctness pass.
- **Reconsider when:** only a new measured physical or implementation failure,
  not the old step-92 per-particle maximum.

### D-006 — Close NCGP6 at the frozen p99 gate

- **Observation:** two clean byte-identical Release builds and two independent
  hydrostatic runs stop at step 112 with position p99
  `2.576882866 mm > 2.5 mm`; RMSE is `0.780996279 mm` and the diagnostic
  maximum is `20.810782528 mm`.
- **Evidence:** binary SHA-256
  `9e3e1f072b3a9ced7f5e2e42a8e35a457a8ee59c5d1ad158c879a7a53af3c2c0`,
  raw stdout SHA-256
  `5ff5d205078534ff62e3e98cb46bfe5dd79a1b9c05c8d02cc24e701e10dbc31a`,
  result root
  `4e72a97b1add42e7c58f839fc66dd918879b4325c23f75e5e5f1cf696ea57009`.
- **Conclusion:** NCGP6 is a real bounded failure, but its stable-ID p99 does
  not by itself decide whether the visible/macroscopic water state is wrong.
  Same-state first-step maximum is `0.113995 um`; density, compression,
  momentum, energy, containment and exact GPU permutation pass.
- **Decision:** stop the frozen sequence. Dam/orifice, 16k/50k, sealed-basin
  50k and performance remain `NOT_RUN`. Do not loosen p99 after observing it.
  Prepare a separately frozen Eulerian field diagnostic before asking for a
  product gate decision.
- **Rejected:** timing the passing 112-step prefix, calling the maximum or p99
  harmless without a field comparison, or treating physical invariants alone
  as proof of acceptable water behavior.
- **Reconsider when:** the exact step-112 CPU/GPU states have root-closed
  fixed-grid mass/density/free-surface evidence at predeclared resolutions.

### D-007 — Close NCGP7 as resolution-sensitive

- **Observation:** both clean runs reproduce NCGP6 exactly. Bulk Eulerian
  errors are below 1% on both grids, but coarse surface RMSE is `17.769 mm`
  while fine wet-column symmetric difference is `1.743%`; the complementary
  surface metrics pass and no coarse metric reaches H7B.
- **Evidence:** byte-identical binary SHA-256 `41d5d28a78f03c0ca150b32a531d3f5d48637f1daec528319281a7fe81e40253`,
  stdout `1b6d31323018bf02b231a241a0af25f112a7adbc987fef69b52446d775f60abb`,
  result root `d11a79d7360b9ea4b1fc3cc85c974d4e97592bc29cdc09483a6dde7f6fe70abc`.
- **Conclusion:** the bulk water state is close on this witness, but the
  current column/quantile surface QoI cannot decide whether its rare edge
  tails are visibly acceptable. This is H7C, not evidence for H7A or H7B.
- **Decision:** keep performance and the remaining corpus `NOT_RUN`. Do not
  add a third grid or change thresholds. Require a separately frozen,
  product-facing surface geometry/topology diagnostic and explicit decision.
- **Reconsider when:** a robust visible-surface QoI is defined before results
  and distinguishes sparse edge support from persistent macroscopic error.

### D-008 — Correct the surface axis before defining the product observer

- **Observation:** `nonlocal_water_corrected_profile()` retains gravity
  `(0,0,-9.81)`, while NCGP7 deposits columns in `x-z` and uses `y` as surface
  height. The hydrostatic lattice likewise places its vertical layers in `z`.
- **Conclusion:** NCGP7's 3-D bulk observables remain meaningful, but its
  wet-column and height metrics do not describe the physical free surface.
  The H7C surface result cannot select or reject a product gate.
- **Decision:** freeze NCGP8 revision 1 before code. It observes the mandatory
  debug-sphere presentation from above along `-z`, at a pitch fixed from the
  particle spacing, and compares silhouette, robust depth distribution and
  connected wet-region topology.
- **Rejected:** adding a third NCGP7 voxel resolution, relabelling `y` as
  vertical, changing gravity, or tuning a threshold after the corrected-axis
  witness is known.
- **Reconsider when:** the exact NCGP8 step-112 result and controls are
  independently reproducible.

### D-009 — Accept H8A as an author candidate, not yet corpus authority

- **Observation:** two clean processes are byte-identical and pass all frozen
  visible-surface bands: silhouette `0.1927%`, depth RMSE `4.441 mm`, p95
  `0.523 mm`, p99 `1.768 mm`, one material component and zero satellite area.
- **Evidence:** binary SHA-256 `431bfac8931150e6cc4949d8f6307ec1cb215edffd6b07be8abb03552b7cf878`,
  stdout `9681e4c17639bf3fc0fd50a4e46edc2fa02170b08b071b19c37e6a8557e98cae`,
  result root `1c953b54eefa402dbe9b66c554e3a21fcd60b2c28224398cd4fa6fcd9e998bd0`.
- **Conclusion:** the stable-ID tail does not become a persistent visible
  surface discrepancy on this one hydrostatic witness. The `507 mm` maximum
  depth pixel remains an explicit warning and requires later flow coverage.
- **Decision:** wait for the mandatory independent review. Do not run the
  remaining 4k corpus or timing from author evidence alone.
- **Reconsider when:** independent review returns GO on the exact candidate,
  or identifies one load-bearing apparatus defect.

### D-010 — Repair NCGP8 evidence closure once

- **Observation:** the initial independent review reproduced every H8A number
  but found four load-bearing apparatus gaps: NCGP7 coarse/fine bulk fields
  were not recomputed, validation flood-fill work was omitted, the top-sheet
  control shifted the whole fixture and JSON did not expose separate image
  roots/work counters.
- **Evidence:** repaired exact commit
  `8f7d98501f3dc595ce9666bb0d27a1c4564eaccb`, tree
  `bbaf24af906b788505dac4f610eba908aca026e9`; clean binary A/B
  `c5213dd1f354f86e93497ef87b6856801c38b6636809179c6fdb8545afb98e86`;
  byte-identical witness stdout
  `f22c68b1e1eeaeff53913b098bb08a14b7b540551cb9fe2bd69bf38fd998f14b`,
  result root
  `ad982ab8cf5363b5222b4a5014076d55df393cba0c1facf01e2361d1e9cbce84`.
- **Conclusion:** the repair leaves every visible metric unchanged and closes
  the four frozen findings. Six retained fields and both comparisons reproduce
  exact NCGP7 roots; ordered depth records and repeated validation work are
  sealed and published; the sheet control moves exactly 16 top-layer samples.
- **Decision:** spend the one allowed repair on this exact batch and only the
  one permitted re-review. Keep the successor corpus and performance blocked
  until that verdict.
- **Rejected:** treating the initial numerical H8A as sufficient, changing an
  H8 threshold or skipping bulk recomputation because parent state roots match.
- **Reconsider when:** the exact repair re-review returns GO or finds a
  remaining load-bearing defect; there is no second repair allowance.

### D-011 — Close NCGP8 GO and authorize only the 4k successor

- **Observation:** the re-review independently rebuilt twice, reproduced the
  112-step witness and recomputed all image/comparison/bulk/result roots from
  evidence. F1--F4 and N1--N3 are closed; no load-bearing defect remains.
- **Evidence:** exact reviewed commit `8f7d9850`, source root `ad0a835e...`,
  author witness `f22c68b1...`, result `ad982ab8...`, retained closure
  `0ed93929...`; reviewer numerical values are exact after replacing only the
  detached-path-dependent binary root and its derived result root.
- **Conclusion:** H8A is supported only for the exact hydrostatic step-112
  visible-sphere presentation. The prior stable-ID NCGP4/NCGP6 failures remain
  historical facts; H8A does not establish dam-break, orifice, 16k/50k or
  frame-time performance.
- **Decision:** close NCGP8 with review GO. Authorize a separately frozen NCGP9
  complete 4k correctness corpus and nothing later. Review allowance is
  exhausted.
- **Residual:** a missing CUDA runtime may abort during workspace construction
  before JSON; this is fail-stop/no false green and later apparatus hardening.
- **Reconsider when:** NCGP9 reaches its first predeclared physical,
  work/capacity or visible-surface result.

### D-012 — Freeze NCGP9 before the complete corpus

- **Observation:** NCGP8 closes only one hydrostatic step-112 visible witness;
  it does not cover a full second, dam break or orifice flow. Reusing the old
  stable-ID p99 would repeat the already rejected product proxy, while omitting
  temporal/bulk controls would overgeneralize one image.
- **Decision:** freeze NCGP9 revision 1 before code. Gate the reviewed visible
  surface on all 720 accepted steps, retained 3-D bulk fields at steps
  `60/120/180/240`, and same-state/physical/permutation invariants throughout.
  Dynamic-flow topology compares CPU/GPU components and satellite fractions;
  only hydrostatic hold retains the absolute single-region satellite ceiling.
- **Rejected:** timing NCGP8 directly, checking only final frames, requiring
  one connected component for physical spray, changing HVP/physics/tolerances,
  or running scenarios in whichever order finishes first.
- **Reconsider when:** the ordered NCGP9 corpus reaches its first exact result;
  later thresholds or scenario order are not tunable from that result.

### D-013 — Select only the predeclared pressure-f64 discriminator

- **Observation:** exact NCGP9 fails before step 1. Density correspondence is
  `2.40e-7` relative RMSE, but the hard pressure kink yields GPU/CPU active
  counts `1362/978`, HVP relative L2 `0.32756` and cosine loss `0.04732`.
- **Evidence:** exact commit `42ded232`, binary `dcc11c81...`, primary stdout
  `da52d965...`, result root `0c32d568...`. The pressure-only discriminator
  has exact active IDs `978/978`, HVP relative L2 `5.45e-7` and cosine loss
  `1.46e-13`; stdout `f353b0e8...`, result `bd2e5552...`.
- **Conclusion:** this is an isolated pressure active-set/coefficients failure,
  not graph ordering, surface, viscosity, inertia or full-state precision.
- **Decision:** retain NCGP9 as `PHYSICS_REFUTED_BOUNDED`. Freeze NCGP10 before
  changing the corpus route, using f64 only for pressure accumulation,
  active-set classification and pressure products. Keep f32 `(hi,lo)` state,
  all tolerances, unpreconditioned solver and 128-HVP ceiling unchanged.
- **Rejected:** an active-set epsilon, tolerance widening, full f64 state,
  CPU-provided masks, skipping the initial operator gate or timing the failed
  route.
- **Reconsider when:** the ordered NCGP10 corpus reaches its first exact result.

### D-014 — Stop NCGP10 at common CPU/GPU hydrostatic fragmentation

- **Observation:** pressure-f64 closes the initial active-set/HVP mismatch and
  completes 81 hydrostatic steps. At step 82 both CPU and GPU show six material
  components and about `2.36%` satellite area, exceeding the frozen `1%`
  hydrostatic limit. Correspondence and all earlier physical gates remain
  close: silhouette `0.0468%`, position RMSE `0.112 mm`, density RMSE
  `0.0269%`, momentum residual `0.235%`, zero positive energy excess and zero
  penetration.
- **Evidence:** exact commit `d2d660d3`, binary `c100933a...`, repeated stdout
  `d1b72200...`, corpus result `482e8374...`, scenario result `1b063d6f...`,
  exact GPU/permuted state/image roots and complete failure image work roots.
- **Conclusion:** the mixed-precision CUDA repair is real. The new failure is
  common physical behavior of the penalty-only CPU/GPU model, not a GPU port
  error or observer mismatch. A uniform lattice under gravity is not its
  discrete hydrostatic equilibrium, and finite unilateral penalty pressure
  cannot carry hydrostatic load at zero compression.
- **Decision:** close NCGP10 `PHYSICS_REFUTED_BOUNDED`; keep dam/orifice,
  16k/50k and timing `NOT_RUN`. Do not weaken the frozen topology gate. Pause
  for an explicit product choice between a pressure-state/augmented-Lagrangian
  redesign and a separately labelled invalid-physics cost diagnostic.
- **Rejected:** another precision change, surface/viscosity tuning, calling the
  six-component reference state hydrostatic, skipping to timing or treating
  CPU/GPU agreement as physical validity.
- **Reconsider when:** the user authorizes one of the two materially different
  next scopes.

### D-015 — Authorize cost measurement without a water-quality claim

- **Observation:** the current pressure-f64 CUDA route is not admissible as a
  water solver because NCGP10 fragments in the shared CPU/GPU physical model,
  but it already contains the complete scalable GPU step whose cost is needed
  to guide the next model iteration.
- **Decision:** the user explicitly selected the second D-014 option. Freeze
  NCGP11 before implementation and measure identical reset single steps at
  4k/16k/50k with the current compensated pressure-f64 route. Exclude reset
  upload from the primary CUDA-event window and report it separately.
- **Claim ceiling:** every number is
  `INVALID_PHYSICS_COST_ONLY / NO_WATER_QUALITY_CLAIM`. It cannot close the
  correctness corpus, R8, runtime integration or product readiness; CPU DFSPH
  remains fallback.
- **Rejected:** silently timing only a passing trajectory prefix, describing
  the result as correct water, weakening NCGP10, hiding a capacity/work failure
  or mixing reset/upload time into the full-step distribution.
- **Reconsider when:** the exact 50k probe either returns a typed capacity/work
  result or admits the frozen two-process sampling window.

### D-016 — Close NCGP11 above the original compute budget

- **Observation:** all 4k/16k/50k profiles admit capacity and deterministic
  work. Two fresh 50k processes measure `p50=1.111/1.116 s`,
  `p95=1.145/1.165 s` and `p99=1.176/1.168 s` for the complete GPU step.
- **Evidence:** exact source commit `982c9235`, byte-identical clean binary
  `9b45d158...`; 50k result roots `c6298eb7...` and `f731a920...`; work root
  `8ce569d4...`, step root `6111deb2...`. Full evidence is linked above.
- **Conclusion:** memory and neighbor capacity are not the blocker: 137 MB,
  5.71 million directed pairs and maximum degree 123 are admitted. The current
  implementation is about 286--291x over the 4 ms p95 budget. About 46% of the
  step is 54 graph builds and about 31% is 46 HVP.
- **Decision:** classify NCGP11
  `INVALID_PHYSICS_COST_ABOVE_ORIGINAL_BUDGET`. Do not call this water or game
  performance. Pause before changing solver/graph work because a cost-only
  optimization successor and a pressure-state physics redesign have different
  claims and validation order.
- **Rejected:** reporting only the ~1 ms standalone neighbor kernel, hiding
  repeated graph work, extrapolating from 4k, or treating capacity admission as
  game feasibility.
- **Reconsider when:** the user selects cost optimization first or physical
  correctness first.

### D-017 — Select physical correctness and isolate pressure-state viability

- **Observation:** the user selected correct physics before further timing.
  NCGP10 already shows that CPU/GPU correspondence and pressure arithmetic are
  not sufficient: the shared penalty-only model cannot hold a zero-compression
  hydrostatic state. Earlier B4E2D5/B4E2D6 research proves the PHR algebra and
  a true-kernel scalar path, but no current nominal trajectory owns that
  pressure state.
- **External check:** the Nonlocal paper presents a unified position objective,
  while implicit incompressible SPH literature independently treats pressure
  as a global constraint solve. Neither source proves the current basin
  discretization or a scalable implementation.
- **Decision:** freeze NCGP12 before code. Keep the corrected Nonlocal density
  kernel and ghost geometry, disable surface/viscosity only to isolate the
  cause, and solve one 512-centre linearized nonnegative pressure QP with an
  independently assembled long-double Jacobian.
- **Claim ceiling:** a pass authorizes only a repeated nonlinear CPU projection
  and tiny hydrostatic trajectory successor. It does not authorize GPU work,
  timing or a claim that the paper's monolithic coupling is already repaired.
- **Rejected:** another penalty/kappa sweep, hiding startup with damping,
  weakening the topology gate, calling DFSPH the Nonlocal result, or optimizing
  the NCGP11 implementation before physical selection.
- **Reconsider when:** NCGP12 returns its first exact route.

### D-018 — Pressure QP closes; ghost density support does not own contact

- **Observation:** two clean byte-identical NCGP12 runs converge in `726`
  coordinate sweeps. Jacobian error is `2.80e-11`, primal/KKT are
  `9.90e-9 / 1.56e-11`, exact maximum/RMS strain are
  `3.57e-6 / 5.80e-7`, and stationarity is `2.08e-20`. All five controls pass.
- **Failure:** only `60/512` multipliers are positive, frozen bottom/top medians
  are both zero, and the trial crosses the exact wall inset by `0.178781 mm`.
- **Evidence:** implementation `4a788ea1`, binary `008799c4...`, repeated
  stdout `be63239a...`, result `df013dd7...`; full evidence is linked above.
- **Conclusion:** the corrected Nonlocal density Jacobian admits a useful
  nonnegative constraint pressure, but ghost density support alone is not the
  wall constraint. This stage does not yet separate penalty from surface as
  the first trajectory cause.
- **Independent review:** fresh-clone Release reproduced binary `008799c4...`,
  stdout `be63239a...` and result `df013dd7...`, independently recomputed every
  root and found no load-bearing defect. It classified the zero layer medians
  as a coarse secondary diagnostic; exact `0.178781 mm` inset crossing remains
  sufficient evidence for a separate contact constraint.
- **Decision:** preserve exact `NONLOCAL_SUPPORT_REDESIGN_REQUIRED` with
  independent `GO` for its bounded claim. Freeze a pressure/contact composition
  with relinearization and only then a tiny hydrostatic trajectory.
- **Rejected:** relabelling the near-zero density error as PASS, weakening the
  exact inset post hoc, treating pressure as a replacement for contact, or
  reintroducing surface before pressure/contact holds.
- **Reconsider when:** a later pressure/contact result contradicts the exact
  retained NCGP12 witness; otherwise this discriminator is closed.

### D-019 — Freeze frictionless pressure/contact composition before code

- **Observation:** NCGP12 independent review returned `GO` and confirmed exact
  inset crossing. Three pre-code NCGP13 audits then found that the first draft
  had hidden sticking contact, an impossible `kappa=0` retained input root, no
  exact baseline radius-equality witness and underspecified precedence,
  momentum, stale-assembly and independent-contact checks.
- **Decision:** freeze NCGP13 revision 2 before implementation. Retain corrected
  `kappa` only in profile identity with zero penalty work; alternate explicit
  pressure projection with frictionless componentwise box projection; bind
  every assembly to its state root; run independent density/inset checks; and
  execute one 512-sample step before the 128-sample 240-step hold.
- **Claim ceiling:** success authorizes only separately frozen tiny CPU surface
  and viscosity discriminators. It is not correct-water, CUDA feasibility,
  performance, runtime integration or an R8 status change.
- **Rejected:** full-segment sticky stopping, quantizing the baseline graph to
  manufacture radius equality, tuning after a trajectory result, skipping to
  4k/GPU, or weakening the exact wall/density gates.
- **Reconsider when:** NCGP13 returns its first exact route and independent
  review closes its apparatus.

### D-020 — Pressure/contact step passes; the trajectory fixture is open

- **Observation:** repaired NCGP13 passes every apparatus control and its
  512-particle Phase-A step in two projection rounds. Maximum/RMS positive
  density strain are `7.40e-5 / 1.10e-5`, exact inset penetration is zero and
  normalized balance is `3.30e-16`.
- **Trajectory result:** Phase B commits step 1, then rejects private trial 2
  only because velocity RMS is `0.076470122842192428 m/s`; speed,
  displacement, energy, momentum and topology remain inside their gates. The
  accepted state remains step 1 and the failing trial is separately sealed.
- **Evidence:** reviewed repair `b1fdcd59`, tree `c541413c`, byte-identical
  Release binary `877c0e99...`, stdout `496c450f...`, final root
  `053a6a92...`. The independent re-review rebuilt twice, recomputed 72 roots,
  reran NCGP12 and returned `GO`. Full evidence is linked above.
- **Geometry finding:** the frozen `4x4x8` block is centred around
  `x,y=0.275..0.425 m` in a `3.0x2.5 m` basin. Lateral walls lie outside the
  `0.15 m` support horizon. Trial 2 is therefore almost exactly the ballistic
  value `sqrt(112/128)*2*g*dt`, with only the bottom 16 particles clamped.
- **Conclusion:** NCGP13 validly refutes its frozen Phase-B fixture, but it
  does not show that explicit pressure plus frictionless contact fails in a
  confined tank. The old label “hydrostatic hold” was physically misleading.
- **Decision:** freeze a CPU-only NCGP14 geometry/work discriminator before
  reintroducing surface or returning to CUDA. Compare open 128, open 512 and a
  tight side/bottom-supported 128-particle tank. Freeze a work-cap lane so QP
  budget exhaustion cannot be mistaken for physical failure.
- **Rejected:** weakening the velocity gate, tuning `gamma`, calling the
  ballistic open column a pressure instability, or timing the current route.
- **Reconsider when:** the tight-tank lane reaches its first independently
  reviewed two-step route with converged QPs.

### D-021 — Freeze NCGP14 Revision 5 and authorize one apparatus repair

- **Observation:** independent long-double oracle work predicts OPEN-128 and
  OPEN-512 reject trial 2, TIGHT4096 reaches its QP ceiling, and unchanged
  TIGHT16384-R8 supports both trials. Static review of the first implementation
  found only receipt, transaction, early-failure, parser and operation-work
  closure defects; two independent final audits found no remaining defect in
  the repaired contract.
- **Evidence:** audited draft SHA-256
  `e436148f31987c6f1c1632cc4fe91d186ed8db8af77ec95a5a876387241cef6a`;
  frozen status-only file SHA-256
  `4573690e22e79c999b2cdcd609747d0cb061ee59df475dd9040450c6678fd880`;
  nested contract root
  `e6d9cdce3a67818024c68ad2da7f4d2405613b7b27953678530b4a415609779f`.
- **Decision:** authorize exactly one C++/CMake apparatus repair against frozen
  Revision 5. Equations, profile, fixtures, tolerances, QP/projection caps and
  numerical lane schedule remain unchanged. No solver run or physical claim is
  admitted until clean reproduction and independent candidate review.
- **Next:** implement the frozen receipt/work/failure semantics, build twice,
  reproduce the finite two-step route, run retained regressions and sanitizers,
  then request independent review.

### D-022 — Confined pressure/contact baseline is independently supported

- **Observation:** the repaired NCGP14 apparatus reproduces the retained open
  128-particle rejection and the open 512-particle size control. The tight
  128-particle lane reaches the frozen 4096-sweep QP ceiling, while the
  otherwise identical 16384-sweep lane converges and commits both trials with
  velocity, density, containment, momentum, energy and topology inside every
  unchanged gate.
- **Evidence:** reviewed commit `d1cfe76c`, tree `bbdf9f83`, byte-identical
  Release binary `f924209d...`, stdout `15a92dff...`, result root
  `54c89f7a...`. The independent re-review rebuilt the repaired commit,
  recomputed all 85 expected/actual work pairs, 37 receipt work roots, the
  20-child aggregate, finalization and `result.v3`, and returned `GO`. Full
  evidence is linked above.
- **Conclusion:** the NCGP13 trajectory failure was primarily a fixture error:
  its open column lacked lateral support and was nearly ballistic. The
  corrected density-pressure operator plus frictionless analytic contact is a
  viable bounded support baseline in the frozen confined fixture.
- **Decision:** close NCGP14 as `INDEPENDENT_GO` for its two-step CPU
  long-double claim. Freeze the next CPU successor by adding corrected
  viscosity and surface to this exact pressure/contact baseline, first over a
  bounded short horizon and then over a longer correctness trajectory. Move
  the same corpus to CUDA only after that stage passes independent review.
- **Rejected:** calling this correct water, weakening the open-lane velocity
  gate, increasing work after the result, returning directly to 50k timing, or
  treating the diagnostic surface census as a surface-enabled trajectory.
- **Reconsider when:** the surface/viscosity successor reaches its first exact,
  independently reviewed route; NCGP14 itself is closed and its review
  allowance is exhausted.

### D-023 — Freeze one unified constrained Nonlocal step before CUDA

- **Observation:** NCGP14 supplies a reviewed pressure/contact constraint
  baseline, but it deliberately disables corrected viscosity and surface. The
  selected Nonlocal formulation couples pressure, viscosity and surface in one
  position-space variational objective; a sequential force kick plus pressure
  projection would test a different algorithm. The old finite compression
  penalty also cannot supply nonzero equilibrium pressure at exact zero
  positive strain.
- **External evidence:** the July 2026 Nonlocal paper identifies operator
  splitting artefacts as the motivation for its unified objective. DFSPH is a
  useful conventional predictor/projection comparator, not authority to rename
  an operator-split successor as the corrected Nonlocal method.
- **Decision:** freeze NCGP15 Revision 2 before code. Minimize the exact
  inertia + corrected normal-viscosity + corrected surface objective subject
  to unilateral corrected-density and analytic box constraints. Use a
  deterministic long-double PHR solve, exact corrected coefficients, explicit
  term oracles, ordered `P/PV/PS/PVS` one-step masks and exactly 16 confined
  full-term steps. All term masks, manufactured boundary/gravity modes, work
  caps, gates and roots are fixed before execution.
- **Evidence identity:** contract SHA-256
  `edce0d46d8ea1772e7c5ee6d5af21a46b27d5ebc623471cbfaf530032c5b6cf6`;
  immutable parent result root
  `54c89f7a0bd4fd13920db325a2b401591cd8cfca3690fc694440d28b42f54221`.
- **Rejected:** restoring the finite penalty, adding surface/viscosity as
  sequential kicks, warm-starting pressure across steps, tuning coefficients
  or work after a result, and returning directly to the old CUDA/50k timing
  path.
- **Consequence:** NCGP15 can support or refute only the exact 128-particle
  CPU long-double short corpus. CUDA correspondence is the sole authorized
  successor after independent GO; 4k/16k/50k correctness and performance stay
  blocked.
- **Reconsider when:** the frozen CPU corpus reaches its first exact physical,
  work or apparatus classification.

### D-024 — NCGP15 term physics passes, mutation apparatus does not

- **Observation:** the initial NCGP15 run failed only because its trace grammar
  omitted the valid unbounded `PROJECTION -> FINAL_GATE` transition. The one
  allowed one-line apparatus repair admits that transition without changing
  physics, tolerances, solver schedule or work.
- **Repaired result:** every corrected formula, viscosity, surface,
  translation, energy, zero-coefficient and transaction control passes. Total
  expected and actual work roots are identical. The repaired run still exits
  `2 / APPARATUS_INCONCLUSIVE` before Phase B because three mutation controls
  are not valid discriminators.
- **Evidence:** repaired commit `9aaf9a3a`, binary `73614eb9...`, stdout
  `6632e5d1...`, result `61951c00...`, total work `4529b808...`; full evidence
  is linked above.
- **Cause:** omitted-`2/h` and finite-penalty use an all-term TIGHT baseline
  that itself reaches the frozen work ceiling. The graph-swap fixture does
  distinguish corrected PASS from mutated work ceiling, but its control
  incorrectly excludes a typed cap from expected rejection.
- **Decision:** close NCGP15 Revision 2 as `APPARATUS_INCONCLUSIVE`; its single
  repair allowance is exhausted and it makes no physical claim. Freeze a new
  apparatus revision with a converged pressure-only baseline for the two
  pressure mutations and explicit typed-cap rejection for graph swap. Keep
  the physical corpus byte-identical.
- **Rejected:** counting mutation caps as PASS after the result, increasing a
  cap, weakening a gate, deleting mutations, or proceeding to CUDA/50k from
  passing term oracles alone.
- **Reconsider when:** the revised mutation corpus passes and independently
  admits entry to all four Phase-B masks.

### D-025 — Freeze NCGP15 Revision 3 without changing physics

- **Observation:** Revision-2 evidence isolates both remaining apparatus
  defects. Omitted-`2/h` and finite-penalty used an all-term TIGHT baseline
  that exhausted work before mutation attribution. Graph swap already gives
  corrected CSR/oracle PASS and a distinct mutated work ceiling, but the
  control excluded that typed non-commit from expected rejection.
- **Decision:** freeze Revision 3 in the existing NCGP15 contract. The two
  pressure mutations use exact TIGHT-128 with the `P` mask; graph swap retains
  its existing fixture. A finite, root/work-exact mutated cap, line-search
  exhaustion or corrected-oracle disagreement is an expected rejection only
  after corrected CSR and oracle commit and pass. Add the repaired unbounded
  trace path as a regression.
- **Guardrail:** equations, coefficients, tolerances, optimizer schedule,
  caps, Phase-B masks and Phase-C bytes remain unchanged. A corrected-route
  cap is not reclassified as mutation success.
- **Reconsider when:** the exact Revision-3 implementation returns its first
  root-closed route; no result permits post-hoc fixture or cap changes.

### D-026 — Replace the unsolved mutation fixture after bounded research

- **Observation:** Revision 3 fixes graph-swap admission, but both corrected
  TIGHT-128 pressure-only baselines stop at the 64-outer-update ceiling with
  96 active multipliers and multiplier fixed-point residual `4.08e-4`. The
  missing-chain and finite-penalty hooks are active and produce distinct
  failures/states; the corrected admission fixture is the blocker.
- **Counterfactual evidence:** an unchanged-solver binary32 `3x3x3` cube at
  `0.04 m` spacing, zero gravity and unbounded contact gives exact corrected
  CSR/all-pairs PASS in `7 / 1540` outer/inner iterations with one active
  multiplier. Missing `2/h` exhausts line search after one accepted inner
  iteration; finite penalty reaches the typed outer ceiling with density max
  `2.25e-2`. Full bounded evidence is linked above.
- **Conclusion:** the two-cycle failure was fixture conditioning, not a dead
  mutation, inactive pressure or permission to increase work. TIGHT-128
  remains the immutable physical Phase-B/C state but is not a valid unified
  mutation-admission control under the frozen ceiling.
- **Decision:** freeze exactly one Revision-4 apparatus correction using the
  small active cube for the two pressure mutations. Retain equations,
  coefficients, tolerances, optimizer schedule/caps and every physical
  Phase-B/C byte. Mutation-local typed noncommit is admissible only after
  corrected CSR/oracle PASS with finite closed exact-work evidence.
- **Rejected:** a third guessed TIGHT variant, larger caps, weaker fixed-point
  gate, deleting mutations, treating a corrected cap as mutation evidence, or
  returning to CUDA before Phase A admits the physical corpus.
- **Reconsider when:** Revision 4 reaches its first root-closed Phase-A/B/C
  classification.

### D-027 — Keep active pressure out of the manufactured-empty route

- **Observation:** the integrated Revision-4 corrected steps reach numerical
  `PASS`, but do not transactionally commit because their unbounded boundary
  tag makes the retained `manufactured_active_empty` gate applicable. The new
  fixture deliberately has one active multiplier, so the gate rejects it by
  design.
- **Counterfactual evidence:** translating the cube inside the existing
  analytical box and using binary32 spacing `0.045 m` gives corrected
  CSR/all-pairs PASS in `14 / 204`, exact zero contact, missing-chain line
  search exhaustion and finite-penalty outer-cap rejection. No solver setting
  or generic gate changes.
- **Decision:** close Revision 4 `APPARATUS_INCONCLUSIVE` and freeze Revision 5
  with only these fixture coordinates/boundary semantics changed. Preserve the
  unbounded manufactured empty-pressure regression exactly.
- **Rejected:** exempting one active fixture from the generic manufactured
  gate, interpreting a private numerical PASS as a committed control, or
  changing the box/profile/caps.
- **Reconsider when:** Revision 5 reaches its first root-closed Phase-A/B/C
  classification.

### D-028 — Replace PHR with a pressure-QP-preconditioned discriminator

- **Observation:** Revision 5 admits Phase A and all nine mutation controls.
  The first unchanged physical mask `P` stops only on the 64-outer-update
  ceiling after 1548 accepted inner iterations. Maximum/RMS positive density
  strain is `3.8861e-7 / 9.8278e-8` and projected KKT maximum is
  `1.8274e-10 m`; complementarity `2.6228e-7` and multiplier fixed point
  `4.0826e-4` remain above their `1e-8` gates.
- **Primary-source evidence:** the SIGGRAPH method and official PeriDyno code
  use SISSM coefficient splitting with a local position solve, not the nested
  PHR algorithm selected by NCGP15. A bounded exact-fixture probe shows the
  upstream default finite-penalty SISSM leaves too much density error, while
  the NCGP15 stiffness makes direct substitution converge poorly. Detailed
  hashes, formulas and observations are linked in the solver diagnosis.
- **Conclusion:** the NCGP15 result is an honest solver-work ceiling, not an
  apparatus failure and not permission to tune work. Copying upstream SISSM
  unchanged also does not meet the frozen density requirement.
- **Decision:** freeze NCGP16 around the independently reviewed NCGP14
  nonnegative-pressure QP as the pressure block inside a semi-implicit/SQP
  outer iteration. First require exact pressure-only correspondence, then run
  unchanged `PV/PS/PVS` and the confined short trajectory.
- **Rejected:** increasing outer updates after the result, dropping the dual
  gates, accepting the near-feasible PHR state, or copying upstream
  `kappa=1`/fixed-iteration SISSM as correct water.
- **Reconsider when:** the one-step pressure block and coupled masks produce
  root-closed candidate/oracle evidence under a separately frozen contract.

### D-029 — Stop apparatus expansion and isolate the free-surface mode

- **Observation:** the NCGP16 pressure-QP candidate closes all four frozen
  one-step masks and the first three confined PVS trajectory steps. Private
  trial 4 is rejected only by maximum velocity (`0.1194025561 m/s > 0.1`),
  while RMS velocity, density strain, force closure, topology and support
  remain inside their gates. Continuing the same private state to eight steps
  makes the maximum velocity grow monotonically to `0.23479 m/s` while maximum
  positive density strain remains near `7.5e-4`.
- **Exact evidence:** `/tmp/ncgp16-transient-layer-pilot.jsonl`, SHA-256
  `c1860255e204088d1fe619d17950750779b253e0b38fc20fd1f73d6f57a3a925`,
  localizes trial-4 motion to the free surface: layer 7 mean vertical velocity
  is `-0.1124822139 m/s`, layer 6 rises at `+0.0336762740 m/s`, and both have
  zero positive pressure multipliers. The matched PV run
  `/tmp/ncgp16-pv-layer-pilot.jsonl`, SHA-256
  `58a19e8f5eac25e83cc6ffc5d8fff519c1203101727677b64c9239e00a2da742`,
  gives trial-4 maximum `0.1191159265 m/s`; enabling surface changes it by only
  `+0.0002866296 m/s` (about `0.24%`, in the wrong direction).
- **Conclusion:** this is not a transient solver spike or whole-column loss of
  wall support. The current corrected surface term is not the missing repair.
  The leading cause is the free-surface pressure/support semantics: an
  underdense top layer has no active unilateral pressure while the relatively
  loose density closure can still declare the nonlinear round closed.
- **Decision:** stop expanding secondary receipt/control machinery until the
  physical discriminator selects between unilateral free-surface support and
  density-closure compliance. Keep the existing NCGP16 apparatus as a
  diagnostic checkpoint, not formal GO evidence.
- **Rejected:** treating the step-4 peak as harmless transient behavior,
  reintroducing or strengthening surface tension without a discriminator, and
  continuing toward CUDA performance while this mode is unresolved.
- **Reconsider when:** a fixed-cap two-lane experiment separates tighter
  density closure from an explicit free-surface pressure formulation without
  tuning a lane to PASS.

### D-030 — Reproduce the original sub-4-ms GPU baseline without promoting it

- **Observation:** a clean detached rebuild of historical commit
  `e2b533b49102bdff6684a7b68aa917ca635cc9e6` preserves the original
  `fused-owner-terms-p1 + compact-csr-u16-p2` implementation. The rebuilt
  binary SHA-256 is
  `35b3d8a9641ea20b51151aae4f28dfa4cd3b096732ae16ce61c92e2c0fea931b`;
  CUDA compiler/runtime are `13.3.73`, device is the RTX 3080 `sm_86`.
- **Correctness evidence:** the historical tiny corpus passes `11/11`.
  Coherent and advected P2 checks both PASS with exact old output and CSR roots.
  Each of four decision runs has `trace_exact`, `trace_memory_exact` and
  `measurement_valid`; coherent output/CSR roots are
  `6b377e876439cbc72f9b5e81cfabaed854188144c576b498a06e2769ceb4aae1` /
  `8e915c74c617f2f172f0da419b17fc5185078840459344c09b6d9818113c6fe7`,
  and advected roots are
  `bd1c5ad3a38b85ca1b7adca2333331d39c033d8b9f326b8d416213a557259e7a` /
  `1b176356938914f036682127fbfb322595fb2b6112043bb3f44247c843990392`.
- **Timing evidence:** coherent A/B p95 are `3.791424 / 3.815744 ms` and p99
  `3.998464 / 4.029344 ms`; advected A/B p95 are
  `3.593728 / 3.673024 ms` and p99 `3.739232 / 3.774688 ms`. Raw report
  SHA-256 values are
  `599a2794f9b449163c74845d1abf4216bfa7a143fc58d450b987b47694eae492`,
  `5f857564a3fb3bd9d8218e5124051dda7220e0d062e143cd5e1ebf332c2b5a7e`,
  `9dac977f2218e77b34ae813e9c20948efb2116c2582ebd7d8bed4ee15a32fde3`
  and `0eb6a73a2a12752187354639ef8e1c247e6db131498e20b7282b25e1eebf8eea`.
  The desktop compositor showed about `39--41%` GPU activity
  before the runs, so these values are conservative diagnostic reproduction,
  not a fresh uncontended performance campaign.
- **Conclusion:** the original sub-4-ms result was real and remains
  reproducible. Its compact CSR, fused owner traversal and preallocated
  decision runner are credible implementation baselines. The result does not
  cover the corrected pressure-QP/free-surface/contact semantics and cannot be
  used as evidence that corrected water fits the budget.
- **Decision:** preserve this exact historical target as the GPU performance
  denominator and reuse candidate. After selecting the CPU physical repair,
  port that repair onto this dataflow one semantic block at a time, requiring
  CPU/GPU correspondence before new timing.
- **Rejected:** calling the old PASS corrected-water performance, comparing it
  directly with the current CPU diagnostic, or discarding the original GPU
  architecture because later correctness apparatus was slow.
- **Reconsider when:** the corrected CPU trajectory passes and its exact
  pressure/free-surface/contact work has a GPU correspondence lane.

### D-031 — Use game-quality acceptance instead of requiring physical exactness

- **Observation:** the original fast GPU candidate passes its complete frozen
  numerical/oracle/trace/capacity corpus and the 50k `4/6 ms` budget. Those
  checks prove faithful execution of that model, but they do not include a
  long confined hold, visible free-surface quality, wall leakage or a
  representative gameplay splash/contact trajectory. Later formula work found
  model differences, but a game does not require laboratory-accurate water.
- **User constraint:** prefer stable, plausible and budget-compliant gameplay
  water over a 100% real-world simulation. A formula difference is not itself
  a blocker unless it causes an observable gameplay failure or breaks a hard
  invariant.
- **Decision:** promote the original sub-4-ms GPU path to the next game-quality
  candidate. Retain deterministic same-state CPU/GPU checks to detect port
  bugs, but judge the model through a small product-facing corpus: confined
  hold/coherence, dam or release motion, visible surface/topology, containment,
  finite state and bounded momentum/energy behavior. Exact long-horizon sample
  identity and agreement with the newer research model are diagnostics, not
  automatic rejection gates.
- **Repair rule:** if the fast path fails a game-quality observable, transplant
  only the smallest responsible semantic block and remeasure. Do not port the
  complete corrected research apparatus by default.
- **Rejected:** equating CPU/GPU agreement with model validity, but also
  rejecting a visually and functionally adequate game solver solely because it
  differs from the corrected research formula.
- **Reconsider when:** the game-quality corpus exposes a visible instability,
  leak, fragmentation, energy growth or contact failure.

### D-032 — Select the original five-iteration GPU route for game-quality timing

- **Observation:** the original fused-owner/compact-CSR CUDA path was run over
  three new multi-step product-facing scenarios using the H3 coefficients: a
  confined hold, a released 4x4x4 block with bottom support and free lateral/
  top clearance, and manufactured face/corner contact. Both five- and
  sixteen-iteration lanes remain finite, contained and connected, with exact
  fixed boundary publication and zero satellite samples.
- **Fixture correction:** the first release draft filled the complete lateral
  and vertical channel, so fixed ghost bulk viscosity measured piston drag,
  not a free release. It travelled `39.646 mm` at five iterations. The hard
  `50 mm` travel threshold was not changed; only lateral/top clearance was
  added before the admitted run.
- **Evidence:** two admitted processes have identical normalized JSON SHA-256
  `9c5314e653ee2237a1852ab869cdcb17689e078fe64f52e53371ca4f53d1e876`
  and result root
  `d0d0330ea63c55986746eba3a3ff61106c78d9748f86ff2172b96413de681d5b`.
  At five iterations, confined vertical drift is `2.934 mm`, released COM
  travel is `62.980 mm`, maximum release speed is `0.771 m/s`, every topology
  is one component with zero satellites, and face/corner masks pass. The
  sixteen-iteration release travels only `23.842 mm` and fails the unchanged
  motion gate. CUDA self-test, P2 correspondence and the CPU H3 corpus still
  pass after adding the observer route.
- **Conclusion:** the new stand is not inherently too strict for the old GPU.
  It selects the original five-iteration work as the better game candidate;
  the sixteen-iteration lane is slower and over-damps this motion witness.
- **Decision:** advance the five-iteration route to full-size performance
  measurement. Keep observers outside the primary timing distribution. Do not
  claim a complete game-water step until analytic contact is executed and
  timed on the GPU path; until then the existing sub-4-ms result remains the
  solver-core denominator.
- **Rejected:** lowering the travel threshold after observing the result,
  requiring corrected-QP internals from a different model, or choosing sixteen
  iterations merely because it performs more solver work.
- **Reconsider when:** a full-size run exceeds the budget, timed contact changes
  the route materially, or a larger visual corpus exposes instability/leakage.

### D-033 — Replace the ghost shell with timed analytic GPU contact

- **Observation:** removing the static ghost shell while retaining independent
  analytic box contact improves the smoke rather than destabilizing it. Both
  five and sixteen iterations pass hold/release/contact; the five-iteration
  released block travels `150.002 mm`, hold drift is `7.046 mm`, topology is
  one component with zero satellites, and maximum GPU/contact-oracle velocity
  discrepancy is `1.779e-6 m/s`.
- **Negative control:** five H3 iterations over `48k + 38,856` ghost samples are
  exact but take about `14.94--16.05 ms`, use u32 neighbor IDs and cannot meet
  the game budget. The representation, not the retained five-iteration core,
  is the first performance failure.
- **Evidence:** final binary `b6b04703...`; smoke normalized SHA
  `ca03f14c...`, result `29816d43...`. Two 48k/no-ghost/contact-included
  processes give p95 `3.797568 / 3.798176 ms` and p99
  `3.808544 / 3.874656 ms`; contact p95/p99 is `0.005120 ms`. Both have exact
  trace/output/CSR roots and 512/512 valid measurements. Full roots, commands
  and raw hashes are in the linked evidence report.
- **Conclusion:** the original compact/fused GPU architecture can meet the
  four-millisecond budget with the selected H3 coefficients and an actual
  timed wall-contact stage. Tens of thousands of fixed ghost samples are not
  required by the bounded game-quality corpus.
- **Decision:** retain five iterations, compact u16 CSR and analytic GPU box
  contact as the current game candidate. Advance to a bounded dynamic visual
  corpus before runtime/product promotion. Keep host observers outside timing.
- **Rejected:** static ghost support for the shipping performance path, sixteen
  iterations without a demonstrated quality benefit, and wholesale transfer
  of the pressure-QP research apparatus.
- **Reconsider when:** a dynamic 4k/16k visual trajectory leaks, fragments,
  grows energy visibly or shows that analytic contact alone is insufficient.

### D-034 — Admit the dynamic visual route with bounded CSR headroom

- **Observation:** NGQ2 revision 1 reached impact step 42 with a finite,
  contained, connected state but required degree `124` against the exact
  `123`-neighbor allocation. Revision 2 changed only dynamic-corpus capacity
  to `160`; both 4k and 16k then completed 96 steps with maximum degree `138`.
- **Evidence:** two processes have exact corpus result root `c2f1f6e7...`, 4k
  trace/result `369ac07d...` / `18f48386...`, and 16k trace/result
  `61c45089...` / `8e3e9f95...`. The 4k/16k front advances are
  `0.604/0.645 m`, wet-area ratios `2.019/1.665`, particle topology is one
  component with zero satellites, and final silhouette satellite areas are
  `0.846%/0%`. Full hashes, timings and frame roots are in the linked dynamic
  visual evidence report.
- **Conclusion:** the original five-iteration GPU model is plausible stylized
  water on this finite dynamic corpus. Its first scaling correction is memory
  headroom, not a changed physical formula.
- **Decision:** retain `N*160`, u16 neighbors, five iterations and analytic GPU
  contact for the next performance measurement. Do not inherit the earlier
  `N*123` 48k timing; rerun it. Presentation smoothing may follow only after
  the revised capacity still meets the budget.
- **Rejected:** classifying revision-1 capacity failure as bad physics,
  increasing capacity repeatedly after results, changing quality thresholds,
  or reintroducing the ghost shell.
- **Reconsider when:** 48k capacity-160 exceeds `4/6 ms`, a longer visual
  corpus exceeds the 2% satellite band, or renderer extraction exposes a
  materially different visual failure.

### D-035 — Admit combined capacity-160 quality and budget evidence

- **Observation:** the version-6 profile changes only compact-u16 CSR
  allocation from `123` to `160` slots per sample. The 4k/16k visual rerun
  preserves every NGQ2 trace/result root. Two sequential 48k processes each
  complete 256 conditioning runs, 64 warmups and 512 valid measurements.
- **Evidence:** p95 is `3.225760 / 3.231616 ms`; p99 is
  `3.269312 / 3.334560 ms`. Input `775f0c4a...`, trace `5b5a0b3f...`, output
  `b49be7bb...` and CSR `bee4107f...` are exact across processes. Device memory
  rises by exactly `3,552,000` bytes to `24,469,770`; the reset timing graph
  remains `5,200,628` pairs with maximum degree `123`, while the admitted
  dynamic visual graph reaches degree `138` within capacity `160`.
- **Conclusion:** the capacity correction required for visible dynamics does
  not consume the four-millisecond game budget on this RTX 3080 witness. The
  original five-iteration model now has one coherent bounded quality-and-speed
  configuration.
- **Decision:** advance to presentation-only surface smoothing/extraction.
  Keep its cost separate, forbid feedback into physics, and retain CPU DFSPH
  plus Proposed architecture status.
- **Rejected:** inheriting the old N*123 timing without measurement, lowering
  capacity after the result, removing timed contact, or treating this finite
  corpus as proof of laboratory fidelity.
- **Reconsider when:** presentation cost exhausts remaining frame budget, a
  longer visual corpus exceeds topology bands, or runtime integration changes
  the measured work.

### D-036 — Reject isotropic presentation smoothing at moving fronts

- **Observation:** NGQ4 revision 1 rejects the desired conversion from sphere
  circles to a continuous cell footprint because its area ratio is `1.333`.
  Revision 2 corrects only that observable, then the unchanged isotropic 3x3
  height pass crosses the developed step-48 front: 4k depth RMSE/p95 become
  `57.3/148.9 mm` despite exact physics and one connected mask.
- **Conclusion:** the first failure is not Nonlocal physics or mask topology.
  Ordinary height blur destroys a meaningful depth edge; widening the depth
  gate or adding more isotropic passes would hide the defect.
- **Decision:** preserve both NGQ4 failures and replace the algorithm class,
  not the gate, under NGQ5. Keep the same spatial stencil and run one frozen
  edge-aware range weighting discriminator.
- **Rejected:** accepting 149 mm p95 as cosmetic, lowering surface resolution,
  or feeding filtered height back into particles.
- **Reconsider when:** only if the root-bound edge-aware control cannot
  separate cross-edge blur from closed-pixel assignment.

### D-037 — Admit an edge-aware connected surface prototype

- **Observation:** a single 3x3 bilateral pass with range sigma equal to the
  `25 mm` particle radius passes all five 4k and five 16k frames. The embedded
  isotropic control repeats `~142--149 mm` moving-front p95, while bilateral
  p95 remains at most `8.257/7.681 mm`; every mesh has one component and zero
  bounding-box expansion.
- **Evidence:** parent corpus `c2f1f6e7...`, controls `8fefb524...`, surface
  results `c58e7c51...` / `7b089aad...`, corpus `a98f189b...`; two processes
  match semantic subset `44712cc5...`. Final meshes contain `10,071/19,708`
  and `33,634/66,510` vertices/triangles.
- **Conclusion:** HG5A is supported bounded: isotropic cross-edge averaging was
  the first presentation failure. A connected surface representation is
  viable without changing the selected GPU water state.
- **Decision:** retain this as a renderer-facing reference, not a production
  implementation. Stop CPU filter refinement; next work is GPU compute/smooth
  normals plus separate timing and visual review.
- **Remaining risk:** CPU extraction is `~3.6/13.9 ms` per keyframe and debug
  lighting still shows particle-scale texture. Blender beauty rendering was
  not available on this host.
- **Reconsider when:** GPU port exceeds its presentation budget, smooth normals
  reveal topology defects, or longer motion violates the frozen surface bands.

### D-038 — Admit the static surface into the real Vulkan presentation path

- **Observation:** both accepted NGQ5 OBJ files cook into neutral B0 content
  and render for 600 frames through the production SDL3/Ash backend. The frame
  plan and runtime report each contain one visible object and one indexed draw.
- **Evidence:** OBJ roots `23ba8765...` / `0a20f711...`; frame-plan roots
  `8eaa50fa...` / `aad92e8a...`; raw JSON `d6a1c5fa...` / `0aacd037...`.
  Release critical p95/p99 is `0.088/0.098 ms` for 4k and `0.193/0.199 ms`
  for 16k, with zero dropped timing samples.
- **Conclusion:** mesh size and existing B0 raster submission are not the next
  bottleneck. The open cost is producing and uploading changing surface data,
  not drawing the accepted static topology.
- **Decision:** retain `xtask water-preview` as the engine-facing developer
  bridge. Next define a presentation-only dynamic buffer ownership boundary;
  do not rebuild the content catalog per frame and do not feed presentation
  data back into simulation.
- **Remaining risk:** no live CUDA-to-render synchronization, GPU bilateral
  extraction, dynamic vertex upload, screenshot artifact or gameplay scene
  integration has passed yet.
- **Reconsider when:** dynamic updates force a public/runtime authority change,
  live surface cost exhausts frame headroom, or changing topology cannot reuse
  a bounded renderer allocation.

### D-039 — Refresh the surface through a declared renderer ring, not the catalog

- **Observation:** the extractor now exports all five accepted keyframes per
  lane without changing any root (corpus `a98f189b...`, final OBJ `23ba8765...`
  / `0a20f711...`). The desktop adapter gained
  `DynamicSurfaceProfileV1`/`DynamicSurfaceUpdateV1` and a per-frame-slot
  host-visible vertex/index ring keyed by one exact catalog mesh revision;
  `xtask water-preview --mesh ...` (repeated) cycles the keyframes on a pure
  pump schedule.
- **Evidence:** two dynamic Release runs per lane are semantically identical:
  catalog/snapshot/frame-plan roots `ed5b6379.../88931184.../f13b8a83...` (4k)
  and `bdd9c475.../3be1373a.../7c3dbf62...` (16k), `75` publications, `150`
  ring refreshes, `59.0/219.6 MB` copied, one dynamic draw per frame and
  `1/599` frame-plan miss/hit. Critical p95/p99 is `182--186/216--245 us` (4k)
  and `537--592/583--609 us` (16k); refresh-frame upload p95 is `140--158` /
  `469--476 us`; idle frames cost `0 us`. The single-mesh path reproduces the
  D-038 roots exactly. Raw JSON `d8b97d58.../769f607f.../a1a73823.../a204588d...`.
- **Conclusion:** changing surface topology does not require rebuilding or
  re-cooking render content, a new snapshot or a new frame plan. The next
  presentation cost is the host-visible copy and host vertex fetch, which
  already exceed the static 16k raster critical path.
- **Decision:** retain the declared ring as the adapter-private,
  presentation-only update boundary (no `crates/contracts` type, no SPEC-30
  wording change, inert unless a run declares a surface). Next move the ring
  to device-local memory and port bilateral extraction to GPU compute, timed
  through the same phase.
- **Rejected:** re-cooking the catalog per keyframe, reusing the skinning
  stream through a fake skeleton record, exposing the ring type as a public
  contract before a runtime consumer exists, and growing capacity at runtime.
- **Remaining risk:** hash-based refresh skipping is content-only, so a
  single-slot hold leaves a stale unread ring by design; `platform` and
  `host-check` both PASS after the change; screenshot readback is still absent.
- **Reconsider when:** a runtime consumer needs the surface inside
  `PresentationSnapshotV3`, device-local staging does not close the 16k gap, or
  SPEC-30 must describe the third vertex path for a shipped feature.

### D-040 — Stream the live solver into the renderer across a process boundary

- **Observation:** the corpus loop rebuilt the fixture and solver every step
  (`29.8 ms` wall at 4k for `1.5 ms` of GPU work). A persistent advected
  solver failed at step 65 (`local_solve`) because the neighbor grid is sized
  once from the initial column; a grid margin equal to the basin extent fixed
  it. Light position download with a 60-frame audit and a worker thread for
  observer/extraction/serialization brought 4k to `4.15 ms` per step.
- **Evidence:** live Release runs of 900 frames: 4k every 4 publishes `338`
  frames at real-time ratio `0.999` with render critical p95 `244 us`; 16k
  every 8 reaches `0.532`; 16k every 16 reaches `0.994`. Frames are validated
  (magic, version, bounds, counts, indices) before conversion; ring refreshes
  equal two per publication; child summaries close with `stream_closed`.
  Raw JSON `7247489f.../f9469ce4.../2cd2d074...`; sources and binaries are
  hashed in the linked evidence report.
- **Conclusion:** the engine can show the accepted five-iteration water live.
  The remaining real-time gap is the CPU reference extractor at 16k, which is
  presentation work outside the physics budget.
- **Decision:** retain the two-process, one-directional binary stream as the
  developer path; keep the solver outside the Cargo workspace. Do not present
  stream-lane trajectories as corpus evidence (float device state, sparse
  audit). Next: ordered extraction workers or GPU extraction for 16k, then a
  device-local ring.
- **Rejected:** linking CUDA into the Rust workspace, rebuilding the solver
  per step, sending particles instead of surfaces to the renderer, and
  dropping frames inside the solver process (back-pressure keeps order).
- **Remaining risk:** `--stream-cycles 0` restarts show a hard cut; a stalled
  consumer blocks the solver by design; the window ran uncapped (~160 fps),
  so pacing is by simulation time only.
- **Reconsider when:** a runtime consumer needs in-process ownership, the
  16k gap survives parallel extraction, or the stream must carry particle
  data for effects.

### D-041 — Close the 16k live gap with ordered extraction workers

- **Observation:** the 16k stream was extractor-bound (`7.05 ms` per step
  with one worker against a `4.17 ms` budget) while execute wall was
  `1.79 ms`. A bounded worker pool with sequence-ordered flushing gives
  `3.53 / 2.45 / 1.88 ms` per step for two, three and four workers, and the
  streamed geometry is byte-identical across worker counts.
- **Evidence:** live runs of 900 frames: 16k every 8 with three workers
  `0.996`, 16k every 4 with four workers `0.997`, 4k every 4 with two workers
  `0.999`; render critical p95 `924 / 985 / 329 us`; frame-source p95 fell
  from `1,077` to `251 us` for 16k after the converter assigns sequences and
  the render thread stops cloning. Raw JSON `4dc98c95.../86db22b5.../bfcf2cef...`.
- **Conclusion:** the frozen CPU extractor is sufficient for live 16k on
  this host when parallelized; no algorithm change was needed.
- **Decision:** default to three workers; keep GPU extraction as the way to
  free the CPU cores rather than as a real-time prerequisite. Next is the
  device-local ring.
- **Rejected:** changing the extraction algorithm for speed, dropping frames
  inside the solver process, and unordered output.
- **Reconsider when:** a host with fewer cores cannot keep three workers, or
  a larger lane exceeds the pool.

### D-042 — Device-local ring with producer-side packing

- **Observation:** with the host-visible ring the 16k refresh cost
  `~0.47 ms` (keyframes) to `~0.9 ms` (live) on the render thread, almost
  all of it packing 51k vertices into the B0 layout, and host vertex fetch
  raised GPU time above the static path. Moving the ring to device-local
  memory alone cut GPU p95 (`441 -> 323 us`) but not the CPU side.
- **Evidence:** packing the payload in `DynamicSurfaceUpdateV1::new` on the
  producer thread and copying staging to device-local buffers at the start
  of the frame gives 16k keyframes critical p95 `186 us` (host ring with
  packing `412 us`; D-039 `537--592 us`) and 16k live p95 `342 us` (host
  `406 us`; before `985--1,011 us`); 4k live p95 `136 us`. Ratios stay at
  `0.997--0.999`. Raw JSON `89227f16.../6abfe74c.../2b0a4272.../091bc14f.../c56e3bc3...`.
- **Conclusion:** render-thread packing, not the copy or the residency, was
  the first cost; device-local residency then halves GPU time. The ring is
  now below the static 16k raster path and stops being a bottleneck.
- **Decision:** `DeviceLocal` is the default residency for `water-preview`;
  `HostVisible` stays selectable as the control. Next work is GPU
  extraction, which is CPU-core relief rather than a frame-time need.
- **Rejected:** compute-written ring before a GPU extractor exists, and
  packing on the render thread with a larger scratch.
- **Reconsider when:** a device without a host-visible staging path appears,
  or memory for the doubled per-slot allocation becomes a constraint.

### D-043 — GPU extraction equivalent to the frozen CPU reference

- **Observation:** the frozen NGQ5 extraction ported to CUDA (splat with a
  64-bit `atomicMax` on depth bits, host largest-component labelling, close,
  fill and bilateral kernels in the reference's neighbour order) reproduces
  the CPU mask and mesh counts exactly on every frame of both lanes. A
  nanometre depth gate failed on frame 0 with identical masks: at pixels
  exactly tangent to a seed-lattice sphere `sqrt(r^2 - d^2)` amplifies
  `long double` versus `double` rounding to `5e-8 / 9e-8 m`.
- **Evidence:** `verify` over `241` (4k) and `121` (16k) frames: `0` raw or
  closed mask mismatches, `0` mesh count mismatches, maximum depth difference
  `2.9e-8 / 6.9e-8 m`; GPU extraction `0.57 / 1.21 ms` per frame against CPU
  `11 / 41 ms`; `gpu` and `verify` streams byte-identical for 4k. Live: 16k
  every 4 with one worker `0.997`, critical p95 `341 us`; 4k `1.000`,
  `87 us`; live verify `192` frames clean. Raw JSON
  `2c1a03cd.../640ced01.../4c80ac8c.../e9ce0c0b.../17730382...`.
- **Conclusion:** the presentation extractor no longer needs CPU cores or
  the reference's diagnostics at runtime; the bridge is solver-bound at both
  lanes with more than two milliseconds of 240 Hz headroom at 16k.
- **Decision:** gate GPU equivalence on identical masks and counts plus
  `1e-6 m` of depth (the engine position quantum) and record the tangent-tie
  reason; default the live bridge to `gpu`; keep the CPU reference as the
  only source of corpus roots, diagnostics and gates.
- **Rejected:** a `long double` emulation on the GPU to chase nanometres,
  changing the reference to `double`, and removing the CPU path.
- **Reconsider when:** a lane with a different pixel pitch or radius
  changes the tangent-tie bound, or a renderer-side Vulkan compute port must
  replace the CUDA one for an in-process consumer.

### D-044 — 48k lives in the bridge at paced 0.7x, not real time

- **Observation:** two 48k lanes (production `4 x 1 x 2 m` fill and a
  `48k-dam` column in `4 x 2 x 2 m`) stream through the GPU extractor at
  `1.4 ms` per frame with degree `<= 140`. Without extraction the 48k step
  costs `3.87 ms` of GPU time and `4.80 ms` of `execute` wall; 16k costs
  `1.51 / 1.77 ms`.
- **Evidence:** live runs of 900 frames: `48k` `0.739`, `48k-dam` `0.710`,
  `48k` paced at `0.7` gives `0.699` with one skipped frame; render critical
  p95 `310--321 us`. 48k verify: `61` frames, `0` mismatches, `1.13e-7 m`.
  Raw JSON `6ccc651b.../5c8638c6.../9e4b90af...`, standalone
  `ef71f217.../bf267dd3.../e03b7e25.../5e7138aa...`.
- **Conclusion:** the bridge adds nothing measurable at 48k; the solver
  alone consumes `93%` of the 240 Hz budget on this RTX 3080, so unpaced
  real time is unreachable here without a solver-side change.
- **Decision:** ship 48k in the developer bridge as paced playback
  (`--stream-rate 0.7`); do not touch the research timing apparatus for the
  `~0.9 ms` wrapper overhead inside this task. Record the fixed 240 Hz
  cadence and the accepted `3.23 ms` p95 as the constraint.
- **Rejected:** lowering the physics cadence for the demo, hiding the deficit
  by frame skipping, and rebuilding the campaign's `execute` timing path for
  a demo gain that still misses real time.
- **Reconsider when:** a faster host, a solver-side iteration or kernel
  change under its own contract, or a lower accepted physics cadence moves
  the 48k step below `~3.5 ms` wall.

### D-045 — Frame capture, run-until-close and a first look

- **Observation:** the adapter gained a bounded developer capture (one
  rendered frame copied through a transfer-source swapchain into host memory
  and reported as sRGB RGBA8) and `water-preview` a PNG writer plus
  `--until-close`; the monitor already runs FIFO at 200 Hz. The first live
  16k capture showed dark streaks and banding that measurement attributed
  to sphere-cap pits: `19--21%` of interior pixels lie `> 50 mm` below the
  maximum of their 8 neighbours even in settled water, while the 50 mm
  lattice ridge is only `7 mm`.
- **Evidence:** capture PNG `85caec85...`, raw JSON `d148e1fb...`, pit
  statistics in the visual-surface report; `platform`/`host-check` PASS after
  the adapter change.
- **Conclusion:** the visible defect was the raw height model, not shading
  or normals; the capture path is required to judge presentation work.
- **Decision:** keep the capture as SPEC-04 developer diagnostics only; no
  root or gameplay path reads it.
- **Rejected:** a software frame limiter (FIFO already bounds the window)
  and committing PNGs (captures stay outside Git).
- **Reconsider when:** a displayless capture target exists under SPEC-04's
  Proposed offscreen path.

### D-046 — NGQ6: dome envelope refuted, 5x5 closing accepted

- **Observation:** revision 1 (dome envelope, support `2 * spacing`) failed
  its frozen `lift p95 <= 2r` gate on frame 11 of both lanes with lifts of
  `0.27 / 0.22 m`: it bridged real vertical gaps in the falling column. An
  offline experiment on the streamed sphere frames showed a grayscale
  closing fills the pits instead: 5x5 reduces `> 50 mm` pits from `19.3%`
  to `2.1%` and `> 100 mm` to `0` at step 480 with a median lift of `5 mm`.
- **Evidence:** dome raw JSON `2cb1944a.../4196eb2d...`; closing verify
  runs `241 / 121 / 61` frames (4k/16k/48k) with `0` gate failures, max
  median lift `10.4 mm`, `0` ceiling violations, CPU/GPU masks and counts
  identical, depth within `2.3e-8 m` (`d6d6967f.../ab0494cb.../c1a82829...`);
  live captures `34c8da95.../0d2d8271.../9462e57d...`.
- **Conclusion:** the p95 lift is the pit depth, so it cannot be a gate; the
  structural bounds (mask unchanged, height under the local raw maximum)
  plus a median-lift limit are the right frozen observables for a closing.
- **Decision:** `closing` is the live default (`--stream-surface-model`);
  the sphere model stays the corpus reference. A static basin mesh and a
  closer camera are part of the preview scene.
- **Rejected:** tuning the dome support after the failure, a wider bilateral
  (pits exceed its range sigma), and an SPH kernel height that changes the
  mask.
- **Reconsider when:** a lane with a different pitch or radius needs another
  element size, or overhangs require a non-height-field representation.

### D-047 — The front monolayer stalls at the wall: analytic contact alone is insufficient

- **Observation:** on the live 16k lane the wet front reaches the far wall
  at step 220 as a one-particle floor sheet (`mean y = 0.025 m`, `100%`
  below `0.06 m`) moving at `+2.5 m/s`. By step 240 the `3.7--4.0 m` band
  holds `608` particles in that monolayer at `+0.06 m/s`, i.e. compressed
  `2.6x` in plane (`~31 mm` spacing), and it stays stopped and single-layer
  until step 360; the surface at the wall stays `~50 mm` while a `0.42 m`
  crest forms at `x = 3.4--3.55 m`. Only from step 400 does the wall band
  gain a second layer and move again.
- **Evidence:** particle dumps `/tmp/nonlocal-wall-dump/p16k-cycle0-step*.bin`
  from `--dump-particles` (every 4 steps, sample order stable), the streamed
  height profile table in this session, and the user's screenshot of the
  same phenomenon. The contact kernel is a positional clamp with free
  tangential motion; the rendered walls coincide with the contact box within
  `12.5 mm`, so there is no geometric obstacle.
- **Conclusion:** a floor monolayer without boundary density support can
  compress in plane until the 3D kernel reads rest density, so it carries no
  pressure and cannot be pushed; the bulk then jumps onto it upstream. The
  accepted dynamic corpus (96 steps, front at `2.7 m`) never contained the
  wall phase, so its PASS does not cover this.
- **Decision:** record the artifact as a solver-model finding, not a
  presentation defect; do not mask it in extraction or rendering. Propose a
  frozen solver-side discriminator (two-layer boundary density support for
  floor and walls in the game profile, gate: wall-band monolayer in-plane
  compression `<= 1.2` and no stall while the bulk arrives, plus step cost)
  and stop pending the user's decision, because it changes physics under
  its own contract and cost.
- **Rejected:** hiding the stalled layer in the surface model, adding
  friction or damping to the clamp to "explain" it, and extending the
  visual corpus claim to the wall phase.
- **Reconsider when:** the user authorizes the solver discriminator, or a
  density-correction-only variant proves equivalent without boundary
  samples.

### D-048 — Density-only boundary support removes the wall stall

- **Observation:** with two fixed lattice layers in every term (revision 1)
  the stall vanished but the 16k front slowed from `2.22` to `0.89 m/s`
  (no-slip drag through the viscosity terms). With the fixed samples in
  density and incompressibility only (revision 2) the front runs at
  `2.61 m/s`, the first `0.25 m` crest after arrival forms `0.06 m` from
  the wall, the stall gate reads `0` frames and the wall band layers up
  within `32` steps; 4k behaves the same (`0.04 m`, `0`, `1.22x`).
- **Evidence:** runs `66d9665c.../c41222dc.../8197a551...` (16k control,
  full, density) and `a62da60e.../06c7b406.../d4b66863...` (4k); gate
  script `lab/scripts/nonlocal_wall_monolayer_gates.py`; live captures
  `47315355.../7689f694...`; 48k cost `3.86 -> 5.35 ms` physics per step
  with one layer (`2d87c255.../c3284983...`).
- **Conclusion:** H7B holds in its density-only form; the frozen G1
  observable (`y < 0.06 m`) fails at `1.53 / 1.63` only because it counts
  second-layer samples squeezed under a loaded column, while the
  floor-touching layer stays within `1.32`.
- **Decision:** make density-only support the live default (`2` layers,
  `1` on 48k); keep corpus commands at `0` layers with unchanged roots. The
  frozen revision-3 split (floor layer `y < 0.04 m`, settled window) reads
  transient compaction under load and a settled floor layer inside the
  gate; no impact-phase compression claim is made.
- **Rejected:** fixed samples in the viscosity/surface terms, retuning the
  compression threshold to fit, and claiming a compression PASS.
- **Reconsider when:** the G1 observable is split and rerun, a 48k
  step-cost decision is made under the solver contract, or the accepted
  dynamic corpus is extended past the wall phase with these layers.

## Hypothesis ledger

| ID | Hypothesis | Current evidence | Next discriminator |
| --- | --- | --- | --- |
| H1 | scalar Jacobi leaves the coupled operator ill-conditioned | falsified as frozen: unpreconditioned succeeds with 69 HVP; no block-Jacobi | closed |
| H2 | repeated trust-region rejects/radius shrink consume work | present in Jacobi trace, but unchanged unpreconditioned rules succeed | no tuning; observe full corpus |
| H3 | binary32 operator cancellation creates a residual floor | falsified on witness by `1.43e-6` HVP relative L2 and exact active signature | closed; no pressure-f64 |
| H4 | stopping/work accounting rejects admissible execution | no count mismatch; real three-HVP diagonal work is counterproductive | remove Jacobi from selected path |
| H5A | step-92 outlier comes from a pressure active-set bifurcation | not selected: outlier CPU/GPU active flags agree on steps 80--92 | reconsider only with a new witness showing threshold divergence |
| H5B | boundary/contact semantics diverge | contact timing differs after accumulated divergence, but same-state one-step passes | no repair selected |
| H5C/H5D | compensated operator or solver/publication mismatch | falsified on exact witness by same-state gradient/HVP and sub-micrometre complete step | closed for this witness |
| H5E | admitted nonlinear trajectories separate near contact | supported bounded: smooth max tail, adjacent-step lower-wall event, small RMSE/p99 | product-level gate decision |
| H7A | stable particle identities separate while Eulerian water fields remain close | not selected: bulk passes, but both surface gates do not pass on both grids | new visible-surface QoI only |
| H7B | the NCGP6 tail reflects a real macroscopic water-state divergence | not selected: no coarse metric reaches the clear-divergence band | reconsider only on new physical evidence |
| H7C | the field verdict is dominated by arbitrary voxel resolution | superseded as a product explanation: NCGP7 surface used the wrong vertical axis; bulk evidence remains close | do not reuse NCGP7 surface metrics |
| H8A | corrected-axis visible sphere geometry remains close | selected bounded with independent GO on exact step-112 witness | freeze complete 4k successor corpus |
| H8B | the stable-ID tail is visible as macroscopic surface divergence | falsified on this witness by silhouette/depth/topology bands | reconsider on later dam/orifice evidence |
| H8C | corrected-axis observer still cannot select a product gate | not selected by author result | review may reopen only for apparatus defect |
| H9A | f32 density error is harmless away from the pressure kink | falsified: tiny density error changes 384 active centres and HVP by 32.8% | closed for primary f32 |
| H9B | f64 pressure coefficients/products close the kink without full f64 state | selected on exact same-state discriminator: active IDs exact, HVP `5.45e-7` | run frozen NCGP10 corpus |
| H10A | remaining topology failure is GPU drift | falsified: CPU/GPU both have 6 components and `~2.36%` satellites; GPU permutation exact | closed |
| H10B | observer noise creates only sparse false satellites | falsified: about 403 wet pixels lie outside the largest component in each route | closed |
| H10C | uniform penalty startup is not hydrostatic equilibrium | supported by exact common fragmentation and prior pressure-state analysis | pressure-state redesign if authorized |
| H12A | explicit nonnegative pressure is a viable repair ingredient | supported independently: QP/KKT/density close, but sole causality remains unresolved | NCGP13 pressure/contact composition |
| H12B | surface tension is the first cause | isolated pressure passes but surface-enabled successor fails | only after NCGP12 |
| H12C | ghost density support alone cannot own wall contact | selected author-side: density closes but exact inset penetration is `0.179 mm` | pressure plus analytic contact |
| H12D | one projection is insufficient, but pressure state is viable | KKT/stationarity pass and only nonlinear density fails | bounded nonlinear successor |
| H13A | explicit pressure plus frictionless analytic contact forms a viable support step | supported independently for the 512-particle one-step fixture | retain as NCGP14 baseline |
| H13B | the alternating pressure/contact composition is insufficient | not selected by Phase A; remains open for a correctly confined trajectory | NCGP14 tight-tank lane |
| H13C | one step passes but the frozen Phase-B state is dynamically unstable | selected exactly at private trial 2, but the state is a freestanding open column rather than a confined hold | do not generalize beyond the fixture |
| H14A | NCGP13 failure is caused by missing lateral support in the fixture | predicts tight-tank 128 passes while open 128/512 remain ballistic | NCGP14 geometry discriminator |
| H14B | the pressure/contact operator still fails with valid wall support | predicts converged tight-tank QPs still violate two-step physical gates | static-equilibrium/support redesign |
| H14C | the apparent tight-tank failure is only the 4096-sweep work ceiling | predicts a frozen larger-cap lane closes the same equations without tolerance changes | dual-cap NCGP14 lane |
| H14A-result | missing lateral support is the first cause of the NCGP13 witness | selected bounded: open 128/512 reject while confined 128 passes unchanged physical gates | closed for the two-step fixture |
| H14B-result | pressure/contact fails even with valid wall support | falsified for the frozen two-step confined fixture; longer coupled dynamics remain untested | surface/viscosity successor |
| H14C-result | the 4096-sweep tight failure is a work ceiling rather than physics | selected exactly: 4096 exhausts, predeclared 16384 closes with maximum 8111 sweeps | retain 16384 cap without tuning |
| H15A | corrected viscosity/surface plus constrained pressure admit one unified short solve | unresolved: Revision 5 Phase A passes, but the first physical P mask reaches the PHR outer-work ceiling | NCGP16 pressure-block SQP masks |
| H15B | corrected surface is the first failing coupled term | PV passes while PS/PVS share the first surface or energy failure; gamma-zero removes it | ordered Phase-B masks |
| H15C | normal viscosity/reference-graph semantics are first failing | PS passes while PV/PVS share the first dissipation failure; lambda-zero removes it | analytic pair plus ordered masks |
| H15D | formulation is viable but the deterministic PHR budget is insufficient | selected as the bounded Revision-5 route: candidate/oracle/state/work agree and only the outer PHR ceiling prevents admission | replace optimizer under a new frozen contract; no cap tuning |
| H16A | more PHR outer updates are the smallest repair | not selected: plausible asymptotically but post-hoc and mismatched to the published solver | do not run |
| H16B | upstream SISSM can be copied unchanged | falsified bounded: default stiffness misses density gates; NCGP15 stiffness is slow/oscillatory | closed for direct copy |
| H16C | dual gates can be dropped because the primal state is close | rejected by frozen complementarity/fixed-point contract | closed |
| H16D | reviewed pressure QP is the missing nonlinear block | selected for the next discriminator; NCGP14 closes the same confined fixture | NCGP16 one-step P then PV/PS/PVS |
| H16E | corrected surface is too weak and causes the trial-4 rejection | strongly disfavored: PV and PVS trial-4 maxima differ by only 0.24%, with PVS slightly worse | do not tune surface on this corpus |
| H16F | unilateral pressure/free-surface closure admits a persistent vertical mode | supported: top layer falls with zero positive multipliers while layer 6 rises and density gates stay closed | fixed-cap closure-vs-formulation discriminator |
| H17A | the original sub-4-ms GPU result was a stale or irreproducible artifact | falsified: exact historical source and roots reproduce in four complete 50k processes | retain as denominator only |
| H17B | the original GPU dataflow remains a useful host for corrected work | plausible: compact CSR/fused traversal retain 50k headroom, but corrected pressure/contact work is absent | port only after CPU physical selection |
| H17C | the original five-iteration GPU model is adequate for game-quality water | selected on the bounded smoke: hold/release/contact pass while sixteen iterations over-damp release | full-size timing plus timed GPU contact |
| H17D | the fast route remains coherent on a moving 4k/16k visible surface | supported bounded after the one-time 123->160 capacity correction; both lanes pass, final satellite area 0.846%/0% | presentation smoothing |
| H17E | the visually required N*160 allocation still fits the 48k game budget | selected bounded: p95 3.226/3.232 ms and p99 3.269/3.335 ms with exact semantics | presentation extraction cost |
| HG5A | isotropic presentation blur crosses real moving-front depth edges | selected bounded: bilateral p95 <=8.257 mm while isotropic control repeats >=141.897 mm | GPU/renderer port |
| HG5B | closed-pixel interpolation is the first depth-error source | falsified on the frozen frames by the edge-aware result | closed for this witness |
| HG5C | one top-down height field cannot form a connected developed-front mesh | falsified bounded: every 4k/16k extracted mesh is one component | reconsider on overhang/splash corpus |
| HG6A | accepted surface meshes are themselves too expensive for the current B0 raster path | falsified bounded for static 4k/16k: p95 `0.088/0.193 ms`, one draw | dynamic upload/GPU extraction |
| HG6B | changing surface topology forces a per-frame render-content rebuild | falsified bounded: declared ring cycles five keyframes with one catalog/snapshot/frame plan and `0` rebuilds | device-local ring, GPU extraction |
| HG6C | a host-visible ring is sufficient for live 16k surface refresh | not selected: refresh p95 `~0.47 ms` plus host vertex fetch exceed the static 16k raster path | device-local staging/compute-written ring |
| HG6D | the live solver can feed the renderer in real time across a process boundary | selected bounded: 4k `0.999` real time at 60 Hz surface; 16k `0.994` at 15 Hz, `0.53` at 30 Hz | ordered extraction workers / GPU extraction |
| HG6E | per-step solver reconstruction, not physics, dominated live stepping | selected: `29.8 -> 4.15 ms` per 4k step with a persistent advected solver, light download and threaded extraction | none; keep persistent state |
| HG6F | the frozen CPU extractor parallelizes to real-time 16k without algorithm change | selected bounded: `2.45 ms` per step with three ordered workers, geometry byte-identical | device-local ring, then GPU extraction |
| HG6G | ring residency, not CPU packing, dominates the 16k refresh cost | falsified: producer packing cut refresh `469 -> 97 us` on the host ring; device-local then halved GPU time only | closed; keep both |
| HG6H | the frozen extraction ports to GPU without changing the accepted surface | selected bounded: identical masks/counts on all verified frames, depth within `6.9e-8 m`, `0.6--1.2 ms` per frame | 48k live lane |
| HG6I | the 48k production size reaches real time through the bridge | falsified on this host: `3.87 ms` GPU per `4.17 ms` step before bridge cost; paced `0.7x` is continuous | solver-side contract, not bridge work |
| HG6J | the visible streaks are particle-scale lattice texture | falsified: 50 mm ridge amplitude `7 mm`; `19--21%` of pixels are sphere-cap pits `> 50 mm` | closed |
| HG6K | a dome envelope over the sphere mask fills pits without inventing water | falsified by its gate: lift p95 `0.27 m` at the falling column | closed; do not retune |
| HG6L | a 5x5 grayscale closing fills pits while leaving the bulk surface | selected bounded: median lift `<= 10 mm`, `0` ceiling violations, pits `> 100 mm` gone, CPU/GPU exact masks | material and front edges |
| HG7A | the invisible obstacle is a geometry or rendering mismatch | falsified: walls and contact box coincide within `12.5 mm`, the front reaches `3.994 m`, the wall band stays `50 mm` thin | closed |
| HG7B | the obstacle is a stalled, in-plane compressed floor monolayer without boundary density support | selected bounded: density-only fixed layers remove the stall, put the crest at the wall and settle the floor layer at `1.13--1.17` (control `2.25--2.27`) | 48k cost under the solver contract |
| HG7C | the stall comes from the contact clamp or the sheet itself | falsified for the stall: unchanged clamp, stall gone with density support | closed |
| HG7E | fixed samples may take part in every term | falsified: no-slip drag slows the front to `0.40x`; density-only keeps `1.18x` | closed |

## Do not retry

- NCGP3 repair/re-review; its allowance is exhausted.
- raw FCR1 profile as physical evidence;
- global one-part f32 state, host-origin localization or high-only graph/contact;
- tolerance widening or HVP above 128; pressure-f64 is allowed only in the
  frozen NCGP10 route selected by the exact isolated pressure-operator error;
- neighbor-only timing as water/frame performance.

## Next action

1. Presentation: water material within the locked B0 shader interface and
   the raw sphere heights at mask boundaries, judged through captures.
2. Extend the accepted dynamic corpus past the wall phase with density-only
   layers (new roots, new evidence) when the solver contract admits them;
   the 48k game candidate is one density-only layer at `4.98 ms` physics
   per step, paced in the bridge.
3. If later runtime integration exceeds the budget,
   transplant only the smallest responsible semantic block; do not port the
   whole research solver automatically.
4. Keep the NCGP16 CPU free-surface result as a diagnostic reference and
   counterexample corpus, not a prerequisite for accepting a simpler game
   model.
5. Preserve CPU DFSPH and keep SPEC-38/ADR-076 Proposed throughout.
