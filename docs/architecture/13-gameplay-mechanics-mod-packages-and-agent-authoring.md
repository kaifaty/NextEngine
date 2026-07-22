# SPEC-13: Gameplay mechanics, mod packages и agent authoring

| Поле | Значение |
|---|---|
| ID | SPEC-13 |
| Статус | Proposed |
| Версия | 1.3 |
| Владелец | Gameplay Extensibility Team |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-008](adr/008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-014](adr/014-artifact-first-review-baselines-and-attestation-v2.md), [ADR-015](adr/015-luau-wasm-package-trust-v2.md) |
| Связанные документы | [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-16](16-ai-assisted-world-and-asset-generation.md), [ADR-010](adr/010-artifact-first-headless-validation-and-review.md), [ADR-017](adr/017-artifact-first-ai-content-generation.md) |
| Заменяет | SPEC-13 v1.2 после human approval exact candidate hash |

## Назначение и product invariants

Подсистема позволяет engine team, мододелам и автоматизированным coding agents создавать игровые механики через один публичный contract. First-party стрельба, магия и последующие mechanics MUST использовать те же package schemas, capabilities, commands, tests и diagnostics, что community packages. Hidden first-party gameplay API запрещён.

Большинство новых механик MUST собираться из data + existing primitives. Luau применяется для author-friendly orchestration, Wasm Component Model — для сложных переносимых algorithms. Новый native Rust code требуется только когда package model не имеет фундаментального primitive; такое расширение проходит обычный RFC/ADR review и затем становится public primitive для всех авторов.

## Source of truth и ownership

| Состояние | Единственный owner/source of truth | Роль package |
|---|---|---|
| Installed package closure | Project `MechanicsLock` + content hashes | объявляет dependencies/capabilities; не определяет load order неявно |
| Mechanic definitions | Cooked immutable package registry | поставляет definitions/patches через validator/cooker |
| Ability instances, statuses и package state | Engine-owned Mechanics Runtime store внутри WorldCommand transaction | reducer предлагает delta только своего namespace |
| RPG attributes/resources/inventory/quest state | RPG Framework | package создаёт validated EffectRequest/WorldCommand candidate |
| Projectile/body/contact pose | Physical Embodiment/PhysicsBackend | package задаёт descriptors/intents; не записывает transforms |
| Creature/physical/skill/model definitions | Cooked immutable content registry + SPEC-14 certification | package поставляет references/assets; не владеет proficiency или active route |
| Presentation cues | PresentationSnapshot + presentation subsystem | package связывает semantic cue с assets; cue не authoritative |
| Agent edits | Filesystem/git после явного apply; до apply — AgentChangeSet | agent не получает runtime authority |
| Required test/review plan | Generated ChangeImpactManifest | package/agent может добавить suites, но не удалять resolved requirements |

Gameplay Extensibility Team владеет Mechanics Runtime, registries, standard primitives, package/lock schemas, reducer host, extension-point ordering и authoring SDK. Package author владеет package source, schemas, migrations и tests, но authoritative commit всегда выполняет engine.

## Public boundary и normative data flow

Public boundary состоит из `MechanicPackageManifest`, `MechanicsLock`, definitions, namespaced schemas, reducer ABI для Luau/WIT, capability catalog, WorldCommand/DomainEvent schemas, TestScenario specialization, ChangeImpact/Evidence references, CLI/JSON authoring contracts и optional MCP projection.

```text
package source
  → validate dependencies/capabilities/schemas/provenance
  → deterministic cook + exact MechanicsLock
  → immutable MechanicRegistry
  → InvokeAbility / MechanicCommand candidate
  → structural + capability + RPG/physics precondition validation
  → budgeted deterministic reducer
  → MechanicDeltaProposal + EffectRequests + event payload proposals
  → engine validation + atomic WorldCommand commit
  → save/replay delta + PresentationCue
```

Package code не получает mutable ECS reference, raw pointer, RuntimeEntityId, filesystem, network, wall clock, OS entropy или vendor handle.

## Уровни расширения

| Уровень | Содержимое | Typical use | Trust/runtime |
|---|---|---|---|
| Data-only package | definitions, tags, assets, explicit patches, tests | новое оружие, spell variants, balance, statuses, recipes | no executable code; strongest portability |
| Luau mechanic package | data + deterministic Luau reducers/orchestration | combo logic, cast phases, contextual interactions | SPEC-07 sandbox/budgets |
| Wasm mechanic plugin | data + WIT component | custom targeting, ballistics, procedural effect algorithms | capabilities/fuel/memory; no ambient authority |
| Native engine contribution | Rust core primitive/backend | новый physics query kind, scheduler/backend capability | не является mod package; RFC/ADR + engine release |

