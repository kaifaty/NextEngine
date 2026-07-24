# SPEC-22: Schema registry, compatibility и migration

| Поле | Значение |
|---|---|
| ID | SPEC-22 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | Asset & Persistence Team |
| Требуемые согласующие | Repository Owner, Architecture Working Group, Runtime Team, RPG Framework Team, World Services Team, Gameplay Extensibility Team, Security & Governance Team, Verification & Evidence Team, Release Engineering |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [ADR-018](adr/018-authoritative-project-composition-and-configuration.md), [ADR-020](adr/020-rpg-domain-authority-and-extension-boundary.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-024](adr/024-requirement-gate-evidence-and-profile-closure.md), [ADR-025](adr/025-schema-content-and-migration-authority.md) |
| Заменяет | отсутствует |

## История принятия

SPEC-22 принят в architecture packet 1.8 как schema-registry and migration foundation contract. Принятие документа не создаёт runtime implementation, verification evidence, gate `PASS`, `vertical-v1` conformance или release readiness.

## Назначение и invariants

SPEC-22 задаёт один engine-owned schema contract для content, project composition, authoritative state, save/replay, commands, events, immutable projections, process protocols и evidence.

- Exact `SchemaRegistryManifestV1` MUST быть общим для `game`, `headless`, `tools` и `capture-worker`.
- Каждый schema/record/field identity MUST быть immutable после allocation и MUST NOT переиспользоваться после retirement.
- Compatibility MUST вычисляться только closed classifier по exact descriptor hashes.
- Current per-schema version `N` MUST применять только policy `ExactOnly` либо exact support window `N`, `N-1`, `N-2`.
- Каждый supported older persisted schema MUST иметь один adjacent deterministic route к `N`, включая explicit identity migration для byte-identical representation.
- Migration MUST создавать полную isolated target generation copy-on-write и публиковать её только целиком.
- Original bytes, previous published generation и active project lock MUST оставаться неизменными при любой ошибке.
- Registry and migration contracts MUST NOT содержать implementation-owned public types.

## Source of truth и ownership

| State | Единственный owner/source of truth | Не является authority |
|---|---|---|
| Field meaning and domain invariants | Owning subsystem named by exact descriptor | consumer decoder, migration runner, presentation or cache |
| Schema, record and field identity allocation | Asset & Persistence Team cumulative allocation ledger | source declaration order, code symbol or local module registry |
| Exact schema bytes and history | `SchemaDescriptorV1` plus exact descriptor hash | generated language type or documentation label |
| Active registry and compatibility table | Atomically published `SchemaRegistryManifestV1` | dynamic plugin scan, package cache or process-local map |
| Migration route and order | Hash-bound migration DAG in the exact registry manifest | filesystem discovery, worker completion or tool order |
| Published content/save generation | Exact immutable generation manifest and atomic current pointer | staging output, partially transformed segment or inspector view |

The owning subsystem authors semantic constraints and pure migration transforms. Asset & Persistence Team allocates identities, validates cumulative history, classifies compatibility, builds the migration plan and owns all-or-nothing publication. Runtime Team consumes only a validated exact registry and cannot override its result. A derived decoder table or storage index is reconstructible and cannot accept an independent write.

## Common identity and hash rules

`schema_id` is a lowercase ASCII dotted identifier of 3 to 128 bytes matching `^[a-z][a-z0-9]*(\.[a-z][a-z0-9-]*)+$`. `schema_version` is a positive monotonic `u32` scoped to one `schema_id`; it is not SemVer and does not imply a shared global `N`.

```text
SchemaKeyV1 {
  schema_id,
  schema_version
}

SchemaRefV1 {
  schema_id,
  schema_version,
  descriptor_sha256
}
```

`SchemaKeyV1` orders first by raw ASCII `schema_id` bytes and then by unsigned `schema_version`. `SchemaRefV1` is an exact reference, never a range. All SHA-256 values are 32 raw bytes in `CanonicalBinaryV1` and lowercase 64-character hexadecimal strings in JCS.

