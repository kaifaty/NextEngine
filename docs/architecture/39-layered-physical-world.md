# SPEC-39: Proposed layered physical-world model

| Field | Value |
|---|---|
| ID | SPEC-39 |
| Status | Proposed |
| Version | 1.4 |
| Last verified | 2026-08-17 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [SPEC-38](38-continuum-material-physics.md), [ADR-027](adr/027-physics-motor-and-animation-layering.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-077](adr/077-layered-physical-world-and-living-structures-track.md), [ADR-081](adr/081-world-dynamics-gap-closure-and-promotion-guardrails.md) |
| Related Proposed composition | [SPEC-41](41-world-substrate-composition.md), [SPEC-42](42-arcane-substrate-and-physical-magic.md), [SPEC-43](43-thermochemical-material-processes.md), [SPEC-44](44-neural-assisted-world-simulation.md), [ADR-078](adr/078-world-substrate-and-arcane-physical-interaction-track.md), [ADR-079](adr/079-thermochemical-material-process-track.md), [ADR-080](adr/080-neural-assistance-as-bounded-proposals.md) |
| Candidate revision note | Version 1.4 applies ADR-081 checkpoint-epoch, float-execution, composition, fault-domain, capacity and shadow-only neural guardrails without changing current PhysX, runtime, save or public-contract semantics |

## Status and purpose

This SPEC defines how future physical phenomena compose without creating a
single universal solver or several writers of the same state. It is a
candidate architecture for post-v1 work. The current production world remains
the Accepted PhysX path from SPEC-26/ADR-058; SPEC-38 continuum state and the
SPEC-40 living-structure state remain Proposed and unimplemented.

The word *layer* means an ownership and transaction boundary. It does not mean
that every layer depends on the one drawn immediately below it, that every
project instantiates every layer, or that an upper layer may mutate a lower
one. The physical world is a fixed-stage DAG of peer state owners connected by
immutable projections and canonical exchange batches.

## Relationship to world substrates

Physical Embodiment is one specialization inside the Proposed SPEC-41
world-substrate composition, not the owner of every simulated law. A future
Arcane and Thermochemical owners from SPEC-42/43 remain peer substrates outside
Physical Embodiment. Arcane can affect rigid, continuum, vegetation or
Thermochemical state only through a typed cross-owner exchange profile;
Thermochemical phase/reaction consequences use the same rule. Neither becomes
an L2 forcing shortcut or a physical-state writer.

An Arcane start command commits its own stage-5 execution-start receipt and
does not remain pending through physics. Each later arcane/physical substep
freezes both owner revisions, stages one canonical exchange batch and publishes
the Arcane debit plus participating physical state and exchange receipt
atomically at the successor `WorldDynamicsStep`. Completion facts use the one existing stage-9
Outcome boundary. The physical part still follows this SPEC's exclusive-writer
rules. Projects that do not activate the arcane capability retain the unchanged
current physical path and do not create empty Arcane owner segments.

## Layer map

| Layer | Role | Authority rule |
|---|---|---|
| L0 Activation and profile closure | Exact content, coordinate, numeric, cadence, capacity and capability identities | Immutable input; missing required closure fails before world activation |
| L1 Canonical schedule and identity | Runtime tick/substep, stable IDs, revisions, ordering, fixed-point publication and roots | Runtime/SPEC-21 chooses boundaries; wall time, worker or GPU completion never chooses a result |
| L2 Environmental forcing projections | Gravity and future bounded wind, temperature, precipitation or field samples for one step | Revision-bound immutable inputs; a forcing source cannot write a simulated owner state |
| L3 Peer state owners | Physical rigid/articulated bodies, continuum regions and living structures; adjacent Thermochemical material parcels | Exactly one writer per field and representation; solver-private caches are reconstructible |
| L4 Coupling and composite commit | Frozen projections, canonical load/reaction/flux batches and one all-or-nothing stage-8 transaction (`PhysicalStep` or successor `WorldDynamicsStep`) | No peer mutation, arrival-order winner, duplicate writer or partial publication |
| L5 Physical outcomes and persistence | Committed contact/topology facts, gameplay proposals and composite owner checkpoints | Outcomes are derived only after physical commit; all required owner segments restore together |
| L6 Presentation | Mesh/pose/particles/foliage/VFX extraction | Read-only projection; camera, renderer cadence and device state never feed authority |

L3 is deliberately plural. Inside Physical Embodiment, PhysX remains the
writer of rigid/articulated pose, velocity and contact; a continuum solver
writes only its material motion/region state; a living-structure solver writes
only its graph, elastic/damage and topology state. The adjacent SPEC-43
Thermochemical owner writes only composition, enthalpy, phase and durable
reaction progress of stable parcels. Mechanical owners retain pose, contact,
spatial mass and topology. Similar-looking quantities do not permit shared
ownership; a phase or combustion consequence uses a separate atomic exchange
profile.

An optional SPEC-44 model is not an L3 owner. Under ADR-081 it may submit one
bounded private proposal only to a report/shadow solve after the corresponding
classical owner step. It cannot change production work, write an exchange
batch, choose a representation, publish a root or count toward a multi-owner
promotion check. Runtime solver advice requires a later Accepted certificate-
backed decision.

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

