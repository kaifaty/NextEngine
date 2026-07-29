# Next Engine: долгосрочный roadmap реализации

| Поле | Значение |
|---|---|
| Статус | Living planning document, не нормативная архитектура |
| Последнее обновление | 2026-07-30 |
| Текущая точка | локально завершённый bootstrap M0–M11, native gate harness и Windows Desktop B0 hardening; следующий Windows package — minimal render content, а Linux-only действия накапливаются в отдельном asynchronous validation backlog |
| Горизонт | developer preview → playable alpha → systemic alpha → creator beta → v1 → post-v1 |
| Источники | Accepted SPEC/ADR, текущий workspace и локальные ProductCheck |

## Назначение

Этот документ отвечает на три практических вопроса:

1. что реализовывать следующим;
2. в каком порядке расширять текущий узкий vertical slice;
3. какое наблюдаемое доказательство завершает каждый этап.

Roadmap не заменяет [продуктовый контракт](architecture/00-product-contract.md),
[архитектурную baseline](architecture/README.md) или профильный SPEC/ADR.
При конфликте действует порядок precedence из архитектурной baseline.
Семантическое изменение Accepted architecture по-прежнему требует отдельного
ADR; изменение приоритета или порядка реализации — нет.

Roadmap намеренно не содержит календарных обещаний. Без известного размера
команды, доступного content-production capacity и shipping hardware точные даты
создали бы ложную точность. Этапы упорядочены по продуктовым зависимостям и
имеют относительный размер. Календарный план следует строить из этого документа
после назначения владельцев и измерения скорости первых двух этапов.

## Как читать статус

| Статус | Значение |
|---|---|
| `DONE_LOCAL` | Реализация и релевантные checks проходят на developer host; это не shipping claim. |
| `NEXT` | Следующий этап критического пути. |
| `PLANNED` | Нужен для v1, но зависит от более ранних этапов. |
| `PARALLEL` | Может развиваться параллельно, но имеет указанную integration gate. |
| `DEFERRED` | Не блокирует v1. |
| `PROPOSED` | Технология или contract ещё не входит в Accepted shipped baseline. |

Этап считается завершённым только когда одновременно выполнены все условия:

- behavior проходит production path, а не test-only shortcut;
- есть focused positive, failure и restart/save coverage;
- релевантные `fast`, `play`, `persistence-replay`, `content-package`,
  `platform` или `performance` имеют честный результат;
- schema, migration, examples, diagnostics и документация обновлены вместе;
- ни один `NOT_RUN` не интерпретирован как `PASS`;
- fallback проверен как реальное поведение, если основная возможность optional.

Наличие public type, Accepted SPEC или компилируемого adapter само по себе не
закрывает этап.

## Текущая точка

[M0–M11 implementation record](nextengine-v1-vertical-slice-implementation-plan.md)
фиксирует рабочий walking skeleton. На 2026-07-29 локально присутствуют:

- canonical commands, command ledger, fixed-stage runtime и atomic RPG
  transactions;
- exact project resolution/cook/activation и content-addressed publication;
- общий production coordinator для `game`, `headless` и runtime-bearing tools;
- atomic save generations, replay, application-session close/recovery;
- движение grounded capsule, статические Box colliders и contact lifecycle;
- pickup/equipment, melee damage, switch, dialogue/quest/relationship transition;
- deterministic two-chunk streaming;
- deterministic NPC affordance planner и procedural avatar projection;
- data-only mechanics, bounded Luau и Wasm Component paths;
- immutable presentation extraction и reference B0 render plan;
- Windows SDL3/ash B0 path с canonical keyboard/lifecycle events, fullscreen,
  swapchain recreation, bounded device-loss recovery и clean package smoke;
- локальная v1 closure matrix.

Это сильный bootstrap, но ещё не пользовательская alpha. Текущий сценарий
жёстко ограничен одним neutral fixture, двумя chunks, одной combat ability,
одним NPC transition и B0-примитивами.

### Реализация по подсистемам

