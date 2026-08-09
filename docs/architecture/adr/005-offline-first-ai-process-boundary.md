# ADR-005: Offline-first AI и process boundary

| Поле | Значение |
|---|---|
| ID | ADR-005 |
| Статус | Accepted |
| Версия | 1.0.2 |
| Дата решения | 2026-07-22 |
| Последняя проверка | 2026-08-09 |
| Нормативные зависимости | [SPEC-01](../01-system-architecture.md), [SPEC-06](../06-ai-agents-perception-and-memory.md) |
| Заменяет | отсутствует |
| Заменён | planner-baseline wording частично заменён [ADR-056](056-deterministic-strategic-agent-and-belief-driven-goap.md); остальные решения сохраняются |

## ADR-030 scope

[ADR-030](030-product-first-development-and-lightweight-validation.md)
заменяет прежние process clauses. Optional isolated `ai-host`, untrusted
proposals, offline correctness и deterministic in-process fallbacks остаются
техническим решением.

## Контекст

Generative services могут отсутствовать, зависнуть, упасть или вернуть недопустимый результат. Игра не может отдавать им ownership simulation, rules или saves.

## Решение

- LLM, embeddings, ASR и TTS MUST исполняться в отдельном optional `ai-host` process. Network access default-deny и не является условием игры.
- Game/headless runtime MUST сохранять deterministic Utility + bounded GOAP/tactical fallback по ADR-056, generic dialogue fallback и complete gameplay loop без `ai-host`.
- `ai-host` выдаёт только versioned `AgentIntent`, speech/media candidates и memory proposals. Все state changes проходят engine validation → `WorldCommand`.
- Small motor policies MAY работать in-process через pinned model runtime; они не относятся к `ai-host`, не используют LLM и подчиняются ADR-013.
- Timeout, malformed response, incompatible protocol, model rejection и process crash MUST деградировать качество, но не корректность gameplay. Restart MUST выполнять handshake и не повторять уже committed command.

## Рассмотренные варианты

- In-process LLM отклонён из-за crash containment, memory pressure и nondeterministic dependencies.
- Cloud-required AI отклонён из-за offline-first product contract.
- Прямые tool calls LLM в mutable world отклонены из-за безопасности и replay.

## Последствия

IPC schema, deadlines, idempotency keys, provenance и telemetry становятся обязательными. Saves хранят authoritative memory records, а не vendor session handles. AI output никогда не входит в fixed physics tick как blocking dependency.

## Product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| Offline loop | Start and play representative scenario without `ai-host` or network | Mandatory gameplay completes and ticks never wait for the service | Use deterministic in-process planners/dialogue |
| Fault isolation | Inject timeout, crash, malformed response and protocol mismatch | No partial mutation or duplicate committed command | Reject the proposal and restart with handshake |

## Supersession

Разрешение обязательного network AI, in-process generative model или прямой mutation требует нового ADR.
