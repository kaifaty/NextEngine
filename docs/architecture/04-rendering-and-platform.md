# SPEC-04: Rendering и platform

| Поле | Значение |
|---|---|
| ID | SPEC-04 |
| Статус | Accepted |
| Версия | 1.8 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-003](adr/003-vulkan-renderer-and-shader-toolchain.md), [ADR-023](adr/023-human-review-decision-v2-and-offline-attestation.md) |
| Заменяет | отсутствует |

## Source of truth и ownership

Authoritative visual inputs — immutable `PresentationSnapshotV2` и cooked
render assets по
[SPEC-30](30-presentation-extraction-and-render-content.md). Rendering Team
владеет canonical snapshot staging/publication at the Runtime-declared
boundary, bounded CPU `PresentationConsumptionStateV1`, private
platform/window/input adapter, graphics-device state, render graph, GPU/UI/VFX
caches, frame interpolation и capability selection. Player Experience владеет
`ActionMapManifest`, `InputContext`, `PlayerActionFrame`, semantic
UI/camera/localization/accessibility state по
[SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md).
GPU resources, widget tree, camera output и current frame не являются gameplay
source of truth; после device loss они MUST быть восстанавливаемы из exact
content/profile/snapshot plus validated consumption-state inputs.

## PlatformHost boundary

`PlatformHost` предоставляет engine-owned `PlatformCapabilitySetV1`, `PlatformTimebaseV1`, `PlatformEventV1` и `NormalizedControlEventV1` по [SPEC-29](29-platform-host-and-application-session.md), window/presentation-target lifecycle, bounded monitor/DPI projection, clipboard opt-in и crash-safe user-data path capability. SDL 3 — `Proposed` adapter. SDL/Win32/XInput/X11/Wayland handles, native events и scancodes MUST NOT пересекать platform adapter; fallback — thin native adapters за тем же engine contract.

Input adapter нормализует physical device controls, но gameplay bindings, context stack, quantization и device-independent action IDs принадлежат SPEC-18 выше platform layer. Window close, focus loss, suspend и surface invalidation приходят typed events. Platform callback, UI callback или camera query MUST NOT менять world state напрямую.

Capture worker не создаёт interactive `PlatformHost`: input/window/monitor/clipboard/surface lifecycle отсутствуют. Его `ApplicationSessionManifestV1` допускает только `DisplaylessOffscreen`; `OffscreenPresentationTarget` принадлежит render API и получает explicit extent/format/color metadata из `CaptureJobManifest`. `headless` допускает только presentation target `None`.

Любое material изменение renderer, VFX, UI или camera output MUST объявлять соответствующую observable category и получать `HumanReviewRequired` через ImpactResolver. Rendering package не может снять этот флаг собственным test manifest; semantic/render automatic gates проходят до qualitative review по SPEC-15/ADR-023, а admission возможен только по verified `HumanReviewDecisionV2::Approve` + `AttestationEnvelopeV2` при всех automatic gates `PASS`. Signed `Reject`/`NeedsChanges` остаются non-admitting feedback.

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

1. Platform adapter publishes a bounded canonical `PlatformEventV1` batch containing `NormalizedControlEventV1`; SPEC-18 resolves it into immutable `PlayerActionFrame`, а только ADR-022 assigns current/next tick.
2. Renderer получает latest two atomically published `PresentationSnapshotV2` values и presentation-only interpolation alpha.
3. Streaming делает validated render resources resident, иначе declared placeholder.
4. RenderGraph instance выбирает paths по immutable CapabilitySet.
5. Backend records/submits, собирает GPU timestamps и presents.
6. Presentation result/telemetry, widget state и interpolated camera targeting не модифицируют authoritative simulation; action-derived intent проходит common command validation.

Для capture flow steps 1/5 window pump/present заменяются validated CaptureJob input и deterministic image readback. Frame index/timestamp выводится из simulation/capture timeline; wall clock и GPU completion order не определяют artifact sequence.

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

