# SPEC-04: Rendering и platform

| Поле | Значение |
|---|---|
| ID | SPEC-04 |
| Статус | Accepted |
| Версия | 2.0 |
| Последняя проверка | 2026-07-25 |
| Нормативные зависимости | [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-29](29-platform-host-and-application-session.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [ADR-003](adr/003-vulkan-renderer-and-shader-toolchain.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md) |
| Заменяет | отсутствует |

## Technical authority boundary

Authoritative visual inputs — immutable `PresentationSnapshotV2` и cooked
render assets по
[SPEC-30](30-presentation-extraction-and-render-content.md). Presentation
extraction publishes the canonical snapshot at the Runtime-declared boundary.
The renderer keeps bounded CPU `PresentationConsumptionStateV1`, private
platform/window/input adapters, graphics-device state, render graph, GPU/UI/VFX
caches, frame interpolation and capability selection. `ActionMapManifest`,
`InputContext`, `PlayerActionFrame` and semantic
UI/camera/localization/accessibility state follow
[SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md).
GPU resources, widget tree, camera output и current frame не являются gameplay
source of truth; после device loss они MUST быть восстанавливаемы из exact
content/profile/snapshot plus validated consumption-state inputs.

## PlatformHost boundary

`PlatformHost` предоставляет engine-owned `PlatformCapabilitySetV1`, `PlatformTimebaseV1`, `PlatformEventV1` и `NormalizedControlEventV1` по [SPEC-29](29-platform-host-and-application-session.md), window/presentation-target lifecycle, bounded monitor/DPI projection, clipboard opt-in и crash-safe user-data path capability. SDL 3 — `Proposed` adapter. SDL/Win32/XInput/X11/Wayland handles, native events и scancodes MUST NOT пересекать platform adapter; fallback — thin native adapters за тем же engine contract.

Input adapter нормализует physical device controls, но gameplay bindings, context stack, quantization и device-independent action IDs принадлежат SPEC-18 выше platform layer. Window close, focus loss, suspend и surface invalidation приходят typed events. Platform callback, UI callback или camera query MUST NOT менять world state напрямую.

Optional capture worker не создаёт interactive `PlatformHost`:
input/window/monitor/clipboard/surface lifecycle отсутствуют. Его
`ApplicationSessionManifestV1` допускает только `DisplaylessOffscreen`;
`OffscreenPresentationTarget` принадлежит render API и получает explicit
extent/format/color metadata из developer-selected capture request. `headless`
допускает только presentation target `None`.

Screenshots, video capture, GPU traces and visual comparison MAY be used as
developer diagnostics or playtesting tools. Their absence does not break the
product contract and they never become gameplay authority.

## Capability tiers

| Tier | Обязательные возможности | Поведение |
|---|---|---|
| B0 Core | Vulkan 1.3, dynamic rendering, synchronization2, timeline semaphores, buffer device address, indirect indexed draw; descriptor indexing только при доступности | MUST запускать representative gameplay scene без RT/mesh shaders; bounded descriptors и conventional vertex/index path разрешены |
| E1 Enhanced | B0 + descriptor indexing tier, draw-indirect-count, mesh shader при доказанном adapter support | GPU-driven compaction и meshlet task/mesh path MAY включаться |
| E2 Ray features | E1 или B0 + ray query/RT subset, достаточный конкретному effect | Optional shadows/reflections/queries; отсутствие не меняет gameplay |

Startup capability negotiation выбирает один declared path для каждого feature
и сохраняет его в session diagnostics. Неподдерживаемая обязательная B0
capability приводит к pre-world diagnostic `GPU_UNSUPPORTED`; optional tier
тихо не эмулируется CPU без явного budget policy.

## Engine-owned render API

Публичная backend boundary включает opaque handles, immutable buffer/image/sampler/pipeline descriptors, explicit usage/lifetime, `RenderGraph`, `ShaderInterface`, `CapabilitySet` и typed errors. `Vk*`, ash types, SPIR-V library objects и shader compiler types запрещены в contracts/RPG/tools schemas.

RenderGraph declares passes, read/write resources, queues и temporal dependencies. Compiler MUST detect read-before-write, cycles, invalid aliasing и missing transitions до submission. Transient resource lifetime выводится graph compiler; imported swapchain, offscreen capture и persistent resources имеют explicit owner. Final presentation graph MUST принимать engine-owned `PresentationTarget`; window swapchain и OffscreenPresentationTarget отличаются только final acquire/export/present adapter, не material/UI/camera passes.

## Submission и geometry

- Cooker MUST генерировать meshlets и conventional indexed geometry из одного canonical source.
- B0 MUST поддерживать CPU-built/compute-assisted visible list + indirect indexed draws.
- E1 MAY использовать task/mesh shaders; отказ compilation/runtime автоматически выбирает заранее cooked B0 pipeline до world start.
- Bindless logical material model использует stable engine indices. На устройстве без достаточного descriptor indexing backend MUST разбивать draws по bounded descriptor tables без изменения material contract.
- Occlusion/culling являются presentation optimization и MUST NOT влиять на simulation visibility/perception.

## Shader toolchain и caches

Shader source компилируется offline. Canonical `ShaderInterface` фиксирует
stages, entry points, resource bindings, push constants, specialization
parameters, vertex/mesh payloads и source map. Platform-neutral artifact key
включает canonical source/include hashes, normalized compiler contract/build
ID, target profile, defines, optimization/debug flags и interface schema
version. Native compiler executable hash MAY сохраняться в developer
diagnostics, but MUST NOT make an otherwise identical Win/Linux artifact key
platform-specific.

Slang и ash остаются `Proposed`. Slang mesh/task path не требуется B0. Fallback compiler chain MUST принять тот же ShaderInterface, выдать SPIR-V + reflection и пройти те же layout tests. Pipeline cache привязан к device/driver UUID, engine build и shader hashes; incompatible cache удаляется без потери gameplay data.

## Frame flow

1. Platform adapter publishes a bounded canonical `PlatformEventV1` batch containing `NormalizedControlEventV1`; SPEC-18 resolves it into immutable `PlayerActionFrame`, а только ADR-022 assigns current/next tick.
2. Renderer получает latest two atomically published `PresentationSnapshotV2` values и presentation-only interpolation alpha.
3. Streaming делает validated render resources resident, иначе declared placeholder.
4. RenderGraph instance выбирает paths по immutable CapabilitySet.
5. Backend records/submits and presents; GPU timestamps are collected only when
   optional profiling is enabled.
6. Presentation result/telemetry, widget state и interpolated camera targeting не модифицируют authoritative simulation; action-derived intent проходит common command validation.

For an optional developer capture, steps 1/5 replace the window pump/present
with validated explicit input and deterministic image readback. Frame
index/timestamp comes from the simulation/capture timeline; wall clock and GPU
completion order do not determine output order.

## Device-loss и failure semantics

При out-of-date interactive target пересоздаётся только presentation chain. При
device loss renderer прекращает submissions, сохраняет typed diagnostic,
invalidates the complete `PresentationCacheManifestV1` generation и atomically
rebuilds only from exact
content/interface/profile/`PresentationSnapshotV2` inputs plus the validated
bounded CPU `PresentationConsumptionStateV1`. Consumption state remains outside
cache invalidation, prevents acknowledged one-shot replay and preserves the
exact pending-one-shot and active continuous-instance maps.
Simulation/session/snapshot roots
остаются неизменны; policy MAY request typed `Suspended`, но device timing не
выбирает gameplay outcome. CPU headless не загружает renderer. Capture-worker
crash/device loss не повреждает source replay и не публикует partial artifacts.
Shader/interface/material/color mismatch является pre-use content failure, а
не best-effort draw.

## Platform packaging

Windows package MUST использовать pinned Vulkan loader strategy и перечислять
runtime dependencies; Linux package MUST объявлять minimum supported glibc or
container target и проверять Vulkan loader/ICD. User GPU driver не включается.
Debug layers/RenderDoc markers MAY быть optional package, но их отсутствие не
меняет cache keys release shaders.

## Product checks

| ID | Scenario | Expected behavior | Fallback |
|---|---|---|---|
| `RENDER-P1` | Run an engine-owned Vulkan B0 gameplay scene on available Win/Linux targets with the selected binding adapter. | No validation errors or leaked objects; the scene remains responsive at the declared product budget; no binding/vendor type escapes the renderer backend. | Fix or replace the adapter behind the same `RenderDevice`; reduce optional visual quality before changing gameplay. |
| `SHADER-P1` | Compile the offline `ShaderInterface` matrix with the selected compiler chain. | VS/FS/compute SPIR-V, canonical reflection and platform-neutral artifact keys match across Win/Linux; layouts match exactly; unavailable optional task/mesh or ray-query paths remain unloaded. | Replace the compiler adapter behind the same interface and reject incompatible shader assets. |
| `RENDER-ASH-P1` | Exercise the B0 scene through the proposed ash adapter. | The renderer behavior matches `RENDER-P1` and no ash type crosses the backend boundary. | Keep the internal/generated binding adapter. |
| `SHADER-SLANG-P1` | Exercise the shader matrix through the proposed Slang adapter. | The behavior matches `SHADER-P1` and source mapping exists for every compiled entry. | Keep the verified GLSL/HLSL-to-SPIR-V compiler adapter. |
| `PLATFORM-P1` | Repeated create/resize/fullscreen/focus/input/surface lifecycle on supported desktop hosts. | No crash or leak; normalized event ordering is stable and native handles remain private. | Use the thin native adapter behind the same platform contract. |
| `RENDER-02` | Force `no RT`, `no mesh shader` and bounded descriptors. | The representative scene remains complete and playable with no missing required material or geometry. Optional screenshots or image diffs may help diagnose regressions but are not the correctness oracle. | Disable the unsupported enhanced path and use the cooked B0 path. |
| `RENDER-03` | Inject swapchain and device loss at representative frame boundaries. | Interactive target recreation or clean suspension/exit completes without authoritative-state corruption; acknowledged presentation cues are not replayed. | Stop recovery attempts, preserve the last complete save/session state and exit cleanly. |
| `PACKAGE-01` | Install and run a clean Win/Linux package. | The package launches the B0 gameplay scene and reports missing runtime dependencies clearly. | Do not distribute the broken target package; repair its loader/dependency declaration. |
| `RENDER-04` | Optionally run a developer capture through the displayless offscreen target. | No window/display/surface/swapchain dependency is created; replay gameplay hash remains unchanged and repeated normalized frame output is stable for the selected profile. | Disable capture tooling and fix the target abstraction; normal game/headless operation remains available. |
