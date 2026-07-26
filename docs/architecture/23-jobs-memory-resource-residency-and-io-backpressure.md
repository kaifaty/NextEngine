# SPEC-23: Jobs, memory, compute-resource residency and I/O backpressure

| Поле | Значение |
|---|---|
| ID | SPEC-23 |
| Статус | Accepted |
| Версия | 1.3 |
| Последняя проверка | 2026-07-26 |
| Нормативные зависимости | [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-021](adr/021-deterministic-population-residency-and-time-advance.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-026](adr/026-deterministic-work-resource-and-streaming-admission.md) |
| Заменяет | отсутствует |

## Назначение и invariants

SPEC-23 определяет bounded production substrate для delegated work и
reconstructible compute resources. Он дополняет, но не заменяет command
admission, stable schedule и completion merge SPEC-21.

- Любая queued, delegated, cancellable или resource-charged runtime work unit
  MUST иметь ровно один closed `JobClassV1`, engine-owned identity, immutable
  input hash, owner, logical order key, resource claim and cancellation scope.
- `AuthoritativeCompute` и `RequiredResource` work MUST NOT silently drop,
  overwrite, coalesce or lose age. Queue pressure retains the exact due record
  at its authoritative owner or blocks the enclosing plan before mutation.
- Wall time, worker count, worker identity, allocator layout, cache warmth,
  I/O completion order and observed duration MAY be measured for conformance,
  but MUST NOT select an authoritative result, merge order, eviction, retry
  target, command or domain outcome.
- Dispatch MAY be parallel. Inputs remain immutable; no mutable ECS, world,
  RPG, Agent, Physical or backend access crosses an async boundary.
- Completion MUST use existing SPEC-21 `CompletionSignalV1`,
  `CompletionAssignmentV1`, current/next cutoff and `TaskMergeProfileV1`.
  A second completion queue, arrival-order merge or “first result wins” path is
  forbidden.
- Cancellation is a deterministic tree operation. It cannot publish a partial
  authoritative delta, event, resource generation or bundle.
- Every queue has finite item, charged-byte and logical-age bounds. Exceeding a
  bound produces a stable decision; it never grows an unbounded hidden queue.
- Memory decisions use canonical logical charges from a hash-bound
  `MemoryBudgetProfileV1`. Measured RSS/VRAM is telemetry, never an authoritative
  allocation or eviction oracle.
- Pins and leases protect only qualified `ComputeResourceKeyV1` values.
  Compute-resource residency is a reconstructible runtime/cache concern and is
  distinct from `WorldResidencyTier`, Runtime entity residency and physical
  LOD.
- Archives, compressed payloads, requests and completion manifests are bounded
  before allocation. Required publication remains atomic and fail-closed.
- `game`, deterministic `headless` and `capture-worker` MUST use the same job,
  budget, residency, cancellation, staging and commit contracts. This document
  does not define capture-farm scheduling, worker-network or CI queue protocol.

## Source of truth и exact ownership

| State / decision | Единственный owner/source of truth | Allowed projection / forbidden duplicate |
|---|---|---|
| Job classes, queue profiles, request admission, cancellation-tree state, logical age and deterministic commit coordination | Runtime subsystem | immutable queue/job traces; no subsystem-private competing scheduler semantics |
| Completion cutoff, assignment, task-result hashing, conflict/stale policy and canonical merge | Runtime subsystem through SPEC-21 | SPEC-23 request record; no second result ledger or arrival-order merge |
| Authoritative payload state and whether due work remains mandatory | Owning subsystem named by `owner_id` | immutable input/revision view; Runtime cannot discard or fabricate owner outcome |
| Immutable content, bundle/archive schema, content hashes and chunk publication state | Asset & Persistence subsystem | compute-residency record/cache; no alternate content authority |
| World calendar, population tier, world-interest rank and mandatory world outcome | World Services subsystem | resource request/source-plan ordinal; compute-resource pressure cannot rewrite tier or cursor |
| Physical pose, contacts, motor route and required physical assets | Physical Embodiment subsystem | immutable claim/pin; resource manager cannot choose physical outcome |
| Logical memory charge, pool reservation, compute-resource pins/leases and eviction plan | Runtime subsystem | private allocator/device/cache statistics; no backend allocation object as authority |
| Presentation cache quality and optional presentation work | Presentation owner | immutable gameplay snapshot only; no presentation pin creates gameplay authority |
| Integrated owner-row time budget | Exact ADR-016 `GameplayBudgetMatrix` | per-job spans charged exactly once; no multiplicative per-job budget |

Runtime owns admission mechanics, not domain meaning. A job result remains an
immutable proposal until the owning subsystem revalidates revisions and the
common transaction commits it at a declared stage.

## Public boundary