Every descriptor and manifest uses a closed outer envelope so its own hash is not self-referential:

```text
descriptor_sha256 = SHA-256(JCS(SchemaDescriptorV1.body))

schema_registry_manifest_sha256 =
  SHA-256(JCS(SchemaRegistryManifestV1.body))
```

The external hash field is not part of `body`. JCS validation follows ADR-022: duplicate keys, non-NFC designated strings, invalid Unicode, non-canonical numbers, negative zero and unknown fields fail before hashing. Arrays whose order is declared canonical MUST already be in that order; the validator does not silently sort malformed input.

## `SchemaDescriptorV1`

`SchemaDescriptorV1` is an immutable pair `{ body, descriptor_sha256 }`. Its body contains exactly:

| Field | Contract |
|---|---|
| `descriptor_schema` | Exact constant `nextengine.schema-descriptor.v1`. |
| `schema_id`, `schema_version` | Exact `SchemaKeyV1`; version is positive and advances by one for every wire or semantic contract change. |
| `owner_context_id` | One stable engine-owned bounded-context ID used as the schema-level default owner. |
| `schema_role` | Closed `SchemaRoleV1`. |
| `encoding` | Closed `SchemaEncodingV1`. |
| `compatibility_policy` | Closed `SchemaCompatibilityPolicyV1`. |
| `maximum_canonical_bytes` | Positive `u64` upper bound validated before allocation or decode. |
| `next_record_id` | Next never-issued positive `u32`; all smaller issued record IDs remain represented in `records`. |
| `records` | Non-empty record ledger ordered by ascending `record_id`. |
| `schema_references` | Unique exact `SchemaRefV1` values ordered by `SchemaKeyV1`, then descriptor hash. |
| `semantic_constraints_sha256` | Hash of the complete engine-owned invariant contract applied after structural validation. |

`SchemaRoleV1` is the closed enum `AuthoritativeState`, `Command`, `DomainEvent`, `Manifest`, `NeutralContent`, `ImmutableProjection`, `ProcessProtocol` or `Evidence`. `SchemaEncodingV1` is the closed enum `CanonicalBinaryV1` or `JcsRfc8785`. `SchemaCompatibilityPolicyV1` is the closed enum `ExactOnly` or `MigrateNMinus2`; no descriptor may declare an arbitrary numeric window.

Unknown enum tag, missing field, extra field, zero version, zero bound, duplicate reference, reference range or reference to a descriptor absent from the same registry closure invalidates the descriptor.

Any change to descriptor `body`, including owner, role, encoding, bounds, references, constraints or allocation ledgers, MUST create exactly the next `schema_version`. Two different descriptor hashes for one `SchemaKeyV1` are never revisions or patches; they are conflicting bytes and classify as `Unsupported`.

Every active field has exactly one effective semantic owner recorded in its own `owner_context_id`. That value normally equals the schema default; an explicitly allocated child owner replaces the default for that field rather than adding a second owner. The exact effective mapping must match `ownership_registry_sha256`.

### Record and field allocation ledger

Every `RecordDescriptorV1` contains:

```text
RecordDescriptorV1 {
  record_id,
  record_state,
  canonical_name,
  introduced_in,
  retired_in,
  next_field_id,
  next_variant_tag,
  fields,
  variants
}
```

`record_id` is a positive `u32` permanently scoped to `schema_id`. `record_state` is `Active` or `Retired`. A retired record retains its complete last active field and variant ledger, has exact `retired_in`, and can never become active again. A new record MUST take exact `next_record_id`; publication increments the counter with checked arithmetic. Gaps, counter rollback, duplicate ID and exhausted `u32` allocation fail closed.

Field identity is:

```text
FieldKeyV1 = (schema_id, record_id, field_id)
```

