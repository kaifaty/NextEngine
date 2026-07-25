# SPEC-13: Gameplay mechanics, mod packages и agent authoring

| Поле | Значение |
|---|---|
| ID | SPEC-13 |
| Статус | Accepted |
| Версия | 1.7 |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [ADR-008](adr/008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-014](adr/014-deterministic-extensions-and-package-trust.md), [ADR-016](adr/016-compositional-gameplay-budgets.md) |
| Заменяет | отсутствует |

## Назначение и product invariants

Подсистема позволяет разработчикам движка, мододелам и автоматизированным coding agents создавать игровые механики через один публичный contract. First-party стрельба, магия и последующие mechanics MUST использовать те же package schemas, capabilities, commands, tests и diagnostics, что community packages. Hidden first-party gameplay API запрещён.

Большинство новых механик MUST собираться из data + existing primitives. Luau применяется для author-friendly orchestration, Wasm Component Model — для сложных переносимых algorithms. Новый native Rust code требуется только когда package model не имеет фундаментального primitive; такое расширение оформляется как RFC/ADR и затем становится public primitive для всех авторов.

## Source of truth и ownership

| Состояние | Единственный owner/source of truth | Роль package |
|---|---|---|
| Installed package closure | Project `MechanicsLock` + content hashes | объявляет dependencies/capabilities; не определяет load order неявно |
| Mechanic definitions | Cooked immutable package registry | поставляет definitions/patches через validator/cooker |
| Ability instances, statuses и package state | Engine-owned Mechanics Runtime store внутри WorldCommand transaction | reducer предлагает delta только своего namespace |
| RPG attributes/resources/inventory/quest state | RPG Framework по SPEC-19 | package создаёт `EffectRequest`, который engine преобразует только в typed RPG operations и валидируемый `RpgTransactionPlan` |
| Projectile/body/contact pose | Physical Embodiment/PhysicsBackend | package задаёт descriptors/intents; не записывает transforms |
| Creature/physical/skill/model definitions | Cooked immutable content registry + SPEC-14 compatibility state | package поставляет references/assets; не владеет proficiency или active route |
| Presentation cues | PresentationSnapshot + presentation subsystem | package связывает semantic cue с assets; cue не authoritative |
| Agent edits | Filesystem/git после явного apply; до apply — AgentChangeSet | agent не получает runtime authority |

Gameplay Extensibility subsystem владеет Mechanics Runtime, registries, standard primitives, package/lock schemas, reducer host, extension-point ordering и authoring SDK. Package author владеет package source, schemas, migrations и tests, но authoritative commit всегда выполняет engine.

## Public boundary и normative data flow

Public boundary состоит из `MechanicPackageManifest`, `MechanicsLock`, definitions, namespaced schemas, reducer ABI для Luau/WIT, capability catalog, WorldCommand/DomainEvent schemas, immutable SPEC-19 RPG views/typed operations, TestScenario specialization, CLI/JSON authoring contracts и optional MCP projection.

```text
package source
  → validate dependencies/capabilities/schemas/provenance
  → deterministic cook + exact MechanicsLock
  → immutable MechanicRegistry
  → InvokeAbility / MechanicCommand candidate
  → structural + capability + RPG/physics precondition validation
  → budgeted deterministic reducer
  → MechanicDeltaProposal + EffectRequests + event payload proposals
  → engine maps effects to typed RPG operations + builds RpgTransactionPlan
  → engine validates Mechanics delta and RPG plan + atomic WorldCommand commit
  → save/replay delta + PresentationCue
```

Package code не получает mutable ECS reference, raw pointer, RuntimeEntityId, filesystem, network, wall clock, OS entropy или vendor handle.

## Уровни расширения

| Уровень | Содержимое | Typical use | Runtime boundary |
|---|---|---|---|
| Data-only package | definitions, tags, assets, explicit patches, tests | новое оружие, spell variants, balance, statuses, recipes | no executable code; strongest portability |
| Luau mechanic package | data + deterministic Luau reducers/orchestration | combo logic, cast phases, contextual interactions | SPEC-07 sandbox/budgets |
| Wasm mechanic plugin | data + WIT component | custom targeting, ballistics, procedural effect algorithms | capabilities/fuel/memory; no ambient authority |
| Native engine contribution | Rust core primitive/backend | новый physics query kind, scheduler/backend capability | не является mod package; engine source change through the normal workflow |

A project policy MAY выдать package более высокий declared budget/capability в пределах hard ceiling, но MUST NOT открывать скрытый API или обходить WorldCommand validation.

## Package identity, manifest и lock

