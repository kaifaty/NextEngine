# ADR-012: Deterministic command identity, ordering и replay

| Поле | Значение |
|---|---|
| ID | ADR-012 |
| Статус | Superseded |
| Версия | 1.0 |
| Дата решения | 2026-07-23 |
| Заменён | [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md) |

Это решение исторически связало command identity с canonical command bytes, передало ordering валидатору и сделало retry, sequence reuse и collision явными результатами admission. Replay и runtime-created durable IDs выводились из causal command history, чтобы arrival order и скрытая генерация идентификаторов не меняли результат.

Действующий контракт полностью определяет [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md): command body v2, stream binding, persisted finite receipt window, collision bookkeeping, deterministic clock/RNG/scheduling и causal identities. Этот файл сохраняет rationale предыдущей версии и не устанавливает текущих требований.