Each `FieldDescriptorV1` contains exact `field_id`, `field_state`, `canonical_name`, `semantic_id`, `wire_shape`, `presence`, optional bounded `canonical_default`, `owner_context_id`, `introduced_in`, optional `retired_in` and `constraint_sha256`.

- `field_id` is a positive `u32`. A new field MUST take exact `next_field_id`, after which the counter advances with checked arithmetic.
- Every issued `field_id` below `next_field_id` appears exactly once in the cumulative ledger. `Active` and `Retired` are the only states.
- A retired field remains in every later descriptor. It cannot be removed, reused, reactivated or assigned a new `semantic_id`.
- `semantic_id` and `wire_shape` are immutable for one `FieldKeyV1`. A different meaning or shape requires retiring the old field, allocating the next ID and registering an adjacent migration.
- `canonical_name`, presence, default, owner or constraint change increments `schema_version` and is never `BackwardCompatible`, except an allowed new optional field described below. It is valid only for a source version explicitly classed `MigrationRequired`; otherwise the pair is `Unsupported`.
- `Required` and `Optional` are the only presence values. An optional field MUST contain an exact canonical default. A required field MUST use `None`.
- Retired entries retain their last active name, semantic ID, shape, owner, constraints and introduction version plus exact retirement version.

`WireShapeV1` is a closed recursive enum aligned with ADR-022 logical values: `Unit`, `Bool`, fixed-width `U8`, `U16`, `U32`, `U64`, `I8`, `I16`, `I32`, `I64`, `F32Bits`, `F64Bits`, `Bytes`, `Utf8Nfc`, `Id128`, `Hash256`, `Struct(record_id)`, `Sequence(element)`, `Map(key,value)`, `Set(element)`, `Option(element)` or `TaggedUnion(record_id)`. Variable-size shapes MUST have exact bounds covered by `constraint_sha256`. Authoritative roles retain ADR-022 non-finite, checked arithmetic and quantization restrictions.

Tagged-union variants use permanent `u8` tags with the same `Active` or `Retired` tombstone rule. `next_variant_tag` is a `u16` counter in `0..=256`; when it is below `256`, a new variant takes its exact value as the `u8` tag and advances the counter, while `256` means exhausted. Variant `0` is valid only when allocated by this rule. Every issued tag remains represented exactly once, and an exhausted tag space requires a new schema identity rather than reuse. CanonicalBinary fields use `field_id` as the field record key. JCS fields use exact `canonical_name` as the member key while `FieldKeyV1` remains their compatibility identity; a JCS key rename therefore requires migration.

The registry validator compares every successive descriptor. It rejects removed tombstones, decreased counters, reused IDs, changed semantic ID or shape, version skips without exact history, and any field whose introduction or retirement version contradicts the descriptor sequence.

## Closed compatibility contract

`SchemaCompatibilityClassV1` is exactly:

| Class | Exact meaning |
|---|---|
| `Exact` | Source and target `SchemaRefV1` are byte-for-byte equal. No transform is needed and the route is empty. |
| `BackwardCompatible` | Source is exact `N-1`; every source field keeps the same identity, name, shape, presence, owner and constraints, and target changes are only newly allocated optional fields with exact canonical defaults. Current reader may activate an immutable logical view without rewriting source bytes. |
| `MigrationRequired` | Source is exact `N-2` and exactly one registered adjacent route performs the complete transformation to current `N`, including any retirement, rename, required-field derivation, presence/default/owner/constraint change, field split/merge or cross-schema movement. |
| `Unsupported` | Every other case, including unknown schema, policy denial, version outside the window, same-key hash drift, missing descriptor, ambiguous route or forbidden identity mutation. |

Classification is directional from exact source ref to exact current target ref. The classifier applies the version rule first, then structural and route validation: exact current ref is `Exact`; exact `N-1` under `MigrateNMinus2` is `BackwardCompatible` only when the additive rule holds; exact `N-2` under that policy is `MigrationRequired` only when the unique two-edge route exists; every other pair is `Unsupported`. A manifest declaration that differs from the recomputed class is invalid. Consumer-specific downgrade, tolerant reader, unknown-field preservation or fallback class is forbidden.

