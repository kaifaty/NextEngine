# Roadmap V20: phase-consistent physical-sound validation

| Field | Value |
| --- | --- |
| Rebaseline date | `2026-09-01` |
| Status | `CLOSED / I1_REPEAT_EXACT_REJECT / ACTUAL_ACOUSTIC_PASS / REPLACED_BY_V21` |
| Replaced by | [Roadmap V21](physical-sound-synthesis-roadmap-v21.md) |
| Replaces | [Roadmap V19](physical-sound-synthesis-roadmap-v19.md), closed before I0 by its development-control rejection |
| Evidence basis | [V19 I0 development control](../development/physical-sound-v19-i0-development-control-result-2026-09-01.md), [V19 F0](../development/physical-sound-v19-f0-residual-harmonic-field-result-2026-09-01.md), [V19 C0](../development/physical-sound-v19-c0-composite-coverage-result-2026-09-01.md), [V18 B0](../development/physical-sound-v18-b0-deterministic-global-baseline-result-2026-09-01.md) and [V15 source insufficiency](../development/physical-sound-v15-s0c-source-sufficiency-role-freeze-result-2026-09-01.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Mandatory fallback | Existing authored/recorded clip for every reject, OOD, unsupported material, absent source or tooling failure |

## Outcome sought

Build an automated, evidence-driven loop that can learn bounded modal impact
sound formulas from internet sources, reject unsupported cases without asking
the user to validate every sound, and bake accepted results into deterministic
clips for the existing presentation path.

V20 first repairs the synthetic integration measuring instrument. It then
continues source acquisition and independent real-audio validation without
changing the product boundary:

```text
internet source registry + immutable role split
  -> source/coverage certificate
  -> candidate formula trained on disclosed generator roles
  -> independent hard + physical + acoustic validator
  -> one untouched protected shadow decision
  -> deterministic offline clip cooker + fallback map

mesh/object/contact
  -> frozen coverage certificate
  -> global modal scaffold + learned contact field
  -> phase-consistent integration evidence

any missing evidence / OOD / reject / fault
  -> authored clip fallback
```

No V20 milestone authorizes runtime inference or training, public contracts,
fallback removal, arbitrary-force transfer, room/listener radiation or a claim
that synthetic truth is real material acoustics.

## Why V20 exists

V18 B0, V19 C0 and V19 F0 each pass their isolated fresh capability gates
twice exactly. Before opening V19 I0, a composition control on already-opened
development objects found:

- B0 frequency error `6.81/14.05 cents` median/p95 and F0's complete 17-gate
  field certificate both pass;
- multiresolution spectrum error `2.325 dB` passes;
- raw early-waveform NRMSE is `0.445` and mixed Hilbert-envelope p95 is
  `0.478`, so both frozen integration gates reject;
- global-only counterfactual error is already `0.414/0.467`, proving that
  allowed pitch error, accumulated phase and modal interference dominate;
- field-only waveform passes at `0.181`, while envelope p95 `0.237` proves the
  old envelope threshold is also tighter than the accepted field contract.

V19 therefore stopped without generating any integration values. V20 does not
loosen a failed test. It preregisters a metric family whose acceptable and
harmful perturbations must first be separable on development evidence.

## Immutable boundaries

- Reuse exact passing B0/C0/F0 artifacts by complete-tree and member hashes.
  They cannot be refit, reselected or reinterpreted in M0/P0c/I1.
- Reuse only opened V19 development identities `1401…1412` for metric design.
- Retire V19 integration band `1601…1612`; its metadata was frozen but its
  meshes, truth, predictions, metrics and waveforms remain unopened.
- Reserve V20 I1 band `1701…1712` plus same-object remesh twins. P0c must freeze
  its exact metadata root before any value is generated.
- If I1 attributes a genuine component deficiency, a successor protocol may
  reserve `1801…1824` train, `1901…1912` development, `2001…2012` test and
  `2101…2112` reintegration. These bands remain metadata- and value-sealed
  until that decision.
- The user records nothing. Real evidence comes from disclosed internet
  sources with source identity, revision, members, provenance and
  redistribution disposition.
- All datasets, audio, arrays, weights, reports and caches stay under the
  external experiment root, never in Git.

## Metric contract to earn at M0

The validator keeps four evidence layers. One scalar may not substitute for
another:

1. **Structural/hard:** dependency hashes, finite/order/range checks, complete
   context, coverage reason, fallback completeness and corruption rejection.
2. **Physical parameters:** per-mode frequency in cents, damping relative
   error, signed gain NRMSE, spatial edge-gradient error and modal-peak
   correspondence.
3. **Phase-tolerant acoustics:** multiresolution STFT spectral convergence plus
   log-magnitude distance, and windowed/banded decay-energy error over the
   impact transient.
4. **Diagnostics:** raw sample NRMSE and mixed-signal Hilbert envelope. They may
   block only when frequencies, onset and phase are identical or when a future
   protocol first freezes an independently justified alignment rule.

M0a used deterministic counterfactual ladders on development only:

- frequency offsets from identity through the accepted cents boundary and
  beyond it;
- damping offsets through the accepted relative boundary and beyond it;
- signed modal-gain and edge-gradient perturbations through their accepted
  boundaries and beyond them;
- mode removal, mode duplication/permutation, polarity/sign corruption,
  onset displacement, impulse/click injection and fallback omission;
- isolated B0-only, F0-only and combined compositions.

For each blocking metric/family, all declared acceptable controls must lie
strictly below all corresponding harmful controls with a preregistered margin.
If the intervals overlap, that metric cannot receive a threshold and the
revision rejects. [M0a rejected exactly](../development/physical-sound-v20-m0-phase-consistent-metric-result-2026-09-01.md): raw MRLM missed the frequency margin narrowly and absolute DE mixed accepted
modal gains with damping. [M0b](../development/physical-sound-v20-m0b-confound-resistant-metric-protocol-2026-09-01.md)
therefore preregisters mean-centered log spectral shape and normalized decay
slope without changing the controls or margin. Thresholds, STFT windows,
normalization, aggregation, reason precedence and mutation severity freeze in
P0c before I1.

Primary literature motivates the candidate family but supplies no transferable
threshold: [DDSP](https://arxiv.org/abs/2001.04643),
[Parallel WaveGAN](https://arxiv.org/abs/1910.11480),
[DiffSound](https://arxiv.org/abs/2409.13486),
[NeuralSound](https://arxiv.org/abs/2108.07425) and
[Differentiable Modal Resonators](https://arxiv.org/abs/2210.15306).

## Evidence lanes

| Lane | Question | Output | Promotion rule |
| --- | --- | --- | --- |
| M — metric validity | Does each automatic metric distinguish the physical defect it claims to gate? | `PhaseConsistentMetricCertificateV0` | Must pass successful controls and causal attribution before P0c. |
| B/C/F — frozen capability | Are global modes, coverage and contact field still exact dependencies? | pinned B0/C0/F0 identities | Reuse only; no training or fresh claim in I1. |
| I — integration | Do frozen components reproduce complete phase-consistent endpoints? | `IntegratedCapabilityV1` | Repeat-exact pass on fresh I1 opens only disclosed-real work whose source prerequisites are also ready. |
| S — internet sources | Are enough complete, provenance-carrying physical groups role-eligible? | source and immutable role certificates | Independent prerequisite; no local recording and no alias leakage. |
| R — disclosed-real candidate | Does one frozen formula improve generator-visible real recordings? | candidate certificate | Candidate never sees validator/holdout/shadow values. |
| V — independent validator | Do hard, physical, acoustic and OOD checks calibrate independently? | frozen validator artifact and policy | Freeze before protected shadow; generator cannot train against it. |
| A — protected admission | Does one untouched real shadow pass exactly once? | `Pass`, `Reject` or `FallbackOOD` | Only `Pass` opens cooker/demo; other outcomes preserve fallback. |
| K/D — product evidence | Can admitted results bake exactly and serve one demo prop? | clip atlas, provenance/fallback map and demo evidence | Still experimental until a promoting ADR and ProductChecks. |

## Milestones

| ID | Package | State | Observable exit criterion |
| --- | --- | --- | --- |
| R0 | V19 failure research and V20 rebaseline | `COMPLETE` | Causal B0-only/F0-only/combined evidence and primary-source limits are recorded; V19 I0 is fail-closed without opening integration values. |
| M0a | Phase-consistent metric calibration | `COMPLETE / REPEAT_EXACT_REJECT / FAMILY_CLOSED` | [Exact result](../development/physical-sound-v20-m0-phase-consistent-metric-result-2026-09-01.md) proves deterministic controls and physical owners but rejects raw MRLM/absolute DE separation. |
| M0b | Confound-resistant metric calibration | `COMPLETE / REPEAT_EXACT_PASS` | [Exact result](../development/physical-sound-v20-m0b-confound-resistant-metric-result-2026-09-01.md) passes every separation, physical, monotonic, access and repeat gate. |
| P0c | V20 integration protocol | `COMPLETE / FROZEN_BEFORE_VALUES` | [Exact I1 rows, midpoint thresholds, counterfactuals and access boundary](../development/physical-sound-v20-p0c-i1-integration-protocol-2026-09-01.md) are frozen before I1 code. |
| I1 | Frozen B0+C0+F0 integration | `COMPLETE / REPEAT_EXACT_REJECT / ACTUAL_ACOUSTIC_PASS` | [Exact result](../development/physical-sound-v20-i1-frozen-integration-result-2026-09-01.md) isolates one F0 remesh reject and one P0c counterfactual-owner defect. |
| G1/F1 | Evidence-triggered component revision | `OPENED_AS_V21_F1` | [V21](physical-sound-synthesis-roadmap-v21.md) reserves fresh paired `1801…2101` bands for a mesh-consistent field family; global B0 remains frozen. |
| S1 | Published-source growth | `PARALLEL / SOURCE_INSUFFICIENT` | Each candidate source gets exact revision/member/axis/provenance evidence or a machine-readable closure reason. |
| M1 | Metal immutable role freeze | `BLOCKED_BY_S1` | Five generator-ready and three evaluation-complete groups satisfy unchanged `4/1/1/1/1`; exposed aliases receive no protected credit. |
| R1 | Disclosed-real Metal tournament | `BLOCKED_BY_I1_AND_M1_GENERATOR_PREREQUISITE` | One preregistered modal candidate beats deterministic and authored controls on generator-visible roles without validator access. |
| V0 | Independent validator calibration | `BLOCKED_BY_M1_EVALUATION_PREREQUISITE` | Hard/physical/acoustic/OOD ensemble calibrates on its exclusive role and freezes code, weights, thresholds and policy. |
| V1 | Independent method holdout | `BLOCKED_BY_R1_V0` | Frozen generator and validator process the untouched method holdout once; reject closes the family. |
| A0 | Metal protected shadow | `BLOCKED_BY_V1` | One untouched shadow opens once and emits immutable `Pass`, `Reject` or `FallbackOOD`. |
| K0/D0 | Cooker and demo prop | `BLOCKED_BY_A0_PASS` | Byte-identical dry clips, provenance and complete fallback map feed one presentation-only scene object through the existing audio path. |
| G2 | Wood then Glass | `AFTER_METAL_A0` | Each material repeats source roles, validator and protected admission without inheriting Metal thresholds blindly. |
| P1 | Production promotion | `POST_RESEARCH / ADR_REQUIRED` | Concrete consumer, admissible real evidence, bounded Linux cost and separate Accepted ADR authorize contracts and ProductChecks. |

## Implementation order

1. Commit the V19 development rejection and fail-closed runner.
2. Preserve the M0a reject and repeat-exact M0b pass as immutable lineage.
3. Preserve frozen P0c formulas, midpoint thresholds and fresh `1701…1712`
   metadata root without generating an I1 value.
4. Commit I1 code/tests before generating any I1 value, then execute exactly
   twice. Attribute any reject before choosing a successor lane.
5. Continue S1 in parallel. R1 requires both I1 pass and generator-ready source
   roles; V0 requires independent evaluation-ready sources.
6. Freeze the generator before V0, freeze the validator before V1, and open the
   protected shadow only after V1 pass.
7. Bake/demo only after A0 pass. Wood and Glass repeat the unchanged role
   isolation and may remain `FallbackOnly` indefinitely.
8. Consider runtime/product promotion only through a separate ADR after a real
   consumer and product checks exist.

## Stop rules

1. Do not run V19 I0, generate `1601…1612` values or reinterpret its old raw
   waveform/envelope thresholds as a pass.
2. Do not choose a V20 metric or threshold from I1, disclosed-real holdout,
   validator, method-holdout or protected-shadow values.
3. Do not remove physical parameter gates merely because a spectral metric
   passes; frequency, damping, gain and topology remain independently binding.
4. Do not make phase-insensitive magnitude loss the only validator. It must be
   paired with modal correspondence, decay/transient evidence and hard checks.
5. Do not refit B0/F0 unless I1 causally attributes a real component failure.
   Any successor gets new identities and a preregistered family.
6. Do not use a prompted sound model, generator embedding or the candidate's
   training loss as the independent validator.
7. No microphone/hammer task for the user, per-sound human approval queue,
   runtime learning/inference, raw PhysX callback mixing or fallback removal.
8. Unknown or incompatible redistribution terms exclude bytes from a
   distributed product even when scientific observations can be cited.

## Definition of done

V20 completes its research objective only when:

- M0b proves the automatic metric contract on successful and harmful controls;
- I1 passes twice exactly on fresh identities;
- enough disclosed Metal sources satisfy immutable generator and evaluation
  roles;
- a frozen generator and independent validator pass one untouched protected
  Metal shadow exactly once;
- accepted outputs cook byte-identical clips with complete provenance and
  fallback for one demo prop;
- Wood and Glass either repeat the same admission discipline or remain explicit
  fallback without blocking the playable game.

This still does not ship runtime ML. Production promotion remains a separate
architecture decision with a concrete consumer, measured Linux budget and
relevant ProductChecks.