`MechanicPackageId` — stable reverse-domain-like lowercase namespace, например `org.nextengine.core.combat` или `community.author.spellpack`. ID не меняется при transfer ownership; publisher/source metadata хранится отдельно и не влияет на capability grant.

`MechanicPackageManifest` MUST содержать:

- package ID, semantic version, package/content hash и human metadata;
- supported engine mechanics API range и required features;
- required/optional dependencies, incompatibilities и feature flags;
- requested capabilities и runtime budgets;
- exported definitions, commands, events, reducers, hooks и cue namespaces;
- durable state schemas, migration graph и uninstall migration;
- explicit patch targets с expected base AssetId/revision hash;
- license, source/provenance и generated/AI-assisted disclosure metadata;
- test scenarios и expected state/output hashes;
- exported creature/physical/policy/skill assets, model provenance и declared SPEC-14 compatibility state when present; compatibility доказывается exact schemas/hashes и product checks, а не publisher metadata.

`MechanicsLock` является generated immutable resolution: exact package versions/hashes, dependency graph, enabled features, granted capabilities, patch order, schema/migration versions и package source. Runtime MUST NOT resolve floating ranges. Dependency cycle, missing required dependency, hash mismatch или incompatible exclusive extension point fails before world mutation.

Load order выводится dependency graph. Несвязанные nodes упорядочиваются по package ID только для reproducibility; автор не может использовать это как implicit override. Override разрешён только `PackagePatch` с target path/ID, precondition hash и conflict policy. Два несовместимых patches дают validation error, не last-writer-wins.

## Universal mechanics model

| Contract | Нормативная роль |
|---|---|
| `AbilityDefinition` | Immutable asset: grants/requirements, target, costs, cooldown, phases, effects, cues и tags |
| `AbilityInstance` | Runtime state machine `Requested → Validated → Preparing → Active → Recovering → Completed`, с `Interrupted/Rejected` terminals |
| `MechanicAffordance` | Machine-readable planner/author descriptor: semantic action, actor/target requirements, cost/time/risk, expected outcome ranges, observability и failure modes |
| `TargetQuery` | Deterministic self/entity/point/ray/cone/volume/chain query с world revision и result limits |
| `CostSpec` | Atomic resource/item/status requirements and consumption policy |
| `CooldownSpec` | Clock domain, duration, group, stacking/reset rules |
| `AttributeDefinition` | Namespaced typed value with base/current bounds, units, visibility и persistence policy |
| `EffectDefinition` | Immutable composition of effect stages, duration/period, stacking, tags, mitigation и semantic cues |
| `EffectRequest` | Runtime source/target/context/spec produced by ability/contact/world rule |
| `EffectTransaction` | Canonical validated composition of one SPEC-19 `RpgTransactionPlan` plus Mechanics Runtime delta, applied atomically or not at all |
| `StatusDefinition/StatusInstance` | Persistent or timed buff/debuff/state with stacking, immunity, dispel и tick policy |
| `ProjectileDefinition` | Spawn/collision/lifetime/ownership/effect mapping; physics owns actual pose/contact |
| `AreaFieldDefinition` | Versioned spatial effect source with cadence, membership query и lifetime |
| `PresentationCue` | Semantic non-authoritative request for VFX/audio/UI/camera feedback |
| `MechanicTestScenario` | Bounded fixture: initial state, package lock, seeds, commands/faults and state/output assertions |
| `MotorCapabilityRequirement` | Required MotorSkillId/capability state/proficiency band для physical ability; actual route проверяет SPEC-14 |

Definitions immutable after cooking. Runtime-specific values находятся в instances/specs и ссылаются на AssetId + exact package revision.

## AI/planner integration

Каждая AbilityDefinition, доступная NPC planner, MUST экспортировать `MechanicAffordance`. Affordance не обещает outcome, а задаёт bounded estimate и preconditions для utility/HTN planning. Optional pure `estimate` reducer MAY уточнять результат из capability-filtered snapshot в отдельном budget; он не выполняет command/effect.

Agent Runtime получает registry только granted/known abilities персонажа и вызывает тот же `InvokeAbility` command, что player/script. Package не добавляет private tactical code. `ai-host` MAY объяснить/предложить affordance, но deterministic planner проверяет availability/cost/target и MotorCapabilityView, работая без LLM. Ability может объявить `agent_visibility = manual-only`; planner-visible ability без complete affordance не проходит validation.

## Effect pipeline и core operations

Effect pipeline MUST иметь фиксированные stages:

