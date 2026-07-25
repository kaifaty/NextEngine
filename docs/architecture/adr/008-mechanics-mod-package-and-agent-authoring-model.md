# ADR-008: Общая mechanics/mod package model и agent-ready authoring

| Поле | Значение |
|---|---|
| ID | ADR-008 |
| Статус | Accepted |
| Версия | 1.0.1 |
| Дата решения | 2026-07-22 |
| Последняя проверка | 2026-07-25 |
| Нормативные зависимости | [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [ADR-014](014-deterministic-extensions-and-package-trust.md), [SPEC-11](../11-security-licensing-and-governance.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## ADR-030 scope

[ADR-030](030-product-first-development-and-lightweight-validation.md)
заменяет прежние process clauses. Единая first-party/community mechanics
package model, engine-owned schemas и validated extension paths остаются
техническим решением.

## Контекст

Luau/Wasm sandbox решает безопасное исполнение, но сам по себе не делает добавление стрельбы, магии, статусов, crafting или community mods удобным. Без общей Ability/Effect/package model first-party mechanics получат private Rust APIs, моды будут зависеть от load order/monkey patches, а coding agents — от неструктурированного чтения всего repository.

## Решение

- Mechanics Runtime, immutable definitions, deterministic reducers, Effect pipeline, explicit hooks и engine-owned namespaced state являются Accepted public gameplay extension model.
- First-party и community mechanics MUST использовать одинаковые package contracts. Hidden first-party gameplay API запрещён.
- Data-only package является preferred path; Luau добавляет authoring logic, Wasm/WIT — переносимый complex algorithm. Native Rust не является mod ABI.
- Package resolution MUST использовать exact content-hashed `MechanicsLock`, dependency DAG и explicit preconditioned patches; implicit load-order override запрещён.
- Package code только предлагает `MechanicDeltaProposal`, EffectRequests и future commands. Engine выполняет common validation и atomic WorldCommand commit.
- Agent authoring MUST строиться на complete machine-readable context, structured diagnostics, scenarios/replay и inspectable `AgentChangeSet`; agent не получает privileged mutation path.
- CLI/JSON является normative authoring API. Local MCP adapter имеет статус `Proposed` projection с fallback на CLI/JSON и не входит в game runtime.

## Рассмотренные варианты

- Hardcoded отдельные combat/magic/crafting subsystems — `Rejected`: создают разные extension models и private first-party paths.
- Только event bus + arbitrary callbacks — `Rejected`: ordering, ownership, saves и conflicts становятся неявными.
- Direct mutable ECS/plugin components — `Rejected`: нарушает ownership, replay и sandbox invariants.
- Native dynamic libraries как community mod ABI — `Rejected`: platform/ABI/crash coupling; WIT/Wasm isolation boundary уже выбрана ADR-014.
- Agent с generic shell/runtime debug mutation — `Rejected`: невозможно гарантировать scope, atomicity, rollback и player safety.
- MCP как единственный SDK — `Rejected`: protocol churn/agent client availability не должны блокировать human/CI workflow.

## Последствия

Нужно поддерживать Mechanics Runtime, package resolver/lockfile, schemas/migrations, standard primitives, fixtures и compatibility tooling как продукт. Core API будет меньше, но строже: новый primitive принимается только если несколько mechanics не могут выразиться composition/reducer model либо если profile доказывает performance need.

Пользователь сможет устанавливать локальные content-hashed packages под
capability ceiling. Engine не определяет PKI или signer lifecycle; distribution
wrapper при необходимости решает это вне runtime contracts. Все packages
проходят те же validators. Saves/replays зависят от exact lock и migrations,
что повышает диагностируемость ценой явного uninstall workflow.

## Product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| Package path | Install first-party and community variants through the same API | Exact lock, validation and replay results match the declared package semantics | Reject the package and retain the prior lock |
| Optional MCP projection | Compare representative read/write operations with CLI/JSON | Structured results and capability denials are equivalent; no implicit network access | Disable MCP and keep the complete CLI/JSON workflow |

## Supersession

Разрешение direct mutable plugin state, private first-party gameplay APIs, implicit load-order overrides или privileged agent mutation требует нового ADR. Добавление public core primitive за сохранённой package model является обычным versioned SDK change.
