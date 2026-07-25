# SPEC-07: RPG framework, scripting и plugins

| Поле | Значение |
|---|---|
| ID | SPEC-07 |
| Статус | Accepted |
| Версия | 1.7 |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [ADR-014](adr/014-deterministic-extensions-and-package-trust.md) |
| Заменяет | отсутствует |

## Source of truth и ownership

RPG Framework владеет authoritative Character, Item, Quest, Dialogue, Faction, InteractiveObject, SkillProficiency state и validation rules по [SPEC-19](19-rpg-domain-and-narrative-state.md). Этот документ владеет Luau/Wasm execution host, capability/budget/circuit-breaker semantics и extension proposal boundary, но не создаёт второй RPG store. Luau scripts, Wasm plugins, AI, motor и importer получают immutable projections и command proposal sink. Ни один extension runtime не владеет ECS, save segments или domain fields.

## Public boundary

Публичная extension-facing RPG boundary включает versioned aggregate/query views и typed operations из SPEC-19, command candidates, validation results, committed DomainEvents и capability-scoped Luau/WIT contexts. `RpgTransactionPlan`, mutable aggregate references, internal ECS components, persistence tables и validator implementation не входят в extension SDK и не передаются scripts/plugins.

## Generic RPG model

| Aggregate | Минимальный authoritative contract |
|---|---|
| `Character` | PersistentId, archetype AssetId, attributes/resources, fixed-point namespaced SkillProficiency map, inventory reference, faction memberships, authoritative directed relationship dimensions, dialogue/interaction state, embodiment reference |
| `Item` | PersistentId для instances, archetype AssetId, quantity/durability/state и capability tags; membership/location выводятся из owning Inventory/World Services state |
| `Quest` | PersistentId, definition AssetId, explicit state machine, variables, participants, causal history |
| `Dialogue` | session ID, participants, node/state, available choices, commitments, transcript policy |
| `Faction` | PersistentId, memberships/ranks, generic policies/relations |
| `InteractiveObject` | PersistentId, interaction state machine и capabilities; reservation reference проверяется против World Services |
| `WorldChunk` | контракт SPEC-03 и resident gameplay records |

Legacy-specific types, VM semantics и hard-coded game names запрещены. Content-specific behaviors выражаются data schemas, tags, Luau modules и generic commands.

## Gameplay mechanics specialization

Стрельба, магия, statuses и другие reusable mechanics MUST использовать [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md): immutable Ability/Effect definitions, engine-owned Mechanics Runtime state, deterministic reducers, explicit package lock/patches и first-party dogfooding. Luau/Wasm host этого RFC обеспечивает execution/sandbox, но не создаёт альтернативный mutable mechanics API.

## Physical skill progression

`SkillProficiency` использует namespaced MotorSkillId и unsigned integer `0…10_000` согласно SPEC-14. `LearnSkill`/progression WorldCommand изменяет только RPG state; resulting SkillCapabilityChanged event запускает deterministic policy resolution. Motor route failure/delay не откатывает committed learning и не разрешает expert вне declared proficiency envelope. Scripts/plugins могут предложить progression command, но не model hash, ActivePolicyRoute или neural weights.

## WorldCommand validation

Каждый domain aggregate предоставляет command handlers и invariants по SPEC-19. Validator проверяет issuer capability, schema/version, target revision, ownership/reservation, resources, skill progression bounds, quest/dialogue transition и physical precondition. Multi-aggregate action компилируется RPG Framework в immutable `RpgTransactionPlan`, валидирует read/write revision set до staging и коммитит mutations/DomainEvents atomically в stable PersistentId/operation/event order; частичный commit запрещён. Extension получает только final typed result/events, а не plan internals или mutable locks.

Scripts/plugins могут только: читать capability-filtered snapshot/query; подписываться на разрешённые DomainEvent; создавать command candidate; управлять своим ephemeral state в quota; логировать structured diagnostics.

Durable package-specific gameplay state не хранится в VM globals. Оно объявляется versioned MechanicState schema и коммитится Mechanics Runtime по SPEC-13; VM получает scoped StateView и возвращает MechanicDeltaProposal.

## Luau scripting

Luau — Accepted gameplay/content scripting technology. Exact VM version pinned в implementation manifest и MUST повторять SCRIPT product checks при upgrade; это не замораживает весь host API до PoC. Каждый package имеет manifest с package ID/version/content hash, requested capabilities, deterministic flag, entry points и compatible engine API range.

Default sandbox:

- separate global environment per package; standard libraries allowlisted;
- no filesystem, network, process, native library, wall clock, random entropy or debug escape;
- randomness только named engine RNG stream;
- state crossing tick/save boundary только declared versioned script state или RPG commands;
- callback default authoritative budget: 100 000 VM instructions, 1 MiB cumulative transient allocations, declared live-memory ceiling, host-call cost units и proposed-command bytes/count из versioned `ExtensionBudgetPolicyV1`;
- total per-gameplay-tick scripting work входит в ADR-016 mechanics row; scheduler defers non-critical callbacks только по deterministic queue policy, а critical command validation не исполняется в script.

Instruction/fuel/allocation/host-call/command counters используют exact integer comparison и определяют reproducible overrun независимо от CPU/load/worker order. Wall watchdog защищает host, но его срабатывание всегда помечает exact run как `Fail(EXTENSION_WALL_WATCHDOG)`, discard-ит весь uncommitted proposal batch, не добавляет authoritative violation/strike или gameplay event/outcome и не может дать `Pass`.

## Wasm plugins

