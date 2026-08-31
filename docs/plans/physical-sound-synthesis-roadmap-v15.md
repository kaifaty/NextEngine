# Roadmap V15: progressive neural modal-field authoring

| Field | Value |
| --- | --- |
| Rebaseline date | `2026-09-01` |
| Status | `ACTIVE_R&D / S0A_COMPLETE / S0B_YCB_CAPABILITY_NEXT / METAL_FIRST / RUNTIME_NOT_AUTHORIZED` |
| Replaces | [Roadmap V14](physical-sound-synthesis-roadmap-v14.md) as the active execution plan |
| Evidence basis | [V15 progressive-admission rebaseline](../development/physical-sound-v15-progressive-material-admission-rebaseline-2026-09-01.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Mandatory fallback | Existing authored/recorded clip for every reject, OOD, unsupported material or tooling failure |

## Outcome

Build an offline authoring tool that learns how compact object modes are
excited across a prop's surface, then cooks a deterministic atlas of plausible
dry impact clips for the existing engine audio path.

V15 deliberately separates three questions:

1. **Representation:** can a bounded neural model recover stable modal poles
   and contact-dependent gains?
2. **Generalization:** can mesh, material, scale and support predict useful
   impacts for a previously unseen prop?
3. **Admission:** can an independent automatic validator accept useful output
   at bounded false-pass risk and select fallback everywhere else?

The first end-to-end protected vertical is Metal because published metadata
currently provides enough candidate groups. Wood and Glass join only after
they independently satisfy the same frozen data and validation gates. No
material minimum is reduced to make the roadmap advance.

## Admitted claim

```text
CanonicalImpactAtlasV1
  mesh + material family + scale + support class
       + surface contact + canonical energy bin
    -> stable object-global frequencies/damping
       + bounded contact-conditioned modal gains
       + uncertainty/OOD
    -> deterministic 48 kHz cooked clips or authored fallback
```

This is a plausibility and contact-consistency claim within an admitted data
domain. It is not recovery of true material constants, raw microphone transfer,
arbitrary impact-force response, room acoustics, rolling, scraping or fracture.

## Non-negotiable boundaries

- Datasets, PCM, arrays, weights, checkpoints, reports and generated WAVs stay
  in the external experiment store and never enter Git.
- Real evidence comes from published internet sources; the user records or
  strikes nothing locally.
- Source identity and protected roles freeze before signal decode.
- Training, validator calibration, method holdout and admission shadow are
  object/recording-parent and publisher-revision disjoint.
- Generator training cannot see validator calibration parameters, protected
  scores, method holdout or admission shadow.
- Runtime receives only cooked audio and bounded metadata. It never loads a
  neural model, dataset or research environment.
- Existing authored clips remain authoritative until a later product package
  and Accepted ADR promote a concrete consumer.

## Data contract and progressive admission

Every admitted material must independently supply:

```text
4 train + 1 generator development + 1 validator calibration
        + 1 method holdout + 1 admission shadow
```

All eight are distinct physical object/recording-parent groups. Generator
train/development may use `training_usable` T0/T1/T2/T3 data, while validator
calibration, method holdout and admission shadow must be unexposed,
`evaluation_complete` T2/T3 groups. V15 additionally keeps N1b's eight-real-
candidate inventory gate before a material is frozen, so protected allocation
does not leave the real representation stages without train/development data.
Synthetic teachers can enlarge training but cannot provide protected credit.

| Material | Current evidence | V15 state | Entry condition |
| --- | --- | --- | --- |
| Metal | S0a proves `12` unexposed physical-group metadata candidates | `FIRST_DOMAIN_CANDIDATE` | S0c still proves at least eight structurally eligible groups and all required axes. |
| Wood | S0a proves `0` unexposed current-source groups; YCB may add independent groups | `PENDING_SOURCE_CERTIFICATE` | YCB/revision adapter yields eight fresh evaluation-complete groups without changing gates. |
| Glass | S0a proves `2` unexposed groups; YCB has only three primary-glass objects | `FALLBACK_ONLY / SOURCE_GROWTH` | A new or newly resolved published source yields the unchanged eight-group shape. |

An admitted material uses one immutable role freeze through representation,
validator and shadow. Later materials run the same versioned pipeline; their
arrival cannot reopen thresholds or results for earlier materials.

## Technical candidate

The first model is intentionally small and structured:

1. deterministic mesh sampling plus local geometry, scale, material family and
   support features;
2. one object encoder for global modal frequencies and positive damping;
3. one surface field for bounded complex or signed modal gains at contact;
4. calibrated epistemic/OOD heads for unsupported geometry and contact
   coverage;
5. the existing differentiable modal renderer and deterministic clip cooker.

Mesh-only is the baseline. DINO/3DGS features are a later ablation, not an
initial dependency. A learned residual remains disabled until the modal branch
passes alone; if enabled later, it gets an independent energy budget and
ablation gate. Direct waveform generation is report-only and cannot satisfy
admission.

## Execution graph

```text
S0 source identity/scope
  -> S1 known-truth oracle
  -> S2 real modal representation
  -> S3 exact-object contact field
  -> S4 cross-object authoring prior
                       \
S0 -> V0 validator data -> V1 frozen validator -> A0 shadow admission
                                                   -> C0 cooker
                                                   -> D0 demo vertical
                                                   -> G0 Wood/Glass growth
```

The validator lane can be built after S0 but cannot admit anything until S4
freezes a generator release.

## Milestones

| ID | Package | State | Observable exit criterion |
| --- | --- | --- | --- |
| S0a | Revision-aware identity and exposure | `COMPLETE / REPEAT_EXACT / ZERO_SIGNAL` | [S0a](../development/physical-sound-v15-s0a-revision-aware-identity-exposure-result-2026-09-01.md) resolves 130 groups and 30 drift conflicts; exact-name exposure corrects the fresh counts to Glass/Wood/Metal `2/0/12`. |
| S0b | YCB capability adapter | `NEXT` | Repeat-exact metadata-only inventory binds YCB object/mesh/material/recording parents and available contact/support/listener axes without downloading or decoding audio. |
| S0c | First-domain role freeze | `BLOCKED_BY_S0A_S0B` | Metal either fills the unchanged `4/1/1/1/1` shape with exact hashes or V15 closes as `SOURCE_INSUFFICIENT`; Wood/Glass receive explicit pending/fallback certificates. |
| S1 | Known-truth neural oracle | `BLOCKED_BY_S0C` | On engine-owned truth scenes, a small model recovers stable poles and held surface gains and rejects instability, contact shuffle, wrong scale/material and coverage-collapse mutations. |
| S2 | Real modal representation | `BLOCKED_BY_S1` | On opened Metal train/development data, structured analysis/synthesis beats classical Q30/DCT and identity-budget controls on frozen spectrum, onset, envelope, decay and mode metrics. |
| S3 | Exact-object few-shot field | `BLOCKED_BY_S2` | One shared revision, using about 20% signal-blind contacts, beats nearest-contact, Euclidean/geodesic RBF, local-linear and geometry-agnostic controls on untouched contacts; otherwise representation closes. |
| S4 | Cross-object prior | `BLOCKED_BY_S3` | Object-disjoint mesh/material prior beats material-mean and nearest-object controls, is contact-continuous and returns calibrated OOD for unsupported geometry/support. |
| V0 | Validator corpus and mutations | `CAN_START_AFTER_S0C` | Independent real groups, wrong-material swaps, onset/spectral/decay artifacts and physics violations are hash-frozen without generator-protected leakage. |
| V1 | Independent Validator V2 | `BLOCKED_BY_V0_S4` | Frozen hard, physics, acoustic, semantic and selective-risk specialists publish parent-grouped false-pass upper bounds together with retained coverage. |
| A0 | Metal admission shadow | `BLOCKED_BY_S4_V1` | One untouched Metal object is opened exactly once and receives immutable `Pass`, `Reject` or `FallbackOutOfDomain`; no threshold/model revision follows. |
| C0 | Deterministic atlas cooker | `BLOCKED_BY_A0_PASS` | Canonical surface regions and energy bins cook twice to byte-identical 48 kHz clips, hashes, bounded metadata and a complete fallback map. |
| D0 | Demo-scene vertical | `BLOCKED_BY_C0` | One non-authoritative demo prop uses the existing contact-to-clip path; fallback, replay, content-package, play and conditional performance checks pass. |
| G0 | Progressive domain growth | `AFTER_A0` | Wood, then Glass, repeat S0c–A0 with the same role shape and validator policy; unavailable domains remain `FallbackOnly`. |
| P0 | Production promotion | `POST_RESEARCH / ADR_REQUIRED` | A visible gameplay prop justifies public content/projection contracts and a new Accepted ADR; no roadmap success alone promotes SPEC-45. |

## Automatic validator

The validator is a release artifact independent from the generator:

- **hard/provenance:** hashes, schema, split/parent isolation, finite bounds and
  byte-repeat checks;
- **physics:** positive damping, stable modes, bounded energy, energy-bin
  monotonicity and surface continuity with declared discontinuities;
- **acoustic:** multi-resolution spectrum, onset, envelope, decay, modal
  persistence and artifact statistics against held real distributions;
- **semantic:** frozen material/shape classifiers and controlled material swaps;
- **selective risk:** OOD calibration, parent-grouped bootstrap confidence and
  a published false-pass upper bound at the retained coverage.

The validator may reject a convincing sound and may accept a plausible sound
that is not the object's unique physical truth. V15 bounds this limitation by
claim, protected real groups, mutations and fallback; it does not disguise the
limitation as solved. Human listening is optional release auditing only.

## Tournament and stop rules

1. Freeze data roles, features, metrics, thresholds, compute ceiling and stop
   rule before the corresponding protected signal is opened.
2. A neural candidate must beat compatible KNN/geodesic/classical controls.
   If it does not, the simpler candidate wins or the stage closes.
3. S1 failure stops all real model training. S2 failure stops spatial learning.
   S3 failure stops unseen-object generalization.
4. No per-object model, threshold, checkpoint, contact or seed selection after
   development roles are opened.
5. Two coherent failures of one architecture family require a new falsifiable
   hypothesis, not a larger sweep or an unconstrained residual.
6. Missing local geometry coverage, source axes or calibrated confidence yields
   `FallbackOutOfDomain`; it never triggers silent extrapolation.
7. No object-41 force-gate reduction, object-92 role reduction, local recording,
   hidden human approval, prompt-to-waveform shortcut or runtime inference.
8. No arbitrary-force claim without a separate published paired-force source
   and a fresh measured-transfer certificate.

## Completion criterion

V15 completes when one immutable Metal generator release and one independent
validator release process one untouched Metal shadow exactly once, publish a
tri-state decision, and either:

- cook byte-identical clips and a complete fallback map for a pass; or
- close the candidate as a reproducible reject/fallback without changing the
  production authored-clip path.

Wood and Glass are explicit domain-growth packages, not hidden prerequisites
for learning whether the approach works. A Metal reject is still a complete
scientific result; it stops the family before more material data is consumed.

## Immediate commit sequence

1. `S0a` — `COMPLETE`: [repeat-exact identity/exposure](../development/physical-sound-v15-s0a-revision-aware-identity-exposure-result-2026-09-01.md)
   prevents current/historical numeric-ID collisions and preserves 12 fresh
   Metal metadata groups.
2. `S0b` — implement repeat-exact YCB metadata/cost adapter with zero audio
   body and zero signal access.
3. `S0c` — publish immutable Metal role descriptor or close source sufficiency;
   publish Wood/Glass pending/fallback certificates in the same result.
4. `S1a` — preregister truth-scene distribution, structured model, controls,
   mutations, metrics, compute ceiling and stop rule.
5. `S1b` — execute the known-truth oracle twice and authorize or reject S2.
6. `V0a` — freeze independent validator corpus/mutations after S0c while S1/S2
   proceed, without exposing generator-protected roles.
7. `S2a/S2b` — preregister and run the real modal-representation tournament.
8. `S3a/S3b` — preregister and run exact-object few-shot contact prediction.
9. `S4a/S4b` — preregister and run object-disjoint authoring plus OOD.
10. `V1/A0/C0/D0` — freeze validator, open shadow once, cook on pass and wire
    only the bounded demo vertical.
