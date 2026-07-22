# SPEC-08: Audio, navigation и world services

| Поле | Значение |
|---|---|
| ID | SPEC-08 |
| Статус | Accepted |
| Версия | 1.1.1 |
| Владелец | World Services Team |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-05](05-physics-animation-and-motor-control.md) |
| Связанные документы | [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md) |
| Заменяет | отсутствует |

## Source of truth и ownership

World Services Team владеет navigation representation/query service, route reservations, simulation calendar/weather state, spatial indices и audio scene descriptions. Navigation plan не владеет фактическим character pose: active traversal outcome принадлежит physical/capsule controller. Audio mixer/device state — presentation only; gameplay hearing использует deterministic acoustic facts, а не звуковую карту устройства.

## Public boundary

Public contracts ограничены `NavigationQuery`, `RoutePlan`, traversal link values, deterministic acoustic facts, calendar/weather/reservation/region snapshots и typed errors. Recast/Detour/Steam Audio/device handles, raw nav poly references и mixer buffers являются adapter internals. WorldCommand остаётся единственным mutable RPG/world входом.

## Navigation intent и physical traversal

Navigation разделена на три contracts:

1. `NavigationQuery` преобразует start/goal/capability profile/world revision в versioned `RoutePlan` с corridor, traversal links, costs и validity token.
2. Tactical layer преобразует ближайший corridor/link в `PhysicalAvatarIntent`.
3. Physical embodiment доказывает фактическое достижение link/waypoint по pose/contact outcomes.

RoutePlan является советом, не teleports и не source of truth. Stuck, push, closed door, moving obstacle или failed jump инвалидируют/перепланируют corridor. Off-mesh link хранит generic action tag и geometric constraints; actual interaction проходит WorldCommand/physical gate.

Recast/Detour — `Proposed` adapter. Engine владеет `NavSurface`, tiled nav data, query/filter, RoutePlan и diagnostics. `dt*` types и raw poly refs запрещены в saves, AI, scripting и bundles; stable nav polygon identity состоит из cooked tile AssetId + local engine index + revision.

## Navigation cooking/streaming

Navmesh строится детерминированно из validated collision geometry и traversal profiles. Build parameters и tool hash входят в ContentHash. Tiles активируются/выгружаются согласованно с WorldChunk. Cross-tile link не становится доступным, пока обе required revisions не validated. Dynamic obstacle overlay — runtime-owned rebuildable state; authoritative door/platform state приходит из RPG/physics snapshots.

## World services

| Service | Authoritative state | Contract/failure |
|---|---|---|
| Calendar/time | integer simulation ticks + calendar mapping в RPG save | renderer/audio получают projection; wall clock не влияет |
| Weather | generic weather state machine + seed/transition tick | presentation samples state; missing effect asset uses fallback |
| Spatial query | runtime index rebuilt from owned transforms/chunks | stale revision rejected; не заменяет physics exact query |
| Reservation | PersistentId resource/agent, interval, priority, expiry | accepted только WorldCommand; deterministic conflict order |
| Trigger/region | cooked volumes + overlap state | physical/capsule pose generates candidate; RPG validates outcome |
| Acoustics facts | source, listener, loudness class, occlusion zone, tick | deterministic gameplay perception; independent from output device |

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
| NAV-P3 | 100 agents, 10 Hz planning | query p95 ≤2 ms/agent amortized, total p95 ≤8 ms reference CPU using budgets/cache | profile | lower planning LOD/frequency |
| AUDIO-P1 | baseline + Steam candidate matrix | baseline always plays; optional adapter acoustic reference error within scenario tolerance, device-loss recovery ≤5 s; 0 gameplay hash differences | audio captures/metrics/replay | baseline attenuation/panning/zones |
| AUDIO-L1 | Steam Audio license/redistribution review | written approval for exact version/artifacts/platform distribution | legal record/SBOM | do not ship adapter |
| WORLD-01 | calendar/weather/reservation save/replay | exact state/event hashes across 100 fixtures | replay report | release block |
| AUDIO-02 | displayless audio evidence, base/candidate sync | canonical PCM/WAV roots exact on pinned worker; event-to-sample alignment within 1 sample; automatic acoustic facts exact; review MP4 contains declared audible track; 0 gameplay hash difference | audio/event/frame hashes, waveform/metrics, media manifest | retain PCM/WAV; review AwaitingCapability until MediaEncoder passes |
