# ADR-026: Deterministic work, compute-resource and streaming admission

| Поле | Значение |
|---|---|
| ID | ADR-026 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-07-24 |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-021](021-deterministic-population-residency-and-time-advance.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](030-product-first-development-and-lightweight-validation.md) |
| Заменяет | отсутствует |
| Заменён | частично [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md): unconsumed generic coordinator, job classes, cancellation tree, memory admission, pins/leases, global eviction, archive credits and persisted-work schemas are no longer current; immutable bounded inputs, no mutable access across async and revision-bound canonical commit remain Accepted |

## Контекст

Current architecture already has a deterministic system DAG, immutable task
completion, current/next cutoff, canonical task-result merge, atomic chunk
publication, compositional performance budget and deterministic world
deferral. Она не задавала единый request-side contract для:

- closed job criticality and queue admission;
- cancellation hierarchy and cancel/result race;
- logical memory charges and critical reserve;
- qualified compute-resource pins, leases and eviction;
- finite queue age/capacity;
- archive/decompression limits and I/O credits;
- exact behavior when required work cannot enter a bounded queue.

Без такого решения implementations могут выбирать work по arrival time,
сбрасывать обязательную задачу при overflow, считать native allocator/RSS
authoritative budget oracle, evict a pinned dependency, let the first completed
worker win, recursively expand an untrusted archive or partially publish
cancelled staging.

## Решение

### Authority and dependency direction

`JobCoordinatorState` is authoritative for generic job request admission,
cancellation-tree state, logical resource charging, compute-residency
coordination and deterministic commit routing. Immutable content/archive
contracts and `GenerationPublisher` control atomic content/chunk/save
publication. Each subsystem state remains authoritative for the meaning and
mutation of its own payload. `WorldServicesState` retains
population/calendar/tier/interest order; `PhysicalWorldState` retains pose and
motor state; presentation remains non-authoritative.

Dependency direction is:

```text
Runtime/budget/world contracts
  → ADR-026
  → downstream detailed job/resource specification
  → downstream world/content consumers
```

This ADR has no dependency on a downstream jobs specification or future world
partition specification. A downstream consumer may supply a canonical
source-plan order, but it cannot redefine request admission, completion merge,
resource criticality or failure semantics.

### One closed job classification

Every queued/delegated/cancellable/resource-charged unit maps to exactly one
engine-owned closed class:

1. authoritative computation;
2. required resource preparation;
3. optional resource preparation;
4. presentation-only work;
5. offline tool work.

Unknown class is rejected. A required parent cannot hide required children in
an optional class. Authoritative computation returns immutable deltas/proposals
to its subsystem transaction; it does not grant a worker direct mutable world
access. Required resource work is a prerequisite for a later commit, not a
second gameplay authority.

### Deterministic request admission, dispatch and result merge

Every job request has a deterministic nominal ID, owner, immutable canonical
input hash, precondition root, cancellation scope, source-plan
hash/revision/ordinal, logical due epoch, declared commit stage and complete
resource claim. A closed batch validates all bounds and revisions before
reservation, removes exact duplicates and sorts only by canonical logical
fields. Arrival time, worker identity/count, cache warmth and wall time are not
keys.

Dispatch may be parallel and implementation-specific. A worker receives owned
immutable data only. It cannot retain mutable ECS, world, RPG, Agent, Physical
or backend access across an async boundary.

Result admission reuses, without modification:

- SPEC-21 `TypedTaskOutcomeV1`;
- `CompletionSignalV1` and `CompletionAssignmentV1`;
- the linearizable current/next cutoff;
- the exact `(commit_stage, owner_id, request_id, result_hash)` merge;
- exact deduplication, `TASK_RESULT_COLLISION` and `STALE_REVISION`.

There is no second completion queue or result ledger. Completion order may
change readiness, but cannot choose a winner, reorder a declared dependency
group or select an authoritative fallback. The complete subsystem transaction
revalidates and commits at its declared stage, or publishes nothing.

### Deterministic cancellation tree

Every job belongs to one bounded cancellation scope; every non-root scope has
one parent. Cancellation is admitted at a logical barrier and propagates
parent-first, then in canonical child/job order. An uncommitted result in the
effective subtree cannot merge. Memory/resource reservations and private
staging release atomically with cancellation.

Committed work is not retroactively cancelled. A native worker panic,
operational timeout or cancellation signal is only a typed failure proposal.
Cancellation of authoritative/required work does not erase the subsystem due
record unless an independently validated domain transition makes it no longer
due. Otherwise the enclosing transaction/plan retains, blocks or fails with a
stable diagnostic.

### Logical memory budget

Every execution uses one content-addressed `MemoryBudgetProfile` with finite
host/device totals and finite per-subsystem pool soft/hard limits, critical
reserves, inflight charges, single charges, pins and leases. Each logical byte
is charged exactly once. Shared resident content is charged to its residency
record; jobs reserve access rather than multiplying that charge.

Admission uses canonical schema lengths, validated decoded sizes and checked
integer accounting. Native allocation size, allocator fragmentation, RSS,
VRAM telemetry and driver behavior are diagnostic telemetry only and cannot
select authoritative admission/eviction. Unexpected physical allocation
failure returns a typed failure and class-specific fallback before
publication.

Per-job/per-resource limits compose inside existing mutually exclusive
ADR-016 subsystem rows. They cannot create additional tick budget or multiply the
8,000/12,000 microsecond integrated ceiling.

### Qualified pins, leases and eviction

Compute-resource identity includes a qualified kind/logical key, exact content
hash and target-profile hash; no unqualified native resource handle is public.
A pin has one owner, bounded purpose and explicit release precondition. A lease
has one owner and finite logical expiry; wall time cannot renew or expire it.

