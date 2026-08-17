# SPEC-43: Proposed thermochemical material processes

| Field | Value |
|---|---|
| ID | SPEC-43 |
| Status | Proposed |
| Version | 1.1 |
| Last verified | 2026-08-17 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [SPEC-38](38-continuum-material-physics.md), [SPEC-39](39-layered-physical-world.md), [SPEC-40](40-structural-vegetation-physics.md), [SPEC-41](41-world-substrate-composition.md), [SPEC-42](42-arcane-substrate-and-physical-magic.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-071](adr/071-canonical-physics-material-lineage.md), [ADR-079](adr/079-thermochemical-material-process-track.md), [ADR-081](adr/081-world-dynamics-gap-closure-and-promotion-guardrails.md) |
| Related research | [Imported world-dynamics source papers](research/world-dynamics-source-papers.md) |
| Candidate revision note | Version 1.1 applies ADR-081 sub-LSB residual, exchange identity, successor-stage, parcel-topology, capacity, fault-domain, checkpoint and budget guardrails |

## Status and bounded purpose

This SPEC defines a candidate owner, state boundary and promotion path for
heat, phase and later reaction processes. It does not claim that temperature,
ice, fire, chemistry or a thermochemical runtime is implemented. Current
PhysX, continuum, vegetation, save/replay and public contracts remain
unchanged.

The first profile is deliberately closed: a sealed calorimetry bench contains
one bounded water parcel and one bounded thermal reservoir. It proves exact
heat transfer and reversible water/ice equilibrium with diagnostic overlays.
It does not simulate free-surface water, load-bearing ice, combustion,
atmosphere, moisture transport, deformable soil, corrosion, ecology or arcane
heating. Those are independent couplers after the base owner passes.

Architecture Package T0A is closed by this SPEC. T0B in the
[thermochemical roadmap](../plans/thermochemical-world/README.md) must freeze
the exact fixed-point scales, water/ice enthalpy curve, conductance law,
fixtures, capacities, thresholds and budget before solver code. The track is
post-v1 and `PLANNED / NOT_ACTIVE`.

## Ownership and material attachment

The Thermochemical owner is the sole writer of material-attached composition,
total enthalpy, phase partition and durable reaction progress. It does not own
pose, velocity, contact, structural topology, particle positions, gameplay
health or arcane quantity.

| State | Sole owner | Boundary rule |
|---|---|---|
| Tick, schedule, command ledger and composite publication | Runtime | No thermochemical clock, async commit or private retry path. |
| Rigid/articulated pose, contact and mechanical mass properties | PhysX/SPEC-26 | Thermochemical results become typed revision-bound requests; they never edit a body or native handle. |
| Continuum sample motion and region topology | SPEC-38 owner | Temperature/phase is not hidden in solver scratch; later coupling uses an explicit parcel-to-region mapping. |
| Tree graph, section damage and detached-body handoff | SPEC-40 owner | Heat or combustion may propose strength/mass/topology effects only through a separately promoted exchange. |
| Composition, enthalpy, equilibrium phase and reaction progress | Thermochemical owner | One canonical owner segment and one writer for every active parcel. |
| Ability/policy definitions | Mechanics | Content selects a validated profile; package code cannot inject trusted heat or declare ignition. |
| Arcane source quantity and conversion loss | Arcane/SPEC-42 | A future arcane edge debits Arcane and credits/debits thermochemical enthalpy atomically. |
| Temperature colors, steam/fire particles and overlays | Presentation | Read-only; visuals never create heat, phase or reaction facts. |

`MaterialParcelId` is an engine-owned stable identity bound to an exact
attachment record. The attachment names the participating owner, full stable
participant identity, definition revision and prior root. It is not a pointer,
ECS row, particle index, PhysX shape handle or render object.

An authored parcel ID is part of the validated content/placement closure. A
runtime-created parcel ID derives from ADR-022 causal command identity and a
canonical creation slot. ID or attachment-provenance collision fails before
activation/commit; arrival-order renaming or random retry is forbidden.

Mechanical owners retain their spatial mass and geometry authority. The
Thermochemical owner stores the closed species inventory for its parcel. The
active profile validates that summed species mass equals the frozen mechanical
parcel mass. V1 is sealed, so total parcel mass cannot change. A later
cross-owner mass transfer must atomically update the species inventory,
mechanical mass/topology and an ownership-transfer receipt; silently changing
one side is forbidden.

`ThermochemicalMaterialProfile` is separate from
`PhysicsMaterialDescriptorV2`, `ContinuumWaterProfileV1` and vegetation
structural profiles. A coupling-material mapping binds exact revisions and
surface/interface laws. A renderer material or tag is never sufficient.

## Canonical state and numeric authority

CPU checked fixed-point execution is the only candidate runtime authority for
T1-T7. An `f64` implementation may be an independent oracle or diagnostic; a
GPU implementation may be correspondence-only. Neither can select a phase,
reaction, branch, event or root.

Each accepted outer step publishes complete future-affecting state:

