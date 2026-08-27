# Automated physical-sound validation research — 2026-08-27

| Field | Result |
| --- | --- |
| Scope | Offline validation of large families of generated physical sounds; no runtime or public-contract promotion |
| Primary question | Can Next Engine accept or reject generated sounds without per-sound human audition? |
| Conclusion | Yes, for a declared source family and parameter domain, by validating the generator with hard, causal, reference-distribution and selective-risk gates; no single general audio model is sufficient |
| Automatic outcome | `Pass`, `Reject` or `FallbackOutOfDomain`; routine `NeedsHumanAudit` is removed from the target pipeline |
| Claim limit | Without any perceptual ground truth, the pipeline can prove signal safety, physical/control coherence and similarity to a frozen real corpus, but not universal subjective naturalness |

## Executive conclusion

The scalable unit of validation is not an individual WAV. It is a versioned
generator over a declared domain such as:

`rigid impact × object family × material family × impact position × force × listener pose`.

The validator samples that domain reproducibly, tests boundaries and causal
relations, compares the result with held-out real recordings and accepts only
the region where a calibrated ensemble has bounded error. Unknown shapes,
materials or source mechanisms do not request a person to listen. They select
the existing authored-clip fallback and become evidence for a later validator
version.

The resulting pipeline can be fully automatic in CI and in a content cooker.
Human listening is not a per-asset gate, and the product owner does not need to
review the sound inventory. Existing human-labelled public data or an optional
one-time/periodic panel can improve the perceptual head, but the operational
decision remains reproducible and model-versioned. A zero-human variant is
also coherent if its claim is deliberately limited to physical consistency and
real-corpus similarity.

## Why one neural judge is not enough

The available evidence argues for decomposition rather than a universal score:

