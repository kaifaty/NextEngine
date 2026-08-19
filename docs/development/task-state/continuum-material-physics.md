# Continuum material physics — current task state

| Field | Value |
| --- | --- |
| Status | `W2_CURRENT_PROFILES_MISS / NONLOCAL_NR2_O2_RETAINED / O3_NEXT / REPORT_ONLY` |
| Updated | `2026-08-20` |
| Task key | `continuum-material-physics` |
| Scope | Proposed architecture and evidence-gated specifications for local water and deformable materials |
| Definition of done | Decision-complete water W0A/W0B–W6 roadmap plus independent terrain/wet/lifecycle DAG; documentation checks pass; no runtime/public-contract claim |
| Authority | Working context only; Accepted SPEC/ADR, main roadmap, exact profiles and future ProductCheck evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** W0F and W0G remain immutable. W0H is closed
  `ACCELERATED_PRESSURE_ROOTS_FROZEN / W1_AUTHORIZED / RESEARCH_ONLY` after
  sealed-48k showed that the inherited active-set PCG globally restarts its
  conjugate direction and misses the unchanged 50-operator gate. W0H keeps the
  same pressure QP, geometry, contact and energy semantics, selects fixed-step
  diagonally scaled APG, and blocks on both compression and projected-KKT
  residuals. W0I rejects the old external trajectories because they cross the
  analytical wall, retains W0H unchanged, and freezes three twice-identical
  hard-clearance reference hashes under attestation root `186e1e31...65c90`.
  W1 then passes the complete seven-scenario Linux serial corpus twice at clean
  commit `e00999e`, with exact normalized report equality and corpus root
  `d38d6bc8...e96835`. Final source-layout commit `fa12d395` moves only the
  attestation policy into a private module and passes one further complete
  clean regression with the same seven scenario roots and corpus root.
- **Selected consumer:** one sealed `4 × 2 × 1 m` basin, `0.75 m` depth,
  nominal `48k`/hard `50k` samples, one `0.5 m`/`50 kg` PhysX crate and debug
  particles; unavailable capability selects an authored dry variant before
  activation.
- **Authority:** private `f64` solve, ties-to-even canonical sample
  position/velocity after every 240 Hz substep, and the next substep starts
  from that state. CPU is canonical; GPU is optional mirror only.
- **Next action:** keep the rooted water profile unchanged. Nonlocal O2 retains
  `nuv-gather-directed-r0 + pointer-swap-o1 + nuv-terms-specialized-o2` after
  exact tiny masks, stiff-surface, reused-instance and full controls. Its
  O2-only adjacent HN-3 total-p95 geometric-mean speedup is `1.0635x`, and
  same-process profiling attributes lower viscosity-launch time. O3
  accumulation-layout research is next. The failed `source-atomic-v0` surface
  record and RC1/O1/O2 boundaries remain immutable, and O2 alone receives no
  aggregate NR2/NR4 credit. The
  crate-private `64`-partition worker path is short-root exact for
  serial/`1/2/4/8` and reaches `3.105×`, but sealed-48k still averages
  `269.643 ms`. A clean standalone RTX 3080 discriminator then measures the
  dominant direct graph port at p95 `37.398 ms` in `f64` and `18.015 ms` in
  mixed arithmetic versus the complete `4 ms` target. The GPU reaches `100%`
  SM utilization and the dependent pressure/matrix traversals dominate, so do
  not spend a long run or launch-only tuning merely refining this miss. The
  [RC1 evidence](../nonlocal-continuum-nr1-rc1-evidence-2026-08-19.md)
  records gather p95 `4.630 ms` for water-48k, `2.632 ms` for water-16k and
  `8.693 ms` for viscous-16k, with `3.116x` geometric-mean p95 speedup on the
  two HN-3 denominator profiles and no added device memory.
  [O1 evidence](../nonlocal-continuum-nr2-o1-evidence-2026-08-20.md) records
  its separate adjacent p95 of `4.626 ms` water-48k, `2.503 ms` water-16k and
  `8.313 ms` viscous-16k with unchanged memory.
  [O2 evidence](../nonlocal-continuum-nr2-o2-evidence-2026-08-20.md) records
  its separate adjacent p95 of `3.595 ms` water-48k, `1.903 ms` water-16k and
  `6.608 ms` viscous-16k with exact runtime correspondence and unchanged
  memory. Nsight Compute counters are explicitly unavailable under
  `ERR_NVGPUCTRPERM`; no counter values are inferred.
