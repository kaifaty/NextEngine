# SPEC-29: Platform host and simple application session

| Поле | Значение |
|---|---|
| ID | SPEC-29 |
| Статус | Accepted |
| Версия | 3.2 |
| Последняя проверка | 2026-08-18 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-04](04-rendering-and-platform.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [ADR-028](adr/028-platform-session-and-presentation-authority.md), [ADR-035](adr/035-bounded-live-recovery-platform-host-and-presentation-cut.md), [ADR-047](adr/047-simple-application-session-and-save-on-close.md), [ADR-082](adr/082-linux-first-development-and-deferred-windows-host.md), [ADR-084](adr/084-public-creator-run-and-project-package-vertical.md) |
| Заменяет | SPEC-29 3.1; admits the generic creator headless run through the existing Application Session and save-on-close contract |

## Platform boundary

The platform adapter normalizes native callbacks into bounded engine-owned
facts. It never mutates gameplay or selects a simulation tick.

- `PlatformCapabilitySetV1` records normalized capabilities and permitted
  presentation targets, without native feature structs or handles.
- `PlatformTimebaseV1` supports adapter ordering/diagnostics only. Host time,
  wall time and render frames are not simulation authority.
- `PlatformEventV1` uses stable host/source identity and sequence. Exact
  duplicate bytes are idempotent; identity collision or sequence gap fails
  closed.
- `NormalizedControlEventV1` carries device-independent bounded values to the
  SPEC-18 action mapper. Platform code cannot emit `WorldCommand` directly.
- `headless` creates no window, display, surface or interactive device.
  Optional displayless capture remains presentation-only.

Native event objects, window/surface/display handles, backend pointers and
driver types remain private adapters.

## Session authority and lifecycle

Runtime owns `ApplicationSessionStateV2`; Assets owns atomic durable snapshot
publication; `ApplicationCoordinator` composes the production path. `game`,
`headless` and runtime-bearing tools share project activation, command/schema,
persistence/replay, system ordering and session transitions.

`next project run` is the current project-neutral tool consumer. It starts an
exact activated external project with `Headless` composition and no
presentation target, executes one production tick, then closes through the
same journal and SaveStore path. Its isolated temporary state makes the final
save a deterministic proof, not a public creator resume slot. The path does not
construct reference-game roles, presentation or aggregate fixtures.

`ApplicationSessionManifestV2` binds session ID, composition root, exact
project lock, launch/platform/runtime/schema/content hashes and presentation
target. It contains no recovery, storage or shutdown policy.

The eight lifecycle states are fixed:

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

Legal edges are only:

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

`ApplicationLifecycleRequestV2` binds request/session identity, expected
revision/state, one legal edge, reason and causal input. A successful transition
publishes one `ApplicationLifecycleEventV2` and one incremented
`ApplicationSessionStateV2`. The durable snapshot retains only the last request
and event. Exact retry returns the retained result when its complete identity
matches; changed bytes under the same ID are a collision.

Every transition follows validate → immutable plan → durable snapshot publish
→ infallible in-memory commit. Failure before pointer replacement preserves the
previous session snapshot and Runtime state.

## Simple session store

The session store contains two alternating slots, a `CURRENT` pointer and one
canonical `session.snapshot.v4.bin` in the selected slot. The snapshot contains:

- current `ApplicationSessionManifestV2` and `ApplicationSessionStateV2`;
- only the last lifecycle request/event;
- at most one close request, journal and receipt;
- the immutable save image needed by a `Prepared` close.

Publication writes and reopens the inactive slot, then atomically replaces
`CURRENT`. A corrupt/incomplete candidate leaves the prior slot current.
Content-addressed session objects, generation object packs, lifecycle archives,
recovery-evidence archives and session-object indexes do not exist.

## Save, Load, suspend and crash

- Durable world publication happens only on explicit player Save and
  save-on-close.
- Suspend records the lifecycle transition but does not publish a world save.
- Crash loses progress since the newest compatible explicit/final save. There
  is no periodic 30-tick recovery checkpoint and no wall-time catch-up.
- Explicit Load is legal only from `Suspended`. It selects the newest
  compatible save through the production `SaveStore`, replaces authoritative
  world state atomically and remains `Suspended`.
- Restart follows the same newest-compatible-save rule. The first rebuilt
  presentation uses a fresh epoch, sequence `0` and previous=current camera
  samples; simulation waits for explicit Resume.
- Corrupt/incompatible save data fails before partial world mutation and leaves
  the current session/world available.

## Exactly-once close

Current close contracts are `CloseSessionRequestV2`,
`CloseSessionJournalV2` and `CloseSessionReceiptV2`. The journal has two stages:

```text
Prepared { close identity, immutable save image/hash }
SavePublished { same identity/image, save_generation_hash }
```

Close proceeds as follows:

1. Validate the request in `Active` or `Suspended`, stop new external proposals
   at the declared boundary and transition through `Quiescing` to `Finalizing`.
2. Build one immutable final save image and atomically publish the `Prepared`
   journal with it.
3. Publish that exact image through the ordinary two-slot `SaveStore` and
   atomically advance the journal to `SavePublished`.
4. Publish `Finalizing → Closed` and one `CloseSessionReceiptV2`, then release
   platform adapters.

After a crash in `Prepared`, retry republishes byte-identical save input. After
`SavePublished`, retry completes close without creating another generation.
There are no attempt counters, deadlines, retry policies, failure dispositions,
last-safe close result, `SessionFinalSaveLedgerV1` or linked recovery session.
A save failure leaves typed close progress in `Finalizing`; it is not silently
reported as success.

## Failure semantics and checks

Malformed/stale/colliding platform or lifecycle input, illegal edge, wrong
project/capability binding, slot corruption, publication fault, unsupported
format or save failure rejects the uncommitted operation and preserves the
previous complete state. Stable diagnostics carry IDs and expected/actual
revisions/hashes.

`play` covers the interactive lifecycle, focus/device behavior and explicit
Resume. `persistence-replay` covers Save → change → Load rollback, close crash
boundaries and exact roots. `platform` is conditional for host/renderer changes.
Focused session-store tests inject faults before and after slot/pointer/save
publication and prove one final save generation per close identity.
Focused creator tests compare repeated authoring runs with the package-embedded
run and require the same exact lock, state/ledger roots, close receipt and final
save generation. Shared application/runtime/save changes also run `play` and
`persistence-replay`; this does not claim interactive creator support.

Current interactive host evidence is collected on native Linux. Windows host
execution is deferred under ADR-082; its absence leaves Windows/R1/R7 claims
open but does not block a Linux feature-development handoff.
