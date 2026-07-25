# ADR-002: Rust-first core, FFI и ECS facade

| Поле | Значение |
|---|---|
| ID | ADR-002 |
| Статус | Accepted |
| Версия | 1.0.1 |
| Дата решения | 2026-07-22 |
| Последняя проверка | 2026-07-25 |
| Нормативные зависимости | [ADR-001](001-product-repository-license-and-platforms.md), [SPEC-02](../02-runtime-ecs-and-data.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## ADR-030 scope

[ADR-030](030-product-first-development-and-lightweight-validation.md)
заменяет прежние process clauses. Rust-first core, narrow FFI boundaries и
engine-owned ECS facade остаются техническим решением.

## Контекст

Runtime нуждается в memory safety, предсказуемом native performance и контролируемом FFI к physics, graphics и ML libraries. ECS library churn не должен определять saves, scripting, plugins или публичный SDK.

## Решение

- Core runtime, RPG framework, cooker и first-party tools MUST быть Rust-first на pinned stable toolchain.
- C/C++ backends MUST входить только через narrow engine-owned FFI crates. Safe Rust wrapper MUST владеть lifetime, threading, error translation и cleanup.
- ECS и scheduler MUST быть скрыты за engine-owned facade. `RuntimeEntityId` является opaque engine type; Bevy entity/component/resource/event types MUST NOT пересекать public crate, save, WIT, Luau или importer boundaries.
- Bevy ECS/app crates имеют статус `Proposed` до focused ECS evaluation. Renderer, UI и asset system Bevy не принимаются этим ADR.
- Public cross-language ABI MUST использовать C-compatible handles/byte buffers либо WIT components; Rust ABI не является стабильным контрактом.

## Рассмотренные варианты

- C++-first core — `Rejected`: не даёт выбранной baseline memory-safety модели.
- Полный Bevy application stack — `Rejected`: создаёт ненужное связывание product architecture с быстро меняющимся framework API.
- Собственный ECS с первого дня — `Rejected` как default bootstrap; engine-owned scheduler остаётся named fallback после PoC.

## Последствия

Workspace MUST включать dependency boundary checks и compile-fail tests. Любое
будущее исключение из workspace `unsafe_code` policy требует отдельного
engine-owned FFI boundary ADR и explicit allowlist; initial allowlist пуст.

## Product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| Portable boundary | Build public crates and scan exported API | No ECS/vendor/FFI type crosses public contracts | Keep backend private or reject the adapter |
| ECS candidate | Run representative fixed-tick workload and snapshot round-trip | Repeat state hashes match and candidate is useful for the workload | Use the engine-owned serial scheduler behind the same facade |

## Supersession

Замена языка core или раскрытие ECS API требует нового ADR. Замена внутренней ECS реализации за существующей facade не требует изменения product contracts, но требует ADR реализации.
