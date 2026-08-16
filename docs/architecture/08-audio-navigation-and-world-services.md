# SPEC-08: Audio, navigation и world services

| Поле | Значение |
|---|---|
| ID | SPEC-08 |
| Статус | Accepted |
| Версия | 2.5 |
| Последняя проверка | 2026-08-16 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-052](adr/052-derived-world-calendar-and-authored-routine-vertical.md), [ADR-072](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md), [ADR-073](adr/073-deterministic-cognition-owner-vertical.md), [ADR-074](adr/074-systemic-strategic-agent-owner-vertical.md) |
| Заменяет | SPEC-08 2.4; admits the bounded R4d World Activity and tier-cognition consumers while physical traversal remains Proposed |

## Source of truth и ownership

World Services currently owns `WorldPartitionManifestV1` logical topology,
`WorldStreamingSnapshotV1` lifecycle state, the derived
`WorldRoutineCatalogV1`/`WorldRoutineSnapshotV1` calendar-routine projection,
`WorldPopulationCatalogV1`/`WorldPopulationSnapshotV1` durable tier and logical
placement state, `WorldNavigationCatalogV1` graph topology,
`WorldActivityCatalogV1`/`WorldActivitySnapshotV1` systemic activity and reconstructible
streaming caches. The current navigation baseline is one revision-bound graph
node per authored chunk, one tile per region and the typed
`NavigationQueryV1`/`NavigationRoutePlanV1` abstract-transfer result. It does
not create tombstones, generic persistent spatial objects, a spatial index,
reservations, dynamic navmesh overlays or physical path following.
Asset & Persistence владеет immutable schema/content
publication, а Core Runtime — fixed `SimulationTick`, ephemeral residency
mapping и transaction boundaries; ни один из них не становится вторым owner
world topology/placement. Navigation plan не владеет фактическим character
pose: active traversal outcome принадлежит physical/capsule controller. Audio
mixer/device state — presentation only; gameplay hearing использует
deterministic acoustic facts, а не звуковую карту устройства.

## Public boundary

Current public contracts are limited to deterministic acoustic facts,
`WorldPartitionManifestV1`, `WorldStreamingSnapshotV1`, the bounded
world-routine, world-population and world-activity catalog/snapshot/command/event projections,
`WorldNavigationCatalogV1`, `NavigationQueryV1`, `NavigationRoutePlanV1` and
typed errors used by their production consumers. Generic tombstones,
cross-chunk object databases, traversal links, reservations and dynamic
obstacle values remain candidate contracts in the Proposed track below.
Recast/Detour/Steam
Audio/device handles, raw nav poly references, runtime entity/task handles и
mixer buffers являются adapter internals. WorldCommand остаётся единственным
mutable RPG/world входом.

## Current graph navigation and proposed physical traversal

ADR-072 promotes the first engine-owned graph/tile production consumer, typed
content/query schemas and mapped `play`, `content-package`,
`persistence-replay` and report-only `performance` evidence under ADR-046.

Navigation разделена на три contracts:

1. Current `NavigationQueryV1` преобразует start/goal/closed capability and
   graph revision в `NavigationRoutePlanV1` с ordered nodes, integer cost and
   plan hash.
2. A future tactical layer преобразует ближайший corridor/link в
   `PhysicalAvatarIntent`.
3. Physical embodiment доказывает фактическое достижение link/waypoint по
   pose/contact outcomes.

RoutePlan является советом, не teleports и не source of truth. Stuck, push, closed door, moving obstacle или failed jump инвалидируют/перепланируют corridor. Off-mesh link хранит generic action tag и geometric constraints; actual interaction проходит WorldCommand/physical validation boundary.

The first production baseline is the current engine-owned deterministic
graph/tile representation and query. Recast/Detour remains a `Proposed` optional adapter
that may be evaluated only against exact engine-owned results. Future engine
contracts may add `NavSurface`, cooked polygon data, query filters, tactical
corridors and diagnostics. `dt*` types and raw poly refs remain forbidden in saves, AI,
scripting and bundles; candidate stable polygon identity is cooked tile
`AssetId` + local engine index + revision.

## Current graph cooking and proposed navmesh streaming

Current graph content is cooked from typed authoring as a domain-relevant
canonical asset. Every node binds one exact chunk ID, every region tile binds
its sorted node set and revision, and the population catalog binds the exact
navigation catalog revision. The graph is activated with the complete current
project package before world publication. Polygon navmesh construction from
neutral collision geometry, asynchronous tile fetch, cross-tile polygon links
and dynamic obstacle overlays remain Proposed; authoritative door/platform
state would still come from RPG/physics snapshots.

## World services

Only rows marked Current are callable/serializable today. Proposed rows require
their own schemas, consumers and promotion checks.