Public schemas MAY contain only engine-owned nominal IDs, fixed-width integers,
canonical bytes/hashes, immutable manifests, bounded typed values and stable
diagnostics. They MUST NOT contain ECS storage/components, `RuntimeEntityId`,
raw pointers, raw allocator objects, task/future/thread/worker handles, OS I/O
or file handles, window/device objects, database connections, vendor/backend
objects, importer structures or native archive-library values.

Public names are qualified. `JobId`, `CancellationScopeId`, `MemoryPoolId` and
`ComputeResourceKeyV1` are nominal engine values; none is a native handle.
Unqualified `resource ID`, pointer-like identity and process-local address are
forbidden.

## Closed job and admission contracts

### `JobClassV1`

```text
JobClassV1 =
  0x01 AuthoritativeCompute
  | 0x02 RequiredResource
  | 0x03 OptionalResource
  | 0x04 PresentationOnly
  | 0x05 OfflineTool
```

| Class | Contract | Queue-pressure result |
|---|---|---|
| `AuthoritativeCompute` | Immutable fixed-stage/shard computation whose validated result may contribute to one owner transaction. It never mutates shared state from a worker. | Retain the exact due record at the owner and deterministically retry, or fail/abort the enclosing transaction/run at maximum logical age; never drop or commit a partial subset. |
| `RequiredResource` | Fetch/decode/validate work required before a declared load, activation, transition, recovery or save operation may commit. It is not itself gameplay authority. | Retain prior active generation and the exact unmet prerequisite; backpressure or fail the enclosing plan before mutation. |
| `OptionalResource` | Prefetch or quality input with an explicit content/profile fallback. | Canonically reject, preempt or cancel with a receipt; fallback may run only when already declared. |
| `PresentationOnly` | Work over immutable presentation inputs that cannot change gameplay. | Canonically coalesce by declared semantic key, reject or cancel; authoritative state remains unchanged. |
| `OfflineTool` | Cook/validate/inspect/package work outside a live mutable world. | Abort the atomic tool operation and preserve the previous published artifact. |

Unknown tags fail closed. A job class cannot change after admission. Splitting
one required unit into optional children does not weaken its parent
criticality: every child needed for parent success inherits the stronger class.

### Job identity, immutable input and logical order

```text
LogicalAgeDomainV1 =
  0x01 SimulationTick
  | 0x02 WorldTick
  | 0x03 ToolOperationOrdinal

PinPurposeV1 =
  0x01 AuthoritativeOwnerState
  | 0x02 RequiredTransition
  | 0x03 InFlightImmutableInput
  | 0x04 AtomicPublication

ImmutableInputReferenceV1 {
  1 input_kind: NamespacedId,
  2 logical_key: CanonicalBinary,
  3 schema_id: SchemaId,
  4 schema_version: u32,
  5 content_hash: Hash256,
  6 canonical_length_bytes: u64,
}

ResourceClaimV1 {
  1 pool_id: MemoryPoolId,
  2 compute_resource_key: Option<ComputeResourceKeyV1>,
  3 charged_bytes: u64,
  4 maximum_inflight_bytes: u64,
  5 maximum_decoded_bytes: u64,
  6 required_pin_purpose: Option<PinPurposeV1>,
  7 requested_lease_expiry_epoch: Option<u64>,
}

JobOrderKeyV1 {
  1 logical_age_domain: LogicalAgeDomainV1,
  2 logical_due_epoch: u64,
  3 source_plan_id: Id128,
  4 source_plan_hash: Hash256,
  5 source_plan_revision: u64,
  6 plan_ordinal: u64,
  7 owner_id: OwnerId,
  8 request_id: Id128,
  9 input_hash: Hash256,
}

JobRequestV1 {
  1 schema_version: u16 = 1,
  2 execution_namespace: Id128,
  3 job_id: JobId,
  4 class: JobClassV1,
  5 owner_id: OwnerId,
  6 request_id: Id128,
  7 cancellation_scope_id: CancellationScopeId,
  8 order_key: JobOrderKeyV1,
  9 commit_stage: RuntimeStageId,
  10 input_schema_id: SchemaId,
  11 input_schema_version: u32,
  12 inline_input: Option<CanonicalBinary>,
  13 input_references: CanonicalSet<ImmutableInputReferenceV1>,
  14 input_hash: Hash256,
  15 precondition_revision_root: Hash256,
  16 resource_claims: CanonicalSet<ResourceClaimV1>,
}
```

Exactly one canonical input closure is hashed. Inline bytes and referenced
immutable objects are ordered by schema and qualified key; referenced bytes
are addressed by exact hash. Mutable service/session state is not input.
`input_hash` covers schema/version, inline bytes, ordered references,
precondition root and resource claims.

`job_id` is recomputed before admission:

```text
job_id = left128(SHA256(
  "nextengine.job-id.v1\0"
  || execution_namespace
  || lp(owner_id)
  || request_id
  || input_hash
))
```