Eviction removes reconstructible bytes only. It never evicts:

- authoritative or dirty subsystem state;
- a live pin or unexpired lease;
- an in-flight authoritative input/result;
- an atomic publication staging generation;
- an unresolved required dependency;
- the only valid content/save generation.

Eligible candidates use one canonical logical order. When nothing is eligible,
optional work is rejected and required work retains its prior active state and
blocks/fails its plan. Compute-resource residency does not change
`WorldResidencyTier`, physical LOD, durable membership or gameplay state.

### Finite queues and no authoritative drop

Every job and I/O queue has positive finite item, byte and logical-age bounds
fixed by an exact profile. Queue pressure has class-specific deterministic
behavior:

- authoritative due work remains in the owning due set with its original age;
- required resource work retains the prior active generation and unmet
  prerequisite;
- optional and presentation work may be canonically rejected, preempted,
  coalesced or cancelled only with an explicit receipt and declared fallback;
- offline tool work aborts without partial artifact publication.

Maximum logical age is not a drop deadline. Exceeding it fails the run,
transaction, load or transition with a stable starvation diagnostic while
preserving the due record/prior state. Resubmission cannot reset age. Wall
watchdogs may abort an uncommitted operation and report a performance
diagnostic, but never choose partial authoritative success.

### Bounded archive/decompression and I/O credits

The exact profile bounds job inputs/result manifests, archive input, entry
count/path/metadata, compressed and uncompressed entry sizes, aggregate
decoded size, expansion ratio, automatic nesting depth, decode chunk size,
queued/inflight bytes, requests and ready results.

All bounds are checked before allocation with checked integer arithmetic.
Untrusted decode writes only private staging. Required strong dependency
groups reserve all logical I/O, decode and memory credits atomically and admit
as a whole or not at all. Logical credits release only at a deterministic
completion/cancellation barrier; faster physical completion cannot admit a
later request first.

Large bytes remain in private content-addressed staging. The public completion
contains a bounded engine-owned reference/receipt and hashes, never raw paths
or native I/O/archive objects. Exact length, content/schema/dependency hashes
and preconditions validate before deterministic publication. Error or
cancellation discards the entire staging generation.

Gameplay handlers never block on I/O. Required content is staged and pinned
before authoritative activation; otherwise the previous state remains and the
owning plan records a deterministic defer/block. Optional quality may degrade
only through a predeclared fallback.

### Persistence, replay and parity

Profile hashes that affect job/resource decisions are fixed in exact project or
tool-run compatibility data and carried into save/replay/run metadata where
applicable. Checkpoints persist logical requests/receipts, subsystem due
age, cancellation scopes, reservations, pins/leases, residency generations
and profile hashes—not native worker, allocator, I/O or decode state.

Restart discards private staging and reconstructs pending work with the same
IDs, input hashes and age. Replay records closed request/resource admission,
cancellation barriers, SPEC-21 completion assignments, eviction plans and
publication generations. `game`, deterministic `headless` and
`capture-worker` share the same semantics.

## Рассмотренные варианты

- One global unbounded thread pool/queue — `Rejected`: no finite admission,
  age, subsystem accounting or restart contract.
- Use worker completion or wall deadline as commit order — `Rejected`: load and
  host scheduling would select authoritative state.
- Drop oldest/lowest authoritative work on overflow — `Rejected`: loses a
  mandatory outcome and causal age.
- Treat every resource as optional cache — `Rejected`: permits eviction of
  required inputs and only-valid generations.
- Use RSS/allocator failure as deterministic budget branch — `Rejected`:
  allocation layout and driver accounting are platform-dependent.
- Cancel a parent after publishing successful children — `Rejected`: creates
  partial authoritative state.
- Stream/decompress directly into active state — `Rejected`: corrupt,
  oversized or cancelled input could partially publish.
- Let each subsystem invent queue/backpressure semantics — `Rejected`: breaks
  integrated accounting and composition-root parity.
- Extend SPEC-21 with a second completion/result protocol — `Rejected`: creates
  conflicting cutoff, collision and replay authority.

## Последствия

- Runtime gains a single request/resource admission and cancellation boundary
  while retaining the accepted SPEC-21 completion path.
- Asset/world consumers can submit canonical plans and immutable work without
  exposing source formats, filesystem state or backend objects.
- Correctness may block a load/transition or fail a run when required capacity
  is unavailable; it cannot be silently traded for a fabricated outcome.
- Optional quality remains degradable through explicit bounded fallback.
- Memory, queue, I/O and residency traces remain useful diagnostics and compose
  with ADR-016 rather than replacing it.
- No technology row is accepted by this decision.

## Product checks

| Scenario | Expected | Fallback |
|---|---|---|
| Arrival, worker-count, completion and cancel/result permutations for one closed request batch | Admission, completion assignment, merge order, cancellation and publication are exact; no worker retains mutable world access | Keep the due record and prior atomic state; serialize execution behind the same contract if needed |
| Queue, memory, pin, lease and eviction boundary cases | Required or authoritative work is never silently dropped and pinned, dirty or only-valid state is never evicted | Reject optional work, retain the active generation and defer or block the enclosing plan |
| Oversized, nested, corrupt or cancelled archive/I/O input followed by restart | Bounds are checked before allocation; private staging never partially publishes; pending work reconstructs with the same IDs and logical age | Discard staging and retain the previous content/save generation |

## Supersession

Changing the closed job classes, allowing arrival/wall/worker order to choose
authoritative state, creating a second completion path, permitting silent
authoritative drop, using native allocation state as an authority, evicting
pinned/required/dirty state, allowing non-atomic cancellation/publication or
moving world/domain ownership into the resource manager requires a new ADR
that explicitly supersedes ADR-026 and updates dependent specifications.