- **Activation gate:** `CONTINUUM-WATER-REF-P1=PASS` is satisfied in the
  dedicated worktree and its checkpoint is merged into mainline by `fe223f9`.
  R8 is active only as isolated W2 research; W3 integration remains blocked by
  the performance miss.
- **Current uncertainty:** Windows/Linux equality, 50k performance, dynamic
  rigid reaction, fast impact and added-mass behavior remain open. Nonlocal
  formula reproduction, deterministic gather reclosure and retained O1/O2 now
  have bounded local evidence; retained O3–O6 attribution and the final NR4
  48k/local-domain decision remain unknown. W1 supplies only Linux serial
  correctness; every later `CONTINUUM-*` ProductCheck is `NOT_RUN`.
- **Do not retry:** public `ContinuumMaterialSystem` first, GPU authority,
  hidden warm-start/float continuation, iterative coupling, sleep before exact
  persistence, or wet terrain before dry-sand evidence.
- **Independent later branches:** SPEC-43 owns temperature/composition/phase;
  SPEC-44 warm-start advice starts only after W1-W6 promotion. Neither changes
  the water worktree's next action or receives water-gate credit.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| [Research report](../continuum-material-physics-research-2026-08-16.md) | `REPORT_ONLY` | Supports solver-family separation; proves no implementation |