For each current descriptor version `N`, rules are exact:

| Policy and source | Required result |
|---|---|
| `ExactOnly`, exact `N` ref | `Exact`; empty route. |
| `ExactOnly`, any other ref | `Unsupported`; no decode for activation. |
| `MigrateNMinus2`, exact `N` ref | `Exact`; empty route. |
| `MigrateNMinus2`, structurally additive exact `N-1` ref | `BackwardCompatible`; direct read through current descriptor is allowed, source bytes remain immutable and no migration route is required for activation. |
| `MigrateNMinus2`, exact `N-1` ref that violates the additive rule | `Unsupported`; the candidate cannot promise this policy and no tolerant read is allowed. |
| `MigrateNMinus2`, exact `N-2` ref | `MigrationRequired`; exactly two transitions `N-2 → N-1` and `N-1 → N`. |
| Any policy, source newer than `N`, older than `N-2`, version zero, unknown ref or same version with another hash | `Unsupported`; fail before authoritative mutation. |

When `N` is `1`, no historical version is synthesized. When `N` is `2`, only `N-1` exists. A schema newly introduced at current `N` has no fabricated older descriptor. The support window is per schema identity and never inferred from a registry revision.

`BackwardCompatible` `N-1` permits direct activation of a current logical view without modifying or republishing the source generation. Newly added optional fields read only their descriptor-bound canonical defaults; no other difference is allowed. A later save or explicit normalization writes a complete new `N` generation atomically. An owning Accepted subsystem MAY require that normalization before activating its own durable state; this only narrows activation and cannot change the class, route or source bytes. `MigrationRequired` `N-2` cannot become active until the exact route has produced and atomically published a complete `N` generation. An explicit `N-1 → N` normalization edge remains required as the second edge of every `N-2` route, and uses an identity migration when canonical payload bytes are unchanged.

## `SchemaRegistryManifestV1`

`SchemaRegistryManifestV1` is an immutable pair `{ body, schema_registry_manifest_sha256 }`. Its body contains exactly:

| Field | Contract |
|---|---|
| `manifest_schema` | Exact constant `nextengine.schema-registry-manifest.v1`. |
| `registry_revision` | Positive monotonic `u64` for this registry publication; it does not replace per-schema versions. |
| `canonicalization_profile_sha256` | Exact ADR-022 JCS and `CanonicalBinaryV1` profile closure. |
| `ownership_registry_sha256` | Exact one-owner mapping used by every descriptor and migration unit. |
| `descriptors` | Full current and supported historical `SchemaDescriptorV1` entries ordered by `SchemaKeyV1`, then descriptor hash. |
| `current_schema_refs` | Exactly one current `SchemaRefV1` for every active `schema_id`, ordered by schema ID. |
| `compatibility_entries` | Exact source-to-current classifications ordered by target key, source key and source hash. |
| `migration_units` | Unique `MigrationUnitDescriptorV1` values ordered by `migration_id` bytes. |
| `migration_dependency_edges` | Unique ordered pairs `(predecessor_id, successor_id)` in canonical byte order. |
| `migration_dag_sha256` | Hash of canonical units, transitions and dependency edges. |
| `registry_limits_sha256` | Exact bounds for descriptor count, record/field count, canonical bytes, migration work and target-generation size. |
| `fixture_set_sha256` | Exact positive, negative and power-fault fixture closure required by the three canonical gates. |

The manifest MUST include exact current descriptor plus `N-1` and `N-2` descriptors when they exist and are admitted by `MigrateNMinus2`. Older descriptor bodies MAY remain in a separately content-addressed historical audit archive, but they are not activation inputs and cannot widen compatibility. Current tombstone ledgers preserve all issued record, field and variant IDs even when older descriptor bodies leave the active window.

