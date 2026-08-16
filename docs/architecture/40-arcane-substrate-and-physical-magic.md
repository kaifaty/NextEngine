# SPEC-40: Proposed arcane substrate and physical magic

| Field | Value |
|---|---|
| ID | SPEC-40 |
| Status | Proposed |
| Version | 1.0 |
| Last verified | 2026-08-16 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [SPEC-31](31-autonomous-quest-lifecycle-and-narrative-director.md), [SPEC-36](36-continuum-material-physics.md), [SPEC-37](37-layered-physical-world.md), [SPEC-38](38-structural-vegetation-physics.md), [SPEC-39](39-world-substrate-composition.md), [ADR-008](adr/008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-020](adr/020-rpg-domain-authority-and-extension-boundary.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-027](adr/027-physics-motor-and-animation-layering.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-071](adr/071-canonical-physics-material-lineage.md), [ADR-074](adr/074-world-substrate-and-arcane-physical-interaction-track.md) |
| Supersedes | none; defines a Proposed arcane research track without changing current gameplay, RPG, mechanics, physics, save or public-contract semantics |

## Status and first consumer

This SPEC defines candidate ownership, numerics, execution, physical coupling,
persistence and promotion rules for a world-level arcane substrate. It does not
claim that magic, a mana field, a spell runtime, Vital/Identity domains or any
`ARCANE-*` ProductCheck exists.

The first player-visible vertical is deliberately smaller than the source
research paper: one package-authored telekinesis ability uses the production
player action, `EffectRequestV1` and `WorldCommand` path; one sealed caster
reservoir is debited; one bounded canonical exchange batch applies force and
torque to one PhysX rigid body; debug presentation shows reservoir, plan and
work accounting. There is no heat, water, ecology, ley network, artifact,
healing, soul, teleportation or matter creation in this vertical.

Package A0 in the [standalone arcane roadmap](../plans/arcane-world/README.md)
must freeze the exact law/profile/fixture before code. The track is post-v1,
`PLANNED / NOT_ACTIVE` and not a fallback requirement for mandatory gameplay.

## Candidate authority and ownership

CPU execution with checked fixed-point publication is the only candidate
authority. Private `f64` work is allowed only inside a fixed declared step and
cannot cross a continuation boundary. GPU work may later provide presentation
or aggregate correspondence; it cannot own reservoir, execution, physical
effect, command, event or checkpoint state.

| State | Owner | Explicit exclusion |
|---|---|---|
| Arcane quantity, reservation, throughput use and active execution state | Arcane owner | Not an RPG character-resource copy or package reducer. |
| Skill/proficiency and inventory/equipment references | RPG Framework | Arcane state cannot rewrite them directly. |
| Ability/template definition, cooldown/phase policy and package state | Mechanics Runtime | Definition is immutable intent, not resource or physical authority. |
| Rigid pose, velocity, contact, mass and inertia | PhysX through SPEC-26 | Arcane code never writes transform/velocity or a backend object. |
| Continuum, tree or future thermal state | its separately promoted owner | An arcane coupler cannot manufacture an absent owner or decorative substitute. |
| Health/tissue and soul/identity | current RPG operation or no current owner | Mana is not health, tissue, soul, `PersistentId` or divine standing. |

## V1 ontology and law closure

The engine term is `ArcaneQuantity`; lore may call it mana, aether, qi, prana
or another authored name. A single mutable `mana: f32` is forbidden. The model
separates at least:

- available quantity or work budget;
- reservoir capacity and safe throughput;
- source/sink identity and transfer/conversion loss;
- optional potential, coherence, entropy and spectrum dimensions.

The first vertical activates quantity, capacity and throughput. Potential,
coherence, entropy and spectrum are fixed profile values unless A0 supplies a
measured consumer, exact state variables and a corpus proving why they must be
mutable. This keeps the ontology extensible without paying for an unconsumed
universal field.

A0 must choose exactly one bounded conservation model for the vertical:

1. closed sealed reservoir with no regeneration during the fixture;
2. declared external source with finite rate/capacity and an explicit source
   receipt; or
3. a hybrid whose every inflow is named and bounded.

Silent generation is forbidden. Physical work must satisfy the frozen
conversion inequality, including declared external work and loss. Matter
creation, teleportation, resurrection, time/causality change and true identity
mutation are not high-cost variants of this model; they require independent
ontological specifications.

## Canonical state and numerics

After each accepted arcane/physical step, complete future-affecting arcane
state includes:

- world/profile/definition revisions, tick/substep and canonical root;
- stable reservoir and execution IDs derived through ADR-022 causal identity;
- fixed-point available, reserved and consumed quantity;
- fixed-point capacity, throughput-window use and conversion residual;
- active effect phase, target/source revisions and remaining duration;
- pending exchange disposition and terminal failure/result code;
- only A0-selected mutable quality/spectrum/channel fields.

A0 freezes units, integer widths/scales, bounds, ties-to-even conversion,
cadence, duration limits, maximum concurrent executions, command/target
capacities and overflow/nonfinite behavior. The next step starts only from
published state. Solver scratch, graph compiler caches, spatial acceleration,
GPU fields and VFX buffers are reconstructible.

Wall time, renderer cadence, worker count, completion order and frame budget
cannot choose iteration count, target, success, cost or representation.
Adaptive iteration, unbounded graph execution and retry-to-green are forbidden.

## Mechanics and execution path