| [SPEC-38](../../architecture/38-continuum-material-physics.md) and [ADR-076](../../architecture/adr/076-continuum-material-physics-track.md) | `Proposed` | Candidate CPU authority, fixed-point boundary, one-pass coupling and exact-active semantics are closed |
| [Standalone water roadmap](../../plans/continuum-water/README.md) | `W1 LINUX PASS / W2 SHORT SCALING IN PROGRESS` | Cycle 2 is implemented; formal full-trajectory equality and percentile credit remain open |
| [W1 clean-tree discriminator](../continuum-water-w1-evidence-2026-08-17.md) | Free-fall `SCENARIO_PASS`; hydro `WATER_DENSITY_NONCONVERGENCE` | Reopens the W0B/W1 numerical boundary; `CONTINUUM-WATER-REF-P1` remains `NOT_RUN` |
| [W1-RC1 independent audit](../continuum-water-w1-rc1-audit-2026-08-17.md) | `EXACT_MATCH / REPORT_ONLY` | Rejects a production-vs-W0B mismatch for the audited projection and requires W0C recalibration |
| [W0C hydro-calibration cycles](../continuum-water-w0c-hydro-calibration-2026-08-17.md) | `EXACT_MATCH / BOUNDARY_AND_INITIALIZATION_CANDIDATES_REJECTED / REPORT_ONLY` | Rejects uniform scaling, ceiling-only repair, `ghost-cell-shell-v1` and zero-velocity settling; triggers adjacent-layer research escalation |
| [W0C analytical volume map](../continuum-water-w0c-volume-map-2026-08-18.md) | `EXACT_MATCH / CANDIDATE_REJECTED / W0C_RESEARCH_ONLY` | Rejects the adjacent non-particle candidate at local partition and first-step gates; requires a new explicit architecture/profile decision |
| [W0D support-complete boundary](../continuum-water-w0d-support-complete-boundary-2026-08-18.md) | `EXACT_MATCH / CANDIDATE_REJECTED / PROFILE_RECLOSURE_REQUIRED` | Falsifies one-layer support truncation as root cause and requires boundary, equilibrium and stabilization to be closed together |
| [W0E constraint-separated redesign](../continuum-water-w0e-constraint-separated-redesign-2026-08-18.md) | `EXACT_MATCH / LOCAL_PROFILE_DISCRIMINATOR_SURVIVED / NOT_SELECTED` | Separates density support, PCG pressure and contact; passes 24/1200 local steps but leaves aperture, capacity, roots and full corpus open |
| [W0F geometry/capacity/root closure](../continuum-water-w0f-geometry-capacity-root-closure-2026-08-18.md) | `EXACT_MATCH / SUCCESSOR_PROFILE_ROOTS_FROZEN / W1_AUTHORIZED` | Closes shared aperture geometry, oriented support, swept contact, static capacity and clean repeatable successor roots; gives no corpus credit |
| [W1 successor energy discriminator](../continuum-water-w1-successor-energy-discriminator-2026-08-18.md) | `REPORT_ONLY / IMPACT_ENERGY_CONTRACT_RECLOSURE_REQUIRED / NO_W1_CREDIT` | Shows the frozen dam-break solver is healthy apart from an inherited energy-contract contradiction; rejects three pressure/contact counterfactuals |
| [W0G impact-energy reclosure](../../plans/continuum-water/00g-impact-energy-contract-reclosure.md) | `SUCCESSOR_IMPACT_ENERGY_ROOTS_FROZEN / W1_AUTHORIZED / RESEARCH_ONLY` | Roots scenario-class energy semantics without changing W0F operations; missing external references still block W1 |
| [W1 canonical-clearance discriminator](../continuum-water-w1-canonical-clearance-discriminator-2026-08-18.md) | `REPORT_ONLY / RUNNER_POLICY_DEFECT_CONFIRMED / NO_W1_CREDIT` | Restores the rooted inclusive `2,500 µm` canonical gate after an unrooted successor-runner zero check stopped orifice at a `1 µm` publication artifact; contact and roots stay unchanged |
| [W1 sealed pressure-solver research](../continuum-water-w1-sealed-pressure-solver-research-2026-08-18.md) | `ACCELERATED_PROJECTED_GRADIENT_SELECTED_FOR_W0H_RECLOSURE / NO_W1_CREDIT` | Exact traces reject a ceiling increase and tested MPRGP variants; fixed step `0.25` survives the complete internal discriminator with compression/KKT gates |
| [W0H accelerated pressure reclosure](../../plans/continuum-water/00h-accelerated-pressure-profile-reclosure.md) | `ACCELERATED_PRESSURE_ROOTS_FROZEN / W1_AUTHORIZED / RESEARCH_ONLY` | Roots the same pressure QP under cold fixed APG, zero-diagonal residual retention and a directional-curvature guard; downstream W1 now passes |
| [W0I external reference attestation](../../plans/continuum-water/00i-external-reference-geometry-attestation.md) | `REFERENCE_GEOMETRY_ATTESTATION_FROZEN / W1_AUTHORIZED / RESEARCH_ONLY` | Rejects geometry-violating external trajectories, freezes three exact hard-clearance reference hashes and leaves W0F/G/H unchanged |
| [W1 hard-clearance reference evidence](../continuum-water-w1-hard-clearance-reference-reclosure-2026-08-18.md) | `LINUX_W1_PASS / CONTINUUM-WATER-REF-P1=PASS` | Two clean complete runs attest all required references, pass 7/7 scenarios and reproduce the same corpus/report projection roots |
| [W2 resource-utilization discriminator](../continuum-water-w2-resource-utilization-2026-08-19.md) | `CYCLE_2_SHORT_ROOT_EXACT / 3.105X / NO_W2_CREDIT` | Workers `1/2/4/8` preserve the short root; `269.643 ms` is still `67.41×` the target, so an architecture decision precedes long gates |
| [W2 algorithm/data-structure research](../continuum-water-w2-algorithms-and-data-structures-research-2026-08-19.md) | `DRAFT_RESEARCH / OPTIONS_NOT_SELECTED / NO_W2_CREDIT` | Separates exact DFSPH layout probes, multilevel-QP research and a new particle-grid profile; records ranked bounded experiments without selecting one |
| [W2 GPU feasibility discriminator](../continuum-water-w2-gpu-feasibility-2026-08-19.md) | `DIRECT_GRAPH_PORT_MISSES_4MS / GPU_ARCHITECTURE_UNDECIDED / NO_W2_CREDIT` | Clean RTX 3080 proxy p95 is `37.398 ms` `f64` and `18.015 ms` mixed before mandatory omitted stages; direct backend transfer is rejected as a 4 ms solution, while broader GPU/solver research remains open |
| [Nonlocal primary-source audit](../nonlocal-unified-continuum-source-audit-2026-08-19.md) | `PRIMARY_SOURCES_INSPECTED / NR1_EXECUTED` | Confirms a published CUDA/SISSM baseline and concrete optimization targets while rejecting universal-solver and immediate-integration claims |
| [Nonlocal bounded research roadmap](../../plans/nonlocal-continuum/README.md) | `NR2_O2_RETAINED_TERM_SPECIALIZATION / O3_NEXT / NO_W2_CREDIT` | Exact viscosity specialization passes O2 retention and admits the layout tournament; DFSPH roots and authority remain unchanged |
| [Nonlocal NR1 baseline evidence](../nonlocal-continuum-nr1-baseline-evidence-2026-08-19.md) | `BASELINE_MISMATCH / NR2_BLOCKED` | Tiny/water/viscous controls pass; source-shaped surface atomics amplify repeated `f32` order noise beyond state tolerances |
| [Nonlocal NR1-RC1 evidence](../nonlocal-continuum-nr1-rc1-evidence-2026-08-19.md) and [plan](../../plans/nonlocal-continuum/03-nr1-deterministic-accumulation-reclosure.md) | `NR1_RECLOSED_GATHER_DIRECTED / NR2_UNBLOCKED` | CPU/CUDA gates and exact stiff-surface repeats pass; this remains the immutable input boundary before retained O1 |
| [Nonlocal NR2-O1 evidence](../nonlocal-continuum-nr2-o1-evidence-2026-08-20.md) and [plan](../../plans/nonlocal-continuum/04-nr2-o1-pointer-swap.md) | `O1_RETAINED_POINTER_SWAP / O2_NEXT` | Copy/swap exactness, stale-state controls, unchanged memory and adjacent retention gates pass; no W2/NR4 credit |
| [Nonlocal NR2-O2 evidence](../nonlocal-continuum-nr2-o2-evidence-2026-08-20.md) and [plan](../../plans/nonlocal-continuum/05-nr2-o2-term-specialization.md) | `O2_RETAINED_TERM_SPECIALIZATION / O3_NEXT` | Exact runtime/specialized correspondence, unchanged memory, adjacent gates and same-process profiler attribution pass; no aggregate NR2/W2/NR4 credit |
| [Umbrella material series](../../plans/continuum-material-physics/README.md) | `SPECIFICATION_ONLY` | Terrain/wet/sleep/transfer dependencies no longer rely on the water critical path |
| [Unified world-dynamics task](world-dynamics-architecture.md) | `READY_FOR_THERMOCHEMICAL_T0B_AND_CLASSICAL_GATES` | Thermochemical and neural work are separately gated downstream tracks |
| `CONTINUUM-WATER-REF-P1` | `PASS / LINUX_W1_PASS / RESEARCH_ONLY` | Admits W2 work; does not imply performance, coupling, persistence, runtime or production readiness |
| Later `CONTINUUM-*` ProductChecks | `NOT_RUN` | No later-stage or production claim is admissible |