Official package status MAY выдавать более высокий declared budget/capability, но MUST NOT открывать скрытый API или обходить WorldCommand validation.

## Package identity, manifest и lock

`MechanicPackageId` — stable reverse-domain-like lowercase namespace, например `org.nextengine.core.combat` или `community.author.spellpack`. ID не меняется при transfer ownership; publisher identity/signature хранится отдельно.

`MechanicPackageManifest` MUST содержать:

- package ID, semantic version, package/content hash и human metadata;
- supported engine mechanics API range и required features;
- required/optional dependencies, incompatibilities и feature flags;
- requested capabilities и runtime budgets;
- exported definitions, commands, events, reducers, hooks и cue namespaces;
- durable state schemas, migration graph и uninstall migration;
- explicit patch targets с expected base AssetId/revision hash;
- license, source/provenance, generated/AI-assisted disclosure и signature metadata;
- test scenarios и expected artifact hashes.
- exported creature/physical/policy/skill assets, model provenance и certification claims when present; claims валидируются SPEC-14 и не следуют из package signature.

`MechanicsLock` является generated immutable resolution: exact package versions/hashes, dependency graph, enabled features, granted capabilities, patch order, schema/migration versions и package source. Runtime MUST NOT resolve floating ranges. Dependency cycle, missing required dependency, hash mismatch или incompatible exclusive extension point fails before world mutation.

Каждый package artifact проходит trust policy SPEC-07/ADR-015: `UnsignedLocal` требует explicit consent и остаётся local/untrusted; official или required distributed package требует non-revoked publisher signature. Trust tier не меняет Mechanics Runtime validation, capability ceiling или deterministic patch semantics.

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
| `EffectTransaction` | Canonical validated result applied atomically to RPG + Mechanics Runtime state |
| `StatusDefinition/StatusInstance` | Persistent or timed buff/debuff/state with stacking, immunity, dispel и tick policy |
| `ProjectileDefinition` | Spawn/collision/lifetime/ownership/effect mapping; physics owns actual pose/contact |
| `AreaFieldDefinition` | Versioned spatial effect source with cadence, membership query и lifetime |
| `PresentationCue` | Semantic non-authoritative request for VFX/audio/UI/camera feedback |
| `MechanicTestScenario` | SPEC-15 TestScenarioManifest specialization: initial fixture, package lock, seeds, commands/faults, probes/assertions/captures |
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
7. produce atomic `EffectTransaction`;
8. commit state и emit DomainEvents;
9. publish PresentationCues.

Package MAY участвовать только в declared extension points. Core effect operations v1:

- modify namespaced Attribute/Resource through owner validator;
- grant/remove/refresh StatusInstance;
- consume/transfer/spawn/despawn generic Item or Character through RPG commands;
- spawn/despawn Projectile or AreaField from cooked descriptors;
- request PhysicalAvatarIntent, bounded impulse/force or normalized physics query;
- start/advance generic Interaction, Dialogue или Quest transition при отдельной capability;
- emit deterministic perception stimulus;
- update package-owned MechanicState namespace.

Direct `set health`, arbitrary component write, raw transform write и unvalidated event injection запрещены. Damage, healing и resource changes выражаются EffectRequest, чтобы armor, immunity, difficulty и audit оставались composable.

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

Coding agent не получает специальной runtime capability. Он использует тот же public SDK, filesystem review workflow и tests, что человек. Agent-friendly означает machine-discoverable, bounded и self-correcting tooling, а не автоматическое доверие к generated output.

`AuthoringContextBundle` MUST содержать exact engine/package hashes, SDK manifest, resolved schemas, commands/events/capabilities/affordances, dependency/ownership/output-category graph, VerificationPolicy/TestScenario/ChangeImpact/diagnostic/fix-it registry, budgets, migration rules, minimal examples и relevant asset metadata. Physical scope дополнительно включает body/morphology/topology/actuator schemas, skill catalog, policy compatibility/normalization schemas, training configs, evaluation/transition suites, certification rules и provenance requirements из SPEC-14. Generation scope дополнительно включает recipe/profile schemas, approved catalog view, adapter capability descriptors, normalization/world budgets и provenance policy из SPEC-16. Bundle не включает secrets, reviewer credentials, raw protected assets, quarantined generated bytes, provider account/session handles, raw training datasets/checkpoints, irrelevant repository files или mutable runtime handles. Все ссылки внутри bundle MUST быть замкнуты либо иметь explicit unavailable reason.

Обязательный workflow:

```text
describe/context → scaffold → edit → validate/fix-it
→ ImpactResolver → agent-fast scenarios/diagnose/minimize
→ changeset replay/fault/capture → EvidenceBundle
→ AgentPolicy + HumanReviewDecision when required → atomic apply/package
```

