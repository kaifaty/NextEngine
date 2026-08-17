# SPEC-01: Системная архитектура

| Поле | Значение |
|---|---|
| ID | SPEC-01 |
| Статус | Accepted |
| Версия | 2.8 |
| Последнее изменение | 2026-08-17 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-052](adr/052-derived-world-calendar-and-authored-routine-vertical.md), [ADR-072](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md), [ADR-081](adr/081-world-dynamics-gap-closure-and-promotion-guardrails.md) |
| Related Proposed architecture | [SPEC-38](38-continuum-material-physics.md), [SPEC-39](39-layered-physical-world.md), [SPEC-40](40-structural-vegetation-physics.md), [SPEC-41](41-world-substrate-composition.md), [SPEC-42](42-arcane-substrate-and-physical-magic.md), [SPEC-43](43-thermochemical-material-processes.md), [SPEC-44](44-neural-assisted-world-simulation.md) |

## Архитектурная форма

Next Engine разделяет durable gameplay contracts, runtime orchestration и
replaceable adapters. Граница существует ради playable offline game,
предсказуемых save/replay и быстрой замены backend, а не ради отдельной
организационной workflow-системы.

Public system boundary находится в `crates/contracts` и содержит только
versioned engine-owned values: nominal IDs, project/content/schema manifests,
`WorldCommand`, `DomainEvent`, immutable queries/snapshots, RPG operations,
world/physics/motor/animation contracts и process/plugin protocols.
Public Rust API сгруппирован по domain namespaces (`ids`, `command`, `project`,
`session`, `persistence`, `rpg`, `platform` и остальные owning domains);
root-level facade и legacy parallel contract families отсутствуют.

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
| Project composition | immutable `ProjectLockV3` | validated atomic activation before world start |
| Gameplay/RPG | revisioned RPG aggregates | validated `WorldCommand` transaction |
| Derived world calendar and bounded routine | immutable `WorldRoutineCatalogV1` + World Services `WorldRoutineSnapshotV1` | exact stage-6 proposal, stage-9 command/event and joint owner commit |
| Bounded population tiers and graph navigation | immutable `WorldPopulationCatalogV1` + `WorldNavigationCatalogV1`; World Services `WorldPopulationSnapshotV1` | exact stage-6 query/proposal, stage-9 command/event and joint owner commit |
| Runtime residency | Core Runtime mapping | deterministic spawn/despawn/streaming commit |
| Physical world | engine-owned physics state and canonical snapshot | fixed step plus validated motor/topology operation |
| Motor route/state | engine-owned policy supervisor state | deterministic route/action commit or procedural fallback |
| Animation state | engine-owned graph/root-motion/IK state | fixed animation stages; root motion is only an intent |
| Agent working state | in-process deterministic agent state | planner commit/proposal path |
| Mechanics packages | exact package lock and Mechanics Runtime state | validated mechanic delta inside command transaction |
| Assets/content | immutable content manifest and blobs | cooker atomic publish |
| Schemas/formats | immutable current schema registry | exact validation; incompatible alpha versions reject typed |
| Save | last complete manifest and segments | atomic persistence transaction |
| Application session | runtime session state and close journal | validated lifecycle transition |
| Presentation | immutable presentation snapshot and reconstructible caches | extraction/consumption only; never simulation write |

Одна строка не может иметь два mutable sources. Backend-side state допустим
только как reconstructible cache.

## Target monorepo layering

```text
crates/contracts            public schemas and nominal IDs
crates/application          production activation/session/run/close/replay coordinator
crates/reference-game       first-party project source, bootstrap and scripted product loop
crates/runtime              schedule, command bus, authority and session state machine
crates/rpg, crates/world    generic RPG and world/streaming domains
crates/project              deterministic project cook/resolve/publish/activate
crates/assets               content, save and durable session-generation publication
crates/platform             normalized engine-owned host facts
crates/presentation         immutable extraction
crates/render               presentation consumer
crates/desktop-sdl-ash      private interactive SDL/ash adapter
crates/physics-api          engine-owned backend interface and reference backend
crates/physics-physx*       replaceable PhysX adapter and narrow FFI boundary
crates/agent                deterministic agent planning
crates/mechanics            abilities, effects and package state
crates/script-luau          Luau capability host
crates/plugin-host          WIT/Wasm capability host
crates/verification         assertions, comparisons and fault scenarios only
tools/xtask                 checks, packaging and runtime-bearing Tools root
apps/game, apps/headless    required production composition roots
```

Dependencies point from roots and adapters toward engine contracts. Contracts
never depend on a vendor implementation. Production layering is
`apps/tools → application → reference-game/runtime/project/assets/platform →
contracts`; verification depends on production crates and no production crate
depends on verification.

## Production data flow

1. Application coordinator validates the exact project lock, opens or resumes
   one durable session and stages Runtime through the closed lifecycle.
2. Platform adapter emits normalized device-independent controls.
3. Player/AI/script/plugin/world logic produces command proposals, never direct
   mutable references.
4. Runtime validates capability, schema, target tick and preconditions, derives
   canonical command identity and reserves it in the durable ledger.
5. Domain systems stage changes against expected revisions.
6. One transaction publishes all owned deltas and ordered `DomainEvent`, or
   publishes nothing.
7. Async work receives immutable inputs and returns a revision-bound result to a
   deterministic commit point.
8. Physics and motor advance at fixed stages; learned output passes the same
   deterministic safety and fallback path as procedural output.
9. Presentation extracts an immutable snapshot and cannot write back to
   simulation.
10. Save/replay serializes only engine-owned authoritative state and declared
   future-affecting scheduler/session state.
11. Close stages Runtime transitions and immutable save bytes, then Assets
    atomically publishes their generation before Runtime commits the in-memory
    plan.

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

Autonomous quest generation, narrative director and divine agency are Proposed
intent in [SPEC-31](31-autonomous-quest-lifecycle-and-narrative-director.md).
ADR-029/ADR-031 current obligations are superseded by ADR-046. No narrative
graph, pantheon or generic cross-context transaction contract is part of the
current system architecture; a future player-visible production consumer must
promote the smallest required boundary through a new ADR and ProductCheck.

## Proposed post-v1 world dynamics

The current system has no `WorldDynamics` service or shared mutable world
database. SPEC-41 uses that name only for a Proposed composition of existing
Runtime/RPG/Mechanics owners with independently promoted Physical,
Thermochemical and Arcane owners. Each owner retains one write set and joins a
cross-owner effect only through immutable revision-bound projections, a typed
bounded exchange and one declared atomic commit.

ADR-081 keeps the current twelve-stage/physical-only stage-8 profile unchanged.
The first non-physical production owner must introduce a successor
`WorldDynamicsStep` at the same stage index with one runtime-owned closed DAG,
one merged PhysX integration, pre-admitted capacities, all-or-none fail-stop
publication, an explicit fault domain, fixed checkpoint epochs for exact PhysX
continuation and a successor combined gameplay-budget row.

SPEC-39 specializes physical owners; SPEC-42 owns only future arcane quantity
and execution; SPEC-43 owns only future material composition, enthalpy, phase
and reaction progress. Vital, Identity and atmosphere have no current owner.
SPEC-44 models are optional stateless N0/N1 report/shadow producers after one
already promoted classical owner, not owners, runtime advisors, laws, backends
or persistence segments. Runtime consumption needs a later mechanically
certified Accepted decision.

All these tracks remain `Proposed`. They add no current crate, public contract,
schedule stage, save segment or mandatory capability. Each becomes current
only through its own production consumer, ProductChecks and Accepted successor
decision under ADR-046.
