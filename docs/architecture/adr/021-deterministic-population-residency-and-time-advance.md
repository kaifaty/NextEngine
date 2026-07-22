# ADR-021: Deterministic population residency и time advance

| Поле | Значение |
|---|---|
| ID | ADR-021 |
| Статус | Proposed |
| Версия | 0.1 |
| Владелец | World Services Team |
| Требуемые согласующие | Architecture Working Group, Runtime Team, Asset & Persistence Team, Agent Intelligence Team, RPG Framework Team, Physical Embodiment Team, Verification & Evidence Team, Release Engineering |
| Дата предложения | 2026-07-22 |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-20](../20-world-simulation-and-population-lifecycle.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-08](../08-audio-navigation-and-world-services.md), [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [ADR-007](007-identities-persistence-and-replay.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Статус предложения

ADR входит в foundation-completeness proposal и остаётся `AwaitingReview` до atomic promotion всего track. Он does not accept a crowd/nav/ECS backend and does not modify physical LOD authority.

## Контекст

Accepted architecture defines WorldChunk streaming, runtime residency, physical LOD, Agent fallback and basic calendar/weather/reservation services. It does not define durable population records, schedule/time-skip semantics or ownership-preserving transitions between active and abstract simulation. A naive implementation can duplicate RPG/AI/physics state, respawn durable entities on chunk load or derive outcomes from visibility/I/O timing.

## Решение

При принятии:

1. World Services owns durable population membership, region/home, schedule cursor, abstract activity and simulation-time advance; it does not own RPG, Agent or physical fields.
2. `WorldResidencyTier` separates Active, Simulated, Abstract and Dormant computation while preserving one PersistentId and mandatory domain outcomes.
3. Runtime spawn/despawn/residency mapping is distinct from durable register/tombstone; chunk unload never means narrative deletion.
4. Schedule and bulk time advance evaluate immutable revisioned facts and submit standard proposals/WorldCommands in deterministic order.
5. Tier/region transitions are `Prepare → Validate → Commit → Stabilize`, retain source state on failure and cannot teleport physics-authoritative avatars.
6. Async workers only return revision-bound proposals through staging; completion order does not select commit order.
7. Budget pressure may deterministically defer declared work but cannot silently drop/reorder mandatory outcomes.

## Рассмотренные варианты

- Copy complete Character/AI/physics state into population service — `Rejected`: creates multiple mutable owners.
- Despawn/unload means delete and respawn from chunk — `Rejected`: breaks PersistentId, quest references and saves.
- Visibility/distance alone selects authoritative LOD — `Rejected`: renderer-dependent semantics.
- Wall-clock offline progression — `Rejected`: unreplayable and machine-dependent.
- Bulk time skip uses bespoke domain mutation — `Rejected`: diverges from WorldCommand validation and package paths.
- Best-effort stale schedule commit — `Rejected`: partial/incorrect causal history.

## Последствия

- Large-world population obtains explicit durable schemas, transitions and gates.
- Abstract simulation is intentionally bounded; unsupported precise outcomes defer/upgrade rather than fabricate physics.
- Save/replay contains more schedule/tier metadata but can diagnose first divergence.
- Performance optimization must remain inside deterministic cadence/overflow rules and shared integrated budgets.

## Gates и fallback

Acceptance requires `WORLD-POP-P1`, `WORLD-TIME-P1` and `WORLD-RESIDENCY-P1`. Failure retains/pins a safe source tier, uses stepped time, or blocks invalid project/save. No retry may transform deterministic divergence into pass.

## Promotion и supersession

ADR does not supersede ADR-007/009/010. It is accepted only with SPEC-17…20 and ADR-018…020. Moving RPG/Agent/physics authority into population records, wall-clock progression, visibility-dependent mandatory outcomes or destructive unload/respawn requires a new superseding ADR.