| Область | Состояние в коде | Главный gap |
|---|---|---|
| Contracts/runtime/ledger | Реализован фундамент | Расширять только вместе с реальным gameplay use case; не строить второй runtime framework. |
| Project/application/session | Реализован production path | Нужна native platform/package closure и дальнейшие real-project lifecycle cases. |
| Persistence/replay | Реализован текущий owner set | Нет реального schema migration graph и будущих owner segments. |
| RPG | Частично: основные aggregates и восемь операций | Нет полного faction/membership, status/effect, quest-graph, reward и divine command lifecycle. |
| Mechanics/packages | Частично: contact melee + Luau/Wasm examples | Нет общего ability phase/cost/cooldown/status lifecycle и creator-facing SDK workflow. |
| Content/cooker | Частично: generic neutral records и reference fixture | Нет production-complete mesh/material/texture/skeleton/animation/audio/navigation catalog и migrations. |
| World/streaming | Частично: deterministic two-chunk transition | Нет general partition interest, resource residency, calendar, population, schedules и region transfers. |
| Jobs/resources | Spec-only | Нет общего bounded job, memory, I/O credit, pin/lease и backpressure substrate. |
| Physics | Частично: upright capsule + static Box | Нет полного shape/body/constraint/query profile и production physical-character stack. |
| Animation/motor | Только procedural projection/contract fragments | Нет skeleton graph, retargeting, IK, root-motion intent или deterministic inference supervisor. |
| Agent AI | Частично: один canonical affordance planner | Нет perception, hierarchy, schedules, memory и 100-NPC workload. |
| Navigation/audio | Spec-only | Нет runtime service, cooker или baseline adapters. |
| Player experience | Частично: normalized input contracts + Windows SDL keyboard/lifecycle path | Нет ActionMap/context service, camera/targeting, semantic UI, localization и accessibility implementation. |
| Presentation/render | Частично: snapshot + B0 primitives + Windows swapchain/device recovery | Нет production material/shader/content path, VFX consumption state, paired same-commit target proof и representative Linux hardware-GPU evidence. |
| Tooling | Частично: repository `xtask` checks | Нет creator-facing `next` CLI, inspectors, scenario/minimizer и stable external SDK workflow. |
| Autonomous narrative | Contract fragments only | SPEC-31 runtime, graph admission, director fallback и divine batch transaction отсутствуют. |

## Продуктовая граница v1

V1 следует оценивать по [SPEC-00](architecture/00-product-contract.md), а не по
количеству полностью реализованных архитектурных документов. Accepted SPEC
задаёт правильную границу и failure semantics, когда соответствующая
возможность реализуется; он не всегда делает всю описанную глубину блокером
первого релиза.

Для v1 обязательны:

- один устанавливаемый cooked project на Windows x86_64 и Linux x86_64;
- связанный movement → interaction → combat → dialogue → quest loop;
- `game`, deterministic `headless` и creator/runtime tools;
- save/load/replay и offline correctness;
- streamed world, generic RPG state и first-party mechanics через public
  package API;
- bounded data-only, Luau и Wasm extension paths;
- engine-owned physics, motor и animation boundaries с процедурным fallback;
- usable player input, camera, semantic UI и минимально доступный presentation
  path;
- neutral content cook/validate/package workflow.

Не являются v1 blocker:

- LLM, ASR, TTS, external `ai-host` и SPEC-16 multimodal dialogue;
- autonomous Narrative Director и divine agency из SPEC-31;
- learned motor policy, Isaac Lab, PhysX или другой vendor physics backend;
- ray tracing, mesh shaders, HDR, advanced VFX и displayless capture;
- Gothic importer;
- full editor, multiplayer, consoles, mobile или macOS shipping.

Рекомендуемый v1 physical scope: устойчивый capsule/procedural controller,
skeletal presentation, retargeting и basic IK. Full articulation и learned
motor policy входят в v1 только если их собственный product check проходит
достаточно рано; fallback остаётся shipping-capable без них.

## Критический путь

```mermaid
flowchart LR
    R0["R0 Bootstrap<br/>DONE_LOCAL"] --> R1["R1 Native developer preview<br/>IN_PROGRESS"]
    R1 --> R2["R2 Playable alpha"]
    R2 --> R3["R3 Scalable content and streaming"]
    R3 --> R4["R4 Systemic living world"]
    R4 --> R5["R5 Physical character integration"]
    R5 --> R6["R6 Creator beta"]
    R6 --> R7["R7 V1 release candidate"]
    R7 --> R8["R8 Post-v1 AI and advanced tracks"]
    R2 -. "parallel R&D; integrate only at R5 gate" .-> P["Physical/animation experiments"]
    P -.-> R5
    R1 -. "never blocks mandatory offline path" .-> O["Optional adapters and ai-host"]
    O -.-> R8
```

