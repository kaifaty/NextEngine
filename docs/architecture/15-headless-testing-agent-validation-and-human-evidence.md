# SPEC-15: Local testing, headless scenarios и debugging

| Поле | Значение |
|---|---|
| ID | SPEC-15 |
| Статус | Accepted |
| Версия | 2.4 |
| Последняя проверка | 2026-08-09 |
| Нормативные зависимости | [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-04](04-rendering-and-platform.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md) |
| Заменяет | SPEC-15 2.3; records future deterministic Strategic Agent scenario requirements |

## Назначение

SPEC-15 задаёт простой local-first способ воспроизводить product behavior,
искать первую причину failure и передавать другому разработчику минимальный
replay. Сценарии используются человеком, coding agent и локальной автоматикой
одинаково.

Testability является свойством production architecture:

- actions проходят production input и `WorldCommand` paths;
- probes читают documented immutable projections;
- save/replay использует production persistence;
- fault injection заменяет declared adapter в composition root;
- test-only mutable ECS/backend access запрещён.

Testing tools не принимают продуктовые решения и не изменяют authoritative
world иначе чем через те же команды, которые доступны игре.

## Scenario contract

`TestScenarioManifest` — immutable versioned описание bounded run. Оно содержит:

- scenario ID/version/hash и краткую цель;
- exact project/content/schema/package/build hashes;
- initial neutral fixture или supported `SaveManifest`;
- named RNG streams/seeds и fixed gameplay/physics/motor rates;
- ordered `ScenarioAction`;
- `ProbeSpec` и `AssertionSpec`;
- optional named fault adapters;
- tick, memory, output-size и process-time safety limits;
- optional `CapturePlan`.

Manifest validation завершается до создания mutable world. Unknown schema,
missing reference, unsupported feature или exceeded static bound отклоняет весь
scenario.

## Actions

`ScenarioAction` поддерживает только production-shaped operations:

- engine-owned semantic control, назначенный на declared simulation tick;
- canonical command proposal с normal issuer/capabilities/preconditions;
- lifecycle action `start`, `save`, `load`, `restart`, `stop`;
- activation/deactivation заранее объявленного adapter/process fault;
- presentation-only capture marker.

Action не содержит raw ECS ID, vendor handle, mutable pointer, arbitrary script
внутри host process или wall-clock sleep. Для durable references используются
`PersistentId` и `AssetId`.

## Probes и assertions

`ProbeSpec` выбирает documented read-only projection:

- authoritative state root или bounded domain value;
- ordered `DomainEvent`, command receipt или stable diagnostic;
- save/replay/schema/content hash;
- normalized physics, motor, resource, render или audio metric.

Probe не влияет на schedule, RNG, cache/resource selection или state hash.

`AssertionSpec` использует один явный oracle:

| Oracle | Contract |
|---|---|
| `Exact` | canonical value, order, ID или hash должны совпасть |
| `Tolerance` | units, reference value и absolute/relative bound указаны явно |
| `Distribution` | fixed seeds, sample count, statistic и threshold указаны явно |
| `Invariant` | required/forbidden property проверяется на объявленном tick interval |
| `PresentationMetric` | image/audio/UI metric проверяет только presentation output |

Implicit epsilon, unordered text-log comparison, hidden environment default и
retry-to-green запрещены. Presentation assertions не получают gameplay
authority.

## Deterministic execution

`game` с null presentation, `headless` и optional `capture-worker` используют
одни command validation, schema/content registry, persistence, RNG и system
order. На одном declared target/profile deterministic scenario MUST выдавать
одинаковые command/event/final-state roots.

Async I/O, model inference, shader compilation и worker jobs возвращают
revision-bound result в staging queue. Их wall-time completion order не может
выбирать authoritative outcome. Повторный run, который расходится при тех же
inputs, возвращает `NONDETERMINISTIC_RESULT`; новый повтор используется только
для диагностики.

Headless run не требует renderer, GPU, monitor, input device или display
server. Попытка headless composition открыть interactive window/display
является boundary failure.

## Diagnostics

Каждый failed scenario возвращает machine-readable diagnostic минимум с:

- stable code и subsystem;
- scenario/action ID;
- first divergent tick, event, schema или lifecycle edge;
- typed expected/actual values либо rejection reason;
- causal command/event IDs, если они существуют;
- relevant project/content/schema/build hashes;
- ссылкой на original replay и minimization status.

