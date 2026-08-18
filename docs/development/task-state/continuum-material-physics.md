# Continuum material physics — current task state

| Field | Value |
| --- | --- |
| Status | `W0E_LOCAL_PROFILE_SURVIVED_W0F_GEOMETRY_ROOT_CLOSURE_REQUIRED` |
| Updated | `2026-08-18` |
| Task key | `continuum-material-physics` |
| Scope | Proposed architecture and evidence-gated specifications for local water and deformable materials |
| Definition of done | Decision-complete water W0A/W0B–W6 roadmap plus independent terrain/wet/lifecycle DAG; documentation checks pass; no runtime/public-contract claim |
| Authority | Working context only; Accepted SPEC/ADR, main roadmap, exact profiles and future ProductCheck evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** W0E is closed
  `LOCAL_PROFILE_DISCRIMINATOR_SURVIVED / NOT_SELECTED`. Separating density
  support, projected-PCG incompressibility and analytical velocity-level
  contact fixes the local second-step failure: independent implementations
  match exactly and the cold profile passes 1200/1200 steps with zero
  penetration. No successor roots, runtime or public schema are authorized.
- **Selected consumer:** one sealed `4 × 2 × 1 m` basin, `0.75 m` depth,
  nominal `48k`/hard `50k` samples, one `0.5 m`/`50 kg` PhysX crate and debug
  particles; unavailable capability selects an authored dry variant before
  activation.
- **Authority:** private `f64` solve, ties-to-even canonical sample
  position/velocity after every 240 Hz substep, and the next substep starts
  from that state. CPU is canonical; GPU is optional mirror only.
- **Next action:** execute W0F. Freeze one geometry contract shared by density
  support and contact, add internal-wall/aperture semantics, raise or replace
  the static boundary capacity for the product extent, and issue new
  document/float/execution/corpus/scenario roots. Then run the full W1 serial
  corpus and cross-target exactness; do not start W2 first.
- **Activation gate:** the main R8 row remains `PLANNED / NOT_ACTIVE` until
  `CONTINUUM-WATER-REF-P1 = PASS`.
- **Current uncertainty:** the surviving W0E contact handles only a static
  outer box. Internal apertures, dynamic rigid reaction, fast impact and
  added-mass behavior are unmeasured. The selected product outer extent needs
  `24,704` explicit two-layer samples, above the old `16,384` capacity. PCG
  cost at 50k and full-corpus/cross-target correctness remain unmeasured;
  every `CONTINUUM-*` ProductCheck is `NOT_RUN`.
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
| [Standalone water roadmap](../../plans/continuum-water/README.md) | `W0E LOCAL SURVIVOR / W0F REQUIRED / W1 BLOCKED` | Constraint separation resolves the local failure; geometry, capacity and successor roots still block corpus credit |
| [W1 clean-tree discriminator](../continuum-water-w1-evidence-2026-08-17.md) | Free-fall `SCENARIO_PASS`; hydro `WATER_DENSITY_NONCONVERGENCE` | Reopens the W0B/W1 numerical boundary; `CONTINUUM-WATER-REF-P1` remains `NOT_RUN` |
| [W1-RC1 independent audit](../continuum-water-w1-rc1-audit-2026-08-17.md) | `EXACT_MATCH / REPORT_ONLY` | Rejects a production-vs-W0B mismatch for the audited projection and requires W0C recalibration |
| [W0C hydro-calibration cycles](../continuum-water-w0c-hydro-calibration-2026-08-17.md) | `EXACT_MATCH / BOUNDARY_AND_INITIALIZATION_CANDIDATES_REJECTED / REPORT_ONLY` | Rejects uniform scaling, ceiling-only repair, `ghost-cell-shell-v1` and zero-velocity settling; triggers adjacent-layer research escalation |
| [W0C analytical volume map](../continuum-water-w0c-volume-map-2026-08-18.md) | `EXACT_MATCH / CANDIDATE_REJECTED / W0C_RESEARCH_ONLY` | Rejects the adjacent non-particle candidate at local partition and first-step gates; requires a new explicit architecture/profile decision |
| [W0D support-complete boundary](../continuum-water-w0d-support-complete-boundary-2026-08-18.md) | `EXACT_MATCH / CANDIDATE_REJECTED / PROFILE_RECLOSURE_REQUIRED` | Falsifies one-layer support truncation as root cause and requires boundary, equilibrium and stabilization to be closed together |
| [W0E constraint-separated redesign](../continuum-water-w0e-constraint-separated-redesign-2026-08-18.md) | `EXACT_MATCH / LOCAL_PROFILE_DISCRIMINATOR_SURVIVED / NOT_SELECTED` | Separates density support, PCG pressure and contact; passes 24/1200 local steps but leaves aperture, capacity, roots and full corpus open |
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
- **Closure:** W0B fixes the Rust/LLVM targets and flags, float-environment
  probes, exact DFSPH operations/order, integer neighbor membership,
  convergence branches, corpus roots, capacities and typed failures. RC1 keeps
  those original roots as rejected-profile evidence. W0C–W0E issued no
  successor roots; only the W0F profile closure may do so before W1 resumes.
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
| H1: fixed-point-boundary CPU DFSPH passes the clean-water corpus | Exact free-fall/repeat roots pass; W0E pressure/contact implementations match independent calculators and pass 1200 local hydro steps | Internal aperture, dam-break, order, sealed-48k, full still horizon and cross-target roots have not run under the successor profile | W0F roots followed by the complete W1 corpus |
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
7. [Research report](../continuum-material-physics-research-2026-08-16.md).

## Next action

1. Close W0F's shared analytical geometry contract for outer boxes, internal
   wall patches and rectangular apertures; density and contact must derive
   from the same rooted solid/open set and match independent calculators.
2. Select an explicit static-boundary capacity of at least `24,704`; the
   recommended admitted value is `32,768`, subject to exact corpus counts and
   threshold tests. Keep dynamic rigid boundary storage separate for W3.
3. Freeze projected-PCG-50, analytical pre-integration contact, cold regular
   lattice, stabilization `NONE` and canonical integer position/velocity in
   new successor roots. Do not silently inherit rejected W0B roots.
4. Run W1 in serial: hydro, free-fall, dam-break, still, orifice, sealed-48k,
   storage-order and capacity cases, followed by exact same/cross-target roots.
   Do not start W2, WG or PhysX coupling meanwhile.

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
- Start terrain code while Package 10T remains profile-unclosed.

## Handoff

- **Workspace claim:** tool-only W1 serial oracle plus bounded clean-tree RC1
  through W0E diagnostics. The W0E candidate is a local survivor, still
  `NOT_SELECTED`; there is no runtime/public schema or ProductCheck PASS.
- **Checks:** implementation commit `7ee1651` has a clean exact-profile report
  with independent boundary/operator/PCG/contact equality, causal-control
  outcomes, 24/1200-step soak PASS and exact free-fall. Final format, tests,
  Clippy and boundary scan are recorded in the linked W0E evidence. All
  `CONTINUUM-*` ProductChecks remain `NOT_RUN`; broad `host-check` was not run.
- **Remaining risk:** internal geometry, successor roots, external/full-corpus
  accuracy, cross-target roots, 50k performance, dynamic coupling, exact
  persistence and all terrain constitutive evidence remain unmeasured.
