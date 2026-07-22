# SPEC-04: Rendering и platform

| Поле | Значение |
|---|---|
| ID | SPEC-04 |
| Статус | Accepted |
| Версия | 1.2 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-23 |
| Нормативные зависимости | [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-003](adr/003-vulkan-renderer-and-shader-toolchain.md), [ADR-015](adr/015-evidence-trust-fixture-separation-and-attestation.md) |
| Заменяет | отсутствует |

## Source of truth и ownership

Authoritative visual inputs — immutable PresentationSnapshot и cooked render assets. Rendering Team владеет platform window/input adapter, Vulkan device state, render graph, GPU caches, frame interpolation и capability selection. GPU resources и current frame не являются gameplay source of truth; после device loss они MUST быть восстанавливаемы из assets/snapshot.

## PlatformHost boundary

`PlatformHost` предоставляет engine-owned window/surface lifecycle, raw timestamped input, monitor/DPI state, clipboard opt-in, high-resolution clock и crash-safe user data paths. SDL 3 — `Proposed` adapter. SDL handles/events MUST NOT пересекать platform crate; fallback — thin Win32 и X11/Wayland adapters за тем же contract.

Input adapter нормализует physical device controls, но gameplay bindings находятся выше platform layer. Window close, focus loss, suspend и surface invalidation приходят typed events. Platform callback MUST NOT менять world state напрямую.

Capture worker не создаёт `PlatformHost`: input/window/monitor/clipboard/surface lifecycle отсутствуют. `OffscreenPresentationTarget` принадлежит render API и получает explicit extent/format/color metadata из CaptureJobManifest.

Любое material изменение renderer, VFX, UI или camera output MUST объявлять соответствующую observable category и получать `HumanReviewRequired` через ImpactResolver. Rendering package не может снять этот флаг собственным test manifest; semantic/render automatic gates проходят до qualitative review по SPEC-15.

## Capability tiers

| Tier | Обязательные возможности | Поведение |
|---|---|---|
| B0 Baseline | Vulkan 1.3, dynamic rendering, synchronization2, timeline semaphores, buffer device address, indirect indexed draw; descriptor indexing только при доступности | MUST запускать vertical slice без RT/mesh shaders; bounded descriptors и conventional vertex/index path разрешены |
| E1 Enhanced | B0 + descriptor indexing tier, draw-indirect-count, mesh shader при доказанном adapter support | GPU-driven compaction и meshlet task/mesh path MAY включаться |
| E2 Ray features | E1 или B0 + ray query/RT subset, достаточный конкретному effect | Optional shadows/reflections/queries; отсутствие не меняет gameplay |

Startup capability negotiation выбирает один declared path для каждого feature и записывает его в RunManifest. Неподдерживаемая обязательная B0 capability приводит к pre-world diagnostic `GPU_UNSUPPORTED`; optional tier тихо не эмулируется CPU без явного budget policy.

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

Shader source компилируется offline. Canonical `ShaderInterface` фиксирует stages, entry points, resource bindings, push constants, specialization parameters, vertex/mesh payloads и source map. Cache key включает canonical source/include hashes, compiler binary hash/version, target profile, defines, optimization/debug flags и interface schema version.

Slang и ash остаются `Proposed`. Slang mesh/task path не требуется B0. Fallback compiler chain MUST принять тот же ShaderInterface, выдать SPIR-V + reflection и пройти те же layout tests. Pipeline cache привязан к device/driver UUID, engine build и shader hashes; incompatible cache удаляется без потери gameplay data.

## Frame flow

1. PlatformHost pumps timestamped events в input buffer.
2. Renderer получает latest two PresentationSnapshot и interpolation alpha.
3. Streaming делает validated render resources resident, иначе declared placeholder.
4. RenderGraph instance выбирает paths по immutable CapabilitySet.
5. Backend records/submits, собирает GPU timestamps и presents.
6. Presentation result/telemetry не модифицируют authoritative simulation.

Для capture flow steps 1/5 window pump/present заменяются validated CaptureJob input и deterministic image readback. Frame index/timestamp выводится из simulation/capture timeline; wall clock и GPU completion order не определяют artifact sequence.

## Device-loss и failure semantics

При out-of-date swapchain пересоздаётся только surface chain. При device loss renderer прекращает submissions, сохраняет typed diagnostic, уничтожает backend state и делает максимум одну controlled reinitialization на том же или lower optional tier. Simulation может быть bounded-paused до 10 s в interactive game; затем user получает save-and-exit path, если core/physics здоровы. CPU headless не загружает renderer. Capture worker crash/device loss не повреждает source replay и не публикует partial artifacts. Shader/interface mismatch и validation error в release build являются pre-use asset failure, а не best-effort draw.

## Platform packaging

Windows package MUST использовать pinned Vulkan loader strategy и перечислять runtime dependencies; Linux package MUST объявлять supported glibc baseline/container target и проверять Vulkan loader/ICD. User GPU driver не включается. Debug layers/RenderDoc markers MAY быть optional package, но их отсутствие не меняет cache keys release shaders.

## Verification gates

| Gate | Сценарий | Threshold | Evidence | Fallback |
|---|---|---|---|---|
| RENDER-P1 | ash Vulkan B0 scene на Win/Linux | 0 validation errors, 0 leaked objects; ≥60 FPS/1080p и p95 GPU ≤16.6 ms на reference Tier-B GPU | RunManifest, validation log, GPU trace, screenshots | internal/generated bindings; optimize B0 path |
| SHADER-P1 | pinned Slang offline matrix | VS/FS/compute byte-identical SPIR-V/reflection across Win/Linux; layout 100%; task/mesh + ray-query correctly pass or capability-skip; deterministic cache keys; RenderDoc source mapping present | hashes, reflection diff, captures | verified GLSL/HLSL→SPIR-V chain |
| PLATFORM-P1 | SDL adapter lifecycle/input/focus/surface tests | 10 000 create/resize/fullscreen/focus cycles, 0 crash/leak; input timestamp ordering exact; same normalized event fixtures Win/Linux | event trace, memory report | native adapters |
| RENDER-02 | forced `no RT`, `no mesh shader`, bounded descriptors | vertical scene image-diff SSIM ≥0.98 vs approved B0 reference, no missing materials/geometry | capture + diff report | release blocking fix |
| RENDER-03 | injected swapchain/device loss at 100 frame points | swapchain recovery 100%; device recovery or clean save-and-exit ≤10 s; 0 authoritative state corruption | fault report + replay hash | disable recovery attempt, clean exit |
| PACKAGE-01 | clean Win/Linux VM install/run | package launches B0 scene, no undeclared shared library, SBOM complete | VM logs, SBOM, package hashes | block platform package |
| RENDER-04 | windowless offscreen presentation target | CAPTURE-01 passes; 0 PlatformHost/window/surface/swapchain dependency in worker graph; same replay gameplay hash exact; canonical frame root repeated exact on pinned worker | render graph/API scan, replay/frame hashes, window/socket trace | fix target abstraction; observable review AwaitingCapability |