| Этап | Статус | Размер | Product outcome |
|---|---|---:|---|
| R0. Walking skeleton | `DONE_LOCAL` | — | Узкий deterministic slice доказал end-to-end architecture. |
| R1. Native developer preview | `IN_PROGRESS` | S–M | Один exact package действительно запускается на обеих shipping targets. |
| R2. Playable alpha | `PLANNED` | L | В slice можно играть через нормальный camera/UI/presentation loop. |
| R3. Scalable content and streaming | `PLANNED` | XL | Движок перестаёт зависеть от hard-coded two-chunk fixture. |
| R4. Systemic living world | `PLANNED` | XL | NPC, schedules, navigation и RPG consequences образуют живой offline world. |
| R5. Physical character integration | `PARALLEL` → `PLANNED` | XL | Physical, animation и motor layers становятся production gameplay path. |
| R6. Creator beta | `PLANNED` | L–XL | Второй проект/пакет создаётся без правки engine internals. |
| R7. V1 release candidate | `PLANNED` | L | Полный v1 scope стабилизирован и упакован для Windows/Linux. |
| R8. Post-v1 tracks | `DEFERRED` | отдельные программы | Optional AI/narrative/importer/advanced rendering не размывают v1. |

## R0 — Walking skeleton

**Статус:** `DONE_LOCAL`.

**Цель:** доказать, что архитектурные границы соединяются в один production
path.

**Доказанный результат:** M0–M11, локальные `play`, `persistence-replay`,
`content-package`, portable `platform`, `performance` и `v1-closure`.

**Открытый риск:** этот этап доказан на Apple Silicon developer host.
Windows/Linux execution не считается выполненным и перенесён в R1.

**Правило сохранения:** каждый следующий этап расширяет текущий slice
вертикально. Переписывание contracts/runtime без нового player-facing outcome
не является roadmap progress.

## R1 — Native Windows/Linux developer preview

**Статус:** `IN_PROGRESS`. Native gate harness и Windows Desktop B0 hardening
реализованы; Windows `platform` и clean `v1-package` smoke проходят локально.
Native Linux target report и package имеют отдельный `PASS`; paired
same-commit Windows report и compare ещё не выполнены. Exact checkpoint,
environment и coordination status ведутся в
[Linux validation backlog](development/linux-validation-backlog.md).

**Текущая execution policy:** native Windows x86_64/MSVC/Vulkan — основной
developer host и приоритет текущих work packages. Linux validation выполняется
отдельными асинхронными checkpoint-сессиями; standalone Linux `PASS` не
закрывает R1/B-01 без matching Windows target `PASS` report и успешного compare.
Отложенные Linux-only действия ведутся в
[Linux validation backlog](development/linux-validation-backlog.md) и
выполняются асинхронными checkpoint-сессиями. Их ожидание не блокирует
несвязанные Windows work packages, но соответствующие R1/v1 criteria до
фактического run остаются открытыми.

**Цель:** превратить portable local closure в честно запускаемый native
developer package.

**Основной scope:**

- стабилизировать private SDL3/ash B0 adapter на Windows x86_64 и Linux x86_64;
- закрыть window/input/focus/resize/fullscreen/surface/device-loss lifecycle;
- проверить platform normalization без влияния native timestamps на simulation;
- собрать atomic distribution с `game`, `headless`, exact cooked project и
  notices;
- запустить release `headless` и bounded interactive release `game` из
  созданного package;
- сравнить project/content/state/ledger roots между target reports.

**Критерии успеха:**

- на обеих targets проходят `host-check`, `play`, `persistence-replay`,
  `content-package`, `platform`, `performance` и `v1-closure`;
- каждый `v1-closure` сообщает `PASS` для native target текущего host и честный
  `NOT_RUN` для другого target; успешный same-commit `native-gate-compare`
  выставляет `native_gate_ready = true`;
- `v1-package` создаёт installable directory и оба release binaries запускаются
  против exact packaged lock;
- B0 scene обрабатывает real input и типовые lifecycle transitions без
  authoritative divergence;
- package не содержит machine-local paths, protected data, credentials или
  отсутствующие notices.

**Hard blockers:**

- paired native Windows/Linux `PASS` reports и package на одном exact commit;
- representative hardware-GPU Linux smoke; текущий checkpoint и pending
  actions записаны в
  [Linux validation backlog](development/linux-validation-backlog.md);