Windows package MUST использовать pinned Vulkan loader strategy и перечислять runtime dependencies; Linux package MUST объявлять supported glibc baseline/container target и проверять Vulkan loader/ICD. User GPU driver не включается. Debug layers/RenderDoc markers MAY быть optional package, но их отсутствие не меняет cache keys release shaders.

## Verification gates

`RENDER-P1` и `SHADER-P1` проверяют Accepted implementation-neutral Vulkan/`ShaderInterface` baseline независимо от выбранных binding/compiler adapters. `RENDER-ASH-P1`, `SHADER-SLANG-P1` и `PLATFORM-P1` являются `CandidateOnly`: их PASS допускает выбор exact Proposed adapter, но не заменяет baseline gate и не входит в обязательный vertical aggregate.

| Gate | Сценарий | Threshold | Evidence | Fallback |
|---|---|---|---|---|
| RENDER-P1 | engine-owned `RenderDevice` Vulkan B0 scene на Win/Linux with the composition-locked binding adapter | 0 validation errors, 0 leaked objects; ≥60 FPS/1080p and p95 GPU ≤16.6 ms on reference Tier-B GPU; public/API scan finds 0 binding/vendor type outside backend | RunManifest, validation log, GPU trace, public API scan and screenshots | fix or replace binding adapter behind the same `RenderDevice`; optimize B0 path |
| SHADER-P1 | compiler-neutral offline `ShaderInterface` matrix using the composition-locked compiler chain | VS/FS/compute SPIR-V, canonical reflection and cache keys are byte-identical across Win/Linux; layouts match 100%; optional task/mesh and ray-query samples pass or capability-skip without loading; compiler types do not cross the tool boundary | compiler manifest, SPIR-V/cache hashes, reflection/layout diff, API scan and captures | fix or replace compiler chain behind the same `ShaderInterface`; block incompatible shader assets |
| RENDER-ASH-P1 | pinned ash adapter executes the complete `RENDER-P1` corpus | `RENDER-P1` thresholds pass for the exact ash version/checksum with 0 ash type outside the renderer backend | candidate manifest, dependency/API scan, validation log, GPU trace and screenshots | do not select ash; use internal/generated bindings behind the same API |
| SHADER-SLANG-P1 | pinned Slang adapter executes the complete `SHADER-P1` corpus | `SHADER-P1` thresholds pass for the exact Slang binary/version/checksum; source mapping is present for every corpus entry | compiler/SBOM manifest, SPIR-V/cache hashes, reflection diff and captures | do not select Slang; use the verified GLSL/HLSL→SPIR-V chain |
| PLATFORM-P1 | SDL adapter lifecycle/input/focus/surface tests | 10 000 create/resize/fullscreen/focus cycles, 0 crash/leak; input timestamp ordering exact; same normalized event fixtures Win/Linux | event trace, memory report | native adapters |
| RENDER-02 | forced `no RT`, `no mesh shader`, bounded descriptors | vertical scene image-diff SSIM ≥0.98 vs approved B0 reference, no missing materials/geometry | capture + diff report | release blocking fix |
| RENDER-03 | injected swapchain/device loss at 100 frame points | swapchain recovery 100%; device recovery or clean save-and-exit ≤10 s; 0 authoritative state corruption | fault report + replay hash | disable recovery attempt, clean exit |
| PACKAGE-01 | clean Win/Linux VM install/run | package launches B0 scene, no undeclared shared library, SBOM complete | VM logs, SBOM, package hashes | block platform package |
| RENDER-04 | windowless offscreen presentation target | CAPTURE-01 passes; 0 PlatformHost/window/surface/swapchain dependency in worker graph; same replay gameplay hash exact; canonical frame root repeated exact on pinned worker | render graph/API scan, replay/frame hashes, window/socket trace | fix target abstraction; observable review AwaitingCapability |
