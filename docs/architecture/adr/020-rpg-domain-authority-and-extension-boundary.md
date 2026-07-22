# ADR-020: RPG domain authority и extension boundary

| Поле | Значение |
|---|---|
| ID | ADR-020 |
| Статус | Proposed |
| Версия | 0.1 |
| Владелец | RPG Framework Team |
| Требуемые согласующие | Architecture Working Group, Runtime Team, Gameplay Extensibility Team, Agent Intelligence Team, Asset & Persistence Team, World Services Team, Security & Governance Team, Verification & Evidence Team |
| Дата предложения | 2026-07-22 |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-19](../19-rpg-domain-and-narrative-state.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-07](../07-rpg-scripting-and-plugins.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [ADR-006](006-scripting-and-plugin-model.md), [ADR-007](007-identities-persistence-and-replay.md), [ADR-008](008-mechanics-mod-package-and-agent-authoring-model.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Статус предложения

ADR входит в foundation-completeness proposal и остаётся `AwaitingReview` до atomic promotion всего track. Он не supersede scripting/plugin/mechanics decisions и не принимает dialogue model proposal.

## Контекст

SPEC-07 перечисляет generic RPG aggregates, но одновременно владеет Luau/Wasm host. Без отдельного domain contract aggregate transitions, inventory/equipment atomicity, quest/dialogue commitments, faction/relationship semantics and migrations могут оказаться реализованы в scripts, mechanics reducers или AI memory, создавая multiple sources of truth.

## Решение

При принятии:

1. RPG Framework remains sole owner of Character, Item/inventory/equipment, Quest, Dialogue, Faction/relationship and InteractiveObject authoritative fields.
2. Every aggregate has PersistentId, schema/revision, immutable view and explicit state-machine/validation rules.
3. All mutations occur inside validated WorldCommand transactions; multi-aggregate mutation/event set commits atomically in stable PersistentId order.
4. Mechanics, Luau, Wasm, AI, UI, physics and world services only read immutable facts and submit standard proposals/commands.
5. Narrative commitment becomes fact only after corresponding domain commands commit; presentation/model text cannot pre-commit state.
6. Save/migration preserves exact definitions, revisions and causal history; incompatible or incomplete input fails closed on a copy.
7. Combat, magic, crafting and trade specialization remain ordinary SPEC-13 packages over generic RPG operations, not hidden subsystems.

## Рассмотренные варианты

- Script-owned quest/dialogue/inventory state — `Rejected`: VM lifecycle and package order become authority.
- Mechanics Runtime owns arbitrary RPG fields — `Rejected`: package state bypasses aggregate ownership and migrations.
- AI memory/LLM output updates relationships/commitments directly — `Rejected`: untrusted proposal becomes authoritative.
- Partial multi-aggregate commit with compensation — `Rejected` for normative domain transaction: replay and failure semantics become ambiguous.
- Separate first-party hardcoded combat/crafting stores — `Rejected` by ADR-008 and public dogfooding rule.

## Последствия

- SPEC-07 may focus on extension execution while SPEC-19 owns domain semantics after promotion.
- More command/state schemas and migrations are explicit, but tests can prove exact ownership and failure behavior.
- Packages remain expressive through generic effects/operations without direct aggregate storage access.
- Dialogue/model innovation remains isolated from commitment and save authority.

## Gates и fallback

Acceptance requires `RPG-DOMAIN-P1`, `RPG-TRANSACTION-P1` and `RPG-MIGRATION-P1`. Failing definition/package/migration cannot be accepted best-effort; fallback is prior compatible schema/package/export tool or clean project/save rejection.

## Promotion и supersession

ADR does not supersede ADR-006/007/008. It is accepted only with SPEC-17…20 and ADR-018/019/021. Moving RPG ownership into script/plugin/mechanics/AI/presentation state, allowing partial domain transaction or introducing hidden first-party domain path requires a new superseding ADR.