- [Meta Audiobox Aesthetics](https://ai.meta.com/research/publications/meta-audiobox-aesthetics-unified-automatic-quality-assessment-for-speech-music-and-sound/)
  is a useful open no-reference model, but it predicts four broad axes of
  enjoyment, usefulness, production complexity and production quality. Those
  axes do not encode impact force, object geometry, modal structure or
  material identity. It is an artifact/production-quality head, not the gate.
- [T2A-Feedback](https://aclanthology.org/2025.acl-long.1147/) reports better
  alignment by splitting automatic audio feedback into event occurrence,
  event sequence and acoustic/harmonic quality. The relevant precedent is the
  decomposed evaluator, not its text-to-audio task or model outputs.
- [Human-CLAP](https://arxiv.org/abs/2506.23553) reports low correlation between
  ordinary CLAPScore and subjective relevance and improves it by fine-tuning
  on subjective scores. This matches the local observation that CLAP can admit
  metal-like glass. Text similarity is therefore only an auxiliary feature.
- FAD is a distribution metric, not a reliable single-item oracle. Its
  [original validation](https://research.google/pubs/frechet-audio-distance-a-reference-free-metric-for-evaluating-music-enhancement-algorithms/)
  reported only moderate correlation with human distortion judgements, and
  [later work](https://www.microsoft.com/en-us/research/publication/adapting-frechet-audio-distance-for-generative-music-evaluation/)
  found sensitivity to sample count, embedding choice and reference-set
  quality.

The evaluator must therefore keep hard failures, physical relations,
reference similarity, generic perceptual quality and uncertainty as separate
evidence. A learned scalar may rank candidates inside a validated region, but
it cannot override a failed hard or causal gate.

## Proposed `PhysicalSoundValidatorP0`

### 1. Hash-closed input and domain declaration

Each run consumes an external manifest containing:

- generator revision and source-family ID;
- exact candidate, reference-corpus and evaluator/model hashes;
- sample rate, output profile and deterministic probe seed;
- declared parameter bounds and units;
- `train`, `calibration`, `holdout` and `shadow` object sets;
- provenance and redistribution status for every corpus component.

Raw recordings, generated probes, checkpoints and feature caches remain out of
Git. The report stores hashes, summaries and exact split identities.

### 2. Hard signal and numerical gate

This extends the implemented classical evaluator and rejects before any learned
score:

- malformed/oversized audio, NaN or non-finite state;
- silence, clipping, excessive DC, missing onset or invalid duration;
- unstable resonators, poles outside the declared stable region and unbounded
  accumulator/voice growth;
- energy above the declared excitation/output envelope;
- alias-prone modes or content above the usable Nyquist guard band;
- non-repeatable PCM/report hashes for an exact profile;
- unsafe CPU, memory, queue or voice-count bounds once a consumer exists.

These are ordinary deterministic tests. They need neither references nor a
neural model.

### 3. Causal and metamorphic gate

The generator is tested as a function, not as a bag of clips. For rigid impact:

- increasing force must increase pre-normalization energy monotonically inside
  the linear regime while leaving modal frequencies stable;
- zero force must produce silence and repeated identical input must reproduce
  identical PCM;
- small impact-position changes must produce bounded, locally continuous mode
  amplitude changes except at declared geometric discontinuities;
- impact position may change modal amplitudes but must not invent unrelated
  modal frequencies for one object revision;
- listener-pose changes must follow the cooked radiation/propagation model and
  must not change source identity;
- gain normalization must not erase the force relation in the underlying
  diagnostic signal;
- material or geometry changes must affect only the declared acoustic model,
  not physics authority or gameplay roots.

Rolling, scraping, cloth, liquid, fire, voice and biological sources require
their own relation suites. Only signal safety, artifact detection, lineage and
selective-decision machinery are shared. A universal material/source validator
would repeat the same modelling error as a universal synthesizer.

### 4. Real-reference and representation gate

The best public starting points for rigid impact are:

- [REALIMPACT](https://graphics.stanford.edu/~djames/publication/realimpact-a-dataset-of-impact-sound-fields-for-real-objects/):
  150,000 controlled recordings of 50 objects with impact location, microphone
  location, contact-force profile, material and RGBD annotations;
- [ObjectFolder Real](https://objectfolder.stanford.edu/objectfolder-real-download): 100 real household
  objects recorded in an anechoic chamber at multiple surface locations with
  impact-hammer force and meshes. Its public project lists material
  classification and contact-localization benchmarks; the repository declares
  CC BY 4.0 for ObjectFolder, subject to per-source asset review. The Real
  download page does not explicitly say that this grant covers its separate
  recording archives, so the current adapter refuses to infer that coverage.

They support matched physical tests that generic environmental-audio datasets
cannot: force classification/regression, contact localization, material and
object-family discrimination, and real-versus-simulation embedding distance.
REALIMPACT's public repository currently says that some raw data/code packaging
is incomplete. Its MIT license names software rather than the recording
archive, so acquisition and dataset terms must be verified before freezing a
corpus.

Candidate representations should be benchmarked rather than selected by
reputation. The shortlist is:

- the existing modal, spectrum, decay, onset and temporal descriptors;
- frozen general-audio embeddings such as the official
  [BEATs](https://github.com/microsoft/unilm/tree/master/beats) checkpoints;
- Human-CLAP only for coarse semantic/material text alignment;
- Audiobox Aesthetics `PQ` only for broad recording/artifact quality;
- a small Next Engine rigid-impact encoder or ranker trained on frozen features
  if general embeddings fail the held-out tests.

All external models are offline optional tools with pinned weights. A missing
model must never download implicitly during a reproducible run; it produces an
explicit unavailable diagnostic and selects the conservative fallback policy.

### 5. Learned specialist heads

One monolithic quality score is replaced by independently testable heads:

| Head | Target | Training/evaluation signal |
| --- | --- | --- |
| Material confusion | `glass`, `steel`, `wood`, ceramic and declared subfamilies | Real corpus labels; leave-object-out |
| Contact position | Surface region or continuous location | Real impact location; leave-position/object-out |
| Force coherence | Force band/regression and monotonic ordering | Measured hammer force plus generated metamorphic pairs |
| Object/geometry identity | Same/different object or family | Contrastive real recordings; leave-object-family-out |
| Artifact quality | Clipping, aliasing, smear, noise, room leakage | Deterministic corruptions plus Audiobox diagnostics |
| Candidate ranker | Candidate closer to the matched real distribution | Matched real/synthetic pairs and corruption severity ordering |
| OOD detector | Input outside admitted corpus/domain | Held-out materials, source mechanisms, rooms and generators |

Training examples need not be manually labelled one by one. Most labels come
from controlled acquisition metadata and deterministic mutations. Subjective
labels, if admitted, are a frozen auxiliary dataset and never a live approval
queue.

### 6. Selective acceptance and bounded risk

The validator should be allowed to abstain. Its automatic result is:

- `Pass`: hard and causal gates pass, the input is in-domain and calibrated
  risk is below the declared bound;
- `Reject`: a diagnosed hard, causal or high-confidence learned failure;
- `FallbackOutOfDomain`: the model is uncertain, evaluators disagree or the
  input is outside the calibrated domain; use authored/cached audio.

The useful top-level metric is not raw classifier accuracy. It is the
**risk–coverage curve**: error among accepted candidates versus the proportion
automatically accepted. The acceptable false-pass risk must be selected from a
frozen calibration split with a confidence bound. Conformal risk control is a
plausible mechanism because it chooses a threshold for a user-defined bounded
loss on held-out calibration data; the
[reference implementation](https://github.com/aangelopoulos/conformal-risk)
describes finite-sample expected-risk control. Its exchangeability assumption
does not survive arbitrary new materials or generators, so OOD detection and
generator/material-group holdouts remain mandatory. No numeric target is
invented before the pilot exposes the achievable curve.

## Validator tests

### Positive controls

- hash-frozen real recordings from admitted object/material families;
- exact synthetic self-match and the current Q30 numeric-transfer corpus;
- known-good steel/wood/glass anchors retained independently of training;
- gain, channel and encoding variants that should preserve the normalized
  judgement.

### Negative and mutation controls

- silence, clipping, DC, truncation and missing onset;
- pitch/modal shifts, removed modes, excessive narrow ringing and wrong decay;
- attack smear, late noise, quantization-floor exaggeration and aliasing;
- force inversion, force-dependent frequency drift and discontinuous position
  response;
- metal-like glass anchors D/F and other previously rejected candidates;
- room/reverb contamination, microphone coloration and unrelated source
  classes;
- adversarial parameter search that maximizes the learned score while violating
  an independent physical or holdout criterion.

Every mutation family has a severity ladder. The score or rejection rate must
be monotonic enough to detect both blind spots and reward hacking.

### Split discipline

Random clip splits are insufficient because adjacent strikes of the same object
leak identity. Report at minimum:

- leave-one-object-out;
- leave-one-object-family-out where data permits;
- leave-position-out and leave-force-band-out;
- leave-generator-revision-out;
- leave-mutation-family-out;
- a final shadow split never used for threshold or model selection.

Results are macro-averaged by material/source family and include confidence
intervals. A global aggregate cannot hide a glass regression behind abundant
steel examples.

### Required report fields

- false-pass and false-reject counts by family and failure tag;
- risk–coverage curve and the selected calibration threshold;
- OOD detection result and coverage by material/object/generator group;
- pairwise/ranking agreement on holdout, not training fit;
- metamorphic relation pass/fail and worst counterexample;
- deterministic repeated-run hashes;
- evaluator/model/corpus lineage and unavailable-component diagnostics;
- adversarial/reward-hack test result;
- cost and cache hit rate kept separate from acoustic quality.

## CI and content-cooker shape

The pipeline can scale without evaluating every point exhaustively:

1. **Per change, cheap:** hard signal, exact determinism, prior counterexamples
   and a small boundary/pairwise probe set.
2. **Nightly or pre-release:** deterministic low-discrepancy/boundary sampling
   across the full admitted parameter domain, all metamorphic relations and
   cached frozen embeddings.
3. **Validator release:** full real-corpus holdouts, selective-risk calibration,
   mutation severity ladders, leave-generator-out and adversarial optimization.
4. **Content cooking:** accept only a generator/profile pair covered by the
   validator manifest; otherwise package the declared authored fallback.

Features are cached by `(wav_hash, evaluator_hash)`. Learned evaluation can run
outside Cargo and outside the shipping engine; the deterministic Rust report is
the integration boundary. This preserves offline play and prevents a model,
network or provider from becoming runtime authority.

## Smallest implementation sequence

### AV-P0A — deterministic generator validator

Extend `xtask physical-sound-eval` with domain probes, hard numerical checks,
metamorphic relations, deterministic mutation ladders and `Pass / Reject /
FallbackOutOfDomain` report vocabulary. Keep learned acceptance disabled. This
already removes manual review for provable failures and physical-control bugs.

### AV-P0B — corpus benchmark, no acceptance authority

Build an external adapter for a license-reviewed ObjectFolder Real subset and,
if available under acceptable terms, REALIMPACT. Cache the current descriptors,
BEATs, Human-CLAP and Audiobox features. Compare heads and feature combinations
under grouped holdouts. Do not fit and evaluate on the same object.

Implementation checkpoint: `xtask physical-sound-benchmark` now supplies the
hash-closed external manifest/feature-matrix boundary, strict license-review
record, object/family-disjoint four-way partitions, real-only development
gallery and deterministic classical/external-feature baseline. It always emits
`NoAcceptanceAuthority`. Actual real-corpus scoring remains blocked until one
recording subset has explicit reviewed dataset terms; see the
[AV-P0B report](physical-sound-corpus-benchmark-av-p0b-2026-08-27.md).

### AV-P0C — selective specialist validator

Train the smallest rigid-impact ranker/classifier that materially improves the
holdout error. Calibrate the acceptance threshold and OOD fallback on separate
splits. Promote automatic `Pass` only if the risk–coverage report and all
mutation/adversarial controls pass.

### AV-P0D — autonomous synthesis loop

Allow parameter search to optimize a Pareto vector of reference fidelity,
causal coherence and cost. The validator remains outside the optimizer's
training/fit split, and a frozen shadow plus reward-hack suite decides whether a
new generator revision is admissible.

## Decision

Adopt the automated selective-validator direction for the next external work
package. Replace per-candidate human validation as a design goal; keep authored
fallback as the automatic abstention result. Do not promote a neural model or a
quality threshold until grouped real-corpus and mutation holdouts demonstrate
its actual risk–coverage envelope.

This is research evidence for proposed SPEC-45, not an Accepted runtime or
content contract and not a shipping-quality claim.