`AgentChangeSet` содержит objective, base file/package hashes, bounded file edits, requested tool operations, generated-asset recipe/job/result/provenance/normalization hashes when applicable, author-declared impact additions, tests/gates run, artifacts и risk flags. Engine-generated ChangeImpactManifest является separate immutable admission input; author/agent не может уменьшить его. Apply MUST поддерживать dry-run, path allowlist, precondition/evidence/review hashes и atomic rollback. Stale base, out-of-root path, undeclared binary, missing resolved gate или invalid HumanReviewDecision blocks apply.

Project `AgentPolicy` задаёт разрешённые roots/package namespaces, tool operations, budgets и approval rules. Low-risk non-observable data/Luau changes MAY auto-apply в disposable branch/worktree после resolved automatic gates. Observable visual/UI/camera/animation/physics/motor/audio impact всегда требует SPEC-15 human review. New capability, durable migration, binary/Wasm, model-weight promotion, physical certification, license/provenance change, protected asset access или release branch mutation дополнительно требует explicit owner review. Policy decision и identity входят в AgentChangeSet audit; agent не может ослабить собственную policy или получить reviewer credential.

Machine diagnostic MAY включать safe fix-it как bounded text/data edit с base hash и schema reference. Fix-it является новым changeset candidate, не auto-executed command.

AgentChangeSet, достигающий observable output, MUST связывать exact base/candidate scenarios, metrics/replay diff, CaptureJob/EvidenceBundle и HumanReviewDecision. Capture обязателен для всех SPEC-15 observable categories; physical-avatar/controller specialization SPEC-05 остаётся обязательной и включает representative failures, а не только successful episode. Human review не может скрыть failed semantic/replay assertion.

## CLI и optional MCP adapter

CLI/JSON является normative source; graphical tools и MCP являются projections одного application service.

| Команда | Contract |
|---|---|
| `next sdk export --scope mechanics --project <p> --out <dir>` | AuthoringContextBundle для exact lock/build |
| `next mod new|describe|graph|validate|pack <package>` | scaffold/introspection/dependency validation/deterministic package build |
| `next mechanic test <package> --scenario <id>` | isolated reducer/ability/effect fixture |
| `next mechanic simulate <package> --scenario <id> --headless` | integrated fixed-tick run + RunManifest/replay |
| `next mod diff <old> <new>` | API/schema/content/capability/save compatibility diff |
| `next mod migrate <save|package-state> --to <version>` | copy-on-write migration/validation |
| `next changeset validate|apply <manifest> [--dry-run]` | safe agent/human change application |
| `next agent serve --transport stdio --project <p>` | optional local MCP projection of approved resources/tools |

Physical/policy/training commands определены SPEC-09 и SPEC-14: `next physical`, `next policy` и `lab`. Они MUST использовать тот же AuthoringContextBundle, AgentChangeSet, exit-code, diagnostic, RunManifest и review contract; training backend не получает private filesystem/runtime authority.

Scenario/impact/capture/evidence/review commands определены SPEC-09/15 и являются mandatory complete CLI/JSON path. MCP MAY читать schemas/results или предложить AgentChangeSet, но не подписывает HumanReviewDecision и не хранит reviewer credentials.

MCP adapter имеет статус `Proposed`, pins stable protocol revision и MUST NOT использовать experimental protocol features для baseline. Resources предоставляют context/schema/graphs/diagnostics/artifacts. Read-only tools mirror describe/validate/test/simulate/diff. Mutating tool может только создать или dry-run/apply AgentChangeSet при explicit capability/user approval; generic shell/file/network tool запрещён. Adapter default local stdio, network-off, project-root scoped, fully audited. Fallback — CLI/JSON с той же semantics.

## Versioning, distribution и uninstall

- Stable mechanics schemas/WIT worlds follow SemVer and N/N-1 compatibility policy; experimental surface clearly namespaced and запрещён required v1 package без pin.
- Cooked package MUST быть standard immutable content-addressed bundle SPEC-03 с package root manifest/lock fragment/SBOM/licenses/provenance/tests; отдельный runtime archive/parser format не создаётся. Signature optional для local community mod, required для official distribution.
- Unsigned package получает только untrusted capability ceiling и явное user consent; подпись не обходит sandbox/validation.
- Save/replay records exact MechanicsLock and package state schemas/hashes.
- Missing required package or incompatible state fails before world mutation.
- Optional package can be removed only если uninstall migration eliminates/transfers all durable state/references. Silent state drop запрещён.
- Hot reload разрешён только development: cancel callbacks, transactionally migrate state, increment registry revision и mark replay/run non-conforming.
- Distributed policy weights являются licensed content assets. Raw datasets и intermediate checkpoints остаются вне engine repository; package содержит только необходимую provenance linkage, exported immutable model и certification artifacts.

