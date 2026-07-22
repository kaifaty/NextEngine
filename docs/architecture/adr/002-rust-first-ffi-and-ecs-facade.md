# ADR-002: Rust-first core, FFI и ECS facade

| Поле | Значение |
|---|---|
| ID | ADR-002 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | Core Architecture |
| Дата решения | 2026-07-22 |
| Последняя проверка evidence | 2026-07-22 |
| Нормативные зависимости | [ADR-001](001-product-repository-license-and-platforms.md), [SPEC-02](../02-runtime-ecs-and-data.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Контекст

Runtime нуждается в memory safety, предсказуемом native performance и контролируемом FFI к physics, graphics и ML libraries. ECS library churn не должен определять saves, scripting, plugins или публичный SDK.

## Решение

- Core runtime, RPG framework, cooker и first-party tools MUST быть Rust-first на pinned stable toolchain.
- C/C++ backends MUST входить только через narrow engine-owned FFI crates. Safe Rust wrapper MUST владеть lifetime, threading, error translation и cleanup.
- ECS и scheduler MUST быть скрыты за engine-owned facade. `RuntimeEntityId` является opaque engine type; Bevy entity/component/resource/event types MUST NOT пересекать public crate, save, WIT, Luau или importer boundaries.
- Bevy ECS/app crates имеют статус `Proposed` до прохождения ECS-P1. Renderer, UI и asset system Bevy не принимаются этим ADR.
- Public cross-language ABI MUST использовать C-compatible handles/byte buffers либо WIT components; Rust ABI не является стабильным контрактом.

## Рассмотренные варианты

- C++-first core — `Rejected`: не даёт выбранной baseline memory-safety модели.
- Полный Bevy application stack — `Rejected`: создаёт ненужное связывание product architecture с быстро меняющимся framework API.
- Собственный ECS с первого дня — `Rejected` как default bootstrap; engine-owned scheduler остаётся named fallback после PoC.

## Последствия

Workspace MUST включать dependency boundary checks и compile-fail tests. Unsafe blocks MUST быть локализованы в backend/FFI crates, документировать invariants и проходить sanitizer/miri там, где применимо.

## Gate для Proposed частей

| Поле | Требование |
|---|---|
| Владелец | Runtime Team |
| Сценарий/команда | `cargo xtask gate ecs-poc --profile release --seed 41041` на Windows/Linux |
| Threshold | 10 000 entities, 200 representative systems; 10 000 fixed ticks дают одинаковый state hash на повторе одной platform/архитектуры; ≥1.5× speedup параллельного schedule над принудительно serial на 8 logical cores; headless snapshot round-trip 100%; ни одного Bevy type в output `cargo public-api` для public crates |
| Evidence | `run-manifest.json`, state hashes, benchmark JSON, public API report, CI logs |
| Fallback | Engine-owned scheduler за той же facade; внутренняя data structure выбирается отдельным implementation ADR |
| Срок повторной проверки | перед bootstrap milestone M1 и при каждом major Bevy upgrade |

## Supersession

Замена языка core или раскрытие ECS API требует нового ADR. Замена внутренней ECS реализации за существующей facade не требует изменения product contracts, но требует ADR реализации.