Each `CompatibilityEntryV1` contains source ref, current target ref, declared class and exact ordered `migration_id` route. `Exact` and `BackwardCompatible` have an empty activation route. `MigrationRequired` has the exact two-unit `N-2 → N-1 → N` route. The registry still carries the unique `N-1 → N` normalization unit used by that route and by explicit offline normalization. `Unsupported` is the closed default for absent or invalid pairs and is not expanded into an unbounded entry list.

`migration_dependency_edges` MUST be byte-for-byte equal to the canonical projection of every unit's `dependency_ids`; disagreement invalidates the manifest. The DAG root is non-self-referential:

```text
migration_dag_sha256 = SHA-256(JCS({
  migration_units,
  migration_dependency_edges
}))
```

The manifest hash is included in `ProjectCompositionLock`, `SaveManifest`, `ReplayManifest`, cooked content closure and migration plan. The registry body MUST NOT include any of those enclosing root hashes, preventing a hash cycle. An enclosing artifact with a different registry hash is incompatible until an explicit migration creates a new exact closure.

## Unique deterministic migration DAG

Each `MigrationUnitDescriptorV1` contains:

```text
MigrationUnitDescriptorV1 {
  migration_id,
  primary_owner_context_id,
  contributor_owner_context_ids[],
  transitions[],
  dependency_ids[],
  transform_kind,
  transform_contract_sha256,
  preconditions_sha256,
  postconditions_sha256,
  resource_policy_sha256,
  fixture_set_sha256
}
```

`migration_id` is a stable namespaced NFC identifier. `transform_kind` is `Identity` or `CanonicalTransform`. A unit has at least one `SchemaTransitionV1 { source_ref, target_ref }`; each transition keeps the same `schema_id` and satisfies `target.schema_version = source.schema_version + 1`. One unit cannot contain two transitions from the same schema lineage. A cross-owner unit MAY move logically related fields only when every affected owner is listed and the postconditions prove single ownership; Asset & Persistence Team remains publication owner.

The registry is valid only when all of the following hold:

1. Every supported non-current source ref has exactly one outgoing adjacent transition toward its current ref.
2. Every transition belongs to exactly one migration unit and exact source and target descriptors exist in the manifest.
3. No direct `N-2 → N`, skipped version, duplicate edge, alternate path, branch toward different current hashes or cycle exists.
4. Every `N-2` route contains exactly its `N-2 → N-1` transition followed by its `N-1 → N` transition; every `N-1` route contains exactly the latter transition.
5. Unit dependencies include all schema-reference and declared cross-owner preconditions. An undeclared read, emitted schema or owner write is invalid.
6. The dependency graph is acyclic. Execution order is the unique Kahn topological order that repeatedly selects the ready unit with lexicographically smallest tuple `(migration_id bytes, transition source keys, transition target keys)`.
7. Equal source generation, target registry, transform-contract, fixture, engine-build and policy hashes produce byte-identical target bytes, diagnostics and generation root on Windows and Linux regardless of worker count.
8. `Identity` requires identical canonical payload bytes and still validates target descriptor and postconditions. `CanonicalTransform` consumes and emits only bounded canonical values.

Rust traits, function pointers, callbacks, asynchronous task values, storage handles and implementation objects are not part of a migration unit. Internal implementations are selected only after exact IDs and hashes validate and cannot change the public plan.

## Full copy-on-write migration protocol

Migration occurs pre-world or in an explicit offline tool. It is never a gameplay command, emits no gameplay `DomainEvent` and does not increment domain revisions or rewrite causal history unless an owning schema contract explicitly defines a new migration provenance record.

The exact protocol is:

