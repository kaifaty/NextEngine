# SPEC-20: Derived world calendar and authored NPC routines

| Field | Value |
|---|---|
| ID | SPEC-20 |
| Status | Accepted |
| Version | 3.1 |
| Last verified | 2026-08-15 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](22-schema-registry-compatibility-and-migration.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-29](29-platform-host-and-application-session.md), [ADR-008](adr/008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-019](adr/019-canonical-player-actions-and-presentation-authority.md), [ADR-021](adr/021-deterministic-population-residency-and-time-advance.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-025](adr/025-schema-content-and-migration-authority.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-034](adr/034-player-targeting-replay-v5-and-mapping-provenance.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-047](adr/047-simple-application-session-and-save-on-close.md), [ADR-048](adr/048-direct-exact-project-lock.md), [ADR-051](adr/051-r3a-packaged-chunk-streaming-commit-boundary.md), [ADR-052](adr/052-derived-world-calendar-and-authored-routine-vertical.md) |
| Replaces | SPEC-20 3.0; separates deterministic R4c/R4d work from optional learned R8 policies while preserving R4a scope |

## Status and admission

This document is the current contract for the first R4 product increment.
ADR-046 admission was completed in the same changeset as the production
consumer: the following five conditions are now part of the Accepted baseline:

1. the reference-alpha relay keeper consumes the authored routine through the
   production application path;
2. the engine-owned command/event, content, owner-segment and replay schemas
   below are implemented rather than listed as scaffolding;
3. `fast`, `play`, `persistence-replay`, `content-package` and the conditional
   report-only performance smoke pass the R4a scenario and its malformed-input
   cases;
4. ADR-052 becomes `Accepted` and explicitly supersedes every affected clause
   listed in its `Supersession` section;
5. affected Accepted SPECs, the architecture index, routing table,
   traceability map and roadmap describe the implemented current formats.

This admission closes R4a only. It does not close R4, B-07, B-12, B-13 or make
the R4b population/navigation contracts current.

## R4a product slice

R4a's sole production consumer is the existing quest giver
(the relay keeper) in the active reference-alpha relay-station chunk. One
authored routine contains
a single `Duty -> Rest` boundary. The exact gated interaction is
`nextengine.reference-alpha.interaction.accept-frontier-relay`, which performs
the existing dialogue `offer -> accepted`, quest `available -> active` and
relationship `0 -> 7` transaction. The separate quest-completion interaction
is unchanged in R4a.

The player-visible flow is:

```text
authored calendar + keeper routine
  -> exact end-of-tick world-time projection
  -> internal World Services routine command
  -> committed activity event and immutable projection
  -> existing interaction validation
  -> existing RPG quest transition
```

R4a intentionally excludes navigation, physical relocation, region transfer,
residency tiers, off-screen outcome synthesis, player wait, time scaling after
world creation, bulk time advance, weather, reservations, memory, learned
policies and a generic jobs/resource scheduler. The keeper remains in the
current active chunk. These exclusions mean R4a does not close R4, B-07, B-12
or B-13.

## Ownership

- Runtime owns `SimulationTick`, command admission, the two existing Ingress
  and Outcome phases, ledger identity and fixed-stage order.
- World Services owns the calendar/routine catalog reference, durable routine
  membership, committed activity and its revision.
- Asset & Persistence owns immutable typed calendar/routine content and its
  exact package/hash closure.
- RPG owns dialogue and quest aggregates. A schedule never writes an RPG
  field; it only changes a World Services activity fact used by ordinary
  interaction validation.
- Player Experience renders availability from immutable projections and cannot
  change a routine, calendar or quest from a widget callback.
- Streaming continues to own its existing reconstructible residency/cache
  state. The routine segment does not copy a chunk state or loaded entity.

`PersistentId` identifies the keeper across save/restart. A
`RuntimeEntityId`, camera visibility, renderer frame, host clock, I/O timing or
worker order cannot select activity or interaction availability.

## Current content contracts

The current implementation uses typed content, not generic property bags or reference-game
constants.

```text
WorldRoutineProfileV1 {
  1 schema_version: u16 = 1,
  2 anchor_simulation_tick: SimulationTick,
  3 anchor_world_tick: WorldTick,
  4 world_ticks_per_simulation_tick_num: u64,
  5 world_ticks_per_simulation_tick_den: u64,
}

WorldRoutineActivityV1 : u8 {
  Duty = 1,
  Rest = 2,
}

WorldRoutineDefinitionV1 {
  1 schema_version: u16 = 1,
  2 subject_id: PersistentId,
  3 initial_activity: WorldRoutineActivityV1 = Duty,
  4 transition_world_tick: WorldTick,
  5 next_activity: WorldRoutineActivityV1 = Rest,
}

WorldRoutineCatalogV1 {
  1 schema_version: u16 = 1,
  2 catalog_asset_id: AssetId,
  3 profile: WorldRoutineProfileV1,
  4 routine: WorldRoutineDefinitionV1,
}
```

The specialized content record uses owner `nextengine.assets`, schema
`nextengine.content.world-routine-catalog`, segment
`nextengine.world-routine-catalog.v1`, schema version `1`,
`NeutralContent` role and `CanonicalBinaryV1` encoding. Its catalog revision is
`domain_hash("nextengine.world-routine-catalog.v1", canonical_record_bytes)`.
The schema descriptor/current ref and field-registry hash are therefore part
of `SchemaRegistryManifestV2`; an opaque blob or generic neutral property map
is not an alternate encoding.

The complete catalog is one ordinary domain-relevant content asset with one
canonical revision hash. The transition tick is greater than the anchor world
tick; V1 admits exactly the closed `Duty -> Rest` pair and rejects every
unknown discriminant. All IDs, bounds, hashes and content references are
validated before project activation. R4a does not create
separate profile and routine assets whose independent revision closure has no
consumer.

