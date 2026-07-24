# ADR-028: Platform session and presentation authority

| Поле | Значение |
|---|---|
| ID | ADR-028 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | Architecture Working Group |
| Требуемые согласующие | Repository Owner, Runtime Team, Rendering Team, Player Experience Team, Asset & Persistence Team, Verification & Evidence Team, Release Engineering |
| Дата решения | 2026-07-24 |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-04](../04-rendering-and-platform.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](../18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [ADR-018](018-authoritative-project-composition-and-configuration.md), [ADR-019](019-canonical-player-actions-and-presentation-authority.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-023](023-human-review-decision-v2-and-offline-attestation.md), [ADR-024](024-requirement-gate-evidence-and-profile-closure.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## История принятия

ADR-028 подготовлен для consolidated architecture packet 1.8. Он фиксирует
authority split между platform adapter, application session, authoritative
runtime и reconstructible presentation. Принятие exact packet root означает
только architecture admission; оно не создаёт runtime implementation, gate
`PASS`, human approval, `vertical-v1` либо release readiness.

## Контекст

Accepted architecture отделяет gameplay от renderer и platform adapters, но не
закрывает один versioned lifecycle contract для `game`, `headless` и
`capture-worker`, exactly-once shutdown/save semantics, либо полный neutral
контракт extraction/material/VFX/cache recovery. Без нового решения реализации
могут:

- использовать OS callbacks как второй session state machine;
- создавать window/surface/display в headless или capture-worker;
- превращать focus, suspend или device loss в прямую gameplay mutation;
- закрывать или сохранять одну session повторно;
- передавать ECS, native event, graphics handle или compiler object через
  public boundary;
- возвращать renderer/UI/VFX cache state в simulation;
- выбирать authoritative input tick или gameplay outcome по wall time,
  presentation frame или device recovery timing.

## Решение

### Authority split

1. Runtime Team владеет `ApplicationSessionManifestV1`, session lifecycle,
   composition/runtime staging, quiesce/finalize coordination и exactly-once
   close/save receipt.
2. Rendering Team владеет platform adapter normalization, canonical
   presentation extraction publication at the Runtime-declared boundary,
   `PresentationConsumptionStateV1`, material/shader-interface consumption,
   render/VFX caches и device recovery. Native objects остаются внутри
   adapters.
3. Player Experience Team владеет action mapping, semantic UI, camera intent,
   localization и accessibility. Platform adapter публикует только normalized
   controls/events; presentation их не превращает в gameplay state.
4. Core Runtime и bounded-context owners остаются единственными владельцами
   authoritative simulation. Presentation consumes immutable snapshots and
   emits non-authoritative cues; data flow никогда не разворачивается назад.
5. Asset & Persistence Team владеет atomic save generation, но session
   coordinator владеет единственным request/receipt lifecycle. Ни один adapter
   не создаёт второй save authority.

### One session state machine

Все required composition roots используют closed state machine:

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

Переходы выполняются только versioned lifecycle request/event contracts.
Unknown, duplicate, stale или illegal transition rejects before mutation.
`Closed` terminal. Repeated close, shutdown, final-save или recovery request
returns the persisted prior terminal receipt when identity and bytes are exact;
an identity collision rejects.

Identity lookup precedes expected-revision validation. A first-seen request
validates its expected revision/state; an existing exact request compares the
complete archived canonical bytes and returns or resumes only its
journal-proven next step. It MUST NOT reject merely because its original
revision became historical after the transition it already initiated.

Exactly-once final save is enforced across owner boundary by a durable
Asset & Persistence `SessionFinalSaveLedgerV1` keyed by
`(session_id, close_request_id)`. Reservation precedes save work; save-generation
publication atomically records one terminal persistence receipt in that entry.
`CloseSessionOperationJournalV1` and the ledger both bind the complete
domain-separated `canonical_close_request_hash` and archived request bytes, not
a save-only subset.
The ledger has `Reserved | RetryPending | Committed | Failed`: a hash-bound
finite retry policy may advance only the same canonical close request through
`RetryPending`, while `Committed` and `Failed` are terminal and can never
publish more than one save generation.
The session then advances one revision per allowed edge
`Active|Suspended → Quiescing → Finalizing → Closed`; the final `Closed` event
and `CloseSessionReceiptV1` publish atomically. A crash between save and session
publication reuses the committed persistence entry and cannot create another
save generation.

`Closed` remains the only terminal lifecycle state.
`CloseSessionReceiptV1.result` is the engine-owned closed enum
`CloseSessionResultV1 = Saved | ClosedUsingLastSafeGeneration`. The latter
requires a terminal failed ledger entry, a verified last-safe generation and an
explicit
`AllowLastSafeGeneration` shutdown policy; it is not a state named
`ClosedWithoutFinalSave`. A retryable ledger outcome keeps the session in
`Finalizing` and may resume only the same close identity within the persisted
attempt limit. Exhaustion under `RequireFinalSave` also retains `Finalizing`
with a durable typed non-success and no close receipt; recovery then starts a
new session from the last safe generation rather than rewriting the failed
close history.

`FinalSaveReceiptV1` binds session/close/full-request/reservation identity,
attempt ordinal, source session/world revisions and exact save
generation/manifest/owner roots under a domain-separated hash. Save generation,
receipt and committed ledger-entry hash publish atomically.

That new session requires a durable `RecoverySessionLinkV1`: it binds the exact
old `Finalizing` state and failed ledger entry, unchanged
`ProjectCompositionLock` and prior application-session manifest, verified
last-safe save and new session ID. The old session becomes immutable non-live
`RecoverySuperseded` history without gaining a fake lifecycle edge. Recovery
does not create a project revision; no different project lock may activate
until the recovered live session and the existing SPEC-17 project lifecycle
close cleanly.

Focus loss, suspend, resume, close request, device loss and recovery are typed
facts. Они могут request a declared transition or rebuild presentation, но не
выбирают command outcome, simulation tick, RPG state или world-time advance.
Crash recovery consumes the last complete composition/save/session receipt and
never guesses from wall clock or an incomplete presentation cache.

### Composition-root parity

`game`, `headless` и `capture-worker` share exact project lock, schema registry,
content catalog, command validation, persistence, replay, schedule and
authoritative owner logic.

- `game` MAY instantiate interactive platform and presentation adapters.
- `headless` MUST instantiate neither window, display, surface nor presentation
  device.
- `capture-worker` MUST remain displayless and MAY instantiate only an explicit
  offscreen presentation target bound by `CaptureJobManifest`.
- `tools` MAY use platform services but MUST NOT become an authoritative runtime
  path.

Compile-time features, process topology, adapter availability and OS event order
MUST NOT alter domain semantics. Unsupported required capability fails before
world activation; optional capability uses its manifest-declared fallback.

### Presentation authority

Presentation extraction copies a bounded immutable projection at a declared
simulation boundary into `PresentationSnapshotV2`. Snapshot epoch/object keys
are engine-owned nominal values. Rendering, UI, audio and VFX may interpolate
or cache this snapshot, but cannot mutate it or return derived state to
simulation.

Extraction fragments are flattened, canonically sorted and partitioned only by
the lock-bound batch profile, so worker count/order cannot change snapshot
bytes. VFX uses epoch-independent event-derived cue, acknowledgment and
continuous-instance identities with explicit lifecycle targets.

Material definitions/instances, shader interface manifests, deterministic
pipeline keys, SDR/HDR boundary metadata and VFX cues are engine-owned serialized
contracts. Backend device objects, OS surfaces, shader compiler objects, native
events and vendor handles remain private adapters.

GPU/UI/VFX caches are reconstructible. Device loss invalidates caches and
in-flight presentation only. Bounded engine-owned CPU
`PresentationConsumptionStateV1` is presentation-session recovery metadata,
not a GPU/UI/VFX cache and not gameplay/save authority; it retains the consumed
cue prefix, one-shot acknowledgment root, bounded pending-one-shot map and
active continuous-instance map across cache invalidation. The same accepted
snapshot/content/profile/consumption-state inputs must rebuild equivalent
presentation state without replaying an acknowledged one-shot; cache loss
cannot roll back or advance simulation.

### Determinism and evidence

- Platform timebase converts native samples to bounded normalized event facts;
  only ADR-022 ingress assignment chooses current/next tick.
- Renderer cadence, wall time, worker completion and device recovery timing
  never select an authoritative action or state transition.
- Gameplay/replay hashes exclude presentation-only state. Pinned capture profile
  separately defines exact normalized frame/audio roots.
- Observable visual, UI, camera, animation, physics, motor or VFX changes follow
  SPEC-15 impact/capture/evidence and require signed
  `HumanReviewDecisionV2::Approve`; automatic PASS alone does not admit them.

## Рассмотренные варианты

- Independent lifecycle per composition root — `Rejected`: parity and recovery
  would diverge.
- OS callbacks own session state — `Rejected`: native order and reentrancy would
  become authority.
- Headless host creates a hidden window — `Rejected`: displayless conformance
  would be false and platform availability could affect simulation.
- Renderer queries ECS directly — `Rejected`: exposes storage and introduces a
  second mutable read/write timing surface.
- Presentation cache serialized as gameplay state — `Rejected`: device/backend
  data would enter saves and replay.
- Device-loss fallback selects another gameplay action — `Rejected`: wall-time
  performance would change authoritative outcome.

## Последствия

- SPEC-29 owns exact platform/session contracts and lifecycle verification.
- SPEC-30 owns extraction/render-content/VFX/cache contracts.
- SPEC-04 remains the admitted baseline renderer/platform owner and is
  synchronized to these engine-owned facades without admitting any proposed
  platform or graphics technology.
- Adapter traits, ECS layout, render graph internals, shader implementation and
  process wiring remain private implementation choices.

## Rollback

Before runtime artifacts exist, rollback means rejecting packet 1.8 and keeping
packet 1.7 authoritative. After implementation, any semantic replacement
requires a new superseding ADR, synchronized SPEC/traceability/evidence updates,
and explicit migration for persisted session or presentation contracts.