A claimed mismatch is `JOB_ID_MISMATCH`. The same ID with different canonical
bytes is `JOB_ID_COLLISION`; no alternate ID is selected. `source_plan_id`,
hash, revision and ordinal preserve an upstream owning plan's canonical order.
SPEC-23 does not reinterpret world-interest or domain rank.

### `JobAdmissionLimitsV1`

Conforming v1 uses this exact maximum profile:

```text
JobAdmissionLimitsV1 {
  1 schema_version: u16 = 1,
  2 max_job_request_bytes: u32 = 524288,
  3 max_inline_input_bytes: u32 = 262144,
  4 max_input_references: u32 = 256,
  5 max_resource_claims_per_job: u32 = 64,
  6 max_jobs_per_closed_batch: u32 = 4096,
  7 max_cancellation_scopes_per_closed_batch: u32 = 4096,
  8 max_children_per_cancellation_scope: u32 = 4096,
  9 max_jobs_per_cancellation_scope: u32 = 4096,
  10 max_cancellation_depth: u16 = 64,
  11 max_result_manifest_bytes: u32 = 4194304,
}
```

`max_result_manifest_bytes` MUST equal SPEC-21
`RuntimeAdmissionLimitsV1.max_task_result_bytes`. Large decoded resources stay
in private immutable staging; the public completion carries only a bounded
engine-owned receipt/reference plus hashes. A project MAY lower a limit before
creating an execution namespace. Increasing one requires a new named profile
and repetition of security/resource checks. Every boundary is checked with
checked arithmetic before allocation or queue mutation.

### Queue profile and closed admission

```text
WorkQueueDescriptorV1 {
  1 queue_id: NamespacedId,
  2 job_class: JobClassV1,
  3 logical_age_domain: LogicalAgeDomainV1,
  4 max_queued_items: u32,
  5 max_queued_charge_bytes: u64,
  6 max_logical_age: u64,
}

WorkQueueProfileV1 {
  1 schema_version: u16 = 1,
  2 queues: CanonicalMap<NamespacedId, WorkQueueDescriptorV1>,
  3 admission_limits_hash: Hash256,
  4 memory_budget_profile_hash: Hash256,
  5 io_backpressure_profile_hash: Hash256,
}

ComputeResourcePolicyV1 {
  1 schema_version: u16 = 1,
  2 policy_id: NamespacedId,
  3 policy_revision: u64,
  4 job_admission_limits_hash: Hash256,
  5 work_queue_profile_hash: Hash256,
  6 memory_budget_profile_hash: Hash256,
  7 archive_decompression_limits_hash: Hash256,
  8 io_backpressure_profile_hash: Hash256,
  9 residency_accounting_schema_hash: Hash256,
  10 eviction_profile_id: NamespacedId,
}
```

```text
JobAdmissionDecisionV1 =
  0x01 Admitted
  | 0x02 DeferredAtOwner
  | 0x03 RequiredAdmissionBlocked
  | 0x04 RejectedOptional
  | 0x05 CoalescedPresentation
  | 0x06 ToolOperationBlocked
  | 0x07 RejectedInvalid

JobAdmissionReceiptV1 {
  1 job_id: JobId,
  2 owner_id: OwnerId,
  3 request_id: Id128,
  4 input_hash: Hash256,
  5 queue_id: NamespacedId,
  6 decision: JobAdmissionDecisionV1,
  7 diagnostic_code: Option<StableDiagnosticCode>,
  8 first_due_epoch: u64,
  9 logical_age: u64,
  10 reservation_root: Option<Hash256>,
}

ClosedJobAdmissionBatchV1 {
  1 schema_version: u16 = 1,
  2 queue_generation: u64,
  3 logical_age_domain: LogicalAgeDomainV1,
  4 logical_epoch: u64,
  5 compute_resource_policy_hash: Hash256,
  6 work_queue_profile_hash: Hash256,
  7 memory_budget_profile_hash: Hash256,
  8 io_backpressure_profile_hash: Hash256,
  9 requests: Vec<JobRequestV1>,
  10 decisions: Vec<JobAdmissionReceiptV1>,
  11 batch_hash: Hash256,
}
```

`batch_hash` is computed over canonical fields 1…10 with a
`nextengine.closed-job-admission-batch.v1` domain; it is not included in its
own preimage. Requests and decisions have equal length and matching ordinals.
`logical_age = logical_epoch - first_due_epoch` uses checked arithmetic in the
same declared domain. `first_due_epoch` survives defer/restart/resubmission.
Queue generation starts at zero, advances exactly once per published closed
batch and never wraps; exhaustion rejects the next close before reservation.

Every `max_*` value is positive, finite and fixed before execution. Default v1
queue instances are:

| Job class | `max_queued_items` | `max_queued_charge_bytes` | `max_logical_age` / domain |
|---|---:|---:|---|
| `AuthoritativeCompute` | 4,096 | 268,435,456 | 120 `SimulationTick` |
| `RequiredResource` | 1,024 | 536,870,912 | 600 `SimulationTick` or `WorldTick`, chosen by the owning plan |
| `OptionalResource` | 2,048 | 536,870,912 | 300 in its declared logical domain |
| `PresentationOnly` | 512 | 268,435,456 | 4 `SimulationTick` |
| `OfflineTool` | 1,024 | 536,870,912 | 10,000 `ToolOperationOrdinal` |

An implementation MAY use a lower-concurrency dispatch, but it MUST preserve
these logical queue decisions. A different capacity/age profile is
content-addressed, fixed in `ProjectCompositionLock` or exact tool run manifest
and repeats all applicable product checks.

At a declared admission barrier Runtime closes one batch, validates all bounds
and source-plan revisions, deduplicates exact requests and sorts by:

```text
(
  logical_age_domain,
  logical_due_epoch,
  job_class,
  source_plan_id,
  plan_ordinal,
  owner_id,
  request_id,
  input_hash,
  job_id
)
```

`ClosedJobAdmissionBatchV1` records queue generation, logical epoch, exact
profile hashes, sorted requests, per-request decisions and batch hash. A
duplicate order/identity key with unequal bytes rejects the equivalence class.
Queue/memory/I/O reservations are computed over the complete sorted batch
before publication. Worker availability or completion readiness is not an
admission key.

Admission has only these class-specific overflow actions:

- `AuthoritativeCompute`: `DeferredAtOwner`; the owner due-work record, first
  due epoch and age remain unchanged. At maximum age the run fails
  `AUTHORITATIVE_WORK_STARVATION` before dropping work.
- `RequiredResource`: `RequiredAdmissionBlocked`; prior active generation and
  source plan remain unchanged. At maximum age the enclosing load/transition
  fails closed as `REQUIRED_RESOURCE_STARVATION`.
- `OptionalResource`: `RejectedOptional` or canonical preemption of a lower
  ranked optional request with an explicit receipt.
- `PresentationOnly`: declared semantic-key coalescing, rejection or
  cancellation; it changes zero authoritative bytes.
- `OfflineTool`: `ToolOperationBlocked`; staging is discarded and the prior
  published artifact remains.

No queue policy may transform an authoritative/required item into an optional
one, reset its age by resubmission or treat a missing receipt as successful
completion.

## Dispatch, completion and canonical commit

Dispatch order and worker count are private. Each job receives owned immutable
inputs and a cancellation observation value. It returns no direct state
reference. `AuthoritativeCompute` logical shards follow SPEC-21
`ScheduleManifestV1`; all shards complete or the owner transaction aborts.

Completion uses SPEC-21 without a competing schema:

1. Encode success/failure as `TypedTaskOutcomeV1`.
2. Recompute `result_hash` and validate the existing 4,194,304-byte bound.
3. Publish `CompletionSignalV1` through the current/next close barrier and
   persist `CompletionAssignmentV1`.
4. Resolve the admitted `job_id` from `(owner_id, request_id, input_hash)` and
   validate class, source-plan/precondition revisions, cancellation scope and
   declared resource charge.
5. Merge only at the declared commit stage in exact
   `(commit_stage, owner_id, request_id, result_hash)` order.
6. Exact duplicates deduplicate; conflicting hashes use existing
   `TASK_RESULT_COLLISION`; stale preconditions use `STALE_REVISION`.
7. Revalidate the complete owner transaction and atomically publish all
   deltas/events/resource generation, or publish none.

Async completion may report readiness but cannot choose among competing
results, reorder a strong dependency group or select an authoritative
fallback. A required group commits only when every declared member is valid;
otherwise the previous active generation remains. A wall watchdog MAY record a
performance/nonconformance fault and abort an uncommitted operation, but it
cannot select a partial result or turn a failed run into `PASS`.

## Cancellation tree

```text
CancellationStateV1 =
  0x01 Open
  | 0x02 CancelPending
  | 0x03 Cancelled
  | 0x04 Closed

CancellationScopeV1 {
  1 schema_version: u16 = 1,
  2 scope_id: CancellationScopeId,
  3 parent_scope_id: Option<CancellationScopeId>,
  4 owner_id: OwnerId,
  5 state: CancellationStateV1,
  6 effective_age_domain: LogicalAgeDomainV1,
  7 effective_epoch: u64,
  8 reason_code: Option<StableDiagnosticCode>,
  9 child_scope_ids: CanonicalSet<CancellationScopeId>,
  10 job_ids: CanonicalSet<JobId>,
}
```

Every job belongs to exactly one scope and every non-root scope has exactly one
parent. Cycles, depth/child overflow, cross-owner mutation or duplicate parent
reject the complete tree update. Cancel request becomes effective only at its
recorded logical barrier. At the same barrier cancel is applied parent-first,
then children by `(tree_depth, scope_id, job_id)`; an uncommitted completion in
the cancelled subtree is rejected before merge.

Cancellation is transactional:

