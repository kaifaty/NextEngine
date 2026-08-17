# World dynamics architecture — current task state

| Field | Value |
|---|---|
| Status | `READY_FOR_THERMOCHEMICAL_T0B_AND_CLASSICAL_GATES` |
| Updated | `2026-08-17` |
| Task key | `world-dynamics-architecture` |
| Scope | Unified owner/coupling architecture for physical, thermochemical, arcane and optional neural-assisted simulation |
| Definition of done | Source papers preserved as non-normative input; SPEC-39/37/36/38/40 links are coherent; SPEC-41/42 and ADR-075/076 define bounded tracks; roadmaps, routing, traceability and R8 status agree; no runtime/public-contract claim |
| Authority | Working context only; Accepted SPEC/ADR, main roadmap and exact ProductCheck evidence outrank this file |

## Resume in 60 seconds

- **World model:** `WorldDynamics` is fixed-stage owner composition, not a
  state owner or service.
- **New owner:** Thermochemical owns parcel composition, enthalpy, equilibrium
  phase and reaction progress; physical owners retain motion/topology.
- **First thermochemical fixture:** sealed water parcel + finite thermal
  reservoir; heat/ice equilibrium only.
- **Neural boundary:** model is optional stateless advice to one already
  promoted classical owner; N0/N1 only, no authoritative surrogate.
- **No retry:** invalid advice is rejected before solve; failure after an
  admitted proposal fails the owner step.
- **Arcane link:** telekinesis base track is unchanged; heat/cooling is a later
  Arcane↔Thermochemical transaction after both base tracks promote.
- **Activation:** Thermochemical R8 waits for
  `THERMOCHEM-ENTHALPY-REF-P1`; neural work waits for a promoted classical
  owner plus a measured bottleneck.
- **Current blockers:** Thermochemical T0B numeric/corpus/budget closure;
  neural N1 target selection; Arcane A0B remains independently open.

## Evidence

| Evidence | Result | Consequence |
|---|---|---|
| [Imported source papers](../../architecture/research/world-dynamics-source-papers.md) | `NON_NORMATIVE` | Papers are preserved verbatim; their imperative language is not repository authority. |
| [Synthesis report](../world-dynamics-unified-architecture-research-2026-08-17.md) | `REPORT_ONLY` | Records adopted/adapted/rejected proposals and remaining uncertainty. |
| [SPEC-41](../../architecture/41-thermochemical-material-processes.md), [ADR-075](../../architecture/adr/075-thermochemical-material-process-track.md) | `Proposed` | Thermochemical owner and base heat/phase track only. |
| [SPEC-42](../../architecture/42-neural-assisted-world-simulation.md), [ADR-076](../../architecture/adr/076-neural-assistance-as-bounded-proposals.md) | `Proposed` | Optional proposal/shadow track only; no model authority or current lane. |
| `THERMOCHEM-*`, `WORLD-NEURAL-*` checks | `NOT_RUN` | No solver, dataset, model, persistence, target or performance claim. |

## Decisions

### D-001 — Composition, not universal infrastructure

- Runtime owns identity/schedule/commit; peer owners exchange immutable
  projections and typed batches.
- Reject global world database, generic coupler, conservation service,
  representation manager and speculative crate tree.
- Reconsider only after multiple promoted consumers prove one identical
  bounded contract.

### D-002 — Thermochemical is one explicit owner

- It owns stable parcel attachment, composition, enthalpy, equilibrium phase
  and durable reaction progress.
- Mechanical owners retain motion, contact, spatial mass and topology; any
  consequence is an atomic typed exchange.
- `ThermochemicalMaterialProfile` is separate from physics/continuum/tree/
  render materials.

### D-003 — Enthalpy is source state

- Temperature and base water/ice phase fractions derive from a frozen curve.
- V1 excludes hysteresis, nucleation, supercooling and metastability so no
  hidden state selects phase.
- CPU checked fixed-point is authority; f64/GPU are oracle/correspondence only.

### D-004 — Heat/phase precede chemistry breadth

- First fixture is sealed and finite; cold means enthalpy removal.
- Combustion is a later reaction/transport profile, not `FireDomain` or VFX.
- Atmosphere, arbitrary plugin reactions, load-bearing ice and material mass
  transfer wait for separate consumer gates.

### D-005 — Models propose, owners decide

- N0 diagnostics and N1 bounded initialization proposals are the only first
  tiers. The model owns no world state and does not extend `ModelLaneV1` yet.
- Classical reference, production baseline and exact persistence come first.
- Missing/invalid advice uses the same deterministic classical initialization;
  admitted-advice failure is not retried.

### D-006 — Exact non-regression is promotion law

- Approximate field metrics are shadow diagnostics only.
- Runtime promotion requires exact roots, outcome/event order and failure
  classes across target and continuation evidence plus end-to-end performance.
- After two unsuccessful evidence-backed cycles, stop the branch rather than
  weaken product or authority gates.

### D-007 — Arcane integration is downstream

- Base telekinesis A-track is unchanged.
- Arcane heat/cooling debits real Arcane quantity and changes real
  thermochemical enthalpy atomically only after both owners promote.
- That edge is the first plausible `WORLD-DYNAMICS-P1` consumer; a model does
  not count as an owner.

## Open decisions

| Track | Required closure |
|---|---|
| Thermochemical T0B | Numeric widths/scales, exact water/ice curve, reservoir/interface law, capacities, golden corpus, thresholds and THOTH budget. |
| Neural N1 | One promoted owner, stable measured bottleneck, bounded proposal shape, shadow comparator and end-to-end value hypothesis. |
| Arcane A0B | Existing reservoir/work/profile blockers in the dedicated arcane task state. |
| Child couplers | Separate profiles and evidence for continuum/ice, rigid thermal effects, vegetation combustion and Arcane heat. |

## Required context

1. [Architecture routing](../../architecture/agent-routing.md), SPEC-21/26 and
   ADR-022/046/053/058.
2. [SPEC-37](../../architecture/37-layered-physical-world.md),
   [SPEC-39](../../architecture/39-world-substrate-composition.md),
   [SPEC-41](../../architecture/41-thermochemical-material-processes.md) and
   [SPEC-42](../../architecture/42-neural-assisted-world-simulation.md).
3. [Thermochemical roadmap](../../plans/thermochemical-world/README.md) and
   [neural roadmap](../../plans/neural-world-physics/README.md).
4. [Arcane task state](arcane-world-architecture.md) for independent A0B work.
5. [Main roadmap](../../roadmap.md), R8.

## Next action

1. Answer and freeze Thermochemical T0B numeric/profile questions.
2. Run only the serial enthalpy/phase oracle after that checkpoint.
3. Do not select or train a neural target until one classical promoted owner
   supplies the N1 evidence.

## Handoff

- Documentation-only Proposed architecture; no runtime, schema, model lane,
  solver, package primitive or ProductCheck implementation.
- Documentation cheap path only. Cargo/host-check and every new executable
  check remain `NOT_RUN`.
- Remaining risk is numerical calibration, physical coupling correctness,
  exact persistence, cross-target parity and measured performance.
