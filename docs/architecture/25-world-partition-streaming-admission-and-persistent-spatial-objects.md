# SPEC-25: World partition, streaming admission and persistent spatial objects

| Поле | Значение |
|---|---|
| ID | SPEC-25 |
| Статус | Accepted |
| Версия | 1.0 |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-23](23-jobs-memory-resource-residency-and-io-backpressure.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [ADR-021](adr/021-deterministic-population-residency-and-time-advance.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-026](adr/026-deterministic-work-resource-and-streaming-admission.md) |
| Заменяет | отсутствует |

## Назначение и invariants

SPEC-25 определяет один public substrate, по которому cooker, World Services,
Runtime, persistence и world streamer независимо реализуют один и тот же
partitioned world.

- `WorldPartitionManifestV1` MUST быть immutable, content-addressed и bound к
  exact schema/content/resource policy hashes из `ProjectCompositionLock`.
  Его `initial_placement_catalog_hash` связывает только immutable authored
  new-world seed catalog и MUST NOT представлять либо хешировать текущие
  mutable placement/tombstone state.
- Текущие durable placements, tombstones, cross-chunk references и logical
  membership indexes MUST публиковаться отдельным immutable
  `WorldPlacementStateV1` generation с exact `SpatialPlacementStateRootV1`.
  Manifest seed hash и current state root являются разными domains и MUST NOT
  подменять друг друга в project lock, save, replay или admission.
- World topology, logical regions, durable placement, tombstones и population
  tier являются World Services-owned state. Asset & Persistence публикует
  immutable content, Runtime владеет resident mapping и commit schedule, а RPG,
  Agent и Physical subsystems сохраняют свои отдельные authoritative fields.
- Durable object MUST сохранять один `PersistentId` через chunk load/unload,
  tier transitions, save/load и replay. `RuntimeEntityId` является ephemeral и
  MUST NOT появляться в manifest, bundle, save, replay или cross-chunk reference.
- Camera visibility, renderer frame rate, wall time, worker identity, filesystem
  order и I/O completion order MUST NOT выбирать residency, admission, tier,
  placement либо mandatory outcome.
- Required streaming unit становится visible только одной atomic publication
  после полной schema/hash/dependency/identity validation. Partial chunk,
  topology, registry или authoritative object publication запрещена.
- Missing fidelity MUST deterministically produce a sufficient-tier upgrade or
  bounded defer. Streamer MUST NOT fabricate contact, traversal, combat,
  inventory, quest, dialogue, relationship or other foreign-owner outcome.
- `game`, `headless` и `capture-worker` MUST use the same partition manifest,
  interest ordering, admission decisions, object identities and state hashes.

## Source of truth и authority split

| State | Единственный owner/source of truth | Allowed projection / forbidden duplicate |
|---|---|---|
| Cooked topology, cell/chunk definitions, dependency closure and immutable initial-placement seed catalog | Asset & Persistence subsystem, exact `ContentManifestV1`/bundle hashes | generation-zero seed input for World Services; no current placement/tombstone authority |
| Logical regions, anchors, current durable placement/tombstone generation and root, interest facts and population tier | World Services subsystem | immutable spatial/population snapshots; no ECS entity or physical pose |
| Resident `PersistentId` to ephemeral runtime mapping and activation commit | Runtime subsystem | diagnostic index; no durable registration authority |
| Character/item/quest/dialogue/faction state | RPG Framework | revisioned facts; no duplicate values in placement record |
| Agent plan, attention and memory | Agent Intelligence subsystem | immutable capability facts; no stream-owned plan |
| Pose, contacts, constraints, traversal result and physical LOD | Physical Embodiment subsystem | quantized placement intent/result; no World Services-owned pose |
| Save generations and owner-segment encoding | Asset & Persistence subsystem | staged copy; no semantic ownership of decoded placement fields |

Partition residency and `WorldResidencyTier` are related but not identical.
Partition residency describes admitted content/data. Population tier describes
which World Services/owner work is due. Physical LOD remains a third, separately
owned state machine. A transition joins them only through immutable
revision-bound inputs and one validated common commit.

## Public contracts

Public schemas contain only engine-owned nominal IDs, bounded values, canonical
hashes and versioned descriptors. They contain no ECS storage/component,
filesystem path, OS object, native task/thread handle, database connection,
importer record, renderer/physics handle or vendor/backend type.

### `WorldPartitionManifestV1`

```text
WorldPartitionManifestV1 {
  schema_version,
  partition_id,
  coordinate_profile_id,
  topology_revision,
  root_region_ids[],
  region_descriptors[],
  cell_descriptors[],
  anchor_descriptors[],
  adjacency_descriptors[],
  chunk_bindings[],
  initial_placement_catalog_hash,
  residency_policy_hash,
  schema_registry_hash,
  content_manifest_hash
}
```

The manifest is encoded canonically without an embedded self-hash. Its
domain-separated SHA-256 is stored by the enclosing content/project lock.
Arrays sort by their canonical nominal IDs; duplicates and alternative
byte encodings of the same identity reject the complete manifest.

`InitialPlacementCatalogV1` is:

```text
InitialPlacementCatalogV1 {
  schema_version,
  partition_id,
  topology_revision,
  ordered_object_seeds[],
  ordered_cross_chunk_references[]
}
```

An object seed contains only the stable object/definition identity, authored
home region, cell/anchor, quantized local transform and immutable owner
reference-set hash needed to construct a generation-zero `Registered` record.
It has no lifecycle transition, placement revision or causal command.
Object seeds sort by `persistent_id`; references use the canonical
`CrossChunkReferenceV1` order.

`initial_placement_catalog_hash` is:

```text
SHA-256(
  "nextengine.initial-placement-catalog.v1\0" ||
  canonical_bytes(InitialPlacementCatalogV1)
)
```

It excludes `placement_generation`, tombstone history, moves, runtime mappings,
logical indexes and every post-creation command. The catalog initializes a new
world exactly once after complete manifest validation. It is a
compatibility/provenance binding, not the root of current World Services state;
loading an existing save MUST restore and validate its separate current
placement state and MUST NOT reapply or merge the seed catalog.

`coordinate_profile_id` resolves to an immutable numeric profile with
right-handed axes, metres and signed integer quantization. Every region/cell
uses closed inclusive-min/exclusive-max integer bounds. NaN, infinity,
platform-native float layout and implicit unit conversion are invalid.

Each `RegionDescriptorV1` declares one stable `region_id`, parent or root,
bounded child IDs, logical tags, anchor IDs and content/residency policy
references. Parent relation MUST form one acyclic forest rooted exactly in
`root_region_ids`. Region overlap MAY be authored only when an explicit
deterministic membership priority is declared.

Each `PartitionCellDescriptorV1` declares stable `cell_id`, containing region,
quantized bounds, neighbor IDs, required and optional chunk IDs, placement
shard ID and minimum residency prerequisites. Every referenced cell, anchor,
chunk, schema and asset MUST resolve in the exact locked closure.

`WorldAnchorDescriptorV1` identifies a logical anchor by nominal ID, region,
cell, quantized transform, capability tags and revision. An anchor is a
location/fact, not a runtime entity, scene-node pointer or physical pose owner.

### Chunk bindings and dependency admission

A `ChunkBindingV1` binds one logical chunk ID to exact content/bundle hashes,
schema set, required/optional dependencies, owning cells and target-neutral
payload. Target-specific presentation variants are selected by SPEC-24 before
staging and never alter the domain subset.

Required dependencies form an acyclic graph. Strong dependency means that the
complete group admits or rejects atomically. Weak presentation dependency MAY
use only its manifest-declared fallback. Reference-only dependency does not
authorize load or mutation. A cycle, missing required hash, target mismatch or
ambiguous dependency kind rejects staging before publication.

The SPEC-03 lifecycle remains canonical:

`Absent → Requested → Staged → Validated → Active → Quiescing → Unloaded`

with `Failed` terminal for the exact revision. SPEC-23 jobs may fetch,
decompress and validate immutable inputs concurrently, but Runtime consumes
their results only at the deterministic commit point described below.

## Deterministic residency interest and admission

`ResidencyInterestV1` contains:

```text
ResidencyInterestV1 {
  interest_id,
  source_kind,
  source_id,
  target_cell_id,
  requested_minimum_tier,
  mandatory,
  authored_priority,
  quantized_distance_bucket,
  activation_world_tick,
  expiry_world_tick,
  expected_topology_revision
}
```

`source_kind` is a closed engine enum. Presentation interest MAY request
content, but `mandatory = true` is valid only for an accepted gameplay,
population, transition, scenario or recovery source. Renderer visibility cannot
be upgraded into authoritative interest.

For one closed admission batch, interests sort by:

```text
(
  mandatory first,
  requested_minimum_tier descending,
  authored_priority descending,
  quantized_distance_bucket ascending,
  activation_world_tick ascending,
  source_kind,
  source_id,
  target_cell_id,
  interest_id
)
```

Every field is canonical integer/nominal data. Ties are impossible after
`interest_id`; duplicates with unequal bytes are identity collisions.

`StreamingAdmissionPlanV1` records exact topology/content/resource-policy
revisions, sorted interests, dependency groups, current pins/leases, accepted
groups, deterministically deferred groups, required evictions, expected
resident generation and plan hash. The planner MUST:

1. reject stale topology/content/resource revisions before reservation;
2. expand each required dependency group before budget comparison;
3. walk the canonical rank once, admitting a whole group only when every
   required schema, content and resource constraint fits;
4. evict only an eligible lower-ranked unpinned group in canonical rank order;
5. record every bounded defer with positive next retry tick and reason;
6. publish no residency change until Runtime revalidates expected revisions at
   a declared simulation commit point.

Equal closed inputs MUST produce byte-identical plans. Worker count, I/O order,
cache warmth and measured duration MAY affect readiness/performance telemetry,
but never rank, eviction choice, target tick or authoritative plan bytes.

## Persistent spatial objects

`PersistentSpatialObjectV1` contains:

```text
PersistentSpatialObjectV1 {
  schema_version,
  persistent_id,
  definition_id,
  definition_hash,
  home_region_id,
  placement_cell_id,
  placement_anchor_id,
  quantized_local_transform,
  placement_revision,
  lifecycle,
  owner_reference_set_hash,
  last_causal_command_id
}
```

`lifecycle` is `Registered` or `Tombstoned`. Tombstoning is an explicit
validated production command and preserves identity/causal history. Chunk
unload, Runtime despawn, tier downgrade and missing presentation asset MUST NOT
create a tombstone.

The placement record owns only logical durable placement. It cannot contain
mutable RPG fields, Agent state, physical pose/contact, runtime mapping or
presentation state. `owner_reference_set_hash` covers canonical immutable
owner IDs/revisions used for cross-segment closure; it is not a copied owner
payload.

`CrossChunkReferenceV1` contains source `PersistentId`, target `PersistentId`,
closed reference kind, required/optional status, expected target definition
hash and fallback. It never contains a pointer or runtime entity. A required
reference resolves before activation of its atomic dependency group. An
optional unresolved reference becomes an explicit `UnresolvedOptional`
projection and MAY use only the declared bounded fallback.

### Mutable placement/tombstone state root

Current World Services placement is published as a complete immutable
generation:

```text
WorldPlacementStateV1 {
  schema_version,
  partition_id,
  topology_revision,
  initial_placement_catalog_hash,
  placement_generation,
  prior_placement_state_root,
  ordered_object_records[],
  ordered_tombstone_history[],
  ordered_cross_chunk_references[],
  logical_membership_index_root,
  last_committed_command_id
}
```

`ordered_object_records` contains the current
`PersistentSpatialObjectV1` records sorted by `persistent_id`.
`ordered_tombstone_history` entries contain exact `persistent_id`,
definition hash, placement revision and causal command ID and sort by that
tuple. Cross-chunk references sort by
`(source PersistentId, target PersistentId, reference kind)`. The logical
membership index is a canonical derived projection of the same current object
records: every `Registered` object maps to exactly one declared membership and
every `Tombstoned` object maps to an explicit non-member marker. It cannot
introduce, omit or silently reactivate an identity.

`SpatialPlacementStateRootV1` is:

```text
SHA-256(
  "nextengine.spatial-placement-state.v1\0" ||
  canonical_bytes(WorldPlacementStateV1)
)
```

The current root is stored by the enclosing save/replay/commit receipt and is
not embedded in `WorldPlacementStateV1`. Generation zero has no prior root and
is deterministically materialized from the validated
`InitialPlacementCatalogV1`: `placement_generation = 0`, every seed becomes one
`Registered` record with `placement_revision = 0`, tombstone history is empty,
references equal the catalog references and the membership index is derived
from those records. Prior-root and last-command fields use their canonical
tagged `None` encoding. Every accepted register, move, reference or tombstone
command MUST declare the expected current state root, placement generation and
affected object revisions. World Services stages the complete next state,
validates object/reference/index closure, increments `placement_generation`
exactly once, binds `prior_placement_state_root` to the accepted old root, sets
`last_committed_command_id` to the accepted command and atomically publishes
the new state root with all affected indexes and admitted Runtime mappings. The
commit receipt records transition kind, command ID, before/after generation and
before/after root. Rejection or crash before that publication preserves the
complete old generation and root; retry uses the normal command-ledger receipt
and returns the same transition receipt rather than creating another
transition.

Chunk unload, Runtime despawn, tier downgrade, cache eviction and presentation
loss do not change `SpatialPlacementStateRootV1`. Activating a different
partition manifest or `initial_placement_catalog_hash` over an existing world
requires an explicit revision-checked copy-on-write placement migration. Its
input is the exact old state root; its output binds the target topology and
seed hash, retains the old root as `prior_placement_state_root` and increments
the generation once. A target seed is never implicitly replayed over current
state. Any collision, missing mapping, stale precondition or publication fault
retains the old manifest/state/save generation.

Moving a durable object is a revision-checked production command. Commit
atomically updates placement, affected region/cell indexes and any admitted
Runtime mapping. Failure retains the entire prior placement/index/runtime
generation. A content chunk cannot re-register an existing `PersistentId` or
overwrite a later durable placement.

## `TierExecutionProfile`

`TierExecutionProfile` is an immutable project-locked descriptor for one
`WorldResidencyTier`:

```text
TierExecutionProfile {
  profile_id,
  tier,
  owner_work_cadence_ticks,
  allowed_activity_classes[],
  required_content_classes[],
  maximum_operations_per_boundary,
  maximum_deferral_world_ticks,
  budget_profile_hash,
  abstract_equivalence_profile_hash,
  fallback_tier
}
```

It does not own a population record, physical LOD, job scheduler or RPG/Agent
state. Cadence changes only when an already-due owner evaluation runs; it
cannot discard a mandatory boundary.

`AbstractOutcomeEquivalenceProfileV1` classifies each activity:

- `ExactAbstract`: declared abstract evaluator must produce the exact same
  mandatory command/event/owner result as the sufficient resident path;
- `RequiresTier`: no abstract result is legal; request the minimum sufficient
  tier and retain the pending outcome;
- `DeferOnly`: retain activity/cursor and use the positive bounded retry from
  SPEC-20/ADR-021;
- `PresentationOnly`: may degrade or disappear without domain mutation.

There is no “approximate authoritative success” class. When required content,
budget or capability is unavailable, the evaluator follows SPEC-20's exact
upgrade/defer matrix and stops at `WORLD_ABSTRACT_OUTCOME_BLOCKED` when the
maximum deferral boundary is reached. A fallback activity is independently
validated and never impersonates completion of the original activity.

## Save, replay and recovery

The World Services save segment stores the exact immutable partition manifest
hash and `initial_placement_catalog_hash` as compatibility bindings, canonical
`WorldPlacementStateV1` bytes and their current
`SpatialPlacementStateRootV1` and the last placement transition receipt, plus
population tier/profile references, interest sources that survive restart and
pending deterministic deferrals. Immutable bundle payload remains in content
storage; ephemeral Runtime mappings, task state and caches are not serialized.

Load MUST validate:

- project, schema registry, content catalog and partition hashes;
- the immutable initial-catalog binding separately from the recomputed current
  `SpatialPlacementStateRootV1`;
- topology forest, bounds, chunk closure and anchor membership;
- unique durable `PersistentId` plus tombstone history;
- every required cross-chunk/cross-segment reference;
- placement generation, current/prior-root and last-transition-receipt
  consistency, logical membership index,
  population/owner revisions and tier prerequisites;
- resource policy compatibility before any world/entity publication.

Migration and repair operate on a complete copy and publish one new generation
only after all owner-segment and content-reference checks pass. The source save
remains immutable.

Replay records closed interest/admission batches, topology/content/resource
revisions, immutable initial-catalog hash, admitted/deferred/evicted group IDs,
and every placement transition's command ID, prior/current generation and
before/after `SpatialPlacementStateRootV1`. It also records tier upgrade/defer
outcomes and commit boundaries. Replay recomputes each state root before
publication; it never substitutes the manifest seed hash for the current root.
First divergence reports the runtime tick, world tick, plan hash,
cell/chunk/object ID and first differing rank/dependency/placement root/owner
revision. Cache warmth and raw I/O timings are diagnostic only.

The normative placement-root corpus MUST cover byte-identical generation-zero
roots across seed declaration permutations; register, move, reference,
tombstone, save/load and replay before/after vectors; stale expected roots;
tampered object, tombstone, reference, logical-index and prior-root fields;
changed initial catalog without an explicit migration; accidental use of
current state bytes as the manifest seed catalog; and a crash at every atomic
publication boundary. The same vectors MUST produce exact roots in `game`,
`headless` and `capture-worker`. Every integrity mismatch rejects before world
publication with `WORLD_PLACEMENT_ROOT_MISMATCH`; a mismatch is never repaired
by reseeding, unordered reconstruction or a repeated run with identical inputs.

## Stable diagnostics and failure semantics

| Code | Required result |
|---|---|
| `WORLD_TOPOLOGY_INVALID` | Reject the complete manifest before registry/world publication; preserve previous topology. |
| `WORLD_PARTITION_REFERENCE_INVALID` | Reject the atomic dependency group; optional reference uses only its declared fallback. |
| `WORLD_OBJECT_ID_COLLISION` | Fail closed; quarantine conflicting content/save and never choose a replacement ID. |
| `WORLD_PLACEMENT_STALE` | Reject the complete move/placement transaction and preserve prior indexes/mapping. |
| `WORLD_PLACEMENT_ROOT_MISMATCH` | Reject save, replay, command or migration before publication when the seed binding, recomputed current root, prior-root chain or logical index differs; preserve the exact prior manifest/state/save generation and never reseed or infer a repair. |
| `WORLD_STREAM_PLAN_STALE` | Discard the complete plan and rebuild from current immutable revisions. |
| `WORLD_STREAM_REQUIRED_UNAVAILABLE` | Retain prior active generation and deterministically defer or fail required activation. |
| `WORLD_STREAM_BUDGET_BLOCKED` | Record the exact ranked defer; never drop mandatory work or evict a pin/lease. |
| `WORLD_STREAM_PUBLICATION_ABORTED` | Discard staging; no partial chunk/object/registry visibility. |
| `WORLD_ABSTRACT_OUTCOME_BLOCKED` | Retain tier/activity/cursor and stop before the unsupported owner outcome. |
| `NONDETERMINISTIC_RESULT` | Stop at the first rank, admission, object or outcome divergence and preserve the prior state. |

## Product checks

| ID | Scenario / command | Expected behavior | Fallback |
|---|---|---|---|
| `WORLD-TOPOLOGY-P1` | `next check world-topology --permutations 1000` | Declaration, order and target permutations produce one exact manifest/topology/initial-placement-catalog root set; cycle, bounds, overlap-priority, anchor, dependency, seed-catalog and hash faults reject before publication. | Reject the topology and retain the previous manifest. |
| `WORLD-OBJECT-P1` | `next check persistent-spatial-objects --cycles 10000 --faults all` | Load, unload, register, move, reference, tombstone, save, replay and migration cycles retain one durable ID, unchanged seed binding and exact placement roots; every fault publishes no partial index, mapping or owner state. | Retain the prior placement root, index and save generation. |
| `WORLD-ABSTRACT-P1` | `next check tier-execution-equivalence --profiles all --repeats 100` | Every tier/activity class produces the exact mandatory owner outcome or exact upgrade/defer result; no fabricated outcome, cursor advance, wall-time/visibility authority or physical-LOD conflation. | Pin a sufficient safe tier or retain the deferred activity. |
| `WORLD-STREAM-P1` | `next check streaming-admission --cycles 10000 --io-permutations all` | Worker, cache and I/O permutations produce exact rank, admission, eviction and commit roots; no authoritative drop, protected eviction, duplicate ID, dangling required reference or partial publication; ready groups commit within two gameplay ticks. | Discard staging, retain the active generation and deterministically defer ranked work. |

## Technical requirements

| ID | Technical requirement |
|---|---|
| REQ-124 | `WorldPartitionManifestV1` MUST define one immutable validated topology of regions, cells, anchors and chunk bindings with canonical units, hashes and complete dependency closure. |
| REQ-125 | Residency interests, dependency expansion, resource admission, eviction and activation MUST use one canonical rank and deterministic commit plan independent of wall time, visibility, worker count, cache warmth and I/O order. |
| REQ-126 | Durable spatial placement, tombstones and cross-chunk references MUST retain one `PersistentId`, remain separate from foreign-owner state and publish atomically across load/unload/move/save/replay. |
| REQ-127 | Every `TierExecutionProfile` MUST preserve mandatory outcomes exactly or produce the declared sufficient-tier upgrade/bounded defer; streaming or physical LOD MUST NOT fabricate an abstract outcome. |

## Failure paths

| ID | Trigger | Required result |
|---|---|---|
| FAIL-050 | Invalid/cyclic topology, bounds/anchor/dependency error, unresolved required cross-chunk reference, duplicate `PersistentId` or stale placement | Reject before publication, preserve the previous topology/placement/save generation and never guess identity, membership or reference target. |
| FAIL-051 | Stale admission plan, unavailable required content/resource, cancellation/backpressure/publication fault or unsupported abstract outcome | Discard staging or retain the exact prior active/tier/activity/cursor state; deterministically defer, pin or block with no authoritative drop, partial publication or fabricated owner outcome. |

## Technology neutrality

This contract chooses no ECS, job system, world-streaming library, database,
navigation/crowd backend, renderer, physics backend, archive implementation or
platform API. Replaceable implementations remain private caches/adapters behind
these engine-owned schemas, authority rules, deterministic admission contract
and exact algorithms.
