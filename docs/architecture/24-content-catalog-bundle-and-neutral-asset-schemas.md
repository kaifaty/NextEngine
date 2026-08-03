# SPEC-24: Content catalog, bundle и neutral asset schemas

| Поле | Значение |
|---|---|
| ID | SPEC-24 |
| Статус | Accepted |
| Версия | 1.1 |
| Последняя проверка | 2026-08-03 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-04](04-rendering-and-platform.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-10](10-gothic-importer-boundary.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](22-schema-registry-compatibility-and-migration.md), [ADR-018](adr/018-authoritative-project-composition-and-configuration.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-025](adr/025-schema-content-and-migration-authority.md), [ADR-044](adr/044-neutral-text-catalog-and-locale-fallback.md) |
| Заменяет | отсутствует |

## Назначение и invariants

SPEC-24 определяет один portable contract между authoring/import boundary,
schema registry, cooker, project composition и runtime content consumers.

- `ContentManifestV1` MUST быть единственным immutable catalog exact cooked
  content revision, активируемым через `ProjectCompositionLock`.
- Логический bundle MUST состоять только из canonical manifest descriptor и
  exact immutable blobs. Directory, archive, object-store или package layout
  является private packing и не меняет logical bundle identity.
- Один `AssetId` MUST разрешаться ровно в один `AssetRevisionRefV1` внутри
  exact manifest. Runtime не разрешает floating revision и не ищет source.
- Schema identity MUST использовать exact `SchemaRefV1` из SPEC-22. Unknown,
  incompatible или hash-mismatched schema отвергается до content publication.
- Required dependency closure MUST быть complete и acyclic. Optional
  presentation content использует только exact declared fallback.
- Target-specific payload MAY изменять только declared presentation bytes.
  Neutral records, domain-relevant payload, IDs, bounds, dependency closure и
  `domain_closure_sha256` MUST оставаться target-independent.
- Import boundary MAY выдавать только bounded records общих neutral schemas и
  sanitized provenance. Source-format, legacy parser/VM и importer-private
  records MUST NOT становиться public runtime schema.
- `game`, `headless` и `capture-worker` MUST читать один
  `ContentManifestV1`, один domain closure и одинаковые schema/dependency
  semantics. Headless MAY не materialize presentation-only blobs.
- Runtime MUST NOT открывать authoring/source formats, source archives, source
  locators или filesystem paths. Runtime lookup равен только
  `AssetId + record_sha256 + blob_sha256`.
- Public content contracts MUST contain only engine-owned nominal IDs,
  `SchemaRefV1`, canonical numbers, hashes, bounded arrays and closed enums.
  ECS storage/components, OS objects, native task/thread handles, vendor or
  backend/compiler objects, API handles, database connections, raw filesystem
  paths and tool-native values are forbidden.

## Source of truth и ownership

| State | Единственный owner/source of truth | Allowed projection / forbidden duplicate |
|---|---|---|
| Schema identity, field meaning and compatibility | SPEC-22 `SchemaRegistryManifestV1` and exact `SchemaRefV1` | Content records reference it; they do not redefine compatibility or migration |
| Authored/imported neutral input before validation | Authoring adapter or external import process | Bounded staging only; never runtime authority |
| Validated content catalog, dependency graph, variants and bundle roots | Asset & Persistence subsystem `ContentManifestV1` | Installer/cache indexes are reconstructible |
| Exact active content revision | `ProjectCompositionLock` content-manifest hash | Composition-root views cannot resolve or replace it |
| Immutable asset/blob bytes | Published logical bundles | CPU/GPU/audio/nav caches are reconstructible |
| Mutable world/RPG/physical/agent state | Owning runtime contexts from SPEC-01 | Neutral scene/chunk records are definitions, not mutable owner state |
| Target presentation selection | Exact target-profile table bound by the project lock | Ambient device, driver, locale, filesystem or worker state cannot select content |
| Source provenance | Canonical `ContentProvenanceV1` hash record | Raw source path, credentials and protected bytes are excluded |

Asset & Persistence subsystem owns validation, cooking, content hashing and atomic
publication. Rendering, Physical Embodiment, Audio/Navigation and World
Services consume immutable neutral records through their engine-owned
boundaries. They MAY build private caches, but cannot write a second content
catalog or reinterpret a schema field.

## Canonical scalar, identity and coordinate profile

The following rules apply to every V1 record:

- `Hash256` is exactly 32 bytes. Its text projection, where required, is 64
  lowercase hexadecimal characters.
- `AssetId` identifies one logical asset. `AssetRevisionRefV1` is exactly
  `{ asset_id: AssetId, record_sha256: Hash256 }`.
- Durable authored object identity uses `PersistentId`. `RuntimeEntityId` MUST
  NOT appear in content, bundle, dependency or cross-chunk records.
- Record-local slots use zero-based `u32` and are valid only inside the exact
  record revision. Cross-record references MUST use `AssetRevisionRefV1`;
  durable object references MUST use `PersistentId`.
- Namespaced IDs are NFC UTF-8, at most 255 bytes. Human labels are NFC UTF-8,
  at most 4,096 bytes, and MUST NOT be used as identity.
- Collections are ordered by their declared semantic order or sorted by full
  canonical encoded key bytes when they are sets/maps. Duplicate normalized
  identity, duplicate set/map key and noncanonical order are invalid.
- Neutral geometry uses a right-handed frame: `+X` right, `+Y` up, `+Z`
  forward. Positive rotation follows the right-hand rule.
- Length is metres, mass is kilograms, duration is seconds, angle is radians
  and audio time is integer sample frames. No implicit source-unit conversion
  survives validation.
- Spatial positions are signed `i64` micrometres. Their inclusive allowed
  range is `-8_388_608_000_000..=8_388_608_000_000`; arithmetic outside it is
  invalid. `AabbI64V1` uses inclusive minimum and exclusive maximum, with
  `min < max` on all axes.
- Unit directions and rotations use signed Q1.30 components. A quaternion is
  `(x, y, z, w)`, rejects all-zero, has exact squared-length bounds
  `2^60 - 2^36 <= sum(component^2) <= 2^60 + 2^36`, and has canonical sign:
  `w > 0`, or when `w = 0`, the first nonzero component of `(x,y,z)` is
  positive.
- Scale uses unsigned Q16.16 in the inclusive range `64..=67_108_864`
  (`1/1024..=1024`). Reflection MUST be baked explicitly; negative or zero
  scale is invalid.
- Colors and normalized scalar weights use unsigned 16-bit normalized values.
  Normals/directions use signed 16-bit normalized components in
  `-32_767..=32_767`; `-32_768` is invalid and vector sign is semantic, never
  canonical-flipped. Unit vectors require
  `32_767^2 - 65_535 <= sum(component^2) <= 32_767^2 + 65_535`.
  Tangents carry a separate handedness value of exactly `-1` or `+1`.
- Floating payload is allowed only where the owning `SchemaDescriptorV1`
  explicitly declares `F32Bits` or `F64Bits`. NaN, infinity, semantic negative
  zero and platform-native layout are invalid.

`NeutralTransformV1` is exactly translation in signed micrometres, canonical
Q1.30 quaternion and positive Q16.16 scale. An adapter that begins with another
unit, axis or handedness MUST convert before constructing a neutral record and
record the conversion recipe hash in provenance.

