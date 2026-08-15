# SPEC-08: Audio, navigation и world services

| Поле | Значение |
|---|---|
| ID | SPEC-08 |
| Статус | Accepted |
| Версия | 2.2 |
| Последняя проверка | 2026-08-15 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-052](adr/052-derived-world-calendar-and-authored-routine-vertical.md) |
| Заменяет | SPEC-08 2.1; admits the bounded routine owner while retaining broader population/navigation as Proposed |

## Source of truth и ownership

World Services currently owns `WorldPartitionManifestV1` logical topology,
`WorldStreamingSnapshotV1` lifecycle state, the derived
`WorldRoutineCatalogV1`/`WorldRoutineSnapshotV1` calendar-routine projection,
reconstructible streaming caches and audio scene descriptions. The manifest's current placement/residency
profile hashes are opaque empty-profile bindings; there is no current durable
placement, tombstone, cross-chunk object or spatial-index public API.
Navigation, placement and route reservations are Proposed ownership targets
described below; no current API or ProductCheck follows from those sections.
Broader population tiers, navigation and weather remain Proposed R4b scope;
the bounded R4a routine is current under SPEC-20/ADR-052. Asset & Persistence владеет immutable schema/content
publication, а Core Runtime — fixed `SimulationTick`, ephemeral residency
mapping и transaction boundaries; ни один из них не становится вторым owner
world topology/placement. Navigation plan не владеет фактическим character
pose: active traversal outcome принадлежит physical/capsule controller. Audio
mixer/device state — presentation only; gameplay hearing использует
deterministic acoustic facts, а не звуковую карту устройства.

## Public boundary

Current public contracts are limited to deterministic acoustic facts,
`WorldPartitionManifestV1`, `WorldStreamingSnapshotV1`, the bounded
world-routine catalog/snapshot/command/event projections and typed errors used by
their production consumers. Durable placement/tombstone/cross-chunk values,
`NavigationQuery`, `RoutePlan` and traversal-link values are candidate
contracts in the Proposed track below.
Recast/Detour/Steam
Audio/device handles, raw nav poly references, runtime entity/task handles и
mixer buffers являются adapter internals. WorldCommand остаётся единственным
mutable RPG/world входом.

## Proposed navigation intent и physical traversal

This complete navigation section is a bounded R4b design recipe, not a current
API, format or check. Promotion requires an engine-owned graph/tile production
consumer, typed content/query schemas and mapped `play`, `content-package`,
`persistence-replay` and conditional `performance` evidence under ADR-046.

Navigation разделена на три contracts:

1. `NavigationQuery` преобразует start/goal/capability profile/world revision в versioned `RoutePlan` с corridor, traversal links, costs и validity token.
2. Tactical layer преобразует ближайший corridor/link в `PhysicalAvatarIntent`.
3. Physical embodiment доказывает фактическое достижение link/waypoint по pose/contact outcomes.

RoutePlan является советом, не teleports и не source of truth. Stuck, push, closed door, moving obstacle или failed jump инвалидируют/перепланируют corridor. Off-mesh link хранит generic action tag и geometric constraints; actual interaction проходит WorldCommand/physical validation boundary.

The first production baseline is an engine-owned deterministic graph/tile
representation and query. Recast/Detour remains a `Proposed` optional adapter
that may be evaluated only against exact engine-owned results. Future engine
contracts own `NavSurface`, tiled nav data, query/filter, RoutePlan and
diagnostics. `dt*` types and raw poly refs remain forbidden in saves, AI,
scripting and bundles; candidate stable polygon identity is cooked tile
`AssetId` + local engine index + revision.

## Proposed navigation cooking/streaming

Navmesh строится детерминированно из validated neutral collision geometry and
traversal profiles SPEC-24. Build parameters и tool/schema hashes входят в
ContentHash. Tiles bind to exact cells/chunks in `WorldPartitionManifestV1` и
проходят current admission SPEC-03/SPEC-25. Future asynchronous fetch pipeline
остаётся Proposed в SPEC-23. Cross-tile link не становится доступным, пока обе
required revisions не validated and atomically active. Dynamic obstacle overlay
— runtime-owned rebuildable cache; authoritative door/platform state приходит
из RPG/physics snapshots.

## World services

Only rows marked Current are callable/serializable today. Proposed rows are
ownership constraints for R4b and require their own schemas, consumers and
promotion checks.

| Status | Service | Authoritative state | Contract/failure |
|---|---|---|---|
| Current | Partition/streaming | `WorldPartitionManifestV1` topology plus `WorldStreamingSnapshotV1` lifecycle | immutable content definitions come from Asset & Persistence; Runtime mapping/cache is reconstructible; invalid group never partially activates |
| Current | Acoustics facts | source, listener, loudness class, occlusion zone, tick | deterministic gameplay perception; independent from output device |
| Proposed R4b | Durable placement | persistent object/chunk binding, tombstone and cross-chunk reference | no current schema/API; future invalid groups fail before publication |
| Proposed R4b | Spatial query | index rebuilt from future owned transforms/chunks | stale revision rejected; never replaces an exact physics query |
| Proposed R4b | Reservation | PersistentId resource/agent, interval, priority, expiry | future mutation only through `WorldCommand`; deterministic conflict order |
| Proposed R4b | Trigger/region | cooked volumes plus overlap state | future physical/capsule fact proposes an outcome; RPG validates it |

