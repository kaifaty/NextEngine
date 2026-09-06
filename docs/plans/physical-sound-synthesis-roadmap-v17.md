# Roadmap V17: scale-separated modes and intrinsic surface operators

| Field | Value |
| --- | --- |
| Rebaseline date | `2026-09-01` |
| Status | `SUPERSEDED_BY_V18 / G0_REPEAT_EXACT_REJECT / O0_F0_I0_NOT_RUN / ADMISSION_BLOCKED / RUNTIME_NOT_AUTHORIZED` |
| Replaces | [Roadmap V16](physical-sound-synthesis-roadmap-v16.md) as the active execution plan |
| Evidence basis | [V17 successor research](../development/physical-sound-v17-factorized-operator-research-2026-09-01.md), [V16 L0b rejection](../development/physical-sound-v16-l0b-known-truth-neural-oracle-result-2026-09-01.md) and [V15 S0c source insufficiency](../development/physical-sound-v15-s0c-source-sufficiency-role-freeze-result-2026-09-01.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Mandatory fallback | Existing authored/recorded clip for every reject, OOD, unsupported material, missing source or tooling failure |

V17 closed at its first frozen dependency. [G0](../development/physical-sound-v17-g0-scale-separated-global-oracle-result-2026-09-01.md)
passed absolute and scale-transfer quality but failed both required complexity
margins against degree-two ridge. Per P0a, O0/F0/I0 were not run. Execution
continues only through [Roadmap V18](physical-sound-synthesis-roadmap-v18.md),
which preserves scale separation as a deterministic baseline and reserves
learning for intrinsic surface structure.

## Goal

Prove or reject three reusable capabilities independently before another
integrated sound model is allowed:

1. recover object-global modal frequencies/damping after deterministic
   dimensional scale separation;
2. recover a continuous signed modal-gain field from sparse contacts using the
   intrinsic surface graph without pooling away context identity;
3. reject missing contact support using model-independent intrinsic coverage.

Only after all three pass may an integrated offline tool bake deterministic
clips for a disclosed-real tournament. V17 does not authorize runtime neural
inference, a public content schema or product promotion.

## Successor shape

```text
ScaleSeparatedModalFieldV0 (research-only)

object/acoustic metadata
  -> deterministic dimensional scales
  -> learned ordered dimensionless modal ratios + damping multipliers

mesh + sparse observed contact gains
  -> intrinsic masked diffusion/message-passing operator
  -> signed per-vertex modal gains + edge continuity

context vertices + topology
  -> multi-source geodesic coverage certificate
  -> InDomain | FallbackOutOfDomain

all pass -> deterministic modal renderer -> offline dry clips
otherwise -> authored clip fallback
```

The material/acoustic parameters belong to the synthesis research profile and
never become physics-material authority. Residual energy, radiation, rooms,
arbitrary force, rolling, scraping and runtime inference remain outside this
claim.

## Evidence lanes and promotion

| Lane | Question | Output | Promotion rule |
| --- | --- | --- | --- |
| G — global | Does dimensionless factorization recover poles/damping on new objects/scales? | `GlobalCapability` | Opens F/I integration only; no real credit. |
| O — coverage | Does intrinsic geometry detect unsupported contacts/topology? | `CoverageCapability` | Becomes a fixed input to F/I; no learned threshold from test. |
| F — field | Does masked intrinsic propagation recover continuous signed gains? | `FieldCapability` | Opens I only after G/O pass. |
| I — integration | Do frozen G+O+F recover waveform endpoints together? | `IntegratedCapability` | May reopen disclosed-real R1 if source-ready. |
| S — sources | Are generator and protected groups actually complete/fresh? | source/role certificates | Independent prerequisite; no synthetic result fills a missing axis. |
| R/V/A | Does it improve disclosed real audio and pass independent protected validation? | candidate, validator, admission certificates | Same separation and one-shot shadow policy as V16. |
| C/D/P | Can admitted outputs cook, demo and later justify production? | deterministic atlas/demo, then ADR | Blocked until admission; authored fallback always complete. |

Every lane exchanges immutable hashes and bounded schemas only. Training state
never crosses into validator/protected roles.

## P0a protocol requirements

P0a freezes all values before implementation:

- a new object/mesh revision set disjoint from every V16 row;
- a cheap object-level G corpus large enough to cover material, topology,
  support, size, aspect and wall interactions without treating nine labels as
  a population;
- a separate F corpus with train/development/test objects and remeshed twins;
- topology-valid O mutations defined by intrinsic farthest regions, component
  removal, holes and coverage thinning;
- train-only normalization, object/group-disjoint roles and one final test
  opening after candidate/control artifacts freeze;
- one small candidate per lane, explicit controls, three fixed seeds only where
  learning exists and no hyperparameter selection from test;
- exact CPU environment, compute ceiling, serialization and two complete
  byte-identical executions;
- zero real/source/protected/network access and external-only generated
  artifacts.

The exact sample counts and thresholds belong in P0a, justified by successful
controls and power/coverage calculations. This roadmap does not invent them
before that calculation.

## Frozen candidate families

### G0 — dimensionless global head

Deterministic preprocessing computes declared modal frequency and damping
scales from acoustic profile, metric geometry and support. The network predicts
only positive ordered dimensionless ratios/multipliers. Controls are:

- the V16 raw-dimensional MLP reconstructed without reading V16 test outcomes;
- material/support mean;
- nearest object in dimensionless feature space;
- a monotone low-order regression/GAM on dimensionless features.

Hidden topology coefficients and truth formulas remain unavailable to every
candidate/control. G0 must pass frequency/damping gates and beat the raw MLP on
new scale combinations; otherwise scale separation is rejected.

### O0 — intrinsic coverage certificate

Compute the complete multi-source graph-geodesic/heat-distance field from
context vertices. The certificate uses preregistered normalized maximum,
quantiles, uncovered area, connected components and boundary/topology facts.
It is compared with V16 Euclidean coverage and ensemble disagreement, but it is
not trained jointly with F0. Valid coverage and each mutation class are grouped
by object/topology.

### F0 — masked intrinsic field operator

Context mask and normalized signed gains are vertex channels. A small learned
diffusion/message-passing stack propagates them over mesh edges; object/global
latent values condition the per-vertex decoder. Loss combines query gain MSE
with an edge-gradient/continuity term fixed in P0a. Required controls are
nearest contact, Euclidean/geodesic RBF, local linear, V16 pooled-context MLP
and a permutation-invariant attention ablation without intrinsic propagation.

Full GINO/NORM, voxel CNN, direct waveform decoder and larger cross-attention
are not first candidates. They require a new hypothesis if the bounded masked
operator fails.

### I0 — integrated tournament

Freeze the passing G0, O0 and F0 artifacts, then evaluate their arithmetic
ensemble through the existing deterministic modal renderer. I0 retains V16
finite/order, waveform, spectrum, envelope, modal-peak, continuity, OOD,
mutation and exact-repeat endpoint families. P0a may correct definitions before
implementation, but no I0 threshold may be selected from its test.

## Execution graph

```text
R0 research complete
        |
        v
P0a disjoint protocol freeze
   |          |           |
   v          v           v
G0 global    O0 coverage  F0 field
   +----------+-----------+
              |
              v
        I0 integrated truth
              |
     pass + 5 generator groups
              v
       R1 disclosed real
              |
       S2 protected freeze
              v
      V1 validator -> A0 shadow once
              |
        pass -> C0 -> D0
        reject/OOD -> authored fallback
```

S1 source-adapter work may proceed in parallel. It cannot shorten P0–I0 or
open protected signal.

## Milestones

| ID | Package | State | Observable exit criterion |
| --- | --- | --- | --- |
| R0 | V17 research/rebaseline | `COMPLETE` | Primary sources and V16 evidence yield three falsifiable successor hypotheses, fallbacks and non-claims. |
| P0a | Disjoint truth protocol | `COMPLETE / FROZEN_BEFORE_IMPLEMENTATION` | [P0a](../development/physical-sound-v17-p0a-disjoint-factorized-truth-protocol-2026-09-01.md) freezes 144 G rows, 60 F/I physical groups plus remeshed twins, topology-valid O mutations, candidates/controls, successful controls, sample rationale, gates, seeds, compute and stop rules with zero external evidence. |
| G0 | Scale-separated global oracle | `COMPLETE / REPEAT_EXACT / REJECT` | `10/12` gates pass; absolute and raw-MLP transfer gates pass, but neural/ridge ratios `0.9497x/0.9239x` miss the required `<=0.90x`. |
| O0 | Intrinsic coverage oracle | `NOT_RUN / V17_CLOSED` | P0a stop rule prevents downstream execution after G0 rejection. |
| F0 | Masked intrinsic field oracle | `NOT_RUN / V17_CLOSED` | P0a stop rule prevents downstream execution after G0 rejection. |
| I0 | Integrated truth tournament | `NOT_RUN / V17_CLOSED` | No passing V17 G/O/F dependency set exists. |
| S1 | Published-source growth | `PARALLEL / SOURCE_INSUFFICIENT` | Each bounded source gains an exact identity/member/axis/freshness certificate or machine-readable closure reason; no user recording. |
| S2 | Metal role freeze | `BLOCKED_BY_S1` | Five generator-ready plus three fresh evaluation-complete Metal groups satisfy unchanged `4/1/1/1/1`. |
| R1 | Disclosed-real tournament | `BLOCKED_BY_I0_AND_5_GENERATOR_GROUPS` | Frozen successor beats compatible real controls without protected access; otherwise close it. |
| V0/V1 | Validator scaffold/release | `SCAFFOLD_AFTER_I0 / RELEASE_BLOCKED_BY_R1_S2` | Synthetic mutation machinery first; real calibration/holdout thresholds freeze independently after generator nomination. |
| A0 | Metal shadow admission | `BLOCKED_BY_V1` | Open one untouched protected shadow once; emit immutable Pass/Reject/FallbackOOD and make no revision afterward. |
| C0/D0 | Cooker/demo | `BLOCKED_BY_A0_PASS` | Byte-identical clips and fallback map feed one presentation-only demo prop through existing audio path and relevant product checks. |
| G1 | Wood then Glass | `AFTER_METAL_A0` | Each material repeats unchanged real-role/validator/admission policy; missing domains remain fallback. |
| P1 | Production promotion | `POST_RESEARCH / ADR_REQUIRED` | A concrete consumer, adequate protected-population evidence and separate Accepted ADR justify contracts and ProductChecks. |

## Stop rules

1. Never reuse V16 test rows, predictions or metrics for model/threshold
   selection; they are regression evidence only after V17 freezes.
2. G0, O0 and F0 fail independently. No good integrated WAV may hide a failed
   certificate, and no later lane may compensate by weakening its gate.
3. One bounded primary candidate per lane. A failed candidate closes that
   hypothesis; larger GINO/NORM/cross-attention/waveform models need a new
   research record and unopened test.
4. O0 mutation regions must be defined intrinsically after context placement;
   UV seams and ambient shortcuts cannot manufacture false OOD labels.
5. Synthetic capability, disclosed-real quality and protected admission remain
   separate certificates. No source label fills geometry, excitation,
   listener, support or freshness evidence.
6. No local microphone/hammer capture, manual per-sound queue, prompt-to-wave,
   raw PhysX callback mixing, runtime training/inference or fallback removal.
7. Datasets, arrays, weights, reports, audio and caches stay external; unknown
   redistribution terms exclude artifacts from product distribution.

## Immediate commit sequence

1. `R0` — `COMPLETE`: record V16 rejection, primary-source research and V17.
2. `P0a` — `COMPLETE`: freeze disjoint corpora, successful controls, exact
   gates and compute before successor implementation.
3. `G0a/G0b` — `COMPLETE / REPEAT_EXACT_REJECT`; preserve reports and close
   the candidate family without test-driven tuning.
4. `O0a/O0b`, `F0a/F0b`, `I0a/I0b` — `NOT_RUN`; superseded by the V18
   dependency graph.
7. `S1a…` — in parallel, add bounded published-source adapters/closure reports.
8. `R1/S2/V1/A0` — proceed only when both capability and source prerequisites
   pass; otherwise preserve external reports and fallback.
9. `C0/D0` — cook/demo only after one-shot admission pass.
10. `P1` — consider production only through a separate Accepted ADR and mapped
    ProductChecks.
