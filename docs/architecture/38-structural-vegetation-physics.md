# SPEC-38: Proposed structural vegetation physics

| Field | Value |
|---|---|
| ID | SPEC-38 |
| Status | Proposed |
| Version | 1.0 |
| Last verified | 2026-08-16 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [SPEC-37](37-layered-physical-world.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-073](adr/073-layered-physical-world-and-living-structures-track.md) |
| Supersedes | none; defines a Proposed living-structures track without changing current runtime, PhysX, content or save contracts |

## Status and selected scope

This SPEC defines the candidate authority, representation, coupling, topology,
persistence and promotion gates for destructible trees. It does not claim a
solver, vegetation runtime, content schema, save version or ProductCheck PASS.
The execution series is [the vegetation physics roadmap](../plans/vegetation-physics/README.md).

The first player-visible vertical is one pinned-active procedural tree beside a
trail. It sways under an immutable wind profile; the player uses the production
interaction/command path to create a directional notch and back cut; the
remaining hinge fails from computed section loads; the detached component is
handed to PhysX and can block the trail. Debug rods, section cells, stress and
contact overlays are sufficient presentation.

The exact tree geometry, calibrated wood profile, solver formulation,
discretization, cut-work mapping, collision capacity and numerical/physical
curve thresholds remain Package V0 blockers. No structural solver code may
start while those values are implicit. Fire, moisture, decay, growth, foliage
contact, snow/ice, root failure, continuum-soil coupling, forest streaming and
GPU authority are not part of the first vertical.

## Candidate authority

Physical Embodiment is the future owner of active living-structure state. CPU
`f64` execution is the only candidate authority for V1. The serial structural
reference selects and freezes one geometrically exact beam/rod formulation
before runtime integration. A GPU implementation may later mirror aggregate
curves but cannot write structure, damage, topology, rigid impulses, commands,
events or checkpoints.

PhysX remains the only writer of rigid/articulated transforms, velocities and
contacts. The living solver writes only the rooted structural graph, its
elastic/damage variables and topology. Foliage animation, bark/cut meshes,
splinters, dust, leaves and fire VFX are presentation and cannot affect a
structural root.

## Structural definition and graph

One definition is a bounded rooted directed tree, not an arbitrary cyclic
graph. Stable `StructureNodeId`, `StructureSegmentId`, `SectionId` and
`CollisionProxyId` values are scoped by one durable structure identity and
never derived from vector index, asset order or backend handle.

Each node declares rest position/orientation, parent segment, junction role and
root-anchor relation. Each segment declares endpoints, tapered cross-section,
rest directors/curvature, mass distribution, material-profile reference,
collision-proxy mapping and optional felling-section mapping. Canonical records
are sorted by stable ID; every reference, bound, positive mass/inertia and graph
closure is validated before activation.

The physics graph is sparse and drives the visual mesh. The visual mesh is not
integrated as structural state. Small twigs and leaves may contribute bounded
mass/drag to an authored segment or foliage cluster without acquiring hidden
mechanical degrees of freedom.

`StructuralWoodProfile` is separate from `PhysicsMaterialDescriptorV2`. The
former owns density, longitudinal/radial/tangential elastic and shear response,
damping, directional strength/fracture and declared moisture reference. The
latter remains rigid contact material. An exact coupling-material map selects
friction/restitution/contact roles for structural collision proxies.

## Canonical active state candidate

After every accepted structural substep, complete future-affecting state is:

- structure/profile/topology revisions, tick/substep and canonical root;
- stable node ID, fixed-point position and orientation;
- fixed-point node linear and angular velocity;
- stable segment/section IDs and fixed-point strain/damage/history variables
  required by the selected formulation;
- exact cross-section cell state in every activated cut/fracture zone;
- root-anchor state and pending topology-transition disposition;
- active representation tier and its transition receipt.

Package V0 must assign exact units, integer widths/scales and bounds to every
field. Quaternion/rotation rules use a named SPEC-21 quantization profile,
canonical sign and checked ties-to-even conversion. The private solver may use
`f64` within a fixed substep, but publication is the only continuation boundary
and the next substep starts from accepted values. Factorizations, residual
scratch, broad-phase structures, modal basis caches and GPU/render buffers are
reconstructed and cannot survive as hidden authority.

Warm start, variable time step, adaptive element insertion/removal, fatigue,
plasticity/creep and arbitrary local fibre refinement are disabled in V1 unless
Package V0 explicitly adds a future-affecting field and a corpus for it.

## Formulation selection gate

The source research recommends Cosserat rods, but that label is insufficient
to implement a stiff branched tree at 240 Hz. Package V1 compares at least:

- a shearable/extensible geometrically exact Cosserat/Timoshenko rod; and
- a constrained or implicit discrete-rod/corotational beam baseline that does
  not require unstable explicit integration of wood's axial wave modes.

Both use the same analytical cantilever, torsion, buckling, tapered-beam and
branched-junction corpus. V1 freezes the smaller formulation/integrator/state
that satisfies the predeclared curves and fixed-step stability bounds. An
external PyElastica revision is aggregate evidence for Cosserat cases, not a
linked engine dependency or authority. No runtime tree graph begins before
`VEGETATION-BEAM-REF-P1 = PASS`.

## Wind and aerodynamic load

V1 uses the SPEC-37 scenario-bound analytical wind projection. Each structural
segment or foliage cluster has authored bounded drag area and coefficient; the
solver evaluates force from canonical relative wind at declared sample points.
Full atmospheric CFD, tree-to-wind back reaction and presentation leaf motion
are absent. Gravity, wind and prescribed pull fixtures record external work so
that energy/momentum residuals are meaningful rather than naive conservation
claims.