V1 structurally contains exactly one subject and one transition, so it can
produce at most one due command in a tick. A second catalog, routine,
transition or unknown field returns `WORLD_ROUTINE_CONTENT_INVALID`; a larger
population requires a new version with its own consumer and is not evidence
that R4b or a 100-NPC workload exists.

The catalog's `subject_id` is the only source for the production
`quest_giver_character_id`. Project activation validates the typed catalog;
production world bootstrap then consumes that ID, constructs/binds exactly one
quest-giver Character and validates the required dialogue/quest/relationship
closure before the first tick. The current reference-session `[0x64; 16]`
constants are removed rather than compared as a second authority. The same
catalog-derived ID is threaded into the existing RPG aggregate,
`PhysicsBodyIdV1`/presentation binding and interaction target. Its existing
static physical descriptor (body/shape slot `0`, translation
`[-500000, 900000, 0]` micrometres) remains fixed reference composition; R4a
does not author a placement record or create a navigation/transfer API.

The cooker source becomes `nextengine.project-authoring.v3` and adds one
top-level typed field rather than encoding routine facts in neutral
properties:

```text
ProjectAuthoringManifestV3 {
  ...existing V2 fields,
  world_routine_catalog: Option<AuthoringWorldRoutineCatalogV1>,
  world_routine_interaction_binding:
    Option<AuthoringWorldRoutineInteractionBindingV1>,
}

NeutralProjectSourceV3 {
  ...existing V2 fields,
  world_routine_catalog_or_none: Option<WorldRoutineCatalogV1>,
  world_routine_interaction_binding_or_none:
    Option<WorldRoutineInteractionBindingV1>,
}

CookedProjectV3 {
  ...existing V2 fields with RpgDefinitionRegistryV2,
  world_routine_catalog_or_none: Option<WorldRoutineCatalogV1>,
}

ActivatedProjectV4 {
  ...existing V3 fields with RpgDefinitionRegistryV2,
  world_routine_catalog_or_none: Option<WorldRoutineCatalogV1>,
}
```

The one source-only binding is also closed and typed:

```text
WorldRoutineInteractionBindingV1 {
  1 interaction_id: SchemaId,
  2 subject_id: PersistentId,
  3 required_activity: WorldRoutineActivityV1 = Duty,
}
```

The authoring value uses the exact public catalog fields with ordinary hex
decoding for `AssetId`/`PersistentId`; it does not accept arbitrary property
keys. Cooker validation emits the canonical catalog as a domain-relevant root
asset: its `catalog_asset_id` occurs exactly once in `root_asset_ids` and the
content manifest, and that entry's record hash is the catalog revision stored
by the routine record. The cooker consumes the binding into exactly one
`InteractionDefinitionV2` and publishes no second runtime binding table.
For the bounded reference-alpha slice this is the only added content
entry: authoring `root_asset_ids` changes from `27` to `28` and the activated
`ContentManifestV1.asset_entries` count changes from `113` to `114`; the
existing interaction entry changes hash/version but does not add a second
entry. Reference-alpha requires both options to be `Some`; their subject IDs must
match and the interaction ID must be exactly
`nextengine.reference-alpha.interaction.accept-frontier-relay`. A project with
no conditioned interaction uses `None` for both and creates no routine owner
segment; `Some`/`None` mismatch is invalid content.

The same current-only API cut introduces `InteractionDefinitionV2` and
`RpgDefinitionRegistryV2`. `ProjectAuthoringManifestV2`,
`NeutralProjectSourceV2`, `CookedProjectV2`, `InteractionDefinitionV1`,
`RpgDefinitionRegistryV1` and `ActivatedProjectV3` are retired API types, not
migration readers.
`RuntimeBootstrapV4` replaces V3 because its `rpg_definitions` field now uses
`RpgDefinitionRegistryV2`; the separate routine state is constructed by the
World Services composition root, not embedded in Runtime bootstrap.
`RuntimeBootstrapV3` is retired rather than retained as an overload.

The reference application cut is also explicit. `ReferenceGameDriverV2` owns
the live routine owner beside Runtime and `WorldStreamerV1`.
`ReferenceLiveStateV2` adds
`world_routine_snapshot_or_none: Option<WorldRoutineSnapshotV1>` and its
restore API validates/publishes checkpoint, streaming and routine state
together. `ReferenceStageCheckpointV2` adds the optional activity/revision and
the application root; `ReferenceRunOutcomeV2` carries the optional routine
snapshot plus `WorldServicesTickCommitV1` results rather than a Runtime-only
root list. `ReferenceLiveDriverRecoveryV1`, `WorldStreamerV1` and the
unversioned `ActivatedProjectPackage` retain their own semantics while taking
the new activated/state types. `ReferenceGameDriverV1`,
`ReferenceLiveStateV1`, `ReferenceStageCheckpointV1` and
`ReferenceRunOutcomeV1` are retired API types, not fallback paths. An
authoring-v2 source returns
`UNSUPPORTED_PROJECT_AUTHORING_FORMAT` from its
outer probe. `ProjectLockV3`, `SchemaRegistryManifestV2`,
`ContentManifestV1`, `MechanicPackageManifestV1` and the package inventory
retain their wire semantics. The core interaction package carries V2
interaction hashes, so its mechanics/package hash and the existing project
lock edges change to bind the condition exactly. No defaults, migration or
in-place rewrite is introduced.

## Exact calendar projection

`WorldTick` is a nominal `u64`, distinct from `SimulationTick`. It is not a
second mutable clock. For simulation tick `s`:

```text
delta = s - anchor_simulation_tick
world_tick(s) = anchor_world_tick
              + floor(delta * world_ticks_per_simulation_tick_num
                      / world_ticks_per_simulation_tick_den)
```

The numerator and denominator are non-zero. Subtraction, multiplication,
addition and conversion use checked integer arithmetic with a `u128`
intermediate. A tick before the anchor, zero divisor or overflow fails before
world activation/load or before the affected tick starts. Floating point,
accumulated seconds, host time, locale and time zone are forbidden.