- SDL3/ash candidate должен пройти target smoke; cross-compilation недостаточно;
- любой cross-target canonical/hash mismatch;
- packaging/runtime dependency, которая не включена или не диагностируется.

**Не блокируют этап:** PhysX, Slang, RT, learned policy, `ai-host`, capture и
Gothic importer.

**Основные источники:** SPEC-04, SPEC-12, SPEC-17, SPEC-29, SPEC-30, ADR-003,
ADR-028, ADR-030.

## R2 — Playable product alpha

**Цель:** превратить технический fixture в небольшой player-facing slice,
который можно пройти без знания engine internals.

**Основной scope:**

- runtime ActionMap/InputContext service поверх существующего canonical input;
- third-person camera, interaction focus и authoritative targeting query;
- semantic HUD, inventory/equipment, dialogue, quest journal, pause/save/load
  flow;
- source locale, deterministic text IDs, pseudo-locale и readable fallback;
- keyboard/mouse и минимум один controller profile через одинаковые action IDs;
- production mesh/material/texture path для B0 renderer;
- baseline sample playback, attenuation/panning, voice limiting и subtitle
  fallback;
- один CC0/engine-owned 20–30 minute project slice с началом, конфликтом и
  завершением.

**Критерии успеха:**

- новый пользователь может запустить package, пройти movement → pickup/equip →
  combat → dialogue → quest → save/load loop без debug commands;
- те же semantic actions проходят через `game` и headless scenario и дают
  одинаковый authoritative result;
- UI/camera/audio/presentation faults не меняют gameplay hash;
- save/load работает до и после каждого крупного шага slice;
- missing device/locale/audio output использует declared fallback;
- `play`, `persistence-replay`, `content-package` и native `platform` проходят
  для alpha project.

**Hard blockers:**

- R1 native package closure;
- выбранный минимальный neutral render-content profile;
- CC0/engine-owned art, UI text и audio fixture с подтверждённой provenance;
- отсутствие hidden direct-mutation path из UI/camera.

Эти blockers ограничивают закрытие R2 и соответствующий alpha claim, но не
начало или продолжение перечисленных Windows work packages.

**Scope guard:** editor, advanced renderer, photoreal assets и procedural world
generation не входят в этот этап.

**Основные источники:** SPEC-04, SPEC-08, SPEC-12, SPEC-18, SPEC-24, SPEC-29,
SPEC-30, ADR-019.

## R3 — Scalable content, jobs and streaming

**Цель:** заменить fixture-specific load/cook behavior на bounded substrate,
пригодный для реального multi-region project.

**Основной scope:**

- закрытый job classification, immutable requests/results, canonical merge,
  cancellation tree и finite queues;
- logical memory/resource budgets, pins, leases, deterministic eviction,
  decompression and I/O credits;
- production `ContentManifest`/bundle container и neutral schemas для scene,
  mesh, material, texture, collision, skeleton, animation, audio, navigation и
  world chunk в реально используемом минимальном профиле;
- schema compatibility registry и первый настоящий copy-on-write migration;
- general world partition interests, dependency groups, placement/tombstones,
  load/unload admission и restart reconstruction;
- cooker cache и atomic publication для multi-region project;
- убрать assumptions, требующие ровно два chunks или reference-only IDs.

**Критерии успеха:**

- representative project с несколькими regions и существенно более чем двумя
  chunks cooks, loads, streams, unloads, saves и restarts через production path;
- worker completion, queue pressure, cancellation и I/O fault permutations
  дают одинаковый committed world root;
- mandatory work не теряется и optional overload использует declared rejection
  или degradation;
- corrupt/oversized/cyclic content и migration inputs fail closed до
  publication;
- первая N−1→N migration сохраняет source generation и публикует только
  complete target generation;
- `content-package`, `persistence-replay`, focused streaming tests и
  `performance` проходят в declared memory/I/O profile.

**Hard blockers:**

- финальный минимальный набор content classes для v1;
- ownership map между Runtime, Assets и World Services без второго mutable
  source;
- измеримый logical resource profile для Windows/Linux;
- migration fixture, представляющий реальную эволюцию schema, а не empty DAG.

**Основные источники:** SPEC-03, SPEC-17, SPEC-21, SPEC-22, SPEC-23, SPEC-24,
SPEC-25, ADR-025, ADR-026.

