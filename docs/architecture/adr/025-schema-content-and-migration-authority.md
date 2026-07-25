# ADR-025: Schema, content и migration authority

| Поле | Значение |
|---|---|
| ID | ADR-025 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-07-24 |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-01](../01-system-architecture.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-19](../19-rpg-domain-and-narrative-state.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [ADR-018](018-authoritative-project-composition-and-configuration.md), [ADR-020](020-rpg-domain-authority-and-extension-boundary.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](030-product-first-development-and-lightweight-validation.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Контекст

Project composition требует один schema registry для `game`, `headless` и optional `capture-worker`, exact registry hash в project/save/replay closure и copy-on-write migrations. Однако без отдельного решения subsystem может переиспользовать удалённый field ID, consumer может самостоятельно угадать compatibility, а migration runner — выбрать неоднозначный путь либо частично опубликовать новые segments. Такие варианты превращают representation drift, tool order и storage behavior в скрытый authoritative input.

Также требуется разделить domain meaning от publication authority. Профильный subsystem state остаётся единственным источником смысла своих fields и invariants, а `SchemaRegistryState` определяет allocation ledger, registry validation и compatibility result; `MigrationPublisher` определяет migration plan и atomic generation publication. Content становится runtime-authoritative только как immutable closure, привязанная к exact registry hash; source files, staging и consumer-local decoder rules authority не получают.

## Решение

1. Каждый engine-owned public schema имеет exact `SchemaDescriptorV1`, а один immutable `SchemaRegistryManifestV1` является единственным registry source of truth для exact project composition. `ProjectCompositionLock`, save и replay MUST связывать точный `schema_registry_manifest_sha256`; runtime roots не могут собирать registry из ambient modules или локальных decoder tables.
2. Профильный subsystem contract является единственным semantic source для своих schema fields, transitions и invariants. `SchemaRegistryState` is authoritative for issued schema, record and field identities, descriptor history and compatibility classification; `MigrationPublisher` is authoritative for the migration DAG and atomic publication. Ни один consumer не получает права переопределять class либо transform.
3. `SchemaKeyV1` равен exact pair `(schema_id, schema_version)`, где `schema_version` — positive monotonic `u32` внутри одного `schema_id`, а не SemVer и не global registry generation. `SchemaRefV1` дополнительно содержит exact `descriptor_sha256`.
4. Field identity задаётся stable `(schema_id, record_id, field_id)`. Выданный `record_id` или `field_id` MUST оставаться в cumulative ledger навсегда. Retired identity не может быть удалён, повторно выдан, реактивирован или получить другое semantic meaning или wire shape.
5. Compatibility является результатом одного closed enum: `Exact`, `BackwardCompatible`, `MigrationRequired` или `Unsupported`. Current engine не угадывает compatibility по unknown fields, source order, implementation language или best effort. Same-version hash drift, unknown schema и input вне declared support window являются `Unsupported`.
6. Schema с policy `ExactOnly` принимает только exact current `N`. Schema с policy `MigrateNMinus2` классифицирует exact `N` как `Exact`, structurally additive `N-1` как `BackwardCompatible` direct-read input и exact `N-2` как `MigrationRequired`. `N-2` имеет ровно один adjacent chain `N-2 → N-1 → N`; skip edge, alternate route, branch, cycle и source older than `N-2` запрещены.
7. `BackwardCompatible` `N-1` MAY быть активирован current reader без изменения source generation; absent newly added optional fields resolve only to descriptor-bound canonical defaults. Любая последующая запись создаёт новую atomic `N` generation. An owning Accepted subsystem MAY require explicit `N-1 → N` normalization before activating its durable state, narrowing activation without changing the compatibility class or mutating source. `MigrationRequired` `N-2` MUST пройти complete copy-on-write migration before activation. The normalization edge is the mandatory second edge of the `N-2` chain; byte-identical representation uses an identity unit.
8. Migration units образуют immutable hash-bound DAG. Каждый schema transition принадлежит ровно одному unit, dependencies объявлены явно, а canonical topological tie-break создаёт один execution order. Transform принимает immutable canonical bytes и exact hashes и выдаёт новые canonical bytes либо stable failure; wall time, worker completion и storage iteration не участвуют в результате.
9. Migration выполняется только как full logical-generation copy-on-write до world activation либо как explicit offline export. Source bytes, source manifest и current published generation остаются immutable. Все target manifests, owner segments, cross-references, content/schema hashes и invariants MUST быть полностью проверены до одного atomic publication point; partial segment, registry или pointer publication запрещена.
10. Unchanged immutable content-addressed blobs MAY сохранять тот же hash/reference, но migration MUST создать и проверить complete target generation closure. Mutable alias, in-place rewrite, best-effort field drop и publication отдельного owner segment до общей validation запрещены.
11. `game`, `headless`, `tools` и `capture-worker` используют один registry validator, compatibility classifier, migration plan contract и publication semantics. Compile-time feature, target, worker count или presentation availability не могут изменить class, route, output bytes либо failure result.
12. Public contracts содержат только engine-owned nominal IDs, positive fixed-width versions, closed enums, bounded canonical values, immutable descriptors, exact SHA-256 hashes и ordered references. Rust traits, callbacks, storage transactions and implementation handles remain private; replaceable implementation details do not enter the registry.
13. Invalid descriptor, reused identity, registry/hash drift, unsupported class, ambiguous route, transform failure, cross-owner invariant failure or publication fault MUST fail closed before activation. The exact original generation and prior published pointer remain unchanged; retry cannot turn a deterministic mismatch green.
14. This decision accepts no external technology. A future change that weakens permanent identity, widens the support window, permits consumer-selected compatibility, non-adjacent or ambiguous migration, in-place mutation or partial publication requires a superseding ADR.

## Рассмотренные варианты

- Decoder-local compatibility rules — `Rejected`: composition roots could classify the same bytes differently.
- Reusing a retired field number after a grace period — `Rejected`: old bytes would acquire new meaning.
- Semantic version ranges for persisted schemas — `Rejected`: range resolution does not identify exact wire bytes or migration history.
- Direct `N-2 → N` shortcut beside adjacent migrations — `Rejected`: it creates two valid routes and potentially two outputs.
- Best-effort unknown-field preservation without a descriptor — `Rejected`: unowned data and constraints escape validation.
- In-place migration or segment-at-a-time publication — `Rejected`: a crash can destroy the only valid generation or expose cross-owner partial state.
- Storage-specific transaction object in the public plan — `Rejected`: it leaks implementation authority across the engine-owned boundary.

## Последствия

- Every schema change carries cumulative identity history, exact descriptor hashes, compatibility entries, adjacent migration units and deterministic fixtures.
- Schema evolution is stricter and may retain tombstones indefinitely, but save/replay/content diagnosis becomes exact and old field identities cannot be reinterpreted.
- Cross-owner migrations require one complete target-generation validation and one publication point; subsystem transforms remain pure and cannot publish independently.
- Project locks and artifacts gain exact registry and migration roots while implementation-specific lookup, storage and execution mechanisms stay private.

## Product checks

| Scenario | Expected | Fallback |
|---|---|---|
| Registry history includes retired/reused IDs, same-version hash drift, unknown fields and unsupported versions | Invalid descriptors are rejected before activation; every supported input has one exact compatibility class | Keep the prior exact registry and project lock |
| `N`, additive `N-1` and exact `N-2 → N-1 → N` fixtures, including ambiguous and cyclic routes | Classification and output bytes are deterministic; exactly one adjacent route exists when migration is required | Use a compatible prior reader or explicit validated export |
| Cross-segment migration with transform, validation or publication fault injection | The complete target generation publishes once only after all hashes, references and invariants pass | Discard staging and retain the original save/content generation |

## Supersession

ADR-025 does not supersede an earlier decision and coexists with ADR-018, ADR-020, ADR-022 and ADR-030. Changing permanent identity, closed compatibility semantics, exact `N`, `N-1`, `N-2` support, unique adjacent migration or all-or-nothing copy-on-write publication requires a new superseding ADR and corresponding updates to affected specifications.
