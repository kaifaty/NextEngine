# SPEC-03: Assets, world streaming и persistence

| Поле | Значение |
|---|---|
| ID | SPEC-03 |
| Статус | Accepted |
| Версия | 1.8 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md) |
| Заменяет | отсутствует |

## Source of truth и ownership

До cooking source files и validated `ProjectManifest` являются authoring source of truth. После atomic publish `ProjectCompositionLock` + `SchemaRegistryManifestV1` + `ContentManifestV1` + `WorldPartitionManifestV1` + immutable content-addressed bundles являются единственным runtime project/schema/asset/world/model source. Save state владеет только mutable progression/deltas, durable placement/tombstones, selected model/route references и declared PolicyState; оно не копирует immutable model weights или asset payload. Asset & Persistence Team владеет schema/content publication, cooker, low-level streaming state machine, atomic lock publication, save transactions и migration execution; semantic schema compatibility задаёт SPEC-22/ADR-025, resource admission — SPEC-23/ADR-026, durable topology/placement — SPEC-25.

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
- Public/policy/evidence manifests MUST использовать RFC 8785 JCS, а authoritative numeric/state segments — `CanonicalBinaryV1` из ADR-022. Canonicalization MUST нормализовать ordering, semantic negative zero, paths/case и NFC strings; duplicate keys, NaN/infinity и case-fold path collisions rejected.
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

Migration uses the unique schema-registry DAG and pure ordered transforms over a complete copy. Runtime-supported direct load is `N`/`N-1`; admitted copy-on-write migration reaches `N` from `N-2` only through the unique validated path defined by SPEC-22. Unknown required segment/field, checksum mismatch, missing content revision, ambiguous path or failed transform → fail-closed with original generation byte-identical. Legacy RPG-owned calendar fields переносятся в `WorldCalendarStateV1` одной atomic copy-on-write transaction: новый RPG segment без calendar fields и новый World Services segment публикуются только вместе после cross-segment validation. Любой duplicate/missing/conflicting calendar field, unsupported mapping, hash/revision mismatch или injected publication failure сохраняет original save bytes и не публикует ни один новый segment. User получает stable code, affected schema/segment и recovery choices.

## Replay

`ReplayManifest` фиксирует initial save/snapshot, exact `ProjectCompositionLock`, `WorldIdentityManifestV1`, named RNG streams, `RuntimeDeterminismProfileV1`, content/build/schema/physical-archetype/model/policy-route/plugin/MechanicsLock hashes, world-level `CommandLedgerV2` snapshot/root с каждым `CommandStreamLedgerV2`, все `ClosedIngressBatchV1` и все `ClosedCommandAdmissionBatchV2`. Каждый ingress batch сохраняет bounded decoded input/completion records, exact assignments и исходную batch boundary. Каждый command-admission batch сохраняет все bounded/authenticated/decodable external `WorldCommandEnvelopeV2`, включая exact duplicates, conflicting bodies, invalid claims, deterministic rejections и finalized retries, с `simulation_tick`/`phase`/`batch_ordinal`, canonical candidate order и исходной batch boundary. Envelope transport metadata заменяется `None`; `CanonicalCommandBodyV2` не содержит собственного ID или transport metadata. Malformed/unauthenticated transport bytes не являются gameplay replay input и MAY сохраняться только отдельно как audit raw hash/structured diagnostic. Runtime-generated `Outcome` повторно выводится production systems; expected receipts/events, включая ActivePolicyRouteChanged, MAY сохраняться как oracle. Replay runner MUST сравнивать ADR-022 per-tick state root, exact command/rejection/receipt/event/outcome, RPG aggregate revision, population/calendar cursor outcomes и указывать first divergence. Cross-target tolerance не применяется к IDs, commands, events, manifests или gameplay outcomes.

## Public boundaries

