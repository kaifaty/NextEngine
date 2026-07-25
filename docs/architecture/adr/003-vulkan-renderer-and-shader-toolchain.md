# ADR-003: Vulkan renderer и compiler-neutral shader contract

| Поле | Значение |
|---|---|
| ID | ADR-003 |
| Статус | Accepted |
| Версия | 1.1 |
| Дата решения | 2026-07-22 |
| Последняя проверка | 2026-07-25 |
| Нормативные зависимости | [ADR-001](001-product-repository-license-and-platforms.md), [SPEC-04](../04-rendering-and-platform.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## ADR-030 scope

[ADR-030](030-product-first-development-and-lightweight-validation.md)
заменяет прежние process clauses. Vulkan baseline, engine-owned renderer
contracts и compiler-neutral shader boundary остаются техническим решением.

## Контекст

Windows и Linux имеют общий Vulkan baseline. Mesh shaders и ray tracing полезны для higher tiers, но не должны быть обязательны для базового play loop. Slang обещает единый shader toolchain, однако отдельные SPIR-V features имеют experimental maturity.

## Решение

- Vulkan 1.3 является Accepted graphics API v1.
- Engine MUST владеть `RenderDevice`, `RenderGraph`, resource descriptors, shader interface schema и capability query. `Vk*`, ash и compiler-specific types MUST NOT покидать renderer backend crates.
- Baseline tier MUST работать без ray tracing, ray query, mesh/task shaders и vendor extensions.
- GPU-driven submission, bindless resources и meshlets являются engine concepts; каждый MUST иметь indexed/indirect fallback в baseline tier.
- ash и Slang имеют статус `Proposed`. Offline compiler MUST выдавать SPIR-V, canonical reflection manifest и deterministic cache key. Runtime shader compilation в packaged game запрещена.
- При несовместимости Slang используется validated GLSL/HLSL → SPIR-V chain без изменения shader interface schema и render API.

## Рассмотренные варианты

- Direct3D 12 + Vulkan dual-backend v1 — `Rejected` из-за двух validation matrices.
- wgpu как primary backend v1 — `Rejected`: требуемый low-level Vulkan control и experimental high-end paths всё равно потребуют escape hatches.
- Обязательные mesh shaders — `Rejected` из-за platform/driver/maturity coverage.

## Последствия

Renderer MUST иметь capability tiers, pipeline cache invalidation и device-loss state machine. Assets MUST cook fallback geometry path. Shader source language не может быть видимым из RPG/runtime contracts.

## Product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| Vulkan baseline | Render representative scene without optional GPU features | Frame completes through `RenderDevice`; device loss is recoverable | Use indexed/indirect baseline paths |
| Binding candidate | Build and run the same scene through the selected Vulkan binding | No binding type escapes the adapter and output remains compatible | Use internal/generated bindings |
| Shader candidate | Offline compile and reflect representative shaders twice | SPIR-V, interface reflection and cache keys are stable | Use the validated GLSL/HLSL compiler chain |

## Supersession

Добавление другого public graphics API требует нового ADR. Замена
binding/compiler за сохранёнными contracts является private implementation
change с relevant renderer checks.