| Status | Service | Authoritative state | Contract/failure |
|---|---|---|---|
| Current | Partition/streaming | `WorldPartitionManifestV1` topology plus `WorldStreamingSnapshotV1` lifecycle | immutable content definitions come from Asset & Persistence; Runtime mapping/cache is reconstructible; invalid group never partially activates |
| Current | Population placement/tier | 100 `WorldPopulationRecordV1` values in `WorldPopulationSnapshotV1` | full catalog/revision/ledger closure; stale or noncanonical state rejects before joint publication |
| Current | Graph query and abstract transfer | `WorldNavigationCatalogV1`, revision-bound query/plan and one route-hash-bound logical transfer | positive integer graph costs and canonical tie-break; `Active` transfer returns `PHYSICAL_TRAVERSAL_REQUIRED` without mutation |
| Current | Systemic activity | `WorldActivityCatalogV1` plus separately persisted `WorldActivitySnapshotV1` | exact revision/evidence validates `Unassigned -> Assigned -> Working -> Completed`; no RPG or pose write |
| Current | Acoustics facts | source, listener, loudness class, occlusion zone, tick | deterministic gameplay perception; independent from output device |
| Proposed R4c+ | Generic persistent objects | tombstone and cross-chunk reference beyond the current population records | no current schema/API; future invalid groups fail before publication |
| Proposed R4c+ | Spatial query | index rebuilt from future owned transforms/chunks | stale revision rejected; never replaces an exact physics query |
| Proposed R4c+ | Reservation | PersistentId resource/agent, interval, priority, expiry | future mutation only through `WorldCommand`; deterministic conflict order |
| Proposed R4c+ | Trigger/region | cooked volumes plus overlap state | future physical/capsule fact proposes an outcome; RPG validates it |

## Audio architecture

`AudioScene` consumes PresentationSnapshot, cooked clips, emitters/listener, rooms/portals и world acoustic parameters. Engine-native baseline MUST поддерживать sample playback, streaming, spatial attenuation/panning, priority/voice limiting и zone reverb fallback без proprietary SDK.

Steam Audio — `Proposed` optional propagation adapter. Его exact version должна быть технически совместима и разрешена для выбранного способа распространения; handles/types не покидают adapter. Propagation result MAY improve rendering and gameplay acoustic estimate only если deterministic precomputed/query mode удовлетворяет replay policy; иначе gameplay hearing остаётся engine deterministic approximation, а Steam Audio — presentation-only.

ASR/TTS принадлежат `ai-host`; audio runtime получает/отдаёт bounded PCM/encoded streams через versioned messages, не model APIs. Отсутствие voice services использует text/subtitle и authored/default voice fallback.

Displayless audio check использует тот же deterministic `AudioScene`, но sink — bounded canonical PCM/WAV, не hardware device. Конфигурация фиксирует listener, buses, sample rate/channels и semantic tick window. Gameplay acoustic facts проверяются отдельно exact/tolerance assertions.

## Data flow

Current bounded flow: `typed graph/tile content → package activation →
NavigationQueryV1 → NavigationRoutePlanV1 → Abstract WorldCommand transfer`.
Future physical flow continues with `tactical PhysicalAvatarIntent → physics
outcome → route progress/replan`.

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

`AUDIO-*` rows are current checks selected by an affected audio change.
`NAV-P1`, `NAV-P3` and the bounded `WORLD-02` branch are current under ADR-072;
physical corridor following remains deferred.

| Check ID | Scenario / command | Expected behavior / fallback |
|---|---|---|
| NAV-P1 | Engine-owned deterministic graph/tile cook and query baseline | The 64 chunk-bound nodes, four region tiles, catalog/plan hashes and route tie-break are exact across repeats and input permutations. Optional Recast must match the engine contract or remain disabled. |
| NAV-P2 | **Deferred physical-path recipe; no current gate.** Door, off-mesh, push, stuck and streamed polygon fixtures | Stale paths reject, bounded fixtures either reach the goal or return no-path, and no path teleports an actor; deterministically replan or idle. |
| NAV-P3 | ADR-016 deterministic `r4-100npc.v1` report-only workload | 1,000 warm-up plus 10,000 measured ticks produce exact 16/32/52 due/query/tier-cognition counts and roots with no starvation/drop/fabricated outcomes. Timing remains report-only; the current unsupported-host navigation p95/p99 `4292/4870 us` exceeds `1250/1500 us` and is not a pass or B-12 evidence. |
| AUDIO-P1 | Engine baseline and optional Steam Audio candidate | Baseline playback always works, device loss recovers without gameplay differences, and optional propagation stays within scenario tolerance; fall back to baseline attenuation/panning/zones. |
| AUDIO-L1 | Steam Audio version and distribution matrix | The selected version is compatible with target platforms and may be redistributed under the project policy; otherwise do not ship the adapter. |
| WORLD-02 | Current abstract-transfer ownership branch plus deferred physical corpus | World Services owns tier/logical placement and commits one exact Abstract transfer; Physics alone owns active traversal, so an Active transfer returns `PHYSICAL_TRAVERSAL_REQUIRED`. Save/Replay V9 preserve the result; broader traversal deterministically replans or idles. |
| AUDIO-02 | Displayless canonical PCM and event synchronization | PCM is deterministic on the pinned sink, event alignment is within one sample, acoustic facts are exact, and gameplay hashes do not depend on audio output; retain gameplay and use the baseline audio path on failure. |

## Strategic Agent reciprocal boundary

Current World Services owns logical location, population tier, authored
routine/activity and `RoutePlan`. The ADR-073/074 consumer reads only the
immutable revision-bound population/route projection and may persist a logical-
route intent; it cannot own route topology, commit a transfer, fabricate
traversal success or teleport an actor. Tier selection uses simulation-owned
region/importance/profile facts and never renderer camera, frustum, FPS or wall
time. R4d tier-wide work classification and owner-validated activity are
current; physical corridor following and fabricated Abstract outcomes remain
forbidden.
