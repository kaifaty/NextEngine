# Structural vegetation physics — current task state

| Field | Value |
| --- | --- |
| Status | `READY_FOR_VEGETATION_V0B_CALIBRATION` |
| Updated | `2026-08-17` |
| Task key | `vegetation-physics` |
| Scope | Proposed layered physical-world model and evidence-gated structural vegetation specifications |
| Definition of done | SPEC-37/SPEC-38/ADR-073, standalone V0–V7 vegetation roadmap, research report and main R8 status are coherent; no runtime/public-contract claim |
| Authority | Working context only; Accepted SPEC/ADR, main roadmap, exact future profiles and ProductCheck evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** V0A product decisions are complete; implementation is
  not ready. Close [V0B](../../plans/vegetation-physics/00-product-profile-and-evidence-closure.md#v0b-required-exact-tree-definition)
  before creating solver code.
- **Selected consumer:** `Next Engine Reference Conifer V1`, 10 m, 12 major
  branches, at most 128 structural segments, rigid root clamp, analytical wind,
  production-path axe notch/back cut, 8x32 section cells, at most 32 tapered-
  capsule proxies and atomic handoff to one PhysX compound body.
- **Authority:** living structures own rooted graph/elastic/damage/topology;
  PhysX alone owns rigid bodies; peer owners publish one composite
  `PhysicalStep` or neither publishes.
- **First implementation:** serial CPU bake-off between a shearable geometrically
  exact rod and a constrained/implicit discrete-rod or corotational baseline.
- **Activation gate:** main R8 row remains `PLANNED / NOT_ACTIVE` until
  `VEGETATION-BEAM-REF-P1 = PASS`.
- **Selected gate:** 1,000 visible, 128 modal, 8 active, one refined and at most
  two falling trees; incremental THOTH CPU 2/3 ms p95/p99 inside integrated
  physical 8/12 ms p95/p99.
- **Current blocker:** exact graph/taper bytes, synthetic wood constants,
  numeric scales, candidate cadence ladders, command traces/curve hashes and
  remaining collision/LOD/memory/transition thresholds are absent.
- **Do not retry:** GPU-first authority, visual mesh/rigid chain as tree state,
  scalar HP, camera/timing LOD, flexible falling crown V1, fire/root/soil inside
  the base milestone, or public contracts before a production consumer.
- **Independent later branches:** SPEC-41 owns heat/moisture/composition and
  combustion progress; SPEC-42 model advice starts only after the classical
  tree track promotes. Neither changes V0B/V1 or receives vegetation credit.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| [Source research review](../vegetation-physics-research-2026-08-16.md) | `REPORT_ONLY` | Supports sparse graph/section damage and corrects solver/authority/LOD scope; proves no implementation |
| [SPEC-37](../../architecture/37-layered-physical-world.md), [SPEC-38](../../architecture/38-structural-vegetation-physics.md) and [ADR-073](../../architecture/adr/073-layered-physical-world-and-living-structures-track.md) | `Proposed` | Candidate owner DAG, tree authority, coupling, fracture, persistence and fallback semantics are closed |
| [Vegetation roadmap](../../plans/vegetation-physics/README.md) | `V0A COMPLETE / V0B OPEN / V1 NOT_STARTED` | Product choices are fixed; code remains blocked on numeric/profile/corpus calibration and later lanes cannot bypass the serial oracle |
| [Unified world-dynamics task](world-dynamics-architecture.md) | `READY_FOR_THERMOCHEMICAL_T0B_AND_CLASSICAL_GATES` | Thermochemical and neural work are separately gated downstream tracks |
| `VEGETATION-*` ProductChecks | `NOT_RUN` | No solver, tree, fracture, coupling, persistence, LOD, target or performance claim is admissible |

## Decisions that constrain the next work

### D-001 — Layers are ownership and transaction boundaries

- **Observation:** stacking water, trees, wind, fire and rigid physics does not
  identify writers or atomic failure.
- **Decision:** activation/profile -> canonical schedule -> immutable forcing
  -> peer state owners -> canonical batches/composite commit -> outcomes/
  persistence -> presentation.
- **Rejected:** shared mutable physical-state bus and a universal solver API.
- **Reconsider when:** two non-rigid production owners need a demonstrated
  common public exchange consumer.

### D-002 — One living-structures owner

- **Decision:** rooted graph, elastic/damage/section/topology state belong to
  Physical Embodiment's living-structure lane. Render mesh and foliage follow.
- **Rejected:** visual mesh simulation, PhysX joint chain and scalar tree HP.
- **Reconsider when:** V1 corpus proves the sparse model cannot satisfy the
  fixed product curves.

### D-003 — CPU formulation gate before runtime

- **Observation:** “Cosserat rod” does not close integrator, stiff axial modes,
  state or cadence.
- **Decision:** V1 compares one shearable geometrically exact rod with a
  constrained/implicit discrete-rod or corotational baseline under one corpus.
  CPU is authority; GPU is optional aggregate mirror.
- **Stop rule:** neither candidate passing the V0 fixed-step profile keeps the
  track research-only; no runtime tree follows.

### D-004 — Section geometry, not HP

- **Decision:** one authored felling zone uses a bounded polar cell lattice;
  cut work changes cells; derived remaining area/centroid/moments and
  directional strength drive hinge/failure.
- **Rejected:** arbitrary cuts/saw/fibres in V1, `fell_now`, hit-point threshold
  and visual notch as authority.
- **Reconsider when:** V3 passes and a concrete second cutting consumer needs a
  broader model.

### D-005 — Staged PhysX coupling and ownership handoff

- **Decision:** PhysX integrates once against frozen tree proxies, emits
  canonical loads, then the tree advances once. A severed component transfers
  atomically to one PhysX compound body and leaves living ownership.
- **Rejected:** two rigid writers, callback mutation, wall-time coupling loops
  and flexible falling crown in the first vertical.
- **Reconsider when:** V4 corpus fails predeclared impact/stability bounds and a
  fixed alternative profile is proposed explicitly.

### D-006 — Exact active persistence before LOD breadth

- **Decision:** V5 exact graph/section/topology save/restart precedes modal/
  sleep persistence and forest streaming. LOD uses only canonical simulation
  facts and integer tokens.
- **Rejected:** reconstructing damage from content, sidecar save, camera/
  frustum/frame-time tier selection and downgrade during cut/contact/fall.

### D-007 — Future phenomena are separate lanes

- **Decision:** thermal/moisture/combustion, roots/soil, tree-to-tree fracture,
  detailed fibres and cross-region forest streaming each require their own
  state/profile/coupling/persistence evidence.
- **Rejected:** one broad `VegetationSystem` milestone or crate tree that
  implies these capabilities.

### D-008 — V0A product and evidence profile

- **Decision:** use the synthetic 10 m `Next Engine Reference Conifer V1`, 12
  major branches, rigid root, 128-segment/32-proxy capacities, one 8x32 felling
  section, axe-only production cut and one detached PhysX compound body.
- **Cadence/state:** 240 Hz outer step, fixed-point continuation and one fixed
  internal candidate profile selected by the V1 bake-off.
- **Evidence:** static <=2%, natural frequencies <=5%, normalized RMSE <=5%,
  maximum curve error <=10%, work/impulse residual <=1%, exact mass, handoff
  CoM <=1 mm and momentum residual <=1%; same-target exact in V1 and cross-
  target exact before promotion.
- **Performance:** 1,000 visible, 128 modal, 8 active, one refined, at most two
  falling; incremental THOTH CPU 2/3 ms p95/p99 inside integrated 8/12 ms.
- **Consequence:** V0A is closed, but these bounds do not manufacture the V0B
  constants, fixture curves or hashes required before code.

## Open V0B calibration

| Decision | Required output |
| --- | --- |
| Exact tree bytes | trunk centreline/taper, 12-branch graph, felling-zone span, segment masses/drag and exact node count within selected capacities |
| Synthetic material | reduced orthotropic constants, strengths/fracture, damping, moisture reference and provenance for `Next Engine Reference Conifer V1` |
| Numeric profile | all fixed-point scales/bounds, rotation/state fields, candidate cadence ladders, convergence and failure codes |
| Fixture corpus | exact calm/steady/gust and pull traces, axe command trace/cut-work coefficients, curve formulas, samples and hashes |
| Remaining thresholds | metric normalization/applicability, collision penetration and LOD transition discontinuity/history limits |
| Remaining performance | static/shader split, tier cadence/transition tokens, THOTH memory/transition budgets and report-only stress fixtures |

## Required context

1. [Agent routing](../../architecture/agent-routing.md) and current SPEC-26/
   ADR-058 rigid authority.
2. [SPEC-37](../../architecture/37-layered-physical-world.md),
   [SPEC-38](../../architecture/38-structural-vegetation-physics.md) and
   [ADR-073](../../architecture/adr/073-layered-physical-world-and-living-structures-track.md).
3. [Vegetation roadmap](../../plans/vegetation-physics/README.md), especially
   V0 and V1.
4. [Research report](../vegetation-physics-research-2026-08-16.md).
5. [Main roadmap](../../roadmap.md), R8, B-10 and performance boundaries.

## Next action

1. Calibrate and freeze the exact tree bytes and synthetic wood constants.
2. Freeze numeric scales, candidate cadence ladders, fixtures, external curve
   hashes and the remaining thresholds/budgets in V0B.
3. Create a dedicated vegetation worktree from that documentation checkpoint.
4. Implement only the serial V1 oracle and stop at the first failed criterion.
5. Update this task-state only when V0B closes or evidence changes the approach.

## Handoff

- **Workspace claim:** documentation-only Proposed architecture/specification;
  no runtime, schema, content format or ProductCheck implementation.
- **Expected checks:** documentation cheap path only; Cargo/host-check and all
  vegetation executable checks remain `NOT_RUN`.
- **Remaining risk:** all physical accuracy, material calibration, fixed-point
  trajectory, coupling stability, exact persistence, forest performance and
  cross-target behavior remain unmeasured.
