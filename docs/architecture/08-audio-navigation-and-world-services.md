# SPEC-08: Audio, navigation и world services

| Поле | Значение |
|---|---|
| ID | SPEC-08 |
| Статус | Accepted |
| Версия | 1.8 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-016](adr/016-compositional-gameplay-budgets.md) |
| Заменяет | отсутствует |

## Source of truth и ownership

World Services Team владеет navigation representation/query service, route reservations, `WorldCalendarStateV1`, simulation calendar/weather state, population schedules, `WorldPartitionManifestV1` logical topology, durable spatial placement/tombstones, spatial indices и audio scene descriptions. Asset & Persistence Team владеет immutable schema/content/bundle publication, а Core Runtime — fixed `SimulationTick`, job/result admission, ephemeral residency mapping и scheduling/transaction boundaries; ни один из них не становится вторым owner world topology/placement. Navigation plan не владеет фактическим character pose: active traversal outcome принадлежит physical/capsule controller. Audio mixer/device state — presentation only; gameplay hearing использует deterministic acoustic facts, а не звуковую карту устройства.

## Public boundary

Public contracts ограничены `NavigationQuery`, `RoutePlan`, traversal link values, deterministic acoustic facts, `WorldCalendarStateV1`, `WorldPartitionManifestV1`, durable placement/tombstone/cross-chunk values, `TierExecutionProfile`, calendar/weather/reservation/region snapshots и typed errors. Recast/Detour/Steam Audio/device handles, raw nav poly references, runtime entity/task handles и mixer buffers являются adapter internals. WorldCommand остаётся единственным mutable RPG/world входом.

## Navigation intent и physical traversal

Navigation разделена на три contracts:

1. `NavigationQuery` преобразует start/goal/capability profile/world revision в versioned `RoutePlan` с corridor, traversal links, costs и validity token.
2. Tactical layer преобразует ближайший corridor/link в `PhysicalAvatarIntent`.
3. Physical embodiment доказывает фактическое достижение link/waypoint по pose/contact outcomes.

RoutePlan является советом, не teleports и не source of truth. Stuck, push, closed door, moving obstacle или failed jump инвалидируют/перепланируют corridor. Off-mesh link хранит generic action tag и geometric constraints; actual interaction проходит WorldCommand/physical gate.

Recast/Detour — `Proposed` adapter. Engine владеет `NavSurface`, tiled nav data, query/filter, RoutePlan и diagnostics. `dt*` types и raw poly refs запрещены в saves, AI, scripting и bundles; stable nav polygon identity состоит из cooked tile AssetId + local engine index + revision.

## Navigation cooking/streaming

Navmesh строится детерминированно из validated neutral collision geometry and traversal profiles SPEC-24. Build parameters и tool/schema hashes входят в ContentHash. Tiles bind to exact cells/chunks in `WorldPartitionManifestV1`; их required dependency group проходит тот же deterministic resource/streaming admission SPEC-23/SPEC-25. Cross-tile link не становится доступным, пока обе required revisions не validated and atomically active. Dynamic obstacle overlay — runtime-owned rebuildable cache; authoritative door/platform state приходит из RPG/physics snapshots.

## World services

| Service | Authoritative state | Contract/failure |
|---|---|---|
| Calendar/time | World Services-owned `WorldCalendarStateV1`: integer `world_tick`, calendar-definition AssetId/hash, integer epoch/scale mapping, revision и last causal command в World Services save segment | renderer/audio получают projection; Runtime `SimulationTick` остаётся отдельным; wall clock не влияет |
| Weather | generic weather state machine + seed/transition tick | presentation samples state; missing effect asset uses fallback |
| Spatial query | runtime index rebuilt from owned transforms/chunks | stale revision rejected; не заменяет physics exact query |
| Reservation | PersistentId resource/agent, interval, priority, expiry | accepted только WorldCommand; deterministic conflict order |
| Trigger/region | cooked volumes + overlap state | physical/capsule pose generates candidate; RPG validates outcome |
| Acoustics facts | source, listener, loudness class, occlusion zone, tick | deterministic gameplay perception; independent from output device |
| Partition/placement | `WorldPartitionManifestV1`, durable object placement/tombstones, cross-chunk references and canonical residency interests | immutable content definitions come from Asset & Persistence; Runtime mapping is ephemeral; invalid group never partially activates |