## Decisions that constrain the next work

### D-001 — Solver family, not solver monoculture

- **Evidence:** the research report and SPEC-38.
- **Decision:** CPU DFSPH for water; APIC/MLS-MPM for one calibrated dry-sand
  profile; separate material-specific SoA.
- **Rejected:** one SPH solver/particle record for water, sand, mud and snow.
- **Reconsider when:** the same frozen corpora show one method strictly
  dominates without weakening state/history semantics.

### D-002 — CPU canonical fixed-point boundary

- **Observation:** an internal `f64` trajectory with hidden pressure/warm-start
  state is not an exact replay contract.
- **Decision:** after each water substep publish only stable sample ID,
  micrometre position and micrometre-per-second velocity; uniform mass is in
  the profile, other solve fields are rebuilt, and warm start is disabled.
- **Closure:** W0B's original roots remain rejected-profile evidence. W0F
  freezes the successor Rust/LLVM profile, shared exact geometry, oriented
  two-layer support, analytical swept contact, `32,768` static capacity and
  the pressure QP under new document/profile/corpus/scenario/fixture roots.
  W0H later replaces only the failing projected-PCG algorithm with a rooted
  cold fixed APG schedule. W1 now validates the complete rooted Linux serial
  corpus twice; cross-target equality remains a promotion gate.