1. Freeze exact source generation reference and verify project, registry, content, owner-segment, definition, package, command-ledger and state hashes before any transform.
2. Resolve one target registry and recompute every compatibility class. Any `Unsupported`, missing or ambiguous entry aborts before staging.
3. Construct immutable `MigrationPlanV1` containing source and target generation hashes, source and target registry hashes, canonical ordered unit IDs, affected owner segments, resource policy and expected target closure.
4. Allocate a private working generation. Source bytes and current publication pointer remain read-only for the entire operation.
5. Copy every logical owner segment and generation manifest into the working closure. Unchanged immutable content-addressed blobs MAY be referenced by the same exact hash, but no mutable object or partially rewritten segment is shared with the source.
6. Execute migration units in canonical DAG order. Inside a unit, records are processed by exact tuple `(schema_id bytes, record_id, durable nominal ID bytes, field_id)`; the unit writes only its declared isolated target set.
7. After each unit, validate target descriptors, bounds, owner-local invariants and step hashes. Do not publish an intermediate unit, segment, registry or pointer.
8. After all units, validate the complete generation: every record resolves to a current exact `SchemaRefV1`; no old/unknown field remains active; all cross-owner references, revisions, causal history, tombstones, content/package/lock compatibility and generation limits pass; then compute every segment hash and target generation root.
9. Persist the full working generation and validate it by reopening through the production reader. Only then atomically publish one current-generation pointer.
10. Decode, compatibility, transform, bound, invariant, hash, write, reopen, power-loss or pointer-swap failure discards or quarantines staging and leaves source bytes, source hash and previous published pointer unchanged.

A process crash yields either the prior valid generation or the complete new generation, never a mix. Recovery never resumes from an unauthenticated intermediate transform. Re-running the same exact plan MUST reproduce the same target root; a different result is `NONDETERMINISTIC_RESULT` and remains blocking.

## Composition-root and public boundary

`game`, `headless` and `capture-worker` validate the same exact registry hash before activation and share one compatibility and migration application service. `tools` may author, validate and execute an offline plan but cannot publish a registry or generation that runtime would reject. No root may compile out a required descriptor or transform while claiming the same project lock.

The public boundary is limited to `SchemaKeyV1`, `SchemaRefV1`, `SchemaDescriptorV1`, `SchemaRegistryManifestV1`, closed compatibility and migration descriptors, immutable `MigrationPlanV1`, canonical values, nominal IDs and hashes. It contains no ECS storage/component value, operating-system object, vendor value, importer representation, filesystem path, database connection/row, task/future handle, raw pointer or backend object. Language bindings and internal Rust traits are generated private adapters, not schema authority.

## Stable diagnostics и failure semantics

| Code / failure | Required outcome |
|---|---|
| `SCHEMA_DESCRIPTOR_INVALID` | Reject missing, unknown, unordered, unbounded or hash-invalid descriptor before registry publication. |
| `SCHEMA_IDENTITY_REUSED` | Reject removed tombstone, reused/reactivated record, field or variant ID, counter rollback or semantic/wire-shape reassignment. |
| `SCHEMA_REGISTRY_MISMATCH` | Reject project/content/save/replay activation on manifest hash or closure mismatch; preserve prior exact lock/generation. |
| `SCHEMA_COMPATIBILITY_UNSUPPORTED` | Reject unknown, future, older-than-window, policy-denied or same-version-different-hash input before mutation. |
| `SCHEMA_COMPATIBILITY_AMBIGUOUS` | Reject class disagreement, missing descriptor or more than one valid route; no best-effort selection. |
| `SCHEMA_MIGRATION_DAG_INVALID` | Reject skip, branch, cycle, duplicate transition, undeclared dependency or non-canonical order before staging. |
| `SCHEMA_MIGRATION_FAILED` | Discard the complete working generation; preserve exact source bytes and previous publication. |
| `SCHEMA_MIGRATION_PUBLICATION_FAILED` | Reopen the prior pointer or leave it unchanged; never expose a partial target closure. |
| `NONDETERMINISTIC_RESULT` | Gate fails with the first differing descriptor, unit, record, field or generation root; retry cannot create `PASS`. |

## Canonical gate descriptors

These are `AcceptedBaseline`, `Blocking` gates. SPEC-22 is their only `descriptor_source_id`; other documents may aggregate or reference them but MUST NOT redefine their semantic descriptor.

