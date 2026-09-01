# Roadmap V21: mesh-consistent physical-sound formula and evidence admission

| Field | Value |
| --- | --- |
| Rebaseline date | `2026-09-01` |
| Status | `ADOPTED / V20_I1_REPEAT_EXACT_REJECT / ACTUAL_ACOUSTIC_PASS / P0D_OPEN / F1_VALUES_SEALED / SOURCE_GROWTH_PARALLEL / ADMISSION_BLOCKED / RUNTIME_NOT_AUTHORIZED` |
| Replaces | [Roadmap V20](physical-sound-synthesis-roadmap-v20.md), closed by its repeat-exact I1 reject |
| Evidence basis | [V20 I1 result](../development/physical-sound-v20-i1-frozen-integration-result-2026-09-01.md), [V21 remesh research](../development/physical-sound-v21-remesh-consistency-research-2026-09-01.md), [M0b metric pass](../development/physical-sound-v20-m0b-confound-resistant-metric-result-2026-09-01.md), [F0 field pass](../development/physical-sound-v19-f0-residual-harmonic-field-result-2026-09-01.md) and [source insufficiency](../development/physical-sound-v15-s0c-source-sufficiency-role-freeze-result-2026-09-01.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Mandatory fallback | Existing authored/recorded clip for every reject, OOD, absent source, unsupported material or tooling failure |

## Outcome sought

Reach an automated internet-data-to-formula loop that does not require the user
to approve every sound:

```text
published internet sources + immutable role split
  -> source and coverage certificate
  -> bounded modal formula trained only on generator-visible roles
  -> independent hard + physical + acoustic validator
  -> one untouched protected decision
  -> byte-exact offline clip cooker + complete fallback map

synthetic mesh/contact truth
  -> B0 global modes + C0 coverage + mesh-consistent F1 contact field
  -> fresh phase-consistent integration certificate

any missing evidence / OOD / reject / fault
  -> authored clip fallback
```

V21 first repairs the two exact V20 blockers. It then rejoins the already
defined real-evidence admission lane. No milestone below authorizes runtime
training/inference, public contracts, arbitrary-force transfer, listener/room
radiation or a claim that synthetic truth is real material acoustics.

## Why V21 exists

V20 I1 ran twice with nine byte-identical files. Its complete combined endpoint
passes MRSC, mean-centered log magnitude, decay-slope and transient-energy
thresholds, while B0 frequencies/damping and F0 absolute gain/gradient also
pass. The reject has only two attributed causes:

1. one Wood/Bowl primary/twin pair has relative gain-metric drift
   `0.1344067 > 0.10`, even though direct probe disagreement passes;
2. P0c incorrectly required the alternating-sign mutation to reject a spectral
   metric in addition to its calibrated signed-gain physical owner.

The second cause is repaired only by a successor protocol; historical P0c/I1
stay rejected. The first opens one new field family, not global-mode tuning,
metric loosening or an unrestricted model search.

## Immutable boundaries

- I1 identities and values `1701…1712` are attribution-only. They cannot choose
  an F1 architecture, basis order, weight, regularization, threshold, context,
  contact or checkpoint.
- B0 and C0 remain exact immutable dependencies. M0b's MRSC/MCLM/DSR/TE code
  and thresholds remain unchanged.
- P0d changes only counterfactual ownership: alternating signs must reject the
  signed-gain physical gate; acoustic metrics remain diagnostic for that case.
- Reserve paired F1 bands: `1801…1824` train, `1901…1912` development,
  `2001…2012` one-shot test and `2101…2112` one-shot reintegration.
- Metadata, meshes and values for a role remain sealed until the preceding
  protocol and implementation commit boundary says otherwise.
- The user records nothing. Real evidence comes only from disclosed internet
  sources with exact identity, revision, members, provenance and rights
  disposition.
- Datasets, audio, arrays, weights, reports and caches remain under
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`, never in Git.

## Technical strategy

### F1 first family

The primary F1 candidate is a small `ContinuousResidualOperatorV1`:

- preserve the learned topology-conditioned continuous prior;
- represent the observed contact residual in a fixed topology-native UV basis;
- solve bounded coefficients deterministically and evaluate the resulting
  continuous function directly at query/probe coordinates;
- train and evaluate explicit primary/remesh pairs;
- retain exact context observations, signed gains, C0 fallback and current
  model/resource ceilings.

The frozen F0 graph-harmonic operator, prior-only, graph harmonic and fixed
kernel residuals are controls. A pair-aware F0-prior ablation tests whether a
better prior alone removes the discrepancy. Candidate choice uses a small
predeclared train/development tournament and a deterministic ranking that puts
hard/coverage/remesh gates before aggregate accuracy.

Research supports continuous integral/spectral formulations on changing point
sets, but does not grant a pass: [GINO](https://arxiv.org/abs/2309.00583),
[MeshGraphNets](https://arxiv.org/abs/2010.03409), and
[Fourier operators on arbitrary domains](https://proceedings.mlr.press/v235/lingsch24a.html).
A large graph/geometry neural operator is an escalation only if both small F1
candidates fail fresh development evidence. It is not an automatic retry.

### Automatic validator remains layered

1. **Hard:** identity, schema, ordering, finite/range, coverage reason,
   corruption and fallback completeness.
2. **Physical:** frequencies, damping, signed modal gains, spatial gradients,
   modal peaks and remesh consistency.
3. **Phase-tolerant acoustic:** MRSC, mean-centered log magnitude, normalized
   decay slope and transient energy.
4. **Diagnostic:** raw waveform, Hilbert envelope, absolute decay energy and
   symmetric/absolute remesh deltas.

No scalar replaces another layer. Synthetic calibration, disclosed-real
candidate quality, independent validation and protected admission remain
separate certificates.

## Work packages and gates

| ID | Package | State | Observable exit criterion |
| --- | --- | --- | --- |
| R0 | V20 I1 attribution | `COMPLETE / REPEAT_EXACT_REJECT` | [Result](../development/physical-sound-v20-i1-frozen-integration-result-2026-09-01.md) isolates one F0 remesh reject and one P0c ownership defect; all actual acoustic gates pass. |
| P0d | Counterfactual owner correction | `OPEN` | Successor protocol preserves every M0b threshold and makes signed-gain rejection sufficient for alternating signs before any fresh integration value. |
| R1 | Remesh successor research | `COMPLETE / DIRECTION_SELECTED` | [Research decision](../development/physical-sound-v21-remesh-consistency-research-2026-09-01.md) selects a small continuous residual tournament and bounds neural-operator escalation. |
| P1a | F1 protocol and metadata freeze | `NEXT` | Exact paired rows, basis candidates, losses, controls, ranking, gates, resource ceilings, role access and output schema freeze before `1801…` mesh/value generation. |
| F1a | Train/development tournament | `BLOCKED_BY_P1A` | On `1801…1824`/`1901…1912`, one predeclared candidate passes hard/coverage/remesh gates and beats frozen F0 by the frozen ranking; otherwise F1 closes before test. |
| P1b | One-shot F1 test freeze | `BLOCKED_BY_F1A` | Winner identity, artifact hashes and unchanged test gates freeze while `2001…2012` values remain unopened. |
| F1b | One-shot F1 capability | `BLOCKED_BY_P1B` | Fresh paired test passes all absolute, gradient, remesh, control, mutation, serialization, access and repeat-exact gates twice. Reject closes F1. |
| P1c | Fresh integration protocol | `BLOCKED_BY_F1B_AND_P0D` | Exact B0+C0+F1 identities, `2101…2112` rows, M0b thresholds, corrected owners and fallback freeze before values. |
| I2 | Phase-consistent reintegration | `BLOCKED_BY_P1C` | Two byte-exact runs pass every hard, physical, remesh, acoustic, counterfactual, access and resource gate. |
| S1 | Published-source growth | `PARALLEL / SOURCE_INSUFFICIENT` | Each source gains exact revision/member/axis/provenance evidence or a machine-readable closure reason; no local recording. |
| M1 | Metal immutable role freeze | `BLOCKED_BY_S1` | Five generator-ready and three evaluation-complete groups satisfy unchanged `4/1/1/1/1`; aliases receive no protected credit. |
| R2 | Disclosed-real Metal tournament | `BLOCKED_BY_I2_AND_M1_GENERATOR` | One preregistered candidate beats deterministic/authored controls without validator-role access. |
| V0 | Independent validator calibration | `BLOCKED_BY_M1_EVALUATION` | Exclusive validator role freezes code, features, weights, thresholds, OOD and decision policy. |
| V1 | Independent method holdout | `BLOCKED_BY_R2_AND_V0` | Frozen generator and validator process the untouched method holdout once; reject closes the family. |
| A0 | Metal protected shadow | `BLOCKED_BY_V1` | One untouched shadow opens once and emits immutable `Pass`, `Reject` or `FallbackOOD`. |
| K0 | Deterministic clip cooker | `BLOCKED_BY_A0_PASS` | Accepted modal records bake byte-identical dry clips plus provenance and complete fallback map. |
| D0 | One demo prop | `BLOCKED_BY_K0` | One presentation-only object uses cooked clips through the existing audio path; gameplay remains independent and offline. |
| G2 | Wood then Glass | `AFTER_METAL_A0` | Each material repeats source roles, validator and protected admission; no inherited Metal threshold by assumption. |
| P2 | Production promotion | `POST_RESEARCH / ADR_REQUIRED` | A concrete consumer, admissible real evidence, measured Linux budget and separate Accepted ADR authorize contracts/ProductChecks. |

## Ordered implementation queue

1. Preserve I1 result, V21 research and this roadmap as the new baseline.
2. Freeze P0d. It is a protocol correction only and generates no fresh value.
3. Freeze P1a metadata and the exact bounded F1 tournament before opening train
   meshes or values.
4. Run F1a on train/development only. If no candidate passes the predeclared
   ranking, close F1 and research a genuinely new F2 family with new bands.
5. Freeze the selected artifact and P1b, commit implementation/tests, then run
   F1b twice on unopened `2001…2012`.
6. Only after F1b pass, freeze P1c and run I2 twice on `2101…2112`.
7. Continue S1 independently. Real candidate work requires both I2 and the
   corresponding immutable generator-role certificate.
8. Freeze generator before validator, validator before method holdout, and
   method holdout before protected shadow.
9. Bake and demonstrate only after A0 `Pass`; keep fallback for every other
   decision. Consider product promotion only through a separate ADR.

## Stop rules

1. Never reinterpret V20 I1 as pass, rerun `1701…1712` to select a fix or lower
   the `0.10` remesh gate after seeing Wood/Bowl.
2. Do not change M0b acoustic thresholds or make spectral rejection own signed
   modal polarity. P0d corrects owner assignment, not values.
3. Do not open F1 test or reintegration values before the relevant protocol,
   code and artifact commit boundary.
4. A rejected one-shot F1b or I2 cannot choose a nearby seed, basis order,
   width, contact, threshold or checkpoint. A successor needs new identities.
5. Do not adopt GINO, MeshGraphNet or another large operator merely from prior
   art; first prove the small continuous family insufficient on fresh dev.
6. Do not use prompted audio, generator embeddings or the candidate training
   loss as the independent validator.
7. No local microphone/hammer work, per-sound human approval queue, runtime
   learning/inference, raw PhysX callback mixing or fallback removal.
8. Unknown or incompatible redistribution terms exclude bytes from a
   distributed product even when scientific observations may be cited.

## Definition of done

The research-to-product vertical is complete only when:

- F1 and I2 pass fresh paired-mesh, physical and phase-consistent synthetic
  evidence twice exactly;
- enough disclosed Metal internet sources satisfy immutable generator and
  evaluation roles;
- one frozen generator and independent validator pass one untouched Metal
  shadow exactly once;
- accepted outputs cook byte-identical clips with provenance and complete
  fallback for one demo prop;
- Wood and Glass repeat the same admission discipline or remain explicit
  fallback without blocking the playable game.

Even then runtime ML is not implied. Production promotion remains a separate
architecture decision with a concrete consumer, bounded Linux cost and the
relevant ProductChecks.