The one boundary's due tick is also exact and derived, never persisted:

```text
world_delta = transition_world_tick - anchor_world_tick       // > 0
due_delta = ceil(
  world_delta * world_ticks_per_simulation_tick_den
  / world_ticks_per_simulation_tick_num)
due_simulation_tick = anchor_simulation_tick + due_delta
```

`ceil` is checked integer ceiling division. Cooker/activation require
`due_delta > 0`, a representable `due_simulation_tick`, and exact witnesses
`world_tick(due_simulation_tick - 1) < transition_world_tick <=
world_tick(due_simulation_tick)`. Thus a zero numerator, overflow ratio or
unreachable boundary fails as `WORLD_CALENDAR_PROFILE_INVALID` before the
world exists; Runtime does not discover malformed arithmetic halfway through
the product scenario.

The catalog is immutable after world creation in R4a. Player `Wait`, a mutable
time scale, re-anchoring and bulk advance require a future consumer and a new
versioned decision.

## Current authoritative snapshot

```text
WorldRoutineRecordV1 {
  1 subject_id: PersistentId,
  2 record_revision: u64,
  3 catalog_asset_id: AssetId,
  4 catalog_revision: ContentHash,
  5 current_activity: WorldRoutineActivityV1,
}

WorldRoutineSnapshotV1 {
  1 schema_version: u16 = 1,
  2 record: WorldRoutineRecordV1,
}
```

Because V1 contains exactly one record, `record_revision` is also the routine
owner generation used by joint validation. A second mutable owner-revision
field would duplicate authority and is therefore absent.

The current activity is committed authoritative state because it gates
gameplay. The catalog anchor must equal the Runtime `next_tick` at fresh world
creation. At activation, where `next_tick == anchor_simulation_tick` and no
tick is complete, activity is `Duty`. On load with
`next_tick > anchor_simulation_tick`, the last completed tick is
`next_tick - 1`; activity must be initial before the boundary projected at
that completed tick and next at or after it. Thus a save after tick `T - 1`
with `next_tick == T` remains pre-boundary when the transition is due in tick
`T`. The due flag and any priority queue are pure derived caches and MUST NOT
be serialized or become a second authority.

R4a checks the single record directly: calendar projection and due discovery
are `O(1)`, and authoritative memory is `O(1)`. A reconstructible min-heap or
timing wheel may be evaluated only when the R4b 100-NPC consumer demonstrates
that a direct record scan is a measured limiting resource.

## Fixed-stage command and event semantics

At stage 6 of simulation tick `T`, World Services reads the last committed
routine snapshot. When `T == anchor_simulation_tick`, the interval is empty;
when `T > anchor_simulation_tick`, it computes
`(world_tick(T - 1), world_tick(T)]` with checked subtraction.
The admitted profile and routine are validated so this interval can cross at
most the single authored boundary for the one R4a subject.

When a boundary is due, the system stages exactly one `InternalSystem`
candidate for the already existing stage-9 Outcome batch:

```text
WorldRoutineCommandV1 : closed tagged union {
 CommitActivityBoundary = 1 {
  1 subject_id: PersistentId,
  2 expected_record_revision: u64,
  3 catalog_asset_id: AssetId,
  4 catalog_revision: ContentHash,
  5 boundary_world_tick: WorldTick,
  6 previous_activity: WorldRoutineActivityV1 = Duty,
  7 current_activity: WorldRoutineActivityV1 = Rest,
 }
}
```

Before an envelope enters the Outcome batch, the stage-6 producer recomputes
the boundary and activities from the exact catalog revision and validates the
complete owner/subject/revision/target/body closure. Stage 9 repeats that
engine-internal proposal validation against the captured staged context before
the Outcome batch is closed. Stage 6 creates the private proposal/envelope only
after its proof; a stage-9 candidate must be byte-exactly that captured
envelope. Any impossible stage-6 state or unexpected stage-9 mismatch is
`WORLD_ROUTINE_INTERNAL_INVARIANT`, not a domain `RejectionCode`: the prepared
joint tick is discarded before the candidate becomes a batch member or any
archive/high-watermark/reservation/receipt/`ReplayCommandResultV2` is created.
No caller-provided plan is trusted. Once this pre-admission proof succeeds, the
ordinary SPEC-21 transaction commits the single record/owner generation plus
one event atomically:

```text
WorldRoutineActivityChangedV1 {
  1 subject_id: PersistentId,
  2 previous_activity: WorldRoutineActivityV1 = Duty,
  3 current_activity: WorldRoutineActivityV1 = Rest,
  4 boundary_world_tick: WorldTick,
  5 record_revision: u64,
}
```

The event `record_revision` is the post-commit revision. Any other activity
pair is invalid in V1 even if a caller can construct its in-memory enum value.
The admitted internal command emits exactly this one event at causal event slot
`0`. Unauthorized commands from every other principal remain subject to the
existing generic authentication/capability/phase/stream rejection path and
emit no routine event.

The committed receipt uses one exact owner-write delta rather than an empty or
implementation-private value:

```text
canonical_world_routine_delta_bytes =
  "nextengine.world-routine-owner-write-set.v1\0"
  || u32_le(1)                         // exactly one record write
  || subject_id[16]
  || u64_le(before_record_revision)
  || u64_le(after_record_revision)     // before + 1
  || catalog_asset_id[16]
  || catalog_revision[32]
  || u8(previous_activity)             // Duty = 1
  || u8(current_activity)              // Rest = 2
  || u64_le(boundary_world_tick)
```

That byte string is the `canonical_delta_bytes` input to the exact SPEC-21
`delta_root`/`transaction_result_root` formulas; the event vector contains the
single slot-`0` event ID above. It is not the routine snapshot encoding and
cannot be replaced by empty bytes, a debug representation or a backend delta.