## Audio architecture

`AudioScene` consumes PresentationSnapshot, cooked clips, emitters/listener, rooms/portals и world acoustic parameters. Engine-native baseline MUST поддерживать sample playback, streaming, spatial attenuation/panning, priority/voice limiting и zone reverb fallback без proprietary SDK.

Steam Audio — `Proposed` optional propagation adapter и проходит technical + license/redistribution gate. Its handles/types do not leave adapter. Propagation result MAY improve rendering and gameplay acoustic estimate only если deterministic precomputed/query mode удовлетворяет replay policy; иначе gameplay hearing остаётся engine deterministic approximation, а Steam Audio — presentation-only.

ASR/TTS принадлежат `ai-host`; audio runtime получает/отдаёт bounded PCM/encoded streams через versioned messages, не model APIs. Отсутствие voice services использует text/subtitle и authored/default voice fallback.

Audio HumanReviewRequired capture использует тот же deterministic AudioScene graph, но sink — bounded canonical PCM/WAV, не hardware device. CapturePlan фиксирует listener, buses, sample rate/channels, semantic tick window и synchronized video track. Gameplay acoustic facts проверяются отдельно exact/tolerance assertions; человек оценивает только audible presentation quality.

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
- Audio capture sink/device/encoder failure → gameplay/replay остаётся valid, review status `AwaitingCapability`; automatic acoustic assertion нельзя заменить прослушиванием.
- PCM/media hash или synchronization mismatch → capture bundle rejected; prior evidence remains.

## Verification gates

| Gate | Сценарий | Threshold | Evidence | Fallback |
|---|---|---|---|---|
| NAV-P1 | Recast deterministic tiled cook Win/Linux | byte-identical neutral nav tiles и query results для 1 000 start/goal pairs | hashes/query report | engine-owned graph nav for slice |
| NAV-P2 | door/off-mesh/push/stuck/stream tests | 100% stale paths rejected; ≥99% bounded fixtures reach or return correct no-path; no teleport | replay + traversal video | replan/graph adapter |
| NAV-P3 | ADR-016 deterministic 100-NPC workload | весь due navigation work/queue handling помещается в exclusive row p95 ≤1 250 us / p99 ≤1 500 us; membership/phase/due trace exact; bounded deterministic deferral, no starvation/drop/unowned span | GameplayBudgetMatrix/workload hashes, query/due/queue/starvation spans | lower deterministic navigation cadence/LOD; integrated PERF-01 remains blocking |
| AUDIO-P1 | baseline + Steam candidate matrix | baseline always plays; optional adapter acoustic reference error within scenario tolerance, device-loss recovery ≤5 s; 0 gameplay hash differences | audio captures/metrics/replay | baseline attenuation/panning/zones |
| AUDIO-L1 | Steam Audio license/redistribution review | written approval for exact version/artifacts/platform distribution | legal record/SBOM | do not ship adapter |
| WORLD-01 | calendar/weather/reservation save/load/replay and legacy RPG-calendar migration across 100 deterministic fixtures | exact `WorldCalendarStateV1`, accepted-command and DomainEvent sequences/hashes before save, after atomic copy-on-write migration/load and during replay; migrated RPG calendar fields absent; every injected failure preserves original bytes; wall clock changes 0 outcomes | before/after save/replay manifests, migration matrix, command/event traces and state roots | fail closed, retain prior save generation and block world-service conformance |
| WORLD-02 | navigation/world-service/physical ownership and traversal corpus | 100% RoutePlan revisions and reservation decisions remain World Services-owned advice/state; physical/capsule controller alone owns traversal pose/outcome; stale plans reject; route/physical outcomes are replay-stable across 100 fixtures | ownership graph, route/reservation revisions, physical outcome trace and replay roots | reject stale plan, deterministic replan/idle fallback; block on owner or replay divergence |
| AUDIO-02 | displayless audio evidence, base/candidate sync | canonical PCM/WAV roots exact on pinned worker; event-to-sample alignment within 1 sample; automatic acoustic facts exact; review MP4 contains declared audible track; 0 gameplay hash difference | audio/event/frame hashes, waveform/metrics, media manifest | retain PCM/WAV; review AwaitingCapability until MediaEncoder passes |
