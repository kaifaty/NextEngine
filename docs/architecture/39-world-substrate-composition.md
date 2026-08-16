# SPEC-39: Proposed world-substrate composition

| Field | Value |
|---|---|
| ID | SPEC-39 |
| Status | Proposed |
| Version | 1.0 |
| Last verified | 2026-08-16 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [SPEC-31](31-autonomous-quest-lifecycle-and-narrative-director.md), [SPEC-37](37-layered-physical-world.md), [ADR-008](adr/008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-020](adr/020-rpg-domain-authority-and-extension-boundary.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-074](adr/074-world-substrate-and-arcane-physical-interaction-track.md) |
| Specialization | [SPEC-40](40-arcane-substrate-and-physical-magic.md) |
| Supersedes | none; adds a Proposed cross-owner composition model without changing current runtime, RPG, Mechanics, PhysX, content, save or public-contract semantics |

## Status and purpose

This SPEC connects gameplay, RPG state, physical embodiment and future world
substrates without creating a universal world solver, a shared mutable state
store or a second command runtime. `WorldDynamics` is an architectural name for
their fixed-stage composition. It is not a new state owner, public service,
crate requirement or generic plugin bus.

Current Accepted owners and paths remain unchanged. The first proposed new
substrate is the bounded arcane lane in SPEC-40. Continuum, living structures
and future thermal state retain their independent SPEC-36/37/38 promotion
gates. Vital, soul, identity, divine and ontological domains have no current
authority or implementation obligation.

## World owner map

| Concern | Sole owner | Cross-owner rule |
|---|---|---|
| Tick, command identity, ledger, schedule and atomic publication | Core Runtime | No substrate creates a private clock, command queue, RNG or receipt path. |
| Character, inventory, equipment, skill/proficiency and current bounded character resources | RPG Framework | A substrate reads revision-bound views and proposes typed operations; it never copies RPG fields. |
| Ability definitions, package reducer state and semantic effect proposals | Mechanics Runtime | First-party and community magic use the same package/capability/`EffectRequestV1` path. |
| Rigid/articulated and promoted non-rigid physical state | Physical Embodiment owners from SPEC-26/37 | External substrates send canonical batches; only the physical owner writes pose, velocity, contact, material or topology state. |
| Future arcane reservoirs, channels, executions and fields | Arcane owner from SPEC-40 | RPG skill and Mechanics definitions are references, not duplicate arcane state. |
| Future tissue, growth and disease | no current owner | A Vital owner requires a concrete production consumer and separate SPEC/ADR. |
| Future soul, true-name, oath or continuity state | no current owner | Identity is not inferred from mana, RPG identity or `PersistentId`. |
| Presentation and diagnostics | Presentation | Read-only extraction; VFX, camera and renderer timing never select an outcome. |

An authored object may participate in several rows, but every mutable field
still belongs to exactly one owner. Similar words such as energy, health,
reserve, damage or identity do not authorize duplicated state.

## Composition layers

```text
L0 Exact project/profile/capability closure
 ↓
L1 Runtime identity, command ledger and fixed schedule
 ↓
L2 Mechanics/RPG validation and immutable cross-owner read set
 ↓
L3 Peer owner candidate states (physical, arcane, later vital/identity)
 ↓
L4 Typed exchange batches and one declared composite commit
 ↓
L5 Committed outcomes, owner segments and semantic queries
 ↓
L6 Read-only presentation and diagnostics
```

Layers are authority and transaction boundaries, not a permission for an upper
layer to mutate a lower one. Each new owner or edge declares exact inputs,
writes, order, numeric profile, capacities, failure semantics and evidence. A
global `WorldState` map, generic quantity bag or arbitrary domain callback is
forbidden.

## Command-to-outcome transaction

Every cross-owner action starts through the current production path:

1. player, AI, package, script or tool emits a bounded mechanics/command
   proposal;
2. Runtime authenticates capabilities, normalizes the canonical command body,
   computes ADR-022 identity and reserves it in the ledger;
3. domain validators freeze the exact owner/profile revisions and construct an
   immutable cross-owner transaction plan;
