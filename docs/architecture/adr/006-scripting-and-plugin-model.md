# ADR-006: Luau gameplay и Wasm Component Model plugins

| Поле | Значение |
|---|---|
| ID | ADR-006 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | RPG Framework Team |
| Дата решения | 2026-07-22 |
| Последняя проверка evidence | 2026-07-22 |
| Нормативные зависимости | [SPEC-07](../07-rpg-scripting-and-plugins.md), [SPEC-11](../11-security-licensing-and-governance.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Контекст

Content authors нуждаются в компактном gameplay scripting, а third-party extensions — в versioned language-neutral boundary и сильной изоляции. Один механизм плохо покрывает обе модели риска.

## Решение

- Luau является Accepted scripting technology для first-party gameplay/content logic. Exact embedded version pinned implementation manifest и повторяет SCRIPT gates при upgrade; полный host API до PoC не замораживается.
- Wasm Component Model + versioned WIT worlds являются Accepted plugin ABI model. Wasmtime является Proposed runtime backend/version.
- Script и plugin получают capability-scoped context, immutable views и command sink. Прямой mutable ECS/world/physics доступ запрещён.
- Luau MUST иметь per-callback instruction/time/allocation budgets, deterministic APIs и isolated environments.
- Plugins MUST иметь signed/hashed manifest, declared WIT range, capabilities, fuel, memory/table/instance limits и compatibility negotiation. Default deny.
- Budget overrun, trap или denied capability MUST остановить только нарушителя, отменить его uncommitted commands и записать structured diagnostic.

## Рассмотренные варианты

- Только Wasm для content scripts отклонён из-за authoring friction.
- Native dynamic libraries отклонены для untrusted plugins из-за отсутствия isolation/stable ABI.
- Общий mutable ECS API отклонён из-за save/replay/security invariants.

## Последствия

Full SDK не замораживается до PoC, но capability names, command validation и version negotiation являются baseline. Hot reload MAY существовать только в tools/development и MUST инвалидировать replay compatibility metadata.

## Gate для Proposed частей

| Поле | Требование |
|---|---|
| Владелец | RPG Framework + Security Teams |
| Сценарий/команда | `cargo xtask gate extensions --suite v1 --runtime wasmtime --script luau` |
| Threshold | 100% forbidden calls denied; 10 000 repeated traps/overruns без runtime crash/leak; script callback принудительно остановлен ≤2 ms после budget threshold; plugin memory cap превышен без роста host RSS >16 MiB после cleanup; WIT N/N-1 negotiation совпадает с compatibility matrix; same seed + same accepted commands дают одинаковый state hash |
| Evidence | run manifest, denial audit log, memory/fuel metrics, compatibility report, fuzz corpus summary |
| Fallback | Pin предыдущую совместимую Luau/Wasmtime version; отключить plugin loading при несовместимости; gameplay продолжает работать на compiled Rust rules и Luau baseline |
| Срок повторной проверки | перед M4 RPG slice и при VM/runtime upgrade |

## Supersession

Замена ролей Luau/Wasm или выдача direct mutable access требует нового ADR. Version upgrade за тем же contract проходит compatibility gate.