`UvTransformV1` is a row-major 2-by-3 affine matrix of signed Q16.16
coefficients. It multiplies the column vector `(u, v, 1)` to produce `(u', v')`;
each coefficient is in the inclusive range `[-1024, 1024]`. This is the only UV
transform representation in V1 material records.

## Common record and provenance envelopes

Every neutral record is a `CanonicalBinaryV1` envelope with
`owner_id = "nextengine.assets"`, its registered schema ID, and
`segment_id = "v1"`. Stable field IDs and the positive monotonic `u32`
`schema_version` come from its exact `SchemaDescriptorV1`.

All schemas owned by this specification use `owner_context_id =
"nextengine.assets"` and compatibility policy `ExactOnly`. A schema change
therefore creates a new descriptor/version and a new cooked generation; a
consumer never reinterpret-casts old bytes. In addition to the ten
`NeutralContent` registrations below, SPEC-24 owns these supporting
registrations:

| Schema ID | Role | Encoding |
|---|---|---|
| `nextengine.content.provenance` | `Manifest` | `JcsRfc8785` |
| `nextengine.content.admission-limits` | `Manifest` | `JcsRfc8785` |
| `nextengine.content.manifest` | `Manifest` | `JcsRfc8785` |
| `nextengine.content.target-capability-profile` | `Manifest` | `JcsRfc8785` |
| `nextengine.content.target-capability-set` | `Manifest` | `JcsRfc8785` |
| `nextengine.content.variant-fallback-plan` | `Manifest` | `JcsRfc8785` |
| `nextengine.content.target-variant-profile` | `Manifest` | `JcsRfc8785` |
| `nextengine.content.bundle-descriptor` | `Manifest` | `JcsRfc8785` |
| `nextengine.content.blob-identity` | `Manifest` | `CanonicalBinaryV1` |

```text
NeutralRecordEnvelopeV1 {
  asset_id,
  schema_ref: SchemaRefV1,
  semantic_class,
  record_revision,
  provenance_sha256,
  declared_bounds,
  dependencies[],
  payload
}
```

`record_revision` is a positive monotonic `u64` scoped to `asset_id`; it is
ordering metadata only, while `record_sha256` is exact identity.

`semantic_class` is a closed enum:

- `DomainRelevant`: may affect collision, navigation, timing, authored object
  identity or other gameplay-facing immutable facts; only portable payload is
  allowed;
- `PresentationOnly`: may have target-specific blobs and cannot alter domain
  state or content dependencies;
- `ToolOnly`: may exist in validated authoring staging but MUST be removed
  before a runtime `ContentManifestV1` is published.

`ContentProvenanceV1` is JCS-canonical and contains:

```text
ContentProvenanceV1 {
  schema_ref: SchemaRefV1,
  provenance_format = "nextengine.content-provenance.v1",
  origin_kind,
  producer_id,
  producer_version,
  producer_contract_sha256,
  source_identity_sha256,
  input_sha256[],
  transform_recipe_sha256,
  coordinate_conversion_sha256,
  license_expression,
  license_record_sha256[]
}
```

`origin_kind` is `ProjectAuthored`, `Generated` or `ImportedNeutral`.
`source_identity_sha256` is a sanitized logical fingerprint, never a source
path. Inputs and license records sort by full hash bytes. Missing producer,
input, conversion or license information required by project policy makes the
record unpublished. Importer build identity is represented only by ordinary
producer fields; no importer struct or source-specific enum crosses this
boundary. `producer_contract_sha256` binds the platform-neutral producer
source/build-recipe contract. Exact host executable hashes belong in the
cook/import `RunManifest`; they do not make equal portable content differ by
host.

The provenance and neutral-record hashes are exact:

```text
provenance_bytes = JCS(ContentProvenanceV1)

provenance_sha256 = SHA256(
  "nextengine.content-provenance.v1\0"
  || u64_le(provenance_bytes.len)
  || provenance_bytes
)

schema_ref_bytes = JCS(NeutralRecordEnvelopeV1.schema_ref)
record_bytes = CanonicalBinaryV1(NeutralRecordEnvelopeV1)

record_sha256 = SHA256(
  "nextengine.neutral-record.v1\0"
  || u32_le(schema_ref_bytes.len)
  || schema_ref_bytes
  || u64_le(record_bytes.len)
  || record_bytes
)
```

The provenance schema ID is `nextengine.content.provenance`.
`schema_ref_bytes` MUST be at most `u32::MAX`; the reference MUST resolve to
the same descriptor as the binary envelope schema ID/version. Neither
`provenance_sha256` nor `record_sha256` is embedded in its own preimage.

## Content admission limits

`ContentAdmissionLimitsV1` is hash-bound by `ProjectCompositionLock`. The V1
ceiling below MAY be lowered by a project but MUST NOT be raised without a new
accepted limits profile:

| Item | Maximum |
|---|---:|
| JCS bytes in `ContentManifestV1.body` | 67,108,864 |
| Asset entries | 1,048,576 |
| Dependency edges | 4,194,304 |
| Root asset references | 65,536 |
| Provenance records | 1,048,576 |
| Bundle descriptors | 65,536 |
| Target variant profiles | 64 |
| Payload candidates per asset role | 64 |
| Canonical bytes of one neutral record | 268,435,456 |
| Stored bytes of one blob | 4,294,967,296 |
| Decoded bytes of one blob | 8,589,934,592 |
| Stored bytes of one logical bundle | 68,719,476,736 |
| Decoded-to-stored ratio | 200:1 |
| Nested schema/container depth | 64 |

The profile schema ID is `nextengine.content.admission-limits`, role
`Manifest`, encoding `JcsRfc8785`. Its external hash is:

```text
limits_bytes = JCS(ContentAdmissionLimitsV1.body)

content_admission_limits_sha256 = SHA256(
  "nextengine.content-admission-limits.v1\0"
  || u64_le(limits_bytes.len)
  || limits_bytes
)
```

`ContentAdmissionLimitsV1.body` includes its exact `SchemaRefV1`, all values
above and any project-lowered values. The external hash is not part of the
body.

Any count, multiplication, offset or decoded-length overflow rejects the whole
staged record or bundle. Limits are decode/admission ceilings; they do not
authorize eager allocation, residency or blocking I/O.

## `ContentManifestV1`

The manifest has a closed JCS body and an external hash:

```text
ContentManifestV1 {
  body: {
    content_manifest_format = "nextengine.content-manifest.v1",
    schema_ref: SchemaRefV1,
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
    target_variant_profiles[],
    bundle_descriptors[],
    domain_closure_sha256
  },
  content_manifest_sha256
}
```

`content_manifest_sha256` is not part of its own preimage:

```text
manifest_bytes = JCS(ContentManifestV1.body)

content_manifest_sha256 = SHA256(
  "nextengine.content-manifest.v1\0"
  || u64_le(manifest_bytes.len)
  || manifest_bytes
)
```

The manifest `schema_ref.schema_id` is exactly
`nextengine.content.manifest`, has role `Manifest`, encoding `JcsRfc8785` and
resolves in the manifest's exact schema registry. Format V1 does not imply
`schema_version = 1`; the positive monotonic version comes only from
`SchemaRefV1`. `manifest_id` and `project_id` are nominal engine-owned IDs;
`content_revision` is a positive monotonic `u64` scoped to `project_id` and is
not a substitute for the manifest hash. `cooker_contract_sha256` binds the
platform-neutral cooker source/build recipe, while `cooker_options_sha256`
binds the exact canonical options. The host executable hash is recorded
separately in the cook `RunManifest`.