The command identity profile is fully pinned:

```text
system_id       = "nextengine.system.world-routine-boundary"
payload_schema  = "nextengine.command.world-routine" version 1
command_kind_id = "nextengine.command-kind.world-routine"
event_schema    = "nextengine.event.world-routine-activity-changed" version 1
capability_id   = "nextengine.capability.world-routine-commit"
capability_scope = None
validator_profile_hash =
  SHA256("nextengine.command-validator.world-routine.v1\0")
priority_class  = 250
phase           = Outcome only
target_tick     = current SimulationTick T
target          = Some(subject_id)
capability_claims = exactly { (capability_id, None) }
preconditions   = exactly one current
  CommandPreconditionV1::authoritative_revision(stage-9 Runtime revision)
stream_slot     = 0, stream_epoch = 0 for this InternalSystem principal
sequence        = expected_record_revision
```

R4a replaces the retired private three-entry descriptor/hash with one
canonical SPEC-21 `CommandKindRegistryV1` containing exactly these four map
entries. Every capability set is a singleton `CapabilityRefV1` with
`scope_hash = None`; the final column is the validator phase policy bound by
the stated validator-profile hash, not an extra V1 registry field.

| Map key `(payload_schema_id, version)` | `command_kind_id` | Priority | `validator_profile_hash` | Required capability | Validator phases |
|---|---|---:|---|---|---|
| (`nextengine.command.noop`, `1`) | `nextengine.command-kind.noop` | 100 | `SHA256("nextengine.command-validator.noop.v1\0")` | `runtime.command.noop` | Ingress, Outcome |
| (`nextengine.command.rpg`, `2`) | `nextengine.command-kind.rpg` | 200 | `SHA256("nextengine.command-validator.rpg.v2\0")` | `rpg.command.propose` | Ingress, Outcome |
| (`nextengine.command.physical`, `1`) | `nextengine.command-kind.physical` | 300 | `SHA256("nextengine.command-validator.physical.v1\0")` | `nextengine.capability.physical-avatar-intent` | Ingress only |
| (`nextengine.command.world-routine`, `1`) | `nextengine.command-kind.world-routine` | 250 | `SHA256("nextengine.command-validator.world-routine.v1\0")` | `nextengine.capability.world-routine-commit` | Outcome only |

The map key must equal each entry's payload schema/version and canonical map
ordering follows SPEC-21. These exact four entries, not a numeric private
registry version or a partially upgraded table, are the input to the canonical
registry envelope/hash and its golden vector.

World bootstrap registers the active `InternalSystem(system_id)` principal,
grants only that capability and derives its one `CommandStreamId` from the
world namespace through `CommandStreamRegistryV1`. External principals and
other internal systems cannot receive the capability. The outer Runtime
revision precondition is the stage-9 phase revision; the payload separately
binds the expected routine record/catalog revision. A successful transition
increments the record revision exactly once, keeping stream sequence
contiguous from initial record revision zero. The generic revision
precondition retains its current exact kind/owner/schema IDs
`runtime.authoritative-revision`, `nextengine.runtime` and
`nextengine.runtime.state`, with empty target key and `constraint_hash = None`.

The principal record is also pinned: `status = Active`,
`capability_subject_id = "nextengine.capability-subject.world-routine-boundary"`
and `provenance_hash = SHA256("nextengine.principal.world-routine-boundary.v1\0")`.
The live authority
registry and Replay V6 `AuthorityGrant` contain exactly the unscoped
`nextengine.capability.world-routine-commit` grant for this principal. The
principal/stream/authority entries are created and validated with the routine
owner before the first tick; partial bootstrap is forbidden.

R4a materializes that complete canonical `CommandKindRegistryV1`
defined by SPEC-21. It does not create a `CommandKindRegistryV2` or V3: the
V1 map's canonical bytes and content hash replace the private descriptor/hash
while its schema and envelope remain V1. Existing
`CanonicalCommandBodyV2`, `WorldCommandEnvelopeV2`, `PrincipalRegistryV1`,
`DomainEventEnvelopeV2`, `CommandStreamRegistryV1` and
`RuntimeDeterminismProfileV1` retain wire semantics. Their in-memory
`CommandPayload`/`EventPayload` dispatch enums gain the typed World Routine
variants selected by the schema IDs above; no opaque or unknown payload is
accepted.

The same R4a cut materializes the already-specified canonical
`ScheduleManifestV1`; an opaque fixed schedule constant is not a valid
substitute. For this first materialization, the previously abstract
`RuntimeStageId` is the following closed `u8` value and `stage_order` is exactly
the vector `1..=12`:

```text
1  InputIngest
2  CandidateAuthentication
3  IngressValidationAndPlan
4  IngressAdmission
5  IngressCommit
6  WorldStreamingCommit
7  AgentPlanning
8  PhysicalStep
9  OutcomeCommit
10 ResidencyCommit
11 StateHash
12 SnapshotPublication
```

These names bind the existing twelve SPEC-02 steps; they do not insert a new
stage. `command_admission_barriers` contains exactly
`(Ingress, 2, 0, AuthenticatedExternalAndQueuedInternal)` and
`(Outcome, 9, 0, InternalSystemOnly)`. Both phase-local allocators begin at
ordinal zero; an implementation literal that numbers the first Outcome batch
as `1` is retired with Replay V5 rather than encoded into the new
schedule/profile. The R4a `reducers` map is empty. Its
`systems` map contains exactly the engine-declared routine producer below;
the fixed built-in bodies of the twelve stages are bound by `stage_order` and
their existing component profile hashes, not duplicated as pretend registered
systems. That map contains
`nextengine.system.world-routine-boundary`, owned by
`nextengine.world-services`, at the existing stage-index 6
`WorldStreamingCommit`. Its descriptor has empty `before`, `after` and
`reducer_ids`; `query_order = PersistentId`; and
`shard_plan_id = "nextengine.shard-plan.world-routine-single"`. That
`LogicalShardPlanV1` names the same system, has `logical_shard_count = 1` and
uses the three required SPEC-21 V1 partition/order/merge enum values. Its exact
access set is:

