# ADR-079: Thermochemical material-process track

| Field | Value |
|---|---|
| ID | ADR-079 |
| Status | Proposed |
| Version | 1.1 |
| Decision date | 2026-08-17 |
| Last verified | 2026-08-17 |
| Normative dependencies | [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-38](../38-continuum-material-physics.md), [SPEC-39](../39-layered-physical-world.md), [SPEC-40](../40-structural-vegetation-physics.md), [SPEC-41](../41-world-substrate-composition.md), [SPEC-42](../42-arcane-substrate-and-physical-magic.md), [SPEC-43](../43-thermochemical-material-processes.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-071](071-canonical-physics-material-lineage.md), [ADR-078](078-world-substrate-and-arcane-physical-interaction-track.md), [ADR-081](081-world-dynamics-gap-closure-and-promotion-guardrails.md) |
| Supersedes | none |
| Superseded by | Partially [ADR-081](081-world-dynamics-gap-closure-and-promotion-guardrails.md): it supersedes current-stage access, generation-key, discarded sub-LSB residual, non-atomic parcel topology, implicit fault-domain and legacy-budget clauses. |

## Context

The imported world-dynamics follow-up paper usefully separates material
identity, enthalpy, phase, species, reactions and combustion from rigid-body
motion. It also proposes broad registries, generic couplers, a conservation
ledger, atmosphere and multi-physics orchestration before Next Engine has one
thermochemical consumer. Copying that infrastructure would violate the
consumer-driven boundary and create ambiguous ownership with continuum,
vegetation and PhysX.

SPEC-39 already reserves a later heat/moisture/combustion owner but did not
define its state, first fixture, failure boundary or promotion path. Arcane
SPEC-42 also names thermal coupling without an actual destination owner.

## Proposed decision

### Create one bounded Thermochemical owner

Adopt SPEC-43. The owner stores stable material-parcel attachments,
composition, total enthalpy, equilibrium phase and durable reaction progress.
Physical owners continue to own motion, contact, spatial mass properties and
topology. RPG, Mechanics and Arcane retain their existing fields.

Material parcels use engine-owned stable identity and exact attachment
revisions. A parcel is not an ECS entity, backend object or generic property
bag. `ThermochemicalMaterialProfile` is separate from physical, continuum,
structural and rendering materials and is connected only by an exact mapping.

### Use enthalpy-first fixed-point authority

CPU checked fixed-point is the candidate runtime authority. Total enthalpy and
species inventory are stored; temperature and equilibrium phase are derived
from a frozen profile. For the first water/ice relation, supercooling,
nucleation, hysteresis and metastability are excluded so no hidden phase
history exists.

Private `f64` is oracle/diagnostic-only and GPU is correspondence-only. A
variable timestep, implicit epsilon, retry with smaller step, unordered float
reduction or backend-selected phase cannot affect authority.

One signed sub-LSB heat residual per active interface is canonical owner state,
participates in the next transfer and is root/checkpoint-bound. Discarding it
as scratch or inventing an implicit deadband is forbidden.

### Start with a sealed calorimetry bench

The first profile contains one bounded water parcel, one finite thermal
reservoir and a closed heat-transfer interface. It proves exact energy/mass
accounting and water/ice equilibrium at 240 Hz. Because this is non-physical
state, its production consumer uses ADR-081's successor `WorldDynamicsStep` at
stage 8 and closed composition profile; the current stage set is unchanged.
T0B freezes numeric scales, material curve, conductance, fixtures, capacities,
thresholds and budget before code.

Free water, mechanical ice, thermal expansion, combustion, atmosphere,
moisture/soil, tree fire and arcane heat are independent couplers. Base heat
and phase checks cannot promote them.

### Couple only by typed atomic exchange

Every heat or later mass/reaction edge freezes exact owner revisions, emits a
canonical bounded batch and publishes all participating candidate roots plus a
receipt atomically. The Thermochemical owner cannot directly set pose, change
tree topology, remove a continuum sample or emit gameplay damage.

Conservation evidence belongs to each exchange receipt and composite
transaction. Do not introduce a universal `ConservationLedger`, global
`CouplingGraph`, shared field database, public reaction callbacks or a
`RepresentationManager`.

Exchange records use ADR-081's complete namespace/owner/world/revision/root/
participant/operation tuple, never thermochemical, save or checkpoint
generation. Parcel split, merge and attachment handoff use one atomic
`MaterialParcelTopologyTransaction`; children derive from causal slots, parents
become tombstones and species mass, enthalpy, progress and mechanical mass
close together.

### Exact active persistence before summaries

The first production consumer adds one required Thermochemical owner segment
to a successor composite checkpoint. Attachments, inventories, enthalpy,
interface residuals, future-affecting reaction progress and exchange receipts restore atomically
with participating owners. Sidecars, ambient-temperature defaults and lossy
sleep before exact persistence are forbidden.

A PhysX-coupled exact-restart profile uses fixed scheduled checkpoint epochs;
save waits for the same rehydration barrier executed in uninterrupted runs.

## Failure and fallback

Invalid profile/attachment, stale state, nonfinite input, fixed-point overflow,
table-domain failure, capacity excess, duplicate batch, mass/energy residual,
destination rejection or corrupt persistence rejects the complete
participating step and retains the prior generation. There is no retry,
smaller timestep, frozen thermochemistry or backend switch.

Worst-case capacities are admitted before freeze; an expressible denial is
ordinary and exhaustion beyond reservation is an invariant. The first primary
gameplay profile faults the whole application session via ADR-081; only
independently provisioned test/training scenes may isolate a narrower slot.

Integrated performance authority is the successor mutually exclusive
`world-dynamics-step` row across every substep in one gameplay tick; standalone
bench figures are stop targets only.

Before activation, authored static material state is the fallback. After
activation, decorative ice/fire, scripted ignition/damage or silent state
omission is forbidden.

## Alternatives rejected

- Store temperature independently beside enthalpy: creates two mutable sources
  for one thermodynamic state.
- Put thermal state inside every solver's private scratch: prevents one
  persistent owner and creates incompatible cross-owner semantics.
- Treat fire as a standalone domain: fire is a reaction/transport process and
  still requires fuel, oxidizer, heat and destination-owner effects.
- Start with atmosphere, arbitrary reactions and all phases: no consumer or
  calibrated corpus justifies the state and contract surface.
- Let Arcane directly ignite or freeze objects: bypasses the destination owner,
  latent heat and atomic source/sink accounting.

## Consequences

- SPEC-43 and this ADR remain `Proposed`; runtime, contracts, schemas and saves
  do not change.
- The next action is T0B numerical/profile/corpus closure, not implementation.
- `THERMOCHEM-ENTHALPY-REF-P1 = PASS` is required before active R8 integration.
- The first independently promoted Thermochemical-to-Arcane transaction can
  satisfy the two-new-owner precondition for future `WORLD-DYNAMICS-P1`; the
  check still must prove the actual atomic composition.