Plugin manifest содержит plugin ID/version/content hash, source/provenance metadata, WIT world ID, supported interface range, requested capabilities, max memory/tables/instances, fuel per call/gameplay tick, deterministic flag и dependencies. Default untrusted caps: 64 MiB linear memory, 1 table, 1 instance, 10 million fuel/call, 20 million fuel/gameplay tick; host MAY выдать меньше.

WIT contracts используют value/resource handles, PersistentId/AssetId и typed `result`; RuntimeEntityId/raw pointer/vendor handle запрещены. Host поддерживает current major N и предыдущий N-1 compatibility adapter в пределах опубликованной matrix. Major mismatch → plugin не загружается, игра продолжает без optional plugin либо отказывает до world load, если project manifest честно объявил plugin required.

Wasmtime — Proposed backend. Component Model feature set pinned; preview/unstable proposals disabled, если они явно не разрешены manifest.

## Capability model

Capabilities granular и namespaced, например `rpg.character.read`, `rpg.command.inventory.propose`, `events.quest.subscribe`, `ui.panel.register`. Capability не подразумевает дочерние права. Install-time grant пересекается с project policy и runtime context; denial возвращает stable code и structured diagnostic. Gameplay package не может делегировать capability другому package.

Все scripts/mechanic packages/plugins имеют content hash и bounded source/provenance metadata. Effective capabilities равны пересечению request, API compatibility, project policy, explicit user grant и hard security ceiling. Hash, provenance или publisher label не выдаёт capability и не обходит sandbox, bounds или common validation.

## Data flow

`DomainEvent/immutable query → package callback → budgeted computation → command candidate → common validator → accepted WorldCommand → RPG transaction`. Extension local state snapshot выполняется после command commit и имеет package schema/version/hash. Hot reload разрешён только tools/dev, отменяет in-flight callbacks и отмечает run non-release/replay-incompatible.

## Failure semantics

- Luau error/authoritative quota overrun → callback abort на exact counter, uncommitted candidates discarded, package strike; critical package после threshold вызывает clean project error, optional отключается. Wall watchdog → `Fail(EXTENSION_WALL_WATCHDOG)` для exact run, никогда не authoritative violation/strike, gameplay event/outcome или `Pass`.
- Wasm trap/fuel/memory violation → instance terminated, resources reclaimed, structured diagnostic; host/game не падает.
- Capability denial → no side effect; script/plugin MAY выбрать documented fallback.
- Incompatible required package/state migration → fail before world mutation.
- Invalid skill ID/proficiency delta → progression command rejected; active motor route и model bytes не меняются.
- Nondeterministic API request из deterministic package → denial.
- Default circuit breaker отключает package/plugin principal после третьего authoritative violation в inclusive sliding window `1_800` gameplay ticks. Violation — только deterministic quota exhaustion, trap, invalid command/capability attempt или schema violation из versioned `ExtensionBudgetPolicyV1`; wall watchdog violation не добавляет.
- На tick нового violation runtime вычисляет `window_start_tick = current_tick.saturating_sub(1_799)`, удаляет entries с `entry_tick < window_start_tick`, добавляет current entry и затем сравнивает count; count ≥3 открывает circuit. Поэтому на ticks `0…1_798` lower bound равен `0`, а начиная с tick `1_799` inclusive window всегда содержит не более `1_800` gameplay ticks. Reset разрешён только declared admin/load/migration command с committed diagnostic; wall seconds, implicit reload и new-session reset запрещены.

## Product checks

| Check ID | Scenario / command | Expected behavior / fallback |
|---|---|---|
| RPG-01 | Generic NPC/dialogue/quest/item/skill fixture | State and event hashes, fixed-point proficiency bounds and save/load behavior are exact; invalid domain behavior blocks the affected package or build. |
| SCRIPT-P1 | Sandbox-escape and adversarial API corpus | Filesystem, network, native and debug escapes are denied; remove the exposed API or pin the prior Luau runtime. |
| SCRIPT-P2 | Instruction, allocation, host-call and command quotas at limit-1/limit/limit+1 | Accept/reject tick and diagnostic are exact, no partial command is committed, and save/reload preserves the ledger; disable the offending package on violation. |
| SCRIPT-P3 | Determinism replay over 100 seeds | Accepted commands and state hashes match exactly; reject a nondeterministic package. |
| SCRIPT-P4 | Save migration N-1→N | Valid fixtures migrate exactly and invalid state fails closed; use the prior package/export path if migration is unavailable. |
| SCRIPT-P5 | Same corpus under CPU, load, worker and watchdog permutations | Deterministic counters and authoritative ledger remain exact; a wall watchdog trip stops the affected run/package and never silently changes gameplay. |
| PLUGIN-P1 | Capability, trap, fuel, memory, table and instance corpus | Denials and overruns are isolated with no partial proposal, host crash or persistent leak; pin Wasmtime or disable the plugin. |
| PLUGIN-P2 | WIT N/N-1 negotiation | The declared matrix is accepted and N-2/major mismatch is rejected before world mutation; use a compatibility adapter or prior runtime. |
| PLUGIN-P3 | Public WIT/value boundary scan | No `RuntimeEntityId`, raw pointer or vendor handle crosses the boundary; current and N-1 worlds return the same typed results. |
| PLUGIN-P4 | Malicious component fuzzing | No sandbox escape, host panic or undefined behavior; disable plugin loading until fixed. |
| PLUGIN-P5 | Required/optional plugin startup failure | Optional failure leaves the project playable; required failure stops before world mutation with a stable code. |

Mechanic package resolution, durable state, agent authoring и first-party shooting/magic checks определены SPEC-13. Physical skill/proficiency/policy routes дополнительно проходят SPEC-14; VM не создаёт альтернативный motor API.
