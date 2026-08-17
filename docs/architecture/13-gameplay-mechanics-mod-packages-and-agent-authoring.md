# SPEC-13: Gameplay mechanics and mod packages

| Поле | Значение |
|---|---|
| ID | SPEC-13 |
| Статус | Accepted |
| Версия | 2.6 |
| Последняя проверка | 2026-08-17 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [ADR-008](adr/008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-014](adr/014-deterministic-extensions-and-package-trust.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](adr/048-direct-exact-project-lock.md), [ADR-052](adr/052-derived-world-calendar-and-authored-routine-vertical.md), [ADR-072](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md), [ADR-073](adr/073-deterministic-cognition-owner-vertical.md), [ADR-074](adr/074-systemic-strategic-agent-owner-vertical.md) |
| Дополнительные зависимости V2.6 | [SPEC-36](36-functional-tissue-condition-and-injury.md), [ADR-075](adr/075-product-grounded-functional-anatomy-and-character-embodiment.md) |
| Заменяет | SPEC-13 2.5; retains the current R4d semantic-affordance consumer and adds the future product-grounded damage/treatment package boundary without changing current package schemas |

## Назначение

Этот SPEC фиксирует только реализованный public path игровых механик и mod
packages. First-party combat и interaction проходят тот же contract, что Luau
и Wasm packages. Отдельного privileged gameplay API для engine content нет.

Current authoring automation — обычный tooling concern. MCP servers, agent
roles, change-set workflows, approval packets и prompt schemas не являются
частью архитектуры движка. Они могут появиться позже как consumers существующих
CLI/contracts, но не создают новые mutation paths или release gates.

## Invariants

- Gameplay state меняется только через validated production `WorldCommand`
  transactions. Package, script и host adapter не получают mutable ECS access.
- `MechanicPackageManifestV1`, `MechanicsLockV1`, definition registries,
  `MechanicAffordanceV1` и `EffectRequestV1` являются engine-owned bounded
  contracts.
- Package identity, content hash, declared capabilities и exact mechanics-lock
  binding проверяются до исполнения.
- Luau и Wasm Component остаются двумя current extension paths. Оба используют
  те же immutable views, capability checks, effect/command validation и
  deterministic ordering.
- Instruction/fuel, allocation, live-memory, host-call, output и command limits
  конечны. Trap или limit overrun публикует ноль частичных изменений.
- Package output является untrusted proposal. Числовые правила, cooldown,
  transition graph, target/reference validity и owner invariants повторно
  проверяются engine runtime.
- `game`, `headless` и tools используют одну exact package/content closure.
  Network, provider availability и renderer cadence не влияют на механику.

## Public boundary

Публичная mechanics boundary содержит только stable IDs, hashes, immutable
definitions/views, capability descriptors, affordances, effect requests,
command proposals, receipts и bounded diagnostics. В неё не входят:

- ECS storage/components, raw pointers или mutable domain objects;
- VM/runtime handles, Rust trait objects, futures или callbacks;
- OS, renderer, physics-backend, importer и database types;
- filesystem paths, credentials, provider sessions или host process objects.

Definitions и package manifests canonical, bounded и content-addressed.
`MechanicsLockV1` перечисляет exact package revisions; runtime не выбирает
«ближайшую» версию и не сканирует ambient catalog.

## Execution path

1. Cooker validates package manifest, provenance, schema/version, capability
   declarations and bounded definitions.
2. Direct project cooking binds the exact `MechanicsLockV1` hash into
   `ProjectLockV3`.
3. Activation validates the package/content/hash closure before publishing
   `ActivatedProjectV8`.
4. Runtime gives a package only declared immutable views and deterministic
   inputs.
5. The package returns bounded `EffectRequestV1` or command proposal values.
6. Engine validators build and atomically commit the production transaction,
   then emit `DomainEvent` records from committed changes only.

Exact duplicate requests follow the command-ledger identity rules. Same
identity with different bytes fails closed. Package iteration order, VM hash
map order and host completion order cannot choose command or event order.

## Luau and Wasm

Luau runs inside the declared sandbox without ambient filesystem, network,
clock, process or native-library access. Wasm Component packages import only
the versioned engine world/capability surface; ambient WASI is denied unless a
future explicit capability contract adds a narrow path.

Both paths must preserve package state through the current content/package
round-trip and use deterministic in-process fallback or typed rejection on
absence, trap or budget exhaustion. A fallback cannot bypass capability or
domain validation.

## Failure semantics

Malformed version/hash, missing definition, forbidden capability, forged
handle, invalid target, transition violation, VM trap or resource exhaustion
rejects before mutation. The previous authoritative state, command ledger and
published project remain unchanged. Diagnostics use stable codes and typed
IDs; rendered text is not the oracle.

Pre-v1 package and authoring formats are current-only under ADR-046. A retired
format returns typed `UNSUPPORTED_*`; runtime does not apply defaults, tolerant
decode or automatic migration and does not delete user data.

## Product check

`content-package` is the governing check. It must exercise first-party,
Luau-scripted and Wasm Component mechanics through the same manifest,
capability, affordance, effect-request and command path, including malformed
input, sandbox escape, trap and every declared budget fallback. Gameplay
changes additionally run `play`; authoritative-state changes additionally run
`persistence-replay`.

## Semantic affordance projection

SPEC-32/ADR-073/074 admits a current bounded `SemanticAffordanceV1` projection
for logical-route request, hold-position, social-exchange commit, activity wait
and systemic settlement. Mechanics Runtime remains owner of ability,
work, gather, craft and trade effects and publishes only capability-filtered,
revision-bound preconditions/effects/cost/time/risk/failure metadata. An
affordance is data, not executable code or mutation authority; execution always
revalidates through the existing effect-request and `WorldCommand` path.
The production systemic executions are current only for ADR-074's authored
work/currency/trade/food closure. Broader work/gather/craft/trade affordance
shapes remain Proposed until another consumer qualifies under ADR-046.

## Future functional-injury extension

SPEC-36/ADR-075 reserve one future package-parity path for tissue damage,
treatment and recovery. First-party and community mechanics will consume the
same immutable contact/body-condition views and submit the same bounded damage
or treatment proposal through `WorldCommand`; packages will not mutate a body
condition, actuator envelope, Physics topology or wound mesh directly.

This paragraph adds no current definition, capability, operation or authoring
format. Exact injury mechanics enter this current-only SPEC only with the first
production consumer and `content-package`/`play`/`persistence-replay` coverage
under ADR-046.
