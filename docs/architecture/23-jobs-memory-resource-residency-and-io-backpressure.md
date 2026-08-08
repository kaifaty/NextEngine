# SPEC-23: Future R3a streaming work intent

| Поле | Значение |
|---|---|
| ID | SPEC-23 |
| Статус | Proposed |
| Версия | 2.0 |
| Последняя проверка | 2026-08-08 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md) |
| Заменяет | SPEC-23 version 1.9 generic scheduler/resource architecture; deferred by ADR-046 |

## R3a intent

The next consumer-driven vertical is exactly:

```text
chunk fetch → decode → validate → canonical commit
```

R3a must make one real chunk travel through production Assets/World/Runtime
boundaries and preserve the current gameplay result. This document does not
predefine a general job system, cancellation tree, pin/lease model, eviction
framework or public resource-scheduler API.

## Guardrails inherited from Accepted architecture

- Work receives immutable revision/hash-bound input and returns an immutable
  bounded result. Mutable ECS/world access is never held across `await` or a
  worker boundary.
- Fetch/decode staging is private and non-authoritative. A completion becomes
  eligible only after content hash, schema/version, bounds, dependencies and
  expected world/project revision validate.
- Runtime commits accepted chunk results at one declared fixed-stage boundary
  and in canonical order independent of worker count, completion order, I/O
  timing or cache warmth.
- Failure, stale result, duplicate/collision, corrupt bytes or capacity denial
  leaves the previous active chunk/world generation unchanged.
- Mandatory work must produce an explicit accepted result or typed failure;
  optional work may use only a bounded declared fallback.
- Logical charges, peak working set, process I/O, device ceilings and relevant
  timings are observed through current Performance V4 evidence. Allocation
  counters are not reintroduced.
- `game` and `headless` share validation and commit semantics. Renderer frame
  rate and presentation availability cannot select authoritative streaming
  state.

## What R3a may introduce

Only primitives demonstrated by the vertical may be made reusable: for
example, a bounded immutable chunk request/result, a private staging queue and
one canonical commit batch. Their visibility should remain private until a
second production consumer demonstrates a stable public contract.

R3a explicitly does not require:

- a universal scheduler or task taxonomy;
- hierarchical cancellation propagation;
- generic resource reservations, pins or leases;
- deterministic global eviction across unrelated resources;
- a pluggable I/O backend protocol;
- a migration framework or legacy format reader;
- speculative R4/R5 workloads.

## Acceptance of a future implementation

The vertical is complete when a real packaged chunk is fetched, decoded,
validated and committed through production paths; corrupt/stale/oversized and
completion-order permutations fail or converge as declared; Save/Load/Replay
and R2 gameplay/ledger roots remain correct; and focused `content-package`,
`persistence-replay` plus the affected performance scenario pass.

Until that implementation exists, this SPEC is not routing authority for
current performance, runtime or content work and has no independent
ProductCheck.