4. participating owners compute private candidate states and typed exchange
   batches at their declared fixed stage;
5. Runtime validates batch uniqueness, revisions, bounds, conservation/cost
   receipts and all candidate roots;
6. all participating owner states, terminal command receipt and ordered events
   publish together, or none publishes;
7. semantic queries and presentation derive only from the committed result.

The first implementation may specialize this flow inside existing Ingress,
`PhysicalStep` and Outcome boundaries, but it cannot add a hidden same-tick
re-entry. If the current ledger/schedule cannot keep a receipt pending until
the composite result, implementation stops until a consumer-backed schedule
decision closes that exact boundary.

## Typed exchange edges

Each cross-owner edge is a separately versioned profile, not one universal
coupler interface. A canonical exchange record contains at least:

- source and destination owner IDs plus full participant identities;
- world generation, command identity, tick/substep and unique batch key;
- source/destination definition, profile and prior-state revisions/roots;
- fixed-point quantities, units, reference frames and moment/COM references;
- declared source debit, destination effect and residual/dissipation receipt;
- finite capacities, exact canonical order and stable failure code.

Exactly one record/batch is allowed for a declared key. An exact duplicate or
a different hash under that key rejects the uncommitted composite action; the
runtime never chooses by arrival, worker or GPU completion order. Direct
references to another owner's ECS/backend buffers are forbidden.

## Representation, streaming and persistence

There is no current generic `RepresentationManager`. Each owner declares its
own bounded representation ladder and `Prepare -> Validate -> Commit ->
Stabilize` transfer. A later shared scheduler may coordinate already-proven
transitions, but cannot own their state or invent a global relevance score.

Authoritative representation choice may use only canonical simulation facts,
world residency, stable identities and manifest integer tokens. Camera,
visibility, measured frame time, wall clock, cache warmth and GPU completion
are forbidden. Exact active persistence precedes any lossy summary. A dirty or
future-affecting owner cannot be silently reconstructed from immutable content.

Every promoted owner adds one hash-bound owner segment to a successor composite
world checkpoint. Cross-owner receipts needed for continuation are in the same
atomic save closure; sidecars and independently published substrate saves are
forbidden. Load validates all required segments before replacing the target
world. Pre-v1 formats remain current-only under ADR-046.

## Mechanics, AI and semantic queries

Magic and other systemic mechanics remain `AbilityDefinition`/affordance/
`EffectRequest` consumers of SPEC-13. A data-driven effect may select a typed
owner operation, but package code cannot construct a trusted exchange batch,
debit a reservoir, write a physical transform or declare success.

AI receives capability-filtered immutable semantic facts such as available
reserve, overload risk or committed physical consequence. It does not inspect
raw field, particle, solver or hidden target state. Planning is a proposal;
owner validation remains the oracle. Events describe committed facts and never
replace owner state or permit same-tick mutation.

## Failure and fallback

Invalid content/profile, capability denial, stale revision, nonfinite value,
fixed-point overflow, capacity excess, batch collision, missing owner,
non-convergence, conservation/cost failure, backend rejection or corrupt
checkpoint publishes no partial owner state, receipt or event. The prior
complete generation remains authoritative.

Before a Proposed substrate capability is activated, a project may omit it or
use a separately authored ordinary mechanic. After activation there is no
silent VFX-only success, free resource use, scripted transform, owner freeze,
backend switch or retry-to-green. Required-capability absence rejects project
activation; an active fatal failure stops the affected run at the last complete
checkpoint.

## Promotion boundary

This composition model has no standalone current ProductCheck. A concrete
substrate first passes its own reference, production-path, coupling,
persistence, cross-target and conditional performance checks. A future
`WORLD-DYNAMICS-P1` is created only when two independently promoted non-RPG
owners participate in one production transaction; it must prove exclusive
writers, complete rollback, exact receipt/event order and presentation
independence.

No public `WorldDynamics`, generic domain, raw query, spell graph or exchange
API is added by this Proposed SPEC. A production consumer introduces only the
smallest current-only contracts it exercises, through a new Accepted ADR.
