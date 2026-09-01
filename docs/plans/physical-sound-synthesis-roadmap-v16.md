# Roadmap V16: decoupled neural learning and automatic admission

| Field | Value |
| --- | --- |
| Rebaseline date | `2026-09-01` |
| Status | `ACTIVE_R&D / L0A_PROTOCOL_FROZEN / L0B_TRUTH_EXECUTION_NEXT / SOURCE_GROWTH_PARALLEL / ADMISSION_BLOCKED / RUNTIME_NOT_AUTHORIZED` |
| Replaces | [Roadmap V15](physical-sound-synthesis-roadmap-v15.md) as the active execution plan |
| Evidence basis | [V16 decoupled-learning rebaseline](../development/physical-sound-v16-decoupled-learning-admission-rebaseline-2026-09-01.md) and [V15 S0c](../development/physical-sound-v15-s0c-source-sufficiency-role-freeze-result-2026-09-01.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Mandatory fallback | Existing authored/recorded clip for every reject, OOD, unsupported material, missing source or tooling failure |

## Goal

Teach an offline tool to turn object geometry, material family, scale, support,
surface contact and a canonical energy bin into a compact modal sound field,
then bake ordinary deterministic 48 kHz clips for the existing engine audio
path.

V16 is organized around three independent questions:

1. **Can it learn the mathematics?** Test a structured neural field against
   engine-owned modes, damping and spatial gains with exact known truth.
2. **Does it improve real sounds?** On disclosed generator-only published data,
   require it to beat nearest-contact and classical modal controls.
3. **Can automation be trusted?** On fresh protected real objects, freeze an
   independent validator and open one shadow exactly once.

Only the third answer can authorize a cooked research vertical. No result in
this roadmap directly promotes a production contract or runtime model.

## Admitted research shape

```text
CanonicalImpactAtlasV1
  mesh + material family + metric scale + support class
       + surface contact + canonical energy bin
    -> object-global modal frequency and positive damping
       + bounded contact-conditioned modal gains
       + calibrated uncertainty/OOD
    -> deterministic dry clips or authored fallback
```

The claim is bounded plausibility and contact consistency inside a declared
domain. It excludes true material-constant recovery, arbitrary-force transfer,
microphone/room radiation, rolling, scraping, fracture and runtime inference.

## Evidence lanes

| Lane | Input | Output | Promotion rule |
| --- | --- | --- | --- |
| L — learnability | deterministic known-truth scenes and adversarial mutations | `Capability` certificate | May open generator-only real development; never admission. |
| R — real candidate | exactly identified generator train/development groups | `CandidateQuality` certificate | May nominate one immutable generator release; never calibrate itself. |
| S — source growth | metadata, revision identity, member/axis certificates and bounded published-source research | generator roles plus sealed protected roles | Protected roles remain unread until L/R nominate a candidate. |
| V — validation | synthetic/T4 mutation scaffolding, then protected calibration and method holdout | immutable validator release | Thresholds freeze before holdout; generator cannot read protected scores. |
| A — admission | one untouched protected shadow | `Pass`, `Reject` or `FallbackOutOfDomain` | Open exactly once; no revision follows. |
| C — product experiment | admitted generator, validator decision and fallback map | cooked atlas and bounded demo | Research-only until a separate Accepted ADR and product check. |

The lanes share hashes and schemas, not mutable training state. A role never
moves from generator to protected after signal access.

## Data invariants

Every material admitted to A retains the complete V15 shape:

```text
4 generator train + 1 generator development
                  + 1 validator calibration
                  + 1 method holdout
                  + 1 admission shadow
```

- All eight roles are distinct physical object/recording-parent groups.
- Protected roles are fresh, publisher-revision disjoint,
  `evaluation_complete` T2/T3 groups.
- Explicitly exposed T2/T3 data may be generator-only when a known schema proves
  its historical role; unknown exposure remains protected/unknown and unusable.
- T0/T1 synthetic data may improve learning but never protected credit.
- T4 audio/video data may train semantic or artifact specialists but never
  provide physical admission credit.
- Datasets, PCM, arrays, weights, reports and generated WAVs remain in the
  external experiment store.
- Published internet evidence is the only real-data acquisition path; the user
  records or strikes nothing.

## Model tournament

The first candidate is deliberately small:

1. deterministic mesh sampling and local geometry descriptors;
2. an object encoder predicting ordered stable frequencies and positive damping;
3. a surface field predicting bounded signed or complex modal gains;
4. an uncertainty head for geometry/contact/support OOD;
5. the existing differentiable modal renderer and deterministic clip cooker.

Required controls are material mean, nearest object, nearest contact, Euclidean
RBF, geodesic RBF, local linear interpolation and a geometry-agnostic model.
Mesh-only is the primary candidate. Image features, 3DGS features, an energy-
bounded residual and direct waveform generation are later ablations; none may
rescue a failed primary tournament.

## Execution graph

```text
R0 V16 freeze
   |
   +--> L0 truth protocol --> L1 truth tournament --> R1 real tournament --+
   |                                                                    |
   +--> S0 exposure roles --> S1 source adapters --> S2 protected freeze +
   |                                                                    |
   +--> V0 validator scaffold --------------------------> V1 release ----+
                                                                         |
                                                          A0 shadow once
                                                                         |
                                              Pass --> C0 cooker --> D0 demo
                                              Reject/OOD --> authored fallback
```

L0 starts without S0–S2. R1 needs a passing L1 and five exact generator groups.
V1 needs an immutable R1 generator plus protected calibration/holdout. A0 needs
all three lanes and cannot be simulated by development data.

## Milestones

| ID | Package | State | Observable exit criterion |
| --- | --- | --- | --- |
| R0 | V16 rebaseline | `COMPLETE` | Capability, candidate-quality and admission claims are separated without changing V15 data minima or SPEC-45. |
| L0 | Known-truth protocol | `COMPLETE / FROZEN_BEFORE_IMPLEMENTATION` | [L0a](../development/physical-sound-v16-l0a-known-truth-neural-oracle-protocol-2026-09-01.md) freezes 18 synthetic object groups, train/development/test topology splits, structured and equal-budget ablation models, classical controls, mutations, metrics, compute ceiling, seeds and stop rule with zero real access. |
| L1 | Known-truth tournament | `NEXT / PROTOCOL_FROZEN` | Two exact reruns recover stable poles/damping and held surface gains, beat every compatible classical control and reject wrong scale/material, contact shuffle, instability and coverage collapse. |
| S0 | Exposure-role recovery | `PARALLEL / ZERO_SIGNAL` | Reclassify every historical Metal alias through an allowlist of known manifest schemas; emit generator-only, protected or unknown evidence with zero source body/signal access. |
| S1 | Published-source adapters | `BLOCKED_BY_S0` | Prove per-object identity, exact member, geometry/scale, contact, excitation, listener, response, support and freshness for bounded ObjectFolder/RealImpact/YCB/other candidates, or close each source with a machine-readable reason. |
| S2 | Metal role freeze | `BLOCKED_BY_S1` | Freeze five generator-ready and three fresh evaluation-complete Metal groups with immutable hashes and no protected signal decode. |
| V0 | Validator scaffold | `AFTER_L0 / NO_ADMISSION_CREDIT` | Implement hard/provenance checks, physics invariants, acoustic mutations, semantic/OOD interfaces and group-aware reporting on synthetic/T4 fixtures; thresholds remain explicitly draft. |
| R1 | Disclosed real tournament | `BLOCKED_BY_L1_AND_5_GENERATOR_GROUPS` | On frozen Metal train/development roles, the structured candidate beats compatible controls on spectrum, onset, envelope, decay, modal persistence and contact continuity; otherwise stop the family. |
| V1 | Independent validator release | `BLOCKED_BY_R1_AND_S2` | Calibrate without generator access, freeze thresholds, then publish method-holdout false-pass/coverage evidence and immutable validator hashes. |
| A0 | Metal shadow admission | `BLOCKED_BY_V1` | Open one untouched Metal shadow exactly once and emit immutable `Pass`, `Reject` or `FallbackOutOfDomain`; no model or threshold revision follows. |
| C0 | Deterministic atlas cooker | `BLOCKED_BY_A0_PASS` | Canonical surface regions and energy bins cook twice to byte-identical clips, metadata, hashes and a complete fallback map. |
| D0 | Demo-scene vertical | `BLOCKED_BY_C0` | One non-authoritative prop uses the existing contact-to-clip presentation path; fallback, replay, content-package, play and conditional performance checks pass. |
| G0 | Wood then Glass growth | `AFTER_A0` | Each material repeats S0–A0 with the same role shape and validator policy; unavailable domains remain `FallbackOnly`. |
| P0 | Production promotion | `POST_RESEARCH / ADR_REQUIRED` | A concrete player-visible prop justifies public content/projection contracts, adequate protected sample-size evidence and a new Accepted ADR. |

## Automatic validator

The validator is a separate release, not a generator loss function:

- **hard/provenance:** schema, hashes, finite bounds, role/parent/revision
  isolation and byte-repeat checks;
- **physics:** positive damping, stable modes, bounded residual energy, energy-bin
  monotonicity and declared surface continuity;
- **acoustic:** multi-resolution spectrum, onset, envelope, decay, modal
  persistence and known artifact detectors;
- **semantic:** frozen material/shape evidence and controlled wrong-material
  swaps, used only inside its measured domain;
- **selective risk:** calibrated OOD, parent-grouped confidence and retained
  coverage reported with the false-pass estimate.

V0 may prove that the machinery catches injected defects. Only V1 may claim a
real-data operating point. The minimal `1/1/1` protected research split is not
a production population guarantee; P0 requires a separate sample-size decision.
Human listening is optional audit evidence and never a per-sound queue.

## Stop rules

1. L1 failure closes real training for this model family.
2. R1 failure against a compatible simple control closes protected admission;
   the simpler control remains the research winner.
3. Two coherent failures of one architecture family require a new falsifiable
   hypothesis, not a wider hyperparameter sweep.
4. Missing geometry coverage, support, source axes or calibrated confidence
   returns `FallbackOutOfDomain`; labels cannot fill a missing measurement.
5. Source search is bounded per source family. If S2 remains impossible, close
   as `ADMISSION_BLOCKED` and preserve any L/R result as report-only.
6. No object-41 force-gate reduction, object-92 role/contact reduction, local
   recording, prompt-to-waveform shortcut, hidden human selection or runtime
   model inference.
7. No checkpoint, seed, threshold, contact or object selection after the
   corresponding development/protected boundary is opened.

## Completion states

V16 has two honest terminal outcomes:

- **admitted research vertical:** L1 and R1 pass, S2/V1 freeze, A0 passes, and
  C0/D0 produce a deterministic demo with complete fallback;
- **closed research result:** the model loses a tournament, the validator
  rejects/OODs the shadow, or protected data remains insufficient after bounded
  source work. No runtime/content authority changes.

A pleasing development WAV alone is never a completion criterion.

## Immediate commit sequence

1. `R0` — `COMPLETE`: adopt V16 and record the unchanged product/data boundary.
2. `L0a` — `COMPLETE`: [freeze](../development/physical-sound-v16-l0a-known-truth-neural-oracle-protocol-2026-09-01.md)
   the multi-object/contact known-truth distribution, model, controls, metrics,
   mutations, compute ceiling and exact stop rules.
3. `L0b` — implement and run the truth tournament twice; authorize or reject R1.
4. `S0a` — classify historical RealImpact exposure by known manifest semantics,
   fail-closing unknown schemas without source-body or signal access.
5. `S1a` — implement bounded per-object member/axis adapters and source-family
   closure reports; freeze S2 only if the unchanged Metal shape becomes real.
6. `V0a` — build validator contracts and mutation fixtures without frozen
   real-data thresholds.
7. `R1a/R1b` — preregister and run the disclosed Metal representation/contact
   tournament after L1 and generator-role readiness.
8. `V1a/V1b` — preregister protected calibration/holdout, freeze the independent
   validator and publish its operating evidence.
9. `A0/C0/D0` — open the shadow once, cook only on pass and wire only the bounded
   demo vertical.
10. `G0` — repeat the complete unchanged pipeline for Wood, then Glass.