`root_assets` sort by `(AssetId canonical bytes, record_sha256)`;
`provenance_records` by their domain-separated provenance hash;
`asset_entries` by `(AssetId canonical bytes, record_sha256)`;
`dependency_edges` by the full tuple defined below; target profiles by
`target_profile_id`; bundle descriptors by `bundle_id`. Duplicate key,
unknown field, omitted required field, self-hash mismatch or alternate
canonical bytes invalidates the manifest.

Every `provenance_records` element is exactly
`{ body: ContentProvenanceV1, provenance_sha256 }`; its external hash is
recomputed, every referenced provenance hash has exactly one element, and an
unreferenced provenance element is invalid.

Each `bundle_descriptors` entry contains exact
`{ BundleDescriptorV1.body, bundle_descriptor_sha256,
bundle_root_sha256 }`. The outer manifest therefore binds the logical
descriptor and complete blob root without requiring a bundle-to-manifest hash
inside the root preimage.

One `ContentAssetEntryV1` contains:

```text
ContentAssetEntryV1 {
  asset_revision: AssetRevisionRefV1,
  schema_ref: SchemaRefV1,
  neutral_record_sha256,
  neutral_record_blob_sha256,
  semantic_class,
  provenance_sha256,
  license_manifest_sha256,
  declared_bounds,
  payload_candidates[],
  owning_bundle_id
}
```

Exactly one entry for an `AssetId` may be selected in a manifest. The
`neutral_record_sha256` MUST equal `asset_revision.record_sha256`; the
referenced record MUST re-encode canonically under the declared
`SchemaRefV1`, and entry `semantic_class` MUST equal the envelope class.
`ToolOnly` entries or candidates are invalid in a published manifest. An
entry is `DomainRelevant` whenever its neutral facts can affect the domain;
individual derived payload roles may still be `PresentationOnly`.
`neutral_record_blob_sha256` MUST resolve inside
`owning_bundle_id` to a portable blob with payload role
`nextengine.content.neutral-record`, encoding profile
`nextengine.encoding.canonical-binary-v1`, absent target profile and decoded
bytes exactly equal to that canonical neutral record. The record hash and blob
hash remain distinct domain-separated identities and both are checked.
`ContentManifestV1` does not reference
`ProjectCompositionLock`, because the lock references the manifest hash and a
back-reference would create a hash cycle.

Each `ContentPayloadCandidateV1` contains exactly:

```text
ContentPayloadCandidateV1 {
  payload_role_id,
  semantic_class,
  variant_key,
  encoding_id,
  blob_identity_sha256,
  blob_sha256
}
```

The identity hash is the domain-separated hash of canonical
`BlobIdentityV1` bytes defined below. A candidate cannot override the owning
asset's schema, neutral record, dependency or provenance.

## Dependency kinds and closure

`ContentDependencyV1` is:

```text
ContentDependencyV1 {
  source: AssetRevisionRefV1,
  kind: ContentDependencyKindV1,
  target: AssetRevisionRefV1,
  payload_role_id,
  fallback,
  activation_scope
}
```

`ContentDependencyKindV1` is a closed enum:

| Tag | Kind | Normative meaning |
|---:|---|---|
| 0 | `Strong` | Target record and every selected required blob MUST have the same `owning_bundle_id` as the source and be valid before source use. |
| 1 | `ActivationRequired` | Target MAY be in another bundle, but the complete target closure MUST be validated before the containing scene/chunk can activate. |
| 2 | `WeakPresentation` | Target is optional and `fallback` MUST name one exact portable blob, exact asset revision/role, or `BuiltinFallbackRefV1`. It MUST NOT affect `domain_closure_sha256`. |
| 3 | `ReferenceOnly` | Typed identity relation only. It does not authorize load, activation, mutation or implicit fallback. |

`fallback` MUST be absent for `Strong`, `ActivationRequired` and
`ReferenceOnly`, and present for `WeakPresentation`. Source/build inputs belong
only to `ContentProvenanceV1`; they are not runtime dependency kinds.
`BuiltinFallbackRefV1` is exactly
`{ fallback_id, engine_contract_sha256, blob_sha256 }` and MUST be present in
the project-lock fallback closure.

Edges sort by
`(source AssetId, source record hash, kind tag, target AssetId, target record
hash, payload role ID, canonical fallback bytes, activation scope)`. Every
cross-record asset reference inside a neutral payload MUST have exactly one
matching edge; hidden dependency inference is forbidden.

Closure is computed without ambient I/O:

1. Validate exact `SchemaRefV1` values against one
   `SchemaRegistryManifestV1`.
2. Sort roots and edges canonically. Starting at `root_assets`, follow
   `Strong` and `ActivationRequired` edges to a fixed point.
3. Reject a cycle in either required edge kind. Cross-chunk logical references
   that may be cyclic MUST be `ReferenceOnly` and use `PersistentId` or exact
   asset revision; they are not activation dependencies.
4. For one exact target profile, add each selected presentation payload.
   A missing `WeakPresentation` target follows only its exact declared
   fallback. Fallback chains MUST be acyclic and terminate in a present
   portable payload or closed engine fallback.
5. Verify every closure asset, schema, record, blob, provenance and license
   hash exactly once. Undeclared extra record/blob inside a logical bundle is
   invalid.
6. Produce canonical `domain_closure_sha256`, target-profile closure hash and
   bundle roots. Reordering source declarations, physical blobs or workers
   MUST change none of these results.

The target-independent projection is exactly:

```text
ContentDomainProjectionV1 {
  domain_projection_format = "nextengine.content-domain-projection.v1",
  schema_registry_manifest_sha256,
  canonicalization_profile_sha256,
  content_admission_limits_sha256,
  root_assets[],
  neutral_records[],
  nonpresentation_dependency_edges[],
  domain_payloads[],
  persistent_ids[]
}
```

`neutral_records` contains, for every asset reached by the `Strong` and
`ActivationRequired` fixed point, its asset revision, exact schema ref,
semantic class, provenance/license hashes and declared bounds. It includes
presentation-class neutral record hashes because neutral schemas themselves
are portable. `nonpresentation_dependency_edges` contains every `Strong`,
`ActivationRequired` and `ReferenceOnly` edge whose source is in that fixed
point; only `WeakPresentation` edges are excluded. `domain_payloads` contains
the complete candidate tuple for every `DomainRelevant` role in that fixed
point. `persistent_ids` is the unique sorted set extracted from those records.

Roots, records and edges use their manifest orders; payloads sort by
`(asset_id, record_sha256, payload_role_id, encoding_id, blob_sha256)` and
persistent IDs sort by canonical raw bytes. The projection excludes selected
presentation-only blobs, target profiles, physical packing and host data:

```text
domain_projection_bytes = JCS(ContentDomainProjectionV1)

domain_closure_sha256 = SHA256(
  "nextengine.content-domain-closure.v1\0"
  || u64_le(domain_projection_bytes.len)
  || domain_projection_bytes
)
```