## Cutting, damage and fracture

There is no tree HP. A validated cut command binds the production interaction,
tool/body/contact identity, target felling zone and expected structure/body
revisions. It contributes bounded fixed-point cut work to an exact section-cell
update. The mapping from tool geometry, relative velocity/impulse, grain
direction and material profile to removed/crushed cells is frozen in Package
V0 and exercised through the same command path in game and headless.

The first cut representation is a bounded polar cross-section lattice at one
authored felling zone. Cells carry closed states such as intact, crushed and
severed plus only the history selected by V0. Remaining area, centroid and
second moments are derived in canonical cell order. Load capacity follows that
remaining geometry and directional material strength; a scalar accumulated HP
or a scripted `fell_now` threshold is forbidden.

Fracture is a staged topology transaction:

1. evaluate section resultants and failure criterion from accepted state;
2. select the unique failing section by a complete `(criterion, SectionId)`
   total order fixed before implementation;
3. partition the rooted graph into anchored and detached components;
4. derive stable identity mapping, mass/centre-of-mass and momentum handoff;
5. validate collision capacity and both owner candidates;
6. atomically publish graph topology plus PhysX body creation, or neither.

Only one load-bearing split per structure per substep is admitted in V1. Bark
tearing, fibres/splinters and secondary fragment clouds remain presentation.
More detailed local fibre refinement is a later model with its own exact state
and transition evidence; it cannot silently replace section cells.

## PhysX coupling and detached handoff

An active standing tree exposes bounded tapered capsule/convex collision
proxies derived from its last accepted graph. During a substep PhysX integrates
dynamic bodies exactly once against that frozen projection and emits canonical
contact loads. The living solver consumes the batch plus wind and advances once
according to SPEC-37. Missing contacts, stale revisions, a duplicate batch or a
capacity overflow rejects the whole composite step.

At the first trunk sever, the selected detached component becomes one bounded
PhysX compound rigid body in V1. The handoff preserves declared mass, centre of
mass, linear momentum and angular momentum within V0 thresholds. After commit,
PhysX alone advances that component. Internal flexible falling-tree dynamics,
tree-to-tree fracture and re-fracturing detached bodies are later profiles.

## LOD and persistence

The first vertical pins one full structural tree active. Production forest LOD
is a later exact representation ladder:

```text
AuthoredStatic -> ShaderWind -> ModalStructural -> ActiveStructural -> RefinedSection
```

Transitions use canonical facts and `Prepare -> Validate -> Commit ->
Stabilize`; renderer visibility and measured frame time are forbidden inputs.
Damage, pending cut/fracture, contact, fall or unstable support blocks a lossy
downgrade. V1 promotion requires repeated adjacent-tier cycles with bounded
pose/velocity/energy/damage error and no lost durable outcome.

Exact active persistence precedes modal/sleep persistence. The structure owner
segment stores the full canonical graph, section state, topology, tier and
transition disposition in the same composite physical checkpoint as required
PhysX state. Save-at-N/resume-to-M must match uninterrupted roots exactly.
Modal/sleep conversion is later approximate persistence with a distinct root
and receipt; a sidecar or reconstruction of damage from immutable content is
forbidden.

## Failure and fallback

Invalid definition/profile, nonfinite value, fixed-point overflow,
non-convergence, capacity excess, stale identity/revision, contact or result
collision, impossible graph partition, conservation/handoff failure, PhysX
rejection or corrupt owner segment publishes no partial state. The prior
composite generation remains authoritative.

Before capability activation the authored scene uses a normal static rigid
tree/collider. After successful activation there is no silent switch to a
scripted fall, decorative damage, frozen structure, GPU authority or static
replacement. A fatal active failure stops the affected physical run and
retains the last complete checkpoint.

## Evidence and promotion boundary

| Check | Required result |
|---|---|
| `VEGETATION-BEAM-REF-P1` | Selected formulation matches frozen static/dynamic beam, torsion, buckling, taper and junction curves; fixed cadence is stable; repeats and insertion permutations have the same target-local root. |
| `VEGETATION-TREE-P1` | One graph passes gravity sag, pull/release, steady wind/gust and modal/active comparison thresholds without joint artifacts or hidden state. |
| `VEGETATION-FRACTURE-P1` | Progressive section-cell cuts, notch/back-cut, asymmetric hinge and invalid/stale/capacity cases produce the frozen failure curves and atomic topology result. |
| `VEGETATION-COUPLING-P1` | Rigid contact load and detached-body handoff preserve declared impulses/mass/CoM/momentum; PhysX remains the only rigid writer. |
| `VEGETATION-PERSISTENCE-P1` | Exact active save/restart equals uninterrupted execution; corrupt/incompatible segments fail before mutation. |
| `VEGETATION-LOD-P1` | Adjacent tier cycles meet frozen discontinuity/history bounds; camera/timing/worker permutations do not change tiers or roots. |
| `VEGETATION-MIRROR-P1` | Optional GPU aggregate correspondence passes without an authority claim. |
| conditional `performance` | The V0 active/modal/visible forest fixture fits the existing THOTH integrated budget or the track remains research-only. |

Same-target exactness is required for reference work; Windows/Linux canonical
root equality is additionally required before production promotion. Heavy
external trajectories, scans, captures and measurements remain outside Git.

No public contract is added for V0/V1. With the production tree consumer, the
smallest candidate set is a structure definition/profile, canonical state,
rigid contact-load batch, topology/handoff record, presentation snapshot and a
successor composite checkpoint. Exact names and versions are assigned only by
the consumer-backed Accepted promotion ADR. Generic solver API, raw graph/cell
mutation, arbitrary plugin fracture callback and gameplay access to solver
nodes remain out of scope.

