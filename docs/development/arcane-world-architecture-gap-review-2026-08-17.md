# Arcane architecture gap review and selected resolutions — 2026-08-17

Status: `ARCHITECTURE_DECISIONS_SELECTED / NUMERIC_CALIBRATION_OPEN`.

## Scope and authority

This review checked SPEC-41/SPEC-42/ADR-078 against the Accepted command
ledger, fixed schedule, Mechanics/RPG ownership, PhysX fixed-step publication,
targeting replay, persistence and consumer-driven contract rules. It is
engineering rationale, not normative authority. The updated SPEC/ADR and
roadmap carry the durable semantics.

The user selected the recommended option for every question in the 27-item
architecture-gap questionnaire. No runtime, public schema or ProductCheck
implementation exists as a result.

## Gap closure

| Gap | Selected resolution | Consequence |
|---|---|---|
| External command could not remain pending from stage 5 through PhysX stage 8 | Stage-5 receipt means `execution started`; stage 8 publishes a separate exchange receipt; completion uses existing Outcome | Current ledger barriers remain unchanged |
| Mutable cooldown/phase had no complete write set | Arcane owns active phase/recast lock; Mechanics owns immutable V1 policy and no mutable telekinesis reducer | Arcane/PhysX are the only new stage-8 writers |
| PhysX could publish before debit validation | PhysX and Arcane produce staging candidates and publish together; failure reconstructs the prior canonical PhysX snapshot | No partial physical/debit result; rollback failure is fatal |
| Multi-tick execution risked package callbacks/re-entry | Active Arcane execution emits per-substep requests; release/cancel are later WorldCommands | Package code runs only through the production proposal path |
| Coupler duplicated the physics API | Private `ArcaneRigidExchangeV1` compiles into existing external force requests | No new public physics-step collection or generic bus |
| Retry and duplicate semantics conflicted | Ledger absorbs exact command retry; duplicate exchange key inside a new closed batch is fatal | Retry remains idempotent and scheduling bugs fail closed |
| Concurrent casts had no deterministic reservation order | Canonical `(reservoir, command, execution, body)` order; stage 5 sequentially reserves each finite-plan ceiling and stage 8 accounts its published per-substep slice without a second reservation | Worker/arrival order cannot overspend a reservoir |
| Ordinary rejection and internal failure were conflated | Expected pre-freeze rejection is per execution; post-freeze invariant rejects the participating step and halts that world without automatic retry | Other valid executions survive ordinary rejection only |
| Private f64 could affect authority | A1-A5 uses checked fixed-point only; f64 is oracle/diagnostic-only | SPEC-21 numeric baseline remains intact |
| Force versus impulse was open | V1 uses force at an application point plus optional free torque; impulse is excluded | Work accounting has one fixed input model |
| Debit was disconnected from physical work | Reserve a conservative maximum, debit positive midpoint work plus maintenance/loss, refund unused | Stationary holding costs resource; negative work does not recharge |
| Arcane source/units were open | One closed authored reservoir, no regeneration/recovery, `1 AQ = 1 J` pre-loss work budget | The first law is falsifiable and bounded |
| Player target provenance was missing | Reuse PlayerActionFrame, TargetingIntent, snapshot-bound physics query, EffectRequest and WorldCommand | Camera/depth/package body IDs cannot choose authority |
| Residency/lifecycle was undefined | Caster and dynamic crate are pinned in one sealed active region; no transfer/despawn in fixture | Cross-region and lifecycle transfer remain later lanes |
| Reservoir/execution identity was ambiguous | Authored reservoir slot bound to caster PersistentId; execution derives from causal command; exact world tuple replaces `world generation` | Save/replay and batch keys have existing identity sources |
| Public contract stage preceded a working consumer | A2 is Proposed documentation/schema/schedule closure; public contracts land only with integrated A3 | ADR-046 consumer-driven rule remains intact |
| Persistence allowed an implicit optional owner | A4 creates a required Arcane owner segment in a successor current-only composite checkpoint | No zero/default segment or sidecar downgrade |
| Debug could read owner internals | Future bounded `ArcanePresentationSnapshotV1` is the only debug projection | Presentation remains read-only |
| Promotion omitted mapped checks | A5 includes all five Arcane checks plus play/content-package/persistence-replay/platform and conditional performance | Traceability and standalone roadmap agree |
| WORLD-DYNAMICS trigger was immediate/ambiguous | First Arcane/PhysX edge uses its coupling check; WORLD-DYNAMICS waits for two new substrate owners beyond Physical Embodiment | No redundant first-vertical gate |

## Remaining A0B uncertainty

Architecture A0A is closed, but code remains blocked until one exact numeric
profile freezes:

- fixed-point storage widths, fractional bits and raw bounds;
- reservoir initial/capacity/throughput/reservation values;
- efficiency, conversion loss and maintenance coefficient raws;
- cadence/profile binding, duration and release/cancel traces;
- crate definition, start pose, application point and force/free-torque curve;
- analytical/golden corpus hashes, thresholds and capacity boundaries;
- incremental CPU/memory and integrated THOTH budget.

The smallest next action is a numeric A0B questionnaire/calibration pass. A1
must not begin from placeholder values or an implementer-selected balance.