## Failure semantics

- Dependency/patch/hook conflict → deterministic pre-world diagnostic; no guessed load order.
- Reducer trap/timeout/budget/schema violation → discard whole proposal, strike/circuit-break package; no partial effect.
- Invalid/stale AgentChangeSet → reject without filesystem mutation.
- MCP unavailable/incompatible → CLI/JSON workflow remains complete.
- Generation adapter unavailable or candidate fails provenance/normalization/world gates → retain catalog/manual/prior asset; no partial binary or reference enters package/changeset.
- Missing migration/package → preserve original save/package; fail-closed or run explicit uninstall migration.
- Package physical behavior fails contacts/safety/media gates → package version rejected; old valid version remains lockable.
- Missing/inconsistent physical model provenance or certification → claim rejected before publish; package MAY remain explicit PrototypeFallback when its manifest permits it.
- PresentationCue missing → declared presentation fallback; authoritative effect remains valid only when package declares cue optional.
- First-party package requires hidden API → architecture gate failure, not exception.
- Agent omits resolved suite/capture/review или declares unknown output non-observable → ImpactResolver replaces with maximal+HumanReviewRequired; apply blocked.
- Automatic gate fails but review is present → review rejected, changeset unchanged.

## Verification gates

| Gate | Сценарий | Threshold | Evidence | Fallback/rollback |
|---|---|---|---|---|
| MECH-01 | first-party ranged + elemental packages through public SDK | 100% commands/schemas/capabilities resolved publicly; 0 hidden/native gameplay dependency; all required fixtures pass | public API scan, package graphs, replay reports | add missing public primitive via RFC/ADR; no hidden bypass |
| MECH-02 | reducer/effect determinism, 100 seeds × 10 repeats | exact proposal, accepted command, event and final state hashes | RunManifests/replay diff | reject nondeterministic reducer/package |
| MECH-03 | hook/dependency/patch permutation and conflict corpus | valid permutations resolve identical DAG/order; 100% cycles/exclusive/precondition conflicts fail before world mutation | resolver report | remove conflicting package/explicit patch revision |
| MECH-04 | save/load/migrate/uninstall N/N-1 + fault points | exact state hashes; 100% invalid/missing migrations fail-closed; original preserved | migration/fault report | pin old package or explicit export/uninstall tool |
| MECH-05 | 100 actors, 1 000 statuses, 200 active abilities | Mechanics Runtime p95 ≤2 ms, p99 ≤4 ms per 30 Hz gameplay tick on reference CPU; bounded queues/memory | profile/queue metrics | lower mechanic LOD/cadence or optimize public primitive |
| MECH-06 | shooting/magic physical interaction suite | exact gameplay outcomes; required PHYS/contact/safety gates pass; material controller changes include SPEC-05 media artifacts | replay/contact traces/GIF/MP4 manifests | reject package version/pin prior |
| MOD-01 | clean resolve/cook/package Win/Linux | exact MechanicsLock and platform-neutral package hashes; 0 implicit override | lock/hash/package reports | block package publish |
| MOD-02 | unsigned/malicious package/capability/provenance corpus | 100% forbidden capabilities/paths/network denied; no host crash/partial commit | security/fuzz/audit report | quarantine/disable package |
| AGENT-01 | cold author workflow using only AuthoringContextBundle | 100% internal references resolve; reference harness fixes one seeded schema error, creates a new data-only spell + ranged variant, validates/tests/simulates without private source knowledge | context closure/fix-it report, changeset, RunManifests | fix SDK/context generator |
| AGENT-02 | stale/path escape/binary/destructive/faulted AgentChangeSets | 100% unsafe/stale sets rejected; valid set dry-run/apply/rollback hashes exact; 0 out-of-root mutation | changeset audit/fault report | manual reviewed patch workflow |
| MCP-P1 | MCP vs CLI parity on pinned stable revision | 100% resource/tool schemas map to CLI/JSON results; read-only and mutation capability tests pass; default run opens 0 network sockets | schema diff, audit log, packet capture | disable MCP adapter; CLI/JSON remains conforming |
| MECH-07 | NPC planner discovers package affordances | NPC uses added spell/ranged variant in 100 deterministic scenarios without custom AI code; 100% planner-visible abilities have valid affordance; offline outcomes schema-correct | affordance registry, planner traces, replay reports | mark ability manual-only/fix affordance; package gate fails |

Gameplay Extensibility Team владеет MECH/MOD/AGENT gates; Security co-owns MOD-02/MCP-P1, Release Engineering принимает artifacts.