First-party magic is a normal SPEC-13 mechanics package. Its immutable ability
definition declares capability, targeting, phases, cooldown/cost policy,
bounded arcane profile reference, physical effect kind and presentation cues.
Package, Luau, Wasm, AI and first-party Rust paths all submit the same bounded
proposal; the engine revalidates source, target, revisions, resource,
throughput, physical capability and project lock.

V1 does not expose a generic spell-graph language. The one telekinesis consumer
compiles to a closed immutable execution plan with the minimal sequence:

```text
Acquire sealed reservoir reservation
 -> Limit quantity/throughput/duration
 -> Bind one rigid target and reference frame
 -> Convert to bounded force/torque request
 -> Maintain for fixed ticks or release
 -> Commit result and accounting receipt
```

A bounded acyclic spell graph may be proposed only after at least two
production abilities demonstrate shared composition. Loops, callbacks, dynamic
node registration, package-defined native code, raw field access and a trusted
package-provided exchange batch are not admitted.

## Arcane-to-rigid staged coupling

The first coupling profile specializes SPEC-37/39:

1. validate the command and freeze caster, reservoir, rigid body, world,
   profile and prior-root revisions;
2. reserve but do not yet publish the maximum arcane debit;
3. derive one fixed-point force/torque or impulse request for the fixed physical
   interval, with explicit world point or COM reference;
4. PhysX validates the batch and integrates the rigid body exactly once;
5. compute the canonical actual-work, residual/loss and unused-reservation
   receipt from accepted physical projections;
6. atomically publish the arcane debit/state, rigid state, terminal receipt and
   ordered events, or publish none.

The exchange record binds full `PhysicsBodyIdV1`, caster/reservoir/execution
IDs, command identity, world/body/arcane revisions, tick/substep, source and
target roots, profile/definition hashes, fixed-point force/impulse/torque,
application point, COM reference, maximum debit and conversion receipt. One
batch is allowed for `(world generation, command, tick, substep, execution,
target body)`. Any duplicate or different hash under that key rejects the
uncommitted action.

Telekinesis never sets a transform or velocity. Contact, gravity and rigid
constraints remain PhysX-owned. The cost law cannot derive physical energy
from impulse magnitude alone; A0 must freeze the work-accounting formula and
successful controls before implementation.

## Persistence and representation

Exact active state is required first. The arcane owner segment, active plans,
reservations and every coupling receipt needed for continuation participate in
the same composite world checkpoint as Runtime, RPG and physical state. A
sidecar, resource reconstruction from presentation or reissuing a command on
load is forbidden. Save-at-N/resume-to-M must equal uninterrupted execution.

Regional summaries, ambient fields, ley graphs, organism channel LOD and lossy
sleep are later representations. They cannot precede exact active persistence
or silently discard depletion, corruption, channel damage, persistent effects
or pending costs. Camera distance and VFX visibility never choose arcane LOD.

## Failure and fallback

Missing capability/profile/target, insufficient quantity, throughput excess,
invalid graph/phase, stale revision, nonfinite/overflow, batch collision,
capacity excess, work-accounting failure, PhysX rejection or corrupt checkpoint
publishes no partial debit, impulse, event or effect. Stable diagnostics
distinguish ordinary gameplay rejection from fatal internal inconsistency.

Before capability activation the project contains no arcane ability and may
use an authored ordinary interaction. Required arcane content on an unsupported
build rejects project activation. After activation there is no free cast,
VFX-only success, direct transform, scripted damage, frozen reservoir, GPU
authority or substitution with an unrelated mechanic.

## Evidence and promotion

| Check | Required result |
|---|---|
| `ARCANE-RESERVOIR-REF-P1` | Frozen reservoir/transfer/throughput/conversion cases preserve declared quantity and loss, reject bounds/faults and produce exact same-target roots. |
| `ARCANE-MECHANICS-P1` | First-party, data/Luau/Wasm and headless paths use the same package, capability, targeting, command and rejection semantics; no private magic mutation path exists. |
| `ARCANE-RIGID-COUPLING-P1` | One production telekinesis trace moves the real PhysX fixture only through the canonical batch; debit/work, duplicate/stale/backend failures and atomic rollback meet frozen thresholds. |
| `ARCANE-PERSISTENCE-P1` | Active save/restart and replay equal uninterrupted roots exactly; corrupt or missing owner closure fails before mutation. |
| `ARCANE-CROSS-TARGET-P1` | Windows/Linux canonical arcane, command, event and physical projection roots are exact before production promotion. |
| conditional `performance` | The predeclared active execution workload fits its incremental and integrated GameplayBudgetMatrix rows without changing authority. |

The isolated roadmap may become active R8 research only after
`ARCANE-RESERVOIR-REF-P1 = PASS`. Production promotion requires all five base
checks plus a consumer-backed Accepted ADR and current-only contracts.

Future coupling checks are independent and cannot borrow base completion:

- `ARCANE-THERMAL-P1` requires a promoted thermal owner and heat/work corpus;
- `ARCANE-CONTINUUM-P1` requires the relevant SPEC-36 water/terrain owner gate;
- `ARCANE-VEGETATION-P1` requires the relevant SPEC-38 persistence/coupling
  gate;
- `ARCANE-VITAL-P1` and `ARCANE-IDENTITY-P1` require separate production
  domains and cannot be inferred from RPG health or divine-standing payloads.

No public arcane contract is added for A0/A1 research. With the telekinesis
consumer, the smallest future candidate set is a profile/definition, canonical
reservoir/execution state, rigid exchange batch, immutable semantic query and a
successor composite checkpoint. Raw fields, generic solver interfaces,
arbitrary plugin graph nodes and direct gameplay access to physical/arcane
internals remain out of scope.
