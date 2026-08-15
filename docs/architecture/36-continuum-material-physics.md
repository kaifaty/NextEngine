# SPEC-36: Proposed continuum material physics

| Field | Value |
|---|---|
| ID | SPEC-36 |
| Status | Proposed |
| Version | 1.0 |
| Last verified | 2026-08-16 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-23](23-jobs-memory-resource-residency-and-io-backpressure.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-072](adr/072-continuum-material-physics-track.md) |
| Supersedes | none |

## Status and scope

This SPEC defines candidate semantics and promotion gates for local continuum
materials. It does not authorize runtime schemas, change the PhysX-only current
baseline or claim that water, mud, soil, snow or off-road coupling is shipped.
The execution sequence is maintained in
[the implementation-spec series](../plans/continuum-material-physics/README.md).

In scope: bounded local free-surface water, rigid coupling, deformable dry
terrain, wet-soil evolution, active-region persistence and read-only visual
extraction. Out of scope: global ocean/weather, arbitrary multiphase chemistry,
erosion of the entire world, ocean-scale waves, network synchronization and a
generic community solver ABI.

## Candidate authority model

`ContinuumRegionState` is the future Physical Embodiment-owned aggregate. One
region has a stable identity, material/profile revisions, simulation cadence,
active/sleep state, conservation totals and a closed representation variant.
The exact schema is deliberately deferred until the first consumer.

The active representation stores stable material samples and material-specific
state. Water needs position, velocity, mass, density/pressure state; an MPM
solid additionally needs deformation gradient and plastic/internal variables.
One universal per-particle struct is forbidden. Implementations use bounded
material-specific SoA storage and stable sample keys.

Spatial hashes, neighbor pairs, MPM grids, pressure matrices, surface meshes,
wetness textures and GPU buffers are caches. They are rebuilt from owner state
and exact profile identity.

## Fixed-stage step candidate

For each declared continuum substep:

1. freeze the prior continuum and rigid projection;
2. construct canonical active-region and stable sample order;
3. build reconstructible neighbor/grid structures;
4. solve internal material forces and boundary constraints;
5. reduce boundary reactions into one bounded canonical impulse batch;
6. validate finite values, bounds, conservation and profile identity;
7. let the Physical/PhysX commit consume the batch and publish the complete
   next physical generation, or publish nothing;
8. extract an immutable presentation projection after commit.

An async/GPU worker receives immutable revision-bound inputs and returns an
immutable result. Completion timing cannot select the simulation tick. A
production asynchronous path must reuse SPEC-21 completion admission and can
only promote reusable job/resource contracts when a second demonstrated
consumer satisfies SPEC-23.

## Solver lanes

### Water reference

The reference uses fixed-resolution CPU `f64` DFSPH with a compact-support
kernel, explicit density and divergence tolerances, fixed maximum iterations,
canonical neighbor order and analytical or sampled boundaries. Failure to
converge returns a typed whole-step failure; it does not publish a partial
state or silently increase the time step.

### Accelerated water

The first GPU implementation is a mirror of the reference scenario corpus. It
records device, driver, compiler, shader/kernel, workgroup, reduction and
profile identity. It cannot emit authoritative commands/events or replace the
CPU oracle. Promotion needs a later Accepted decision choosing exact replayed
results, a device-closed authoritative profile, or a reduced deterministic
gameplay model.

### Deformable solids and soils

Dry sand/snow/soil starts with APIC/MLS-MPM and an explicit constitutive law,
initially Drucker-Prager for sand. The temporary grid transfers momentum and
computes stress while material identity/history stay on samples. MCC or
rate-dependent alternatives are comparative spikes, not default profiles.

Wet mud starts only after dry terrain. Its first bounded model MAY couple a
saturation field to permeability, cohesion/yield and drag. A true two-phase
poromechanics implementation is selected only if the simplified profile fails
declared drainage, shear and wheel/foot scenarios.

## Boundaries and coupling

Analytical SDF boundaries are preferred for simple shapes. Curved or authored
geometry may use a validated density-map/MLS boundary projection. Sampling
resolution, normals, transforms and surface-contact material lineage are exact
profile inputs.

Rigid reaction is expressed as force/impulse and torque on stable body IDs.
The reduction order is fixed. PhysX owns final body/articulation integration;
the continuum solver owns only its region state. Terrain contact exposes an
engine-owned patch query/result, never raw particles, MPM nodes or PhysX types.

## Persistence and streaming

The future owner segment stores exact active state or a separately versioned
sleep representation plus profile identity and conservation totals. It never
stores neighbor lists, grid nodes or render meshes.

Sleep conversion is admitted only at a fixed commit boundary. The transition
must report mass, linear/angular momentum, volume and material-history error.
If the profile cannot satisfy the declared threshold, the region stays active
or the transition fails. Streaming cannot discard dirty authoritative material
state, use wall time for eviction or reconstruct changed terrain from immutable
content alone.

## Presentation

Presentation extraction publishes bounded stable sample/field records after
the physical commit. Render backends may reconstruct an anisotropic surface,
screen-space fluid, spray/foam, terrain mesh and wetness decals. Missing GPU
capability selects a declared visual fallback only; collision, buoyancy,
terrain resistance and gameplay queries continue to use committed physical
state.

## Promotion sequence

1. CPU water reference lab and golden corpus.
2. Boundary and one rigid-body coupling vertical.
3. Read-only water presentation.
4. GPU correspondence mirror.
5. Dry deformable-terrain lab and tire/foot patch vertical.
6. Active-region save/unload/reload.
7. Saturation/wet-mud model.
8. Production consumer proposal with exact contracts, fallback and budgets.

No step promotes a later step by implication. Adaptivity follows a stable
fixed-resolution solver and must prove conservation across split/merge.

## Failure semantics

Invalid content/profile, nonfinite value, capacity overflow, non-convergence,
stale revision, result collision, conservation violation or corrupt owner
segment rejects the complete candidate. The previous physical generation is
retained. Optional activation may fall back only to the profile-declared static
water/rigid-ground representation; an active authoritative region cannot be
silently replaced mid-run.

## Product checks before promotion

- water: hydrostatics, dam break, free fall, moving wall and rigid float/impact;
- conservation: mass, linear/angular momentum and bounded energy drift;
- determinism: repeat, worker-count, insertion-order and save/restart cases;
- terrain: angle of repose, column collapse, shear box, sinkage and tire/foot
  force/deformation curves;
- wet soil: infiltration, drainage, saturation-dependent shear and hysteresis;
- boundaries: leak, tunneling, thin feature and fast-body cases;
- presentation: cadence/device loss changes no authoritative root;
- security: all sizes, iterations, material parameters and decoded state are
  bounded before allocation;
- performance: one declared active region on named Windows/Linux profiles.

Until a production consumer exists, these are specification targets and their
status is `NOT_RUN`, not evidence of implementation.