- no child result, memory reservation, pin, resource generation or
  `DomainEvent` is partially published;
- committed work is not retroactively cancelled;
- private staging is discarded as a whole and logical reservations release at
  the same deterministic barrier;
- cancellation of optional/presentation work may be terminal;
- cancellation of authoritative/required work does not erase the owner due
  record. It either follows an already validated domain cancellation that
  makes the work no longer due, or retains/blocks/fails the enclosing operation
  with a stable diagnostic;
- worker panic or a native cancellation signal is merely a typed failure
  proposal and cannot mutate the cancellation tree outside this path.

## `MemoryBudgetProfileV1` and logical charging

```text
MemoryPoolClassV1 =
  0x01 AuthoritativeState
  | 0x02 RequiredStaging
  | 0x03 ReconstructibleHostCache
  | 0x04 ReconstructibleDeviceCache
  | 0x05 PresentationTransient
  | 0x06 ToolingTransient

MemoryPoolBudgetV1 {
  1 pool_id: MemoryPoolId,
  2 owner_id: OwnerId,
  3 class: MemoryPoolClassV1,
  4 soft_limit_bytes: u64,
  5 hard_limit_bytes: u64,
  6 critical_reserve_bytes: u64,
  7 max_single_charge_bytes: u64,
  8 max_inflight_charge_bytes: u64,
  9 max_pin_count: u32,
  10 max_lease_count: u32,
}

MemoryBudgetProfileV1 {
  1 schema_version: u16 = 1,
  2 profile_id: NamespacedId,
  3 profile_revision: u64,
  4 host_logical_hard_limit_bytes: u64,
  5 device_logical_hard_limit_bytes: u64,
  6 pools: CanonicalMap<MemoryPoolId, MemoryPoolBudgetV1>,
  7 accounting_schema_hash: Hash256,
}
```

All limits are finite. For each pool
`soft_limit <= hard_limit`, `critical_reserve <= hard_limit` and single/inflight
limits cannot exceed its hard limit. Checked sums of mutually exclusive host
and device pools cannot exceed their totals. Optional and presentation work
cannot borrow `AuthoritativeState` or `RequiredStaging` reserve.

`ResourceClaimV1` assigns every logical byte to exactly one pool and records
host/device/staging/decompressed charges plus the qualified compute-resource
key when shared. Admission reserves the full declared charge before dispatch.
Shared resident content is charged once in its residency record; jobs acquire a
pin/lease/reservation rather than multiplying the content charge.

Canonical logical charge comes from validated schema lengths, decoded-size
headers and profile accounting rules. Backend allocation size, pointer layout,
allocator fragmentation, resident-set sampling and device-driver accounting
are telemetry only. If physical allocation fails despite an admitted logical
charge, the operation returns a typed failure and follows its class fallback;
it cannot choose a different authoritative result. If an actual validated
payload exceeds its reserved charge, the whole result is rejected as
`RESOURCE_CHARGE_MISMATCH` before publication.

The `resource-reference-v1` profile retains the canonical
scene ceilings of 12,884,901,888 host-resident bytes (12 GiB) and
5,905,580,032 device-resident bytes (5.5 GiB). These measured ceilings are product-check
thresholds, not authoritative eviction inputs.

Every job span is charged exactly once to an existing mutually exclusive
ADR-016 `GameplayBudgetMatrix` row. Per-job, per-resource and per-entity
budgets do not multiply or alter the integrated 8,000/12,000 microsecond
contract.

## Qualified compute-resource residency, pins, leases and eviction

```text
ComputeResourceKeyV1 {
  1 resource_kind: NamespacedId,
  2 logical_key: CanonicalBinary,
  3 content_hash: Hash256,
  4 target_profile_hash: Hash256,
}

ComputeResidencyStateV1 =
  0x01 Absent
  | 0x02 Reserved
  | 0x03 Staged
  | 0x04 Validated
  | 0x05 Resident
  | 0x06 Quiescing
  | 0x07 Evicted
  | 0x08 Failed

EvictionClassV1 =
  0x01 PresentationTransient
  | 0x02 OptionalReconstructible
  | 0x03 ToolingReconstructible
  | 0x04 RequiredIdleReconstructible

ComputeResourceResidencyV1 {
  1 schema_version: u16 = 1,
  2 key: ComputeResourceKeyV1,
  3 owner_id: OwnerId,
  4 pool_id: MemoryPoolId,
  5 state: ComputeResidencyStateV1,
  6 charged_bytes: u64,
  7 residency_generation: u64,
  8 source_revision: u64,
  9 last_logical_use_epoch: u64,
  10 pin_ids: CanonicalSet<ResourcePinId>,
  11 lease_ids: CanonicalSet<ResourceLeaseId>,
}
```

This lifecycle describes compute bytes/cache residency only. It does not
replace the SPEC-03 `WorldChunk` lifecycle, own `WorldResidencyTier`, create a
`RuntimeEntityId`, tombstone a durable object or write physical pose.

