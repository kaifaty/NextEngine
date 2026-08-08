# SPEC-20: Future world simulation and population intent

| Поле | Значение |
|---|---|
| ID | SPEC-20 |
| Статус | Proposed |
| Версия | 2.0 |
| Последняя проверка | 2026-08-08 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md) |
| Заменяет | SPEC-20 version 1.4 Accepted schema design; current obligations removed by ADR-046 |

## Intent

Next Engine is intended to support a living offline world whose NPCs and
world-owned facts progress consistently outside the player's immediate view.
This document records direction only. It does not define current public
contracts, serialized schemas, a ProductCheck or an implementation requirement.

The first production population consumer will determine the smallest useful
contract. Full calendar, population-record, schedule, residency-tier and bulk
advance schemas from the earlier design are intentionally not retained here;
they remain available in Git history.

## Guardrails for a future vertical

Any future implementation must preserve these existing Accepted invariants:

- World Services owns calendar/population facts; RPG, AI, streaming and
  presentation consume immutable revision-bound projections rather than share
  mutable state.
- Gameplay mutation occurs only through validated `WorldCommand` transactions
  and committed `DomainEvent` records.
- Host clock, frame rate, wall-time sleep and worker completion order cannot
  select an authoritative world outcome.
- `PersistentId` identifies durable actors. Loaded ECS/runtime identity remains
  ephemeral and cannot duplicate or replace durable authority.
- Residency or simulation detail is an execution choice, not a second copy of
  NPC/RPG state. Promotion/demotion must preserve one technical source of truth.
- Stepped and any future bulk advance must stop at the same observable
  decision boundaries and produce the same committed roots for the declared
  scope.
- Bounds, overflow, missing content and incompatible state fail before partial
  mutation. Optional AI/model absence uses a deterministic in-process fallback.
- Save/replay use only the current supported formats until a public v1 support
  promise creates a demonstrated migration need.

## Admission to Accepted architecture

A future change may promote a narrow population contract only together with:

1. one production consumer in the playable application;
2. the minimal engine-owned contract and owner mapping required by it;
3. focused stepped/restart/replay behavior through production paths;
4. a ProductCheck that observes the player-visible result;
5. an ADR naming which part of this intent becomes Accepted.

Until then routing for current runtime, streaming, RPG and performance work must
use their Accepted SPECs and must not require this Proposed document.
