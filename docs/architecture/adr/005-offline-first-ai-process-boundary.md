# ADR-005: Offline-first AI и process boundary

| Поле | Значение |
|---|---|
| ID | ADR-005 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | Agent Intelligence Team |
| Дата решения | 2026-07-22 |
| Последняя проверка evidence | 2026-07-22 |
| Нормативные зависимости | [SPEC-01](../01-system-architecture.md), [SPEC-06](../06-ai-agents-perception-and-memory.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Контекст

Generative services могут отсутствовать, зависнуть, упасть или вернуть недопустимый результат. Игра не может отдавать им ownership simulation, rules или saves.

## Решение

- LLM, embeddings, ASR и TTS MUST исполняться в отдельном optional `ai-host` process. Network access default-deny и не является условием игры.
- Game/headless runtime MUST сохранять deterministic utility/HTN/tactical fallback, generic dialogue fallback и complete gameplay loop без `ai-host`.
- `ai-host` выдаёт только versioned `AgentIntent`, speech/media candidates и memory proposals. Все state changes проходят engine validation → `WorldCommand`.
- Small motor policies MAY работать in-process через pinned model runtime; они не относятся к `ai-host`, не используют LLM и подчиняются ADR-004.
- Timeout, malformed response, incompatible protocol, model rejection и process crash MUST деградировать качество, но не корректность gameplay. Restart MUST выполнять handshake и не повторять уже committed command.

## Рассмотренные варианты

- In-process LLM отклонён из-за crash containment, memory pressure и nondeterministic dependencies.
- Cloud-required AI отклонён из-за offline-first product contract.
- Прямые tool calls LLM в mutable world отклонены из-за безопасности и replay.

## Последствия

IPC schema, deadlines, idempotency keys, provenance и telemetry становятся обязательными. Saves хранят authoritative memory records, а не vendor session handles. AI output никогда не входит в fixed physics tick как blocking dependency.

## Gate для Proposed частей

Не применяется к process boundary. Storage/model/runtime candidates имеют собственные gates в SPEC-06 и SPEC-05.

## Supersession

Разрешение обязательного network AI, in-process generative model или прямой mutation требует нового ADR.
