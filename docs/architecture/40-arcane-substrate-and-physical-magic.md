# SPEC-40: Proposed arcane substrate and physical magic

| Field | Value |
|---|---|
| ID | SPEC-40 |
| Status | Proposed |
| Version | 1.2 |
| Last verified | 2026-08-17 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [SPEC-31](31-autonomous-quest-lifecycle-and-narrative-director.md), [SPEC-36](36-continuum-material-physics.md), [SPEC-37](37-layered-physical-world.md), [SPEC-38](38-structural-vegetation-physics.md), [SPEC-39](39-world-substrate-composition.md), [ADR-008](adr/008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-019](adr/019-canonical-player-actions-and-presentation-authority.md), [ADR-020](adr/020-rpg-domain-authority-and-extension-boundary.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-027](adr/027-physics-motor-and-animation-layering.md), [ADR-034](adr/034-player-targeting-replay-v5-and-mapping-provenance.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-071](adr/071-canonical-physics-material-lineage.md), [ADR-074](adr/074-world-substrate-and-arcane-physical-interaction-track.md) |
| Supersedes | SPEC-40 1.1; binds future heat/cooling to the separately promoted SPEC-41 owner without changing the A0-A5 telekinesis path |
| Related Proposed destination | [SPEC-41](41-thermochemical-material-processes.md), [ADR-075](adr/075-thermochemical-material-process-track.md) |

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

Package A0A architecture closure is complete. Package A0B in the
[standalone arcane roadmap](../plans/arcane-world/README.md) must still freeze
the exact numeric profile, fixture curves, thresholds and budgets before code.
The track is post-v1, `PLANNED / NOT_ACTIVE` and not a fallback requirement for
mandatory gameplay.

## Candidate authority and ownership

CPU execution with checked fixed-point arithmetic and publication is the only
candidate authority for A1-A5. Add/subtract/multiply/divide, dot/cross products,
rounding and bounds use the SPEC-21 checked fixed-point profile with `i128`
intermediates and ties-to-even. `f64` is oracle/diagnostic-only and cannot
select a request, debit, outcome, branch or root. GPU work may later provide
presentation or aggregate correspondence; it cannot own reservoir, execution,
physical effect, command, event or checkpoint state.

| State | Owner | Explicit exclusion |
|---|---|---|
| Arcane quantity, reservation, throughput use, active phase and recast lock | Arcane owner | Not an RPG character-resource copy or package reducer. |
| Skill/proficiency and inventory/equipment references | RPG Framework | Arcane state cannot rewrite them directly. |
| Ability/template definition and immutable cooldown/phase policy | Mechanics Runtime | V1 has no mutable telekinesis package reducer/cooldown state; definition is intent, not execution authority. |
| Rigid pose, velocity, contact, mass and inertia | PhysX through SPEC-26 | Arcane code never writes transform/velocity or a backend object. |
| Continuum, tree or thermochemical state | its separately promoted owner | An arcane coupler cannot manufacture an absent owner or decorative substitute. |
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
coherence, entropy and spectrum are fixed profile values unless A0B supplies a
measured consumer, exact state variables and a corpus proving why they must be
mutable. This keeps the ontology extensible without paying for an unconsumed
universal field.

V1 uses one closed sealed reservoir with no regeneration, recovery, external
source or negative-work recharge. One `ArcaneQuantity` unit (`AQ`) represents a
budget of one joule of maximum deliverable mechanical work before declared
conversion/control loss. Lore may rename the unit but cannot change the
profile conversion.

Silent generation is forbidden. Physical work must satisfy the fixed debit and
loss equation below. External sources, regeneration and energy recovery require
independent later profiles. Matter creation, teleportation, resurrection,
time/causality change and true identity mutation are not high-cost variants of
this model; they require independent ontological specifications.

## Canonical state and numerics

After each accepted arcane/physical step, complete future-affecting arcane
state includes:

