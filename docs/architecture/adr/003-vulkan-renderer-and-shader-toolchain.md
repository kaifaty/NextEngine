# ADR-003: Vulkan renderer и compiler-neutral shader contract

| Поле | Значение |
|---|---|
| ID | ADR-003 |
| Статус | Accepted |
| Версия | 1.1 |
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

## Gates для Proposed частей

Accepted baseline gates `RENDER-P1` и `SHADER-P1` принадлежат engine-owned `RenderDevice`/`ShaderInterface` contracts SPEC-04 и не выбирают binding или compiler. Proposed adapters имеют отдельные `CandidateOnly` IDs, чьи единственные canonical descriptors находятся в verification-gate table SPEC-04:

| Candidate reference | Canonical descriptor source | Decision / fallback |
|---|---|---|
| RENDER-ASH-P1 | SPEC-04 | ash остаётся `Proposed`; при candidate FAIL не выбирать ash и использовать internal/generated Vulkan bindings за тем же `RenderDevice`. |
| SHADER-SLANG-P1 | SPEC-04 | Slang остаётся `Proposed`; при candidate FAIL не выбирать Slang и использовать verified GLSL/HLSL → SPIR-V chain за тем же `ShaderInterface`. |

Оба candidate gate повторно выполняются перед M2 renderer bootstrap и при любом upgrade exact adapter. Их PASS не закрывает `VS-07` без независимых baseline `RENDER-P1` и `SHADER-P1`.

## Supersession

Добавление другого public graphics API требует нового ADR. Замена binding/compiler за сохранёнными contracts следует gate и оформляется implementation ADR.
