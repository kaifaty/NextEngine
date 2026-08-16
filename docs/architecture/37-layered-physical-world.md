# SPEC-37: Proposed layered physical-world model

| Field | Value |
|---|---|
| ID | SPEC-37 |
| Status | Proposed |
| Version | 1.0 |
| Last verified | 2026-08-16 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [SPEC-36](36-continuum-material-physics.md), [ADR-027](adr/027-physics-motor-and-animation-layering.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-073](adr/073-layered-physical-world-and-living-structures-track.md) |
| Supersedes | none; adds a Proposed composition model without changing current PhysX, runtime, save or public-contract semantics |

## Status and purpose

This SPEC defines how future physical phenomena compose without creating a
single universal solver or several writers of the same state. It is a
candidate architecture for post-v1 work. The current production world remains
the Accepted PhysX path from SPEC-26/ADR-058; SPEC-36 continuum state and the
SPEC-38 living-structure state remain Proposed and unimplemented.

The word *layer* means an ownership and transaction boundary. It does not mean
that every layer depends on the one drawn immediately below it, that every
project instantiates every layer, or that an upper layer may mutate a lower
one. The physical world is a fixed-stage DAG of peer state owners connected by
immutable projections and canonical exchange batches.

## Layer map

| Layer | Role | Authority rule |
|---|---|---|
| L0 Activation and profile closure | Exact content, coordinate, numeric, cadence, capacity and capability identities | Immutable input; missing required closure fails before world activation |
| L1 Canonical schedule and identity | Runtime tick/substep, stable IDs, revisions, ordering, fixed-point publication and roots | Runtime/SPEC-21 chooses boundaries; wall time, worker or GPU completion never chooses a result |
| L2 Environmental forcing projections | Gravity and future bounded wind, temperature, precipitation or field samples for one step | Revision-bound immutable inputs; a forcing source cannot write a simulated owner state |
| L3 Physical state owners | Rigid/articulated bodies, continuum regions, living structures and later thermal/chemical state | Exactly one writer per field and representation; solver-private caches are reconstructible |
| L4 Coupling and composite commit | Frozen projections, canonical load/reaction/flux batches and one all-or-nothing `PhysicalStep` | No peer mutation, arrival-order winner, duplicate writer or partial publication |
| L5 Physical outcomes and persistence | Committed contact/topology facts, gameplay proposals and composite owner checkpoints | Outcomes are derived only after physical commit; all required owner segments restore together |
| L6 Presentation | Mesh/pose/particles/foliage/VFX extraction | Read-only projection; camera, renderer cadence and device state never feed authority |

L3 is deliberately plural. PhysX remains the writer of rigid and articulated
pose/velocity/contact state. A continuum solver writes only its material
state. A living-structure solver writes only its graph, elastic/damage and
topology state. A later heat/moisture/combustion owner writes only its declared
energy/material variables. Similar-looking quantities do not permit shared
ownership.

## First forcing boundary

The first living-structure consumer does not create a weather subsystem. Its
wind is an immutable analytical forcing profile bound to the scenario:

- right-handed MKS and the same gravity/cadence identity as the physical step;
- stable field/profile revision and exact evaluation tick/substep;
- bounded mean vector plus a closed deterministic gust function;
- fixed-point coefficients and output quantization before the structural
  solver consumes the sample;
- no camera, renderer, wall-clock or process RNG input.

A later World Services weather consumer may own semantic weather and compile a
revision-bound physical forcing projection. That requires its own consumer and
cannot silently replace the first profile or write structural state directly.

## Owner and exchange rules

Every L3 owner declares:

1. one immutable definition/profile closure;
2. complete future-affecting canonical state and root;
3. solver-private reconstructible caches;
4. accepted inbound projection/batch types and exact expected revisions;
5. one canonical outbound batch per declared key;
6. finite capacities and deterministic failure codes;
7. persistence classification and presentation extraction.

