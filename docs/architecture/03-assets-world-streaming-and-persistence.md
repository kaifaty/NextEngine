# SPEC-03: Assets, world streaming и persistence

| Поле | Значение |
|---|---|
| ID | SPEC-03 |
| Статус | Accepted |
| Версия | 1.11 |
| Последняя проверка | 2026-07-26 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md) |
| Заменяет | отсутствует |

## Source of truth и ownership

До cooking source files и validated `ProjectManifest` являются authoring source of truth. После atomic publish `ProjectCompositionLock` + `SchemaRegistryManifestV1` + `ContentManifestV1` + `WorldPartitionManifestV1` + immutable content-addressed bundles являются единственным runtime project/schema/asset/world/model source. Save state владеет только mutable progression/deltas, durable placement/tombstones, selected model/route references и declared PolicyState; оно не копирует immutable model weights или asset payload. Asset & Persistence subsystem владеет schema/content publication, cooker, low-level streaming state machine, atomic lock publication, save transactions и migration execution; semantic schema compatibility задаёт SPEC-22/ADR-025, resource admission — SPEC-23/ADR-026, durable topology/placement — SPEC-25.

## Public boundary и data flow

Importer, будущий editor и first-party authoring tools MUST выдавать public `NeutralAuthoringModel`. Validator и cooker MUST быть общими для tools CI и local use. `game` и `headless` MUST читать один cooked schema и не иметь source-format parsers. Boundary data flow полностью однонаправлен до runtime; mutable save deltas никогда не возвращаются в source/cooked asset как неявная authoring запись.

```text
source files / NeutralImportModel
        ↓ adapters with provenance
NeutralAuthoringModel
        ↓ canonical validation
ValidatedAuthoringModel
        ↓ deterministic cooker
SchemaRegistryManifestV1 + ContentManifestV1 + immutable bundles
        ↓ WorldPartitionManifestV1 bindings
WorldChunks
        ↓ verified streaming
game / headless / inspectors
```

## NeutralAuthoringModel

Каждый record MUST содержать immutable schema ID/version и field IDs из `SchemaRegistryManifestV1`, AssetId или PersistentId согласно роли, typed properties, declared dependency kinds, source provenance reference и validation location. Model MUST использовать generic engine concepts и canonical units: metres, kilograms, seconds, radians, right-handed coordinates, UTF-8 NFC strings. Physical archetype/skill/policy records MUST ссылаться на contracts SPEC-14 и exact model hashes, а не tool-specific training objects. Source-specific metadata MAY сохраняться только в namespaced opaque diagnostic extension, удаляемом cooker перед runtime bundle. Scene/mesh/material/texture/skeleton/animation/audio/collision/navigation/world-chunk records используют exact neutral schemas SPEC-24.

## Cooking и content addressing

- Cooker input включает exact source hashes, importer/tool versions, schema versions, target profile и deterministic options.
- Public and policy manifests MUST использовать RFC 8785 JCS, а authoritative numeric/state segments — `CanonicalBinaryV1` из ADR-022. Canonicalization MUST нормализовать ordering, semantic negative zero, paths/case и NFC strings; duplicate keys, NaN/infinity и case-fold path collisions rejected.
- Bundle key MUST включать SHA-256 canonical payload. Published bundle immutable; изменение создаёт новый hash.
- Publish MUST быть atomic: staging → validate schema registry, compatibility,
  content/dependency/variant hashes, provenance/license and resource bounds →
  rename/index commit.
- Equal canonical inputs на Windows/Linux MUST давать byte-identical platform-neutral bundles. Platform-specific GPU/audio payload MAY различаться, но имеет отдельный target key и deterministic hash.

## World chunks и streaming

