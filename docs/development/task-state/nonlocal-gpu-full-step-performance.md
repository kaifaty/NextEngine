# Nonlocal GPU full-step performance — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / NCGP9_F32_REFUTED / NCGP10_PRESSURE_F64_FROZEN` |
| Updated | `2026-08-31` |
| Task key | `nonlocal-gpu-full-step-performance` |
| Scope | Diagnose the corrected compensated solver work ceiling, close the correctness corpus, then measure the 50k full GPU step |
| Definition of done | complete frozen correctness followed by two-process 50k p95/p99 evidence, or the first honest bounded refutation |
| Authority | Working context only; SPEC-38, ADR-076/081, frozen NCGP1--NCGP4 contracts and exact evidence outrank this file |

## Resume in 60 seconds

- **Goal:** measure the complete corrected Nonlocal GPU water step on 50,000
  particles against `p95 <= 4 ms`, `p99 <= 6 ms` on RTX 3080.
- **Current boundary:** performance is still `NOT_RUN`; only the neighbor stage
  has prior `~1.0--1.18 ms p95` evidence.
- **First failing fact:** exact corrected NCGP3 hydrostatic hold fails GPU step
  39 at 126/128 HVP; CPU succeeds. NCGP3 is closed `INCONCLUSIVE` because its
  240-step ordering, reverse-energy apparatus, result closure and rollback
  handling were incomplete.
- **Current action:** NCGP9 primary f32 is exactly refuted at its initial 4k
  operator gate. Its predeclared pressure-f64 discriminator passes. Implement
  the separately frozen NCGP10 mixed-precision corpus; performance remains
  blocked.
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
- `docs/development/nonlocal-gpu-complete-4k-evidence-2026-08-31.md`
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

## Do not retry

- NCGP3 repair/re-review; its allowance is exhausted.
- raw FCR1 profile as physical evidence;
- global one-part f32 state, host-origin localization or high-only graph/contact;
- tolerance widening or HVP above 128; pressure-f64 is allowed only in the
  frozen NCGP10 route selected by the exact isolated pressure-operator error;
- neighbor-only timing as water/frame performance.

## Next action

1. Implement NCGP10 by changing only the GPU route from
   `CompensatedScaleF32` to `CompensatedScalePressureF64` and sealing the
   predeclared discriminator plus arithmetic variant.
2. Run the unchanged ordered complete 4k corpus and stop at its first exact
   physical, work/capacity, visible or bulk result.
3. Keep 16k/50k, sealed-basin correctness and complete-step timing blocked
   until NCGP10 passes.