```text
ThermochemicalOwnerStateV1 {
  profile_revision,
  tick,
  completed_substep,
  parcels_sorted_by_id: ThermochemicalParcelStateV1[],
  interface_heat_residuals_sorted_by_id[(interface_id, signed_sub_lsb)],
  canonical_root,
}

ThermochemicalParcelStateV1 {
  parcel_id,
  attachment_revision,
  species_masses_fixed[],
  total_enthalpy_fixed,
  equilibrium_phase_class,
  reaction_progress_fixed[],
}
```

T0B selects exact widths, scales, raw bounds and checked intermediate rules.
Canonical collections sort by stable species/reaction ID. Every arithmetic
operation uses SPEC-21 ties-to-even and rejects overflow. Nonfinite source
data, missing table interval, unordered float reduction, implicit epsilon,
variable timestep and result-dependent iteration count are forbidden.

For the water/ice V1 profile, total enthalpy and species mass are the stored
source variables. Temperature and equilibrium ice/liquid fractions are
uniquely derived from the frozen piecewise enthalpy relation. Supercooling,
nucleation, hysteresis, metastability and history-dependent phase selection
are excluded because they would require additional durable state. A profile
cannot store both an independently mutable temperature and enthalpy.

The base reaction table is empty and its reaction-progress collection is
canonically empty. Adding even one reaction is a later profile with its own
species/element closure and evidence.

Each active interface owns one signed sub-LSB heat residual in canonical
orientation. T0B fixes its finer integer scale, bound and ties-to-even update.
The next transfer evaluates `law_joules + prior_residual`, publishes the rounded
equal-and-opposite joules and stores the signed remainder for that interface.
The residual participates in state roots and checkpoints; it cannot be
discarded as scratch or replaced by an implicit deadband. A physical deadband
requires an explicit material-law profile.

Internal lookup indexes, neighbor/interface caches, derived temperature/phase
fractions and presentation meshes are reconstructible. The next step begins
only from the published fixed-point state, interface residuals and exact
attachment projections.

## Base heat-transfer step

T1/T2 use one sealed set of parcels and a closed interface graph. An interface
is immutable profile data with stable endpoints, orientation, area and a
bounded conductance law. The graph cannot be modified by package callbacks
during a step.

The base profile runs once per 240 Hz substep, but Thermochemical state is not
admitted to the current physical-only stage-8 access set. Its first production
consumer therefore uses ADR-081's successor twelve-stage profile with
`WorldDynamicsStep` at stage 8 and a closed owner/DAG manifest; the current
runtime profile is unchanged. A later slower or multi-rate profile must define
an integer cadence, exact accumulation state and coupling semantics in a
separate decision; wall time cannot skip or merge thermal steps.

For each fixed interval:

1. freeze parcel states, attachments, interface/profile revisions and the
   exact mechanical mass projection;
2. derive temperatures from canonical enthalpy using the frozen profile;
3. evaluate every interface in canonical endpoint order, adding its prior
   signed sub-LSB residual;
4. round one fixed-point heat transfer, store the signed remainder and emit an
   equal-and-opposite source/destination entry;
5. reduce entries using declared exact reducers and checked bounds;
6. update candidate enthalpies once, derive candidate phase classes and
   validate mass/energy receipts;
7. publish all participating parcel states and the closed transfer receipt, or
   publish none.

The base fixture has no ambient infinite sink. Its finite thermal reservoir is
another parcel whose enthalpy changes by the opposite amount. `cold` is
enthalpy removal, not a second substance or negative-energy inventory.

Each `ThermochemicalHeatBatchV1` candidate binds the ADR-081 tuple:
`world_namespace`, source/destination owner IDs, applicable destination
`world_id`, expected owner revisions and roots, tick/substep, edge profile,
source/destination parcel IDs, interface ID and operation slot. It also binds
attachment/profile revisions, signed fixed-point joules, residual before/after
and result roots. Exactly one record is allowed for that key. An exact duplicate
or different hash under the same key rejects the complete step as an internal
invariant. Arrival or worker order never selects a record. Thermochemical,
checkpoint or save generation is not an exchange-key field.

## Physical and world coupling

The base owner can run without changing mechanical representation. Any effect
on another owner is a separate typed edge under SPEC-39/41:

- water/ice motion, density or collision changes wait for the relevant
  SPEC-38 owner and `THERMOCHEM-CONTINUUM-P1`;
- thermal expansion, pressure or rigid material change waits for an explicit
  PhysX material/topology consumer and `THERMOCHEM-RIGID-P1`;
- drying, strength loss or combustion of a tree waits for SPEC-40 exact
  persistence and `THERMOCHEM-VEGETATION-P1`;
- arcane heat/cooling waits for both base promotions and
  `ARCANE-THERMOCHEMICAL-P1`.

The edge freezes every owner projection, builds private candidate states and
publishes all owner roots plus an exchange receipt in one declared composite
transaction. A thermochemical process cannot directly set a rigid mass,
delete a continuum sample, break a tree edge, apply gameplay damage or emit a
success event as a substitute for destination-owner validation.

### Parcel topology and ownership transfer

