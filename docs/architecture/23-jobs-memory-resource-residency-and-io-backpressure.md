# SPEC-23: Future generic jobs and resource work

| Поле | Значение |
|---|---|
| ID | SPEC-23 |
| Статус | Proposed |
| Версия | 2.2 |
| Последняя проверка | 2026-08-09 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-051](adr/051-r3a-packaged-chunk-streaming-commit-boundary.md) |
| Заменяет | SPEC-23 version 1.9 generic scheduler/resource architecture; deferred by ADR-046 |

## Current status after bounded R3

ADR-051 accepts exactly one consumer-driven vertical:

```text
chunk fetch → decode → validate → canonical commit
```

The completed four-region/64-chunk project travels through production
Assets/World/Runtime boundaries while preserving the gameplay result. Its
pinned content generation, bounded private workers, immutable opaque result
and paired fixed-stage commit remain implementation-local R3 evidence governed
by SPEC-03/SPEC-25/ADR-051.

This SPEC remains `Proposed`. It does not promote those private primitives into
a general job system, cancellation tree, pin/lease model, eviction framework or
public resource-scheduler API, and it is not routing authority for current R3.

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

## What a future consumer may justify

Only primitives demonstrated by more than this single vertical may be made
reusable. The R3 bounded immutable chunk request/result, private staging queue
and canonical commit batch remain private until another production consumer
demonstrates a stable public contract.

R3 did not introduce and future work must not assume:

- a universal scheduler or task taxonomy;
- hierarchical cancellation propagation;
- generic resource reservations, pins or leases;
- deterministic global eviction across unrelated resources;
- a pluggable I/O backend protocol;
- a migration framework or legacy format reader;
- speculative R4/R5 workloads.

## Promotion condition

This SPEC may become Accepted only when a second production consumer requires a
shared scheduler/resource contract and its concrete semantics, fallback,
budgets and ProductCheck are known. Completed R3b reused the existing private
R3a path and did not demonstrate or accept that generic need.

Until promotion, this SPEC is not routing authority for current performance,
runtime or content work and has no independent ProductCheck.
