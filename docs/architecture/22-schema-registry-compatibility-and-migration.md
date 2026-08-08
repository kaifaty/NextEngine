# SPEC-22: Current schema registry and format compatibility

| Поле | Значение |
|---|---|
| ID | SPEC-22 |
| Статус | Accepted |
| Версия | 2.0 |
| Последняя проверка | 2026-08-08 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [ADR-025](adr/025-schema-content-and-migration-authority.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](adr/048-direct-exact-project-lock.md) |
| Заменяет | SPEC-22 version 1.7 generic N-2 migration framework |

## Current policy

Pre-v1 alpha formats are current-only. The engine supports the exact formats
produced by the current build and fails early on retired or unknown versions.
It does not retain readers, defaults or migration DAGs for experimental
formats that have no public support promise.

| Family | Current form | Retired examples | Required result |
|---|---|---|---|
| Project authoring | `nextengine.project-authoring.v2` | authoring v1 | typed `UNSUPPORTED_PROJECT_AUTHORING_FORMAT` |
| Project activation | `ProjectLockV3`, `ActivatedProjectV3` | resolver/composition locks | typed `UNSUPPORTED_PROJECT_FORMAT` or `PROJECT_LOCK_INVALID` |
| Schema registry | `SchemaRegistryManifestV2` | registry V1 | typed unsupported project/registry result |
| Runtime/save | `RuntimeSnapshotV3`, `WorldCheckpointV4`, `SaveManifestV2` | retired alpha state forms | family-specific typed unsupported result |
| Replay | `ReplayManifestV5` | `ReplayManifestV4` and earlier | `UNSUPPORTED_REPLAY_MANIFEST_VERSION` |
| Input mapping | `InputMappingReceiptV2` | V1 | typed unsupported mapping/version result |
| Package | current package manifest with `project_lock_sha256` | pre-direct-lock packages | typed `UNSUPPORTED_PACKAGE_FORMAT` |

The bounded outer format/version probe runs before nested canonical decode,
hash traversal, store publication or Runtime mutation. Unsupported input is not
deleted or rewritten.

`SaveManifestV2`, `WorldCheckpointV4`, `ReplayManifestV5`, the two-slot
`SaveStore` and `CommandLedgerV2` wire semantics remain unchanged by this
policy. `ReplayManifestV5` validates its own current closure directly; it is not
projected through a removed V4 type.

## `SchemaRegistryManifestV2`

The current registry is an exact project artifact, not a compatibility engine:

```text
SchemaRegistryManifestV2 {
  registry_revision,
  canonicalization_profile_sha256,
  ownership_registry_sha256,
  descriptors[] {
    schema_ref { schema_id, schema_version, descriptor_sha256, role, encoding },
    owner_context_id,
    field_registry_sha256
  },
  current_schema_refs[],
  schema_registry_manifest_sha256
}
```

Descriptors and current refs are bounded, canonical, sorted and unique. Every
current ref must resolve to an exact included descriptor. Roles and encodings
are closed engine-owned enums. Registry revision and schema versions are
positive. Same identity with different bytes is a mismatch, not an alternate
compatible revision.

The registry contains no historical descriptor window, compatibility table,
migration units, DAG, implementation callbacks or storage handles. Its exact
hash is bound by `ProjectLockV3` and validated equally by required composition
roots.

## Validation and publication

For every current artifact family:

1. probe bounded outer format/version;
2. decode using declared canonical limits;
3. reject unknown fields, duplicate identities, invalid order/bounds and hash
   mismatch;
4. validate all exact references and owner/content/project closure;
5. stage the complete candidate privately;
6. atomically publish, or preserve the previous generation/project/state.

No tolerant reader may infer a missing value or activate a partial closure.
Inspection tools use the same decoder and may report incompatibility, but do
not gain a private mutation or upgrade path.

## Future migrations

Migration becomes an Accepted requirement only after the project declares its
first publicly supported v1 persisted format and a real production format
change exists. That change must define the smallest necessary source/target
window, an explicit offline or pre-activation transform, copy-on-write
publication, failure behavior and a ProductCheck. Generic N-1/N-2 support is
not reserved in advance.

Git history is the historical specification for removed alpha layouts. Full
legacy schema listings are intentionally not duplicated into research docs.

## Product checks

`persistence-replay` proves current Save/Load/Replay closure, direct V5 compare
and early typed rejection of retired/corrupt data without partial mutation.
`content-package` proves the current registry/project/package hash closure.
Focused contract tests cover canonical ordering, duplicate identity, missing
descriptor, unknown field, bounds and hash tamper cases.
