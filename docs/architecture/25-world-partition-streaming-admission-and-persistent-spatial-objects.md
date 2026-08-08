# SPEC-25: Current world partition manifest and R3a boundary

| Поле | Значение |
|---|---|
| ID | SPEC-25 |
| Статус | Accepted |
| Версия | 2.1 |
| Последняя проверка | 2026-08-08 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](22-schema-registry-compatibility-and-migration.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [ADR-026](adr/026-deterministic-work-resource-and-streaming-admission.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](adr/048-direct-exact-project-lock.md), [ADR-051](adr/051-r3a-packaged-chunk-streaming-commit-boundary.md) |
| Заменяет | SPEC-25 1.0 generic admission planner, pins/leases/eviction, population tiers and unimplemented spatial-object schemas |

## Назначение

Current contract фиксирует только cooked partition closure, которую уже
производит cooker и валидирует project activation. Он не определяет generic
streaming scheduler, mutable population state или persistent-object database.

Reference alpha содержит два authored chunk bindings: relay station и
frontier. Оба входят в immutable project closure до запуска мира. Текущий R3a
consumer использует этот manifest без изменения wire shape.

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

`initial_placement_catalog_sha256` and `residency_policy_sha256` are exact
current closure bindings. Their presence does not create an implemented
placement catalog, population tier, admission-plan or eviction API. A future
consumer either uses these bindings with an Accepted contract or replaces them
through an explicit current format revision.

## Current cook and activation

`project.authoring.json` is untrusted input. Cooker validates the declared
regions, chunks and required assets, writes canonical `world-partition.json`
and binds its hash into `ProjectLockV3`. Package publication includes the
manifest and every referenced content blob.

Activation reads `ProjectLockV3`, verifies the complete hash closure, decodes
the partition with current limits and publishes one `ActivatedProjectV3` only
after all project artifacts agree. It does not resolve versions from a
catalog, scan ambient files or repair a partial closure.

Activation по-прежнему eager и проверяет оба alpha chunks целиком. World затем
использует pinned generation, чтобы повторно fetch/decode target chunk через
production R3a path; eager activation не превращается в lazy project loader.

## Persistence and replay

Save/replay bind the already activated project/content/world hashes through
their current manifests. Runtime entity mappings, pending file operations and
cache state are reconstructible and are not serialized.

Load accepts only the current compatible project closure. Missing or changed
partition/content bytes reject before world mutation and preserve the source
save. There is no alpha placement migration or implicit reseed path.

## Current R3a vertical

R3a доказывает ровно один path:

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

Those concepts require a demonstrated consumer and their own ProductCheck;
they are not prerequisites for R3a.

## Failure semantics

| Failure | Required result |
|---|---|
| Unknown format/version/field | typed current project/format rejection before publication |
| Hash or project-closure mismatch | discard staging and retain the prior active project |
| Duplicate/missing region, chunk or asset reference | reject the complete partition |
| Decode bound exceeded or noncanonical JCS | reject before unbounded allocation or world creation |
| Packaged fetch/decode/validation fault | retain the declared `Requested` root, prior active generation and decoded cache; allow retry/restart |

## Product checks

| Check | Current evidence |
|---|---|
| focused project/contracts tests | canonical roundtrip, ordering, bounds, unknown-field and hash/reference failures |
| `content-package` | cooker/package/activation agree on the exact partition and full blob closure |
| `persistence-replay` | save after `Requested`, process restart and re-fetch converge to the same final root and retain exact ledger roots |
| `performance --scenario smoke --mode report` | 1,000 transitions perform real packaged I/O and record existing V4 logical staging charges; 30 seconds remains report-only |
| `host-check` | current workspace contract and structural checks pass |

No separate world-admission, population-tier or migration check exists in the
current baseline.
