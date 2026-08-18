# SPEC-04: Rendering и platform

| Поле | Значение |
|---|---|
| ID | SPEC-04 |
| Статус | Accepted |
| Версия | 2.5 |
| Последняя проверка | 2026-08-18 |
| Нормативные зависимости | [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-29](29-platform-host-and-application-session.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [ADR-003](adr/003-vulkan-renderer-and-shader-toolchain.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-036](adr/036-thoth-reference-performance-profile.md), [ADR-045](adr/045-low-overhead-hard-performance-evidence.md), [ADR-082](adr/082-linux-first-development-and-deferred-windows-host.md) |
| Заменяет | SPEC-04 2.4; moves active renderer development/report evidence to Linux while preserving deferred Windows release targets |

## Technical authority boundary

Authoritative visual inputs — immutable `PresentationSnapshotV3` и cooked
render assets по
[SPEC-30](30-presentation-extraction-and-render-content.md). Presentation
extraction publishes the canonical snapshot at the Runtime-declared boundary.
The renderer keeps private platform/window/input adapters, graphics-device
state, render graph, GPU/UI caches, frame interpolation and capability
selection. `ActionMapManifest`,
`InputContext`, `PlayerActionFrame` and semantic
UI/camera/localization/accessibility state follow
[SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md).
GPU resources, widget tree, camera output и current frame не являются gameplay
source of truth; после device loss они MUST быть восстанавливаемы из exact
content/profile/snapshot inputs.

## PlatformHost boundary

`PlatformHost` предоставляет engine-owned `PlatformCapabilitySetV1`, `PlatformTimebaseV1`, `PlatformEventV1` и `NormalizedControlEventV1` по [SPEC-29](29-platform-host-and-application-session.md), window/presentation-target lifecycle, bounded monitor/DPI projection, clipboard opt-in и crash-safe user-data path capability. SDL 3 — `Proposed` adapter. SDL/Win32/XInput/X11/Wayland handles, native events и scancodes MUST NOT пересекать platform adapter; fallback — thin native adapters за тем же engine contract.

Input adapter нормализует physical device controls, но gameplay bindings, context stack, quantization и device-independent action IDs принадлежат SPEC-18 выше platform layer. Window close, focus loss, suspend и surface invalidation приходят typed events. Platform callback, UI callback или camera query MUST NOT менять world state напрямую.

Optional capture worker не создаёт interactive `PlatformHost`:
input/window/monitor/clipboard/surface lifecycle отсутствуют. Его
`ApplicationSessionManifestV2` допускает только `DisplaylessOffscreen`;
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
2. Renderer получает latest two atomically published `PresentationSnapshotV3` values и presentation-only interpolation alpha.
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
invalidates affected private cache generations и atomically rebuilds only from
exact content/interface/profile/`PresentationSnapshotV3` inputs.
Simulation/session/snapshot roots
остаются неизменны; policy MAY request typed `Suspended`, но device timing не
выбирает gameplay outcome. CPU headless не загружает renderer. Capture-worker
crash/device loss не повреждает source replay и не публикует partial artifacts.
Shader/interface/material/color mismatch является pre-use content failure, а
не best-effort draw.

## Platform packaging

Shipping package использует `PackageManifestV4`. Его `runtime_profile` MUST
объявлять target ABI, canonical direct-library list каждого binary, maximum
required GLIBC и все внешние runtime prerequisites; manifest также связывает
exact `project_lock_sha256`. Earlier package artifacts current runtime rejects
typed unsupported and never migrates in place.

Windows x86_64 MSVC profile использует dynamic system VC++ x64 runtime и
системный Vulkan loader `vulkan-1.dll`; выбранная loader ABI strategy требует
Vulkan API 1.3. Linux x86_64 GNU profile использует Ubuntu 22.04 / glibc 2.35
как minimum baseline, системный loader `libvulkan.so.1`, Vulkan API 1.3 и
активную X11 либо Wayland desktop session. На обеих targets Vulkan ICD/GPU
driver является внешним user-installed prerequisite; loader и driver MUST NOT
включаться в package. SDL3 для R1 компонуется статически.

Packaging check MUST сверять заявленные direct dependencies с PE64/ELF64
binary. ELF с machine-local `RPATH`/`RUNPATH`, non-standard x86_64 GNU
interpreter или требованием GLIBC выше 2.35 отклоняется до публикации. Unknown
или незаявленная third-party library также является package failure.

Copied release binaries MUST запускаться из package с очищенным environment,
изолированными state/home/temp paths и без inherited `LD_*`, `VK_*` или `SDL_*`
loader overrides. Linux smoke MAY сохранить только desktop-session variables,
то есть `DISPLAY`, `WAYLAND_DISPLAY`, `XDG_RUNTIME_DIR`,
`DBUS_SESSION_BUS_ADDRESS`, `XAUTHORITY`, необходимые выбранному X11/Wayland
host; Windows smoke MAY сохранить `SYSTEMROOT`/`WINDIR`. Каждый smoke имеет
30-second timeout и bounded output.
Runtime profile, dependency, ABI и timeout failures используют стабильные коды
`NATIVE_GATE_PACKAGE_RUNTIME_PROFILE_INVALID`,
`NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING`,
`NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED` и
`NATIVE_GATE_PACKAGE_SMOKE_TIMEOUT`.

Debug layers/RenderDoc markers MAY быть optional package, но их отсутствие не
меняет cache keys release shaders.

## B0 performance profile

Текущий development check выполняет B0 workload на native Linux и сохраняет
timing только как `REPORT_ONLY`. Hard B0 design следует ADR-036 и принимается
только на полном `ref-win-thoth-v1` в `release`, но ADR-082 откладывает этот
Windows execution до explicit pre-R7 bring-up:

- primary `1920×1080`: `p95 <= 14,000 us`, `p99 <= 16,670 us`;
- fallback `b0-safe-720p30`, `1280×720`: `p99 <= 33,330 us`.

Полный frame sample равен
`max(cpu_extract_and_submit_us, gpu_timestamp_duration_us)`. VSync wait
исключается, missed deadlines сохраняются отдельным counter. Три
representative 60-second окна — exploration, combat и UI/dialogue — каждое
использует 600 warm-up и 3,600 measured frames. Nearest-rank percentiles
сохраняют все outliers.

Fallback выбирается только при launch в v1. Его `PASS` не меняет primary
`FAIL`. Render quality, timestamp query state, profiler state и chosen
presentation profile не меняют commands/events/replay result или
authoritative roots.

Production workload `r2-alpha-render.v3` загружает `projects/reference-alpha`
через authoring → cook → activate, использует production Vulkan adapter на
текущем Linux X11/Wayland desktop host и
выполняет шесть независимых profile/window runs: exploration, combat и
UI/dialogue отдельно для primary и fallback. Каждый run сохраняет свои 600
warm-up и 3 600 measured samples, exact Vulkan timestamp-query accounting,
canonical logical resource charges, process/device counters и authoritative
roots. Report mode может завершить внешний ProductCheck как `PASS` только с
вложенным `REPORT_ONLY`; hard timing `PASS` по-прежнему требует clean commit,
полного THOTH fingerprint, compatible ten-run baseline и всех preflight checks
ADR-036/ADR-045. Статические render fixtures остаются smoke и не подменяют этот
workload.

## Product checks

| ID | Scenario | Expected behavior | Fallback |
|---|---|---|---|
| `RENDER-P1` | Run an engine-owned Vulkan B0 gameplay scene on the active Linux target with the selected binding adapter; run Windows later during explicit bring-up. | No validation errors or leaked objects; Linux timing is `REPORT_ONLY`, deferred THOTH primary/fallback evidence remains unclaimed, and no binding/vendor type escapes the renderer backend. | Fix or replace the adapter behind the same `RenderDevice`; select declared `b0-safe-720p30` at launch without hiding primary failure or changing gameplay. |
| `SHADER-P1` | Compile the offline `ShaderInterface` matrix with the selected compiler chain. | VS/FS/compute SPIR-V, canonical reflection and platform-neutral artifact keys match across Win/Linux; layouts match exactly; unavailable optional task/mesh or ray-query paths remain unloaded. | Replace the compiler adapter behind the same interface and reject incompatible shader assets. |
| `RENDER-ASH-P1` | Exercise the B0 scene through the proposed ash adapter. | The renderer behavior matches `RENDER-P1` and no ash type crosses the backend boundary. | Keep the internal/generated binding adapter. |
| `SHADER-SLANG-P1` | Exercise the shader matrix through the proposed Slang adapter. | The behavior matches `SHADER-P1` and source mapping exists for every compiled entry. | Keep the verified GLSL/HLSL-to-SPIR-V compiler adapter. |
| `PLATFORM-P1` | Repeated create/resize/fullscreen/focus/input/surface lifecycle on supported desktop hosts. | No crash or leak; normalized event ordering is stable and native handles remain private. | Use the thin native adapter behind the same platform contract. |
| `RENDER-02` | Force `no RT`, `no mesh shader` and bounded descriptors. | The representative scene remains complete and playable with no missing required material or geometry. Optional screenshots or image diffs may help diagnose regressions but are not the correctness oracle. | Disable the unsupported enhanced path and use the cooked B0 path. |
| `RENDER-03` | Inject swapchain and device loss at representative frame boundaries. | Interactive target recreation or clean suspension/exit completes without authoritative-state corruption; incomplete private cache state is never exposed. | Stop recovery attempts, preserve the last complete save/session state and exit cleanly. |
| `PACKAGE-01` | Install and run a clean Win/Linux package. | `PackageManifestV4` matches exact project lock, PE/ELF imports and the declared ABI baseline; isolated copied `game`/`headless` launches pass and missing runtime prerequisites have stable diagnostics. | Do not distribute the broken target package; repair its loader/dependency declaration. |
| `RENDER-04` | Optionally run a developer capture through the displayless offscreen target. | No window/display/surface/swapchain dependency is created; replay gameplay hash remains unchanged and repeated normalized frame output is stable for the selected profile. | Disable capture tooling and fix the target abstraction; normal game/headless operation remains available. |
