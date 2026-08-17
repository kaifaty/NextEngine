# SPEC-40: Proposed structural vegetation physics

| Field | Value |
|---|---|
| ID | SPEC-40 |
| Status | Proposed |
| Version | 1.3 |
| Last verified | 2026-08-17 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [SPEC-39](39-layered-physical-world.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-077](adr/077-layered-physical-world-and-living-structures-track.md), [ADR-081](adr/081-world-dynamics-gap-closure-and-promotion-guardrails.md) |
| Candidate revision note | Version 1.3 applies ADR-081 float-execution, collision-proxy, contact-load, staged-fracture, checkpoint-epoch, capacity, fault-domain and budget guardrails |
| Related Proposed tracks | [SPEC-43](43-thermochemical-material-processes.md), [SPEC-44](44-neural-assisted-world-simulation.md), [ADR-079](adr/079-thermochemical-material-process-track.md), [ADR-080](adr/080-neural-assistance-as-bounded-proposals.md) |

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

V0A product and architecture decisions are closed below. Exact graph/taper
bytes, synthetic material constants, field quantization scales, fixed internal
candidate cadences, fixture traces and reference-curve hashes remain V0B
calibration blockers. No structural solver code may start while those values
are implicit. Fire, moisture, decay, growth, foliage contact, snow/ice, root
failure, continuum-soil coupling, cross-region forest streaming and GPU
authority are not part of the first vertical.

### Selected V0A profile

The first profile is `Next Engine Reference Conifer V1`, an explicitly
synthetic engine test tree with no species-realism claim. It is a 10 m tree
with 12 major branches, no more than 128 structural segments and no more than
32 tapered-capsule collision proxies. Its root is a rigid clamp; uprooting is
excluded. One authored trunk felling zone uses exactly 8 radial rings by 32
angular sectors, or 256 section cells.

The outer structural cadence is fixed at 240 Hz. The V1 formulation bake-off
selects one fixed internal substep/iteration profile from the frozen corpus;
variable time step and result-dependent adaptivity are forbidden. Canonical
continuation state is fixed-point after every outer step; `f64` exists only
inside that step. A cut receives bounded canonical impulse, blade direction,
relative motion and grain coefficient through the production axe command path.
Derived remaining area, centroid, moments and directional stress/strength
govern failure, with at most one split per structure per substep.

The selected representation ladder is `AuthoredStatic -> ShaderWind ->
ModalStructural -> ActiveStructural -> RefinedSection`. The production forest
fixture contains 1,000 visible, 128 modal, 8 active, one refined and no more
than two simultaneously falling trees. The incremental vegetation CPU budget
on THOTH is a standalone 2/3 ms p95/p99 stop target. Integrated authority is
the future ADR-081 `world-dynamics-step` row, not an arithmetic share of the
existing 8/12 ms matrix. Exact memory and transition budgets remain V0B
blockers.

## Candidate authority

Physical Embodiment is the future owner of active living-structure state. CPU
`f64` execution under a closed ADR-081 `CanonicalFloatExecutionProfile` is the
only candidate authority for V1. The profile fixes target/toolchain features,
FMA, rounding/subnormal behavior, mathematical primitives, reduction/
factorization/tie order and convergence branches. The serial structural
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

After every accepted 240 Hz structural outer step, complete future-affecting
state is:

- structure/profile/topology revisions, tick/substep and canonical root;
- stable node ID, fixed-point position and orientation;
- fixed-point node linear and angular velocity;
- stable segment/section IDs and fixed-point strain/damage/history variables
  required by the selected formulation;
- exact cross-section cell state in every activated cut/fracture zone;
- root-anchor state and pending topology-transition disposition;
- active representation tier and its transition receipt.

V0B must assign exact units, integer widths/scales and bounds to every
field. Quaternion/rotation rules use a named SPEC-21 quantization profile,
canonical sign and checked ties-to-even conversion. The private solver may use
`f64` within a fixed outer step, but publication is the only continuation
boundary and the next outer step starts from accepted values. Factorizations,
residual scratch, broad-phase structures, modal basis caches and GPU/render
buffers are reconstructed and cannot survive as hidden authority.

Production additionally requires exact Windows/Linux canonical roots on
boundary and adversarial rounding/convergence fixtures. Failure after two
coherent remediation cycles leaves the solver research-only or requires a
separate fixed-point/soft-float authority decision.

Warm start, variable time step, adaptive element insertion/removal, fatigue,
plasticity/creep and arbitrary local fibre refinement are disabled in V1 unless
V0B explicitly adds a future-affecting field and a corpus for it.

Temperature, moisture, chemical composition, drying and combustion do not
become private structural fields by implication. A future profile uses the
SPEC-43 Thermochemical owner and an exact parcel-to-structure attachment;
strength/mass/topology consequences cross one atomic typed batch and require
`THERMOCHEM-VEGETATION-P1`. Fire presentation or an arcane effect cannot
substitute for that owner.

SPEC-44 neural assistance is also downstream of the complete classical tree
track. It may begin only as report/shadow diagnostics or proposals after
formulation, coupling and exact persistence pass; it cannot change production
work or select fracture, topology, representation tier, failure class or a
structural root. Runtime advice requires a later certificate-backed Accepted
decision under ADR-081.

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

V1 uses the SPEC-39 scenario-bound analytical wind projection. Each structural
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
direction and material profile to removed/crushed cells is frozen in V0B and
exercised through the same command path in game and headless.