1. authorize source/capability;
2. validate target/revision/immunity;
3. compute costs и reserve required resources;
4. apply source modifiers;
5. apply target defense/resistance/stacking transforms;
6. clamp/validate numeric and structural invariants;
7. produce an ordered typed RPG-operation set, validate one `RpgTransactionPlan` and compose the atomic `EffectTransaction`;
8. commit state и emit DomainEvents;
9. publish PresentationCues.

Package MAY участвовать только в declared extension points. Core effect operations v1:

- modify namespaced Attribute/Resource through owner validator;
- grant/remove/refresh StatusInstance;
- consume/transfer/spawn/despawn generic Item or Character only through SPEC-19 typed RPG operations;
- spawn/despawn Projectile or AreaField from cooked descriptors;
- request PhysicalAvatarIntent, bounded impulse/force or normalized physics query;
- start/advance generic Interaction, Dialogue или Quest transition при отдельной capability;
- emit deterministic perception stimulus;
- update package-owned MechanicState namespace.

Direct `set health`, arbitrary aggregate/component write, raw transform write и unvalidated event injection запрещены. Damage, healing и resource changes выражаются `EffectRequest`; engine alone converts accepted effects into revision-checked typed RPG operations and an atomic `RpgTransactionPlan`, чтобы armor, immunity, difficulty и effect accounting оставались composable.

## Reducers и durable extension state

`MechanicReducer` является deterministic bounded function:

```text
(immutable ContextView, MechanicCommand, own StateView, named RNG)
    → Result<MechanicDeltaProposal, StableRejection>
```

`MechanicDeltaProposal` может содержать patch только namespaced MechanicState, core EffectRequests, future-tick command candidates, event payload proposals и PresentationCues. Host проверяет schema, target revision, capability, quota, finite values и owner invariant, затем коммитит всё или ничего. Authoritative `DomainEvent` создаётся engine только после успешного атомарного commit; reducer не может объявить незафиксированное изменение свершившимся фактом.

Mechanics Runtime физически владеет storage и save segment; package reducer владеет semantics своей schema. Durable state key включает package ID, schema ID/version и owning PersistentId/world scope. Unknown required schema или отсутствующая migration загружается fail-closed. Opaque unvalidated bytes не допускаются.

## Extension points, ordering и conflicts

V1 публикует только именованные hooks: `ability.authorize`, `ability.cost`, `ability.phase`, `target.filter`, `effect.source-transform`, `effect.target-transform`, `effect.immunity`, `effect.post-commit`, `status.lifecycle`, `projectile.behavior` и `presentation.cue-map`.

Hook registration declares input/output schema, required capability, dependency constraints и one of `normal`, `before:<package>`, `after:<package>`, `exclusive:<slot>`. Engine строит DAG; cycle/missing target/duplicate exclusive slot — pre-world error. При отсутствии explicit relation stable tie-breaker — package ID. Arbitrary numeric global priority и runtime registration запрещены.

Pre-commit hooks являются pure transformations/proposals и не выполняют side effects. Post-commit hook может только предложить command на future tick или cue; reentrant mutation текущей transaction запрещена.

## Capability namespaces

Минимальный public catalog MUST различать `mechanics.definition.export`, `mechanics.effect.propose`, `mechanics.state.<package>.write`, `mechanics.hook.<name>.register`, `physics.query.<kind>`, `physics.impulse.propose`, `rpg.inventory.propose`, `rpg.character.spawn.propose`, `rpg.quest.propose`, `presentation.cue.publish`, `agent.context.read`, `project.changeset.propose` и `project.changeset.apply`. Wildcard grant запрещён в distributed package; project policy выдаёт каждое sensitive право явно. Patch чужого namespace требует `package.patch.<target>` и user/project grant.

## Стрельба как ordinary package

First-party ranged combat package MUST реализовать минимум:

- `fire`, `reload`, `aim` AbilityDefinitions;
- weapon/ammunition AttributeDefinitions, CostSpec и cooldown/action phases;
- projectile либо authoritative PhysicsBackend ray query на command tick;
- PhysicalAvatarIntent для aim/recoil/manipulation вместо изменения skeleton/body pose;
- ContactEvent continuity → hit candidate → EffectRequest, с защитой от repeated/resting contacts;
- semantic shot/muzzle/impact PresentationCues;
- deterministic fixtures для miss/hit/armor/ricochet/reload/interruption/save/replay.

Renderer depth/culling не может определять попадание. Hitscan query и projectile contact принадлежат physics/world source of truth.

## Магия как ordinary package

First-party elemental magic package MUST реализовать минимум:

