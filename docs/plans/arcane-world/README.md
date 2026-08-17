# Arcane world — standalone implementation roadmap

Status: `PLANNED / NOT_ACTIVE`; post-v1 isolated program. Governing candidate
architecture: [SPEC-39](../../architecture/39-world-substrate-composition.md),
[SPEC-40](../../architecture/40-arcane-substrate-and-physical-magic.md) and
[ADR-074](../../architecture/adr/074-world-substrate-and-arcane-physical-interaction-track.md).
Package A0A architecture closure is `COMPLETE`; numeric/calibration Package
A0B is `OPEN` and every `ARCANE-*` ProductCheck is `NOT_RUN`.

This directory is the resume and execution surface for a future dedicated
arcane worktree. The main [Next Engine roadmap](../../roadmap.md) keeps the
track inactive until `ARCANE-RESERVOIR-REF-P1 = PASS`. Work here cannot change
the current command, Mechanics, RPG, PhysX, save/replay or public-contract
baseline by implication.

## First product result

One package-authored telekinesis ability targets one real PhysX rigid fixture
through the production player-action, `EffectRequestV1` and `WorldCommand`
path. One sealed caster reservoir supplies a bounded debit. One canonical
force/torque exchange batch moves the body; PhysX remains the only rigid
writer. Debug overlays show the reservoir, throughput, execution phase,
exchange vector and work/loss receipt.

No heat, water, terrain, vegetation, ecology, regional field, ley network,
artifact, rune, anti-magic, healing, soul/identity, divine source,
teleportation, transformation, resurrection or matter creation belongs to the
base vertical.

## Stage graph

```text
A0A Architecture/schedule/lifecycle closure      COMPLETE
 └─ A0B Numeric/fixture/evidence calibration      OPEN / BLOCKS CODE
     └─ A1 Serial reservoir + transfer oracle     NOT_STARTED
         └─ A2 Proposed schema/schedule closure   NOT_STARTED
             └─ A3 Mechanics + Arcane ↔ PhysX     NOT_STARTED
                 └─ A4 Exact active persistence  NOT_STARTED
                     └─ A5 Production promotion  NOT_STARTED

A3 ── AT Future thermal coupling                 BLOCKED / SEPARATE OWNER
A3 ── AC Future continuum coupling               BLOCKED / SPEC-36 GATES
A4 ── AE Regional field + ecology                NOT_STARTED / LATER
A4 ── AA Artifacts/runes/anti-magic              NOT_STARTED / LATER
A4 ── AV Vital and Identity lanes                OUTSIDE BASE
```

| Stage | Exit evidence | Blocks |
|---|---|---|
| A0A | Start/exchange receipt split, owner write set, fixed-point authority, targeting/residency/identity, PhysX staging, work-law shape, failure isolation, persistence and promotion sequence are frozen | A0B; `COMPLETE` |
| A0B | Exact integer raws/scales, capacity/throughput, efficiency/loss/maintenance coefficients, cadence/duration, fixture force curve, traces, capacities, thresholds and budgets are frozen | every code stage |
| A1 | `ARCANE-RESERVOIR-REF-P1 = PASS`; serial transfer/debit/loss and failure roots are exact | main-roadmap research activation, A2 |
| A2 | Proposed consumer schemas, exact schedule/access sets and profile/fixture hashes are documentation-complete; no public contract lands yet | integrated A3 |
| A3 | `ARCANE-MECHANICS-P1 = PASS` and `ARCANE-RIGID-COUPLING-P1 = PASS`; production targeting/package start path and Arcane/PhysX substeps publish under the selected receipt split | A4, later physical couplers |
| A4 | `ARCANE-PERSISTENCE-P1 = PASS`; save/restart/replay equal uninterrupted active roots | A5, later lossy/regional representations |
| A5 | `ARCANE-CROSS-TARGET-P1 = PASS`, `play`, `content-package`, `persistence-replay`, `platform`, declared conditional performance gate and consumer-backed Accepted decision | production claim |
| AT | `ARCANE-THERMAL-P1` after a separately promoted thermal owner | no base stage |
| AC | `ARCANE-CONTINUUM-P1` after the relevant SPEC-36 owner gate | no base stage |
| AE | Separate regional-law/ecology profile and exact persistence | no base stage |
| AA | At least two artifact/spell consumers before a shared graph/circuit contract | no base stage |
| AV | Separate Vital/Identity SPEC/ADR and product consumer | no base stage |