## R4 — Systemic living world

**Цель:** перейти от scripted encounter к offline world, где NPC и world state
продолжают согласованно жить вне непосредственного контакта с игроком.

**Основной scope:**

- `WorldCalendarState`, stepped/bounded bulk time и World Services save owner;
- population records, `Dormant/Abstract/Simulated/Active` tiers, schedules,
  wake/defer rules и region transfers;
- deterministic graph navigation baseline, tiled cook/streaming, route validity
  и physical traversal handoff;
- perception facts, bounded planner hierarchy, habits and deterministic memory
  baseline;
- faction/membership/relationship operations, reusable dialogue/quest
  conditions and consequences;
- general ability phase/cost/cooldown/effect/status lifecycle through public
  mechanics API;
- integrated 100-NPC budget with deterministic cadence/deferral and no
  starvation;
- deterministic acoustic gameplay facts; hardware audio remains presentation.

**Критерии успеха:**

- multi-region scenario с 100 NPC воспроизводит exact schedule/tier/command/
  event/final roots across repeats, save/restart and worker permutations;
- NPC проходит navigation → interaction/combat → consequence только через
  AgentIntent/WorldCommand/Mechanics paths;
- unsupported abstract outcome upgrades or defers and никогда не фабрикует
  success;
- stepped и bulk world time сходятся на одинаковых boundaries;
- faction, quest, inventory и relationship consequences переживают unload,
  save/load и NPC tier changes;
- due AI/navigation work укладывается в ADR-016 thresholds или применяет
  declared deterministic cadence reduction;
- mandatory gameplay остаётся корректным без network и `ai-host`.

**Hard blockers:**

- R3 job/resource/partition substrate;
- navigation content and query baseline;
- календарный owner segment и migration из любого legacy representation;
- полный owner-safe RPG operation set для выбранного systemic scenario;
- authored NPC schedules, regions and fallback activities.

**Основные источники:** SPEC-06, SPEC-08, SPEC-13, SPEC-19, SPEC-20, SPEC-23,
SPEC-25, ADR-016, ADR-020, ADR-021, ADR-026.

## R5 — Physical character, animation and motor integration

**Статус:** R&D может идти как `PARALLEL` с R2–R4, production integration
закрывается после R4 substrate.

**Цель:** сделать физическое воплощение персонажа частью production gameplay,
не связывая correctness с конкретным vendor backend или model.

**Обязательный v1 scope:**

- расширить reference physics до нужных gameplay shapes, layers, materials,
  bounded queries, sensors и dynamic/kinematic interactions;
- устойчивый capsule locomotion profile: slopes, stairs, push, fall/recovery;
- neutral skeleton/clip/graph schemas, retargeting и basic IK;
- root motion остаётся intent и проходит physical validation;
- deterministic procedural motor/safety/recovery route;
- physical and animation state save/load/replay;
- один humanoid physical archetype, используемый player и NPC через public
  contracts.

**Optional integration scope:**

- ONNX observation/action path и policy supervisor;
- full articulation;
- PhysX 5.9.0 parity adapter.

**Критерии успеха:**

- player/NPC проходят push, slope, stair, trip, carry, fall/recovery и melee
  fixtures без teleport/direct health mutation;
- contact/query/order, committed physical projection and gameplay outcome
  воспроизводятся согласно выбранному deterministic profile;
- skeleton/retarget/IK failures используют declared pose/procedural fallback и
  не повреждают authoritative state;
- 16 representative avatars укладываются в SPEC-05 budget на declared
  reference CPU profile либо используют deterministic physical LOD;
- save/load/replay сохраняет physical/animation owner state;
- learned/vendor path при отсутствии или failure возвращается к тому же
  shipping-capable procedural path.

**Hard blockers:**

- минимальный v1 physical archetype и animation asset profile;
- general collision/query features, которых требует выбранный gameplay;
- cross-target numeric profile и canonical projection;
- licensed/CC0 skeleton, clips and retarget fixtures;
- ясная граница authoritative physical state против presentation pose.

**Не являются hard blocker:** достижение PhysX parity, training hardware,
learned policy quality или full articulation. Если они не проходят вовремя,
v1 остаётся на reference/procedural route.

**Основные источники:** SPEC-05, SPEC-14, SPEC-26, SPEC-27, SPEC-28, ADR-009,
ADR-013, ADR-027, ADR-032, ADR-033.