`ComputeResourcePinV1` contains qualified key, nominal pin ID, owner, closed
purpose (`AuthoritativeOwnerState`, `RequiredTransition`,
`InFlightImmutableInput` or `AtomicPublication`), acquire
epoch and explicit release precondition. Pins are admitted atomically under
the profile's finite count/byte bounds. They do not expire implicitly.

`ComputeResourceLeaseV1` contains qualified key, nominal lease ID, owner,
purpose, grant epoch, strictly greater logical expiry epoch, charge and bounded
renewal count. Renewal is a new validated decision at a deterministic barrier;
wall time cannot extend or expire a lease.

An eviction plan captures exact budget/profile/source revisions, current
residency generation, admitted claims, pins, leases, canonical candidates,
evictions, deferrals and plan hash. Eviction is allowed only for
reconstructible bytes with no live pin, unexpired lease, dirty owner state,
in-flight authoritative input/result, atomic publication or unresolved strong
dependency. Candidate order is:

```text
(
  eviction_class,
  lease_expiry_epoch,
  last_logical_use_epoch,
  canonical ComputeResourceKeyV1 bytes
)
```

Required admission first reclaims lower-class optional/presentation candidates
in that order. If no eligible candidate exists, optional work rejects and
required work retains its prior generation and blocks/fails its plan. Eviction
never emits a gameplay `DomainEvent`, changes a world tier, deletes durable
state or invalidates the only valid save/content generation.

## Archive, decompression and I/O backpressure

### Exact archive/decompression limits

```text
ArchiveDecompressionLimitsV1 {
  1 schema_version: u16 = 1,
  2 max_archive_input_bytes: u64 = 4294967296,
  3 max_entry_count: u32 = 131072,
  4 max_path_bytes: u32 = 4096,
  5 max_metadata_bytes: u64 = 16777216,
  6 max_single_compressed_entry_bytes: u64 = 1073741824,
  7 max_single_uncompressed_entry_bytes: u64 = 2147483648,
  8 max_total_uncompressed_bytes: u64 = 8589934592,
  9 max_expansion_ratio_numerator: u32 = 256,
  10 max_expansion_ratio_denominator: u32 = 1,
  11 max_nested_auto_expansion_depth: u16 = 0,
  12 max_decode_chunk_bytes: u32 = 16777216,
}
```

All size/count sums and ratio comparisons use checked integer arithmetic before
allocation. Runtime automatic nested archive expansion is forbidden; an
archive-shaped payload may remain opaque only when its declared schema does
not request expansion. Paths use the canonical relative-path rules ADR-022.
An entry with zero compressed bytes MUST also have zero decoded bytes; every
non-empty entry MUST satisfy
`decoded_bytes * ratio_denominator <= compressed_bytes * ratio_numerator`
without overflow.
Duplicate/case-fold-colliding paths, link/device entries, trailing data,
unknown required compression/schema, length/hash mismatch or limit overflow
reject the whole staged archive.

### `IoBackpressureProfileV1`

```text
IoBackpressureProfileV1 {
  1 schema_version: u16 = 1,
  2 profile_id: NamespacedId,
  3 profile_revision: u64,
  4 max_queued_requests: u32,
  5 max_queued_input_bytes: u64,
  6 max_inflight_requests: u32,
  7 max_inflight_input_bytes: u64,
  8 max_inflight_output_bytes: u64,
  9 max_ready_results: u32,
  10 max_ready_result_manifest_bytes: u64,
  11 max_logical_queue_age: u64,
  12 archive_limits_hash: Hash256,
  13 memory_budget_profile_hash: Hash256,
}
```

Every value is positive and finite; ready-result manifest bytes also obey the
SPEC-21 4,194,304-byte per-result limit. An immutable I/O request names
`AssetId`/qualified content hash, canonical byte range, expected compressed and
decoded lengths, schema/version, source-plan ordinal, class and preconditions.
Raw paths and native I/O objects remain adapter-private.

Backpressure uses logical credits:

1. Close and canonically sort one request batch using the owning source-plan
   order plus `JobOrderKeyV1`.
2. Validate input/archive/decode bounds and reserve queued, inflight,
   decompressed and memory charges for the complete strong dependency group.
3. Admit a whole required group or none. Optional/presentation requests may be
   canonically rejected/preempted according to their class.
4. Dispatch may complete in any physical order. Logical credits are released
   only when the corresponding completion/cancellation is processed at its
   deterministic barrier; faster worker completion cannot admit a later
   request first.
5. Decode into private staging in bounded chunks. Validate exact total length,
   content/schema/dependency hashes and preconditions before producing the
   bounded `CompletionSignalV1`.
6. Publish a validated generation only at the owning deterministic commit
   point. Cancellation/error discards the complete staging generation.