## Target variants

Target variant selection is authored and cooked before runtime activation; it
is not hardware scoring performed by a composition root.

```text
TargetVariantProfileV1 {
  schema_ref: SchemaRefV1,
  target_profile_id,
  target_triple_id,
  capability_profile_body: TargetCapabilityProfileBodyV1,
  capability_profile_sha256,
  fallback_target_profile_id,
  selections[],
  profile_closure_sha256
}

TargetCapabilityProfileBodyV1 {
  schema_ref: SchemaRefV1,
  capability_profile_format = "nextengine.target-capability-profile.v1",
  target_triple_id,
  required_capabilities[],
  prohibited_capability_ids[]
}

TargetCapabilityAtomV1 {
  capability_id,
  capability_contract_sha256
}

TargetCapabilitySetV1 {
  body: {
    schema_ref: SchemaRefV1,
    capability_set_format = "nextengine.target-capability-set.v1",
    target_triple_id,
    available_capabilities[]
  },
  target_capability_set_sha256
}

VariantSelectionV1 {
  asset_revision,
  payload_role_id,
  selected_blob_sha256,
  source_variant_key,
  fallback_used
}
```

For supported v1 runtime profiles, `target_triple_id` is exactly
`windows-x86_64` or `linux-x86_64`. Apple Silicon macOS MAY host the portable
cooker/tool validation described here, but is not a game, renderer, packaging
or shipping target profile. `fallback_target_profile_id` is optional and, when
present, MUST resolve to a profile with the same `target_triple_id`.

The profile-body and capability-set schema IDs are exactly
`nextengine.content.target-capability-profile` and
`nextengine.content.target-capability-set`. Capability IDs are namespaced NFC
UTF-8 identities. Required and available
capability atoms sort by `(capability_id canonical bytes,
capability_contract_sha256)` and permit at most one atom per capability ID.
Prohibited IDs sort by canonical bytes. A duplicate ID or an ID appearing in
both required and prohibited sets invalidates the profile. The profile body's
`target_triple_id` MUST equal its enclosing target profile. Hashes exclude
their external hash field. Only engine-owned capability-contract IDs are
declared; vendor feature strings, driver objects and device handles are
forbidden. A profile has at most 1,024 required and 1,024 prohibited IDs; one
capability set has at most 4,096 available atoms.

```text
capability_profile_bytes = JCS(TargetCapabilityProfileBodyV1)

capability_profile_sha256 = SHA256(
  "nextengine.target-capability-profile.v1\0"
  || u64_le(capability_profile_bytes.len)
  || capability_profile_bytes
)

target_capability_set_bytes = JCS(TargetCapabilitySetV1.body)

target_capability_set_sha256 = SHA256(
  "nextengine.target-capability-set.v1\0"
  || u64_le(target_capability_set_bytes.len)
  || target_capability_set_bytes
)
```

`TargetCapabilitySetV1` is a validated platform-adapter snapshot, not a live
hardware query. Project resolution binds its exact hash in the
`ProjectCompositionLock` target-profile closure before content selection. At
activation the adapter MUST present the same canonical set/hash; a changed
driver, device or capability set requires a new lock resolution and cannot
trigger runtime reselection.

A target profile is supported by one capability set if and only if:

1. both exact schema refs and external hashes validate;
2. all three target-triple IDs (requested target, profile body and capability
   set) are byte-identical;
3. every required capability ID exists in the available set with the exact
   `capability_contract_sha256`;
4. no prohibited capability ID exists in the available set.

Extra available capabilities have no effect. Evaluation visits required IDs
in canonical order, then prohibited IDs in canonical order. The first failure
reason is the closed value `TargetTripleMismatch`,
`RequiredCapabilityMissing`, `CapabilityContractMismatch` or
`ProhibitedCapabilityPresent`; declaration order, device enumeration order and
marketing/version strings are never inputs.

### Profile fallback resolution

For one lock-requested `target_profile_id`, the resolver MUST perform this
exact operation before resolving payload candidates:

1. Resolve the requested profile by exact ID and initialize an empty visited
   set and ordered trace.
2. Validate its schema/profile hashes and same-target-triple constraint. A
   missing profile or malformed edge is terminal
   `CONTENT_VARIANT_PROFILE_INVALID`.
3. Append the profile ID to the trace. A repeated ID, including a self-edge,
   is terminal `CONTENT_VARIANT_FALLBACK_CYCLE`; the canonical diagnostic
   reports the cycle beginning at the first occurrence.
4. Evaluate the profile against the lock-bound `TargetCapabilitySetV1`. The
   first supported profile is the effective profile and terminates traversal.
5. If unsupported, follow exactly its `fallback_target_profile_id`. An absent
   optional field terminates with `CONTENT_VARIANT_CAPABILITY_UNSATISFIED`; a
   present ID that does not resolve is invalid under Step 2. Otherwise repeat
   Step 2. Traversal cannot exceed the manifest ceiling of 64 profiles.

The pointer chain is the only traversal order. The resolver never sorts
profiles by ID, compares feature counts, scores devices or backtracks. All
profile chains MUST be structurally valid and acyclic at manifest publication,
including chains not selected by the current lock.

A successful lock target-profile closure contains one exact:

```text
VariantFallbackPlanV1 {
  body: {
    schema_ref: SchemaRefV1,
    fallback_plan_format = "nextengine.variant-fallback-plan.v1",
    content_manifest_sha256,
    requested_target_profile_id,
    target_capability_set_sha256,
    profile_visits[],
    effective_target_profile_id,
    profile_fallback_used,
    effective_profile_closure_sha256
  },
  variant_fallback_plan_sha256
}

VariantProfileVisitV1 {
  target_profile_id,
  capability_profile_sha256,
  evaluation,
  failure_reason,
  capability_id
}
```

`VariantFallbackPlanV1.body.schema_ref.schema_id` is exactly
`nextengine.content.variant-fallback-plan`. `profile_fallback_used` is true
exactly when requested and effective profile IDs differ. `evaluation` is the
closed enum `Supported` or `Unsupported`. An unsupported visit contains its
exact closed failure reason and applicable capability ID; a supported visit
omits both. Every visit before the final one is unsupported, and the final
visit is the sole supported/effective profile. The ordered visit trace has at
most 64 entries. Runtime validates this record against its exact manifest and
capability set and consumes only the effective profile's bound selection
table; it does not repeat or alter resolution after world mutation. The plan
hash excludes its own external field:

```text
variant_fallback_plan_bytes = JCS(VariantFallbackPlanV1.body)

variant_fallback_plan_sha256 = SHA256(
  "nextengine.variant-fallback-plan.v1\0"
  || u64_le(variant_fallback_plan_bytes.len)
  || variant_fallback_plan_bytes
)
```

### Payload candidate resolution

Each `ContentPayloadCandidateV1` has one payload role, semantic class,
encoding ID, exact blob descriptor/hash and a variant key. Variant key is
either `Portable` or one exact `target_profile_id`.

For every `(asset revision, payload role, effective target profile)` the
cooker MUST:

1. select the unique candidate whose variant key equals the effective profile;
2. otherwise select the unique `Portable` candidate;
3. otherwise, only for a `WeakPresentation` role, resolve its declared
   fallback using the same effective profile;
