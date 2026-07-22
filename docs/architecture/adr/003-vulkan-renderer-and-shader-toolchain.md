# ADR-003: Vulkan renderer и compiler-neutral shader contract

| Поле | Значение |
|---|---|
| ID | ADR-003 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | Rendering Team |
| Дата решения | 2026-07-22 |
| Последняя проверка evidence | 2026-07-22 |
| Нормативные зависимости | [ADR-001](001-product-repository-license-and-platforms.md), [SPEC-04](../04-rendering-and-platform.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Контекст

Windows и Linux имеют общий Vulkan baseline. Mesh shaders и ray tracing полезны для higher tiers, но не должны быть обязательны для vertical slice. Slang обещает единый shader toolchain, однако отдельные SPIR-V features имеют experimental maturity.

## Решение

- Vulkan 1.3 является Accepted graphics API v1.
- Engine MUST владеть `RenderDevice`, `RenderGraph`, resource descriptors, shader interface schema и capability query. `Vk*`, ash и compiler-specific types MUST NOT покидать renderer backend crates.
- Baseline tier MUST работать без ray tracing, ray query, mesh/task shaders и vendor extensions.
- GPU-driven submission, bindless resources и meshlets являются engine concepts; каждый MUST иметь indexed/indirect fallback в baseline tier.
- ash и Slang имеют статус `Proposed`. Offline compiler MUST выдавать SPIR-V, canonical reflection manifest и deterministic cache key. Runtime shader compilation в packaged game запрещена.
- При провале Slang используется verified GLSL/HLSL → SPIR-V chain без изменения shader interface schema и render API.

## Рассмотренные варианты

- Direct3D 12 + Vulkan dual-backend v1 — `Rejected` из-за двух validation matrices.
- wgpu как primary backend v1 — `Rejected`: требуемый low-level Vulkan control и experimental high-end paths всё равно потребуют escape hatches.
- Обязательные mesh shaders — `Rejected` из-за platform/driver/maturity coverage.

## Последствия

Renderer MUST иметь capability tiers, pipeline cache invalidation и device-loss state machine. Assets MUST cook fallback geometry path. Shader source language не может быть видимым из RPG/runtime contracts.

## Gate для Proposed частей

| Поле | Требование |
|---|---|
| Владелец | Rendering Team |
| Сценарий/команда | `cargo xtask gate renderer-poc --matrix win-vulkan,linux-vulkan --capture` |
| Threshold | ash path проходит validation layers без error; Slang offline одинаковой pinned версии даёт byte-identical SPIR-V/reflection/cache keys на Windows и Linux для VS/FS/compute; task/mesh и ray-query samples либо проходят, либо capability-gated и не загружаются; 100% resource bindings совпадают с reflection; RenderDoc capture отображает source mapping; baseline scene ≥60 FPS при 1080p на reference Tier-B GPU и запускается при отключённых RT/mesh features |
| Evidence | compiler manifest, SPIR-V hashes, validation logs, RenderDoc capture, frame-time JSON, screenshots |
| Fallback | Internal/generated Vulkan bindings вместо ash; GLSL/HLSL → SPIR-V verified compiler chain вместо Slang |
| Срок повторной проверки | перед M2 renderer bootstrap и при upgrade ash/Slang |

## Supersession

Добавление другого public graphics API требует нового ADR. Замена binding/compiler за сохранёнными contracts следует gate и оформляется implementation ADR.
