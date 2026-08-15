# SPEC-17: Direct project composition and activation

| Поле | Значение |
|---|---|
| ID | SPEC-17 |
| Статус | Accepted |
| Версия | 2.2 |
| Последняя проверка | 2026-08-16 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [ADR-018](adr/018-authoritative-project-composition-and-configuration.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](adr/048-direct-exact-project-lock.md), [ADR-052](adr/052-derived-world-calendar-and-authored-routine-vertical.md), [ADR-072](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md) |
| Заменяет | SPEC-17 2.1; updates the direct path to authoring V4 and `ActivatedProjectV5` |

## Назначение

Current project path is deliberately direct:

```text
nextengine.project-authoring.v4
  → cook and validate exact artifacts
  → ProjectLockV3
  → validate complete hash closure
  → ActivatedProjectV5
```

There is no runtime dependency resolver. `ProjectManifestV1`, requirements,
semantic-version ranges, package catalog records/snapshots, selected records,
resolver profiles, `project.json`, `catalog.json` and
`composition-lock.json` are retired alpha formats.

## Sources of truth

| State | Source of truth | Not authority |
|---|---|---|
| Editable project intent | `nextengine.project-authoring.v4` source | cooked cache or runtime defaults |
| Exact runnable closure | `ProjectLockV3` | filesystem discovery or catalog selection |
| Current schema set | `SchemaRegistryManifestV2` referenced by the lock | dynamic registration |
| Cooked content/world/mechanics | exact manifests and blobs referenced by the lock | source paths or importer records |
| Published runtime project | atomically validated `ActivatedProjectV5` | partially decoded candidate |
| Application session | SPEC-29 session contracts | project authoring or lock policy fields |

Authoring describes only project-owned composition intent. Session recovery,
storage layout and shutdown policy do not belong to authoring or project lock;
save-on-close is the current application behavior from ADR-047.

## `ProjectLockV3`

`ProjectLockV3` contains exactly the project ID/revision, authoring hash,
schema/content/world/mechanics hashes, runtime/launch/platform profile hashes,
allowed presentation targets and its own canonical hash:

```text
ProjectLockV3 {
  project_id,
  project_revision,
  authoring_sha256,
  schema_registry_manifest_sha256,
  content_manifest_sha256,
  world_partition_manifest_sha256,
  mechanics_lock_sha256,
  runtime_determinism_profile_sha256,
  launch_profiles_sha256,
  platform_capability_profile_sha256,
  platform_timebase_profile_sha256,
  allowed_presentation_targets[],
  project_lock_sha256
}
```

Every hash is exact and non-zero; the target set is closed, sorted and unique.
The lock contains no requirements, version ranges, selected catalog records,
resolver trace, recovery policy, shutdown policy, storage policy, native handle
or backend-owned value.

## Cooking and package output

The cooker reads one bounded current authoring document and publishes:

- `project-lock.json`;
- `SchemaRegistryManifestV2`;
- content manifest;
- world partition manifest;
- mechanics lock and package payloads;
- render content catalog;
- exact referenced blobs and notices.

All outputs are staged privately. The cooker validates schemas, bounds,
canonical bytes, provenance and the complete reference graph before replacing
the output directory/current package. Failure leaves the prior output intact.
Cooker caches are reconstructible and do not enter lock identity unless their
produced artifact hash is explicitly referenced.

## Atomic activation

Activation accepts `ProjectLockV3` directly and must:

1. validate its current format and canonical hash;
2. fetch every exact referenced manifest/blob from the supplied package;
3. validate schema, content, world, mechanics, runtime, launch and platform
   profile hashes plus all nested references;
4. validate neutral records, localization fallback closure, animation/skeleton
   relations and render catalog against the content manifest;
5. build one immutable `ActivatedProjectV5` candidate, including exact world-routine, 100-record population and navigation catalog closure;
6. publish it atomically only after the complete closure passes.

No component may substitute a compatible-looking record, resolve a range,
scan another directory or activate a partial closure. `game`, `headless` and
runtime-bearing tools use the same activation implementation and exact lock.

## Current-only formats

Until the first publicly supported v1 format is declared, authoring, lock,
registry and package families are current-only under ADR-046. A recognizable
retired format returns its typed `UNSUPPORTED_*` before nested decode or
publication. There are no defaults, aliases, automatic upgrades or in-place
rewrites. Source bytes remain untouched so a user can open them with the
matching older build or an explicit future export tool.

## Failure semantics and checks

Malformed/noncanonical authoring, hash mismatch, missing reference, duplicate
identity, dependency cycle, forbidden target, unsupported format or nested
closure mismatch rejects the complete candidate and preserves the previously
published project. Diagnostics include stable code plus expected/actual hashes
or identities.

`content-package` is the governing ProductCheck: cook the reference project,
reopen the package, validate the complete hash closure and activate the same
`ActivatedProjectV5` in required roots. `fast` covers focused contract tests;
packaging or host changes additionally run `platform`/`v1-package` as selected
by SPEC-12.