4. otherwise terminate with `CONTENT_VARIANT_REQUIRED_MISSING`.

Two candidates with the same asset/role/variant key are ambiguous and invalid.
There is no hash, encoding, declaration-order or lexicographic tie-break:
ambiguity terminates with `CONTENT_VARIANT_AMBIGUOUS` before any portable or
weak fallback is considered. A missing payload in a supported effective
profile does not cause another profile hop; profile fallback is capability
fallback, not content-error recovery.

An asset/role weak fallback restarts Steps 1–3 on the exact declared fallback
asset/role with the same effective profile. The visited key is
`(asset revision, payload role)`; repetition is a dependency fallback cycle.
An exact portable-blob or `BuiltinFallbackRefV1` target is terminal after its
hash/contract validates. A missing, corrupt or exhausted declared weak
fallback terminates with `CONTENT_DEPENDENCY_MISSING`; no implicit second
fallback is searched. The exact selected table and its canonical hash are
bound by `ProjectCompositionLock`; runtime never searches, compiles, transcodes
or guesses another candidate.

In `VariantSelectionV1`, `source_variant_key` is the actual effective-profile
key or `Portable`. Its `fallback_used` is true exactly when a portable
candidate or declared weak fallback supplied the result. Profile-chain use is
recorded separately by
`VariantFallbackPlanV1.body.profile_fallback_used`.

Candidates sort by `(payload_role_id, variant_key, encoding_id, blob_sha256)`;
profile selections sort by
`(asset_id, record_sha256, payload_role_id, selected_blob_sha256)`. Source
declaration order is never a tie-break. Each profile's `selections` MUST equal
the result of the payload algorithm when that profile is effective; a
precomputed mismatch invalidates the manifest.

### Canonical target-variant verification vectors

| Vector | Fixture | Required result |
|---|---|---|
| `TV-CAP-PRIMARY` | Requested profile requirements are an exact subset of the capability set. | Trace contains only requested profile; it is effective; exact-profile candidate wins over portable. |
| `TV-CAP-ONE-HOP` | Requested profile has one missing or contract-mismatched requirement; its fallback is supported. | Trace is exactly requested then fallback; fallback profile is effective. |
| `TV-CAP-MULTI-HOP` | Two unsupported profiles precede one supported terminal profile. | Resolver follows pointer order through all three and selects only the third profile table. |
| `TV-CAP-PORTABLE` | Effective profile has no exact candidate and one portable candidate. | Portable candidate is selected; no further profile hop occurs. |
| `TV-CAP-NO-CONTENT-HOP` | Effective profile has no exact/portable candidate, while its profile fallback has an exact candidate. | `CONTENT_VARIANT_REQUIRED_MISSING`; content absence does not traverse the profile fallback edge. |
| `TV-CAP-WEAK` | Effective profile has no exact/portable candidate for a weak role and has one declared fallback. | Only the declared asset/role, blob or builtin fallback is selected. |
| `TV-CAP-EXHAUSTED` | Every profile in a valid terminal chain is unsupported. | `CONTENT_VARIANT_CAPABILITY_UNSATISFIED`; no lock/profile publication. |
| `TV-CAP-CYCLE` | Self-edge and multi-profile cycle fixtures. | `CONTENT_VARIANT_FALLBACK_CYCLE` with canonical cycle members; no candidate selection. |
| `TV-CAP-INVALID-EDGE` | Missing profile, cross-target edge, bad capability-profile hash or duplicate capability ID. | `CONTENT_VARIANT_PROFILE_INVALID` before candidate selection. |
| `TV-CAP-AMBIGUOUS` | Two candidates share one asset/role/variant key. | `CONTENT_VARIANT_AMBIGUOUS`; no tie-break or fallback. |
| `TV-CAP-LOCK-MISMATCH` | Activation capability-set hash differs from the lock-bound hash. | `CONTENT_VARIANT_CAPABILITY_SET_MISMATCH` before world mutation; retain prior lock/content. |
| `TV-CAP-PERMUTE` | Capability, profile, candidate and source declarations are permuted 10,000 ways. | Identical canonical sets, trace, effective profile, selection table, diagnostics and hashes. |

Only `PresentationOnly` payload may use target-specific variant keys.
`DomainRelevant` payload MUST have exactly one `Portable` candidate and the
same blob hash in every profile. For the same exact lock, Windows x86_64 and
Linux x86_64 MAY have different render/texture/audio presentation blobs, but
MUST have identical schema registry, neutral records, IDs, dependencies,
collision/navigation/world-chunk facts and `domain_closure_sha256`.

One published manifest contains the complete declared target-profile table;
host-local partial profile manifests are invalid. A supported cooker host MUST
generate or consume the verified exact blob for every declared target profile,
and the bytes for the same profile MUST be identical across cooker hosts.
Different Windows/Linux selected blobs therefore coexist in one identical
multi-target manifest rather than creating two domain catalogs.

The profile schema ID is exactly
`nextengine.content.target-variant-profile`. Its closure hash excludes its own
field:

```text
profile_projection_bytes = JCS({
  schema_ref,
  target_profile_id,
  target_triple_id,
  capability_profile_body,
  capability_profile_sha256,
  fallback_target_profile_id,
  selections
})

profile_closure_sha256 = SHA256(
  "nextengine.content-target-profile.v1\0"
  || u64_le(profile_projection_bytes.len)
  || profile_projection_bytes
)
```

## Logical manifest + blob container

`ContentBundleV1` is a logical container, not an archive ABI:

```text
ContentBundleV1 {
  envelope: {
    content_manifest_sha256,
    bundle_descriptor,
    bundle_descriptor_sha256,
    bundle_root_sha256
  },
  blobs: exact set<ContentBlobV1>
}
```

`BundleDescriptorV1.body` contains
`bundle_descriptor_format = "nextengine.bundle-descriptor.v1"`, exact
`SchemaRefV1` for `nextengine.content.bundle-descriptor`, bundle ID/revision,
semantic class, ordered asset revisions, required bundle IDs, schema refs,
blob descriptors and decoded/stored total lengths. It contains neither byte
offsets nor storage locations.

The descriptor's asset revisions MUST equal exactly the manifest entries whose
`owning_bundle_id` is this bundle. Each blob descriptor is exactly
`{ blob_identity: BlobIdentityV1, blob_identity_sha256, blob_sha256 }`, and the
set MUST equal every neutral-record blob and payload-candidate blob owned by
those entries. `required_bundle_ids` is the sorted unique set of distinct
target bundles reached by cross-bundle `ActivationRequired` edges. A
`Strong` edge across bundle IDs, a self requirement or a cycle in the required
bundle graph is invalid.

The corresponding descriptor hash is:

```text
descriptor_bytes = JCS(BundleDescriptorV1.body)

bundle_descriptor_sha256 = SHA256(
  "nextengine.bundle-descriptor.v1\0"
  || u64_le(descriptor_bytes.len)
  || descriptor_bytes
)
```

A blob is exactly:

```text
ContentBlobV1 {
  blob_identity: BlobIdentityV1,
  blob_identity_sha256,
  blob_sha256,
  stored_bytes
}
```

It has one `BlobIdentityV1` `CanonicalBinaryV1` record with
`owner_id = "nextengine.assets"`,
`schema_id = "nextengine.content.blob-identity"` and
`segment_id = "v1"`:

