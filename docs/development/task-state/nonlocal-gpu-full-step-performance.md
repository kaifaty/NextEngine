# Nonlocal GPU full-step performance — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / NCGP15_REVISION_3_INCONCLUSIVE / REVISION_4_FIXTURE_FREEZE_NEXT` |
| Updated | `2026-09-01` |
| Task key | `nonlocal-gpu-full-step-performance` |
| Scope | Diagnose the corrected compensated solver work ceiling, close the correctness corpus, then measure the 50k full GPU step |
| Definition of done | complete frozen correctness followed by two-process 50k p95/p99 evidence, or the first honest bounded refutation |
| Authority | Working context only; SPEC-38, ADR-076/081, frozen NCGP1--NCGP4 contracts and exact evidence outrank this file |

## Resume in 60 seconds

- **Goal:** measure the complete corrected Nonlocal GPU water step on 50,000
  particles against `p95 <= 4 ms`, `p99 <= 6 ms` on RTX 3080.
- **Current boundary:** correct-water performance is still `NOT_RUN`. NCGP11
  measured only the implementation cost of the physically failed NCGP10 route
  and found 50k `p95=1.145--1.165 s`, `p99=1.168--1.176 s`.
- **First failing fact:** exact corrected NCGP3 hydrostatic hold fails GPU step
  39 at 126/128 HVP; CPU succeeds. NCGP3 is closed `INCONCLUSIVE` because its
  240-step ordering, reverse-energy apparatus, result closure and rollback
  handling were incomplete.
- **Current action:** freeze the evidence-backed NCGP15 Revision 4 mutation
  fixture. The binary32 `3x3x3`, `0.04 m` pressure-only cube gives corrected
  CSR/oracle PASS and typed rejection for both surviving pressure mutations.
  Do not return to CUDA or timing until the revised apparatus reaches the
  physical masks.
- **Latest exact result:** NCGP14 independently supports the two-step
  TIGHT-128 pressure/contact lane at the unchanged physical tolerances with a
  `16384`-sweep QP ceiling. OPEN-128/512 remain valid negative controls and
  TIGHT4096 stops only on work. Final re-review is `GO`.
- **Product ceiling:** tool-only Proposed benchmark. CPU DFSPH remains fallback;
  no Rust/public/runtime/PhysX/renderer contract changes.

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
- `docs/development/nonlocal-gpu-complete-4k-evidence-2026-08-31.md`
- `docs/development/nonlocal-gpu-pressure-f64-corpus-evidence-2026-08-31.md`
- `docs/development/nonlocal-gpu-invalid-physics-cost-evidence-2026-08-31.md`
- `docs/development/nonlocal-gpu-pressure-state-discriminator-evidence-2026-08-31.md`
- `docs/development/nonlocal-gpu-pressure-contact-trajectory-evidence-2026-09-01.md`
- `docs/development/nonlocal-gpu-confined-pressure-contact-evidence-2026-09-01.md`
- `docs/development/nonlocal-gpu-unified-constrained-evidence-2026-09-01.md`
- `docs/development/nonlocal-gpu-unified-mutation-fixture-research-2026-09-01.md`
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
| H15A | corrected viscosity/surface plus constrained pressure admit one unified short solve | unresolved: corrected term controls pass, but Revision-3 TIGHT mutation baselines cap before Phase B/C | run the evidence-backed small active mutation control, then unchanged masks |
| H15B | corrected surface is the first failing coupled term | PV passes while PS/PVS share the first surface or energy failure; gamma-zero removes it | ordered Phase-B masks |
| H15C | normal viscosity/reference-graph semantics are first failing | PS passes while PV/PVS share the first dissipation failure; lambda-zero removes it | analytic pair plus ordered masks |
| H15D | formulation is viable but the deterministic PHR budget is insufficient | unresolved: TIGHT mutation baselines cap, while a smaller active corrected solve passes unchanged caps | typed primary work-ceiling route only; no cap tuning |

## Do not retry

- NCGP3 repair/re-review; its allowance is exhausted.
- raw FCR1 profile as physical evidence;
- global one-part f32 state, host-origin localization or high-only graph/contact;
- tolerance widening or HVP above 128; pressure-f64 is allowed only in the
  frozen NCGP10 route selected by the exact isolated pressure-operator error;
- neighbor-only timing as water/frame performance.

## Next action

1. Freeze and implement NCGP15 Revision 4, changing only the two pressure
   mutation fixtures and mutation-local typed-noncommit admission; retain all
   physical bytes, equations, tolerances and caps.
2. Run its Phase A controls, ordered `P/PV/PS/PVS` masks and exactly 16
   full-term confined steps, then close two clean builds, sanitizers and one
   independent review.
3. Only after independent GO, port the identical frozen corpus to CUDA and establish
   CPU/GPU correspondence before 4k, 16k and 50k performance measurements.
4. Preserve CPU DFSPH and keep SPEC-38/ADR-076 Proposed throughout.