Exchange records use the ADR-081 tuple: exact `world_namespace`, source and
destination owner IDs, applicable destination `world_id`, expected source and
destination revisions/roots, tick/substep, edge profile, stable participant
IDs and operation slot. They additionally carry fixed-point quantities, units
and explicit reference points for moments. `world generation`, save/checkpoint
generation and an undefined owner generation are forbidden key fields. Records
have a complete total order. Command retries are intercepted by the ledger.
Within a new closed owner batch, an exact duplicate or a different hash under
the same unique key is an internal invariant that rejects the uncommitted
composite step; the runtime never selects whichever result arrived first.

Broad `PhysicalLayer`, generic solver, arbitrary field map or plugin-owned raw
particle/node API is not a V1 public contract. Exact exchange schemas are
introduced only with their first production consumer under ADR-046.

## Canonical float execution candidate

Final fixed-point publication does not make an upstream float trajectory
canonical. A water or living-structure solver using authoritative private
`f64` MUST bind the ADR-081 `CanonicalFloatExecutionProfile` before solver
code: exact toolchain/target features, FMA contraction, rounding/subnormal
behavior, deterministic mathematical primitives, reduction/factorization/tie
order and fixed or quantized-exact convergence branching.

The serial same-target corpus remains the research gate. Production requires
exact Windows/Linux roots on boundary and adversarial convergence/rounding
cases. Failure after two coherent remediation cycles keeps the solver
research-only or requires a separate fixed-point/soft-float authority decision.

## Candidate stage-8 dynamics DAG

For a selected set of active owners, one substep is governed by one exact
`WorldDynamicsCompositionProfile` and is:

1. bind L0/L1 identities and freeze every prior owner projection;
2. evaluate declared L2 forcing projections from canonical inputs;
3. merge every canonical rigid input before one PhysX integration, then run
   downstream owner candidates in the profile's stable topological order;
4. canonicalize and validate all exchange batches before destination use;
5. integrate each owned representation exactly once under that profile;
6. validate all candidate roots, conservation/error receipts and capacities;
7. publish all participating owner states plus exchange receipts atomically;
8. derive gameplay proposals and L6 presentation from the committed result.

The profile binds the complete acyclic owner/edge DAG, access sets, cadence,
reducers, capacities, prior/result roots, checkpoint segments, performance row
and fault domain. Pairwise couplers cannot each integrate PhysX when several
edges are active. Runtime owns the coordinator; a post-execution validation
failure publishes no candidate and faults the declared domain rather than
attempting rollback-to-running without an exact clone proof.

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

The first PhysX-coupled production consumer additionally defines a fixed
positive checkpoint-epoch length. At every scheduled epoch boundary, whether
or not a save was requested, the live run reconstructs a fresh PhysX scene from
the canonical closure, validates the continuation witness and atomically swaps
it in. Save requests publish only after that same barrier. The uninterrupted
comparison run executes the same barriers, so save timing cannot change
physics. ADR-059's bounded training prefix is not an unbounded-world fallback.

The first consumer that adds a production owner creates the next current-only
composite checkpoint version. Later consumers create successors rather than
assuming that a speculative version number or absent segment can be filled by
defaults. Solver caches, spatial acceleration, modal basis caches, render
meshes and GPU buffers are rebuilt and remain outside authority. Failure of
scheduled rehydration to preserve exact roots and declared physical-quality
bounds blocks production promotion.

## Failure and fallback

Invalid profile/content and worst-case capacity closure reject before owner
activation. User-expressible capacity denial before freeze is an ordinary
action/transition rejection. Nonfinite value, overflow, non-convergence, stale
post-freeze revision, batch collision, missing frozen participant, exhaustion
beyond a reserved bound, coupling/backend rejection, topology-conservation
failure and corrupt persistence reject the complete participating step. The
prior committed generation remains the only authority.

A capability may select a separately authored fallback only before activation.
After activation there is no silent decorative substitution, owner freeze while
peers advance, mid-run backend switch, partial result or retry-to-green. The
composition profile binds `Running -> Faulted -> DiagnosticSaved -> Closed |
ExplicitRestore`. For the first primary gameplay world, a fatal fault stops the
whole application session; only independently provisioned training/test scene
slots may use narrower isolation.

## Promotion checks

This model has no standalone implementation check. Each concrete owner must
pass its own reference, coupling, persistence, cross-target and conditional
performance gates. A future composition check `PHYSICAL-LAYERS-P1` is created
only when at least two non-rigid Physical Embodiment production owners exist;
it must then prove
writer exclusivity, batch collision rejection, atomic multi-owner failure and
presentation independence. Until then, water, vegetation and thermochemical
checks remain independent and none proves another. A Thermochemical coupling
check does not count as a second Physical Embodiment owner by itself. Neural
checks can prove an optional accelerator only and never close this composition
gate.

Before the first integrated consumer, a successor `GameplayBudgetMatrix` adds
one mutually exclusive `world-dynamics-step` row whose measurement window
contains every physical substep, merge, validation and root publication in one
gameplay tick. PHYS-P4 and standalone solver numbers remain evidence, not this
row's authority.