- world/profile/definition revisions, tick/substep and canonical root;
- authored stable reservoir ID plus execution IDs derived through ADR-022
  causal identity;
- fixed-point available, reserved and consumed quantity;
- fixed-point capacity, throughput-window use and conversion residual;
- active effect phase, target/source revisions and remaining duration;
- pending exchange disposition and terminal failure/result code;
- only A0B-selected mutable quality/spectrum/channel fields.

A0B freezes integer widths/scales, raw bounds, cadence, efficiency/loss and
maintenance coefficients, duration limits, maximum concurrent executions,
command/target capacities and overflow behavior. The next step starts only
from published state. Solver scratch, graph compiler caches, spatial
acceleration, GPU fields and VFX buffers are reconstructible.

Wall time, renderer cadence, worker count, completion order and frame budget
cannot choose iteration count, target, success, cost or representation.
Adaptive iteration, unbounded graph execution and retry-to-green are forbidden.

## Mechanics and execution path

First-party magic is a normal SPEC-13 mechanics package. Its immutable ability
definition declares capability, targeting, phase/cooldown policy, bounded
arcane profile reference, physical effect kind and presentation cues. V1 does
not mutate package reducer state. Package, Luau, Wasm, AI and first-party Rust
paths all submit the same bounded proposal; the engine revalidates source,
target, revisions, resource, throughput, physical capability and project lock.

The due stage-5 Ingress transaction creates the Arcane-owned active execution,
recast lock and one conservative reservation for the complete finite declared
plan, then finalizes the ordinary `CommandReceipt` as `execution started`.
That reservation is partitioned into canonical per-substep maximum-debit
slices. It does not claim that a later physical substep succeeded. Each active
execution consumes only its published current slice when it produces a bounded
stage-8 request, without calling package code again or reserving the same work
twice. Release and cancel are new `WorldCommand` values applied only at their
declared future boundary.

V1 does not expose a generic spell-graph language. The one telekinesis consumer
compiles to a closed immutable execution plan with the minimal sequence:

```text
Acquire sealed reservoir reservation
 -> Limit quantity/throughput/duration
 -> Bind one snapshot-proven rigid target and reference frame
 -> Convert to bounded force/torque request
 -> Maintain for fixed ticks or release
 -> Commit per-substep exchange receipt
 -> Publish completion through the existing Outcome boundary
```

A bounded acyclic spell graph may be proposed only after at least two
production abilities demonstrate shared composition. Loops, callbacks, dynamic
node registration, package-defined native code, raw field access and a trusted
package-provided exchange batch are not admitted.

## Targeting, identity and first residency profile

The live and headless path is fixed:

```text
PlayerActionFrame
 -> TargetingIntentV1
 -> snapshot-bound PhysicsQueryBatchV1 / PhysicsQueryResultV1
 -> EffectRequestV1
 -> WorldCommand
```

Camera pose, depth buffer, renderer visibility and a package-supplied body ID
cannot select authority. The selected result binds its exact physics snapshot
selector and ordered query trace under ADR-034.

The first fixture keeps one caster and one dynamic rigid crate in the same
sealed, pinned active region. Streaming, region transfer and despawn are
forbidden during the fixture. Before owner freeze, a missing caster/target
terminates the execution deterministically, applies no force and returns the
unused reservation. After freeze, participant removal cannot be scheduled
before the substep closes; a missing participant is an internal invariant.

The reservoir is an authored stable slot bound to the caster `PersistentId`.
An execution ID derives from the causal start command plus a contiguous
execution slot. Exchange identity uses the exact tuple `world_namespace`,
physics `world_id`, expected prior `world_revision`, tick, substep, execution
ID and target body; the ambiguous term `world generation` is not a schema
field.

## Arcane-to-rigid staged coupling

The first coupling profile specializes SPEC-37/39:

