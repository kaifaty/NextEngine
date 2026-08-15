# SPEC-25: Current bounded world partition, streaming and population binding

| Поле | Значение |
|---|---|
| ID | SPEC-25 |
| Статус | Accepted |
| Версия | 2.4 |
| Последняя проверка | 2026-08-16 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](22-schema-registry-compatibility-and-migration.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [ADR-026](adr/026-deterministic-work-resource-and-streaming-admission.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](adr/048-direct-exact-project-lock.md), [ADR-051](adr/051-r3a-packaged-chunk-streaming-commit-boundary.md), [ADR-052](adr/052-derived-world-calendar-and-authored-routine-vertical.md), [ADR-072](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md) |
| Заменяет | SPEC-25 2.3; binds the admitted population/navigation catalogs to the unchanged four-region/64-chunk partition without promoting generic residency/eviction |

## Назначение

Current contract фиксирует cooked partition closure, которую производит
cooker и валидирует project activation, plus exact bindings to the separate
R4b population/navigation content and residency policy. Mutable population
state remains a separate World Services owner; this document does not define a
generic streaming scheduler or persistent-object database.

Reference alpha содержит четыре authored regions и 64 chunk bindings. Existing
relay station/frontier identities сохраняются, а remaining bindings покрывают
`relay-station`, `frontier`, `high-pass` и `river-basin` по 16 chunks. Все они
входят в immutable project closure до запуска мира. Current R3 consumer
использует manifest без изменения wire shape.

## Authority and invariants

- Asset & Persistence публикует immutable `WorldPartitionManifestV1`.
- `ProjectLockV3.world_partition_manifest_sha256` связывает exact canonical
  manifest bytes.
- `schema_registry_manifest_sha256` и `content_manifest_sha256` внутри manifest
  должны совпасть с активируемой project closure.
- Chunk и required asset references используют engine-owned IDs и exact
  revisions; filesystem paths, runtime entities, task handles и backend types
  не входят в contract.
- Cooker и activation используют один codec и validator.
- Invalid, stale, noncanonical или incomplete closure отклоняется до world
  publication; prior active project остаётся неизменным.
- Pre-public alpha versions не мигрируются и не получают defaults.

## Current wire shape

```text
WorldPartitionManifestV1 {
  partition_id,
  coordinate_profile_id,
  topology_revision,
  root_region_ids[],
  chunk_bindings[],
  initial_placement_catalog_sha256,
  residency_policy_sha256,
  schema_registry_manifest_sha256,
  content_manifest_sha256,
  world_partition_manifest_sha256
}

WorldChunkBindingV1 {
  chunk_id,
  region_id,
  chunk_asset: AssetRevisionRefV1,
  required_asset_ids[]
}
```

`schema_version = 1` and
`world_partition_format = "nextengine.world-partition.v1"` are part of the
canonical JCS representation. The self hash is not embedded in the JCS body;
it is recomputed with the format domain and stored by the Rust value and
`ProjectLockV3`.

Construction and decode:

1. require positive `topology_revision`;
2. sort root region IDs and chunk bindings by canonical ID;
3. sort and de-duplicate every binding's `required_asset_ids`;
4. reject duplicate roots/chunks and a chunk whose region is not a declared
   root;
5. enforce current bounded decode/count limits;
6. reject unknown fields/closed values and noncanonical JCS;
7. recompute and compare the exact manifest hash before activation.

`initial_placement_catalog_sha256` now equals the exact current
`WorldPopulationCatalogV1` revision, and `residency_policy_sha256` binds the
16/32/52 counts, 3/15/60 periods, courier identity/start tick and navigation
revision. The manifest still does not embed mutable records or create a
generic admission-plan, interest, pin, lease or eviction API. Those require a
future consumer and explicit contract revision.

## Current cook and activation

`project.authoring.json` is untrusted input. Cooker validates the declared
regions, chunks and required assets, writes canonical `world-partition.json`
and binds its hash into `ProjectLockV3`. Package publication includes the
manifest and every referenced content blob.

Activation reads `ProjectLockV3`, verifies the complete hash closure, decodes
the partition with current limits and publishes one `ActivatedProjectV5` only
after all project artifacts agree. It does not resolve versions from a
catalog, scan ambient files or repair a partial closure.

Activation по-прежнему eager и проверяет все 64 alpha chunks целиком. World затем
использует pinned generation, чтобы повторно fetch/decode target chunk через
production bounded path; eager activation не превращается в lazy project loader.

## Persistence and replay

Save/replay bind the already activated project/content/world hashes through
their current manifests. The separate `WorldPopulationSnapshotV1` serializes
the 100 durable logical placement/tier records and Replay V7 proves their
ledger closure. Runtime entity mappings, pending file operations and cache
state remain reconstructible and are not serialized.

Load accepts only the current compatible project closure. Missing or changed
partition/content bytes reject before world mutation and preserve the source
save. There is no alpha placement migration or implicit reseed path.

## Current bounded R3 partition

R3a установил ровно один path:

```text
chunk fetch → bounded decode → hash/reference validation → canonical commit
```

Production consumer связывает запрос с exact manifest binding, project/content/
schema hashes, topology revision, world generation и упорядоченной asset
closure. Bounded packaged load и paired Runtime/World commit следуют SPEC-03 и
ADR-051. При этом по-прежнему нет public contract для:

- generic jobs or scheduler topology;
- cancellation trees;
- pins, leases or eviction policy;
- residency interests or admission plans;
- population tiers or abstract outcomes;
- persistent spatial-object placement/migration schemas.

R3b применяет этот же path к canonical route из 64 bindings. Initial и
gameplay-target выбираются по exact Rust-only reference roles; initial идёт
первым, остальные bindings сортируются по `(region_id, chunk_id)`. Каждый
переход публикует `Requested`, затем `Active/Unloaded` на следующем existing
tick; source становится `Unloaded`, target остаётся единственным `Active`, а
generation увеличивается ровно один раз. Save в `Requested` не сохраняет
staging bytes: restart заново строит request и повторяет packaged fetch.

ADR-072 changes only the exact placement/residency bindings: initial
population placement is now the separate typed catalog/snapshot closure, and
the navigation graph binds one node to every chunk. The existing R3 streaming
state machine and 64 chunk bindings do not change. Generic spatial objects,
interest/residency admission and eviction remain future concepts.

## Current bounded R4b population relation

- `WorldNavigationCatalogV1` contains four region tiles and 64 unique nodes;
  every node ID equals one declared chunk ID and resolves to its region tile.
- `WorldPopulationCatalogV1` binds that exact navigation revision and contains
  100 stable subjects with home/initial/goal nodes.
- `WorldPopulationSnapshotV1` owns current region/node/tier separately from
  `WorldStreamingSnapshotV1`; loading a logical NPC into a tier does not load a
  player chunk or publish a physical pose.
- One `Abstract` courier transfer may cross relay-station to frontier only
  against a validated route-plan hash. `Active` transfer cannot mutate this
  logical binding and requires the separate physical traversal path.
- Runtime, streaming, routine and population candidates share one final
  preflight; stale partition or population generation publishes none.

## Failure semantics

| Failure | Required result |
|---|---|
| Unknown format/version/field | typed current project/format rejection before publication |
| Hash or project-closure mismatch | discard staging and retain the prior active project |
| Duplicate/missing region, chunk or asset reference | reject the complete partition |
| Population/navigation hash, node/chunk or policy mismatch | reject the complete project before World Services activation |
| Decode bound exceeded or noncanonical JCS | reject before unbounded allocation or world creation |
| Packaged fetch/decode/validation fault | retain the declared `Requested` root, prior active generation and decoded cache; allow retry/restart |

## Product checks

| Check | Current evidence |
|---|---|
| focused project/contracts tests | canonical four-region/64-chunk ordering, bounds, unknown-field, duplicate ID, wrong class and dependency mismatch failures |
| `content-package` | cooker/package/activation agree on 4 regions, 64 chunks, typed routine/population/navigation catalogs and 116 packaged entries |
| `persistence-replay` | save after `Requested`, process restart, exact pinned reactivation/re-fetch and the separate population segment complete with the uninterrupted six-owner root |
| `performance --scenario smoke --mode report` | 1,000 transitions perform real packaged I/O and record existing V4 logical staging charges; 30 seconds remains report-only |
| `performance --scenario r3-multiregion-streaming --mode report` | 1,000 transitions cycle over the canonical 64-chunk route with production default two workers; only `streaming_world` is authoritative and the scenario remains report-only |
| `performance --scenario r4-100npc --mode report` | 1,000 warm-up plus 10,000 measured joint ticks execute one graph query per due population record with exact roots and no drop/starvation; unsupported-host timing remains `NOT_RUN`, not B-12 evidence |
| `host-check` | current workspace contract and structural checks pass |

No generic world-admission, eviction or alpha migration check exists in the
current baseline.