```text
BlobIdentityV1 {
  schema_ref: SchemaRefV1,
  blob_identity_format = "nextengine.content-blob-identity.v1",
  payload_role_id,
  semantic_class,
  encoding_id,
  target_profile_id,
  stored_length,
  decoded_length,
  decoded_sha256
}
```

`target_profile_id` is optional and MUST be absent for portable/domain
payload. `encoding_id` is an exact namespaced engine-owned codec-profile ID
whose decoder and parameters are bound by the cooker contract and target
capability profile; no ambient codec default is allowed. Unknown or
incompatible profiles are rejected. The hashes are:

```text
decoded_sha256 = SHA256(
  "nextengine.content-decoded.v1\0"
  || u64_le(decoded_bytes.len)
  || decoded_bytes
)

blob_identity_bytes = CanonicalBinaryV1(BlobIdentityV1)

blob_identity_sha256 = SHA256(
  "nextengine.content-blob-identity.v1\0"
  || u32_le(blob_identity_bytes.len)
  || blob_identity_bytes
)

blob_sha256 = SHA256(
  "nextengine.content-blob.v1\0"
  || u32_le(blob_identity_bytes.len)
  || blob_identity_bytes
  || u64_le(stored_bytes.len)
  || stored_bytes
)
```

`blob_identity_bytes` MUST be shorter than or equal to `u32::MAX`; lengths in
the identity MUST match actual bytes. Decoder recomputes both hashes and
rejects trailing bytes, decompression overflow, ratio overflow or alternate
encoding. If `stored_length = 0`, `decoded_length` MUST also be zero;
otherwise `decoded_length <= stored_length * 200` under checked `u64`
arithmetic.

Bundle root leaves consist of one descriptor leaf followed by blob leaves
sorted by `blob_sha256`. An odd node is carried with an explicit domain:

```text
descriptor_leaf = SHA256(
  "nextengine.bundle-descriptor-leaf.v1\0"
  || bundle_descriptor_sha256
)

blob_leaf = SHA256(
  "nextengine.bundle-blob-leaf.v1\0"
  || blob_sha256
)

parent = SHA256("nextengine.bundle-node.v1\0" || left || right)
carry  = SHA256("nextengine.bundle-carry.v1\0" || node)

bundle_root_sha256 = SHA256(
  "nextengine.bundle-root.v1\0"
  || u64_le(leaf_count)
  || merkle_root
)
```

At every level nodes retain left-to-right leaf order; adjacent pairs use
`parent`, an unpaired final node uses `carry`, and the operation repeats until
one `merkle_root` remains. A bundle with zero assets or zero blobs is invalid.
A physical pack MAY reorder or segment exact blobs,
but the logical descriptor, blob set and root remain unchanged. A different
encoding, cooker option or stored byte creates a different blob/root.

## Neutral schema catalog

The following registered `SchemaKeyV1` names and V1 meanings are mandatory.
Each exact `SchemaRefV1` also binds its positive monotonic `u32` version and
descriptor hash. Unknown fields are rejected rather than preserved into
runtime. All eleven entries have `schema_role = NeutralContent` and
`encoding = CanonicalBinaryV1`.

### `nextengine.content.scene`

`NeutralSceneV1` contains scene bounds, a node table, typed attachments and
root node IDs. Each `SceneNodeIdV1` is a record-scoped opaque `Id128`; nodes
sort by ID, reference parent by ID, and form an acyclic forest whose roots
equal the declared root set. A node contains `NeutralTransformV1`, semantic
tags and a bounded attachment list.

Attachment is a closed union of mesh/material binding, light definition,
audio emitter/zone, collision reference, navigation marker, trigger/region
definition or exact child-scene revision. It contains no mutable component or
runtime object. Maximums: 1,048,576 nodes, hierarchy depth 256 and 64
attachments per node.

### `nextengine.content.mesh`

`NeutralMeshV1` contains bounds, primitives and canonical attribute streams.
Required position stream uses signed micrometres. Optional normal/tangent uses
signed normalized 16-bit values; texture coordinates use signed Q16.16; color
and skin weights use unsigned normalized 16-bit; joint references use stable
`JointKeyV1`, not backend indices. Indices are zero-based `u32` local slots.

Primitive topology is the closed enum `Triangles`, `Lines` or `Points`.
Triangle winding is counter-clockwise when viewed from the front in the
canonical coordinate frame. Maximums: 16,777,216 vertices, 50,331,648 indices,
65,535 primitives, 8 UV sets and 256 morph targets. Every index MUST be in
range; each weight tuple MUST sum exactly to 65,535 after canonical
normalization.

### `nextengine.content.material`

`NeutralMaterialV1` is a semantic material graph limited to closed V1
parameters: base color, metallic, roughness, emissive, normal scale, occlusion,
alpha mode/cutoff, double-sided flag and engine-owned feature tags. Texture
slots reference exact texture revisions plus UV set and `UvTransformV1`.
It contains no shader source, compiler object, pipeline/layout descriptor or
graphics API enum.

Maximums: 128 scalar/vector parameters, 64 texture slots and 32 feature tags.
Normalized physical parameters use unsigned normalized 16-bit values; emissive
intensity uses nonnegative Q16.16 with an explicit upper bound of 65,535.

### `nextengine.content.texture`

`NeutralTextureV1` contains dimension kind, extent, layer/face/mip semantics,
color-space tag, alpha semantics, sampler-independent texel meaning and one
portable canonical pixel blob. V1 neutral texel encoding is one of
`R8Unorm`, `Rg8Unorm`, `Rgba8Unorm`, `Rgba16Unorm` or `Rgba16FloatCanonical`.
`Rgba16FloatCanonical` is IEEE-754 binary16 little-endian with semantic
negative zero normalized to positive zero; NaN and infinity are invalid.

Maximum 2D extent is 16,384 by 16,384; 3D extent is 2,048 on each axis; array
layers are at most 2,048; cube layers have exactly six faces; mip count is at
most 15 and each level has exact derived extent and byte length. Target
compression is a presentation blob variant and does not replace the neutral
pixel record.

### `nextengine.content.skeleton`

`NeutralSkeletonV1` contains a sorted joint table and declared roots.
`JointKeyV1` is a record-scoped namespaced NFC identity. Each joint references
an optional parent key, canonical bind `NeutralTransformV1` and semantic role
tags. The parent graph is acyclic; every key is unique and every parent exists
in the same skeleton revision. Target artifacts derive inverse-bind data from
this exact hierarchy and bind-transform representation under the
`cooker_contract_sha256`; inverse-bind matrices are not a second neutral source
of truth.

Maximums: 1,024 joints, 64 roots and 32 semantic tags per joint. Joint array
position is not durable identity and no physics/articulation handle is stored.

### `nextengine.content.animation`

`NeutralAnimationV1` references one exact skeleton revision and contains
integer-timed channels, markers and optional presentation root-motion intent.
Time is unsigned microseconds from zero; duration is at most
86,400,000,000 microseconds. Channels reference `JointKeyV1` and a closed
property enum `Translation`, `Rotation`, `Scale` or `MorphWeight`.