`WorldChunk` является neutral content payload SPEC-24, bound через `WorldPartitionManifestV1` к cells/regions/anchors SPEC-25. Он содержит bounds, logical coordinates, schema version, typed required/optional dependencies, persistent namespace, definition records и content hashes; durable placement/tombstone state не копируется в immutable chunk. Runtime state machine: `Absent → Requested → Staged → Validated → Active → Quiescing → Unloaded`, с `Failed` как terminal для конкретной revision.

Chunk становится `Active` только на simulation commit point после полной проверки schema/content/variant/resource/partition revisions, required dependencies и duplicate PersistentId. SPEC-23 jobs return immutable staged results; SPEC-25 canonical interest/admission plan determines whole dependency groups and commit order. Cross-chunk reference использует PersistentId и resolver state; direct pointer/RuntimeEntityId в bundle запрещён. Unload MUST сохранить dirty durable state либо явно доказать его transient nature до despawn. Cancellation, backpressure or I/O order cannot partially publish a chunk or drop mandatory work.

## Persistence

`SaveManifest` MUST содержать engine/game build identity, project ID, exact `ProjectCompositionLock` hash, `SchemaRegistryManifestV1`, `ContentManifestV1`, resource-policy and `WorldPartitionManifestV1` hashes, exact MechanicsLock hash/package-state schemas, tick/time settings, loaded chunk revisions, `WorldIdentityManifestV1`, `RuntimeDeterminismProfileV1`, named RNG stream states, physical archetype/model catalog/active route hashes, PolicyState schemas, plugin/script hashes, полный world-level `CommandLedgerV2` с каждым `CommandStreamLedgerV2` (high-watermark/state/reservations/4096-receipt window/count/root) и ordered segment table. Каждая command reservation и каждый ordinary receipt сохраняют полный body hash. Каждый segment имеет ровно одного owner, schema/version, byte length и ADR-022 domain-separated SHA-256. RPG aggregates/revisions находятся в RPG segment; `WorldCalendarStateV1`, population/schedule cursors, durable placements/tombstones and abstract activities — в World Services segment; transition/PolicyState — в Motor Runtime segment; immutable bundle/model payloads, worker queues and cache state в save запрещены.

Save procedure: freeze logical commit point → snapshot owner segments → write new staging save → fsync files/manifest where platform permits → validate → atomic pointer update. Autosave MUST NOT перезаписывать единственную известную валидную generation.

Migration uses the unique schema-registry DAG and pure ordered transforms over a complete copy. Runtime-supported direct load is `N`/`N-1`; validated copy-on-write migration reaches `N` from `N-2` only through the unique path defined by SPEC-22. Unknown required segment/field, checksum mismatch, missing content revision, ambiguous path or failed transform → fail-closed with original generation byte-identical. Legacy RPG-owned calendar fields переносятся в `WorldCalendarStateV1` одной atomic copy-on-write transaction: новый RPG segment без calendar fields и новый World Services segment публикуются только вместе после cross-segment validation. Любой duplicate/missing/conflicting calendar field, unsupported mapping, hash/revision mismatch или injected publication failure сохраняет original save bytes и не публикует ни один новый segment. User получает stable code, affected schema/segment и recovery choices.

## Replay

`ReplayManifest` фиксирует initial save/snapshot, exact `ProjectCompositionLock`, `WorldIdentityManifestV1`, named RNG streams, `RuntimeDeterminismProfileV1`, content/build/schema/physical-archetype/model/policy-route/plugin/MechanicsLock hashes, world-level `CommandLedgerV2` snapshot/root с каждым `CommandStreamLedgerV2`, все `ClosedIngressBatchV1` и все `ClosedCommandAdmissionBatchV2`. Каждый ingress batch сохраняет bounded decoded input/completion records, exact assignments и исходную batch boundary. Каждый command-admission batch сохраняет все bounded/authenticated/decodable external `WorldCommandEnvelopeV2`, включая exact duplicates, conflicting bodies, invalid claims, deterministic rejections и finalized retries, с `simulation_tick`/`phase`/`batch_ordinal`, canonical candidate order и исходной batch boundary. Envelope transport metadata заменяется `None`; `CanonicalCommandBodyV2` не содержит собственного ID или transport metadata. Malformed/unauthenticated transport bytes не являются gameplay replay input и MAY сохраняться только отдельно как raw hash/structured diagnostic. Runtime-generated `Outcome` повторно выводится production systems; expected receipts/events, включая ActivePolicyRouteChanged, MAY сохраняться как oracle. Replay runner MUST сравнивать ADR-022 per-tick state root, exact command/rejection/receipt/event/outcome, RPG aggregate revision, population/calendar cursor outcomes и указывать first divergence. Cross-target tolerance не применяется к IDs, commands, events, manifests или gameplay outcomes.