Rendered text является projection stable fields. Локализованный текст,
unordered logs и stack address не используются как identity failure.

## Replay minimization

`scenario minimize` ищет наименьший action/fault prefix или subset, который
сохраняет:

- exact failure code;
- first-cause category;
- project/content/schema/build hashes;
- required initial checkpoint и RNG state.

Minimizer не редактирует world state, не ослабляет assertion и не заменяет
failure другим. Если minimization не завершается либо не может сохранить
failure identity, original replay остаётся основным reproducer, а причина
minimization записывается отдельно.

## Optional capture

Capture — необязательная помощь при visual, UI, camera, animation, physics,
motor, VFX или audio debugging. Она не нужна для non-visual ProductCheck и не
является gameplay oracle.

`CapturePlan` задаёт replay, semantic start/end markers, camera/view, resolution,
FPS, color/audio format, overlays и output quota. Displayless capture SHOULD
использовать `OffscreenPresentationTarget` без window, surface, monitor, input
enumeration или display server. Capture timing и encoding не входят в
simulation hash.

Worker сначала воспроизводит exact replay и сверяет gameplay root, затем
создаёт bounded PNG/WAV/GIF/MP4 output в локальном artifact directory. Device
loss, encoder absence или quota overflow оставляет replay/diagnostics
доступными и удаляет incomplete media file. Media MAY иметь SHA-256 для
удобного сравнения, но остаётся disposable local debug output.

## Current command surface

Current supported entry points are `cargo run -p xtask -- <ProductCheck>` and
focused crate tests. A public `next` scenario/capture CLI, GUI projection or MCP
protocol is not a current contract. Tool output хранится только в явном local
output directory, ignored by source control by default. Удаление локальных
debug artifacts не меняет source, package или product status.

## Fixture hygiene

Обычные scenarios используют engine-owned/CC0 neutral fixtures. Они не зависят
от Gothic installation, imported bytes или machine-local paths.

Importer smoke MAY использовать user-provided installation только во временном
isolated root. После smoke ordinary scenario получает лишь bounded neutral
validation metadata; source/imported bytes не копируются в repository, package
или reusable fixture. Secret/protected-data detection останавливает run и
помещает temporary output в quarantine.

## Связь с ProductCheck

| ProductCheck | Использование scenario tools |
|---|---|
| `fast` | focused unit/property/negative scenarios и schema validation |
| `play` | один bounded neutral gameplay scenario через production paths |
| `persistence-replay` | save/load/restart, exact replay и corrupt-input cases |
| `content-package` | neutral validate/cook/load/package и sandbox failures |
| `platform` | тот же релевантный scenario на затронутом target |
| `performance` | тот же scenario с declared numeric sampling method |

Coding agent MAY создавать или запускать scenario, читать diagnostics и
предлагать fix. Он не получает дополнительных runtime permissions и не может
подменить failed assertion текстовым объяснением.

## Failure semantics

| Failure | Behavior |
|---|---|
| Invalid scenario/action/probe schema | reject before world creation |
| Deterministic repeat mismatch | return `NONDETERMINISTIC_RESULT`; retain both roots |
| Fault adapter escapes declared boundary | abort run; authoritative checkpoint unchanged |
| Probe attempts mutable/private access | reject probe |
| Minimization fails | retain original replay and diagnostic |
| Headless attempts interactive host access | fail boundary check |
| Capture device/encoder unavailable | keep replay and diagnostics; skip optional media |
| Capture gameplay root mismatch | discard media and report divergence |
| Output quota exceeded | stop writer and remove incomplete local file |
| Protected data detected | stop run and quarantine temporary output |

## Future scenarios

SPEC-31 narrative/divine work is Proposed and has no current scenario schema or
ProductCheck. A production consumer must first define the smallest observable
vertical through ordinary actions, immutable probes and replay.

SPEC-32 deterministic Strategic Agent work is likewise Proposed until R4c/R4d
consumers exist. Its future scenarios must use production perception, memory,
`AgentIntent` and `WorldCommand` paths and immutable probes. Required coverage
includes hidden-fact isolation, bounded GOAP failure, emergency resume/replan,
NPC-to-NPC behavior without `ai-host`, owner-segment save/replay and tiered
100-NPC no-fabrication behavior. Learned-policy training remains an optional R8
lane and is not a prerequisite for these deterministic scenarios.
