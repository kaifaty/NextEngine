# SPEC-24: Current neutral content and package closure

| Поле | Значение |
|---|---|
| ID | SPEC-24 |
| Статус | Accepted |
| Версия | 2.7 |
| Последняя проверка | 2026-08-18 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-04](04-rendering-and-platform.md), [SPEC-10](10-gothic-importer-boundary.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-22](22-schema-registry-compatibility-and-migration.md), [SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [ADR-014](adr/014-deterministic-extensions-and-package-trust.md), [ADR-044](adr/044-neutral-text-catalog-and-locale-fallback.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](adr/048-direct-exact-project-lock.md), [ADR-052](adr/052-derived-world-calendar-and-authored-routine-vertical.md), [ADR-072](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md), [ADR-073](adr/073-deterministic-cognition-owner-vertical.md), [ADR-074](adr/074-systemic-strategic-agent-owner-vertical.md) |
| Заменяет | SPEC-24 2.6; extends the exact current base-skinning profile with the bounded R5g pose-corrective set |

## Scope

This SPEC describes content contracts exercised by the current cooker,
reference project and package. It intentionally removes speculative archive
ABI, target-profile resolver, capability-scoring/fallback plans and full future
navigation/collision/world schema listings. Git history retains those designs;
a future production consumer must reintroduce only the fields it demonstrates.
The bounded domain-specific graph catalog admitted by ADR-072 is current; it
does not restore the removed generic navmesh or bundle ABI.

## Invariants

- `ContentManifestV1` is the exact immutable catalog referenced by
  `ProjectLockV3` and shared by `game`, `headless` and tools.
- One `AssetId` resolves to one exact `AssetRevisionRefV1` inside the manifest.
  Runtime does not choose floating revisions or search source directories.
- Every record has a current exact `SchemaRefV1`, canonical bytes, finite
  bounds, provenance/license binding and content hash.
- Required dependencies are present and acyclic before activation. Optional
  presentation content uses only a declared fallback.
- Durable/runtime identity is `PersistentId`; record indices, ECS IDs, paths,
  importer/backend types and native handles never become content identity.
- Mutable RPG/world/physics state is not copied into neutral content. Content
  seeds definitions; save owner segments remain authority after creation.
- Importer output is untrusted bounded neutral input and passes the same cooker
  validation as project-authored records.

## `ContentManifestV1`

The implemented current body contains:

```text
ContentManifestV1 {
  schema_ref,
  manifest_id,
  project_id,
  content_revision,
  schema_registry_manifest_sha256,
  canonicalization_profile_sha256,
  content_admission_limits_sha256,
  cooker_contract_sha256,
  cooker_options_sha256,
  root_assets[],
  provenance_records[],
  asset_entries[],
  dependency_edges[],
  domain_closure_sha256,
  content_manifest_sha256
}
```

`ContentAssetEntryV1` binds exact asset revision, schema ref, neutral record
blob hash, semantic class (`DomainRelevant | PresentationOnly`), provenance,
license manifest and owning bundle ID. `ContentDependencyEdgeV1` binds exact
source/target IDs, dependency kind and required flag.

Root refs, provenance, asset entries and edges are bounded, canonical, sorted
and unique. Required edges form an acyclic graph. Every root and edge endpoint
must exist; every entry resolves one exact provenance record; entry record hash
must match its asset revision. `domain_closure_sha256` is recomputed from
canonical asset/dependency facts.

`ContentProvenanceV1` contains stable provenance/license IDs, bounded
attribution and its canonical hash. Raw source paths, credentials and protected
source bytes are excluded. Required license/NOTICE data travels in the package.

## Current neutral records

`NeutralRecordV1` is the small generic current definition envelope:

```text
NeutralRecordV1 {
  schema_ref,
  asset_id,
  kind,
  record_id: PersistentId,
  persistent_references[],
  asset_dependencies[],
  properties[]
}
```

Its closed current kinds are Scene, Collider, CharacterDefinition,
ItemDefinition, InventoryDefinition, EquipmentDefinition, DialogueDefinition,
QuestDefinition, RelationshipDefinition, InteractionDefinition,
AbilityDefinition and WorldChunk. References/properties are bounded, sorted and
unique; the schema ID must exactly match the kind.

The implemented specialized neutral records are:

- `NeutralMeshV1`, `NeutralMaterialV1`, `NeutralTextureV1` and
  `B0RenderContentProfileV1`, cooked into `RenderContentCatalogV1`;
- `NeutralBaseSkinningProfileV1`, binding one exact mesh, source skeleton and
  `BodySchemaAssetV1` revision to stable render-joint IDs, explicit animation-
  joint/body-semantic mappings, bounded LBS influences, sparse authored pose
  correctives classified as `Essential | Detail` and a mandatory bind-pose
  fallback;
- `NeutralSkeletonV1` and `NeutralAnimationV1` with exact clip-to-skeleton
  binding;
- `NeutralAudioV1` with canonical bounded PCM/audio metadata;
- `TextCatalogV1` with deterministic locale fallback from ADR-044.
- domain-specific `WorldPopulationCatalogV1` and
  `WorldNavigationCatalogV1`, each carried as an exact root asset rather than
  a generic neutral navigation schema;
- domain-specific `AgentCognitionCatalogV1`, carried as one required exact root
  and bound to an existing population subject.
- `BodySchemaAssetV1`, carried as one DomainRelevant root with exact asset
  revision, frozen `BodySchemaV1` bytes and the admitted deterministic compiler
  profile. The reference CharacterDefinition has one required dependency on
  this asset.

These contracts use engine-owned fixed-width/canonical values, exact asset
revisions and checked bounds. Render records validate index/attribute lengths,
primitive support, texture extent/data/color semantics and material bindings.
Animation validates stable joint keys, hierarchy, channel/key order and exact
skeleton revision. Base-skinning validation requires an acyclic complete render
hierarchy, one-to-one source-joint/body-semantic mappings, exactly one to four
positive influences per mesh vertex, exact `u16::MAX` weight sums and a
positive per-frame instance bound. The current corrective subset admits at
most 64 canonical corrective IDs and 1,048,576 total sparse vertex deltas. Each
record names one existing render-joint translation axis, unequal signed
start/full driver deltas within ±2 m, one LOD class and unique in-range vertex
indices with non-zero per-component-bounded deltas (±0.5 m). At least one
corrective is Essential, and conservative full-delta accumulation must retain
every corrected bind vertex inside the exact mesh bounds. Audio validates
sample format/rate/channels/frame bounds.

The reference alpha's scene/collider/RPG/world definitions use the generic
`NeutralRecordV1`; this SPEC does not promise the detailed future neutral
navigation, general collision, scene graph or bundle-container schemas removed
from version 1.2.

## Localization

Each `TextCatalogV1` has stable catalog asset/revision, bounded locale,
optional fallback locale and sorted `(TextId, template)` entries. Exactly one
source locale has no fallback; all others form an existing acyclic chain to it.
Pseudo-locale is ordinary content. Catalogs are PresentationOnly and rendered
strings never enter gameplay identity or roots.

Missing locale/entry/argument follows the declared fallback, then produces the
readable placeholder and stable `LOCALIZATION_RESOURCE_MISSING` diagnostic.

## Cooking and activation

The cooker canonicalizes and validates every current record, constructs
schema/content/world/mechanics/render closure, writes exact blobs plus notices
and publishes `project-lock.json`. Physical package layout is private; there is
no public bundle archive ABI or runtime catalog resolver.

`ActivatedProjectV8` revalidates:

- exact `ProjectLockV3`, `SchemaRegistryManifestV2`, `ContentManifestV1` and
  world/mechanics hashes;
- every neutral record against its manifest entry;
- unique asset IDs and complete dependencies/provenance;
- localization fallback closure;
- audio and animation/skeleton exact references;
- render catalog canonical round-trip and equality to manifest render entries;
- every base-skinning profile against its exact mesh vertex closure, source
  skeleton joints, body semantics, required root/dependency revisions and
  canonical profile hash, including corrective driver IDs, LOD classes,
  sparse vertex indices/deltas and corrected bind bounds;
- exact population/navigation catalog revisions, 100-record closure and graph
  references against the 64 world chunks;
- exact cognition catalog revision, sorted seed beliefs, bounded cadence/
  planner profile and subject binding to the population/RPG/bootstrap closure;
- exact activity catalog revision, systemic-work profile, worker/commitment/
  workplace/item bindings and their population/RPG/bootstrap closure.
- exact `BodySchemaAssetV1` record/hash, frozen reference schema generation,
  admitted compiler profile, required root and CharacterDefinition dependency.

Only the complete candidate publishes. Failure leaves the prior activated
project/content untouched. Equal canonical inputs produce byte-identical
portable records and hashes; private target payloads cannot change domain
closure or authoritative output.

## Current-only compatibility

Pre-v1 content/authoring/package formats are exact-current only. Retired
formats return typed `UNSUPPORTED_*` before nested activation; no defaults,
catalog selection, automatic migration or in-place rewrite occurs. Rejected
source bytes are preserved.

Under ADR-046 the current alpha `RenderContentCatalogV1` payload is not a
public persisted ABI. R5f added the base-skinning family and R5g adds its exact
pose-corrective field to that current shape; an older catalog must be recooked
by its matching toolchain and is never silently decoded or migrated as the new
shape.

## Public boundary and checks

Public content contains only engine-owned IDs, schema refs, hashes, bounded
canonical values, manifests and immutable records. ECS/storage, OS/window,
task/thread, vendor/compiler, database, importer and filesystem types are
forbidden.

`content-package` is the governing check: cook and reopen the 37-root/123-entry
reference project, validate the direct-lock closure, Luau/Wasm content,
provenance/NOTICE, localization, audio/animation, body schema, base-skinning
profile/correctives and render catalog; malformed version/hash/bounds,
mapping/weights/driver/delta/LOD data, duplicate, cycle, missing dependency and
forbidden-type cases publish nothing.
Renderer-facing changes additionally run `platform`/`visual-smoke`; gameplay
content changes additionally run `play` and state-bearing changes run
`persistence-replay`.
