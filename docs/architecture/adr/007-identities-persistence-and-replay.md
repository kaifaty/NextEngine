# ADR-007: Stable IDs, persistence и replay

| Поле | Значение |
|---|---|
| ID | ADR-007 |
| Статус | Superseded |
| Версия | 1.0.1 |
| Дата решения | 2026-07-22 |
| Заменён | [ADR-012](012-deterministic-command-identity-and-replay.md) |

Это решение исторически разделило ephemeral ECS handles, durable object IDs и asset references. Оно также закрепило fail-closed загрузку и миграцию, а replay трактовало как повтор causal inputs, а не применение производных событий как второго источника состояния.

ADR-007 был непосредственно заменён [ADR-012](012-deterministic-command-identity-and-replay.md), который затем полностью заменил [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md). Текущий контракт identity, persistence и replay определяет только ADR-022; этот файл остаётся историческим контекстом.
