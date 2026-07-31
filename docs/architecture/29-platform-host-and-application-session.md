# SPEC-29: Platform host and application session

| Поле | Значение |
|---|---|
| ID | SPEC-29 |
| Статус | Accepted |
| Версия | 2.4 |
| Последняя проверка | 2026-07-31 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-04](04-rendering-and-platform.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [ADR-018](adr/018-authoritative-project-composition-and-configuration.md), [ADR-019](adr/019-canonical-player-actions-and-presentation-authority.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-028](adr/028-platform-session-and-presentation-authority.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-035](adr/035-bounded-live-recovery-platform-host-and-presentation-cut.md), [ADR-037](adr/037-packed-session-object-storage.md) |
| Заменяет | отсутствует |

## История принятия

SPEC-29 подготовлен как часть consolidated architecture packet 1.8. Он
фиксирует engine-owned platform normalization, one application-session state
machine и composition-root parity. Версия 2.0 сохраняет эти technical
contracts и заменяет admission-oriented verification обычными product checks
по ADR-030. Версия 2.2 зафиксировала implementation shape bounded active-run
checkpoint/restart и platform-causal lifecycle rollback. Версия 2.3
синхронизирует exact cadence, archive/retention bounds, session-bound
platform-host identity и presentation recovery cut с ADR-035; новые public
contracts или technology decisions не вводятся. Версия 2.4 специализирует
private physical representation current+previous logical generations по
ADR-037 без изменения cadence, generation identity или recovery semantics.

## Назначение и invariants

- Platform callback MUST produce a bounded typed event, never mutate gameplay.
- Only ADR-022 ingress assignment may map normalized control input to a
  simulation tick. Native timestamp, callback order and renderer frame are not
  authority.
- `game`, `headless` and `capture-worker` MUST share project activation,
  command/schema/persistence/replay/runtime ordering and session transitions.
- `headless` and optional `capture-worker` MUST make zero window, display and
  interactive surface creation attempts. Developer capture uses only an
  explicit displayless offscreen target owned by presentation.
- Close, final-save and terminal receipt MUST be exactly-once by stable request
  identity and survive restart.
- Focus, suspend, resume, device loss and shutdown MUST be typed lifecycle
  inputs with explicit bounded behavior.
- Public contracts contain only engine-owned nominal IDs, closed enums,
  fixed-width values, canonical hashes and immutable descriptors. They contain
  no native event, OS/window/surface/display object, driver handle, task/future,
  ECS storage, importer record or vendor/backend type.

## Technical authority boundary

| State | Authoritative representation | Не является authority |
|---|---|---|
| Native event collection and capability probing | private platform adapter | event-loop object, driver callback, monitor handle |
| Normalized platform facts | immutable `PlatformEventV1` stream | native event bytes after normalization |
| Device-independent actions | SPEC-18 action mapper | physical key, controller, scan code |
| Ingress tick assignment | Runtime `IngressAssignmentV1` | `PlatformTimebaseV1`, native time or render frame |
| Session state and transition revision | Runtime `ApplicationSessionStateV1` | OS process state, window visibility, UI mode |
| Project/runtime activation | exact `ProjectCompositionLock` plus Runtime activation receipt | filesystem, environment or launcher cache |
| Save generation | persistence transaction and published save manifest | close callback or session UI |
| Complete live state at a fixed tick | coordinator-owned in-memory runtime/RPG/physics/streaming/input/presentation closure | renderer frame, partial subsystem cache |
| Durable active-run checkpoint | newest complete atomically published session generation for the same session ID | every volatile tick, staging directory or partially written object set |
| Interactive presentation device | private renderer/platform adapter | simulation or save |
| Terminal close/save receipt | Runtime-owned close semantics, atomically published by Assets session store | repeated callback result reconstructed from memory |

## Implementation mapping

The accepted boundary is implemented by these production owners:

| Contract/behavior | Implementation owner |
|---|---|
| Manifest/state validation, nine legal edges, exact retry and collision detection | `next_runtime::session::ApplicationSessionMachine` |
| Immutable lifecycle/observation plans and post-publication in-memory commit | `next_runtime::session` |
| Content-addressed session objects, private bounded generation object packs, generation manifest, single live-session registry, atomic pointer and validated current+previous logical session-generation retention | `next_assets::session::SessionStore` |
| Project activation, ID derivation, 30 Hz live execution, bounded durable checkpointing, full exact lifecycle archive, carry-forward recovery-link evidence, session-bound platform-host registration, same-session active restart/presentation cut and platform-causal suspend/resume/close sequencing | `next_application::ApplicationCoordinator` |
| Production replay execution over `RuntimeReplayDriver` | `next_application::replay` |
| First-party source/bootstrap/scenario/presentation bindings | `next_reference_game` |
| Platform-event normalization only | `next_platform` and private `next_desktop_sdl_ash` adapter |
| Expected-value comparisons and fault scenarios | `next_verification`; never a production dependency |

Every coordinator state change follows validate → immutable plan → one Assets
generation publication → infallible Runtime commit. A failure before the
generation pointer replacement leaves the preceding generation authoritative;
staged content-addressed objects are not committed state. `game`, `headless`
and runtime-bearing `tools` all enter through this coordinator. Headless does
not construct the desktop adapter.

The interactive reference path produces one complete in-memory live closure and
`PresentationSnapshotV2` at every committed 30 Hz simulation boundary. This
volatile publication lets a 30/60/144 Hz renderer repeat the newest complete
snapshot, but it is not a durable filesystem generation.

The bounded durable cadence is exact: publish an active-run checkpoint at tick
`0`, then after every `30` committed ticks, and force one atomically when
processing suspend or close. The private checkpoint closure binds the exact
session/project, Runtime/RPG/physics/world-streaming state, input resolver
state, presentation snapshot/camera state, logical sequences, revisions and
roots. A process crash resumes the same active session from the newest complete
durable checkpoint and therefore MAY lose at most `29` already committed
in-memory ticks. Recovery never invents those ticks, replays renderer callbacks
or converts paused wall time into simulation catch-up. The first rebuilt
presentation snapshot after authoritative restart uses a fresh recovery epoch,
sequence `0` and cut camera records; renderer interpolation cannot cross that
boundary.

Every durable checkpoint and lifecycle publication follows stage → validate →
atomic pointer replacement → infallible in-memory commit. A publication failure
restores the preceding session machine, durable snapshot and object closure.
`SessionStore` validates the complete logical tree before pruning and retains
only the current and immediately previous session generations, so pointer
rollback still has one known-good predecessor. Per ADR-037, logical objects MAY
be concatenated into private canonical-indexed generation pack files; legacy
objects remain raw. Every load slices exact bytes and verifies every final
logical object hash; pack ordinals/offsets do not enter generation identity or
authoritative roots. Full
canonical request/event objects for
the current session remain reachable through a bounded lifecycle archive
(`1 040` total entries, at most `1 024` platform-sourced requests). A
required-save recovery carries forward a separately validated ordered chain
of at most `64` evidence entries. Each entry binds canonical
`RecoverySessionLinkV1` bytes, the exact prior durable snapshot and its full
referenced object closure; aggregate object-reference and unique-object budgets
are `16 384` and `1 984`. This retention rule is session-specific and does not
prune generic content generations.

## Public contracts

### `PlatformCapabilitySetV1`

```text
PlatformCapabilitySetV1 {
  schema_version,
  capability_set_id,
  host_target_profile_id,
  normalized_capabilities[],
  required_capability_failures[],
  presentation_target_kinds[],
  input_classes[],
  timebase_profile_id,
  canonical_hash
}
```

`normalized_capabilities` is a sorted unique list of engine-owned capability
tags and bounded numeric limits. It never contains extension pointer, device
name used as identity, native feature struct or backend enum. Probe output is
normalized and hashed before composition staging. Required absence rejects
before Runtime staging with `PLATFORM_CAPABILITY_REQUIRED`; optional absence
selects only the exact fallback bound by `ProjectCompositionLock`.

For an interactive target, the application coordinator registers one current
adapter lifetime only after validating the complete capability set. Its
canonical hash MUST equal
`ApplicationSessionManifestV1.platform_capability_set_hash_or_none`; the
coordinator then issues a fresh opaque `host_instance_id` bound to that
session/generation. Registration state is ephemeral and non-authoritative.
After restart a new adapter lifetime registers again and receives a fresh host
ID.

`headless` requires no presentation target capability. `capture-worker` may
require `DisplaylessOffscreen` but MUST reject a profile that can satisfy it
only by creating an interactive window/display/surface. A presentation target
`None` cannot register or accept an interactive host.

### `PlatformTimebaseV1`

```text
PlatformTimebaseV1 {
  schema_version,
  timebase_id,
  epoch_id,
  ticks_per_second_num,
  ticks_per_second_den,
  maximum_sample_delta,
  wrap_policy,
  canonical_hash
}
```

The adapter converts a native sample to a checked monotonic
`platform_sample_tick: u64` within one immutable epoch. Conversion overflow,
backward sample outside declared duplicate rules, epoch mismatch or undeclared
wrap rejects the sample. `PlatformTimebaseV1` supports diagnostics and ordering
within the raw adapter batch only; it MUST NOT compute `SimulationTick`,
`WorldCalendarStateV1` or command sequence.

### `PlatformEventV1`

```text
PlatformEventV1 {
  schema_version,
  platform_event_id,
  host_instance_id,
  source_class,
  source_sequence,
  platform_sample_tick,
  kind,
  payload,
  capability_set_hash
}
```

`kind` is a closed enum:

```text
Control | FocusChanged | SuspendRequested | ResumeRequested |
CloseRequested | DeviceConnected | DeviceDisconnected |
PresentationDeviceLost | PresentationDeviceRestored |
CapabilityChanged | FatalHostFault
```

Events sort and deduplicate by
`(host_instance_id, source_class, source_sequence, platform_event_id)`. Exact
duplicate bytes collapse; same identity with different bytes is
`PLATFORM_EVENT_IDENTITY_COLLISION`. A missing sequence is reported and the
affected batch fails closed; callback arrival order is never used as fallback.

Every first-seen interactive event MUST match both the currently registered
`host_instance_id` and its exact capability-set hash. An unregistered host
fails with `PLATFORM_CAPABILITY_REQUIRED`; a stale host lifetime or changed
binding fails with `PLATFORM_EVENT_IDENTITY_COLLISION`. An exact event already
bound to an archived lifecycle/close request remains eligible only for its
journal-proven idempotent retry, even after a fresh host registers; it cannot
start a new transition.

Lifecycle payloads contain closed reasons and bounded values. `Control` embeds
exactly one `NormalizedControlEventV1`; unknown kind/payload combinations,
unbounded text or extra fields reject.

### `NormalizedControlEventV1`

```text
NormalizedControlEventV1 {
  schema_version,
  control_sample_id,
  device_class,
  device_instance_nonce,
  control_path_id,
  phase,
  quantized_value[],
  modifier_set[],
  platform_sample_tick,
  source_sequence
}
```

`phase` is `Started | Changed | Completed | Cancelled`. Values are signed
fixed-width integers normalized by a project-locked profile; NaN, infinity,
native endian blobs and implicit ranges are invalid. `device_instance_nonce`
separates concurrent physical devices inside the local session but MUST NOT
become gameplay identity or durable save data.

The SPEC-18 mapper consumes a closed ordered batch and emits the same
`PlayerActionFrame` schema for interactive and scenario input. ADR-022 then
persists current/next ingress assignment. Platform code cannot emit
`WorldCommand` directly.

## Application session contracts

### `ApplicationSessionManifestV1`

```text
ApplicationSessionManifestV1 {
  schema_version,
  session_id,
  composition_root,
  project_composition_lock_hash,
  launch_profile_hash,
  platform_capability_set_hash_or_none,
  runtime_determinism_profile_hash,
  schema_registry_hash,
  content_manifest_hash,
  recovery_policy_hash,
  shutdown_policy_hash,
  recovery_session_link_hash_or_none,
  presentation_target_kind,
  canonical_hash
}
```

The manifest body omits `canonical_hash`; its value is
`SHA-256("nextengine.application-session-manifest.v1\0" || JCS(body))`.

`composition_root` is `Game | Headless | Tools | CaptureWorker`.
`presentation_target_kind` is `None | Interactive | DisplaylessOffscreen`.
Only these combinations are valid:

| Root | Target | Contract |
|---|---|---|
| `Game` | `Interactive` or manifest-declared `None` diagnostic mode | Interactive target is adapter-owned; authoritative substrate is unchanged. |
| `Headless` | `None` | No display/window/surface/device creation. |
| `Tools` | `None` or `Interactive` | Tools never become authoritative gameplay path. |
| `CaptureWorker` | `DisplaylessOffscreen` | Optional developer capture input; no interactive host objects. |

The manifest is immutable for one session. A normal session has no recovery
link; a recovered session MUST bind the exact `RecoverySessionLinkV1` described
below. Changing composition, capability, recovery, shutdown or target profile
otherwise requires a new session identity and a new activation path.

### State and lifecycle messages

```text
ApplicationSessionStateV1 {
  schema_version,
  session_id,
  state,
  revision,
  application_session_manifest_hash,
  project_composition_lock_hash,
  active_runtime_revision_or_none,
  active_save_generation_hash_or_none,
  last_transition_id_or_none,
  terminal_receipt_hash_or_none,
  canonical_hash
}
```

`revision` is monotonically increasing. The manifest/project-lock hashes are
immutable for the session. A terminal receipt hash is present iff state is
`Closed`. The body omits `canonical_hash`; its value is
`SHA-256("nextengine.application-session-state.v1\0" || JCS(body))`.

The state enum and only normal path are:

```text
Created
  → CompositionStaged
  → RuntimeStaged
  → Active
  ↔ Suspended
  → Quiescing
  → Finalizing
  → Closed
```

The exact allowed edge set is:

```text
Created → CompositionStaged
CompositionStaged → RuntimeStaged
RuntimeStaged → Active
Active → Suspended
Suspended → Active
Active → Quiescing
Suspended → Quiescing
Quiescing → Finalizing
Finalizing → Closed
```

`ApplicationLifecycleRequestV1` contains request ID, session ID, expected
revision/state, requested transition, typed reason, policy hash and causal input
reference. `ApplicationLifecycleEventV1` records request ID, from/to states,
before/after revisions, exact outcome, save/activation/diagnostic references and
event hash.

Every transition is validate → stage → atomic publish:

1. before expected-state validation, look up the request identity in the
   durable session journal;
2. if found, compare the complete archived canonical request bytes/hash:
   mismatch is an identity collision, while an exact completed request returns
   its prior event/receipt without revalidating the now-historical expected
   revision;
3. if new, validate schema, identity, expected session revision/state and legal
   edge;
4. resolve exact required capability/project/runtime/save preconditions;
5. stage immutable subsystem results without publishing session state;
6. atomically publish one next session revision and its lifecycle event;
7. reconstruct optional caches only after the transition.

No error publishes a partial state. Exact retry returns the prior event/receipt.
Same request ID with different canonical bytes fails
`SESSION_REQUEST_IDENTITY_COLLISION`.

The durable archive stores the full canonical request and event bytes as
content-addressed objects, not only their hashes. The current session retains
at most `1 040` lifecycle entries, of which at most `1 024` may be sourced by
platform events. An exact retry reuses its entry; a first-seen platform
transition beyond the bound fails before publication with
`SESSION_PLATFORM_LIFECYCLE_BUDGET_EXCEEDED`. Silent eviction or accepting a
forgotten request ID as new is forbidden.

### Suspend, recovery and device loss

- Focus loss emits a fact. The project-locked policy may request `Suspended`,
  keep `Active`, or cancel local controls; it cannot mutate domain state.
- Suspend closes the input batch at the ordinary ADR-022 boundary, records all
  accepted assignments, quiesces external proposals and atomically publishes a
  forced active-run checkpoint with one session transition. A publication fault
  leaves the prior active generation and in-memory state unchanged.
- Resume validates the same lock/save/runtime revisions before `Active`, starts
  fixed-step pacing from a zero accumulator and does not guess elapsed world
  time or catch up ticks spent suspended. Same-session process restart first
  validates the persisted presentation snapshot as evidence, then publishes a
  fresh authoritative-recovery epoch at sequence `0`; every rebuilt camera is
  a cut whose previous/current sample is equal.
- A close request likewise forces the newest complete live checkpoint before
  its lifecycle/close journal becomes current. Exact duplicate platform events
  return the archived event/progress; a changed event under the same identity
  is a collision.
- Presentation device loss never changes session state by itself. It invalidates
  presentation caches and emits typed diagnostic/event. Policy MAY request
  `Suspended`; simulation behavior remains declared and replayable.
- Fatal host fault requests `Quiescing` if the runtime can still journal safely;
  otherwise an active session restarts under the same session ID from its newest
  complete tick-0/30 durable checkpoint. A failed required-save recovery remains
  the distinct path that creates one linked new live session from the verified
  last-safe save. An incomplete transition is never inferred as committed.

## Exactly-once close and save

One close operation starts with:

```text
CloseSessionRequestV1 {
  schema_version,
  close_request_id,
  session_id,
  starting_session_revision,
  starting_session_state = Active | Suspended,
  shutdown_policy_hash,
  final_save_policy,
  bounded_deadline_class,
  reason,
  causal_input_reference,
  canonical_close_request_hash
}
```

`canonical_close_request_hash` is
`SHA-256("nextengine.close-session-request.v1\0" ||
CanonicalBinaryV1(CloseSessionRequestV1 with the hash field omitted))`.
The complete canonical bytes are retained in a content-addressed close-request
archive; hashing only a save subrequest is forbidden.

Before any close edge, Runtime atomically creates or reads:

```text
CloseSessionOperationJournalV1 {
  session_id,
  close_request_id,
  canonical_close_request_hash,
  close_request_archive_ref,
  starting_session_revision,
  starting_session_state,
  stage = Registered | Quiesced | Finalizing | SaveRetryPending |
          SaveTerminal | Closed,
  quiesce_event_hash_or_none,
  finalizing_event_hash_or_none,
  final_save_ledger_entry_hash_or_none,
  close_session_receipt_hash_or_none,
  canonical_hash
}
```

First registration validates the request's starting revision/state. A later
lookup compares the archived full request before any state check. An exact
retry MUST NOT compare the old starting revision to the current revision;
instead it validates that current session state/events/ledger hashes equal the
journal-proven stage and executes exactly the next missing stage. Any gap,
extra edge, impossible current state or changed bytes is fail-closed. `Closed`
returns the prior receipt.
The journal `canonical_hash` is
`SHA-256("nextengine.close-session-operation-journal.v1\0" || JCS(body))`,
where `body` omits only that hash.

Before save publication, Asset & Persistence reserves the exact
`(session_id, close_request_id)` in a durable `SessionFinalSaveLedgerV1`:

The deadline class controls only host watchdog/escalation reporting. Wall time
cannot choose a ledger attempt, retry count, lifecycle edge, final-save
disposition or `CloseSessionResultV1`; interruption leaves the last durable
entry/state for exact recovery.

```text
SessionFinalSaveLedgerV1 {
  schema_version,
  entries[] {
    session_id,
    close_request_id,
    canonical_close_request_hash,
    close_request_archive_ref,
    reservation_hash,
    status = Reserved | RetryPending | Committed | Failed,
    attempt_count,
    maximum_attempts,
    last_failure_code_or_none,
    save_generation_hash_or_none,
    final_save_receipt_hash_or_none,
    entry_hash
  },
  canonical_hash
}
```

`reservation_hash` domain-separates the immutable ledger key, full close
request hash/archive reference, starting session revision, final-save policy,
shutdown policy hash and maximum attempts through
`FinalSaveReservationBodyV1`. It never changes across attempts.

```text
FinalSaveReservationBodyV1 {
  schema_version,
  session_id,
  close_request_id,
  canonical_close_request_hash,
  close_request_archive_ref,
  starting_session_revision,
  final_save_policy,
  shutdown_policy_hash,
  maximum_attempts
}

reservation_hash = SHA-256(
  "nextengine.final-save-reservation.v1\0"
  || JCS(FinalSaveReservationBodyV1)
)
```

Each `entry_hash` is
`SHA-256("nextengine.session-final-save-ledger-entry.v1\0" || JCS(entry body
with entry_hash omitted))`; entries sort by
`(session_id, close_request_id)` before the enclosing ledger hash is computed.
`final_save_ledger_entry_hash` everywhere else means this exact value.
The enclosing `SessionFinalSaveLedgerV1.canonical_hash` is
`SHA-256("nextengine.session-final-save-ledger.v1\0" ||
JCS({schema_version, ordered_entries}))`; its own hash field is omitted.

Reservation is atomic with the persistence operation intent. Save publication
is atomic with changing the same entry to `Committed` and storing one
`FinalSaveReceiptV1`. The hash-bound shutdown policy contains
`maximum_attempts: u16 >= 1` and
`failure_disposition = RequireFinalSave | AllowLastSafeGeneration`.
The initial `Reserved` entry has `attempt_count: u16 = 0`. The only eligible
attempt ordinal is `k = attempt_count + 1`, and admission requires
`k <= maximum_attempts`. Every atomically published attempt outcome
`RetryPending | Committed | Failed` stores `attempt_count = k`. A crash before
that outcome publication retains the prior count and therefore retries the
same `k`; it cannot consume an ordinal merely by starting backend work. A
retryable failure before the attempt limit stores `RetryPending`, count `k` and
the stable failure code atomically; exact replay of the same close request
explicitly resumes that ledger entry and never relies on a wall-clock timer. A
non-retryable failure or exhausted limit stores terminal `Failed` with count
`k`. Failed attempts publish no save generation, and `Committed` or `Failed`
can never return to a non-terminal ledger state. Exact retry of a terminal
entry returns it and its receipt, when present. Same
`(session_id, close_request_id)` with different request bytes is
`SESSION_FINAL_SAVE_IDENTITY_COLLISION`. A crash after save publication but
before any later session transition therefore discovers `Committed` and MUST
NOT create a second generation.

The persistence receipt is exact:

```text
FinalSaveReceiptV1 {
  schema_version,
  session_id,
  close_request_id,
  canonical_close_request_hash,
  reservation_hash,
  attempt_ordinal,
  source_session_revision,
  source_world_revision,
  save_generation_hash,
  save_manifest_hash,
  authoritative_owner_segments_root,
  canonical_hash
}
```

`attempt_ordinal` is in `1..=maximum_attempts`. The receipt body omits
`canonical_hash`, whose value is
`SHA-256("nextengine.final-save-receipt.v1\0" || JCS(body))`.
For a committed publication, `attempt_ordinal` MUST equal the resulting
ledger entry `attempt_count`; retry-pending and failed entries have no receipt.
Session/close/request/reservation identity, ordinal, source revisions and all
save roots validate before publication. The save generation, receipt and
ledger transition to `Committed` publish atomically; the committed entry stores
the exact receipt hash, so neither side may be rebound to another request,
attempt or generation. Unknown/extra fields or one-field/hash tamper reject
before session progress.

`CloseSessionProgressV1` is the durable non-Closed operation result:

```text
CloseSessionProgressV1 {
  schema_version,
  session_id,
  close_request_id,
  canonical_close_request_hash,
  operation_journal_hash,
  current_session_revision,
  current_session_state = Finalizing,
  final_save_ledger_entry_hash,
  attempt_count,
  result = RetryPending | FinalSaveRequiredFailed,
  canonical_hash
}
```

`RetryPending` permits only explicit continuation of the same canonical close
request. `FinalSaveRequiredFailed` is terminal for that close request, retains
the session in `Finalizing`, publishes no terminal close receipt and requires a
later recovery to start from the identified last safe generation under a new
session ID.
Its canonical hash is
`SHA-256("nextengine.close-session-progress.v1\0" || JCS(body))`.

`CloseSessionResultV1` is the closed enum
`Saved | ClosedUsingLastSafeGeneration`.

The session journal then persists:

```text
CloseSessionReceiptV1 {
  close_request_id,
  session_id,
  canonical_close_request_hash,
  starting_revision,
  quiesce_event_hash,
  final_save_ledger_entry_hash,
  final_save_receipt_hash_or_none,
  last_safe_generation_hash_or_none,
  finalizing_event_hash,
  preclose_operation_journal_hash,
  closed_event_hash,
  terminal_session_revision,
  result: CloseSessionResultV1,
  canonical_hash
}
```

`Saved` requires a `Committed` ledger entry, present final-save receipt and
absent last-safe fallback hash. `ClosedUsingLastSafeGeneration` requires a
terminal `Failed` entry, absent final-save receipt, present verified last-safe
generation and `AllowLastSafeGeneration`; it is a result value, never an
additional lifecycle state. All other field/status combinations reject.
The receipt canonical hash is
`SHA-256("nextengine.close-session-receipt.v1\0" || JCS(body))`.

The coordinator:

1. registers or exact-matches `CloseSessionOperationJournalV1`, then resumes
   from its journal-proven stage;
2. from `Registered`, atomically publishes exactly one
   `Active|Suspended → Quiescing` revision/event and advances to `Quiesced`;
3. stops admitting new external proposals at the declared barrier;
4. drains or deterministically cancels staged non-authoritative work and
   completes already committed subsystem work;
5. reserves or reads the durable final-save ledger entry;
6. atomically publishes exactly one `Quiescing → Finalizing` revision/event
   and advances the operation journal to `Finalizing`;
7. executes or explicitly resumes the bounded ledger attempt sequence,
   publishing at most one final save generation/receipt for the exact close ID;
8. for `Committed`, or for `Failed` with the allowed and verified last-safe
   fallback, atomically publishes exactly one `Finalizing → Closed`
   revision/event, `CloseSessionReceiptV1` and journal stage `Closed`;
9. releases adapters only after durable `Closed` publication.

Every `RetryPending` ledger publication atomically records journal stage
`SaveRetryPending`; `Committed` or `Failed` atomically records `SaveTerminal`
and the exact entry hash. Resume accepts only the state/ledger combination
declared by that stage and cannot repeat an already referenced event or
attempt.

`RetryPending` retains `Finalizing` and returns `CloseSessionProgressV1`; an
exact retry or restart may execute only the next bounded attempt. Terminal
`Failed + RequireFinalSave` also retains `Finalizing`, but returns
`FinalSaveRequiredFailed` and is not retryable under that close ID. Terminal
`Failed + AllowLastSafeGeneration` may close only with
`CloseSessionResultV1::ClosedUsingLastSafeGeneration` after verifying and
binding the last safe generation. Each lifecycle event advances exactly one
revision and one allowed edge; no transition publishes both `Finalizing` and
`Closed`. There is no `ClosedWithoutFinalSave` state and no silent success.
Restart with the same close request reads the durable ledger/session journals,
completes only a permitted missing ledger attempt or transition and returns the
durable outcome without another committed save generation.

### Recovery from terminal required-save failure

When `FinalSaveRequiredFailed` prevents `Closed`, the old
`ApplicationSessionStateV1` remains immutable `Finalizing`; recovery MUST NOT
rewrite it to `Closed` or invent a receipt. A new live session is possible only
through:

```text
RecoverySessionLinkV1 {
  schema_version,
  prior_session_id,
  prior_session_state_hash,
  prior_session_revision,
  prior_close_request_id,
  canonical_close_request_hash,
  failed_final_save_ledger_entry_hash,
  project_composition_lock_hash,
  prior_application_session_manifest_hash,
  last_safe_save_generation_hash,
  last_safe_save_manifest_hash,
  new_session_id,
  prior_session_disposition = RecoverySuperseded,
  recovery_reason,
  canonical_hash
}
```

Runtime validates the failed terminal ledger entry, exact prior `Finalizing`
state/journal, last-safe generation/manifest and unchanged project
lock and prior application-session manifest. The
`prior_application_session_manifest_hash` MUST equal the manifest hash stored
in the exact `ApplicationSessionStateV1` named by
`prior_session_state_hash`; that manifest MUST bind the same
`project_composition_lock_hash`. Runtime then atomically publishes the link,
marks the prior session non-live in the durable session registry and stages the
new `ApplicationSessionManifestV1` with this exact link hash, same
`project_composition_lock_hash` and last-safe save generation.
`RecoverySuperseded` is a registry disposition, not a lifecycle state or edge.

Per ADR-035 and ADR-037, logical session-generation retention is bounded to
current and immediately previous complete generations. Physical completeness
includes every exact raw object or canonical-indexed object pack; the generation
directory remains the complete portable physical closure.
Every recovered live session
instead carries an ordered evidence archive containing the full canonical
bytes and content hash of every retained `RecoverySessionLinkV1`, the exact
prior durable snapshot and a canonical sorted hash list for its referenced
manifest, lifecycle request/event, close/journal/failed-ledger and live closure
objects. The chain MUST keep the same project lock, validate every named prior
`Finalizing` state and full lifecycle/close evidence, connect each
`new_session_id` to the next `prior_session_id`, terminate at the current
manifest/session and remain within `64` entries, `16 384` object references and
`1 984` unique carry-forward objects. Missing, reordered, disconnected or
hash-mismatched evidence rejects before activation. Exhausting any bound fails
before publication with `SESSION_RECOVERY_EVIDENCE_BUDGET_EXCEEDED`.

This bounded evidence keeps prior immutable state/events as read-only canonical
history without retaining unbounded raw generation directories. It does not
rewrite historical session state, fabricate `Closed` or loosen exact link
validation.

The link canonical hash is
`SHA-256("nextengine.recovery-session-link.v1\0" || JCS(body))`, with only
`canonical_hash` omitted. Unknown/extra fields and any one-field/hash mismatch
reject before the live-session registry changes.

There is exactly one live session for a project activation. Recovery performs
no project resolution, activation or revision change: the SPEC-17 project
lifecycle remains on the exact previously `Activated` composition. A different
project lock/revision cannot stage or activate until the recovered live session
has completed its declared shutdown and that prior project lifecycle is cleanly
`Closed`. Exact recovery-link retry returns the same link/new session; changed
prior/new identity, lock, save or request bytes is
`SESSION_RECOVERY_LINK_IDENTITY_COLLISION`.

## Stable diagnostics

Required stable codes include:

- `PLATFORM_CAPABILITY_REQUIRED`;
- `PLATFORM_EVENT_SCHEMA_INVALID`;
- `PLATFORM_EVENT_IDENTITY_COLLISION`;
- `PLATFORM_EVENT_SEQUENCE_GAP`;
- `PLATFORM_TIMEBASE_INVALID`;
- `PLATFORM_FORBIDDEN_PRESENTATION_TARGET`;
- `SESSION_MANIFEST_INVALID`;
- `SESSION_TRANSITION_INVALID`;
- `SESSION_EXPECTED_REVISION_STALE`;
- `SESSION_REQUEST_IDENTITY_COLLISION`;
- `SESSION_QUIESCE_FAILED`;
- `SESSION_FINAL_SAVE_IDENTITY_COLLISION`;
- `SESSION_FINAL_SAVE_FAILED`;
- `SESSION_FINAL_SAVE_RETRY_EXHAUSTED`;
- `SESSION_RECOVERY_INCOMPATIBLE`;
- `SESSION_RECOVERY_LINK_IDENTITY_COLLISION`;
- `SESSION_PLATFORM_LIFECYCLE_BUDGET_EXCEEDED`;
- `SESSION_RECOVERY_EVIDENCE_BUDGET_EXCEEDED`;
- `SESSION_TERMINAL_RECEIPT_MISSING`.

Diagnostics use the SPEC-09 envelope with stable typed expected/actual fields,
session/event/request IDs and causal references. Rendered text is not the oracle.

## Product checks

| ID | Scenario | Expected behavior | Fallback |
|---|---|---|---|
| `PLATFORM-HOST-P1` | Exercise capability, event, timebase, focus, suspend and device-loss cycles with unregistered/stale/wrong-capability host lifetimes, exact archived retries and malformed/forbidden-target cases. | First-seen interactive events match the coordinator-registered current host/capability binding; stale bindings reject before mutation, exact archived retries remain idempotent, and `headless`/optional `capture-worker` create no interactive host objects. | Reject the event/required host profile or select only a project-declared optional adapter fallback before activation. |
| `PLATFORM-INPUT-P1` | Feed equivalent native samples in different callback and batch permutations around the current/next ingress boundary. | Canonical control/action bytes and ADR-022 assignments match; wall time, native timestamp and render frame never select the simulation tick. | Reject the affected input batch and retain prior authoritative state. |
| `SESSION-P1` | Run every legal and representative illegal lifecycle transition for `game`, `headless`, `tools` and optional `capture-worker`, including duplicate/colliding platform suspend, resume and close events, archive exhaustion and publication faults. | Each accepted edge publishes one complete revision/event plus full canonical request/event objects; exact retry returns the archived result, collision/budget exhaustion fail closed, and publication failure rolls back in-memory and durable state to the prior generation. | Preserve the prior complete session revision and reject the transition. |
| `SESSION-RECOVERY-P1` | Crash active live runs before/at/after ticks `0`, `30` and `60`; inject suspend/close, pointer, object-pack/index, pruning, close-registration, final-save, receipt, recovery-evidence-chain and restart faults. | Every 30 Hz tick has a complete in-memory snapshot; durable checkpoints occur only at tick `0`, every `30` ticks and forced suspend/close boundaries. Active restart keeps the same session, loses at most `29` ticks without wall-time catch-up and publishes a fresh presentation epoch/sequence `0` with camera cuts. Current+previous logical generations (including exact object packs) and at most `64` carried recovery entries retain exact prior snapshot/object bytes; pack or chain tamper/overflow fails closed. Required-save recovery still creates exactly one linked new live session, and a close ID creates at most one final save/terminal receipt. | Resume the newest complete same-session checkpoint, retain journal-proven `Finalizing` progress, or use only the policy-allowed last-safe save for one linked recovery session. |

## Requirements

| ID | Требование |
|---|---|
| REQ-140 | `PlatformCapabilitySetV1`, `PlatformTimebaseV1`, `PlatformEventV1` and `NormalizedControlEventV1` MUST normalize bounded platform facts without exposing native types or selecting authoritative tick/outcome; a first-seen interactive event MUST match the current coordinator-registered session host and exact capability-set hash. |
| REQ-141 | Every composition root MUST use `ApplicationSessionManifestV1` and the exact closed lifecycle with validated atomic revision transitions. |
| REQ-142 | `headless` and optional `capture-worker` MUST remain displayless while sharing the same authoritative project/runtime/schema/persistence/replay substrate as `game`. |
| REQ-143 | Platform lifecycle, close, bounded final-save attempts, terminal receipt and recovery MUST be exactly-once by a full canonical request identity and journal-proven progress across callback duplication, faults and restart. Active-run restart MUST preserve the same session from the newest tick-0/30 or forced durable checkpoint with at most 29 ticks of rollback, no wall-time catch-up and a fresh presentation epoch/sequence-0 camera cut. Current+previous complete logical generations, including every exact legacy raw object or canonical-indexed object pack, full bounded lifecycle request/event objects and at most 64 ordered recovery evidence entries carrying exact prior snapshot/object closures MUST fail closed on gap/tamper/overflow; only `Closed` has a terminal close receipt, and failed required-save recovery MUST bind one new live session to the same project activation and verified last-safe generation. |

## Failure paths

| ID | Trigger | Required result |
|---|---|---|
| FAIL-058 | Malformed/unknown/colliding platform event, unregistered/stale/wrong-capability host, invalid timebase/capability, illegal lifecycle transition or forbidden interactive target | Reject the exact event/profile/transition before authoritative mutation; accept an old-host event only as an exact archived retry, preserve prior session state and never derive tick/outcome from callback order or wall time. |
| FAIL-059 | Suspend/device-loss/close/save/finalize crash, active-checkpoint/archive/object-pack/retention fault or budget exhaustion, retryable or terminal save failure, stale starting revision on exact retry, full-request/receipt/recovery-link collision or tamper, duplicate request or restart at any publication boundary | Lookup and compare the archived full request first; return exact prior journal progress/receipt or execute only its next missing step. Roll back a failed publication, preserve current+previous complete logical generations with exact raw/packed object closures plus bounded lifecycle/recovery-link evidence, never infer volatile ticks after the newest durable checkpoint and cut presentation on authoritative restart. Create at most one committed final save and terminal receipt, retain `Finalizing` with typed non-success when policy cannot close, and use a new linked session only for exact prior lock/activation/last-safe-save recovery. |

## Technology neutrality

This specification selects no window/event library, operating-system API,
graphics API, input library, UI toolkit, process supervisor or persistence
backend. Implementations remain private adapters behind engine-owned contracts.
Architecture acceptance alone does not establish Windows/Linux shipping,
macOS shipping or GPU behavior. Optional capture hardware and developer media
tools do not affect session correctness.