## R6 — Creator beta and SDK

**Цель:** доказать, что движок расширяется не только его авторами и не только
через Rust source edits.

**Основной scope:**

- stable non-interactive `next` CLI: project resolve/validate/diff, cook,
  content/package/save/plugin validation, replay and package;
- read-only inspectors для RPG, world, mechanics, input/player, AI and physical
  projections;
- scenario validate/run/minimize и stable JSON diagnostics;
- public package templates, versioned data/Luau/Wasm APIs and examples;
- ability, item, NPC, dialogue, quest, world-chunk and physical-archetype
  authoring workflows;
- N/N−1 plugin/package/schema compatibility and migration examples;
- deterministic cooker cache, actionable source spans and basic license/SBOM
  output;
- второй small project или expansion package, созданный только через public
  contracts.

**Критерии успеха:**

- clean checkout + documented toolchain создаёт, cooks, validates, runs and
  packages второй project без изменения engine crates;
- first-party и community-style mechanics используют одинаковый SDK,
  capabilities and validators;
- malformed assets, sandbox escapes, quota overflow and incompatible package
  versions fail with stable diagnostics and no partial publication;
- replay inspector находит first divergent tick/subsystem;
- cold authoring exercise может добавить одну ability, один quest/NPC и один
  streamed chunk через documented public workflow;
- `fast`, `play`, `persistence-replay` and `content-package` включают второй
  project/package как regression fixture.

**Hard blockers:**

- стабилизированный minimal schema surface из R3–R5;
- отсутствие hidden first-party bootstrap APIs;
- понятная distribution/license policy для packages;
- budget на документацию, examples and diagnostics, а не только runtime code.

**Scope guard:** full graphical editor не нужен; CLI/JSON + schemas должны быть
достаточны для v1.

**Основные источники:** SPEC-07, SPEC-09, SPEC-11, SPEC-13, SPEC-15, SPEC-22,
SPEC-24, ADR-008, ADR-014, ADR-025.

## R7 — V1 release candidate and release

**Цель:** стабилизировать выбранный product scope, а не добавлять новые
архитектурные подсистемы.

**Основной scope:**

- feature/content freeze и закрытие только release-blocking defects;
- final representative cooked project и clean-install packages;
- save/schema compatibility matrix и upgrade/rollback documentation;
- Windows/Linux performance, input, renderer, audio and lifecycle hardening;
- accessibility baseline, source locale and fallback behavior;
- license notices, provenance, dependency inventory and protected-data scan;
- user/developer getting-started, package authoring and troubleshooting docs;
- crash/recovery, long-session and corrupted-input suites;
- release versioning and reproducible package manifest.

**Критерии успеха:**

- все 16 MUST из SPEC-00 покрыты observable product behavior;
- final exact commit проходит релевантные checks на native Windows x86_64 и
  Linux x86_64;
- fresh packages запускают `game`, `headless` and tools, activate exact cooked
  lock, save/load and complete the representative loop;
- no open release-blocking data-loss, security, deterministic divergence,
  install/launch or offline-correctness defect;
- target performance profiles соблюдены либо качество снижено только через
  declared non-authoritative/LOD fallback;
- package содержит required notices and no forbidden artifacts;
- все `NOT_RUN` перечислены как non-claims; shipping-required target check не
  может оставаться `NOT_RUN`.

**Hard blockers:**

- незакрытый R1 target/package gate;
- незавершённый representative content slice;
- incompatible save without migration/export path;
- P0/P1 data-loss, security, determinism, offline or installation defect;
- неизвестная provenance любого distributed artifact.

**Основные источники:** SPEC-00, SPEC-11, SPEC-12, SPEC-15, SPEC-17, SPEC-29,
ADR-001, ADR-030.

## R8 — Post-v1 programs

Эти направления получают отдельные implementation plans после v1 либо раньше
как изолированные experiments без влияния на mandatory gameplay:

