# Arcane world architecture — current task state

| Field | Value |
|---|---|
| Status | `READY_FOR_ARCANE_PACKAGE_A0` |
| Updated | `2026-08-16` |
| Task key | `arcane-world-architecture` |
| Scope | Proposed world-substrate composition and bounded physical-magic specifications |
| Definition of done | SPEC-39/SPEC-40/ADR-074, SPEC-37 linkage, standalone roadmap, routing/traceability and R8 status are coherent; no runtime/public-contract claim |
| Authority | Working context only; Accepted SPEC/ADR, main roadmap and future exact profiles/ProductCheck evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** owner/transaction architecture is specified;
  implementation is blocked on [A0 law/profile closure](../../plans/arcane-world/README.md#a0-blockers).
- **First consumer:** one package-authored telekinesis ability, one sealed
  caster reservoir and one real PhysX rigid fixture through production
  `EffectRequestV1`/`WorldCommand` paths.
- **Authority:** Runtime owns command/schedule/commit; RPG owns skills and
  character aggregates; Mechanics owns definitions/package state; Arcane owns
  its resource/execution state; PhysX alone owns rigid state.
- **Transaction:** reserve resource, derive one canonical force/torque batch,
  integrate PhysX once, then publish arcane debit + rigid state + receipt/events
  atomically or publish nothing.
- **Activation gate:** main R8 remains `PLANNED / NOT_ACTIVE` until
  `ARCANE-RESERVOIR-REF-P1 = PASS`.
- **Current blocker:** exact conservation/source law, units, fixed-point scales,
  reservoir/throughput profile, target trace, force/work accounting, thresholds,
  capacities and budgets are not selected.
- **Do not retry:** universal world solver/bus, separate first-party magic API,
  scalar mana plus scripted effects, direct transforms/damage, GPU authority,
  camera/timing LOD, lossy state before exact persistence, or Vital/Identity/
  ontological scope in the first milestone.

## Evidence

| Evidence | Result | Consequence |
|---|---|---|
| [Source-paper review](../arcane-world-layer-research-2026-08-16.md) | `REPORT_ONLY` | Retains owner/coupler/source-accounting ideas and rejects speculative/conflicting infrastructure |
| [SPEC-39](../../architecture/39-world-substrate-composition.md), [SPEC-40](../../architecture/40-arcane-substrate-and-physical-magic.md), [ADR-074](../../architecture/adr/074-world-substrate-and-arcane-physical-interaction-track.md) | `Proposed` | Candidate ownership, transaction, fallback and promotion boundary only |
| [Arcane roadmap](../../plans/arcane-world/README.md) | `A0 OPEN / A1 NOT_STARTED` | No code or public schema is authorized |
| `ARCANE-*` ProductChecks | `NOT_RUN` | No reservoir, mechanics, coupling, persistence, target or performance claim is admissible |

## Decisions that constrain next work

### D-001 — `WorldDynamics` is composition, not an owner

- **Decision:** existing owners compose through Runtime's fixed schedule,
  immutable views, typed batches and atomic transactions.
- **Rejected:** generic domain trait objects, global quantity map, shared
  mutable world database and universal `CouplingGraph` public API.
- **Reconsider when:** two promoted new owners require the same proven public
  consumer contract rather than only private profile-specific batches.

### D-002 — Arcane state is separate from RPG and physics

- **Decision:** Arcane owns quantity/reservations/throughput/execution; RPG
  retains skills/inventory/current character aggregates; Mechanics definitions
  remain immutable intent; physical owners retain all physical state.
- **Rejected:** storing mana in package reducer or duplicating pose/health/
  tree/water state inside Arcane.
- **Reconsider when:** a production consumer demonstrates an atomic owner move
  with migration and non-regression evidence.

### D-003 — One telekinesis vertical first

- **Decision:** closed typed execution plan and one Arcane-to-PhysX batch are
  the smallest proof of systemic magic.
- **Rejected:** starting with three couplers, a generic spell graph, ley field,
  ecology, artifacts, healing or identity.
- **Reconsider when:** base coupling/persistence passes and a second production
  ability cannot be expressed without a shared bounded composition format.

### D-004 — Source and work accounting are mandatory

- **Decision:** every physical effect binds a finite debit, conversion/loss
  rule and destination receipt; impulse is not treated as energy by itself.
- **Rejected:** free regeneration, mana-cost constants disconnected from actual
  physical work and VFX/scripted success.
- **Stop rule:** inability to freeze and pass the A0 work/conservation corpus
  keeps the track research-only.

### D-005 — Exact state before fields and metaphysics

- **Decision:** exact active owner state/save/replay precedes regional field,
  ley, ecology, sleep or lossy summaries. Vital and Identity need separate
  owners and promotion.
- **Rejected:** soul equals mana, divine standing grants identity authority,
  or resurrection/teleportation as merely expensive base spells.

## Open A0 decisions

| Decision | Required output |
|---|---|
| Arcane law | closed/finite-source/hybrid choice, named sources/sinks and regeneration exclusion or exact rate |
| Numeric state | units, fixed-point widths/scales/bounds, optional quality/spectrum state and stable failures |
| Reservoir | initial/capacity/throughput/loss/recovery values and concurrent-execution limits |
| Schedule | exact command reservation, physical substep and atomic terminal receipt order |
| Telekinesis | body fixture, target/reference rules, force/torque curve, duration, cancel/release and work formula |
| Evidence | analytical controls, thresholds, repeats/order permutations, save/replay and Windows/Linux roots |
| Budget | incremental CPU/memory and integrated GameplayBudgetMatrix limits plus report-only stress case |

## Required context

1. [Architecture routing](../../architecture/agent-routing.md), SPEC-13/19/21
   and ADR-008/020/022/046.
2. [SPEC-37](../../architecture/37-layered-physical-world.md),
   [SPEC-39](../../architecture/39-world-substrate-composition.md),
   [SPEC-40](../../architecture/40-arcane-substrate-and-physical-magic.md) and
   [ADR-074](../../architecture/adr/074-world-substrate-and-arcane-physical-interaction-track.md).
3. [Arcane roadmap](../../plans/arcane-world/README.md).
4. [Research report](../arcane-world-layer-research-2026-08-16.md).
5. [Main roadmap](../../roadmap.md), R8 and B-10.

## Next action

1. Answer A0 law/profile/fixture questions and record exact units/bounds.
2. Freeze work-accounting controls, hashes, thresholds and budget before code.
3. Create a dedicated worktree from that documentation checkpoint.
4. Implement only the serial A1 reservoir/transfer oracle.
5. Update this state only when A0 closes or evidence changes the approach.

## Handoff

- **Workspace claim:** documentation-only Proposed architecture; no runtime,
  schema, package primitive or ProductCheck implementation.
- **Expected checks:** documentation cheap path only; Cargo/host-check and all
  executable arcane checks remain `NOT_RUN`.
- **Remaining risk:** law balance, numerical exactness, work conversion,
  atomic schedule integration, save/replay, target parity and performance are
  unmeasured.