```text
reads = {
  ("nextengine.runtime", "nextengine.runtime-snapshot", 2),
  ("nextengine.assets", "nextengine.content.world-routine-catalog", 3),
  ("nextengine.assets", "nextengine.content.world-routine-catalog", 4),
  ("nextengine.world-services", "nextengine.world-routine-snapshot", 2),
}
writes = {
  ("nextengine.world-services", "nextengine.world-routine-proposal", 1),
}
```

Field 2 of Runtime snapshot is `next_tick`; catalog fields 3/4 are the profile
and routine; routine-snapshot field 2 is the record; proposal field 1 is the
private single optional staged candidate. The system neither writes a live
owner nor adds an edge across a stage boundary. These access keys and the
one-shard plan are canonical schedule bytes and golden vectors, not
registration-order defaults. It remains registered when a project has no
routine catalog and deterministically emits no proposal, so the runtime profile
does not vary with project content.

One engine-owned determinism-bundle builder produces the exact canonical
`CommandKindRegistryV1`, `ScheduleManifestV1` and
`RuntimeDeterminismProfileV1` bytes plus their hashes. The cooker receives the
resulting exact profile hash and writes it into the unchanged `ProjectLockV3`;
activation, Runtime bootstrap, save and replay compare against the same
builder output. A separate cooker/activation constant such as a hand-written
`runtime_determinism_profile_sha256` is forbidden. Consequently the registry
and schedule hash changes flow through the runtime profile, project lock and
world identity roots, and a stale old-profile lock/input fails before atomic
project/world activation.

The command receives ordinary canonical body/hash/command identity, receipt
and causal event identity. No extra admission barrier or scheduler is added.
Player and agent Ingress work at tick `T` observes the snapshot committed at
the end of `T - 1`; the stage-9 transition appears in the tick-`T` state root
and presentation, and gates Ingress beginning at `T + 1`. This explicit
start/end-of-tick rule removes same-tick interaction ambiguity.

This requires one narrow extension of the current paired commit path; it is
not an already-existing RPG-only mutation. A private
`PreparedRuntimeWorldServicesTickV1` carries the staged Runtime transaction,
the staged routine owner and an optional prepared streaming publication.
Stage 6 derives the routine proposal from the jointly captured bases; stage 9
validates and applies it only to the staged routine copy. Joint validation
checks Runtime generation, routine `record_revision`/catalog revision and any
streaming publication before live mutation. Final
`commit_validated_world_services_tick` returns `Result` and begins with one
final read-only preflight of every captured Runtime generation, routine record
revision/catalog hash and streaming base hash. A stale validated token returns
`PREPARED_WORLD_SERVICES_GENERATION_STALE` before any write. Only after that
preflight succeeds does the method publish Runtime/RPG/physics, routine and
optional streaming generations in fixed order with no fallible work after the
first live write. Any preparation, validation or final-preflight failure
publishes none of them; a validated token is single-use.

The commit call holds simultaneous exclusive mutable access to the Runtime,
routine owner and streamer from the first preflight read through the last
infallible swap. It invokes no callback, task, I/O or `await`, so no owner can
change between successful preflight and publication.

Implementation threads one private `WorldRoutineStageContextV1` through the
existing tick preparation call. It contains the immutable activated catalog,
the captured committed projection and a mutable staged routine copy. The
stage-6 producer writes only its pending proposal; the stage-9
`CommandPayload::WorldRoutine` executor validates/mutates only that staged
copy. Neither `RuntimeState` nor the Runtime-owned `StagedAuthoritativeState`
acquires a second routine owner, and the context cannot be supplied to ordinary
Runtime-only `prepare_tick` entry points.

This does not move the Accepted streaming boundary. The existing
`WorldStreamingStageContext` still validates and logically publishes its
prepared transition at `WorldStreamingCommit`, after `IngressCommit` and
before `PhysicalStep`; physics and every later staged system observe that
stage-6 choice. The eventual infallible live-generation swaps merely realize
the already validated tick, as in ADR-051. Routine proposal derivation shares
stage 6 but routine mutation remains the priority-250 stage-9 Outcome; it
cannot delay or reclassify the streaming publication.

The existing `TickReport` keeps its Runtime-owned shape and is not silently
extended with World Services fields. The evidence-bearing validation path used
by scenario, checkpoint and replay transiently materializes the complete staged
core checkpoint (including the physics catalog), streaming/routine canonical
bytes, their sorted full descriptors and the application root before live
publication. Its commit returns one immutable `next_runtime` workspace result,
not a durable `crates/contracts` schema:

```text
WorldServicesTickCommitV1 {
  runtime_report: TickReport,
  world_streaming_snapshot: WorldStreamingSnapshotV1,
  routine_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
  streaming_transition_or_none: Option<WorldTransitionCommitV1>,
  application_owner_segments: Vec<SaveSegmentDescriptor>,
  application_state_root: StateRoot,
}
```

`application_owner_segments` is strictly sorted and unique by the complete
segment tuple and describes the exact committed bytes. The replay compare-point
writer consumes the descriptors/root directly. Canonical segment bytes are not
fields of this result and no no-reread claim is made: a later Save uses its
ordinary freeze/snapshot boundary and validates the same helper output. A
project without a routine catalog returns
`routine_snapshot_or_none == None` and the corresponding four-segment closure;
that closure uses the existing
`world_checkpoint_with_streaming_v1_state_root`. The reference R4a profile
always returns the five-segment closure. Root or
descriptor construction is validation work and cannot occur after the first
live write.