Keys sort strictly by time; duplicate time in one channel is invalid.
Translation values are signed micrometres, rotations are canonical Q1.30
quaternions, scale values are positive Q16.16 and morph weights are unsigned
normalized 16-bit. Interpolation is the closed enum `Step`, `Linear` or
`CubicHermite`; `CubicHermite` is forbidden for rotation and stores explicit
incoming/outgoing signed fixed-point tangents in value units per second
(`i64` micrometres/second for translation and signed Q16.16/second for scale
or morph). Tangent and resampling rounding is round-to-nearest, ties-to-even
under the exact cooker contract; runtime does not substitute another curve
evaluator. Maximums:
8,192 channels, 16,777,216 keys and 65,535 markers. Root motion is an authored
intent curve; it MUST NOT directly write physics pose or authoritative
transform.

### `nextengine.content.audio`

`NeutralAudioV1` contains sample rate, channel layout, frame count, loop/cue
frames, loudness metadata and one canonical PCM blob. Sample rate is
8,000..=192,000 Hz; channel count is 1..=8; duration is at most 21,600 seconds.
V1 PCM encoding is `PcmS16Le`, `PcmS24LePacked` or
`PcmF32LeCanonical`; float samples are finite and in `[-1, 1]`.

Loop/cue positions are integer frames within the clip and sort strictly.
Maximum cue count is 65,535. Device format, mixer buffer, codec-library state
and voice-service object are absent. Target encoded streams are
presentation-only variants; gameplay acoustic/timing facts remain portable.

### `nextengine.content.collision`

`NeutralCollisionV1` contains bounds, material tags and a closed shape union:
`Box`, `Sphere`, `Capsule`, `ConvexHull`, `TriangleMesh` or `HeightField`.
All geometry uses the canonical coordinate profile. Mesh/heightfield data is
inline canonical geometry or an exact neutral mesh revision plus declared
primitive range; it never references a physics backend shape.

Maximums: 65,535 shapes, 65,535 vertices per convex hull, 16,777,216 triangles
per record and 16,384 samples on either heightfield axis. Convex hulls require
at least four non-coplanar vertices; triangle indices are in range; radius and
height are positive.

### `nextengine.content.navigation`

`NeutralNavigationV1` contains traversal profile hashes and deterministic
tiles. A tile has stable record-local `u32` tile ID, quantized origin/bounds,
vertices, polygons, canonical adjacency and generic off-mesh links. Polygon
identity is exact navigation `AssetId + tile ID + local polygon ID + record
revision`; it is not a third-party polygon reference.

Off-mesh link contains stable local ID, endpoints, direction, cost, clearance
bounds and generic capability/action tag. Maximums: 65,535 tiles, 65,535
polygons per tile, 262,144 vertices and 262,144 links per tile. Polygon/edge
indices MUST be in range, adjacency MUST be reciprocal when declared
bidirectional, and every cost is unsigned Q16.16 with the inclusive bound
`0..=4_294_967_295` on its stored integer.

### `nextengine.content.world-chunk`

`NeutralWorldChunkV1` contains logical chunk ID, coordinate-profile hash,
logical coordinate as exactly three signed `i32` cells, canonical bounds,
scene/collision/navigation/audio asset revisions, typed dependency references,
persistent namespace and immutable definition seeds.

`PersistentObjectSeedV1` contains unique `PersistentId`, exact definition
asset revision, initial `NeutralTransformV1`, owner-initialization hashes and
semantic tags. It is the immutable authored seed for a new world generation;
current placement, tombstone, RPG fields, agent plan, physical pose and runtime
residency are not copied into the chunk. A save/world owner segment supersedes
the seed after initialization.

Cross-chunk references use `PersistentId` and optional exact chunk asset
revision; direct pointer and ephemeral entity ID are invalid. Maximums:
262,144 definition seeds, 65,535 required/optional chunk dependencies and
1,048,576 cross-chunk references. Duplicate `PersistentId` anywhere in one
manifest domain closure rejects the complete publication.

This record defines immutable content only. Downstream world partition,
interest, residency, activation ordering and durable placement ownership are
separate consumers and MUST NOT be encoded as mutable fields here.

### `nextengine.content.text-catalog`

`TextCatalogV1` (ADR-044) is the only neutral schema that carries localized
string payload. It contains `catalog_asset_id` with positive monotonic
`revision`, a bounded BCP-47 subset `locale` tag (ASCII lowercase,
hyphen-separated alphanumeric subtags, first subtag alpha 2–8, at most 35
characters), an optional declared `fallback_locale_or_none` and bounded
`(text_id, template)` entries sorted by stable `SchemaId` text ID without
duplicates. Templates are NFC strings whose only formatting tokens are
positional placeholders `{0}`..`{15}`, matching the typed arguments of the
UI text references that resolve against the catalog.

Locale fallback is project-declared content, not host state: every catalog
points at its fallback locale, exactly one catalog declares no fallback and
is the source/default locale, every declared fallback exists in the same
project content set, and all pointer chains are acyclic and terminate at the
source locale. Cook rejects a missing, duplicate, cyclic or unterminated
chain fail-closed before publication. Catalogs publish with semantic class
`PresentationOnly`: localized text never enters the domain closure, cannot
change gameplay hashes or replay outcomes, and a pseudo-locale is an
ordinary additional catalog, not special code. Missing resources resolve to
the declared chain and then to a readable placeholder with the stable
`LOCALIZATION_RESOURCE_MISSING` diagnostic (SPEC-18); rendered text never
becomes durable identity.

Maximums: 4,096 entries per catalog, 1,024 bytes per template. Date/unit
format rules and plural/gender forms are absent in V1 and arrive only as a
new schema version.

## Import and runtime boundary

`NeutralImportModel` remains a strict bounded subset of the common record
envelopes:

1. external process emits exact `SchemaRefV1`, canonical IDs/units,
   `ContentProvenanceV1`, declared dependency kinds and one of the ten neutral
   payloads;
2. engine validator re-encodes every record, checks structural limits,
   provenance/license policy, schema hashes, dependency closure and forbidden
   extensions;
3. source-specific diagnostic extensions MAY be inspected in private staging
   but MUST be removed before neutral record hashing;
4. common cooker receives only validated neutral envelopes and produces
   `ContentManifestV1`/bundles by the same path used for project-authored
   records;
5. runtime receives only manifest, bundle descriptors and blobs and performs
   zero source-format/source-location access.

An unknown source record is diagnosed or rejected at the import boundary; it
is never serialized as an opaque runtime component. A source locator MAY
survive only as a sanitized hash in provenance or a redacted tool diagnostic
outside runtime bundles.

## Atomic publication

Publication is one transaction:

`Parsed → Canonicalized → ClosureValidated → BundlesStaged → RootValidated → Published`,
with `Rejected` terminal for that candidate.

Before `Published`, the publisher MUST verify:

- exact schema-registry manifest and every `SchemaRefV1`;
- all record/provenance/license/cooker hashes and admission bounds;
- required, fallback and target-profile graph closure;
- `domain_closure_sha256`, every bundle descriptor/blob/root and total length;
- zero undeclared entry, forbidden public type, runtime source locator or
  case-fold-colliding logical name;
- exact project-lock content reference and target/profile compatibility.