## Audio architecture

`AudioScene` consumes PresentationSnapshot, cooked clips, emitters/listener, rooms/portals и world acoustic parameters. Engine-native baseline MUST поддерживать sample playback, streaming, spatial attenuation/panning, priority/voice limiting и zone reverb fallback без proprietary SDK.

Steam Audio — `Proposed` optional propagation adapter. Его exact version должна быть технически совместима и разрешена для выбранного способа распространения; handles/types не покидают adapter. Propagation result MAY improve rendering and gameplay acoustic estimate only если deterministic precomputed/query mode удовлетворяет replay policy; иначе gameplay hearing остаётся engine deterministic approximation, а Steam Audio — presentation-only.

ASR/TTS принадлежат `ai-host`; audio runtime получает/отдаёт bounded PCM/encoded streams через versioned messages, не model APIs. Отсутствие voice services использует text/subtitle и authored/default voice fallback.

Displayless audio check использует тот же deterministic `AudioScene`, но sink — bounded canonical PCM/WAV, не hardware device. Конфигурация фиксирует listener, buses, sample rate/channels и semantic tick window. Gameplay acoustic facts проверяются отдельно exact/tolerance assertions.

## Data flow

Future navigation flow: `cooked collision → graph/tile build → tiled bundles →
runtime query → RoutePlan → tactical PhysicalAvatarIntent → physics outcome →
route progress/replan`.

`DomainEvent/world state → deterministic acoustic fact → AI perception`; параллельно `PresentationSnapshot + clip/voice stream → AudioScene → optional propagation → mixer/device`.

## Failure semantics

- Missing/corrupt nav tile → area unavailable с diagnostic; agent выбирает alternate/idle fallback, не проходит сквозь geometry.
- Invalid/stale current partition or stream-admission plan → reject the
  complete group and retain prior topology/active generation.
- Future invalid placement/tombstone/cross-chunk input → reject before
  publication; this rule creates no current placement API.
- Resource/backpressure fault → required World Services work remains queued/deferred with exact age/reason or blocks before its mandatory boundary; it is never silently dropped.
- Stale RoutePlan revision → reject before movement command, request replan.
- Nav query budget exceeded → bounded partial/no-path result и retry policy, simulation tick не блокируется.
- Audio device loss → gameplay продолжается, mixer reconnects; acoustic gameplay facts сохраняются.
- Optional propagation init/runtime failure → engine-native audio fallback без world mutation.
- Voice stream late/crashed → subtitle/text fallback; dialogue state не ждёт playback completion, если content явно не требует authored timing event.
- Audio check sink/device/encoder failure → gameplay/replay остаётся valid; acoustic check повторяется на доступном canonical PCM sink.
- PCM или event-to-sample synchronization mismatch → audio check fails; gameplay state не изменяется.

## Product checks

`AUDIO-*` rows are current checks selected by an affected audio change. The
navigation/world-traversal rows are deferred R4b recipes and create no current
gate while their production consumer and promoting ADR do not exist.

| Check ID | Scenario / command | Expected behavior / fallback |
|---|---|---|
| NAV-P1 | **Deferred R4b recipe; no current gate.** Engine-owned deterministic graph/tile cook and query baseline | Neutral graph/tile artifacts and 1,000 start/goal query results are byte-identical across required targets. Optional Recast must match the engine contract or remain disabled. |
| NAV-P2 | **Deferred R4b recipe; no current gate.** Door, off-mesh, push, stuck and streaming fixtures | Stale paths are rejected, bounded fixtures either reach the goal or return no-path, and no path teleports an actor; deterministically replan or idle. |
| NAV-P3 | **Deferred R4b recipe; no current gate.** ADR-016 deterministic 100-NPC workload after the population/navigation consumer exists | Due navigation stays within p95 ≤1,250 us / p99 ≤1,500 us with bounded deferral and no starvation, drop or unowned span; until then the workload remains typed `NOT_RUN`/unavailable. |
| AUDIO-P1 | Engine baseline and optional Steam Audio candidate | Baseline playback always works, device loss recovers without gameplay differences, and optional propagation stays within scenario tolerance; fall back to baseline attenuation/panning/zones. |
| AUDIO-L1 | Steam Audio version and distribution matrix | The selected version is compatible with target platforms and may be redistributed under the project policy; otherwise do not ship the adapter. |
| WORLD-02 | **Deferred R4b recipe; no current gate.** Navigation/world-service/physical ownership and traversal corpus | World Services owns route/reservation state, the physical controller alone owns traversal outcome, stale plans reject, and replay is stable; deterministically replan or idle. |
| AUDIO-02 | Displayless canonical PCM and event synchronization | PCM is deterministic on the pinned sink, event alignment is within one sample, acoustic facts are exact, and gameplay hashes do not depend on audio output; retain gameplay and use the baseline audio path on failure. |

## Strategic Agent reciprocal boundary

Future R4b World Services owns logical location, population tier, authored
routine/job assignment and `RoutePlan`. SPEC-32 may read only immutable
revision-bound projections and may request navigation/placement work; it cannot
own route topology, fabricate traversal success or teleport an actor. Tier
selection uses simulation-owned region/distance/importance/profile facts and
never renderer camera, frustum, FPS or wall time. These semantics do not create
a current navigation schema before the R4b production consumer.
