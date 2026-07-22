# ADR-008: Общая mechanics/mod package model и agent-ready authoring

| Поле | Значение |
|---|---|
| ID | ADR-008 |
| Статус | Accepted |
| Версия | 1.0.1 |
| Владелец | Gameplay Extensibility Team |
| Дата решения | 2026-07-22 |
| Последняя проверка evidence | 2026-07-22 |
| Нормативные зависимости | [ADR-001](001-product-repository-license-and-platforms.md), [ADR-006](006-scripting-and-plugin-model.md) |
| Связанные документы | [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-11](../11-security-licensing-and-governance.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Контекст

Luau/Wasm sandbox решает безопасное исполнение, но сам по себе не делает добавление стрельбы, магии, статусов, crafting или community mods удобным. Без общей Ability/Effect/package model first-party mechanics получат private Rust APIs, моды будут зависеть от load order/monkey patches, а coding agents — от неструктурированного чтения всего repository.

## Решение

- Mechanics Runtime, immutable definitions, deterministic reducers, Effect pipeline, explicit hooks и engine-owned namespaced state являются Accepted public gameplay extension model.
- First-party и community mechanics MUST использовать одинаковые package contracts. Hidden first-party gameplay API запрещён.
- Data-only package является preferred path; Luau добавляет authoring logic, Wasm/WIT — переносимый complex algorithm. Native Rust не является mod ABI.
- Package resolution MUST использовать exact content-hashed `MechanicsLock`, dependency DAG и explicit preconditioned patches; implicit load-order override запрещён.
- Package code только предлагает `MechanicDeltaProposal`, EffectRequests и future commands. Engine выполняет common validation и atomic WorldCommand commit.
- Agent authoring MUST строиться на complete machine-readable context, structured diagnostics, scenarios/replay и reviewable `AgentChangeSet`; agent не получает privileged mutation path.
- CLI/JSON является normative authoring API. Local MCP adapter имеет статус `Proposed` projection с fallback на CLI/JSON и не входит в game runtime.

## Рассмотренные варианты

- Hardcoded отдельные combat/magic/crafting subsystems — `Rejected`: создают разные extension models и private first-party paths.
- Только event bus + arbitrary callbacks — `Rejected`: ordering, ownership, saves и conflicts становятся неявными.
- Direct mutable ECS/plugin components — `Rejected`: нарушает ownership, replay и sandbox invariants.
- Native dynamic libraries как community mod ABI — `Rejected`: platform/ABI/crash/security coupling; WIT/Wasm уже выбран ADR-006.
- Agent с generic shell/runtime debug mutation — `Rejected`: невозможно гарантировать scope, audit, rollback и player safety.
- MCP как единственный SDK — `Rejected`: protocol churn/agent client availability не должны блокировать human/CI workflow.

## Последствия

Нужно поддерживать Mechanics Runtime, package resolver/lockfile, schemas/migrations, standard primitives, fixtures и compatibility tooling как продукт. Core API будет меньше, но строже: новый primitive принимается только если несколько mechanics не могут выразиться composition/reducer model либо если profile доказывает performance need.

Пользователь сможет устанавливать локальные unsigned packages в untrusted ceiling; official packages подписываются, но проходят те же validators. Saves/replays становятся зависимы от exact lock и migrations, что повышает диагностируемость ценой явного uninstall workflow.

## Gate для Proposed частей

| Поле | Требование |
|---|---|
| Владелец | Developer Experience + Security Teams |
| Сценарий/команда | `next gate MCP-P1 --adapter mcp --protocol <pinned-stable> --compare cli-json --transport stdio` |
| Threshold | 100% exported resource/tool schemas и structured outputs эквивалентны CLI/JSON; read-only/mutating capabilities enforced; stale changesets rejected; default run opens 0 network sockets; no generic shell/filesystem/network tool |
| Evidence | protocol/version manifest, schema diff, tool audit, packet capture, fault report |
| Fallback | MCP adapter disabled; complete `next sdk`, `next mod`, `next mechanic` и `next changeset` CLI/JSON workflow |
| Срок повторной проверки | перед developer preview и при каждом MCP protocol revision |

## Supersession

Разрешение direct mutable plugin state, private first-party gameplay APIs, implicit load-order overrides или privileged agent mutation требует нового ADR. Добавление public core primitive за сохранённой package model является обычным versioned SDK change.