V1 replay является read-only re-execution: runner rehydrates initial checkpoint,
проверяет его state root, принимает только записанные authoritative inputs и
останавливается на первом divergence. Подмена input, `branch()` и
counterfactual continuation не входят в v1 replay API.

## Public boundaries

Public schemas: NeutralAuthoringModel, `ProjectCompositionLock`, `SchemaRegistryManifestV1`, `ContentManifestV1`, neutral bundle/variant descriptors, `WorldPartitionManifestV1`, WorldChunk, durable spatial-object references, MechanicPackage/MechanicsLock references, physical archetype/policy/skill references, SaveManifest, ReplayManifest, TestScenario fixture references и typed diagnostics. Filesystem paths, package-manager state, worker/task/allocator state, training sessions/optimizer state, archive library handles, ECS IDs и importer legacy types не входят в contracts. Asset access в runtime — `AssetId + resolved revision`, не raw path.

Scenario fixtures MAY быть cooked content bundles с neutral/generated assets и exact provenance. They use the same content validation and cannot implicitly change `ContentManifestV1` or save state.

## Failure paths

| ID | Trigger | Required result |
|---|---|---|
| `ASSET_REQUIRED_MISSING` | Corrupt or missing required bundle | Do not activate the chunk; a critical startup chunk fails before world mutation. |
| `ASSET_OPTIONAL_MISSING` | Missing optional asset | Use only the schema-declared placeholder/fallback and emit a diagnostic. |
| `ASSET_IO_CANCELLED` | I/O cancellation | Discard staging and retain the current active revision. |
| `SAVE_INCOMPATIBLE` | Save corruption, checksum mismatch or incompatibility | Preserve source bytes and publish no partial state. |
| `SAVE_REQUIRED_PACKAGE_MISSING` | Missing required mechanic package, archetype, model, `PolicyState` schema or migration | Reject before world mutation; preserve the original generation. A declared downgrade runs only as a new copy-on-write generation. |
| `STREAM_BUDGET_BLOCKED` | Streaming budget, queue, memory, archive/decompression or I/O limit | Defer, reject or quarantine according to SPEC-23; never silently drop authoritative work or unload active authority without quiesce. |
| `SCHEMA_COMPATIBILITY_AMBIGUOUS` | Schema, compatibility or migration ambiguity | Reject registry/load/migration before publication and retain the prior registry/save generation. |
| `WORLD_STREAM_INPUT_INVALID` | Invalid content variant, topology, placement or cross-chunk reference | Reject the complete dependency group and retain the prior active world generation. |
| `CONTENT_HASH_COLLISION` | Same logical hash resolves to different bytes | Quarantine the bundle and stop activation before mutation. |

## Product checks