1. collect active executions in canonical order `(reservoir_id, command_id,
   execution_id, body_id)` and sequentially account their already published
   current reservation slices in one Arcane candidate;
2. terminate ordinary ineligible executions before physical freeze while
   retaining other valid executions; insufficient capacity at start was an
   ordinary stage-5 rejection and cannot create a partially reserved plan;
3. freeze caster, reservoir, rigid body, world, profile and prior-root
   revisions for the accepted set;
4. derive one fixed-point force plus optional free torque request for each
   fixed physical interval, with explicit application point and COM reference;
   impulse is not part of the V1 profile;
5. compile the private `ArcaneRigidExchangeV1` records into the existing
   `PhysicsStepInputV1.external_force_requests` collection; V1 adds no public
   physics-step field or generic coupling bus;
6. execute PhysX exactly once on staging and build its candidate projection/
   snapshot without publication;
7. compute canonical work, maintenance/loss, debit and unused-reservation
   exchange receipts from the accepted candidate projection;
8. validate every Arcane/PhysX candidate root and atomically publish Arcane
   plus rigid state and exchange receipts, or publish none;
9. on internal failure discard staging and reconstruct the adapter from the
   prior canonical snapshot; rollback failure stops the instance.

The exchange record binds full `PhysicsBodyIdV1`, caster/reservoir/execution
IDs, command identity, exact world tuple, body/arcane revisions, tick/substep,
source and target roots, profile/definition hashes, fixed-point force/free
torque, application point, COM reference, maximum debit and conversion
receipt. An exact command retry is resolved by the ledger and emits no new
exchange. Inside a newly constructed closed batch, an exact duplicate key or
different bytes under that key is a fatal internal invariant and rejects the
complete participating `PhysicalStep` before PhysX publication.

### V1 work and maintenance law

All operands below are fixed-point values from the frozen request and the
quantized pre/post PhysX projections. For force application point `p`, COM `c`,
linear velocity `v`, angular velocity `omega`, force `F` and free torque `T`:

```text
v_point = v + omega cross (p - c)
v_mid = ties_even((v_point_before + v_point_after) / 2)
omega_mid = ties_even((omega_before + omega_after) / 2)
positive_work = max(0, ties_even(dt * (dot(F, v_mid) + dot(T, omega_mid))))
maintenance = ties_even(dt * (k_active + k_force * l1(F) + k_torque * l1(T)))
debit = positive_work + maintenance + declared_conversion_loss
unused = reserved_maximum - debit
```

A0B freezes the coefficient raws, efficiency/loss rule and a conservative
`reserved_maximum` slice bound from the declared force, torque, velocity,
duration and cadence ceilings. The sum of all slices is the full reservation
acquired at stage 5. Each committed substep debits its slice, returns that
slice's unused amount and leaves later slices reserved; termination returns
all unconsumed future slices. Negative mechanical work is dissipated and never
recharges the reservoir. A stationary held body still pays maintenance.
Overflow, negative unused quantity or debit above the current slice rejects
the complete participating step.

Telekinesis never sets a transform or velocity. Contact, gravity and rigid
constraints remain PhysX-owned. A0B must freeze the remaining coefficient raws,
fixture force curve and successful controls before implementation.

## Persistence and representation

Exact active state is required first. At A4, a successor current-only composite
checkpoint requires an Arcane owner segment whenever the capability is active.
The segment stores active plans, recast locks, reservations and every exchange
receipt needed for continuation. Arcane and PhysX restore into staging,
validate their complete cross-owner closure and publish together. An absent
segment cannot default to a zero reservoir. A sidecar, resource reconstruction
from presentation or reissuing a command on load is forbidden. Save-at-N/
resume-to-M must equal uninterrupted execution.

Regional summaries, ambient fields, ley graphs, organism channel LOD and lossy
sleep are later representations. They cannot precede exact active persistence
or silently discard depletion, corruption, channel damage, persistent effects
or pending costs. Camera distance and VFX visibility never choose arcane LOD.

