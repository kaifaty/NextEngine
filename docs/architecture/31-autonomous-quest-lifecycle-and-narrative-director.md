# SPEC-31: Future narrative director and divine agency intent

| Поле | Значение |
|---|---|
| ID | SPEC-31 |
| Статус | Proposed |
| Версия | 2.0 |
| Последняя проверка | 2026-08-08 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md) |
| Заменяет | SPEC-31 version 1.0 Accepted feature/schema design; ADR-029/031 obligations removed by ADR-046 |

## Intent

Next Engine may later support world-local quest opportunities, an optional
Narrative Director and independently authored divine patrons. This document is
direction, not a current feature contract. It defines no current schemas,
commands, ProductChecks, migration obligations or R3 work.

The detailed quest-graph, divine-standing, pantheon-conflict, offer/covenant,
cross-context transaction and LLM request/response listings from the previous
design are intentionally not copied here. They remain available in Git history
if a production vertical later needs evidence for a smaller design.

## Guardrails for future design

Any promoted implementation must preserve existing Accepted boundaries:

- RPG Framework remains the sole owner of quest/domain state. Dialogue, UI,
  NPC planner intent and model output are not mutation authority.
- Optional model output is untrusted, bounded and capability-filtered. It may
  propose typed values only; Runtime/RPG validators decide and atomically
  commit through production `WorldCommand` paths.
- Authored mandatory story facts/endings must remain reachable without a
  model, network or external process.
- Model/service absence, crash, timeout or invalid output cannot block a
  simulation tick. A deterministic in-process authored fallback is required.
- Replay consumes recorded external input and never reissues a model/network
  request for the past.
- Generated content belongs to one world/save and never silently rewrites
  project source or package content.
- A divine patron sees only authored, explicitly permitted committed facts.
  Divine choices cannot reveal hidden state or supply numeric effects directly.
- If multiple owners change, the future vertical must demonstrate one atomic
  validated commit and canonical event order; no generic cross-context wrapper
  is reserved before that need exists.
- Formats remain current-only until a public v1 support promise and real
  production evolution justify migration.

## Consumer-driven promotion

The first production consumer should implement one player-visible vertical,
for example a single bounded quest opportunity admitted from an immutable
world fact with deterministic fallback. Only contracts exercised by that
vertical may become Accepted.

Promotion requires a new ADR, minimal public contracts, explicit ownership,
positive/failure coverage through production paths and a ProductCheck proving
offline gameplay plus save/replay behavior. Until then current quest work routes
to SPEC-19 and current AI work routes to SPEC-06; this Proposed SPEC must not be
a prerequisite for unrelated changes.