| ID | Scenario / command | Expected behavior | Fallback |
|---|---|---|---|
| ASSET-01 | Cook the same fixture twice on Windows and Linux | Platform-neutral artifacts are byte-identical under `CANON-01`; second-run cache hit rate is at least 90%. | Reject noncanonical cooker output. |
| ASSET-02 | Malformed, cyclic and missing-dependency corpus | Every case returns the expected diagnostic and no partial publication occurs. | Quarantine the input. |
| STREAM-01 | 10 000 randomized load/unload cycles with cancellation, decompression and commit-point faults | Publication occurs only at a deterministic simulation commit point; no duplicate IDs, leaks, dangling active refs or partial authoritative publication; ready results commit within two gameplay ticks. | Discard staging, retain the active generation and reduce concurrent requests. |
| SAVE-01 | Inject power failure at every save write boundary | One complete valid generation loads at every fault point. | Retain the prior generation. |
| SAVE-02 | Migrate N-2, N-1 and N fixtures plus corrupt cases | Valid fixtures produce the expected exact hash; invalid cases fail closed with source bytes unchanged. | Use a compatible older executable or explicit export. |
| REPLAY-01 | 100 deterministic fixtures plus cutoff and command-admission negative batches | Closed ingress/command batches, assignments, IDs, ledger state, receipts, events, outcomes and state roots are exact; first divergence is stable-coded. | Stop replay at the first divergence. |
| CONTRACT-01 | Importer, tools, game and headless schema compatibility | All roots use the same schema registry and runtime links no source-format parser. | Keep source parsing in a separate adapter/process. |

## Narrative and divine persistence extension

For projects using [SPEC-31](31-autonomous-quest-lifecycle-and-narrative-director.md)
and [ADR-029](adr/029-rpg-owned-quest-graph-and-optional-narrative-director.md),
save generation дополнительно хранит current
`QuestGraphRevisionV1`, generated definitions, causal lineage, pending director
request/boundary metadata и hashes принятых canonical candidates. Эти данные принадлежат
конкретному world/save и MUST NOT публиковаться обратно в authored
assets/packages без отдельного будущего authoring workflow.

RPG save segment также сохраняет opportunity-admission source revisions,
disclosure history/channel, offer cooldown, recognized prior-fact IDs, selected
term variant и frozen `ChallengeAssessmentV1`/`QuestRewardContractV1` revisions.
После load уже допущенный или раскрытый Quest не восстанавливается повторным
planner/dialogue/LLM запросом.

Replay и save/restart используют записанный candidate/result bytes и никогда не
вызывают `ai-host`, сеть или модель для восстановления уже произошедшего.
Unknown schema/policy/hash, corrupt generated definition или incomplete causal
closure отклоняет новую load/migration generation до publication и сохраняет
предыдущую generation.

The RPG segment additionally stores `DivineStandingV1` aggregates, open and
terminal `DivineOfferV1` records, covenant/vow/warning history, intervention
cooldowns and committed judgment lineage. Save closure binds the exact
byte-ordered world-locked patron set,
`DivinePatronDefinitionV1`, `DivineEpistemicPolicyV1`,
`PantheonRelationGraphV1`, standing-band/intervention/conflict-resolver hashes,
open/consumed divine hooks, pending `DivineJudgmentBatchBaseV1`, per-god request
metadata, `NarrativeDecisionBoundaryV1`, the boundary-closing ingress generation/
`SimulationTick` and selected candidate/fallback/resolution hashes.

A save taken with only some per-god completions assigned stores those exact
assignments and the still-pending patrons; restart cannot ask already assigned
patrons again or commit a partial standing vector. At the recorded boundary,
missing patrons select their deterministic fallback. Replay injects the exact
recorded divine completion bytes through SPEC-21 and makes zero `ai-host`,
network or model calls.

Missing/corrupt patron, pantheon, candidate or resolution artifact, mismatched
base/standing revision or incomplete batch closure fails before world
publication and preserves the prior save generation. Runtime-generated divine
content remains world/save-local and cannot modify authored patron/pantheon
assets.

A schema migration may transform representation while preserving the exact
patron set, standing IDs/revisions and causal history. A different patron set or
pantheon graph hash is `DIVINE_PANTHEON_WORLD_MISMATCH` before publication;
recovery uses the original content closure or a new world, never inferred
add/remove/retirement.