The ordinary `ReferenceGameDriverV2` live-advance path uses the same joint
Runtime/routine/optional-streaming generation validation and final stale-base
preflight, but does not construct application evidence that the caller would
discard every tick. `state`, `prepared_state` and `validated_state` materialize
the same canonical checkpoint/descriptors/root on an actual recovery, save or
replay boundary, before any such state is published. Both paths retain the
same fixed-order, no-fallible-work-after-first-write owner publication.

The private prepared/validated structs do not become public contracts or a
generic cross-context transaction framework. They specialize the existing R3
paired Runtime/World Services pattern; SPEC-23 remains Proposed. Agent planning
at stage 7 therefore still observes the prior committed routine state, while
the completed tick root includes the stage-9 transition.

An external principal cannot submit `CommitActivityBoundary`. Missing due work,
more than one due transition, stale revision, content mismatch or an internal
identity conflict is a deterministic failure; it is never silently deferred or
retried to green.

## Interaction condition

Authoring v3 attaches this typed condition to
`nextengine.reference-alpha.interaction.accept-frontier-relay`:

```text
WorldRoutineActivityConditionV1 {
  1 subject_id: PersistentId,
  2 required_activity: WorldRoutineActivityV1,
}

InteractionDefinitionV2 {
  ...existing V1 fields,
  availability_condition_or_none:
    Option<WorldRoutineActivityConditionV1>,
}
```

`interaction_definition_hash_v2` uses the existing V1 field-byte order, then
appends option tag `0` for `None` or tag `1` followed by the condition's raw
16-byte `PersistentId` and one-byte activity discriminant. The complete
preimage uses domain `nextengine.interaction-definition.v2`; V1's domain is
not reused. `RpgDefinitionRegistryV2` keeps the V1 fields and canonical sort
rules but changes `interactions` to `Vec<InteractionDefinitionV2>`. Its hash is
the existing ordered concatenation of dialogue, quest, relationship, V2
interaction and ability definition hashes followed by the mechanics-lock hash,
under domain `nextengine.rpg-definition-registry.v2`. Package manifests bind
the V2 interaction hashes. There is no dual V1/V2 lookup or fallback.

The acceptance definition carries `Some(subject_id, Duty)`; the separate
`complete-frontier-relay` definition carries `None`. Activation rejects a
condition unless the activated project also has exactly one catalog whose
subject matches and whose closed activity set contains the required value.
The source-only `WorldRoutineInteractionBindingV1` has already been consumed
by the cooker at this point. This field, not a generic neutral property or
subject-wide convention, is the carrier that binds the condition to one exact
interaction.

The immutable public query result is explicit and revision-bound:

```text
InteractionAvailabilityCodeV1 : u8 {
  Available = 0,
  WorldRoutineActivityUnavailable = 1,
}

InteractionRoutineRevisionBindingV1 {
  1 subject_id: PersistentId,
  2 routine_record_revision: u64,
}

InteractionAvailabilityV1 {
  1 schema_version: u16 = 1,
  2 interaction_id: SchemaId,
  3 interaction_definition_hash: ContentHash,
  4 routine_binding_or_none:
    Option<InteractionRoutineRevisionBindingV1>,
  5 code: InteractionAvailabilityCodeV1,
}
```

Code `1` has stable diagnostic
`WORLD_ROUTINE_ACTIVITY_UNAVAILABLE`. These query types live with the
engine-owned mechanics contracts and are immutable/non-durable; they are not a
new RPG operation or a UI-owned fact. Code `1` requires `Some` and an exact
condition match; an unconditioned interaction returns `Available` with `None`.
Canonical bytes use the numbered field order and option tags `0/1`; the hash is
`contract_hash(b"nextengine.interaction-availability.v1\0", canonical_bytes)`.
Replay records the complete typed value, not the rendered diagnostic string.

The interaction query and stage-9 outcome builder both read an immutable
revision-bound World Services projection. The private
`InteractionBuildContext` gains that read-only projection; it does not receive
a mutation callback. The interaction plan captures the exact routine record
revision and rechecks it before the priority-200 RPG command; the priority-250
routine command follows later in the same closed Outcome batch. `Duty` returns
`Available` and permits that exact acceptance interaction. `Rest` returns the
stable unavailable query code and constructs no RPG plan/command. The already
accepted `PlayerActionFrame` keeps `InputMappingReceiptV2`: its mapping code is
`Accepted`, with zero derived commands for this action. Dialogue, quest and
relationship revisions remain unchanged. The accepted
dialogue/quest/relationship mutation remains the existing RPG transaction; the
routine event never fabricates a quest transition.

## Save and replay

The routine snapshot is a separate segment:

```text
owner_id  = "nextengine.world-services"
schema_id = "nextengine.world-routine-snapshot"
segment_id = "world-routine"
version = 1
```

It coexists with `WorldStreamingSnapshotV1` under the same owner. Segment
identity and uniqueness use the complete `(owner_id, schema_id, segment_id)`
tuple; uniqueness by owner alone is invalid. `SaveManifestV2` and
`WorldCheckpointV4` retain their wire semantics. In particular,
`WorldCheckpointV4.state_root` remains the existing three-segment
Runtime/RPG/physics root. Following the R3 streaming pattern, R4a adds
`world_checkpoint_with_streaming_and_routine_v1_state_root`, which applies the
same canonical leaf framing to runtime, RPG, physics, world-streaming and
world-routine bytes. This five-segment application root is used by the R4a
application, save/restart checks and Replay V6; it does not silently redefine
the V4 checkpoint field.

R4a introduces current-only `ReplayManifestV6`. Its initial closure for
the R4a profile contains runtime, RPG, physics, world-streaming and
world-routine segments. Initial bytes reuse `ReplayOwnerSegmentV2`.
`ReplayTickManifestV6` starts from the ordered V5 tick facts, adds the missing
production streaming assignment below and otherwise retains their nested wire
types under the new current schema name. Its generic Outcome command batch,
command results and event vector carry the routine command/event without an
optional side channel. No `ReplayTickManifestV5` value is nested in a V6
manifest.