## Selected A0A architecture

- Stage-5 `CommandReceipt` means `execution started`; it never stays pending
  through stage 8 or claims physical success.
- Arcane owns reservation, active phase and recast lock. Mechanics owns only
  the immutable V1 policy and has no mutable telekinesis reducer state.
- Active execution emits fixed-point per-substep requests without another
  package callback. Release/cancel are later `WorldCommand` values.
- Private `ArcaneRigidExchangeV1` compiles into existing PhysX external force
  requests. V1 uses force at a point plus optional free torque, not impulse.
- PhysX and Arcane build staging candidates, validate the work/loss receipt and
  publish together. Internal rollback reconstructs the prior canonical PhysX
  snapshot. A post-freeze invariant halts that world without automatic retry;
  rollback failure stops the instance.
- Exact retries terminate in the ledger. Duplicate exchange keys are fatal
  internal invariants. Concurrent executions sort canonically and reserve one
  finite-plan ceiling sequentially from the stage-5 Arcane candidate. Stage 8
  accounts published per-substep slices and never reserves the same work twice.
- A1-A5 authority is checked fixed-point only. `f64` is oracle/diagnostic-only;
  GPU is presentation/correspondence-only.
- V1 has one closed authored reservoir, no regeneration/recovery and
  `1 AQ = 1 J` pre-loss maximum work budget. Negative work does not recharge;
  stationary holding pays maintenance.
- Targeting uses the production snapshot-bound query trace. Caster and dynamic
  crate remain in one sealed pinned region for the first fixture.
- A4 adds a required Arcane owner segment to a successor composite checkpoint;
  no absent/default segment or sidecar is allowed.

## A0B blockers

Before implementation, freeze:

- exact fixed-point storage widths, fractional bits, raw bounds and stable
  numeric failure codes;
- reservoir capacity, initial quantity, safe throughput window and maximum
  reservation values;
- efficiency/conversion loss and `k_active`, `k_force`, `k_torque` coefficient
  raws for the selected work equation;
- exact existing physics cadence/profile binding, execution duration and
  release/cancel command traces;
- crate mass/shape/material/start pose, application point, force/free-torque
  curve and analytical controls;
- predeclared work/trajectory/failure thresholds and golden input/profile hashes;
- concurrent execution/batch capacities and exact same-target/cross-target
  root requirements;
- incremental CPU/memory and integrated GameplayBudgetMatrix limits.

Any missing item is an explicit blocker, not a solver or gameplay implementer
choice.

## Program invariants

- Runtime remains the only command/ledger/schedule authority.
- RPG owns skills/inventory/current character resources; Mechanics owns the
  immutable V1 ability policy and no mutable telekinesis reducer; Arcane owns
  its declared resource/execution state; PhysX owns rigid state.
- CPU checked fixed-point calculation/publication is authority; `f64` is
  oracle/diagnostic-only and GPU is presentation/correspondence-only.
- Every physical result has a named arcane source, finite debit and conversion/
  loss receipt; impulse magnitude alone is not an energy account.
- No direct transform, material deletion, damage/health update or owner-private
  buffer access.
- Exact active persistence precedes regional summaries, fields, sleep or lossy
  ecology state.
- Public contracts and a generic spell graph appear only with production
  consumers and an Accepted promotion decision.

## Worktree protocol

1. Close numeric A0B in documentation before creating arcane runtime code.
2. Create a dedicated worktree from that checkpoint and implement only the A1
   serial oracle.
3. After A1 PASS, merge the evidence checkpoint and change the main R8 row to
   an active research track.
4. Each stage uses production inputs and immutable probes; a visual effect or
   compiling type does not satisfy an exit criterion.
5. A failed frozen law/coupling gate keeps the program research-only. Changing
   the law, target scope, budget or GPU authority requires a new explicit
   decision rather than threshold tuning after results.
