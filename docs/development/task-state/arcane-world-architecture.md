# Arcane world architecture — current task state

| Field | Value |
|---|---|
| Status | `READY_FOR_ARCANE_NUMERIC_A0B` |
| Updated | `2026-08-17` |
| Task key | `arcane-world-architecture` |
| Scope | Proposed world-substrate composition and bounded physical-magic specifications |
| Definition of done | SPEC-39/SPEC-40/ADR-074, SPEC-37 linkage, standalone roadmap, routing/traceability and R8 status are coherent; no runtime/public-contract claim |
| Authority | Working context only; Accepted SPEC/ADR, main roadmap and future exact profiles/ProductCheck evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** A0A owner/schedule/coupling architecture is selected;
  implementation is blocked on [numeric A0B closure](../../plans/arcane-world/README.md#a0b-blockers).
- **First consumer:** one package-authored telekinesis ability, one sealed
  caster reservoir and one real PhysX rigid fixture through production
  `EffectRequestV1`/`WorldCommand` paths.
- **Authority:** Runtime owns command/schedule/commit; RPG owns skills and
  character aggregates; Mechanics owns immutable V1 policy and no mutable
  telekinesis reducer; Arcane owns resource/execution; PhysX alone owns rigid
  state.
- **Transaction:** stage-5 receipt means `execution started`; active Arcane
  state emits fixed-point force/free-torque requests, then stage 8 publishes
  Arcane debit + staged PhysX state + exchange receipt atomically or nothing.
- **Activation gate:** main R8 remains `PLANNED / NOT_ACTIVE` until
  `ARCANE-RESERVOIR-REF-P1 = PASS`.
- **Current blocker:** exact fixed-point raws/scales, reservoir/throughput and
  maintenance/loss values, fixture force trace, thresholds, capacities and
  budgets are not selected.
- **Do not retry:** universal world solver/bus, separate first-party magic API,
  scalar mana plus scripted effects, direct transforms/damage, GPU authority,
  camera/timing LOD, lossy state before exact persistence, or Vital/Identity/
  ontological scope in the first milestone.

## Evidence

| Evidence | Result | Consequence |
|---|---|---|
| [Source-paper review](../arcane-world-layer-research-2026-08-16.md) | `REPORT_ONLY` | Retains owner/coupler/source-accounting ideas and rejects speculative/conflicting infrastructure |
| [SPEC-39](../../architecture/39-world-substrate-composition.md), [SPEC-40](../../architecture/40-arcane-substrate-and-physical-magic.md), [ADR-074](../../architecture/adr/074-world-substrate-and-arcane-physical-interaction-track.md) | `Proposed` | Candidate ownership, transaction, fallback and promotion boundary only |
| [Gap review and selected resolutions](../arcane-world-architecture-gap-review-2026-08-17.md) | `A0A COMPLETE / A0B OPEN` | All 27 recommended architecture choices are selected; numeric calibration still blocks code |
| [Arcane roadmap](../../plans/arcane-world/README.md) | `A0A COMPLETE / A0B OPEN / A1 NOT_STARTED` | No code or public schema is authorized |
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

- **Decision:** V1 uses one closed non-regenerating reservoir and
  `1 AQ = 1 J` pre-loss work budget. Stage 5 reserves one conservative ceiling
  for the finite plan and partitions it into per-substep slices; stage 8 debits
  positive midpoint force/torque work plus maintenance/loss, refunds the
  current unused slice and never reserves it again. Negative work never
  recharges.
- **Rejected:** free regeneration, mana-cost constants disconnected from actual
  physical work and VFX/scripted success.
- **Stop rule:** inability to freeze and pass the A0B work/conservation corpus
  keeps the track research-only.

### D-005 — Exact state before fields and metaphysics

- **Decision:** exact active owner state/save/replay precedes regional field,
  ley, ecology, sleep or lossy summaries. Vital and Identity need separate
  owners and promotion.
- **Rejected:** soul equals mana, divine standing grants identity authority,
  or resurrection/teleportation as merely expensive base spells.

### D-006 — Start receipt and exchange receipt are separate

- **Decision:** stage-5 `CommandReceipt` commits Arcane execution start,
  reservation and recast lock. Stage 8 uses a separate exchange receipt and
  atomically publishes staged Arcane/PhysX candidates; completion uses the one
  existing Outcome boundary.
- **Rejected:** keeping an external command receipt pending across stages,
  PhysX-first publication, debit-first publication and same-tick re-entry.
- **Reconsider when:** a production substrate cannot express correct behavior
  through start plus exchange receipts and supplies a consumer-backed ledger
  change with replay evidence.

### D-007 — V1 is fixed-point force/free-torque with production targeting

- **Decision:** A1-A5 authoritative math is checked fixed-point; private
  `ArcaneRigidExchangeV1` compiles to existing PhysX external force requests.
  Target selection uses ADR-034 snapshot-bound production queries. The first
  caster/crate fixture is sealed and pinned in one region.
- **Rejected:** authoritative f64, impulse selection, a new public physics-step
  collection, direct package body IDs, camera/depth targets and first-fixture
  streaming.
- **Reconsider when:** the base coupling passes and a new consumer proves the
  current force/query/residency profile insufficient.

### D-008 — Failure scope and lifecycle are explicit

- **Decision:** pre-freeze expected rejection is per execution and refunds
  unused reservation. Post-freeze stale/missing state, duplicate exchange,
  numeric/work mismatch, PhysX rejection or rollback failure rejects the
  participating step and halts that world without automatic retry; rollback
  failure stops the instance.
- **Rejected:** treating internal faults as gameplay rejection or aborting all
  physics for an ordinary insufficient-resource cast.

## Open numeric A0B decisions

| Decision | Required output |
|---|---|
| Numeric state | fixed-point widths, fractional bits, raw bounds and stable failures |
| Reservoir | initial/capacity/throughput/maximum-reservation values and concurrent-execution limits |
| Work coefficients | efficiency/loss plus `k_active`, `k_force`, `k_torque` raws |
| Telekinesis fixture | crate descriptor/start pose, profile cadence, force/free-torque curve, duration and production release/cancel traces |
| Evidence | analytical/golden hashes, thresholds, repeats/order permutations, save/replay and Windows/Linux roots |
| Budget | incremental CPU/memory and integrated GameplayBudgetMatrix limits plus report-only stress case |

## Required context

1. [Architecture routing](../../architecture/agent-routing.md),
   SPEC-13/18/19/21/26 and ADR-008/019/020/022/034/046.
2. [SPEC-37](../../architecture/37-layered-physical-world.md),
   [SPEC-39](../../architecture/39-world-substrate-composition.md),
   [SPEC-40](../../architecture/40-arcane-substrate-and-physical-magic.md) and
   [ADR-074](../../architecture/adr/074-world-substrate-and-arcane-physical-interaction-track.md).
3. [Arcane roadmap](../../plans/arcane-world/README.md).
4. [Research report](../arcane-world-layer-research-2026-08-16.md).
5. [Gap review](../arcane-world-architecture-gap-review-2026-08-17.md).
6. [Main roadmap](../../roadmap.md), R8 and B-10.

## Next action

1. Answer the numeric A0B profile/fixture questions and record exact raws.
2. Freeze work-accounting coefficients, corpus hashes, thresholds and budget.
3. Create a dedicated worktree from that documentation checkpoint.
4. Implement only the serial A1 reservoir/transfer oracle.
5. Update this state only when A0B closes or evidence changes the approach.

## Handoff

- **Workspace claim:** documentation-only Proposed architecture; no runtime,
  schema, package primitive or ProductCheck implementation.
- **Expected checks:** documentation cheap path only; Cargo/host-check and all
  executable arcane checks remain `NOT_RUN`.
- **Remaining risk:** numeric balance, work coefficients, exact fixture
  response, rollback implementation, save/replay, target parity and performance
  are unmeasured.
