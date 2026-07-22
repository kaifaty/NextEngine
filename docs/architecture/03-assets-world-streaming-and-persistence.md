# SPEC-03: Assets, world streaming и persistence

| Поле | Значение |
|---|---|
| ID | SPEC-03 |
| Статус | Accepted |
| Версия | 1.4 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-23 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-012](adr/012-deterministic-command-identity-and-replay.md) |
| Заменяет | отсутствует |

## Source of truth и ownership

До cooking source files являются authoring source of truth. После atomic publish `ContentManifest` + immutable content-addressed bundles являются единственным runtime asset/world/model source. Save state владеет только mutable progression/deltas, selected model/route references и declared PolicyState; оно не копирует immutable model weights или asset payload. Asset & Persistence Team владеет schemas, cooker, streaming state machine, save transactions и migrations.

## Public boundary и data flow

Importer, будущий editor и first-party authoring tools MUST выдавать public `NeutralAuthoringModel`. Validator и cooker MUST быть общими для tools CI и local use. `game` и `headless` MUST читать один cooked schema и не иметь source-format parsers. Boundary data flow полностью однонаправлен до runtime; mutable save deltas никогда не возвращаются в source/cooked asset как неявная authoring запись.

```text
source files / NeutralImportModel
        ↓ adapters with provenance
NeutralAuthoringModel
        ↓ canonical validation
ValidatedAuthoringModel
        ↓ deterministic cooker
ContentManifest + immutable bundles + WorldChunks
        ↓ verified streaming
game / headless / inspectors
```

## NeutralAuthoringModel

Каждый record MUST содержать schema ID/version, AssetId или PersistentId согласно роли, typed properties, declared dependencies, source provenance reference и validation location. Model MUST использовать generic engine concepts и canonical units: metres, kilograms, seconds, radians, right-handed coordinates, UTF-8 NFC strings. Physical archetype/skill/policy records MUST ссылаться на contracts SPEC-14 и exact model hashes, а не tool-specific training objects. Source-specific metadata MAY сохраняться только в namespaced opaque diagnostic extension, удаляемом cooker перед runtime bundle.

## Cooking и content addressing

- Cooker input включает exact source hashes, importer/tool versions, schema versions, target profile и deterministic options.
- Public/policy/evidence manifests MUST использовать RFC 8785 JCS, а authoritative numeric/state segments — `CanonicalBinaryV1` из ADR-012. Canonicalization MUST нормализовать ordering, semantic negative zero, paths/case и NFC strings; duplicate keys, NaN/infinity и case-fold path collisions rejected.
- Bundle key MUST включать SHA-256 canonical payload. Published bundle immutable; изменение создаёт новый hash.
- Publish MUST быть atomic: staging → validate hashes/dependencies/license manifest → rename/index commit.
- Equal canonical inputs на Windows/Linux MUST давать byte-identical platform-neutral bundles. Platform-specific GPU/audio payload MAY различаться, но имеет отдельный target key и deterministic hash.

## World chunks и streaming

`WorldChunk` содержит bounds, logical coordinates, schema version, required/optional dependencies, persistent namespace, spawn records и content hashes. Runtime state machine: `Absent → Requested → Staged → Validated → Active → Quiescing → Unloaded`, с `Failed` как terminal для конкретной revision.

Chunk становится `Active` только на simulation commit point после полной проверки required dependencies и duplicate PersistentId. Cross-chunk reference использует PersistentId и resolver state; direct pointer/RuntimeEntityId в bundle запрещён. Unload MUST сохранить dirty durable state либо явно доказать его transient nature до despawn.

## Persistence

`SaveManifest` MUST содержать engine/game build identity, project ID, schema registry hash, content manifest hash, exact MechanicsLock hash/package-state schemas, tick/time settings, loaded chunk revisions, RNG stream states, physical archetype/model catalog/active route hashes, PolicyState schemas, plugin/script hashes, каждый command-stream ledger и ordered segment table. Каждый segment имеет owner, schema/version, byte length и ADR-012 domain-separated SHA-256. SkillProficiency находится в RPG segment; transition/PolicyState — в Motor Runtime segment; immutable weights в save запрещены.