| Track | Architecture status | Entry condition |
|---|---|---|
| Autonomous quest lifecycle, Narrative Director and divine agency | SPEC-31/ADR-029/ADR-031 Accepted, runtime отсутствует | R4 world/RPG boundaries и R6 scenario/tooling готовы; template fallback реализуется первым. |
| Text-canonical multimodal dialogue/model packs | SPEC-16/ADR-017 `Proposed` | Явное решение о promotion, privacy/budget policy и text-only fallback. |
| External `ai-host` | Optional | Stable bounded process protocol, recorded-input replay and complete in-process fallback. |
| Learned/full-articulation motor | Optional technology | R5 reference/procedural baseline, exact observation/action checks and runtime/training parity. |
| PhysX production backend | ADR-033 technology remains `Proposed` | Полная parity на Windows/Linux; reference backend остаётся oracle/fallback. |
| Advanced renderer/HDR/RT/VFX/capture | Optional | B0 v1 path стабилен; feature has bounded fallback and target-specific product check. |
| Gothic importer | Optional separate repository/process | Neutral schemas стабильны, legal/provenance boundary проверен; parent repo остаётся независимым. |
| Full editor | Outside v1 | Creator beta CLI/JSON workflows показали реальные high-friction authoring operations. |

Принятие SPEC-31 не означает, что он должен задерживать v1. Его реализация
зависит почти от всех systemic boundaries R3–R6 и поэтому раньше будет создавать
parallel authority или fixture-only code.

## Сквозные workstreams

Некоторые работы идут постоянно и не должны превращаться в отдельный
инфраструктурный этап.

### Product slice

Каждый этап должен оставлять более богатый запускаемый project. Нельзя
реализовать world services, animation или tools только на synthetic unit
fixtures и отложить integration до R7.

### Determinism and persistence

Любой новый authoritative owner сразу получает:

- command/transaction path;
- owner segment and canonical state root;
- save/load/restart and corrupt-input coverage;
- replay compare point;
- game/headless parity.

### Security and content provenance

Любой новый parser, package surface, model or external process считается
untrusted с первого changeset. Bounds, hashes, capabilities, quotas, redaction
and notices добавляются вместе с happy path.

### Platform

Windows checks запускаются вместе с каждым work package, который меняет
platform, renderer, physics, packaging or performance. Требующие native Linux
host действия добавляются в
[отдельный backlog](development/linux-validation-backlog.md) и могут
выполняться асинхронно пакетами на зафиксированных exact-commit checkpoints.
Отсутствие Linux run не блокирует unrelated Windows development, но stage exit
или shipping claim, требующий Linux evidence, остаётся открытым. Накопленные
Linux checks нельзя молча переносить за соответствующий stage/release gate.

### Physical R&D

Physics/animation experiments могут идти параллельно после R1, но не меняют
default route до R5 integration gate. Неуспех vendor/model candidate не
блокирует capsule/procedural product.

## Blocker register

| ID | Blocker | Блокирует закрытие | Условие снятия |
|---|---|---|---|
| B-01 | `OPEN`: standalone Linux `PASS` записан; paired same-commit Windows report и cross-target compare отсутствуют. Coordination хранится в [Linux validation backlog](development/linux-validation-backlog.md). | R1, R7 | На одном exact clean commit собраны Windows/Linux target `PASS` reports и packages с matching roots, а `native-gate-compare` сообщает `native_gate_ready = true`. |
| B-02 | SDL3/ash остаётся `Proposed` target candidate; Windows B0 path проходит локально, software-Vulkan Linux evidence записан, а representative hardware-GPU action поставлен в [Linux validation backlog](development/linux-validation-backlog.md). | R1, R2 | B0 lifecycle/input/device-loss checks проходят на обеих targets либо выбран thin adapter за тем же contract. |
| B-03 | Нет production render/content profile и достаточного CC0 content | R2, R5, R7 | Зафиксирован минимальный mesh/material/texture/skeleton/audio profile и lawful fixture/project. |
| B-04 | Нет общего job/resource/backpressure substrate | R3–R5 | Finite queues, canonical merge, logical budgets, pin/lease/eviction and fault checks реализованы production owners. |
| B-05 | Schema migration DAG фактически пуст | R3, R7 | Реальная N−1→N copy-on-write migration проходит valid/corrupt/fault matrix. |
| B-06 | World ограничен двумя chunks | R3, R4 | General partition interest/admission and multi-region save/restart scenario проходят. |
| B-07 | Нет calendar/population/navigation services | R4 | World owner segment, schedules/tiers and graph navigation baseline проходят systemic scenario. |
| B-08 | Physics ограничена capsule + static Box; animation/motor отсутствуют | R5 | V1 physical profile and procedural animation/motor fallback проходят physical product checks. |
| B-09 | Нет external creator CLI/SDK workflow | R6, R7 | Второй project/package создаётся cleanly только public tools/contracts. |
| B-10 | Content scope может расти быстрее playable loop | Все этапы | Для каждого этапа назначен один representative scenario и явно записан non-goal list. |
| B-11 | Недостаточная content/documentation capacity | R2, R6, R7 | Назначены owners и budget для art/audio/text/examples/docs/provenance. |
| B-12 | Target performance profile не измерен на representative world | R4, R5, R7 | Recorded CPU/GPU/memory/I/O profile проходит declared thresholds или включает bounded fallback. |