Only after the complete generation validates may one atomic registry pointer
publish `{project_lock_sha256, content_manifest_sha256, bundle_root_set}`.
`bundle_root_set` sorts unique `(bundle_id, bundle_root_sha256)` pairs by
canonical bundle ID bytes.
Failure at any prior step discards or quarantines candidate staging and leaves
the prior published manifest/bundles byte-identical and active. A crash during
the pointer change MUST recover either the complete prior generation or the
complete new generation, never a mixture.

An already published bundle is immutable. Repair, recook, variant change,
schema migration or provenance correction creates new hashes and a new
manifest; in-place rewrite is forbidden.

## Stable diagnostics and failure semantics

Every `CONTENT_VARIANT_*` diagnostic uses the standard diagnostic envelope
with canonical context
`{ content_manifest_sha256, requested_target_profile_id,
target_capability_set_sha256, visited_target_profile_ids[],
current_target_profile_id, effective_target_profile_id, asset_revision,
payload_role_id, capability_id, reason }`; fields not applicable to the first
cause are absent, not null. Visited IDs retain pointer traversal order. Within
one profile, structural/hash errors precede capability evaluation; capability
causes use the order defined above; after an effective profile is found,
candidate ambiguity precedes missing-candidate/fallback errors. Equal invalid
inputs therefore produce one byte-identical first-cause diagnostic.

| Code / trigger | Required result |
|---|---|
| `CONTENT_MANIFEST_INVALID` | Reject the complete manifest before registry/world mutation. |
| `CONTENT_SCHEMA_MISMATCH` | Reject record/manifest before cooking or activation; retain prior exact schema/content generation. |
| `CONTENT_BOUNDS_INVALID` | Reject the affected staged record; do not clamp, truncate or partially publish. |
| `CONTENT_DEPENDENCY_CYCLE` | Reject the complete required/fallback closure with canonical cycle members. |
| `CONTENT_DEPENDENCY_MISSING` | Reject required closure; weak presentation uses only its declared fallback. |
| `CONTENT_PROVENANCE_INVALID` | Quarantine candidate and publish no record/blob. |
| `CONTENT_BUNDLE_ROOT_MISMATCH` | Quarantine the complete bundle; no blob from it becomes visible. |
| `CONTENT_BLOB_MISSING` | Reject required bundle/profile before activation; retain prior published generation. |
| `CONTENT_BUNDLE_EXTRA_ENTRY` | Reject undeclared bytes/record/blob instead of ignoring them. |
| `CONTENT_VARIANT_PROFILE_INVALID` | Reject a missing/cross-target/malformed profile edge, bad capability-profile hash or noncanonical capability set before candidate selection. |
| `CONTENT_VARIANT_FALLBACK_CYCLE` | Reject the complete profile chain and report canonical pointer-order cycle members; do not inspect payload candidates. |
| `CONTENT_VARIANT_CAPABILITY_UNSATISFIED` | Reject lock resolution when a valid profile chain terminates without one supported profile; do not choose by similarity or feature count. |
| `CONTENT_VARIANT_CAPABILITY_SET_MISMATCH` | Reject activation before world mutation when the presented capability-set hash differs from the lock-bound `VariantFallbackPlanV1`. |
| `CONTENT_VARIANT_AMBIGUOUS` | Reject target profile before publication; declaration/hash/encoding order is not a tie-break and no fallback is attempted. |
| `CONTENT_VARIANT_REQUIRED_MISSING` | Fail pre-world for a required role without hopping to another profile; optional presentation follows only its exact declared weak fallback. |
| `CONTENT_PUBLICATION_INCOMPLETE` | Discard staging and recover prior complete generation. |
| `CONTENT_HASH_COLLISION` | Security-fatal quarantine; never salt, rename or select by arrival order. |
| `CONTENT_FORBIDDEN_PUBLIC_TYPE` | Reject schema/record and report the owning boundary; do not preserve opaque runtime extension. |
| `NONDETERMINISTIC_RESULT` | Stop at the first differing record, blob or root and preserve the prior content generation. |

## Product checks

| ID | Scenario / command | Expected behavior | Fallback |
|---|---|---|---|
| CONTENT-P1 | `next check neutral-content --targets windows-x86_64,linux-x86_64 --permutations 10000` | Scene, mesh, material, texture, skeleton, animation, audio, collision, navigation and world-chunk records re-encode to byte-identical manifests/domain roots; malformed schema, ID, unit, axis, bounds, duplicate, cycle, missing dependency, provenance and forbidden-boundary cases reject; runtime opens no source format or path. | Reject or quarantine the candidate and retain the prior exact manifest; imported data may be narrowed only by an explicit mapping. |
| BUNDLE-P1 | `next check content-bundle-publication --targets windows-x86_64,linux-x86_64 --packing-permutations 10000 --fault-points all` | Packing and worker order produce one logical descriptor/blob/root set; missing, extra, corrupt, overflow, ratio, collision and hash cases reject; every publication fault exposes either the complete prior or complete new generation; all runtime roots observe the same domain closure. | Discard or quarantine staging and retain the published manifest and bundle set. |
| VARIANT-P1 | `next check target-variant-selection --targets windows-x86_64,linux-x86_64 --profile-permutations 10000` | The twelve canonical vectors and profile/candidate permutations produce exact fallback plans and selections; invalid edges, capability mismatch, ambiguity, cycle, missing required payload and undeclared fallback reject; only declared presentation blob hashes may differ by target. | Follow only the lock-bound profile chain and exact portable/weak fallback; otherwise reject the required profile and retain prior content. |

## Technical requirements

| ID | Technical requirement |
|---|---|
| REQ-120 | One exact `ContentManifestV1` MUST bind the schema registry, canonical asset revisions, provenance, typed acyclic dependency closure and domain root; runtime MUST resolve no floating revision or ambient source. |
| REQ-121 | Every published content bundle MUST be an immutable logical descriptor plus exact blob set with the specified record/blob/Merkle hashes and one atomic no-partial-publication transition shared by `game`, `headless` and `capture-worker`. |
| REQ-122 | Scene, mesh, material, texture, skeleton, animation, audio, collision, navigation and world-chunk content MUST use the bounded engine-owned neutral schemas, canonical units/axes/IDs and sanitized provenance above, with no source/tool/runtime-adapter type in public contracts. |
| REQ-123 | Target variants MUST deterministically select the first capability-supported profile in the exact lock-bound pointer chain, bind its `VariantFallbackPlanV1`, permit target differences only for `PresentationOnly` blobs, preserve one exact domain closure and resolve candidates only by exact-profile, portable, then declared weak fallback priority. |

## Failure paths

| ID | Trigger | Required result |
|---|---|---|
| FAIL-048 | Unknown/mismatched schema, noncanonical or oversized neutral record, invalid unit/axis/bounds/ID/provenance, duplicate identity, required dependency cycle/missing target, or forbidden source/tool/runtime-adapter type | Reject the affected record and complete manifest closure before cooking/publication; emit the stable first-cause diagnostic; publish no partial content or opaque approximation. |
| FAIL-049 | Missing/extra/corrupt blob, descriptor/root collision or mismatch, decoded-length/ratio overflow, publication fault, invalid capability profile/set, capability-set lock mismatch, exhausted/cyclic/cross-target profile chain, or ambiguous/missing target variant | Reject or quarantine the complete bundle/profile before visibility; emit the canonical first-cause diagnostic; expose only the complete prior or complete new generation; optional presentation may use only its exact bound weak fallback. |
