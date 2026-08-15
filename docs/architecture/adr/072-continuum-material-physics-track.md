# ADR-072: Continuum material physics track

| Field | Value |
|---|---|
| ID | ADR-072 |
| Status | Proposed |
| Version | 1.0 |
| Decision date | 2026-08-16 |
| Last verified | 2026-08-16 |
| Normative dependencies | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-23](../23-jobs-memory-resource-residency-and-io-backpressure.md), [SPEC-25](../25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](../30-presentation-extraction-and-render-content.md), [ADR-027](027-physics-motor-and-animation-layering.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-071](071-canonical-physics-material-lineage.md) |
| Supersedes | none |
| Superseded by | none |

## Context

Next Engine needs local water, deformable ground, sand, mud and snow that can
interact with characters and vehicles without making renderer state, a global
voxel volume or an optional GPU path authoritative. The current physics
baseline owns rigid bodies and articulations through PhysX 5.9.0 and exposes
only exact engine-owned snapshots. Its `PhysicsMaterialDescriptorV2` describes
solid contact coefficients; it is not a constitutive law for continuum matter.

The research brief proposed one broad particle material world. External
evidence supports Lagrangian material samples, but not one SPH discretization
for every material. DFSPH is a strong incompressible free-surface water
baseline, while APIC/MLS-MPM is better aligned with history-dependent
elastoplastic sand, snow and deformable terrain. GPU floating-point execution
cannot be assumed bit-exact across the shipping targets.

ADR-046 also forbids accepting speculative public schemas before a production
consumer demonstrates them. This decision therefore defines a Proposed track,
not a shipped backend or current wire format.

## Proposed decision

### One ownership boundary, multiple numerical lanes

Physical Embodiment SHOULD eventually own active continuum state alongside the
current rigid/articulated world. Durable material identity and constitutive
history live on bounded Lagrangian samples or region records. Spatial hashes,
neighbor lists, temporary grids, pressure systems, surface meshes, SDFs and GPU
buffers are reconstructible compute or presentation structures.

The first numerical lanes are:

1. fixed-resolution CPU `f64` DFSPH as the water reference and oracle;
2. GPU DFSPH as a non-authoritative correspondence mirror until promoted by a
   later Accepted decision and cross-target evidence;
3. APIC/MLS-MPM with an elastoplastic constitutive model for dry sand, snow and
   deformable soil;
4. saturation-dependent soil/water coupling only after the dry terrain lane
   passes its conservation and product checks.

This is a family of solvers behind one engine-owned owner boundary, not a
single generic mega-solver or a public vendor-neutral plugin ABI.

### Rigid coupling and authority

PhysX remains the only current production writer of rigid-body and articulation
state. A continuum step MAY produce one bounded, canonically ordered impulse
batch for a declared physical substep. The PhysX step validates and consumes
that batch; no continuum backend writes PhysX transforms or maintains a second
rigid-body authority. Reaction state enters the next coupled substep or an
explicit bounded coupling iteration.

Promotion to production MUST resolve the current ADR-058 wording with a new
Accepted ADR: PhysX can remain the sole rigid/articulation backend while a
separate continuum solver becomes an owned Physical Embodiment component.

### Materials, persistence and presentation

`PhysicsMaterialDescriptorV2` MUST remain the solid-contact descriptor.
Continuum constitutive profiles require a separate consumer-backed schema and
cannot be added before the first runtime-bearing consumer.

Active-to-sleep conversion is an explicit lossy model transition, not a
claimed lossless particle round trip. Production persistence requires bounded
mass/momentum/volume and constitutive-history error, a versioned owner segment,
and save/replay parity. Until those checks pass, a modified region remains
active or persistent deformation is disabled.

Presentation consumes immutable extracted samples/fields. Surface meshing,
foam, spray, wetness masks and terrain tessellation are reconstructible and
cannot feed gameplay queries or state roots.

### Determinism and fallback

The CPU reference uses stable sample IDs, fixed cadence, canonical neighbor and
reduction order, finite profiles and explicit failure codes. GPU kernels MAY
use faster order-dependent reductions only as mirrors. Quantizing their final
output does not prove deterministic equivalence.

Every production profile requires a deterministic bounded fallback: static
water/solid ground, an already accepted previous region generation, or feature
unavailability before activation. Silent solver substitution is forbidden.

## Product impact

The track aims at player-visible local water and deformable terrain while
preserving offline correctness and the current v1 critical path. It is an
optional post-v1 program and does not change the current physics backend,
save/replay formats, content schemas or public contracts.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `CONTINUUM-WATER-REF-P1` | Bounded dam break, hydrostatic column and moving-boundary cases on the CPU reference | Conservation, density/divergence, repeatability and failure bounds pass | Keep the track offline and use static water/solid collision |
| `CONTINUUM-COUPLING-P1` | Water impulse against a dynamic PhysX body | Canonical impulse batch conserves bounded momentum and never creates a second body writer | Disable two-way coupling |
| `CONTINUUM-MIRROR-P1` | CPU/GPU water trajectory corpus | Declared per-field correspondence thresholds pass without an authority claim | CPU reference remains the oracle |
| `CONTINUUM-TERRAIN-P1` | Tire/foot shear and dry-sand collapse | Terrain deformation and force curves pass conservation and regression checks | Use accepted rigid terrain |
| `CONTINUUM-PERSISTENCE-P1` | Modify region, save, unload, restart and reload | Same owner root and bounded wake transition reproduce | Pin region active or reject persistent deformation |
| conditional `performance` | One declared active region on the target profile | Budgets pass without changing authoritative outcomes | Reduce declared quality before world creation or disable the optional feature |

## Considered alternatives

- One SPH formulation for water, sand, mud and snow — rejected because solid
  plasticity and constitutive history fit MPM-style transfers better.
- Permanent Eulerian voxel/heightfield authority — rejected as the universal
  representation; allowed as a bounded sleep or presentation approximation.
- GPU-first canonical solver — rejected until device/compiler/profile closure
  and cross-target evidence exist.
- Extend `PhysicsMaterialDescriptorV2` — rejected because surface contact and
  continuum constitutive state have different ownership and lifecycle.
- Accept all proposed contracts now — rejected by ADR-046; schemas follow the
  first production consumer.
- Neural or 2026 variational methods as the baseline — rejected for the first
  implementation; retained as isolated research comparators.

## Consequences

- [SPEC-36](../36-continuum-material-physics.md) and the implementation-spec
  series remain Proposed until a concrete product scenario is selected.
- First implementation is a CPU water lab, not a general runtime subsystem.
- Dry terrain follows water coupling; wet mud follows dry terrain.
- A production promotion must update SPEC-26, persistence, presentation,
  routing, traceability and the roadmap in one coherent change.
- No current crate, schema, ProductCheck result or roadmap blocker is changed.
