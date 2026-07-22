# ADR-007: Stable IDs, persistence и replay

| Поле | Значение |
|---|---|
| ID | ADR-007 |
| Статус | Superseded |
| Версия | 1.0.1 |
| Владелец | Runtime & Persistence Teams |
| Дата решения | 2026-07-22 |
| Последняя проверка evidence | 2026-07-23 |
| Нормативные зависимости | [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md) |
| Заменяет | отсутствует |
| Заменён | [ADR-012](012-deterministic-command-identity-and-replay.md) |

## Контекст

ECS handles оптимизированы для runtime, но нестабильны между runs и implementations. Saves, chunks, importer output, scripts и replay требуют долговечных ссылок и явной migration.

## Решение

- `RuntimeEntityId`, `PersistentId` и `AssetId` MUST быть разными nominal types и не иметь неявного преобразования.
- RuntimeEntityId MUST NOT появляться в save, replay, cooked asset, Luau API, WIT, AI IPC или importer output.
- PersistentId MUST быть 128-bit opaque value с namespace/creation rules, uniqueness validation и tombstones для удалённых durable entities. Формат не кодирует repository или game IP.
- Saves MUST иметь versioned `SaveManifest`, сегменты с checksums и explicit ordered migrations. Unknown required schema, missing content или failed migration загружаются fail-closed без изменения исходного save.
- Replay MUST фиксировать initial snapshot, seeds, build/schema/content/model hashes и ordered external WorldCommand stream. Derived DomainEvents MAY сохраняться только для диагностики и MUST сверяться, а не применяться как второй source of truth.
- AssetId разрешает logical reference; exact bytes определяются manifest + ContentHash.

## Рассмотренные варианты

- Сериализация ECS IDs отклонена как implementation coupling.
- Path-based asset identity отклонена из-за rename/casing/platform variance.
- Best-effort save loading отклонён из-за скрытой corruption и nondeterministic partial state.

## Последствия

Runtime MUST поддерживать resolver PersistentId↔RuntimeEntityId, dangling-reference diagnostics и deterministic spawn namespace. Importer MUST детерминированно отображать source identity в отдельный namespace. Migrations являются code + fixtures и не выполняются in-place над единственной копией.

## Gate для Proposed частей

Не применяется: identity/persistence/replay rules Accepted. Конкретный serialization codec MAY быть Proposed только за canonical engine schema и должен получить отдельный gate.

## Supersession

Packet 1.5 заменил это решение [ADR-012](012-deterministic-command-identity-and-replay.md), сохранив stable-ID/fail-closed semantics и уточнив causal command identity, encoding и replay ledger. Дальнейшее изменение требует нового superseding ADR и migration plan.