Any parcel split, merge or attachment handoff is one atomic
`MaterialParcelTopologyTransaction` with the participating mechanical topology
transaction. Child IDs derive from the causal transaction identity plus
canonical child slots; successfully replaced parents become tombstones and
cannot be reused. Prepare/validate closes species mass, total enthalpy,
reaction progress, interface residual disposition and corresponding mechanical
mass before either owner publishes. There is no interval in which parent and
children are simultaneously authoritative and no thermochemical-only split.

## Reactions and combustion ladder

Heat and water/ice phase closure precede chemistry. A later reaction package
uses a closed, bounded species and reaction table. It must conserve declared
elements and mass, account reaction enthalpy exactly and publish durable
progress only when the kinetics law is history-dependent.

Combustion is a thermochemical reaction/coupling profile, not a universal
`FireDomain` and not a VFX system. It requires fuel, oxidizer, ignition law,
heat release, products and finite transport. Flame/light/smoke presentation
derives from committed reaction facts. Arbitrary plugin reactions, a global
species namespace, atmosphere and explosion mechanics are not part of V1.

## Persistence, streaming and representation

T5 introduces the first successor composite checkpoint only with a production
consumer. It stores exact active parcels, attachments, species inventory,
enthalpy, interface residuals, required reaction progress and cross-owner
receipts in the same atomic generation as participating owners. Missing or
corrupt required state cannot default to ambient temperature or equilibrium
content.

If the production composition includes PhysX and requires exact restart, it
binds a fixed positive checkpoint epoch. Each scheduled epoch reconstructs and
validates a fresh PhysX scene before atomic swap; saves wait for that barrier and
uninterrupted comparison runs execute the same barriers.

Sidecar saves, recomputing heat history from presentation, independent owner
publication and lossy thermal sleep before exact persistence are forbidden.
Regional summaries, atmosphere cells and dormant reaction approximation are
later representation transitions selected only from canonical facts and
integer budgets, never camera, wall time or measured frame cost.

## Failure and fallback

Worst-case parcel, interface, residual, reaction and topology-transaction
capacities are admitted before freeze. User-expressible denial is ordinary;
post-freeze exhaustion beyond the reserved bound is an invariant fault.

Invalid profile/content, missing attachment, stale revision, nonfinite input,
fixed-point overflow, table-domain failure, capacity excess beyond a reserved bound, duplicate batch,
mass/energy residual, destination rejection or corrupt checkpoint rejects the
complete participating step. The prior generation remains authoritative.
There is no clamp-to-green, retry with a smaller timestep, frozen thermal
state while mechanics advances or backend switch.

Before capability activation a project may use authored static material
states and ordinary mechanics. After activation, silent ambient-temperature
reset, decorative ice/fire, scripted ignition/damage or thermochemical state
omission is forbidden. A required but unsupported capability fails project
activation.

The first primary-gameplay profile maps a fatal thermochemical/world-dynamics
fault to the whole application session through `Running -> Faulted ->
DiagnosticSaved -> Closed | ExplicitRestore`. Only independently provisioned
test/training scenes may declare narrower isolation.

## Evidence and promotion

| Check | Required result |
|---|---|
| `THERMOCHEM-ENTHALPY-REF-P1` | The frozen sealed water/ice and thermal-reservoir corpus produces exact same-target roots, mass conservation and bounded energy residuals; all numeric/profile faults reject atomically. |
| `THERMOCHEM-HEAT-P1` | Interface/order/worker permutations produce the same transfer receipts and roots; sealed source plus sink energy is conserved exactly under the selected fixed-point law. |
| `THERMOCHEM-PHASE-P1` | Heating/cooling traverses the frozen water/ice curve without hidden hysteresis or double-owned temperature and matches the independent oracle thresholds. |
| `THERMOCHEM-PERSISTENCE-P1` | Save-at-N/resume-to-M equals uninterrupted roots exactly and corrupt/missing attachment or owner state fails before publication. |
| `THERMOCHEM-CROSS-TARGET-P1` | Windows/Linux canonical parcel, receipt, command and event roots match exactly before production promotion. |
| conditional `performance` | The frozen active-parcel/interface workload meets its standalone stop target and the complete combined workload passes the successor mutually exclusive `world-dynamics-step` row across every substep in one gameplay tick. |

Only `THERMOCHEM-ENTHALPY-REF-P1 = PASS` activates an R8 integration track.
Production promotion requires the base checks, affected `play`,
`content-package`, `persistence-replay`, `platform`, conditional `performance`,
a real production consumer and an Accepted successor ADR.

Coupler checks are independent. Passing heat/phase does not promote
combustion, continuum, vegetation, rigid or arcane coupling.

## Deferred public contracts

The lab adds no public schemas. The first production consumer may add only the
parcel definition/profile, canonical state, attachment, heat/exchange receipt,
presentation snapshot and composite checkpoint segment that it actually
uses. A generic reaction callback API, raw cell/species buffers, universal
quantity map, `CouplingGraph` service and plugin-defined solver interface are
excluded until multiple consumers prove one bounded contract.
