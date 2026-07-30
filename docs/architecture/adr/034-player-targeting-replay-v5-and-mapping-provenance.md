# ADR-034: Player targeting replay V5 и exact mapping provenance

| Поле | Значение |
|---|---|
| ID | ADR-034 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-07-30 |
| Последняя проверка | 2026-07-30 |
| Нормативные зависимости | [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-18](../18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](../22-schema-registry-compatibility-and-migration.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [ADR-019](019-canonical-player-actions-and-presentation-authority.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-025](025-schema-content-and-migration-authority.md), [ADR-032](032-grounded-capsule-physics-checkpoint-version-boundary.md) |
| Заменяет | частично ADR-032: обозначение `ReplayManifestV4` как current generated replay и соответствующее последствие о version boundary; не заменяет правила physics snapshot V1/V2 или replay V2 |
| Заменён | не заменён |

## Контекст

`ReplayManifestV4` связывает closed ingress, command batches, physics step,
contacts, events и authoritative roots, но не сохраняет exact факты нового
player-targeting path. В частности, из V4 нельзя доказать, какой assigned
semantic action создал targeting intent, какой authoritative physics query был
выполнен и какие ordered query results определили command outcome.

Legacy `InputMappingReceiptV1` также содержит не более одного optional
`derived_command_id`. Один canonical player frame теперь может одновременно
породить movement command и interaction outcome command. Добавление этих полей
в существующие V1/V4 bytes нарушило бы exact-only правило ADR-025/SPEC-22.

## Решение

- Current generated replay MUST использовать `ReplayManifestV5`. V5 повторно
  использует те же nested canonical checkpoint/legacy-stage representations и
  compare values; сам V5 envelope несовместим с V4. Каждый tick дополнительно
  хранит:
  - ordered `TargetingIntentV1` и `AuthoritativeTargetingQueryV1`;
  - один closed `PhysicsQueryBatchV1` и exact ordered
    `PhysicsQueryResultV1`;
  - `InputMappingReceiptV2` для каждого mapped player frame;
  - compare hashes query batch, query results и targeting intent/query trace.
- `InputMappingReceiptV2` MUST связывать assigned tick, logical source
  identity/sequence и payload hash с ordered per-action mapping results.
  Каждый accepted action содержит contiguous command range, а каждый command
  reference — canonical command ordinal, source action ordinal, mapper slot и
  exact `CommandId`. Composite frame не может терять второй command или
  приписывать command другому action.
- V5 validation MUST до replay проверять schema versions, bounds, canonical
  ordering, request/result cardinality, intent/query/assignment closure и все
  compare hashes. Replay повторно выполняет production mapper, targeting,
  physics-query и command paths и сравнивает exact V2 receipts, queries,
  results, commands, events и roots. Первый mismatch fail-closed с
  deterministic replay diagnostic.
- `ReplayManifestV4` и `InputMappingReceiptV1` сохраняются как legacy
  exact-read/replay contracts для существующих fixtures. Они не являются
  достаточным evidence для player-targeting/query provenance и не публикуются
  новым persistence path. Replay V3 остаётся `Unsupported`.
- Existing local V4 generations не переписываются in place. Новое evidence
  пересоздаётся как V5; migration route между V4 и V5 не объявляется.

Это решение не меняет `WorldCheckpointV4`, `SaveManifestV2`,
`PhysicsWorldCheckpointV1`, physics snapshot V2 или physics backend authority.

## Product impact

Movement и interaction могут находиться в одном player frame без потери
replay provenance. Target selection воспроизводится как факт authoritative
snapshot/query path, а не выводится из camera, depth buffer или stored expected
command. Camera остаётся presentation-only.

Вне scope остаются controller profile, Semantic UI, Linux native validation и
PhysX production promotion.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `persistence-replay` | Composite movement + pickup frame и authoritative `ClosestPoint` targeting | V5 round-trip и replay дают exact V2 command references, query facts, command/event/state/ledger roots | Любой receipt/query/hash mismatch отклоняется до ложного `PASS` |
| `play` | Live и headless producers используют один ActionMap/InputContext resolver | Одинаковые semantic frames проходят production ingress и дают одинаковый authoritative result | Camera/presentation fault не меняет gameplay roots |
| `host-check` | Contract/JCS/runtime tests | Unknown versions, malformed ranges и mismatched query traces fail closed | Legacy V4 остаётся только отдельным exact replay path |

## Рассмотренные варианты

- **Дополнить V4 и V1 новыми полями.** Отклонено: same-version canonical bytes
  изменились бы, нарушая exact-only compatibility.
- **Хранить только resulting commands и повторно вывести query facts.**
  Отклонено: stored command не доказывает snapshot/query provenance и может
  скрыть ошибку mapper или targeting selection.
- **Удалить V4 decoder сразу.** Отклонено: bounded legacy replay остаётся
  полезным для существующих fixtures и не ослабляет current V5 evidence, если
  новый persistence path никогда не публикует V4.

## Последствия

- Persistence/replay fixtures и JCS codec получают отдельный V5 envelope.
- Runtime tick report сохраняет legacy V1 receipt только для V4 compatibility
  и отдельный strict V2 receipt для current V5.
- SPEC-03, SPEC-18, SPEC-22, traceability и roadmap должны называть V5 current
  replay contract.
- Будущее несовместимое изменение V2 receipt или V5 replay требует новой
  schema version; optional fields не добавляются под прежним version tag.