- cast/channel/release/interruption phases;
- resource cost, cooldown, requirements и generic TargetQuery;
- data-only instant, projectile, beam-like sampled query, AreaField и Status effects;
- resistance/immunity/stacking через common Effect pipeline;
- summon только generic Character spawn command с AssetId и capability;
- semantic cast/impact/status cues;
- deterministic fixtures для interruption, insufficient resource, immune target, stacking, area membership, save/replay.

Новый spell, использующий существующие primitives, MUST создаваться без Rust rebuild. Custom targeting/reducer MAY быть Luau/Wasm package code.

## First-party dogfooding rule

Engine repository MAY содержать reference mechanic packages, но apps/game не может link internal combat/magic crate, недоступный SDK. Architecture test MUST доказать, что first-party packages используют только exported package schemas, Luau/WIT APIs и core operations. Private native fast path допускается лишь как семантически эквивалентный optimization за public primitive и проходит parity test.

## Agent-ready authoring contract

Coding agent не получает специальной runtime capability. Он использует тот же public SDK, bounded filesystem workflow и product checks, что человек. Agent-friendly означает machine-discoverable schemas, deterministic validation и actionable diagnostics, а не доверие к generated output.

`AuthoringContextBundle` MUST содержать exact engine/package hashes, SDK manifest, resolved schemas, commands/events/capabilities/affordances, dependency and state-ownership graph, budgets, migration rules, minimal examples и relevant asset metadata. Physical scope дополнительно включает body/morphology/topology/actuator schemas, skill catalog, policy compatibility/normalization schemas и training/evaluation configs из SPEC-14. Bundle не включает secrets, raw protected assets, raw training datasets/checkpoints, irrelevant repository files или mutable runtime handles.

Основной workflow:

```text
describe/context → scaffold → edit → validate/fix-it
→ focused product scenarios → replay/fault checks
→ dry-run → atomic apply/package
```

`AgentChangeSet` содержит objective, base file/package hashes, bounded file edits, requested tool operations, generated-asset provenance, checks run и risk flags. Apply MUST поддерживать dry-run, path allowlist, base-hash preconditions и atomic rollback. Stale base, out-of-root path, undeclared binary, failed required product check или missing capability blocks apply.

Project `AgentPolicy` задаёт разрешённые roots/package namespaces, tool operations и budgets. New capability, durable migration, binary/Wasm, model-weight change, license/provenance change, protected asset access или release-branch mutation требуют explicit user authorization. Agent не может ослабить policy, которая ограничивает его собственный changeset.

Machine diagnostic MAY включать safe fix-it как bounded text/data edit с base hash и schema reference. Fix-it является новым changeset candidate, не auto-executed command.

## CLI и optional MCP adapter

CLI/JSON является normative source; graphical tools и MCP являются projections одного application service.

| Команда | Contract |
|---|---|
| `next sdk export --scope mechanics --project <p> --out <dir>` | AuthoringContextBundle для exact lock/build |
| `next mod new\|describe\|graph\|validate\|pack <package>` | scaffold/introspection/dependency validation/deterministic package build |
| `next mechanic test <package> --scenario <id>` | isolated reducer/ability/effect fixture |
| `next mechanic simulate <package> --scenario <id> --headless` | integrated fixed-tick run + replay summary |
| `next mod diff <old> <new>` | API/schema/content/capability/save compatibility diff |
| `next mod migrate <save\|package-state> --to <version>` | copy-on-write migration/validation |
| `next changeset validate\|apply <manifest> [--dry-run]` | safe changeset application |
| `next agent serve --transport stdio --project <p>` | optional local MCP projection of declared resources/tools |

Physical/policy/training commands определены SPEC-09 и SPEC-14: `next physical`, `next policy` и `lab`. Они MUST использовать тот же `AuthoringContextBundle`, `AgentChangeSet`, exit-code и diagnostic contract; training backend не получает private filesystem/runtime authority.

MCP MAY читать schemas/results или предложить `AgentChangeSet`, но не обходит capability checks и path restrictions.

MCP adapter имеет статус `Proposed`, pins stable protocol revision и MUST NOT использовать experimental protocol features в required workflow. Resources предоставляют context/schema/graphs/diagnostics/results. Read-only tools mirror describe/validate/test/simulate/diff. Mutating tool может только создать или dry-run/apply AgentChangeSet при explicit capability и user confirmation; generic shell/file/network tool запрещён. Adapter default local stdio, network-off and project-root scoped; every operation returns a structured result. Fallback — CLI/JSON с той же semantics.

## Versioning, distribution и uninstall

