# Roadmap V15: progressive neural modal-field authoring

| Field | Value |
| --- | --- |
| Rebaseline date | `2026-09-01` |
| Status | `ACTIVE_R&D / S0C_SOURCE_INSUFFICIENT / S0D_SOURCE_GROWTH_NEXT / METAL_FIRST / RUNTIME_NOT_AUTHORIZED` |
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
| Metal | [S0c](../development/physical-sound-v15-s0c-source-sufficiency-role-freeze-result-2026-09-01.md) finds `12` fresh ObjectFolder routes (`4` compact, `8` preflight), `9` YCB exact parents and no role-eligible row under the unchanged axes | `SOURCE_INSUFFICIENT / FIRST_GROWTH_DOMAIN` | S0d proves five generator-ready plus three fresh evaluation-complete groups from published sources, then reruns role freeze. |
| Wood | S0c retains `0` fresh ObjectFolder routes and `3` adapter-referenced YCB parents, only `2` with non-ambiguous geometry route | `PENDING_SOURCE_GROWTH` | A source yields the unchanged five generator plus three protected groups. |
| Glass | S0c retains `2` preflight-only ObjectFolder/RealImpact groups and `0` exact YCB object parents | `FALLBACK_ONLY / SOURCE_GROWTH` | A new published source yields the unchanged five generator plus three protected groups. |

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
S0 source identity/scope -> S0d published-source growth -> S0c role rerun
  -> S1 known-truth oracle
  -> S2 real modal representation
  -> S3 exact-object contact field
  -> S4 cross-object authoring prior
                       \
S0d/S0c -> V0 validator data -> V1 frozen validator -> A0 shadow admission
                                                   -> C0 cooker
                                                   -> D0 demo vertical
                                                   -> G0 Wood/Glass growth
```

The validator lane starts only after S0d supplies a complete role freeze and
cannot admit anything until S4 freezes a generator release.

## Milestones

| ID | Package | State | Observable exit criterion |
| --- | --- | --- | --- |
| S0a | Revision-aware identity and exposure | `COMPLETE / REPEAT_EXACT / ZERO_SIGNAL` | [S0a](../development/physical-sound-v15-s0a-revision-aware-identity-exposure-result-2026-09-01.md) resolves 130 groups and 30 drift conflicts; exact-name exposure corrects the fresh counts to Glass/Wood/Metal `2/0/12`. |
| S0b | YCB capability adapter | `COMPLETE / REPEAT_EXACT / ZERO_SIGNAL` | [S0b](../development/physical-sound-v15-s0b-ycb-capability-cost-result-2026-09-01.md) inventories 77 objects and 926 OSF entries; exact target parents are Metal/Wood/Glass `9/3/0`, while usable/evaluation credit remains zero. |
| S0c | First-domain role freeze | `COMPLETE / REPEAT_EXACT / SOURCE_INSUFFICIENT` | [S0c](../development/physical-sound-v15-s0c-source-sufficiency-role-freeze-result-2026-09-01.md) joins 130 groups and 18 target YCB rows with zero signal access; Metal/Wood/Glass all have zero eligible assignments under the unchanged policy. |
| S0d | Published-source growth | `NEXT / BLOCKS_S1` | Resolve exposure class and publisher-side per-object member/axis certificates or add a published T2/T3 source until Metal has five generator-ready plus three fresh evaluation-complete groups; otherwise close each investigated source family without signal access. |
| S1 | Known-truth neural oracle | `BLOCKED_BY_S0D_AND_ROLE_FREEZE` | On engine-owned truth scenes, a small model recovers stable poles and held surface gains and rejects instability, contact shuffle, wrong scale/material and coverage-collapse mutations. |
| S2 | Real modal representation | `BLOCKED_BY_S1` | On opened Metal train/development data, structured analysis/synthesis beats classical Q30/DCT and identity-budget controls on frozen spectrum, onset, envelope, decay and mode metrics. |
| S3 | Exact-object few-shot field | `BLOCKED_BY_S2` | One shared revision, using about 20% signal-blind contacts, beats nearest-contact, Euclidean/geodesic RBF, local-linear and geometry-agnostic controls on untouched contacts; otherwise representation closes. |
| S4 | Cross-object prior | `BLOCKED_BY_S3` | Object-disjoint mesh/material prior beats material-mean and nearest-object controls, is contact-continuous and returns calibrated OOD for unsupported geometry/support. |
| V0 | Validator corpus and mutations | `BLOCKED_BY_S0D_ROLE_FREEZE` | Independent real groups, wrong-material swaps, onset/spectral/decay artifacts and physics violations are hash-frozen without generator-protected leakage. |
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
2. `S0b` — `COMPLETE`: [repeat-exact YCB capability/cost inventory](../development/physical-sound-v15-s0b-ycb-capability-cost-result-2026-09-01.md)
   binds exact vertical parents with zero audio body and zero signal access.
3. `S0c` — `COMPLETE`: [repeat-exact source certificate](../development/physical-sound-v15-s0c-source-sufficiency-role-freeze-result-2026-09-01.md)
   closes the current sources with zero assignments and zero signal access.
4. `S0d` — resolve generator-only historical exposure and publisher-side
   per-object axis/member evidence, then search one additional published T2/T3
   source; rerun role freeze only after the unchanged Metal shape is reachable.
5. `S1a` — preregister truth-scene distribution, structured model, controls,
   mutations, metrics, compute ceiling and stop rule.
6. `S1b` — execute the known-truth oracle twice and authorize or reject S2.
7. `V0a` — freeze independent validator corpus/mutations after role freeze while S1/S2
   proceed, without exposing generator-protected roles.
8. `S2a/S2b` — preregister and run the real modal-representation tournament.
9. `S3a/S3b` — preregister and run exact-object few-shot contact prediction.
10. `S4a/S4b` — preregister and run object-disjoint authoring plus OOD.
11. `V1/A0/C0/D0` — freeze validator, open shadow once, cook on pass and wire
    only the bounded demo vertical.