## Failure and fallback

Capability denial, insufficient quantity/throughput, ineligible or pre-freeze
missing target and explicit cancel are ordinary rejections/terminations for one
execution. They publish no force, return unused reservation and do not prevent
other valid executions in the closed set.

Post-freeze stale/missing participant, nonfinite/overflow, duplicate exchange
key, different bytes under one key, work-accounting mismatch, PhysX rejection,
rollback failure or corrupt checkpoint is fatal. It rejects the complete
participating `PhysicalStep` and halts advancement of the affected world at
its prior complete generation; the active execution is not automatically
retried. Rollback failure or untrusted backend state stops the instance. Stable
diagnostics encode the class and never convert an internal failure into
ordinary gameplay.

Before capability activation the project contains no arcane ability and may
use an authored ordinary interaction. Required arcane content on an unsupported
build rejects project activation. After activation there is no free cast,
VFX-only success, direct transform, scripted damage, frozen reservoir, GPU
authority or substitution with an unrelated mechanic.

## Evidence and promotion

| Check | Required result |
|---|---|
| `ARCANE-RESERVOIR-REF-P1` | Frozen reservoir/transfer/throughput/conversion cases preserve declared quantity and loss, reject bounds/faults and produce exact same-target roots. |
| `ARCANE-MECHANICS-P1` | At integrated A3, first-party, data/Luau/Wasm and headless paths use the same package, authoritative targeting, start-command and rejection semantics; no private magic mutation path exists. |
| `ARCANE-RIGID-COUPLING-P1` | One production telekinesis trace moves the real PhysX fixture only through the canonical batch; debit/work, duplicate/stale/backend failures and atomic rollback meet frozen thresholds. |
| `ARCANE-PERSISTENCE-P1` | Active save/restart and replay equal uninterrupted roots exactly; corrupt or missing owner closure fails before mutation. |
| `ARCANE-CROSS-TARGET-P1` | Windows/Linux canonical arcane, command, event and physical projection roots are exact before production promotion. |
| conditional `performance` | The predeclared active execution workload fits its incremental and integrated GameplayBudgetMatrix rows without changing authority. |

The isolated roadmap may become active R8 research only after
`ARCANE-RESERVOIR-REF-P1 = PASS`. Production promotion requires all five base
checks, `play`, `content-package`, `persistence-replay`, `platform`, conditional
`performance`, a consumer-backed Accepted ADR and current-only contracts.

Future coupling checks are independent and cannot borrow base completion:

- `ARCANE-THERMOCHEMICAL-P1` requires a promoted SPEC-41 owner and exact
  source-debit/enthalpy/phase corpus. Heat credits enthalpy; cooling removes
  enthalpy into a declared sink. Arcane cannot directly select temperature,
  ignition, phase, strength, damage or presentation success;
- `ARCANE-CONTINUUM-P1` requires the relevant SPEC-36 water/terrain owner gate;
- `ARCANE-VEGETATION-P1` requires the relevant SPEC-38 persistence/coupling
  gate;
- `ARCANE-VITAL-P1` and `ARCANE-IDENTITY-P1` require separate production
  domains and cannot be inferred from RPG health or divine-standing payloads.

No public arcane contract is added for A0A/A0B/A1 research. A2 freezes Proposed
schema and schedule detail only; public contracts land only in the coherent
integrated A3 consumer checkpoint. The smallest future candidate set is a
profile/definition, canonical reservoir/execution state, private rigid exchange
record, immutable semantic query, `ArcanePresentationSnapshotV1` and a
successor composite checkpoint at A4.
The presentation snapshot contains only reservoir level, execution phase,
target identity, wrench and work/loss receipt; it exposes no mutable/raw owner
state. Raw fields, generic solver interfaces, arbitrary plugin graph nodes and
direct gameplay access to physical/arcane internals remain out of scope.