- Stable mechanics schemas/WIT worlds follow SemVer and N/N-1 compatibility policy; experimental surface clearly namespaced and запрещён required v1 package без pin.
- Cooked package MUST быть standard immutable content-addressed bundle SPEC-03 с package root manifest/lock fragment/SBOM/licenses/provenance/tests и exact content hashes; отдельный runtime archive/parser format не создаётся.
- Любой package получает capabilities только из explicit project/user grant в пределах hard ceiling и проходит hash, bounds, schema, license и provenance validation. Publisher/source metadata не обходит sandbox и не выдаёт capability.
- Save/replay records exact MechanicsLock and package state schemas/hashes.
- Missing required package or incompatible state fails before world mutation.
- Optional package can be removed only если uninstall migration eliminates/transfers all durable state/references. Silent state drop запрещён.
- Hot reload разрешён только development: cancel callbacks, transactionally migrate state, increment registry revision и пометить output development-only, непригодным как `persistence-replay` result.
- Distributed policy weights являются licensed content assets. Raw datasets и intermediate checkpoints остаются вне engine repository; package содержит только необходимую provenance linkage и exported immutable model.

## Failure semantics

- Dependency/patch/hook conflict → deterministic pre-world diagnostic; no guessed load order.
- Reducer trap/timeout/budget/schema violation → discard whole proposal, strike/circuit-break package; no partial effect.
- Invalid/stale AgentChangeSet → reject without filesystem mutation.
- MCP unavailable/incompatible → CLI/JSON workflow remains complete.
- Missing migration/package → preserve original save/package; fail-closed or run explicit uninstall migration.
- Package physical behavior fails contact or safety checks → package version rejected; old valid version remains lockable.
- Missing/inconsistent physical model provenance or compatibility metadata → physical route is rejected before publish; package MAY remain explicit `PrototypeFallback` when its manifest permits it.
- PresentationCue missing → declared presentation fallback; authoritative effect remains valid only when package declares cue optional.
- First-party package requires hidden API → contract violation, not exception.
- Apply-time validation failure → apply is rejected atomically and the changeset remains unchanged; a failed ProductCheck reports only the affected behavior.

## Product checks

| Check ID | Scenario / command | Expected behavior / fallback |
|---|---|---|
| MECH-01 | First-party ranged and elemental packages through the public SDK | Every command, schema and capability resolves publicly with no hidden gameplay dependency; add a missing public primitive rather than a private bypass. |
| MECH-02 | Reducer/effect determinism, 100 seeds × 10 repeats | Proposal, accepted command, event and final state hashes match exactly; reject a nondeterministic reducer/package. |
| MECH-03 | Hook/dependency/patch permutations and conflicts | Valid permutations resolve one DAG/order and all cycles, exclusive-slot and precondition conflicts fail before world mutation. |
| MECH-04 | Save/load/migrate/uninstall N/N-1 with injected faults | Valid state is exact, invalid or missing migration fails closed, and the original remains unchanged; pin the old package or use explicit export/uninstall. |
| MECH-05 | ADR-016 fixture with 100 actors, 1,000 statuses and 200 active abilities | Mechanics stays within p95 ≤2,000 us / p99 ≤4,000 us with no starvation, dropped due work or unowned span; lower deterministic LOD/cadence or optimize a public primitive. |
| MECH-06 | Shooting and magic physical interaction suite | Gameplay outcomes and contact/safety behavior are exact; reject the package version or pin the prior one on failure. |
| MOD-01 | Clean resolve/cook/package on Windows and Linux | `MechanicsLock` and platform-neutral package hashes match with no implicit override; otherwise stop packaging. |
| MOD-02 | Malicious package, capability and provenance corpus | Forbidden capabilities, paths and network access are denied with no host crash or partial commit; quarantine the package. |
| MOD-P2 | Tampered hash, provenance mismatch and capability-policy matrix | Content hashes reject tampering; effective capabilities equal the explicit bounded grant; publisher/source metadata changes no capability. |
| AGENT-01 | Cold author workflow using only `AuthoringContextBundle` | References resolve and the workflow can fix a schema error, create a data-only spell/ranged variant, validate and simulate without private source knowledge. |
| AGENT-02 | Stale, path-escape, binary, destructive and faulted `AgentChangeSet` values | Unsafe/stale sets are rejected, valid dry-run/apply/rollback is exact, and no out-of-root mutation occurs. |
| MCP-P1 | MCP versus CLI parity on a pinned stable revision | Resource/tool schemas return the same results, capability checks hold, and default operation opens no network socket; disable MCP and use CLI/JSON on mismatch. |
| MECH-07 | NPC planner discovers package affordances | NPCs use added abilities without custom AI code, planner-visible abilities have valid affordances, and offline outcomes remain schema-correct; otherwise mark the ability manual-only. |
