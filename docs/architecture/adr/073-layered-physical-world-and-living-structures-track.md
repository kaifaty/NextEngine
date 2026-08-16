# ADR-073: Layered physical world and living-structures track

| Field | Value |
|---|---|
| ID | ADR-073 |
| Status | Proposed |
| Version | 1.0 |
| Decision date | 2026-08-16 |
| Last verified | 2026-08-16 |
| Normative dependencies | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-25](../25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](../30-presentation-extraction-and-render-content.md), [SPEC-36](../36-continuum-material-physics.md), [SPEC-37](../37-layered-physical-world.md), [SPEC-38](../38-structural-vegetation-physics.md), [ADR-027](027-physics-motor-and-animation-layering.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-071](071-canonical-physics-material-lineage.md), [ADR-072](072-continuum-material-physics-track.md) |
| Supersedes | none; proposes a future composition model and vegetation lane without changing Accepted PhysX authority |
| Superseded by | none |

## Context

The current engine has one Accepted rigid/articulation owner and one Proposed
continuum-material lane. Destructible trees add sparse elastic structure,
directional damage and topology changes that fit neither a rigid-body chain nor
a continuum particle monoculture. A top-level diagram that simply stacks
water, trees, wind, fire, rigid bodies and gameplay leaves writers, order,
failure and persistence ambiguous.

The source vegetation brief correctly argues for a sparse structural skeleton,
anisotropic damage and physical cutting rather than a visual-mesh solver or
tree HP. It also proposes GPU-first storage, relevance LOD, many subsystems and
dozens of module boundaries before a calibrated product case exists. Those
parts would conflict with CPU/cross-target authority, consumer-driven contracts
and the current product-first scope.

## Proposed decision

### Layer by ownership and commit, not by effect name

Adopt the SPEC-37 layer map: immutable activation/profile closure; canonical
schedule/identity; revision-bound environmental forcings; peer physical state
owners; canonical coupling/composite commit; committed outcomes/persistence;
read-only presentation.

Rigid/articulated, continuum and living-structure solvers are peer owners under
Physical Embodiment. They exchange frozen projections and bounded canonical
batches at one fixed `PhysicalStep`; none directly mutates another. Future
thermal/combustion and root/soil owners join only through separately promoted
exchange profiles. There is no generic solver bus or raw shared field store.

PhysX remains the sole writer of rigid/articulation state. A later production
consumer requires a narrow Accepted decision that preserves this rule while
adding exact owner segments and coupling records.

### Add one living-structures lane

Create one Proposed living-structures lane for trees and similar sparse rooted
structures. Its canonical candidate is CPU `f64` with a fixed-point boundary;
GPU is optional correspondence only. It owns a stable rooted structural graph,
elastic/damage state, section cells and topology. Visual mesh, foliage and VFX
remain projections.

The first player-visible consumer is one procedural trail-side tree: analytical
wind, production-path notch/back-cut, geometry-derived hinge failure, atomic
graph split and a bounded detached-component handoff to PhysX. The standing
tree uses structural collision proxies; after handoff PhysX alone advances the
detached compound body.

No exact rod formulation is selected by the word *Cosserat*. A serial bake-off
against a constrained/implicit discrete-rod or corotational baseline must first
freeze the formulation, integrator, fixed internal cadence and complete
continuation state. This is `VEGETATION-BEAM-REF-P1` and is the entry gate for
runtime integration.

### Damage is section state, not HP

The initial destructive representation is one bounded polar cell lattice at a
declared felling zone. Validated tool/contact input modifies cell state through
a frozen fixed-point cut-work mapping. Remaining area, centroid and section
moments determine load capacity. Fracture selects one unique failing section,
partitions the graph and commits topology plus PhysX handoff atomically.

Detailed strands, arbitrary fracture surfaces, saw kerf simulation, secondary
fragmentation and micro-scale cutting are later research. Presentation may add
splinters or cut meshes only after the structural result.

### Exact active state before forest breadth

One tree remains pinned active until exact active save/restart passes. Modal,
shader and sleep tiers are explicit representation transitions driven only by
canonical simulation facts and integer budgets. Camera visibility, measured
frame time and GPU completion cannot select authority. Damage or unstable
contact blocks lossy downgrade.

Forest LOD follows the destructible vertical and exact persistence, but it is
required before production promotion. Fire/moisture, decay, roots, deformable
soil, tree-to-tree fracture and GPU authority are independent future lanes and
do not receive completion credit from the base tree.

### Defer public contracts and exact biomechanical profile

Do not add vegetation types to `crates/contracts` for the lab. Package V0 must
freeze one tree definition, calibrated orthotropic wood profile, wind/cut
fixtures, numeric state scales, capacities, reference curves and thresholds
before solver code. USDA clear-wood tables and graphics papers inform that
profile but are not themselves a calibrated living-tree corpus.

The first production tree consumer may introduce only definition/profile,
canonical state, rigid exchange, topology/handoff, presentation and composite
checkpoint contracts needed by the vertical. A new consumer-backed Accepted
ADR assigns their exact current-only versions.

## Failure and fallback

Any invalid profile, nonfinite/overflow, non-convergence, stale revision,
batch collision, capacity excess, impossible graph split, handoff conservation
failure, backend rejection or corrupt checkpoint rejects the complete coupled
step. The prior generation remains authoritative; there is no partial tree or
rigid commit, retry-to-green or frozen-tree continuation.

Before activation, an authored static rigid tree is the deterministic fallback.
After activation, a scripted fall, decorative damage, static replacement or GPU
switch is forbidden. Failure stops the affected physical run and preserves the
last complete checkpoint.

## Promotion and stop conditions

The vegetation roadmap remains `PLANNED / NOT_ACTIVE` while Package V0 is open.
After V0 closure, the isolated serial oracle may run; only
`VEGETATION-BEAM-REF-P1 = PASS` permits an active R8 integration track.

Production promotion additionally requires tree/wind, fracture, rigid coupling,
exact persistence, LOD and Windows/Linux root gates plus a THOTH performance
profile fixed before measurement. If the selected active/modal forest workload
misses its declared budget after two evidence-backed optimization cycles, the
track remains research-only. Reducing the tree/branch gate, enlarging the
budget or granting GPU authority requires a new explicit decision.

## Alternatives rejected

- Visual mesh or rigid-joint chain as structural authority: wrong degrees of
  freedom and joint-like failure.
- One scalar HP/damage value: cannot represent directional notch, hinge area or
  hidden local damage.
- Full volumetric/fibre tree from day one: no product/budget evidence supports
  the state and topology cost.
- GPU-first authority: final quantization does not prove equivalent trajectory
  history across targets.
- Camera-distance LOD: renderer state would select physical outcomes.
- Fire/root/soil inside the base tree milestone: they have different owners,
  state variables and validation corpora.
- A generic `VegetationSystem` plugin API and many crates before the first
  consumer: rejected by ADR-046 and B-10.

## Consequences

- SPEC-37, SPEC-38 and this ADR remain Proposed; current runtime/schema/save
  semantics and the PhysX baseline do not change.
- The next implementation action is Package V0 calibration, not solver code.
- The physical-world model can add phenomena without shared mutable state or a
  universal solver.
- Vegetation and continuum programs remain independent until an explicit
  root/soil or water/vegetation exchange consumer exists.
