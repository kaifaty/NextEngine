# SPEC-08: Audio, navigation и world services

| Поле | Значение |
|---|---|
| ID | SPEC-08 |
| Статус | Accepted |
| Версия | 1.9 |
| Последняя проверка | 2026-08-08 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md) |
| Заменяет | SPEC-08 1.8 speculative population/calendar and generic jobs clauses |

## Source of truth и ownership

World Services владеет navigation representation/query service, route
reservations, `WorldPartitionManifestV1` logical topology, durable spatial
placement/tombstones, spatial indices и audio scene descriptions. Future
calendar/weather/population authority остаётся Proposed в SPEC-20 и не является
current contract. Asset & Persistence владеет immutable schema/content
publication, а Core Runtime — fixed `SimulationTick`, ephemeral residency
mapping и transaction boundaries; ни один из них не становится вторым owner
world topology/placement. Navigation plan не владеет фактическим character
pose: active traversal outcome принадлежит physical/capsule controller. Audio
mixer/device state — presentation only; gameplay hearing использует
deterministic acoustic facts, а не звуковую карту устройства.

## Public boundary

Public contracts ограничены `NavigationQuery`, `RoutePlan`, traversal link
values, deterministic acoustic facts, `WorldPartitionManifestV1`, durable
placement/tombstone/cross-chunk values и typed errors. Recast/Detour/Steam
Audio/device handles, raw nav poly references, runtime entity/task handles и
mixer buffers являются adapter internals. WorldCommand остаётся единственным
mutable RPG/world входом.

## Navigation intent и physical traversal

Navigation разделена на три contracts:

1. `NavigationQuery` преобразует start/goal/capability profile/world revision в versioned `RoutePlan` с corridor, traversal links, costs и validity token.
2. Tactical layer преобразует ближайший corridor/link в `PhysicalAvatarIntent`.
3. Physical embodiment доказывает фактическое достижение link/waypoint по pose/contact outcomes.

RoutePlan является советом, не teleports и не source of truth. Stuck, push, closed door, moving obstacle или failed jump инвалидируют/перепланируют corridor. Off-mesh link хранит generic action tag и geometric constraints; actual interaction проходит WorldCommand/physical validation boundary.

Recast/Detour — `Proposed` adapter. Engine владеет `NavSurface`, tiled nav data, query/filter, RoutePlan и diagnostics. `dt*` types и raw poly refs запрещены в saves, AI, scripting и bundles; stable nav polygon identity состоит из cooked tile AssetId + local engine index + revision.

## Navigation cooking/streaming

Navmesh строится детерминированно из validated neutral collision geometry and
traversal profiles SPEC-24. Build parameters и tool/schema hashes входят в
ContentHash. Tiles bind to exact cells/chunks in `WorldPartitionManifestV1` и
проходят current admission SPEC-03/SPEC-25. Future asynchronous fetch pipeline
остаётся Proposed в SPEC-23. Cross-tile link не становится доступным, пока обе
required revisions не validated and atomically active. Dynamic obstacle overlay
— runtime-owned rebuildable cache; authoritative door/platform state приходит
из RPG/physics snapshots.

## World services

| Service | Authoritative state | Contract/failure |
|---|---|---|
| Spatial query | runtime index rebuilt from owned transforms/chunks | stale revision rejected; не заменяет physics exact query |
| Reservation | PersistentId resource/agent, interval, priority, expiry | accepted только WorldCommand; deterministic conflict order |
| Trigger/region | cooked volumes + overlap state | physical/capsule pose generates candidate; RPG validates outcome |
| Acoustics facts | source, listener, loudness class, occlusion zone, tick | deterministic gameplay perception; independent from output device |
| Partition/placement | `WorldPartitionManifestV1`, durable object placement/tombstones, cross-chunk references and canonical residency interests | immutable content definitions come from Asset & Persistence; Runtime mapping is ephemeral; invalid group never partially activates |

## Audio architecture

`AudioScene` consumes PresentationSnapshot, cooked clips, emitters/listener, rooms/portals и world acoustic parameters. Engine-native baseline MUST поддерживать sample playback, streaming, spatial attenuation/panning, priority/voice limiting и zone reverb fallback без proprietary SDK.

