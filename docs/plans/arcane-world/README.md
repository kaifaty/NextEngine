# Arcane world — standalone implementation roadmap

Status: `PLANNED / NOT_ACTIVE`; post-v1 isolated program. Governing candidate
architecture: [SPEC-39](../../architecture/39-world-substrate-composition.md),
[SPEC-40](../../architecture/40-arcane-substrate-and-physical-magic.md) and
[ADR-074](../../architecture/adr/074-world-substrate-and-arcane-physical-interaction-track.md).
Package A0 is open and every `ARCANE-*` ProductCheck is `NOT_RUN`.

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
A0 Law/profile/fixture/evidence closure          OPEN / BLOCKS CODE
 └─ A1 Serial reservoir + transfer oracle        NOT_STARTED
     └─ A2 Production mechanics execution path   NOT_STARTED
         └─ A3 Atomic Arcane ↔ PhysX telekinesis NOT_STARTED
             └─ A4 Exact active persistence      NOT_STARTED
                 └─ A5 Production promotion      NOT_STARTED

A3 ── AT Future thermal coupling                 BLOCKED / SEPARATE OWNER
A3 ── AC Future continuum coupling               BLOCKED / SPEC-36 GATES
A4 ── AE Regional field + ecology                NOT_STARTED / LATER
A4 ── AA Artifacts/runes/anti-magic              NOT_STARTED / LATER
A4 ── AV Vital and Identity lanes                OUTSIDE BASE
```

| Stage | Exit evidence | Blocks |
|---|---|---|
| A0 | Exact source/conservation law, units, reservoir/throughput profile, fixed-point scales, schedule, target fixture, force/work formula, traces, capacities, thresholds and budgets are frozen | every code stage |
| A1 | `ARCANE-RESERVOIR-REF-P1 = PASS`; serial transfer/debit/loss and failure roots are exact | main-roadmap research activation, A2 |
| A2 | `ARCANE-MECHANICS-P1 = PASS`; first-party/data/Luau/Wasm use the same bounded package/command path | A3 |
| A3 | `ARCANE-RIGID-COUPLING-P1 = PASS`; telekinesis debit and PhysX result publish atomically | A4, later physical couplers |
| A4 | `ARCANE-PERSISTENCE-P1 = PASS`; save/restart/replay equal uninterrupted active roots | A5, later lossy/regional representations |
| A5 | `ARCANE-CROSS-TARGET-P1 = PASS`, declared performance gate and consumer-backed Accepted decision | production claim |
| AT | `ARCANE-THERMAL-P1` after a separately promoted thermal owner | no base stage |
| AC | `ARCANE-CONTINUUM-P1` after the relevant SPEC-36 owner gate | no base stage |
| AE | Separate regional-law/ecology profile and exact persistence | no base stage |
| AA | At least two artifact/spell consumers before a shared graph/circuit contract | no base stage |
| AV | Separate Vital/Identity SPEC/ADR and product consumer | no base stage |

## A0 blockers

Before implementation, freeze:

- closed, finite-source or hybrid law; the recommended oracle starts with a
  closed sealed reservoir and no regeneration;
- exact arcane unit and conversion relation to bounded mechanical work;
- reservoir capacity, initial quantity, safe throughput window, loss and
  recovery policy;
- whether potential/coherence/entropy/spectrum are immutable profile values or
  justified mutable state;
- fixed tick/substep order, integer scales/bounds and stable failure codes;
- target body/scene, command trace, application point, force/torque curve,
  duration and release/cancel behavior;
- conservation/work formula, analytical controls and predeclared thresholds;
- concurrent execution/batch capacities and exact same-target/cross-target
  root requirements;
- incremental CPU/memory and integrated GameplayBudgetMatrix limits.

Any missing item is an explicit blocker, not a solver or gameplay implementer
choice.

## Program invariants

- Runtime remains the only command/ledger/schedule authority.
- RPG owns skills/inventory/current character resources; Mechanics owns
  definitions/package state; Arcane owns only its declared resource/execution
  state; PhysX owns rigid state.
- CPU fixed-point publication is authority; private `f64` cannot survive an
  accepted step and GPU is presentation/correspondence only.
- Every physical result has a named arcane source, finite debit and conversion/
  loss receipt; impulse magnitude alone is not an energy account.
- No direct transform, material deletion, damage/health update or owner-private
  buffer access.
- Exact active persistence precedes regional summaries, fields, sleep or lossy
  ecology state.
- Public contracts and a generic spell graph appear only with production
  consumers and an Accepted promotion decision.

## Worktree protocol

1. Close A0 in documentation before creating arcane runtime code.
2. Create a dedicated worktree from that checkpoint and implement only the A1
   serial oracle.
3. After A1 PASS, merge the evidence checkpoint and change the main R8 row to
   an active research track.
4. Each stage uses production inputs and immutable probes; a visual effect or
   compiling type does not satisfy an exit criterion.
5. A failed frozen law/coupling gate keeps the program research-only. Changing
   the law, target scope, budget or GPU authority requires a new explicit
   decision rather than threshold tuning after results.