A gameplay command handler never blocks on I/O. Required bytes are staged and
pinned before the authoritative activation plan commits. If they are not ready,
the prior active state remains and the owning plan records one deterministic
defer/block; the runtime does not choose a substitute based on which request
finishes first. Optional work may reduce quality only through its already
declared fallback.

## Persistence, replay and composition-root parity

One `ComputeResourcePolicyV1` binds the exact hashes of
`JobAdmissionLimitsV1`, `WorkQueueProfileV1`, `MemoryBudgetProfileV1`,
compute-residency accounting policy, `ArchiveDecompressionLimitsV1` and
`IoBackpressureProfileV1`. Its revision and root hash are fixed in
`ProjectCompositionLock` or the exact offline tool run manifest. When the
policy can affect authoritative readiness/admission, its hash is also
compatibility data in build/save/replay/root `RunManifest` and capture job.

An atomic checkpoint stores only canonical job admission batches/receipts,
pending owner due records and first-due epochs, cancellation-tree state,
logical reservations/pins/leases/residency generations and relevant profile
hashes. It never serializes worker/native task state, allocator state, open I/O
objects or partial decode buffers. On restart private staging is discarded and
pending work is reconstructed with the same job/request/input IDs and age.

Replay records closed job and resource-admission decisions, cancellation
barriers, SPEC-21 completion assignments, result hashes, residency/eviction
plans and publication generations. Worker/cache/I/O timing is diagnostic only.
`game`, `headless` and `capture-worker` given the same closed inputs and
assignments MUST produce exact admission, cancellation, merge, eviction,
publication, event and authoritative state roots.

## Stable diagnostics

| Code | Required outcome |
|---|---|
| `JOB_ID_MISMATCH` | Reject before queue/resource reservation; no alternate ID. |
| `JOB_ID_COLLISION` | Reject the conflicting equivalence class and fail closed for internal authoritative collision. |
| `JOB_REQUEST_RESOURCE_LIMIT` | Reject before allocation/admission using the exact exceeded field and N−1/N/N+1 boundary. |
| `JOB_QUEUE_BACKPRESSURE` | Apply the exact class action and preserve queue generation, owner due record and first-due age. |
| `JOB_QUEUE_GENERATION_EXHAUSTED` | Reject the next close before reservation/publication; retain the previous valid queue checkpoint. |
| `AUTHORITATIVE_WORK_STARVATION` | Fail the transaction/run at maximum logical age; retain diagnostic replay state and never drop the due work. |
| `REQUIRED_RESOURCE_STARVATION` | Block/fail the load or transition, retain prior active generation and exact unmet prerequisite. |
| `JOB_CANCELLED` | Reject every uncommitted result in the effective subtree and publish no partial delta/resource generation. |
| `TASK_RESULT_COLLISION` | Use SPEC-21 conflict receipt; no completion wins by arrival. |
| `STALE_REVISION` | Discard the complete result before merge and retain current owner/resource state. |
| `MEMORY_BUDGET_EXCEEDED` | Reject optional admission or block required plan before hard-limit overrun; never borrow protected reserve. |
| `RESOURCE_CHARGE_MISMATCH` | Reject complete staged result before residency/publication. |
| `RESOURCE_PIN_LIMIT` | Reject new pin/plan atomically; retain all existing pins and resident state. |
| `RESOURCE_LEASE_INVALID` | Reject invalid/expired/overflowing lease or renewal; wall time is not consulted. |
| `RESOURCE_EVICTION_BLOCKED` | Retain pinned/leased/dirty/required state; reject optional work or block required plan. |
| `ARCHIVE_RESOURCE_LIMIT` | Reject the whole archive before allocation/publication and quarantine untrusted staging when required by policy. |
| `ARCHIVE_INVALID` | Reject all entries and publish no partial directory/bundle/resource generation. |
| `IO_BACKPRESSURE` | Stop new physical dispatch, retain logical reservations and apply the exact class fallback. |
| `IO_RESULT_INVALID` | Discard complete decode staging; retain prior active generation. |
| `NONDETERMINISTIC_RESULT` | Stop on the first admission, cancellation, merge, eviction or publication divergence; repeating identical inputs cannot select another result. |

Diagnostics are stable engine-owned envelopes with owner, stage/logical epoch,
job/request/scope/qualified-resource IDs, expected/actual bounded values,
profile hashes and remediation. Backend strings and rendered text are not the
oracle.

## Product checks