Exchange records use full stable participant identity, source/destination
owner and state revisions, world generation, tick/substep, profile and prior
state roots, fixed-point quantities, units and explicit reference points for
moments. Records have a complete total order. An exact duplicate or a different
hash under the same unique batch key rejects the uncommitted composite step;
the runtime never selects whichever result arrived first.

Broad `PhysicalLayer`, generic solver, arbitrary field map or plugin-owned raw
particle/node API is not a V1 public contract. Exact exchange schemas are
introduced only with their first production consumer under ADR-046.

## Candidate `PhysicalStep` DAG

For a selected set of active owners, one substep is:

1. bind L0/L1 identities and freeze every prior owner projection;
2. evaluate declared L2 forcing projections from canonical inputs;
3. run each independent owner candidate, or the fixed dependency order of a
   declared coupling profile;
4. canonicalize and validate all exchange batches before destination use;
5. integrate each owned representation exactly once under that profile;
6. validate all candidate roots, conservation/error receipts and capacities;
7. publish all participating owner states plus exchange receipts atomically;
8. derive gameplay proposals and L6 presentation from the committed result.

The first rigid/living coupling profile specializes step 3: PhysX integrates
dynamic bodies once against the frozen structural collision projection and
emits canonical contact loads; the living solver consumes those loads plus
wind and advances its own graph once. PhysX already owns the reaction applied
to dynamic bodies; the structural solver never rewrites their pose or velocity.
The structural collision projection is updated only from the accepted graph
for the next substep. This intentional one-substep stagger is profile identity
and must pass the coupling corpus; it cannot be replaced by an implicit loop
selected by timing.

Detached structural components cross an explicit topology/ownership
transaction. After a successful handoff, PhysX owns the new rigid bodies and
the living owner retains only the anchored/remnant graph plus the transition
receipt. Both owners cannot simulate the same detached mass.

## Topology, LOD and streaming

Topology and representation changes use `Prepare -> Validate -> Commit ->
Stabilize`. A failed change leaves the source representation authoritative.
Mass, centre of mass, linear/angular momentum, stable identity mapping and
durable outcome have profile-owned checks at the transition boundary.

Simulation LOD may use only canonical facts such as declared interaction,
damage/instability, world residency, quantized simulation distance, stable ID
and manifest integer budget tokens. Camera/frustum/occlusion, measured CPU/GPU
time, wall clock and worker completion order are forbidden inputs.

The first vegetation vertical is pinned active in one loaded region. Forest
sleep, modal conversion, cross-region transfer and eviction are later explicit
representation transitions. A dirty owner cannot be evicted until its exact or
declared approximate durable representation is published atomically.

## Persistence

Each active owner stores complete future-affecting state in an owner segment.
A restorable physical checkpoint is a composite closure containing the exact
owner/profile/revision roots and every exchange/topology receipt needed for
continuation. Missing, stale, corrupt or independently committed sidecar state
fails before target-world publication.

The first consumer that adds a production owner creates the next current-only
composite checkpoint version. Later consumers create successors rather than
assuming that a speculative version number or absent segment can be filled by
defaults. Solver caches, spatial acceleration, modal basis caches, render
meshes and GPU buffers are rebuilt and remain outside authority.

## Failure and fallback

Invalid profile/content, nonfinite value, overflow, non-convergence, stale
revision, batch collision, missing participant, capacity excess, coupling or
backend rejection, topology-conservation failure and corrupt persistence reject
the complete participating `PhysicalStep`. The prior committed generation
remains the only authority.

A capability may select a separately authored fallback only before activation.
After activation there is no silent decorative substitution, owner freeze while
peers advance, mid-run backend switch, partial result or retry-to-green.

## Promotion checks

This model has no standalone implementation check. Each concrete owner must
pass its own reference, coupling, persistence, cross-target and conditional
performance gates. A future composition check `PHYSICAL-LAYERS-P1` is created
only when at least two non-rigid production owners exist; it must then prove
writer exclusivity, batch collision rejection, atomic multi-owner failure and
presentation independence. Until then, water and vegetation checks remain
independent and neither proves the other.