- **Rejected:** float owner state, final-output-only quantization and GPU-first
  canonical execution.
- **Reconsider when:** a later consumer and exact cross-target evidence require
  a versioned larger continuation state.

### D-003 — One-pass composite coupling

- **Decision:** freeze rigid input, solve water, emit one canonical reaction
  batch, let PhysX integrate once, then publish both owners or neither.
- **Rejected:** delayed reaction, wall-time-selected iteration count, duplicate
  contact writers and partial water/rigid commits.
- **Reconsider when:** the W3 corpus fails a frozen stability threshold and a
  bounded fixed-iteration alternative passes without changing authority.

### D-004 — Sealed pinned-active exact persistence first

- **Decision:** one region, no halo/transfer, exact active state in a composite
  checkpoint; sleep/wake is a later lossy conversion with separate roots and
  repeated-cycle evidence.
- **Rejected:** sidecar save, active-root equals sleep-root claim, camera-driven
  eviction and reconstructing dirty state from immutable content.
- **Reconsider when:** W5 passes and a real streaming consumer requires 20X or
  21X.

### D-005 — Product and budget gate

- **Decision:** 50k is the production particle gate; 100k is stress/report.
  Promotion must stay inside current THOTH physical `4/6 ms` p95/p99 and
  integrated `8/12 ms` ceilings.
- **Stop rule:** after two evidence-backed W2 optimization cycles, a miss keeps
  the track `RESEARCH_ONLY`. A smaller gate, larger budget or GPU authority is
  a new decision.

### D-006 — Terrain material ladder

- **Decision:** exact calibrated dry sand → serial reference → exclusive
  prescribed-wheel contact → exact active persistence → saturation/drainage →
  optional closed free-water flux.
- **Rejected:** generic soil claim, simultaneous PhysX/MPM ground contact,
  full vehicle first and wet material before dry persistence.
- **Current blocker:** Package 10T has no calibrated exact constants/curve
  thresholds; terrain code must not start until it closes them.

### D-007 — External water references share hard geometry

- **Observation:** the original SPlisHSPlasH dam-break reference crosses the
  analytical wall before the first W0H curve mismatch; matching its later
  height would require accepting forbidden states.
- **Decision:** W0I keeps W0F/G/H immutable, adds an independent predictive
  hard-contact adapter and requires three exact twice-reproduced reference
  hashes before production-credit execution.
- **Rejected:** no-contact curve matching, density-map/barrier tuning against
  the invalid splash, symmetric explosive internal Akinci support and
  shape-only `CWREFV1` admission.
- **Reconsider when:** a new pinned independent implementation supplies a
  predeclared profile that passes the same hard geometry and unchanged curve
  thresholds; it requires a new attestation root and cannot inherit W0I credit.

## Hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: fixed-point-boundary CPU DFSPH passes the Linux clean-water corpus | Two clean complete runs at `e00999e` pass 7/7 scenarios, attest 3/3 required references and reproduce the exact corpus and normalized report roots | No counterexample under the frozen Linux W1 profile | `CLOSED / LINUX_W1_PASS` |
| H2: 50k water fits the current THOTH budget without a new rooted algorithm/authority profile | bounded sealed region, `3.105×` CPU-worker speedup and strong GPU reconstruction acceleration | short 8-worker CPU mean is `269.643 ms`; clean direct GPU proxy p95 is still `37.398 ms` `f64` / `18.015 ms` mixed before omitted stages | `REJECTED_FOR_CURRENT_CPU_AND_DIRECT_GPU_PROFILES`; decide stop versus new rooted profile |
| H3: one-pass coupling is stable for the basin crate | narrow consumer and fixed cadence | fast impact/added-mass behavior is unmeasured | W3 float/impact corpus and reaction closure |
| H4: one Drucker-Prager profile covers the first wheel scenario | established dry-sand model | exact source material and curve thresholds are not selected | Package 10T calibration closure |
| H5: source-faithful Nonlocal/SISSM can justify a separately rooted continuum profile | gather reclosure and O1/O2 pass all gates; retained O2 p95 is `3.595 ms` water-48k and `6.608 ms` viscous-16k; same-process profiler confirms lower specialised viscosity time | source-atomic stiff surface remains invalid; O3–O6, final profiler and production constraints are incomplete; Compute counters are unavailable | specify O3 accumulation-layout tournament in order; no NR4 claim yet |

## Required context

1. [Agent routing](../../architecture/agent-routing.md), SPEC-26 and current
   physics ADRs.
2. [Main roadmap](../../roadmap.md), R8, B-10 and performance boundaries.
3. [SPEC-38](../../architecture/38-continuum-material-physics.md) and ADR-076.
4. [Water roadmap](../../plans/continuum-water/README.md), especially W0B/W1.
5. [W0C calibration reclosure](../../plans/continuum-water/00c-hydro-calibration-reclosure.md).
6. [W0E constraint-separated reclosure](../../plans/continuum-water/00e-constraint-separated-profile-reclosure.md).
7. [W0F geometry/capacity/root closure](../../plans/continuum-water/00f-geometry-capacity-and-root-closure.md).
8. [W0G impact-energy reclosure](../../plans/continuum-water/00g-impact-energy-contract-reclosure.md).
9. [W0H accelerated-pressure reclosure](../../plans/continuum-water/00h-accelerated-pressure-profile-reclosure.md).
10. [W0I external reference attestation](../../plans/continuum-water/00i-external-reference-geometry-attestation.md).
11. [Hard-clearance reference evidence](../continuum-water-w1-hard-clearance-reference-reclosure-2026-08-18.md).
12. [Sealed pressure-solver research](../continuum-water-w1-sealed-pressure-solver-research-2026-08-18.md).
13. [Research report](../continuum-material-physics-research-2026-08-16.md).
14. [W2 algorithm/data-structure research](../continuum-water-w2-algorithms-and-data-structures-research-2026-08-19.md).
15. [Nonlocal research roadmap](../../plans/nonlocal-continuum/README.md) and
    [source audit](../nonlocal-unified-continuum-source-audit-2026-08-19.md).

## Next action

1. Specify and execute Nonlocal NR2 O3 from the retained
   `nuv-gather-directed-r0 + pointer-swap-o1 + nuv-terms-specialized-o2`
   identity, keeping layout work separate from O5 pass fusion.
2. Preserve the failed source-atomic record and RC1/O1/O2 evidence boundaries;
   do not promote adjacent speedups directly to aggregate NR2 or NR4 credit.
3. Do not run long percentile/corpus repetitions merely to refine the current
   CPU/direct-GPU miss or a failed Nonlocal feasibility cutoff.
4. Keep Windows equality explicitly deferred for production promotion; do not
   infer it from Linux worker equality.
5. Do not start W3 PhysX coupling, runtime/public contracts, persistence or
   production promotion before W2 closes.

## Do not retry