| Gate | Primary owner | Contributors | Reproducible command/scenario | Pass threshold | Required evidence | Fallback | VS / profile closure |
|---|---|---|---|---|---|---|---|
| `SCHEMA-P1` | Asset & Persistence Team | Runtime Team, Security & Governance Team | `next gate SCHEMA-P1 --scenario schema-registry-v1 --targets windows-x86_64,linux-x86_64 --roots game,headless,capture-worker --fixtures 10000` | 10,000 valid declaration/order permutations produce byte-identical descriptor and registry bytes/hashes on both targets and all roots; 100% duplicate, gap, counter rollback, removed tombstone, reused/reactivated ID, hash/order/bound/reference and forbidden-public-type fixtures reject before publication; all roots report one exact registry hash | descriptor and registry corpus, cumulative record/field/variant allocation ledger, cross-target golden bytes/hashes, root manifests, ownership graph, public API/schema scan and diagnostics | reject candidate registry and retain the prior exact registry and ProjectCompositionLock; gate remains blocking | VS-01, VS-02, VS-11 |
| `COMPAT-P1` | Asset & Persistence Team | Runtime Team, Release Engineering | `next gate COMPAT-P1 --scenario schema-compatibility-v1 --versions n-2,n-1,n,n+1 --permutations 10000` | 100% corpus pairs classify exactly one of `Exact`, `BackwardCompatible`, `MigrationRequired`, `Unsupported`; `N` is exact, additive `N-1` is direct-read backward-compatible, `N-2` requires the exact two-edge migration, while non-additive `N-1`, same-version hash drift, future, older and policy-denied input are unsupported over 10,000 order/target permutations; 0 consumer-specific or ambiguous result | compatibility matrix, exact source/target descriptor hashes, structural diff reports, route proofs, negative corpus, cross-target classifier bytes and diagnostics | reject unsupported input; use the exact current generation, a prior compatible executable or explicit validated export; no tolerant activation | VS-01, VS-02, VS-08 |
| `MIGRATION-P1` | Asset & Persistence Team | Runtime Team, RPG Framework Team, World Services Team, Gameplay Extensibility Team | `next gate MIGRATION-P1 --scenario schema-generation-migrations --versions n-2,n-1,n --targets windows-x86_64,linux-x86_64 --roots game,headless,capture-worker --power-faults all` | Every valid `N-2 → N-1 → N`, `N-1 → N` and exact `N` fixture produces one byte-identical complete target-generation root across targets, roots and worker counts; 100% missing, duplicate, skip, alternate, branch, cycle, undeclared dependency/write, transform, bound, cross-owner, reopen and power/publication faults expose 0 partial registry/segment/pointer and preserve exact original bytes/hash and prior generation | registry and migration DAG manifests, canonical plans/unit orders, transform-contract and fixture hashes, before/after owner segments and full generation roots, cross-owner invariant report, fault/power matrix, reopen audit, publication trace and preserved-original proof | discard or quarantine the entire working generation, retain the original and prior published pointer, and use a prior compatible executable or explicit validated export; no partial retry | VS-02, VS-11, VS-13 |

Unavailable implementation evidence is non-PASS. Neither an agent, retry nor human decision can synthesize success or waive an automatic failure.

## Requirements