| ID | Scenario / command | Expected behavior | Fallback |
|---|---|---|---|
| `JOB-P1` | `next check deterministic-jobs --workers 1,2,8,16 --permutations 10000 --faults all` | Admission, dispatch, completion, cancellation and restart permutations produce exact batch, assignment, merge, event and state roots; N−1/N/N+1 bounds are exact; no mutable async access, partial publication, arrival-selected result or silent authoritative drop. | Serialize dispatch through the same profile, retain due work and block/fail the complete transaction. |
| `RESOURCE-MEMORY-P1` | `next check resource-memory --boundary-matrix all --faults all` | Every logical pool, charge and reserve boundary returns the exact code without overflow; the reference scene stays within 12,884,901,888 host bytes and 5,905,580,032 device bytes; worker/cache/allocator permutations change no authoritative result. | Reclaim eligible reconstructible bytes or use a declared fallback; otherwise retain prior authority and block the plan. |
| `RESOURCE-RESIDENCY-P1` | `next check compute-resource-residency --cycles 10000 --permutations all --faults all` | Reserve, stage, validate, pin, lease, renew, evict and restart cycles produce exact roots; pinned, leased, dirty, in-flight, authoritative and only-valid-generation state is never evicted or partially published. | Evict only canonical eligible candidates; otherwise reject optional work or retain the required generation. |
| `IO-BACKPRESSURE-P1` | `next check bounded-io-archive --cycles 10000 --limits n-1,n,n+1 --faults all` | Queue, I/O, decode, cancel and restart permutations produce exact decisions; no unbounded allocation, authoritative drop, partial publication or gameplay result selected by I/O order; ready required results commit or reject stale within two gameplay ticks and commit work stays within 2,000 microseconds. | Reduce physical concurrency, reject/preempt optional work and discard staging; required work remains pending or fails closed. |

## Technical requirements

| ID | Technical requirement |
|---|---|
| REQ-116 | Every delegated runtime work unit MUST use one closed `JobClassV1`, deterministic identity/order/admission, immutable input, bounded cancellation tree and the existing SPEC-21 completion/canonical merge path; no mutable async access or partial authoritative publication is allowed. |
| REQ-117 | Every runtime and tool execution profile MUST provide one hash-bound `MemoryBudgetProfileV1` with finite owner/pool/charge/reserve bounds, checked admission and exactly-once accounting; measured allocator/RSS/device state MUST NOT choose an authoritative outcome. |
| REQ-118 | Qualified compute-resource residency MUST use revisioned records, finite pins/leases, canonical eviction and atomic generation publication; pinned, leased, dirty, in-flight, authoritative and only-valid-generation state MUST never be evicted. |
| REQ-119 | Job and I/O queues, inline inputs, result manifests, archives and decompression MUST have finite hash-bound capacity/size/logical-age limits and deterministic credit backpressure; authoritative/required work MUST remain recorded until commit, validated cancellation or explicit blocking failure and MUST NOT be silently dropped. |

## Failure paths

| ID | Trigger | Required result |
|---|---|---|
| FAIL-046 | Invalid, duplicate, conflicting, oversized or stale job; queue overflow or maximum logical age; cancellation/result race; worker panic or restart during uncommitted work | Reject the complete equivalence class/result or atomically cancel/abort it; preserve the owner due record, first-due age and prior checkpoint; publish no partial delta, event or resource generation. |
| FAIL-047 | Memory/pin/lease/residency exhaustion, charge mismatch, archive/decompression violation, I/O backpressure/starvation, cancellation or publication fault | Reject optional admission or retain the complete prior active state and exact required pending record; evict only eligible reconstructible bytes, discard staging and fail before authoritative mutation. |

## Technology neutrality

This contract chooses no ECS scheduler, async runtime, allocator, compression or
archive library, filesystem/storage API, database, render-resource manager,
world-streaming implementation or vendor backend. Replaceable implementations
remain private behind these engine-owned schemas and exact contracts.

SPEC-23 does not create world partition/interest ordering, content schema,
capture-worker network scheduling or a new `GameplayBudgetMatrix`. Downstream
owners may provide canonical source plans and plan ordinals, but cannot bypass
job/resource admission, weaken required criticality, reset logical age or
publish outside the common deterministic commit path.

## Narrative work specialization

The [SPEC-31](31-autonomous-quest-lifecycle-and-narrative-director.md)
uses closed job classes, immutable inputs, finite queues, logical age and the
SPEC-23 current/next completion path for request construction, optional external
generation, result validation and template fallback. Candidate bytes are
bounded before validation and never receive mutable owner state.

Queue/resource denial MAY отменить optional external generation, но MUST
сохранить due narrative boundary/hook и выполнить deterministic template
fallback через тот же validated command. Required fallback work не может быть
silently dropped, а measured worker speed или resource availability не меняет
fixed decision boundary либо canonical merge order.

Divine-role work MAY run one job per eligible patron, but every job owns a
separate request identity and immutable input while sharing only the
content-addressed `DivineJudgmentBatchBaseV1`. Completion order is not a merge
order and receiving all completions early does not commit early. At the exact
`NarrativeDecisionBoundaryV1`, resource-denied or unfinished patrons use their
individual template fallback; the canonical selected set is resolved and
committed as one bounded atomic batch. Queue pressure cannot silently remove a
patron from the base, choose the most attentive/fastest patron, move the
boundary or publish a partial standing vector.
