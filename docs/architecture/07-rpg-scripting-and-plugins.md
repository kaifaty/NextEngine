# SPEC-07: RPG framework, scripting и plugins

| Поле | Значение |
|---|---|
| ID | SPEC-07 |
| Статус | Accepted |
| Версия | 1.2 |
| Владелец | RPG Framework Team |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [ADR-006](adr/006-scripting-and-plugin-model.md) |
| Заменяет | отсутствует |

## Source of truth и ownership

RPG Framework владеет authoritative Character, Item, Quest, Dialogue, Faction, InteractiveObject, SkillProficiency state и validation rules. Luau scripts, Wasm plugins, AI, motor и importer получают immutable projections и command proposal sink. Ни один extension runtime не владеет ECS, save segments или domain fields.

## Public boundary

Публичная RPG boundary включает versioned aggregate/query views, typed command candidates, validation results, committed DomainEvents и capability-scoped Luau/WIT contexts. Внутренние ECS components, таблицы persistence, mutable aggregate references и validator implementation не входят в SDK и не передаются scripts/plugins.

## Generic RPG model

| Aggregate | Минимальный authoritative contract |
|---|---|
| `Character` | PersistentId, archetype AssetId, attributes/resources, fixed-point namespaced SkillProficiency map, inventory reference, faction memberships, authoritative directed relationship dimensions, dialogue/interaction state, embodiment reference |
| `Item` | PersistentId для instances, archetype AssetId, quantity/durability/state, owner/location, capability tags |
| `Quest` | PersistentId, definition AssetId, explicit state machine, variables, participants, causal history |
| `Dialogue` | session ID, participants, node/state, available choices, commitments, transcript policy |
| `Faction` | PersistentId, memberships/ranks, generic policies/relations |
| `InteractiveObject` | PersistentId, interaction state machine, capabilities, occupancy/reservations |
| `WorldChunk` | контракт SPEC-03 и resident gameplay records |

Legacy-specific types, VM semantics и hard-coded game names запрещены. Content-specific behaviors выражаются data schemas, tags, Luau modules и generic commands.

## Gameplay mechanics specialization

Стрельба, магия, statuses и другие reusable mechanics MUST использовать [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md): immutable Ability/Effect definitions, engine-owned Mechanics Runtime state, deterministic reducers, explicit package lock/patches и first-party dogfooding. Luau/Wasm host этого RFC обеспечивает execution/sandbox, но не создаёт альтернативный mutable mechanics API.

## Physical skill progression

`SkillProficiency` использует namespaced MotorSkillId и unsigned integer `0…10_000` согласно SPEC-14. `LearnSkill`/progression WorldCommand изменяет только RPG state; resulting SkillCapabilityChanged event запускает deterministic policy resolution. Motor route failure/delay не откатывает committed learning и не разрешает expert вне declared proficiency envelope. Scripts/plugins могут предложить progression command, но не model hash, ActivePolicyRoute или neural weights.

## WorldCommand validation

Каждый domain aggregate предоставляет command handlers и invariants. Validator проверяет issuer capability, schema/version, target revision, ownership/reservation, resources, skill progression bounds, quest/dialogue transition и physical precondition. Multi-aggregate action использует declared transaction boundary и stable lock/order по PersistentId; частичный commit запрещён.

Scripts/plugins могут только: читать capability-filtered snapshot/query; подписываться на разрешённые DomainEvent; создавать command candidate; управлять своим ephemeral state в quota; логировать structured diagnostics.

Durable package-specific gameplay state не хранится в VM globals. Оно объявляется versioned MechanicState schema и коммитится Mechanics Runtime по SPEC-13; VM получает scoped StateView и возвращает MechanicDeltaProposal.

## Luau scripting

Luau — Accepted gameplay/content scripting technology. Exact VM version pinned в implementation manifest и MUST повторять SCRIPT gates при upgrade; это не замораживает весь host API до PoC. Каждый package имеет manifest с package ID/version/content hash, requested capabilities, deterministic flag, entry points и compatible engine API range.

Default sandbox:

- separate global environment per package; standard libraries allowlisted;
- no filesystem, network, process, native library, wall clock, random entropy or debug escape;
- randomness только named engine RNG stream;
- state crossing tick/save boundary только declared versioned script state или RPG commands;
- callback default budget: 100 000 VM instructions, 2 ms wall-clock hard guard, 1 MiB transient allocations; project может уменьшить, увеличение выше 1 000 000 instructions/8 ms/8 MiB требует trusted-package declaration;
- total per-frame scripting budget default 4 ms; scheduler defers non-critical callbacks, но critical command validation не исполняется в script.

