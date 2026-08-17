# ADR-077: Layered physical world and living-structures track

| Field | Value |
|---|---|
| ID | ADR-077 |
| Status | Proposed |
| Version | 1.1 |
| Decision date | 2026-08-16 |
| Last verified | 2026-08-16 |
| Normative dependencies | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-25](../25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](../30-presentation-extraction-and-render-content.md), [SPEC-38](../38-continuum-material-physics.md), [SPEC-39](../39-layered-physical-world.md), [SPEC-40](../40-structural-vegetation-physics.md), [ADR-027](027-physics-motor-and-animation-layering.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-071](071-canonical-physics-material-lineage.md), [ADR-076](076-continuum-material-physics-track.md) |
| Candidate revision note | Version 1.1 records the selected V0A vegetation profile without changing Accepted PhysX authority; the imported candidate was renumbered to avoid the occupied mainline namespace |
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

Adopt the SPEC-39 layer map: immutable activation/profile closure; canonical
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

V0A selects the synthetic `Next Engine Reference Conifer V1`: 10 m height,
12 major branches, at most 128 structural segments, a rigid root clamp and at
most 32 tapered-capsule proxies. One felling zone has an 8 by 32 polar section
lattice. The fixed outer cadence is 240 Hz, while the serial bake-off selects a
fixed internal cadence from a predeclared corpus. Canonical continuation state
is fixed-point at each outer boundary and private `f64` state cannot cross it.

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

Do not add vegetation types to `crates/contracts` for the lab. V0A closes the
product choices; V0B must freeze the exact tree graph/taper, calibrated
synthetic orthotropic profile, wind/cut fixtures, numeric state scales,
capacities, reference curves and remaining thresholds before solver code. USDA
clear-wood tables and graphics papers inform that profile but are not
themselves a calibrated living-tree corpus.

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

The vegetation roadmap remains `PLANNED / NOT_ACTIVE` while V0B is open. V0A
is complete and records product scope, authority, cut/section model, LOD gate,
performance budgets and validation targets. After V0B calibration closure, the
isolated serial oracle may run; only
`VEGETATION-BEAM-REF-P1 = PASS` permits an active R8 integration track.

Production promotion additionally requires tree/wind, fracture, rigid coupling,
exact persistence, LOD and Windows/Linux root gates plus a THOTH performance
profile fixed before measurement. Its production gate is 1,000 visible,
128 modal, 8 active, one refined and at most two falling trees, with an
incremental THOTH vegetation budget of 2/3 ms p95/p99 inside the existing
integrated physical 8/12 ms p95/p99 budget. If that workload misses its
declared budget after two evidence-backed optimization cycles, the
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

- SPEC-39, SPEC-40 and this ADR remain Proposed; current runtime/schema/save
  semantics and the PhysX baseline do not change.
- The next action is V0B numerical/profile/corpus calibration, not solver code.
- The physical-world model can add phenomena without shared mutable state or a
  universal solver.
- Vegetation and continuum programs remain independent until an explicit
  root/soil or water/vegetation exchange consumer exists.
