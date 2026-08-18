# Continuum material physics — current task state

| Field | Value |
| --- | --- |
| Status | `W0H_ACCELERATED_PRESSURE_ROOTS_FROZEN_W1_CLEAN_LINUX_RERUN_PENDING` |
| Updated | `2026-08-18` |
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
  residuals.
- **Selected consumer:** one sealed `4 × 2 × 1 m` basin, `0.75 m` depth,
  nominal `48k`/hard `50k` samples, one `0.5 m`/`50 kg` PhysX crate and debug
  particles; unavailable capability selects an authored dry variant before
  activation.
- **Authority:** private `f64` solve, ties-to-even canonical sample
  position/velocity after every 240 Hz substep, and the next substep starts
  from that state. CPU is canonical; GPU is optional mirror only.
- **Next action:** commit W0H, produce two clean identical closure projections,
  then run W1 on Linux from hydro in frozen serial order and obtain independent
  hydro/dam-break/orifice reference curves. Windows equality is deferred from
  the current execution scope, not waived for production promotion. Do not
  start W2, WG or PhysX coupling first.
- **Activation gate:** the main R8 row remains `PLANNED / NOT_ACTIVE` until
  `CONTINUUM-WATER-REF-P1 = PASS`.
- **Current uncertainty:** the fixed APG candidate completes the entire
  internal seven-scenario discriminator, including repeated still/sealed and
  all storage orders, but those runs predate the W0H freeze and receive no W1
  credit. Clean W0H closure and rooted corpus reruns remain pending. Required
  external hydro/dam-break/orifice curves are absent. Windows equality, 50k
  performance, dynamic rigid reaction, fast impact and added-mass behavior are
  also open; every `CONTINUUM-*` ProductCheck is `NOT_RUN`.
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
| [Standalone water roadmap](../../plans/continuum-water/README.md) | `W0F CLOSED / W1 READY_TO_RUN` | Successor geometry, capacity and roots are closed; the complete W1 corpus is next |
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
| [W0H accelerated pressure reclosure](../../plans/continuum-water/00h-accelerated-pressure-profile-reclosure.md) | `ACCELERATED_PRESSURE_ROOTS_FROZEN / W1_AUTHORIZED / RESEARCH_ONLY` | Roots the same pressure QP under cold fixed APG, zero-diagonal residual retention and a directional-curvature guard; clean W1 and external references remain pending |
| [Umbrella material series](../../plans/continuum-material-physics/README.md) | `SPECIFICATION_ONLY` | Terrain/wet/sleep/transfer dependencies no longer rely on the water critical path |
| [Unified world-dynamics task](world-dynamics-architecture.md) | `READY_FOR_THERMOCHEMICAL_T0B_AND_CLASSICAL_GATES` | Thermochemical and neural work are separately gated downstream tracks |
| `CONTINUUM-*` ProductChecks | `NOT_RUN` | No solver, performance, persistence or production claim is admissible |

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
  cold fixed APG schedule. W1 must validate the complete rooted corpus before
  any selection claim; cross-target equality remains a promotion gate.
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

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: fixed-point-boundary CPU DFSPH passes the Linux clean-water corpus | W0F/W0G freeze the physical and energy inputs; W0H APG completes the entire internal discriminator with exact repeat/order roots | The complete pass is diagnostic and predates W0H root freeze; required hydro/dam-break/orifice references are missing | Complete the clean W0H-rooted Linux W1 serial corpus and required external comparisons |
| H2: 50k CPU water fits the current THOTH budget | bounded sealed region and fixed profile | published prior art does not prove Next Engine 240 Hz cost | W2 exact 10k/50k/100k workload after W1 PASS |
| H3: one-pass coupling is stable for the basin crate | narrow consumer and fixed cadence | fast impact/added-mass behavior is unmeasured | W3 float/impact corpus and reaction closure |
| H4: one Drucker-Prager profile covers the first wheel scenario | established dry-sand model | exact source material and curve thresholds are not selected | Package 10T calibration closure |

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
10. [Sealed pressure-solver research](../continuum-water-w1-sealed-pressure-solver-research-2026-08-18.md).
11. [Research report](../continuum-material-physics-research-2026-08-16.md).

## Next action

1. Commit W0H and run its closure twice from a clean tree; compare bounded
   semantic projections exactly.
2. Fail on the first named scenario/criterion; do not retry, change a root,
   loosen a threshold or continue a partial corpus after failure.
3. Run the complete W0H-rooted Linux corpus from hydro and compare every
   bounded root; obtain independent hydro/dam-break/orifice curves and leave
   Windows equality explicitly deferred for production promotion.
4. Promote only if the documented W1 exit criteria pass. Do not start W2, WG,
   PhysX coupling, runtime/public contracts or ProductCheck claims meanwhile.

## Do not retry

- Add continuum types to `crates/contracts` during W1.
- Use SPlisHSPlasH source as a linked/copied engine implementation.
- Parallelize or add GPU before the serial correctness gate.
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
- Start terrain code while Package 10T remains profile-unclosed.

## Handoff

- **Workspace claim:** tool-only successor water profile with immutable W0F
  geometry/QP roots, W0G scenario-class energy roots and W0H accelerated-
  pressure roots. It is `W1_AUTHORIZED / RESEARCH_ONLY`, with no
  runtime/public schema or ProductCheck PASS.
- **Checks:** clean checkpoint `a56e57a` has exact independent
  geometry/support/density/contact comparisons, exact W0E hydro/free-fall
  regression, orifice preflight PASS and two identical bounded clean report
  projections. Format, 64 library tests, 143 xtask tests, strict Clippy and
  boundary scan pass as recorded in the linked W0F evidence. The current W0G
  worktree passes formatting, all 72 library tests, strict package Clippy and
  all six boundary checks; clean W0G closure and full-corpus evidence are still
  pending. The current W0H worktree passes all 76 crate tests and a dirty
  exact-profile closure with production/independent first-step equality;
  strict Clippy, boundary scan and two clean closures are pending. All
  `CONTINUUM-*` ProductChecks remain `NOT_RUN`; broad `host-check` was not run.
- **Remaining risk:** full Linux corpus and external-reference accuracy,
  Windows/Linux root equality, 50k performance, dynamic coupling, exact
  persistence and all terrain constitutive evidence remain unmeasured.