The command references exactly one consumer-specific
`StructuralContactLoadBatch` record. Its stable contact/load identity can be
consumed for cut work at most once; retry or a second command finds the existing
consumption receipt. A canonical `CutWorkReceipt` allocates the bounded input
among rigid equal-and-opposite reaction, structural elastic/kinetic work,
section-cell damage/fracture and declared dissipation. The same impulse cannot
enter both a generic contact event and a separate cut debit.

The first cut representation is a polar cross-section lattice of exactly 8
radial rings by 32 angular sectors at one authored felling zone. Cells carry
closed states such as intact, crushed and severed plus only the history selected
by V0B. Remaining area, centroid and
second moments are derived in canonical cell order. Load capacity follows that
remaining geometry and directional material strength; a scalar accumulated HP
or a scripted `fell_now` threshold is forbidden.

Fracture crosses the existing Outcome/topology-command boundary:

1. evaluate section resultants and failure criterion from accepted state;
2. select the unique failing section by a complete `(criterion, SectionId)`
   total order fixed before implementation;
3. publish a bounded `PendingFracture` fact in the accepted structural
   candidate and propose one stage-9 internal Outcome command;
4. the validated Outcome command partitions the graph, derives the stable body
   identity and owns one `PhysicsTopologyTransaction`;
5. validate collision capacity, mass/centre-of-mass and momentum handoff;
6. atomically replace the pending component with the anchored graph plus one
   staged PhysX body, or publish neither; the body activates next substep.

While pending, the component cannot split again, downgrade, transfer or be
advanced by both owners. The structural solver never creates durable PhysX
topology in the same stage-8 step.

Only one load-bearing split per structure per substep is admitted in V1. Bark
tearing, fibres/splinters and secondary fragment clouds remain presentation.
More detailed local fibre refinement is a later model with its own exact state
and transition evidence; it cannot silently replace section cells.

## PhysX coupling and detached handoff

An active standing tree exposes no more than 32 tapered-capsule collision
proxies derived from its last accepted graph. Every active proxy maps to exactly
one stable PhysX kinematic body/shape pair for its lifetime; replacement uses a
validated topology/representation transition rather than backend-handle reuse.
During a substep PhysX integrates dynamic bodies exactly once against that
frozen projection and emits the consumer-specific
`StructuralContactLoadBatch`. The batch binds the ADR-081 exchange tuple,
quantized impulse and moment at an explicit reference point, one stable contact/
load ID and an equal-and-opposite reaction receipt. Generic contact-event bounds
are not interpreted as exact structural loads. The living solver consumes the
batch plus wind and advances once according to SPEC-39. Missing contacts, stale
revisions, a duplicate batch or capacity overflow rejects the whole composite
step.

At the first trunk sever, the selected detached component becomes one bounded
PhysX compound rigid body in V1. The handoff preserves declared mass, centre of
mass, linear momentum and angular momentum within V0B thresholds. After commit,
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

The PhysX-coupled production profile binds a fixed positive checkpoint epoch.
At every epoch it rebuilds a fresh PhysX scene from the composite canonical
closure, validates the continuation witness and swaps atomically. Save requests
wait for this scheduled barrier, and uninterrupted comparison runs execute the
same barriers.

## Failure and fallback

Worst-case node, segment, proxy, contact/load, pending-fracture and rigid-body
capacities are reserved before activation/action freeze. User-expressible
denial is ordinary; exhaustion beyond the admitted bound is an invariant fault.

Invalid definition/profile, nonfinite value, fixed-point overflow,
non-convergence, capacity excess beyond a reserved bound, stale identity/revision, contact or result
collision, impossible graph partition, conservation/handoff failure, PhysX
rejection or corrupt owner segment publishes no partial state. The prior
composite generation remains authoritative.

Before capability activation the authored scene uses a normal static rigid
tree/collider. After successful activation there is no silent switch to a
scripted fall, decorative damage, frozen structure, GPU authority or static
replacement. A fatal active failure stops the affected physical run and
retains the last complete checkpoint.

For the first primary gameplay world, that fatal failure faults the whole
application session through `Running -> Faulted -> DiagnosticSaved -> Closed |
ExplicitRestore`. Only independently provisioned test/training scenes may use
narrower isolation.

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
| conditional `performance` | The selected V0A active/modal/visible forest fixture meets its standalone THOTH stop target; before integration the full combined workload passes the successor mutually exclusive `world-dynamics-step` row across every substep in one gameplay tick, or the track remains research-only. |

The frozen accuracy thresholds are: static curve error at most 2%, natural
frequency error at most 5%, aggregate normalized RMSE at most 5%, maximum
curve error at most 10%, external-work-aware work/impulse residual at most 1%,
exact mass, detached handoff CoM error at most 1 mm and handoff momentum
residual at most 1%. Exact collision-penetration and LOD-transition thresholds
remain V0B blockers.

Same-target exactness is required for V1 reference work; Windows/Linux
canonical root equality is additionally required before production promotion.
Exact active save/resume is required before any lossy modal/sleep persistence.
Heavy
external trajectories, scans, captures and measurements remain outside Git.

No public contract is added for V0/V1. With the production tree consumer, the
smallest candidate set is a structure definition/profile, canonical state,
rigid contact-load batch, topology/handoff record, presentation snapshot and a
successor composite checkpoint. Exact names and versions are assigned only by
the consumer-backed Accepted promotion ADR. Generic solver API, raw graph/cell
mutation, arbitrary plugin fracture callback and gameplay access to solver
nodes remain out of scope.