Steam Audio — `Proposed` optional propagation adapter. Его exact version должна быть технически совместима и разрешена для выбранного способа распространения; handles/types не покидают adapter. Propagation result MAY improve rendering and gameplay acoustic estimate only если deterministic precomputed/query mode удовлетворяет replay policy; иначе gameplay hearing остаётся engine deterministic approximation, а Steam Audio — presentation-only.

ASR/TTS принадлежат `ai-host`; audio runtime получает/отдаёт bounded PCM/encoded streams через versioned messages, не model APIs. Отсутствие voice services использует text/subtitle и authored/default voice fallback.

Displayless audio check использует тот же deterministic `AudioScene`, но sink — bounded canonical PCM/WAV, не hardware device. Конфигурация фиксирует listener, buses, sample rate/channels и semantic tick window. Gameplay acoustic facts проверяются отдельно exact/tolerance assertions.

## Data flow

`cooked collision → nav build → tiled bundles → runtime query → RoutePlan → tactical PhysicalAvatarIntent → physics outcome → route progress/replan`.

`DomainEvent/world state → deterministic acoustic fact → AI perception`; параллельно `PresentationSnapshot + clip/voice stream → AudioScene → optional propagation → mixer/device`.

## Failure semantics

- Missing/corrupt nav tile → area unavailable с diagnostic; agent выбирает alternate/idle fallback, не проходит сквозь geometry.
- Invalid/stale partition, placement, cross-chunk reference or stream-admission plan → reject the complete group, retain prior topology/placement/active generation and deterministically replan/defer.
- Resource/backpressure fault → required World Services work remains queued/deferred with exact age/reason or blocks before its mandatory boundary; it is never silently dropped.
- Stale RoutePlan revision → reject before movement command, request replan.
- Nav query budget exceeded → bounded partial/no-path result и retry policy, simulation tick не блокируется.
- Audio device loss → gameplay продолжается, mixer reconnects; acoustic gameplay facts сохраняются.
- Optional propagation init/runtime failure → engine-native audio fallback без world mutation.
- Voice stream late/crashed → subtitle/text fallback; dialogue state не ждёт playback completion, если content явно не требует authored timing event.
- Audio check sink/device/encoder failure → gameplay/replay остаётся valid; acoustic check повторяется на доступном canonical PCM sink.
- PCM или event-to-sample synchronization mismatch → audio check fails; gameplay state не изменяется.

## Product checks

| Check ID | Scenario / command | Expected behavior / fallback |
|---|---|---|
| NAV-P1 | Recast deterministic tiled cook on Windows and Linux | Neutral nav tiles and 1,000 start/goal query results are byte-identical; use the engine-owned graph navigation fallback if the adapter differs. |
| NAV-P2 | Door, off-mesh, push, stuck and streaming fixtures | Stale paths are rejected, bounded fixtures either reach the goal or return no-path, and no path teleports an actor; replan or use the graph adapter. |
| NAV-P3 | ADR-016 deterministic 100-NPC workload | Due navigation stays within p95 ≤1,250 us / p99 ≤1,500 us with bounded deferral and no starvation, drop or unowned span; lower deterministic cadence/LOD if needed. |
| AUDIO-P1 | Engine baseline and optional Steam Audio candidate | Baseline playback always works, device loss recovers without gameplay differences, and optional propagation stays within scenario tolerance; fall back to baseline attenuation/panning/zones. |
| AUDIO-L1 | Steam Audio version and distribution matrix | The selected version is compatible with target platforms and may be redistributed under the project policy; otherwise do not ship the adapter. |
| WORLD-02 | Navigation/world-service/physical ownership and traversal corpus | World Services owns route/reservation state, the physical controller alone owns traversal outcome, stale plans reject, and replay is stable; deterministically replan or idle. |
| AUDIO-02 | Displayless canonical PCM and event synchronization | PCM is deterministic on the pinned sink, event alignment is within one sample, acoustic facts are exact, and gameplay hashes do not depend on audio output; retain gameplay and use the baseline audio path on failure. |
