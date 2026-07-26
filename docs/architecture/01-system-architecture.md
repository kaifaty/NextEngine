# SPEC-01: Системная архитектура

| Поле | Значение |
|---|---|
| ID | SPEC-01 |
| Статус | Accepted |
| Версия | 2.2 |
| Последнее изменение | 2026-07-26 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md) |

## Архитектурная форма

Next Engine разделяет durable gameplay contracts, runtime orchestration и
replaceable adapters. Граница существует ради playable offline game,
предсказуемых save/replay и быстрой замены backend, а не ради отдельной
организационной workflow-системы.

Public system boundary находится в `crates/contracts` и содержит только
versioned engine-owned values: nominal IDs, project/content/schema manifests,
`WorldCommand`, `DomainEvent`, immutable queries/snapshots, RPG operations,
world/physics/motor/animation contracts и process/plugin protocols.

ECS components/storage, allocator/task handles, OS/window/device objects,
database connections, importer records, compiler objects и vendor/backend types
остаются private implementation details.

## Composition roots

| Root | Назначение | V1 |
|---|---|---|
| `game` | interactive offline game | required |
| `headless` | тот же authoritative runtime без renderer/window | required |
| `tools` | cooker, validators, inspectors, package/replay utilities | required |
| `capture-worker` | displayless screenshots/video/audio for debugging or marketing | optional |
| `ai-host` | replaceable LLM/ASR/TTS/embedding adapters | optional |
| external importer | отдельный process/repository, выдающий neutral artifacts | optional |

`game` и `headless` используют один project lock, schema/content closure,
command validator, persistence/replay code и system ordering. Presentation
adapter, host capability и session identity могут различаться, но gameplay
outcome — нет.

## Технические источники состояния

| Состояние | Authoritative representation | Изменение |
|---|---|---|
| Project composition | immutable `ProjectCompositionLock` | validated atomic activation before world start |
| Gameplay/RPG | revisioned RPG aggregates | validated `WorldCommand` transaction |
| World calendar/population | World Services state | validated command/world-advance transaction |
| Runtime residency | Core Runtime mapping | deterministic spawn/despawn/streaming commit |
| Physical world | engine-owned physics state and canonical snapshot | fixed step plus validated motor/topology operation |
| Motor route/state | engine-owned policy supervisor state | deterministic route/action commit or procedural fallback |
| Animation state | engine-owned graph/root-motion/IK state | fixed animation stages; root motion is only an intent |
| Agent working state | in-process deterministic agent state | planner commit/proposal path |
| Mechanics packages | exact package lock and Mechanics Runtime state | validated mechanic delta inside command transaction |
| Assets/content | immutable content manifest and bundles | cooker atomic publish |
| Schemas/migrations | immutable schema registry and migration graph | validated registry/copy-on-write publication |
| Save | last complete manifest and segments | atomic persistence transaction |
| Application session | runtime session state and close journal | validated lifecycle transition |
| Presentation | immutable presentation snapshot and reconstructible caches | extraction/consumption only; never simulation write |

Одна строка не может иметь два mutable sources. Backend-side state допустим
только как reconstructible cache.

## Target monorepo layering

```text
crates/contracts          public schemas and nominal IDs
crates/core-runtime       schedule, command bus, ECS facade
crates/resource-runtime   jobs, memory/residency and I/O staging
crates/rpg                generic RPG domain
crates/world              calendar, population and spatial services
crates/physics-api        engine-owned backend interface
crates/physics-*          replaceable backend adapters
crates/physical-archetypes body/skill/policy manifests and supervisor
crates/render-api         presentation contracts
crates/render-*           renderer adapters
crates/agent-runtime      deterministic AI and optional ai-host client
crates/mechanics          abilities, effects, statuses and package state
crates/scripting          Luau capability host
crates/plugin-host        WIT/Wasm capability host
crates/assets             neutral content, streaming and persistence
tools/*                   cooker, validators, inspectors and packaging
apps/game, apps/headless  required composition roots
lab/*                     isolated offline training/experiments
```

Dependencies point from roots and adapters toward engine contracts. Contracts
never depend on a vendor implementation.

## Production data flow

1. Platform adapter emits normalized device-independent controls.
2. Player/AI/script/plugin/world logic produces command proposals, never direct
   mutable references.
3. Runtime validates capability, schema, target tick and preconditions, derives
   canonical command identity and reserves it in the durable ledger.
4. Domain systems stage changes against expected revisions.
5. One transaction publishes all owned deltas and ordered `DomainEvent`, or
   publishes nothing.
6. Async work receives immutable inputs and returns a revision-bound result to a
   deterministic commit point.
7. Physics and motor advance at fixed stages; learned output passes the same
   deterministic safety and fallback path as procedural output.
8. Presentation extracts an immutable snapshot and cannot write back to
   simulation.
9. Save/replay serializes only engine-owned authoritative state and declared
   future-affecting scheduler/session state.

## Scheduling and async boundary

Authoritative mutation occurs only inside declared simulation stages. I/O,
decompression, shader/model compilation and optional process IPC finish in
bounded staging queues. Worker completion order cannot select a winner,
fallback, target tick or partial generation.

No task holds mutable ECS/domain/backend authority through `await` or a frame
boundary. A stale result is rejected by revision/hash preconditions.

## Failure semantics

- Invalid project/schema/content fails before mutable world creation.
- Backend initialization selects an already declared fallback or stops startup.
- `ai-host` absence/timeout uses deterministic in-process behavior.
- Plugin/script trap or deterministic fuel/resource exhaustion discards its
  uncommitted proposals without crashing the host.
- Physics/motor failure preserves the last complete checkpoint and selects only
  a deterministic declared fallback.
- Renderer/device failure drops reconstructible presentation state while
  gameplay continues or suspends explicitly.
- Save/session close failure preserves the prior complete save and uses a
  durable idempotent close journal.
- Tool/importer crash leaves staging unpublished.
- Unsupported optional hardware disables only the affected optional feature.

## Product checks

| Check | ProductCheck | Expected behavior |
|---|---|---|
| `ARCH-BOUNDARY` | `fast` | no forbidden dependency edge or vendor type in public contracts |
| `ROOT-PARITY` | `play`, `persistence-replay` | fixed scenarios produce equal commands/events/gameplay state in `game` with null presentation and `headless` |
| `OFFLINE` | `play` | network/LLM/`ai-host` absent does not break mandatory gameplay loop |
| `SAVE-REPLAY` | `persistence-replay` | save/load/replay reaches the same authoritative state |
| `PACKAGE-DOGFOOD` | `content-package` | first-party mechanics and archetypes use only public package APIs |

These are normal executable checks, not admission objects. Они не требуют
отдельного serialized approval/process graph. A failed check is fixed before
claiming the affected feature works.

## Autonomous narrative and divine architecture

[SPEC-31](31-autonomous-quest-lifecycle-and-narrative-director.md) and
[ADR-029](adr/029-rpg-owned-quest-graph-and-optional-narrative-director.md)
define the Accepted autonomous-quest and optional narrative-director boundary.
Runtime implementation remains product work and is not implied by architecture
acceptance.

[ADR-031](adr/031-rpg-owned-divine-standing-and-atomic-pantheon-judgment.md)
extends that track with RPG-owned divine standing and a directed
pantheon conflict resolver. Several independent god-role LLM requests may run
as optional async work, but they read one immutable pre-decision snapshot and
publish only at one fixed world decision boundary through the Runtime-owned
`CrossContextTransactionPlanV1`. RPG, Mechanics and quest-graph owner subplans
commit together or not at all; external completion order cannot move the
boundary or become merge order.