```text
WorldStreamingReplayInputV1 : closed tagged union {
  None = 0,
  BeginTransition = 1 {
    target_chunk_id: SchemaId,
    expected_base_world_state_hash: ContentHash,
    expected_next_world_state_hash: ContentHash,
  },
  CompletePendingTransition = 2 {
    target_chunk_id: SchemaId,
    expected_base_world_state_hash: ContentHash,
    expected_loaded_result_hash: ContentHash,
    expected_next_world_state_hash: ContentHash,
  },
}
```

The tick position is the completion assignment; worker/I/O timing is not
recorded. `BeginTransition` calls the ordinary begin path. `Complete` rebuilds
the pending load from the pinned activated project, exact-compares its result
hash, then calls the ordinary loaded-commit path. Every expected hash is
checked before joint tick publication. The compare-point streaming descriptor
and application root are the expected output, so a second receipt-shaped
replay side channel is unnecessary.

```text
ReplayTickManifestV6 {
  tick: u64,
  world_streaming_input: WorldStreamingReplayInputV1,
  closed_ingress_batch: ClosedIngressBatchV1,
  direct_external_commands: Vec<ReplayCommandRecord>,
  expected_ingress_command_batch: ClosedCommandAdmissionBatchV2,
  expected_physics_step_input: PhysicsStepInputV2,
  expected_contact_batch: ClosedPhysicsContactBatchV1,
  expected_targeting_intents: Vec<TargetingIntentV1>,
  expected_authoritative_targeting_queries:
    Vec<AuthoritativeTargetingQueryV1>,
  expected_physics_query_batch: PhysicsQueryBatchV1,
  expected_physics_query_results: Vec<PhysicsQueryResultV1>,
  expected_outcome_command_batch: ClosedCommandAdmissionBatchV2,
  expected_mapping_receipts: Vec<InputMappingReceiptV2>,
  expected_interaction_availability: Vec<InteractionAvailabilityV1>,
  expected_command_results: Vec<ReplayCommandResultV2>,
  expected_events: Vec<DomainEvent>,
}

ReplayComparePointV6 {
  tick: u64,
  state_root: StateRoot,
  command_ledger_hash: CommandLedgerHash,
  owner_segments: Vec<SaveSegmentDescriptor>,
  closed_ingress_batch_hash: ContentHash,
  ingress_command_batch_hash: ContentHash,
  physics_step_input_hash: ContentHash,
  contact_batch_hash: ContentHash,
  physics_query_batch_hash: ContentHash,
  physics_query_results_hash: ContentHash,
  targeting_query_trace_hash: ContentHash,
  outcome_command_batch_hash: ContentHash,
  interaction_availability_hash: ContentHash,
}

ReplayManifestV6 {
  schema_version: u32 = 6,
  compatibility: SaveCompatibility,
  initial_owner_segments: Vec<ReplayOwnerSegmentV2>,
  initial_state_root: StateRoot,
  authority: Vec<AuthorityGrant>,
  ticks: Vec<ReplayTickManifestV6>,
  compare_points: Vec<ReplayComparePointV6>,
}
```

`ReplayComparePointV6` replaces three hard-coded owner hash fields with
`owner_segments: Vec<SaveSegmentDescriptor>`, strictly sorted and unique by
the full tuple. Each descriptor therefore binds owner, schema, segment,
version, byte length and content hash; the replay runner exact-compares it
against the canonical bytes produced at that tick. The compare point also
retains the complete application state root, ledger hash and every V5
batch/query/trace hash. Mapping receipts remain in the V6 tick record. Initial
segments and compare-point descriptors are strictly sorted and unique by the
full tuple. Interaction availability values remain in canonical action
evaluation order; their batch hash is
`SHA256("nextengine.interaction-availability-batch.v1\0" || u32_le(count) ||
each lp(canonical_availability_bytes))`. Both root fields are the complete
project-bound application root. Replay V5 becomes a retired pre-v1 format and returns
`UNSUPPORTED_REPLAY_MANIFEST_VERSION` before nested decode; there is no
V5-to-V6 migration.

The V6 runner restores `WorldStreamerV1` and the optional routine owner from
their initial segments, then drives every tick through the same
prepare/validate/final-preflight/commit path. A runner-side list of hard-coded
transition ticks is forbidden.

For an activated catalog the routine segment is required; with catalog `None`
it is forbidden, and no conditioned interaction may exist. Save/replay loading
validates this project-bound segment closure, exact catalog hash, bootstrapped
subject binding, `next_tick`/activity relation and world-time projection before
publishing any state. A missing, extra, duplicate, corrupt or incompatible
routine segment preserves the previous valid generation and loaded world.

The routine snapshot and its dedicated Runtime ledger stream are one load
invariant. For the V1 `Duty -> Rest` catalog, only these closed states exist:

- `Duty` with `record_revision = 0` requires the exact genesis dedicated
  stream: `Open`, slot/epoch `0/0`, no high-watermark or greatest target tick,
  empty pending/receipt window, finalized count `0`, genesis receipt-chain
  root and no collision incident;
- `Rest` with `record_revision = 1` requires exactly one committed sequence-`0`
  receipt in that still-`Open` stream, no pending or extra sequence, no
  collision incident, high-watermark `0`, greatest target tick equal to the
  derived due tick, finalized count `1`, and the recomputed receipt-chain root.
  Its body/archive/identity occurrence,
  command ID, registry hash, target, catalog fields, slot-`0` event ID and
  transaction-result root must exact-match the canonical command, event and
  owner-delta bytes defined above.

Any other revision/activity/stream combination is
`WORLD_ROUTINE_LEDGER_CLOSURE_INVALID` and fails before publishing Runtime or
World Services state. With catalog `None`, the routine segment, conditioned
interaction and dedicated routine principal/stream/grant are all forbidden;
the schedule's registered producer remains a deterministic no-op.