| Requirement | Нормативное требование | Primary owner | Contributors / required approvers | Blocking gates | Required evidence | Fail-closed fallback | VS / profile closure |
|---|---|---|---|---|---|---|---|
| `REQ-112` | Every engine-owned schema MUST have one exact `SchemaDescriptorV1`; every schema, record, field and variant identity MUST remain cumulative, immutable and non-reusable in one hash-bound `SchemaRegistryManifestV1` shared by all required roots. | Asset & Persistence Team | Repository Owner, Runtime Team, Security & Governance Team | `SCHEMA-P1` | Descriptor/registry golden corpus, cumulative allocation ledger, ownership graph, root registry hashes and public-boundary scan. | Reject the candidate registry before activation and retain the prior exact registry and ProjectCompositionLock. | VS-01, VS-02, VS-11 |
| `REQ-113` | Compatibility MUST use only the closed classes: exact `N` is `Exact`, structurally additive `N-1` is direct-read `BackwardCompatible`, exact `N-2` is `MigrationRequired`, and non-additive `N-1`, same-version hash drift, unknown, future and older input are `Unsupported`. | Asset & Persistence Team | Runtime Team, Release Engineering | `COMPAT-P1` | Compatibility matrix, source/target descriptor hashes, structural diff and route proofs, negative corpus and deterministic classifier report. | Reject unsupported input without mutation and require exact current bytes, a prior compatible executable or explicit validated export. | VS-01, VS-02, VS-08 |
| `REQ-114` | Every supported older persisted schema MUST resolve through one hash-bound adjacent migration DAG with one canonical order; `N-2` MUST traverse exact `N-2 → N-1 → N` and no skip, alternate, branch or cycle. | Asset & Persistence Team | RPG Framework Team, World Services Team, Gameplay Extensibility Team | `MIGRATION-P1` | Registry/DAG manifest, unique-route proof, canonical unit plan/order, transform/fixture hashes and cross-target target roots. | Reject the route before staging, retain the source generation and use a prior compatible executable or explicit validated export. | VS-02, VS-11, VS-13 |
| `REQ-115` | Migration MUST build, validate, reopen and publish one complete copy-on-write target generation atomically; any fault MUST expose no partial registry, owner segment or current pointer and MUST preserve exact original bytes/hash. | Asset & Persistence Team | Runtime Team, Release Engineering, Security & Governance Team | `MIGRATION-P1` | Before/after full-generation manifests and roots, owner/cross-reference validation, fault/power matrix, reopen/publication audit and preserved-original proof. | Discard or quarantine the entire working generation and retain the original bytes and prior published pointer. | VS-02, VS-11, VS-13 |

## Failure paths

| Failure requirement | Failure / trigger | Primary owner | Contributors / required approvers | Нормативный путь | Blocking gates | Required evidence | Fail-closed fallback | VS / profile closure |
|---|---|---|---|---|---|---|---|---|
| `FAIL-044` | Invalid/tampered registry or descriptor, missing tombstone, reused identity, incompatible exact hash, unsupported version or ambiguous compatibility class | Asset & Persistence Team | Runtime Team, Security & Governance Team | Reject before registry, project, content, save or replay activation; publish no inferred descriptor/class and preserve the prior exact registry, lock and generation. | `SCHEMA-P1`, `COMPAT-P1` | Descriptor/registry and compatibility negative corpus, allocation ledger diff, hash/route proof, activation audit and unchanged prior-root proof. | Retain the prior exact registry and generation; require exact supported input or a validated offline export. | VS-01, VS-02, VS-08, VS-11 |
| `FAIL-045` | Missing, duplicate, skipped, branching or cyclic migration; transform, bound, cross-owner, write, reopen, crash or publication fault | Asset & Persistence Team | Runtime Team, RPG Framework Team, World Services Team, Release Engineering | Abort the whole plan, discard or quarantine the isolated working generation, publish no segment/registry/pointer subset and preserve exact original bytes/hash and prior generation. | `MIGRATION-P1` | DAG negative corpus, canonical plan/unit trace, transform and cross-owner validation, all-boundary fault/power matrix, publication/recovery audit and preserved-original hashes. | Keep the original and prior published pointer immutable and use a prior compatible executable or explicit validated export. | VS-02, VS-11, VS-13 |

## Architecture-admission boundary

This specification allocates contracts, requirements, failures and gate descriptors only. Runtime suites, migration binaries, fixture artifacts and generation publications remain future implementation work. Architecture packet admission does not imply `SCHEMA-P1`, `COMPAT-P1`, `MIGRATION-P1`, any vertical gate or release readiness has passed.
