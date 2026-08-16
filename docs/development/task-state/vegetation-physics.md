# Structural vegetation physics — current task state

| Field | Value |
| --- | --- |
| Status | `READY_FOR_VEGETATION_PACKAGE_00` |
| Updated | `2026-08-16` |
| Task key | `vegetation-physics` |
| Scope | Proposed layered physical-world model and evidence-gated structural vegetation specifications |
| Definition of done | SPEC-37/SPEC-38/ADR-073, standalone V0–V7 vegetation roadmap, research report and main R8 status are coherent; no runtime/public-contract claim |
| Authority | Working context only; Accepted SPEC/ADR, main roadmap, exact future profiles and ProductCheck evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** architecture is ready, implementation is not. Close
  [V0](../../plans/vegetation-physics/00-product-profile-and-evidence-closure.md)
  before creating solver code.
- **Selected consumer:** one pinned-active procedural trail-side tree; analytical
  wind; production-path notch/back cut; section-derived hinge failure; atomic
  detached-component handoff to one PhysX compound body; debug presentation.
- **Authority:** living structures own rooted graph/elastic/damage/topology;
  PhysX alone owns rigid bodies; peer owners publish one composite
  `PhysicalStep` or neither publishes.
- **First implementation:** serial CPU bake-off between a shearable geometrically
  exact rod and a constrained/implicit discrete-rod or corotational baseline.
- **Activation gate:** main R8 row remains `PLANNED / NOT_ACTIVE` until
  `VEGETATION-BEAM-REF-P1 = PASS`.
- **Current blocker:** exact reference tree, wood/anchor profile, cut mapping,
  numeric scales, curve thresholds, capacities and forest workload are absent.
- **Do not retry:** GPU-first authority, visual mesh/rigid chain as tree state,
  scalar HP, camera/timing LOD, flexible falling crown V1, fire/root/soil inside
  the base milestone, or public contracts before a production consumer.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| [Source research review](../vegetation-physics-research-2026-08-16.md) | `REPORT_ONLY` | Supports sparse graph/section damage and corrects solver/authority/LOD scope; proves no implementation |
| [SPEC-37](../../architecture/37-layered-physical-world.md), [SPEC-38](../../architecture/38-structural-vegetation-physics.md) and [ADR-073](../../architecture/adr/073-layered-physical-world-and-living-structures-track.md) | `Proposed` | Candidate owner DAG, tree authority, coupling, fracture, persistence and fallback semantics are closed |
| [Vegetation roadmap](../../plans/vegetation-physics/README.md) | `V0 OPEN / V1 NOT_STARTED` | Code is blocked on profile/evidence closure; later lanes cannot bypass the serial oracle |
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

## Open V0 decisions

| Decision | Required output |
| --- | --- |
| Reference identity | synthetic test wood or named species/condition, with explicit claim boundary |
| Tree geometry | exact height/taper/branch graph, felling zone, masses, drag and collision capacities |
| Wood and anchor | reduced orthotropic constants, strengths/fracture, damping and fixed root constraint |
| Numeric profile | all fixed-point scales/bounds, rotation/state fields, outer/internal cadence and iterations |
| Cut model | tool/contact trace, ring/sector counts, cut-work mapping, notch/back-cut curve and failure key |
| Forest gate | exact tier counts/DOFs/proxies/cells, transition schedule and THOTH p95/p99/memory budgets |

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

1. Answer the six V0 decision groups with product/design and material evidence.
2. Freeze exact fixtures, external curve hashes and thresholds in V0.
3. Create a dedicated vegetation worktree from that documentation checkpoint.
4. Implement only the serial V1 oracle and stop at the first failed criterion.
5. Update this task-state only when V0 closes or evidence changes the approach.

## Handoff

- **Workspace claim:** documentation-only Proposed architecture/specification;
  no runtime, schema, content format or ProductCheck implementation.
- **Expected checks:** documentation cheap path only; Cargo/host-check and all
  vegetation executable checks remain `NOT_RUN`.
- **Remaining risk:** all physical accuracy, material calibration, fixed-point
  trajectory, coupling stability, exact persistence, forest performance and
  cross-target behavior remain unmeasured.