## Failure semantics

| Failure | Required result |
|---|---|
| Invalid/overflowing calendar profile | `WORLD_CALENDAR_PROFILE_INVALID`; reject project/load or stop before the tick, with no partial mutation |
| Missing/duplicate/oversized catalog, second routine/transition, invalid condition binding or bootstrap subject collision | `WORLD_ROUTINE_CONTENT_INVALID`; reject complete project activation/world bootstrap before the first tick |
| Impossible stage-6 routine state or non-byte-exact stage-9 dedicated candidate | `WORLD_ROUTINE_INTERNAL_INVARIANT`; abort the prepared joint tick before Outcome admission, command/archive/ledger mutation or owner publication |
| Wrong principal/capability/stream/sequence, phase, schema or outer Runtime precondition from any other submitted command | retain the existing generic code and that branch's existing no-reservation or terminal-receipt semantics; no routine-specific code masks it |
| Runtime/routine/streaming joint-generation validation fails | publish none of the staged generations and retain every prior live owner |
| Missing/duplicate/corrupt save or replay segment | fail closed before world publication; preserve previous valid generation/source bytes |
| `ai-host`, learned model or navigation absent | no effect on R4a; authored routine and stationary interaction remain the complete path |

The one fatal code covers absent/mismatched activated owner/catalog,
subject/target, record revision, transition or due-tick facts; a bounded
structured detail MAY name the failed internal assertion but is not a second
hash-visible result code. The private `OutcomeProposal` constructor is not a
public/direct/script/plugin input. Stale live bases remain the separate
retryable `PREPARED_WORLD_SERVICES_GENERATION_STALE` final-preflight result,
before ledger mutation. Corrupt saved closure remains
`WORLD_ROUTINE_LEDGER_CLOSURE_INVALID`; a mismatched Replay V6 oracle remains
`NONDETERMINISTIC_RESULT`. This preserves SPEC-21's terminal receipt for every
actually admitted deterministic rejection while preserving the
genesis-or-one-committed-receipt routine load invariant.

## Current ProductChecks

These checks map the current R4a production consumer and its fail-closed
variants.

| Check | Scenario | Expected |
|---|---|---|
| `WORLD-ROUTINE-P1` through `play` | Fork two isolated production runs from the same `Duty`/quest-available pre-boundary state. In branch A submit `accept-frontier-relay` before the boundary; in branch B cross the boundary without accepting, query and submit the same interaction in `Rest` | Branch A performs dialogue `offer -> accepted`, quest `available -> active` and relationship `0 -> 7`, and the journal shows `Frontier Relay - Active`. Branch B query returns `WORLD_ROUTINE_ACTIVITY_UNAVAILABLE`; the accepted input receipt has zero derived commands, those RPG revisions stay unchanged and the journal shows `Frontier Relay - Available`. No navigation/tier/model path is required |
| `WORLD-ROUTINE-REPLAY-P1` through `persistence-replay` | Save with `next_tick == T` immediately before a tick-`T` boundary and again with `next_tick == T + 1`, restart the process and replay the same assigned inputs, including typed begin/complete streaming assignments; inject snapshot/stream receipt mismatches | Exact routine commands/events/delta and receipt-chain roots, full segment descriptors, typed interaction-availability values/hash and five-segment application root match uninterrupted execution without an off-by-one load rejection or runner-hardcoded transition tick; mismatched routine/ledger closure fails before publication |
| `WORLD-ROUTINE-CONTENT-P1` through `content-package` | Cook/activate authoring v3 and inject zero/overflow ratio, invalid boundary/activity/condition, missing catalog binding, duplicate catalog, bootstrap ID collision and an old/mismatched runtime-profile lock | Valid reference content produces the exact typed V3/V2/V4 chain, `28` root IDs, `114` content entries and canonical schedule/registry/profile closure; every invalid candidate fails before partial project/world publication |
| `fast` | Focused canonical codec, boundary arithmetic, stage-6/stage-9 internal-invariant faults, owner-delta/receipt roots, routine-stream load closure, joint commit, exact four-entry registry/schedule/profile/project-lock roots, segment order and retired-version tests | Exact bytes/diagnostics are stable; malformed dedicated proposals produce `WORLD_ROUTINE_INTERNAL_INVARIANT` with no batch/archive/receipt/owner publication, the schedule retains Ingress stage `2`/Outcome stage `9`, and no direct mutation, hardcoded keeper ID, opaque profile constant or owner-only segment lookup remains |
| conditional `performance --scenario smoke --mode report` | Compare the ordinary fixed-stage path with the stage-6 O(1) calendar/routine producer enabled | Runtime/root parity holds and the routine span/logical charge is attributed; verdict remains `REPORT_ONLY`, does not run `r4-100npc` and does not close B-12 |

`r4-100npc` remains an honest `NOT_RUN`/unavailable workload after R4a. It is
not a gate for this first consumer.

## Future expansion

R4b may promote residency tiers, graph navigation, region transfer and the
representative 100-NPC profile only with their own production consumers and
checks. R4c may promote deterministic SPEC-32 cognition after that substrate;
R4d may add the systemic work/currency/trade/food vertical. Learned policy
documents SPEC-33/34 and ADR-050/053/054 are optional R8 work and do not block
R4 or v1. A future bulk-time consumer
must stop at the same observable command boundaries as stepped execution and
cannot restore the removed ADR-021 migration or generic scheduler by default.

Future Strategic Agent cadence uses integer period/phase bound through the
project profile and may run only in declared fixed stages. Events enqueue work
for the next permitted boundary; they do not cause same-tick subscriber
re-entry. `Active`, `Simulated`, `Abstract` and `Dormant` map the requested
precision, while physical LOD remains separate. Abstract activity upgrades,
defers or blocks any traversal/combat/trade/quest outcome it cannot prove.