Save procedure: freeze logical commit point → snapshot owner segments → write new staging save → fsync files/manifest where platform permits → validate → atomic pointer update. Autosave MUST NOT перезаписывать единственную известную валидную generation.

Migration является pure ordered transform `N → N+1` с fixture tests. Migration работает с копией; unknown required segment, checksum mismatch, missing content revision или failed transform → fail-closed. User получает stable code, affected segment и recovery choices; исходные bytes сохраняются.

## Replay

`ReplayManifest` фиксирует initial save/snapshot, seed streams, content/build/schema/physical-archetype/model/policy-route/plugin/MechanicsLock hashes, `CommandStreamId`/principal ledgers и canonical accepted external WorldCommand stream. Runtime-generated events, включая ActivePolicyRouteChanged, сохраняются как optional oracle. Replay runner MUST сравнивать ADR-012 per-tick state root, exact command/rejection/event outcomes и указывать first divergence. Cross-target tolerance не применяется к IDs, commands, events, manifests или gameplay outcomes.

## Public boundaries

Public schemas: NeutralAuthoringModel, ContentManifest, WorldChunk, MechanicPackage/MechanicsLock references, physical archetype/policy/skill references, SaveManifest, ReplayManifest, TestScenario fixture references и typed diagnostics. Filesystem paths, training sessions/optimizer state, archive library handles, ECS IDs и importer legacy types не входят в contracts. Asset access в runtime — `AssetId + resolved revision`, не raw path.

Scenario fixtures MAY быть cooked content bundles с neutral/generated assets и exact provenance. Evidence/media artifacts не являются runtime assets, хранятся под explicit external artifact root и ссылаются на runtime content только по immutable hash. Approved baseline не может неявно изменить ContentManifest или save.

## Failure semantics

- Corrupt/missing required bundle → chunk не активируется; если critical startup chunk — чистый отказ запуска до world mutation.
- Optional asset missing → declared placeholder/fallback с diagnostic, если schema разрешает optional.
- I/O cancellation → staging discarded; current active revision остаётся.
- Save corruption/incompatibility → исходник не меняется, partial state не создаётся.
- Missing required mechanic package/state migration → save не открывается; original сохраняется, пользователь получает exact lock/migration diagnostic.
- Missing required physical archetype/model/PolicyState schema → save не открывается до world mutation; explicit predeclared downgrade выполняется только как copy-on-write migration с новым manifest hash.
- Streaming budget overrun → throttle/preempt optional requests; authoritative active objects не выгружаются без quiesce.
- Hash collision evidence → security fatal; bundle quarantined.
- Evidence/media bytes обнаружены в runtime/source bundle без explicit content role → validation failure; artifact остаётся во внешнем store.

## Verification gates

| Gate | Сценарий | Threshold | Evidence | Fallback/rollback |
|---|---|---|---|---|
| ASSET-01 | cook same fixture twice на Win/Linux | platform-neutral artifacts byte-identical по `CANON-01`; cache second run ≥90% hits | manifests, raw-byte hashes, timing JSON | blocking canonicalization fix |
| ASSET-02 | malformed/cyclic/missing deps corpus | 100% classified diagnostics; 0 partial publish | validator report | quarantine input |
| STREAM-01 | 10 000 randomized load/unload cycles | 0 duplicate IDs/leaks/dangling active refs; p99 commit ≤2 gameplay ticks after I/O ready | memory + resolver report | reduce concurrent requests |
| SAVE-01 | power-failure injection at every write boundary | одна из двух валидных generations загружается в 100% points | fault matrix | retain prior generation |
| SAVE-02 | migration fixtures N-2,N-1,N + corrupt cases | valid fixtures exact expected hash; invalid 100% fail-closed | migration report | require older executable/export tool |
| REPLAY-01 | 100 vertical fixtures | exact accepted command ledgers/rejections/events/outcomes; state roots exact на same target triple, first divergence stable-coded | replay report, command ledgers | release blocking |
| CONTRACT-01 | importer/editor/game/headless schema compatibility | 100% same schema registry; 0 source parser link in runtime | SBOM/link map/schema report | blocking package split |
