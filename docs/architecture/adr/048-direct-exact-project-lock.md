# ADR-048: Direct exact project lock

| Поле | Значение |
|---|---|
| ID | ADR-048 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-08-08 |
| Последняя проверка | 2026-08-08 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-22](../22-schema-registry-compatibility-and-migration.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-047](047-simple-application-session-and-save-on-close.md) |
| Заменяет | resolver/catalog semantics и policy closure [ADR-018](018-authoritative-project-composition-and-configuration.md) |

## Контекст

Alpha имеет одного production consumer и один authored project. Public SemVer,
requirements, catalog snapshot, selected records и deterministic backtracking
не решают текущую продуктовую задачу: cooker уже знает exact content closure.
Эти contracts создавали отдельный package-resolution subsystem, дополнительные
артефакты и validation branches без второго реального package source.

Session recovery/shutdown policy также не принадлежит project composition после
ADR-047: current close policy всегда save-on-close.

## Решение

1. Tool-side authoring input имеет current-only format
   `nextengine.project-authoring.v2`. Cooker вычисляет hash exact authoring bytes
   и напрямую создаёт immutable `ProjectLockV3`.
2. `ProjectLockV3` содержит только project ID/revision, authoring hash, exact
   schema/content/world/mechanics/runtime/launch/platform profile hashes,
   canonical allowed presentation targets и собственный canonical hash.
3. Current schema registry имеет format `SchemaRegistryManifestV2`.
   Alpha не читает V1 registry и не выполняет migration.
4. Cooker публикует только `project-lock.json`, schema registry, content
   manifest, world partition, render catalog, cooked render payloads и content
   blobs. `project.json`, project catalog и composition lock не публикуются.
5. Activation читает один exact lock, проверяет generation identity, все
   опубликованные hash edges, fixed runtime/launch/platform profiles, complete
   file inventory и mechanics/render/content closure, затем атомарно возвращает
   `ActivatedProjectV3`. До успешной проверки active project отсутствует.
6. Runtime roots не выбирают versions и не имеют package resolver API.
   Package selection появится только с первым production consumer, которому
   недостаточно direct exact lock, и потребует отдельного Proposed→Accepted
   решения по ADR-046.
7. Старые authoring, project-lock, schema-registry и package-manifest formats
   возвращают typed `UNSUPPORTED_*`/invalid-package result. Они не получают
   defaults, auto-upgrade или in-place rewrite; исходные пользовательские bytes
   не удаляются.
8. `SaveManifestV2`, `WorldCheckpointV4`, `ReplayManifestV5` и
   `CommandLedgerV2` не меняют wire shape или semantics. Их существующее поле
   project-composition-lock hash теперь содержит canonical `ProjectLockV3` hash.

## Отклонённые варианты

- Сохранить catalog/resolver как private implementation — отклонено: cooker уже
  получает exact records, а скрытая вторая selection model не добавляет
  correctness.
- Оставить старые files для диагностики — отклонено: они создают две похожие
  project authorities и усложняют package inventory.
- Ввести migration V2→V3 — отклонено по ADR-046: alpha formats current-only.

## Проверка

- `cargo test -p next_contracts -p next_project --all-targets`;
- `cargo run -p xtask -- content-package`;
- `cargo run -p xtask -- host-check`;
- package build/validation на schema version 4;
- structural scan: production source не содержит project resolver/catalog,
  requirements, selected records или retired project/session policy symbols.