Public schemas: NeutralAuthoringModel, `ProjectCompositionLock`, `SchemaRegistryManifestV1`, `ContentManifestV1`, neutral bundle/variant descriptors, `WorldPartitionManifestV1`, WorldChunk, durable spatial-object references, MechanicPackage/MechanicsLock references, physical archetype/policy/skill references, SaveManifest, ReplayManifest, TestScenario fixture references и typed diagnostics. Filesystem paths, package-manager state, worker/task/allocator state, training sessions/optimizer state, archive library handles, ECS IDs и importer legacy types не входят в contracts. Asset access в runtime — `AssetId + resolved revision`, не raw path.

Scenario fixtures MAY быть cooked content bundles с neutral/generated assets и exact provenance. Evidence/media artifacts не являются runtime assets, хранятся под explicit external artifact root и ссылаются на runtime content только по immutable hash. Approved baseline не может неявно изменить ContentManifest или save.

## Failure semantics

- Corrupt/missing required bundle → chunk не активируется; если critical startup chunk — чистый отказ запуска до world mutation.
- Optional asset missing → declared placeholder/fallback с diagnostic, если schema разрешает optional.
- I/O cancellation → staging discarded; current active revision остаётся.
- Save corruption/incompatibility → исходник не меняется, partial state не создаётся.
- Missing required mechanic package/state migration → save не открывается; original сохраняется, пользователь получает exact lock/migration diagnostic.
- Missing required physical archetype/model/PolicyState schema → save не открывается до world mutation; explicit predeclared downgrade выполняется только как copy-on-write migration с новым manifest hash.
- Streaming budget overrun → throttle/preempt optional requests; authoritative active objects не выгружаются без quiesce.
- Schema/compatibility/migration ambiguity → reject registry/load/migration before publication and retain exact prior registry/save generation.
- Queue, memory, archive/decompression or I/O limit → deterministic bounded defer/reject/quarantine according to SPEC-23; required authoritative work is never silently dropped.
- Invalid content variant, partition topology, durable placement or cross-chunk reference → reject the complete dependency/admission group and retain prior active world generation.
- Hash collision evidence → security fatal; bundle quarantined.
- Evidence/media bytes обнаружены в runtime/source bundle без explicit content role → validation failure; artifact остаётся во внешнем store.

## Verification gates

| Gate | Сценарий | Threshold | Evidence | Fallback/rollback |
|---|---|---|---|---|
| ASSET-01 | cook same fixture twice на Win/Linux | platform-neutral artifacts byte-identical по `CANON-01`; cache second run ≥90% hits | manifests, raw-byte hashes, timing JSON | blocking canonicalization fix |
| ASSET-02 | malformed/cyclic/missing deps corpus | 100% classified diagnostics; 0 partial publish | validator report | quarantine input |
| STREAM-01 | 10 000 randomized load/unload cycles with cancellation, decompression and commit-point fault injection | Publication occurs only at a declared deterministic simulation commit point; 0 duplicate IDs, leaks, dangling active refs or partial authoritative publication; p99 commit ≤2 gameplay ticks after I/O ready | staging/commit trace, fault corpus, memory and resolver report | reject/discard staged result, retain previous active generation and reduce concurrent requests |
| SAVE-01 | power-failure injection at every write boundary | одна из двух валидных generations загружается в 100% points | fault matrix | retain prior generation |
| SAVE-02 | migration fixtures N-2,N-1,N + corrupt cases | valid fixtures exact expected hash; invalid 100% fail-closed | migration report | require older executable/export tool |
| REPLAY-01 | 100 vertical fixtures + cutoff/admission negative batch corpus | 100% `ClosedIngressBatchV1` и `ClosedCommandAdmissionBatchV2` hashes/boundaries/assignments exact; command batches сохраняют every bounded/authenticated/decodable duplicate/conflict/invalid-claim/rejection/finalized-retry envelope, malformed/unauthenticated transport остаётся audit-only; exact computed IDs/full body hashes, `CommandLedgerV2` root, `CommandStreamLedgerV2` reservations/receipts/rejections/events/outcomes; state roots exact на same target triple, first divergence stable-coded | replay report, closed-batch manifests/hashes, assignment traces, transport audit, ledger snapshots/receipt roots | release blocking |
| CONTRACT-01 | importer/editor/game/headless schema compatibility | 100% same schema registry; 0 source parser link in runtime | SBOM/link map/schema report | blocking package split |
