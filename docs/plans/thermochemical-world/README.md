# Thermochemical world roadmap

| Field | Value |
|---|---|
| Status | `PLANNED / NOT_ACTIVE` |
| Architecture | [SPEC-43](../../architecture/43-thermochemical-material-processes.md), [ADR-079](../../architecture/adr/079-thermochemical-material-process-track.md) |
| Source context | [Research-paper manifest](../../architecture/research/world-dynamics-source-papers.md) |
| Current checkpoint | `T0A COMPLETE / T0B OPEN / T1 NOT_STARTED` |
| Activation gate | `THERMOCHEM-ENTHALPY-REF-P1 = PASS` |
| v1 impact | None; optional post-v1 R8 program |

This file is the isolated source of execution order for a future
thermochemical worktree. It is planning context, not shipped architecture or
ProductCheck evidence. Current runtime, schemas, saves and PhysX behavior do
not change.

## Fixed scope

The first vertical is a sealed calorimetry bench:

- one stable water parcel and one finite thermal-reservoir parcel;
- one closed heat-transfer interface;
- reversible equilibrium water/ice phase relation;
- checked fixed-point CPU authority;
- exact mass/energy receipts, save/restart and diagnostic overlays;
- no free water, mechanical ice, atmosphere, combustion, soil moisture,
  vegetation fire, arcane heat or generic reaction/plugin API.

## Dependency DAG

```text
T0A Architecture closure (complete)
 └─ T0B Numeric/profile/corpus closure (open)
     └─ T1 Serial enthalpy/phase oracle
         └─ T2 Deterministic parcel heat transfer
             └─ T3 Production calorimetry fixture + debug presentation
                 └─ T4 Exact active save/restart
                     └─ T5 Cross-target and performance closure
                         └─ T6 Base production promotion

After T6 only:
 ├─ TC Continuum water/ice mechanics
 ├─ TV Vegetation drying/combustion
 ├─ TR Rigid thermal-material effects
 ├─ TA Arcane heat/cooling
 └─ TX Bounded reaction/combustion ladder
```

The child lanes are independent. None is a shortcut to T1-T6 and no child
inherits a PASS from base heat/phase evidence.

## Package status

| Package | Status | Exit |
|---|---|---|
| T0A Architecture | `COMPLETE` | Owner, attachment, enthalpy-first authority, fixed-step batch, atomic coupling, persistence and failure semantics are explicit in SPEC-43/ADR-079. |
| T0B Numeric/profile/corpus | `OPEN` | All blockers below are exact, hash-bound and reviewable. |
| T1 Serial oracle | `NOT_STARTED` | `THERMOCHEM-ENTHALPY-REF-P1 = PASS`; performance recorded but not blocking. |
| T2 Heat transfer | `NOT_STARTED` | `THERMOCHEM-HEAT-P1` and `THERMOCHEM-PHASE-P1` pass order/fault corpus. |
| T3 Production fixture | `NOT_STARTED` | Real content/owner activation and immutable presentation use production paths; no test-only mutation. |
| T4 Persistence | `NOT_STARTED` | `THERMOCHEM-PERSISTENCE-P1 = PASS`; corrupt closure fails before mutation. |
| T5 Target/performance | `NOT_STARTED` | `THERMOCHEM-CROSS-TARGET-P1` plus frozen conditional budget pass. |
| T6 Promotion | `NOT_STARTED` | Consumer-backed Accepted ADR and only required current contracts land together. |

## T0B blockers

No solver code starts until one reviewed profile fixes:

- fixed-point descriptors for mass, enthalpy, temperature projection, phase
  fraction, heat flow and every checked intermediate;
- exact water/ice mass, initial states, heat capacities, transition enthalpy,
  phase interval convention and piecewise lookup bytes;
- exact reservoir state, interface geometry, conductance law, fixed 240 Hz
  interval and run duration;
- maximum parcels/interfaces/transfers, decoded bytes and failure codes;
- independent oracle implementation/version and golden fixture hashes;
- conservation and curve thresholds, N-1/N/N+1 faults, order/worker repeats
  and save-at-N/resume-to-M points;
- THOTH incremental CPU/memory and integrated GameplayBudgetMatrix rows plus
  one report-only stress profile.

Absence of any value is a blocker, not solver discretion.

## Execution rules

- Use one sealed always-active fixture through T4. Streaming, parcel transfer,
  lossy sleep and representation conversion are excluded.
- T1 is serial correctness only. Record performance but do not optimize before
  the oracle passes.
- T2 may parallelize only with fixed logical shards, canonical reductions and
  exact roots independent of worker count.
- T3 uses authored project activation and immutable committed projections.
  Presentation is debug-only and never authority.
- T4 creates no sidecar. Exact Thermochemical state participates in one
  successor composite checkpoint with every required attachment owner.
- After two evidence-backed performance optimization cycles without meeting
  the frozen gate, retain research status. Changing workload, budget or
  authority requires an explicit decision.

## Promotion and fallback

The main roadmap stays `PLANNED / NOT_ACTIVE` through T0B. T1 may run in an
isolated worktree after T0B. Only `THERMOCHEM-ENTHALPY-REF-P1 = PASS` permits
an active R8 integration row.

Before activation, authored static material state is allowed. After
activation, ambient-temperature reset, decorative ice/fire, frozen
thermochemistry, retry-to-green or backend switch is forbidden. A fatal
thermochemical/coupling failure retains the prior complete checkpoint and
stops the affected world.

## Handoff checklist

- [ ] T0B profile and corpus hashes are frozen.
- [ ] Worktree begins from a clean documentation checkpoint.
- [ ] Evidence and heavy outputs use the external store; no datasets or runs
  enter the repository.
- [ ] Every status transition names the exact ProductCheck artifact.
- [ ] Main roadmap changes only when the activation or promotion fact changes.