- Add continuum types to `crates/contracts` during W1.
- Treat a direct flattened-neighbor CUDA port, CUDA graphs or launch fusion
  alone as sufficient for `4 ms`; measured work is dominated by the dependent
  pressure/matrix graph traversals, not reductions or idle hardware.
- Link or vendor the full PeriDyno framework into Next Engine for the bounded
  feasibility question.
- Implement Semi-Implicit Pairwise Descent from its title while the public
  paper/code remain unavailable.
- Count adaptive early exit as fixed-iteration speedup or accept position delta
  alone as a convergence residual.
- Use SPlisHSPlasH source as a linked/copied engine implementation.
- Skip W2's deterministic parallel/equality gates or add GPU authority before
  W2 closes.
- Persist density, pressure, neighbor, warm-start or render cache as V1 water
  authority.
- Hide a non-convergent frame by retaining the previous water state and
  continuing the evidence run.
- Increase the 20-iteration ceiling, loosen the density tolerance or alter the
  boundary/lattice under the existing W0B roots.
- Retry `ghost-cell-shell-v1`, a uniform boundary multiplier, or another
  ceiling-only variant; the bounded W0C cycle has already rejected them.
- Retry `zero-velocity-settle-v1` or a position-only damping coefficient; its
  exact transcript shows growing solver demand and wall penetration.
- Retune the volume-map `0.8` factor or virtual-point offset under W0C; the
  face/edge/corner correction factors differ and the candidate is closed.
- Add a density map, XSPH/stabilization term, warm pressure continuation,
  settling output or retry without a named scenario failure and a new joint
  profile decision. W0E passes its local gate without them.
- Raise the W0H 50-operator ceiling, restore global active-set PCG, retry the
  rejected standard/projected-expansion MPRGP variants or use APG step `0.5`.
- Drop zero-inverse-diagonal rows from compression/KKT reductions, introduce
  backtracking/dynamic steps or hide pressure state between canonical frames.
- Reuse the old non-clearance SPlisHSPlasH files, tune density/contact to their
  invalid splash height, or admit an external file by shape without exact W0I
  hash attestation.
- Start terrain code while Package 10T remains profile-unclosed.

## Handoff

- **Workspace claim:** tool-only successor water profile with immutable W0F
  geometry/QP roots, W0G scenario-class energy roots, W0H accelerated-pressure
  roots and exact W0I reference attestation. It is
  `CONTINUUM-WATER-REF-P1=PASS / LINUX_W1_PASS / RESEARCH_ONLY`, with no
  runtime/public schema or production-backend claim.
- **Checks:** W1 checkpoint `e00999e` is clean. The complete
  seven-scenario Linux corpus passes twice, all required external hashes are
  attested, corpus root is `d38d6bc8...e96835`, and normalized report SHA is
  `2dffa4e3...52a0bf` for both runs. Final layout checkpoint `fa12d395` passes
  one further complete clean regression with identical roots, 78/78 crate
  tests, strict Clippy, formatting and all six boundary checks. Its deliberately
  stopped second full run has no status. W2 checkpoint `c2915cf` passes 91/91
  crate tests, Clippy and boundary scan; its clean short serial/1/2/4/8 roots
  match exactly. Broad `host-check` was not run.
- **GPU discriminator:** clean CUDA commit `01a4c18` passes its exact
  sealed-lattice self-test. A 50-run APG40 proxy reports p95 `37.398 ms` `f64`
  and `18.015 ms` mixed; Nsight Systems attributes the solve primarily to the
  pressure and matrix traversals. It is `REPORT_ONLY`, has no root or
  ProductCheck credit, and does not change CPU authority.
- **Selected next research:** Nonlocal O2 retains exact compile-time viscosity
  specialization. O3 accumulation-layout research is next from the
  gather/swap/specialized identity; all work remains standalone/report-only
  with no W2 or production credit.
- **Remaining risk:** full-trajectory and Windows worker equality, formal
  percentiles, dynamic coupling, persistence and terrain evidence remain open.
