# Continuum material physics — current task state

| Field | Value |
| --- | --- |
| Status | `W0C_HYDRO_RECALIBRATION_READY` |
| Updated | `2026-08-17` |
| Task key | `continuum-material-physics` |
| Scope | Proposed architecture and evidence-gated specifications for local water and deformable materials |
| Definition of done | Decision-complete water W0A/W0B–W6 roadmap plus independent terrain/wet/lifecycle DAG; documentation checks pass; no runtime/public-contract claim |
| Authority | Working context only; Accepted SPEC/ADR, main roadmap, exact profiles and future ProductCheck evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** the W1 serial safe-Rust CPU DFSPH oracle is
  implemented. Its clean exact-profile free-fall control passes, but the first
  interacting nominal case, `CW-HYDRO-001`, fails its first density solve at
  iteration 20 with `74,482,699 ppb`. W1-RC1 independently reproduces the full
  input/boundary projection, residual curve and representative rows exactly;
  W0C calibration, not a W1 repair, is next. No runtime or public schema is
  authorized.
- **Selected consumer:** one sealed `4 × 2 × 1 m` basin, `0.75 m` depth,
  nominal `48k`/hard `50k` samples, one `0.5 m`/`50 kg` PhysX crate and debug
  particles; unavailable capability selects an authored dry variant before
  activation.
- **Authority:** private `f64` solve, ties-to-even canonical sample
  position/velocity after every 240 Hz substep, and the next substep starts
  from that state. CPU is canonical; GPU is optional mirror only.
- **Next action:** execute
  [W0C](../../plans/continuum-water/00c-hydro-calibration-reclosure.md): density
  contribution decomposition, unchanged-profile extended curve, then
  boundary-first calibrated candidates with no corpus credit.
- **Activation gate:** the main R8 row remains `PLANNED / NOT_ACTIVE` until
  `CONTINUUM-WATER-REF-P1 = PASS`.
- **Current uncertainty:** the implementation-mismatch hypothesis is rejected
  for the RC1 audit projection; the smallest defensible boundary,
  initialization or iteration-policy revision is not selected. Full-corpus
  correctness and 50k real-time cost remain unmeasured; every `CONTINUUM-*`
  ProductCheck is `NOT_RUN`.
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
| [Standalone water roadmap](../../plans/continuum-water/README.md) | `W0B INVALIDATED_BY_RC1 / W0C READY / W1 BLOCKED` | The bounded serial oracle exists, but a successor numeric profile is required before W1 resumes |
| [W1 clean-tree discriminator](../continuum-water-w1-evidence-2026-08-17.md) | Free-fall `SCENARIO_PASS`; hydro `WATER_DENSITY_NONCONVERGENCE` | Reopens the W0B/W1 numerical boundary; `CONTINUUM-WATER-REF-P1` remains `NOT_RUN` |
| [W1-RC1 independent audit](../continuum-water-w1-rc1-audit-2026-08-17.md) | `EXACT_MATCH / REPORT_ONLY` | Rejects a production-vs-W0B mismatch for the audited projection and requires W0C recalibration |
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
  those original roots as rejected-profile evidence; W0C must issue successor
  roots before promotion.
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
| H1: fixed-point-boundary CPU DFSPH passes the clean-water corpus | Exact free-fall and same-target repeat roots pass | Independent RC1 reproduces hydro non-convergence under the original profile | W0C boundary-first counterfactual calibration and external aggregate check |
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
6. [Research report](../continuum-material-physics-research-2026-08-16.md).

## Next action

1. Add W0C counterfactual-only contribution decomposition and fixed
   `40/80/160/320` unchanged-profile convergence probes.
2. Review primary DFSPH/Akinci sources and reference behavior before encoding
   a candidate formula.
3. Evaluate boundary-derived candidates first; consider initialization next
   and a larger fixed ceiling only after stable justified density behavior.
4. Select at most one candidate, produce successor roots and rerun independent
   audit plus free-fall/hydro. Do not start W2, WG or PhysX coupling.

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
- Start terrain code while Package 10T remains profile-unclosed.

## Handoff

- **Workspace claim:** tool-only W1 serial oracle implementation plus bounded
  clean-tree discriminator and independent RC1 audit; no runtime/public schema
  or ProductCheck PASS.
- **Checks:** workspace `host-check` PASS; exact-profile free-fall
  `SCENARIO_PASS`; hydro scenario evidence `FAILED`; RC1 independent audit
  `EXACT_MATCH / REPORT_ONLY`; all `CONTINUUM-*` ProductChecks remain
  `NOT_RUN`.
- **Remaining risk:** successor hydro calibration, primary-source/reference
  validation, full-corpus accuracy, 50k performance, coupling stability, exact
  persistence implementation and all terrain constitutive evidence remain
  unmeasured.