## Решения, которые нужно принять вовремя

Эти решения не должны блокировать начало roadmap, но должны быть закрыты до
указанного gate.

| Decision | Deadline | Рекомендация |
|---|---|---|
| Минимальный v1 visual/content profile | до R2 implementation freeze | B0 raster, mesh/material/texture, one humanoid skeleton, source locale; no HDR/RT requirement. |
| UI toolkit/backend | до R2 UI integration | Private replaceable adapter behind semantic UI; не вводить widget types в contracts. |
| Baseline navigation | до R4 | Начать с engine-owned deterministic graph/tile representation; Recast remains replaceable candidate. |
| V1 physical scope | до R5 content production | Capsule/procedural + skeletal/IK mandatory; learned/full articulation optional. |
| Physics backend | до R5 integration | Reference remains default/oracle; promote PhysX only after target parity. |
| Creator surface | до R6 | Stable CLI/JSON first; graphical editor after real creator workflow data. |
| Narrative/LLM priority | после R7 scope freeze | Template/deterministic behavior first; external model only as optional candidate source. |

## Ближайшая implementation queue

Ближайшая очередь является Windows-first. Linux validation не занимает позицию
в implementation queue: требующие native Linux host действия накапливаются в
[Linux validation backlog](development/linux-validation-backlog.md) и
выполняются отдельными checkpoint-сессиями. Это не блокирует начало следующего
Windows increment. Оба same-commit reports и успешный compare остаются
обязательными перед закрытием R1/B-01.

Следующие work packages рекомендуется выполнять в этом порядке:

Завершённый Windows baseline: **Desktop B0 hardening (`DONE_LOCAL_WINDOWS`)**.
SDL keyboard input нормализуется в engine-owned events; resize/focus/minimize/
restore/fullscreen, surface recreation и bounded device-loss recovery проходят
production adapter path. Оставшиеся target claims учитываются отдельно и не
возвращают этот implementation package в активную очередь.

1. **Minimal render content (`NEXT`):** neutral mesh/material/texture records,
   cooker, B0 upload and fallback material.
2. **Player action and camera:** ActionMap/context resolution, third-person
   camera, interaction focus and targeting query.
3. **Semantic UI:** HUD, inventory/equipment, dialogue, quest journal,
   pause/save/load and pseudo-locale.
4. **Baseline audio:** clips, emitters/listener, priority/voice limits,
   attenuation/panning and subtitle fallback.
5. **Playable alpha project:** заменить technical fixture на один complete
   CC0/engine-owned 20–30 minute slice.
6. **Jobs/resources vertical:** сначала content cook/stream use case, затем
   shared bounded admission primitives.
7. **General partition and migration:** multi-region streaming plus first real
   save/content schema migration.
8. **Living-world vertical:** calendar + small population + graph navigation,
     затем масштабирование к integrated 100-NPC scenario.

Каждый package должен быть отдельным product increment с focused checks. Work
package 6 не следует начинать как универсальный scheduler design без package 5
и конкретного streaming workload.

## Обновление roadmap

Roadmap обновляется после завершения этапа или существенного изменения
product scope:

1. обновить current-state table по фактическому коду;
2. приложить ссылки на ProductCheck commands/results без превращения их в
   обязательный evidence dossier;
3. перевести только реально выполненный gate;
4. перенести оставшиеся gaps/blockers в следующий этап;
5. оформить ADR лишь если изменена Accepted semantics;
6. сохранить optional/Proposed tracks как non-blocking, пока их собственная
   promotion/integration gate не пройдена.

Главный показатель прогресса — не число contracts или crates, а то, насколько
более богатый проект можно запустить, пройти, сохранить, воспроизвести,
расширить публичным SDK и упаковать на обе shipping platforms.