Instruction quota определяет reproducible overrun; wall guard защищает host и не используется как simulation decision.

## Wasm plugins

Plugin manifest содержит plugin ID/version/hash/signature metadata, WIT world ID, supported interface range, requested capabilities, max memory/tables/instances, fuel per call/frame, deterministic flag и dependencies. Default untrusted caps: 64 MiB linear memory, 1 table, 1 instance, 10 million fuel/call, 20 million fuel/frame; host MAY выдать меньше.

WIT contracts используют value/resource handles, PersistentId/AssetId и typed `result`; RuntimeEntityId/raw pointer/vendor handle запрещены. Host поддерживает current major N и предыдущий N-1 compatibility adapter в пределах опубликованной matrix. Major mismatch → plugin не загружается, игра продолжает без optional plugin либо отказывает до world load, если project manifest честно объявил plugin required.

Wasmtime — Proposed backend. Component Model feature set pinned; preview/unstable proposals disabled, если не перечислены в manifest и evidence.

## Capability model

Capabilities granular и namespaced, например `rpg.character.read`, `rpg.command.inventory.propose`, `events.quest.subscribe`, `ui.panel.register`. Capability не подразумевает дочерние права. Install-time grant пересекается с project policy и runtime context; denial возвращает stable code и audit record. Gameplay package не может делегировать capability другому package.

## Data flow

`DomainEvent/immutable query → package callback → budgeted computation → command candidate → common validator → accepted WorldCommand → RPG transaction`. Extension local state snapshot выполняется после command commit и имеет package schema/version/hash. Hot reload разрешён только tools/dev, отменяет in-flight callbacks и отмечает run non-release/replay-incompatible.

## Failure semantics

- Luau error/budget overrun → callback abort, uncommitted candidates discarded, package strike; critical package после threshold вызывает clean project error, optional отключается.
- Wasm trap/fuel/memory violation → instance terminated, resources reclaimed, audit diagnostic; host/game не падает.
- Capability denial → no side effect; script/plugin MAY выбрать documented fallback.
- Incompatible required package/state migration → fail before world mutation.
- Invalid skill ID/proficiency delta → progression command rejected; active motor route и model bytes не меняются.
- Nondeterministic API request из deterministic package → denial.
- Repeated 3 violations/60 s default → circuit-break package до explicit reload/new session.

## Verification gates

| Gate | Сценарий | Threshold | Evidence | Fallback |
|---|---|---|---|---|
| RPG-01 | generic NPC/dialogue/quest/item/skill vertical fixture | exact expected state/event hashes, fixed-point proficiency bounds и save/load parity | replay/report | release block |
| SCRIPT-P1 | sandbox escape/adversarial API corpus | 100% filesystem/network/native/debug escape denied | audit + fuzz report | remove API/pin prior Luau |
| SCRIPT-P2 | instruction/wall/allocation overruns | stop ≤2 ms after hard threshold observation; 0 partial command; host healthy 10 000 runs | metrics | disable offending package |
| SCRIPT-P3 | determinism replay | exact accepted commands/state hash for 100 seeds | replay report | mark/reject nondeterministic package |
| SCRIPT-P4 | save migration N-1→N | 100% fixtures exact; invalid state fail-closed | migration report | require old package/export |
| PLUGIN-P1 | capability denial suite | 100% denied, 0 side effects | audit log | disable plugin |
| PLUGIN-P2 | trap/fuel/memory fuzz | 10 000 cases, 0 host crash/leak; RSS residual ≤16 MiB | sanitizer/memory report | pin Wasmtime/disable loading |
| PLUGIN-P3 | WIT N/N-1 negotiation | 100% declared matrix; N-2/major mismatch cleanly rejected | compatibility report | compatibility adapter/pin prior runtime |
| PLUGIN-P4 | malicious component fuzz 24 CPU-hours | 0 sandbox escape/host panic/UB | fuzz report | plugin feature release-blocked |
| PLUGIN-P5 | required/optional failure startup | optional project remains playable; required fails before world mutation with stable code | scenario report | remove/replace plugin |

Mechanic package resolution, durable state, agent authoring и first-party shooting/magic gates определены SPEC-13. Physical skill/proficiency/policy routes дополнительно проходят SPEC-14; VM не создаёт альтернативный motor API.
