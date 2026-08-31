# Roadmap V14: data-first neural physical sound authoring

| Field | Value |
| --- | --- |
| Rebaseline date | `2026-08-31` |
| Status | `SUPERSEDED_BY_V15 / N1B_REPEAT_EXACT_COVERAGE_INSUFFICIENT / RUNTIME_NOT_AUTHORIZED` |
| Replaces | [Roadmap V13](physical-sound-synthesis-roadmap-v13.md) as the active execution plan |
| Replaced by | [Roadmap V15](physical-sound-synthesis-roadmap-v15.md), progressive material admission |
| Evidence basis | [V14 data-first neural rebaseline](../development/physical-sound-v14-data-first-neural-rebaseline-2026-08-31.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Mandatory fallback | Existing authored/recorded clips for every reject, OOD, missing-data or tooling failure |

## Outcome

Build an offline neural authoring pipeline that turns a prop's mesh, bounded
material/scale/support hints and an impact query into a deterministic atlas of
plausible dry impact clips. The network learns object modes and how their gains
change over the surface; the engine receives only ordinary cooked audio and a
complete fallback map.

V14 does **not** promise recovery of the true material constants or an arbitrary
measured force-to-audio transfer function. Those are stronger independent
claims. The first useful claim is narrower:

```text
LearnedCanonicalImpactPrior
  mesh + bounded acoustic hints + contact + canonical energy
      -> stable modal field + uncertainty
      -> deterministic 48 kHz clip atlas
```

For exact dataset objects, a few-shot reconstruction lane checks whether the
representation can predict untouched contacts. For new game props, a shared
prior produces plausible candidates and must be willing to select fallback.

## Why V13 is rebaselined

V13 built the right research substrate—versioned records, an exposure ledger,
signal-blind role freezes and repeat-exact source adapters—but made recovery of
one perfect canonical object the gate before all model work.

Two independent source failures now falsify that sequencing:

- ObjectFolder object `41` has valid force impulses but insufficient shared
  broadband coverage for a measured transfer claim.
- ObjectFolder object `92` has 36 microphone/force pairs but is missing one
  required metadata member; the archive has already advanced to object `93`.

Neither failure says that a learned canonical impact field is impossible.
Object `41` and `92` remain closed negative fixtures; their gates are not
lowered. RealImpact `93_GreenGoblet` remains a five-impact derived-response
control. V14 moves source enrichment off the sequential critical path and
starts with a multi-object dataset contract plus a known-truth oracle.

## Product and research boundaries

- Training, datasets, weights, arrays, generated WAVs and validator inference
  stay outside the repository and outside runtime.
- Gameplay hearing and simulation correctness remain independent of audio.
- The first model predicts a bounded structured sound field; it is not a
  prompt-to-audio model and does not generate arbitrary ambience or effects.
- The deterministic cooker is the only bridge into the current clip path.
- SPEC-45 remains `Proposed`; V14 adds no public contract or production status.
- Internet-published data is the only real evidence source. No local impact
  recording, microphone, hammer or force sensor is required from the user.

## Data program

| Tier | Source role | Permitted claim | Forbidden claim |
| --- | --- | --- | --- |
| T0 known truth | Engine-owned modal/FEM/BEM scenes and ModalSound meshes | Representation recovery, invariants, controlled OOD | Realism or material identity |
| T1 synthetic teacher | ObjectFolder 2.0 implicit object models | Broad contact/force curriculum and distillation | Independent real-quality validation |
| T2 sparse real contact | ObjectFolder-Real | Few-shot contact reconstruction, real spectral/decay calibration | Universal arbitrary-force transfer unless a separate coverage certificate passes |
| T3 dense real listener | RealImpact preprocessed response | Derived-response and later listener/radiation controls | Raw paired-force provenance |
| T4 audio-only real | Existing identified internet corpus | Material, artifact and real/mutation validator specialists | Geometry, contact, force or transfer evidence |

All source bytes are hash-closed in the external experiment store. Objects are
partitioned before signal decode. Training may use a signal-blind quality mask;
method holdout and admission shadow require their declared axes to be complete
and may not be repaired after opening.

## Model program

The first candidate has four explicit parts:

1. a mesh/local-surface encoder with material, scale and support conditioning;
2. an optional few-shot audio encoder for exact-object reconstruction;
3. a stable global frequency/damping head plus contact-conditioned bounded
   complex modal gains and uncertainty;
4. the existing deterministic differentiable modal renderer and clip cooker.

Mesh-only features are the baseline. Image encoders, DINO features and 3DGS are
allowed only as later capacity ablations after the mesh baseline and controls
are frozen. An unconstrained residual is not part of the first candidate. A
direct neural waveform decoder may measure a report-only quality ceiling, but
cannot bypass the structured candidate, validator or cooker.

## Milestones

| ID | Package | State | Observable exit criterion |
| --- | --- | --- | --- |
| N0 | Evidence rebaseline | `COMPLETE` | V12/V13 negatives preserved; source recovery removed from the critical path; V14 claim and fallback are explicit. |
| N1 | Dataset Contract V1 | `IN_PROGRESS / N1B_COMPLETE_COVERAGE_INSUFFICIENT / N1C_SCOPE_DECISION_NEXT` | N1c either proves at least `8` eligible real object groups per admitted material and freezes roles, or narrows the first protected domain before signal decode; protected signal counters stay zero. |
| N2 | Known-truth neural oracle | `BLOCKED_BY_N1_ROLE_SCOPE_FREEZE` | A small network recovers stable poles/gains and held surface responses on synthetic objects; instability, contact shuffle, wrong material/scale and coverage-collapse mutations reject. |
| N3 | Real analysis/synthesis representation | `BLOCKED_BY_N2` | On opened train/development audio, learned modal tokens reconstruct spectrum, onset, envelope and decay within frozen absolute gates for all three material families; identity and classical Q30/DCT controls are reported. |
| N4 | Exact-object few-shot contact field | `BLOCKED_BY_N3` | With about 20% signal-blind contacts, one shared model beats nearest-contact, Euclidean/geodesic RBF, local-linear and geometry-agnostic controls on untouched contacts for at least two objects per material, or closes as `REPRESENTATION_REJECTED`. |
| N5 | Cross-object authoring prior | `BLOCKED_BY_N4` | An object-disjoint mesh/material model produces contact-continuous candidates for unseen objects, beats material-mean/nearest-object controls and emits calibrated OOD instead of silent collapse. |
| N6 | Independent Validator V2 | `CAN_DEVELOP_AFTER_N1 / ADMISSION_BLOCKED_BY_N5` | Frozen hard, physics, acoustic, corpus and selective-risk specialists bound parent-grouped false-pass risk and retain useful real coverage on method holdout. Generator training cannot see validator calibration/model parameters. |
| N7 | Deterministic atlas cooker | `BLOCKED_BY_N5_N6` | Canonical contact regions and energy bins cook twice to byte-identical 48 kHz clips, bounded metadata and a complete fallback map; no weights are needed for playback. |
| N8 | One-shot shadow + demo | `BLOCKED_BY_N7` | One untouched Glass, Wood and Metal object each receives exactly one immutable `Pass`, `Reject` or `FallbackOutOfDomain`; admitted clips run through the existing demo contact path with fallback/fault non-regression. |
| N9 | Domain growth | `BLOCKED_BY_N8` | New geometry/support/material domains are added by the same pipeline without per-object thresholds or reopened shadows; failed domains remain durable `FallbackOnly`. |
| N10 | Production impact prop | `POST_RESEARCH / ADR_REQUIRED` | One concrete prop justifies engine-owned content/projection contracts and passes focused content-package, play and conditional performance checks under a new Accepted ADR. |
| N11 | Rolling and scraping | `OUTSIDE_V14_CRITICAL_PATH` | Separately scoped persistent-contact corpus and exciter after the impact vertical; no impact-model noise tuning is relabelled as rolling/scraping. |
| N12 | Measured transfer upgrade | `OPTIONAL / SOURCE_BLOCKED` | A different published source passes broadband paired-force coverage before any arbitrary-force claim is reopened. |

## N1 — Dataset Contract V1

N1 replaces the search for one perfect object with a strict multi-object pool.
It must:

- inventory source/member identity, geometry/contact/audio axes and acquisition
  cost without reading PCM or numeric force arrays;
- select object groups using only publisher metadata and structural presence;
- freeze train, generator development, validator calibration, method holdout
  and admission shadow by object and publisher revision;
- keep Glass, Wood and Metal represented in every protected evaluation role;
- distinguish `training_usable`, `evaluation_complete` and `source_ood` without
  letting a training-only sample create evaluation credit;
- bind all exclusions before any signal-derived feature exists;
- carry forward the exposure ledger so previously opened objects cannot become
  fresh shadow evidence.

The target of eight eligible groups per material is a go/no-go inventory gate,
not permission to keep downloading indefinitely. If it cannot be met from
published sources under bounded acquisition, N1 publishes exact coverage and
V14 narrows the first domain before any waveform decode.

[N1b](../development/physical-sound-v14-n1b-real-source-metadata-inventory-result-2026-09-01.md)
now supplies that exact negative inventory: only `1` unexposed Glass, `7` Wood
and `18` Metal real metadata candidates are proven under the current source
and revision rules. ObjectFolder 2.0 supplies `26/622/151` synthetic teachers,
but T1 cannot replace protected T2/T3 evidence. N1c is therefore a bounded
source/identity and scope decision; it is not permission to assign incomplete
roles or open waveforms.

The minimum signal-blind role shape per material is `4 train + 1 generator
development + 1 validator calibration + 1 method holdout + 1 admission shadow`.
Publisher/project revisions remain an additional grouping boundary; counts do
not permit the same source revision to leak across incompatible roles.

## N2–N4 — prove the representation before generalization

N2 is a known-truth unit test for learning. It answers whether the selected
network/output parameterization can recover a field whose modes and gains are
known exactly. It is no longer blocked by a real raw-force source.

N3 then learns an analysis/synthesis representation from real train/development
recordings with direct spectral, modal, onset, envelope and decay objectives.
It must beat the already-rejected generic codec/compact-representation failure
modes rather than merely improve waveform MSE.

N4 adds contact and geometry. The first evaluation is exact-object few-shot,
because it separates representation feasibility from cross-object
generalization. One frozen hyperparameter/model revision is used across all
opened objects. If a classical interpolator wins, it becomes the candidate;
“uses a neural network” is not itself a success criterion.

## N5 — useful authoring for unseen props

N5 removes the exact-object reference requirement. The model learns a prior
over object-global modes from mesh, acoustic material family, scale and support,
then predicts contact gains and uncertainty. Optional public few-shot references
may refine an object, but zero-shot generation is evaluated separately and may
have a narrower admitted domain.

Success is not exact waveform identity for an unseen prop. It requires:

- object-disjoint material/shape evidence against simple baselines;
- stable positive damping and bounded energy for every cooked query;
- surface continuity except at declared geometry/support discontinuities;
- monotonic response across canonical impact-energy bins;
- calibrated `FallbackOutOfDomain` for unsupported geometry/material/support;
- no protected-object choice, threshold or model revision after opening.

## N6 — automatic validation at scale

The validator is a separate release, not a loss function copied from training:

- **hard:** provenance, hashes, splits, schema, finite values, bounds and exact
  repeats;
- **physics:** stable modes, positive decay, energy bounds, excitation
  monotonicity and contact continuity;
- **acoustic:** multi-resolution spectrum, onset, envelope, decay and artifact
  statistics against held real distributions;
- **semantic:** frozen material/shape specialists using source/object-disjoint
  real data and controlled wrong-material negatives;
- **risk:** calibrated OOD and a confidence upper bound on parent-grouped false
  passes, reported together with coverage.

Human audition is optional report-only auditing of a validator release. It is
never required for every object, clip or iteration.

## N7–N10 — deterministic product bridge

The cooker samples a bounded surface-region × canonical-energy grid and emits
ordinary clips, exact hashes, coverage and fallback. Runtime does not load the
network, dataset or validator. Unsupported queries choose the authored clip
fallback deterministically.

N8 opens each admission-shadow object once. A pass proves only its declared
research domain. It does not make SPEC-45 shipped. N10 begins only when a
visible gameplay prop needs the capability; that package owns the later ADR,
content contract, committed contact projection and ProductChecks.

## Stop rules

1. No threshold, role, contact, object or model selection after its protected
   signal role is opened.
2. Two coherent failures of the same architecture family require a new
   discriminating hypothesis, not a larger sweep.
3. Failure of N2 stops real neural training; failure of N3 stops contact-field
   training; failure of N4 stops zero-shot generalization.
4. A neural candidate must materially beat compatible classical controls. If
   it does not, the simpler candidate wins or the domain becomes fallback-only.
5. A validator cannot admit the generator release used to train or tune it.
6. No object-41 force gate reduction, object-92 role reduction, local recording,
   hidden human approval or runtime neural inference.
7. No residual head until the modal/contact branch passes its own held gates.
8. No arbitrary-force claim without a fresh measured-transfer certificate.

## Completion criterion

V14 research is complete when one immutable generator/cooker revision and one
independent validator release process untouched Glass, Wood and Metal shadow
objects exactly once, publish tri-state decisions, bake byte-identical clips
for every pass and prove complete deterministic fallback for every other case.

`Reject` or `FallbackOutOfDomain` on one or more materials is an honest result;
it narrows coverage. If all three reject, V14 still closes successfully as a
reproducible negative result and the current authored clip path remains the
product answer.

## Immediate commit sequence

1. `N0a` — `COMPLETE`: V14 research note, roadmap rebaseline and durable state.
2. `N1a` — `COMPLETE / REPEAT_EXACT_ZERO_SOURCE_SIGNAL`: [protocol](../development/physical-sound-v14-n1a-dataset-contract-v1-protocol-2026-09-01.md)
   and [result](../development/physical-sound-v14-n1a-dataset-contract-v1-result-2026-09-01.md)
   freeze quality, exposure, object/recording grouping and exact `4/1/1/1/1`
   per-material role invariants.
3. `N1b` — `COMPLETE / REPEAT_EXACT / COVERAGE_INSUFFICIENT`:
   [protocol](../development/physical-sound-v14-n1b-real-source-metadata-inventory-protocol-2026-09-01.md)
   and [result](../development/physical-sound-v14-n1b-real-source-metadata-inventory-result-2026-09-01.md)
   bind `1,150` object rows and `70` archive identities with zero signal read.
4. `N1c` — resolve the bounded `71…100` revision lineage and additional
   published T2/T3 source option, then freeze roles only for a domain that
   meets the unchanged go/no-go coverage certificate; otherwise narrow N1.
5. `N2a` — freeze the synthetic scene distribution, controls, losses, metrics,
   compute ceiling and stop rule before training.
6. `N2b` — execute repeat-exact known-truth oracle and either authorize N3 or
   close the representation family.
