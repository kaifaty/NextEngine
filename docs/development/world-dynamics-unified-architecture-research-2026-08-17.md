# Unified world-dynamics architecture synthesis — 2026-08-17

| Field | Value |
|---|---|
| Status | `REPORT_ONLY` working rationale |
| Inputs | [Imported source-paper manifest](../architecture/research/world-dynamics-source-papers.md) |
| Normative result | [SPEC-41](../architecture/41-world-substrate-composition.md), [SPEC-43](../architecture/43-thermochemical-material-processes.md), [SPEC-44](../architecture/44-neural-assisted-world-simulation.md), [ADR-079](../architecture/adr/079-thermochemical-material-process-track.md), [ADR-080](../architecture/adr/080-neural-assistance-as-bounded-proposals.md) |
| Authority | This report does not override SPEC/ADR status or ProductCheck evidence. |

## Question

How can the physical, thermochemical and arcane proposals form one causal
world without a shared mutable database, duplicate state writers, speculative
public infrastructure or a learned solver becoming authority?

## Selected architecture

```text
L0 Project/profile/capability closure
 ↓
L1 Runtime identity, fixed schedule, command ledger, atomic publication
 ↓
L2 Mechanics/RPG validation + immutable owner projections
 ↓
L3 Peer state owners
    ├─ PhysX rigid/articulated
    ├─ Continuum water/terrain (Proposed)
    ├─ Living structures (Proposed)
    ├─ Thermochemical parcels (Proposed)
    └─ Arcane reservoir/execution (Proposed)
 ↓
L4 Profile-specific exchange batches + composite commit
 ↓
L5 Owner segments, semantic queries and committed outcomes
 ↓
L6 Read-only presentation and diagnostics

Optional neural assistant → bounded proposal to one L3 owner candidate
                          → never an owner, law or commit path
```

`WorldDynamics` names this composition. It is not a service, crate, scheduler,
database, generic coupler or state owner.

## Adopted and adapted ideas

| Source idea | Next Engine resolution |
|---|---|
| Enthalpy-first thermochemistry | Adopted as a Proposed Thermochemical owner with fixed-point canonical state and a sealed water/ice fixture. |
| Species, phase and reaction progress | Adopted only as bounded parcel state; arbitrary reactions, atmosphere and global registries are deferred. |
| Conservation ledger | Adapted into edge-local receipts inside the atomic composite transaction; no global mutable ledger service. |
| Material coupling graph | Adapted into separately versioned typed exchange profiles; no public generic graph. |
| Fire as coupled physics/chemistry | Adopted as a later reaction/transport profile, not a domain or presentation fact. |
| Neural warm starts | Adopted only as post-promotion report-only proposals to an existing classical owner. |
| Classical correction and fallback | Narrowed: invalid advice is rejected before solve; after admitted advice there is no retry. Exact roots are required for promotion. |
| Trust ladder | N0 diagnostics and N1 proposals retained; learned corrections, constitutive surrogates and world evolution are not admitted. |
| Arcane source/sink accounting | Retained from SPEC-42; future heat/cooling must exchange real enthalpy with Thermochemical state. |

## Rejected architecture shortcuts

- one universal `WorldState`, `WorldDynamicsService`, `CouplingGraph`,
  `ConservationLedger` or `RepresentationManager`;
- generic public domain traits, raw solver buffers or plugin callbacks before
  multiple production consumers;
- temperature stored independently beside authoritative enthalpy;
- Arcane direct ignition/freezing, scripted damage or material deletion;
- fire, cold, steam or ice inferred from VFX;
- GPU or learned solver authority, tolerance-based canonical equality and
  automatic backend switching;
- retrying the classical solve after an admitted learned proposal fails;
- online training or hidden recurrent state in world simulation;
- lossy sleep, regional summaries or sidecar saves before exact active
  persistence.

## Ownership closure

The new Thermochemical owner writes composition, enthalpy, equilibrium phase
and reaction progress of stable parcels. Mechanical owners write motion,
contact, spatial mass properties and topology. Arcane writes quantity,
reservation and execution. Runtime writes schedule/ledger/commit state. A
learned component writes nothing authoritative.

Cross-owner effects are explicit:

- heat transfer moves equal-and-opposite fixed-point energy between parcels;
- future phase/mechanical change updates Thermochemical and the destination
  physical owner atomically;
- future combustion updates composition/enthalpy and separately validated
  structural/continuum/rigid effects;
- arcane heating debits Arcane and changes thermochemical enthalpy in one
  composite transaction;
- a model may propose private solver initialization but cannot affect the
  owner boundary unless exact-root non-regression is proven.

## Dependency decisions

```text
Classical owner reference → production baseline → exact persistence
                                      ├─ domain integration/promotion
                                      └─ optional neural data/shadow branch

Thermochemical T0B → enthalpy oracle → heat/phase → exact persistence
                                                   ├─ continuum/ice
                                                   ├─ vegetation/combustion
                                                   └─ arcane heat/cooling

Arcane A0B → reservoir oracle → rigid telekinesis → exact persistence
                                                   └─ later Arcane↔Thermochemical
```

The first Arcane-to-Thermochemical transaction can become the first candidate
for `WORLD-DYNAMICS-P1` because it combines two independently promoted new
substrate owners. Neural assistance never counts toward that threshold.

## Remaining uncertainty

- Thermochemical T0B has no selected numeric scales, material curve,
  conductance fixture, thresholds or performance budget.
- No classical owner has been selected as the first neural target; selection
  waits for promoted production evidence and a measured bottleneck.
- Reaction kinetics, mass-transfer topology, atmosphere, load-bearing ice,
  tree combustion and arcane heat are deliberately unspecific child tracks.
- Public schema shapes and successor checkpoint numbers wait for real
  production consumers under ADR-046.

## Smallest next actions

1. Close Thermochemical T0B and run only the serial enthalpy/phase oracle.
2. Keep Arcane on its existing A0B numeric closure and rigid telekinesis path.
3. Select no neural model target until a classical promoted owner produces a
   stable, instrumented performance bottleneck.
