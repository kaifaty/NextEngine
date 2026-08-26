# Physical sound quality evaluation research — 2026-08-26

| Field | Value |
| --- | --- |
| Status | `RESEARCH_COMPLETE / CLASSICAL_Q0_Q1_IMPLEMENTED / HUMAN_CALIBRATION_OPEN` |
| Question | How can Codex judge the impact-sound experiment well enough to run useful improvement iterations without asking the product owner to audition every candidate? |
| Result | Build a calibrated offline quality oracle over frozen real references, physical/perceptual descriptors, auxiliary learned embeddings and a small held-out human-preference set; no single metric or general audio model is an admissible judge |
| Trigger | Product-owner audition reports that the current P0 only remotely resembles the intended sound |
| Architecture boundary | Offline evidence tooling over PresentationOnly PCM; no gameplay authority, runtime training, public schema or P1 promotion |

## Executive conclusion

The current experiment has an engineering pass and a perceptual fail. Exact,
non-silent PCM and correct demo isolation prove that the signal reaches the
speaker without affecting gameplay. They do not prove that the signal resembles
a real struck object. The five heuristic modal presets were never fitted to a
recording or high-quality solver, so further manual coefficient tuning would be
an unmeasured search.

Codex can run mostly independent iterations after one bounded calibration
phase, but cannot discover the product owner's intended timbre from the current
WAV alone. The required teacher is a frozen target corpus plus a small set of
human comparisons. The durable interface to Codex should be a local evaluator
that returns a versioned JSON report, ranked candidates, uncertainty and named
failure modes. Codex can then change one hypothesis at a time, render a bounded
candidate batch, keep only improvements that generalize to held-out cases and
request another human judgment only when the evaluator is uncertain or its
subscores disagree.

The evaluator must be an ensemble. Signal validity, matched physical features,
control relationships, embedding distances, learned human preference and cost
answer different questions. Collapsing them into one uncalibrated scalar makes
the optimizer exploit the metric rather than improve the sound.

## What the negative audition changes

The observation is not evidence that modal synthesis is impossible. A controlled
study found that listeners could not distinguish matched low-parameter modal
syntheses from recordings better than chance for its ten-object corpus, but its
parameters were explicitly selected to match recordings. That supports fitting
a compact model; it does not support the current unfitted steel/wood/glass
heuristics. Source: [Lutfi et al., JASA 2005](https://pubmed.ncbi.nlm.nih.gov/16119360/).

The correct revised claim is therefore:

- the current P0 is acoustically inadequate by direct product-owner evidence;
- compact modal impact remains a live model family only after reference fitting;
- blind preset tuning is blocked until an evaluator is calibrated;
- production contact/content work remains blocked independently of quality.

## Primary-source findings

### A matched impact corpus is available in principle

[RealImpact](https://samuelpclarke.com/realimpact/) contains 150,000 controlled
48 kHz recordings of 50 real objects with impact location, microphone location,
contact-force profile, material label and geometry-related data. This is much
closer to the required calibration problem than a generic sound-effects library:
force and listener position can be separated from object response. Its data and
redistribution terms still need an exact review before use; any download or
derived corpus remains in an external research store, not the repository.

For a first local oracle, the complete dataset is unnecessary. A stratified
subset or engine-owned recordings of one metal-, wood- and glass-like object at
three impact positions and three force bands is enough to test whether the
current model family can rank obvious improvements. Multiple takes remain
necessary to estimate recording variance rather than fitting microphone noise.

### Human material judgment uses both spectral and temporal structure

Listening experiments on wood, metal and glass impacts found that material
similarity depends strongly on long-term spectral energy, while action
similarity depends on temporal-envelope distribution and onset density.
Removing spectral cues destroyed material recognition while retaining action
information; scrambling temporal envelopes produced the complementary effect.
Source: [Hjortkjær and McAdams, JASA 2016](https://orbit.dtu.dk/en/publications/spectral-and-temporal-cues-for-perception-of-material-and-action-/).

Separate work on perceived impact force found that listeners outperformed
models based only on signal power or spectral centroid and appeared to infer
force in relation to resonant material properties. Source:
[Traer and McDermott, JASA 2018](https://iro.uiowa.edu/esploro/outputs/journalArticle/Human-inference-of-force-from-impact/9984065470302771).

Consequently, the oracle needs both per-clip descriptors and relational tests:
more force should change energy plausibly without changing the object's modal
identity; impact position should change modal participation more than modal
frequencies; repeated takes should form a tighter cluster than different
objects; source quality must be measured before room propagation.

### Generic automatic metrics are useful but not sufficient

Fréchet Audio Distance compares distributions in an audio-embedding space. The
original paper reported better correlation with human distortion judgments
than several signal distances, but only moderate correlation and a music-
enhancement setting. Source: [Kilgour et al., Interspeech 2019](https://research.google/pubs/frechet-audio-distance-a-reference-free-metric-for-evaluating-music-enhancement-algorithms/).

Later work shows that FAD depends on sample count, embedding choice and the
quality/diversity of the reference corpus. Source:
[Gui et al., ICASSP 2024](https://www.microsoft.com/en-us/research/publication/adapting-frechet-audio-distance-for-generative-music-evaluation/).
The DCASE 2024 sound-synthesis challenge accordingly used PANN embeddings and
FAD to shortlist systems, then used blinded human ratings for the final order;
the final ranking ignored FAD order inside that shortlist. Source:
[DCASE 2024 Task 7](https://dcase.community/challenge2024/task-sound-scene-synthesis).

For this tiny matched corpus, FAD is therefore a secondary distribution check,
not a per-clip loss or promotion gate. It becomes more useful after enough
reference/candidate samples exist and must be bootstrapped at equal sample
counts with a frozen embedding version.

[ViSQOL](https://github.com/google/visqol) is a reproducible full-reference
spectro-temporal metric and supports 48 kHz audio, but its own documentation
states that it was designed around codec/VoIP degradations and may perform
poorly on generative models. It can diagnose aligned reference/candidate pairs
after local human calibration; its MOS-LQO is not automatically a realism score.

[CLAP](https://github.com/microsoft/CLAP) aligns audio with natural-language
concepts and supports zero-shot classification/retrieval. It can test whether a
clip is closer to “a wooden block struck once” than “a metallic bell,” but that
is semantic fit, not waveform realism. Likewise,
[Audiobox Aesthetics](https://ai.meta.com/research/publications/meta-audiobox-aesthetics-unified-automatic-quality-assessment-for-speech-music-and-sound/)
provides no-reference production-quality/usefulness/enjoyment/complexity
predictions calibrated to broad human ratings; it is an optional artifact guard,
not a physical-fidelity oracle.

### Human calibration remains the ground truth

ITU-R BS.1534-3 standardizes MUSHRA, a multi-stimulus test with hidden reference
and anchors for intermediate audio quality. Source:
[ITU-R BS.1534-3](https://www.itu.int/rec/R-REC-BS.1534-3-201510-I/en).
The Next Engine pilot should be MUSHRA-like rather than claiming standards
compliance: loudness-matched and randomized candidates, a hidden real reference,
the current P0 as a poor anchor and separate ratings for realism, material,
impact-force plausibility and objectionable artifacts. A paired A/B preference
is cheaper for active learning; real-versus-synthetic identification is a useful
held-out check because it directly matches the impact-synthesis literature.

The product owner need not score every generation. One seed session and later
uncertainty-triggered audits can teach a small pairwise ranker. The number of
labels is chosen after measuring label consistency; inventing a universal count
or correlation threshold before the pilot would be false precision.

### Codex needs a tool-readable ear, not an unsupported assumption

Official OpenAI documentation for GPT-5-Codex lists audio input as unsupported,
while the separate GPT-Audio model accepts audio inputs. Sources:
[GPT-5-Codex](https://developers.openai.com/api/docs/models/gpt-5-codex) and
[GPT-Audio](https://developers.openai.com/api/docs/models/gpt-audio).
The reliable design is therefore for Codex to invoke the local evaluator and
reason over exact reports. A separate audio model may add qualitative tags or
pairwise criticism only after its repeatability and agreement with held-out
human labels are measured. It must not be the only score, silently require a
network/provider, or decide runtime behavior.

## Proposed `PhysicalSoundQualityOracleP0`

This is an external experiment shape, not a public engine schema.

### Frozen inputs

- reference-corpus manifest: clip hash, object/material/geometry identity,
  impact location, force trace or force band, microphone pose, acquisition gain,
  room/take identity, license/provenance and split;
- candidate manifest: synth commit, exact parameter/model hash, excitation,
  sample rate, render command and PCM hash;
- evaluator profile: preprocessing, loudness policy, onset alignment, feature
  definitions, embedding/model revisions, metric versions and learned-ranker
  hash;
- immutable `train`, `calibration` and leave-one-object/position-out `holdout`
  assignments chosen before optimization.

Raw references, model weights, embeddings and generated candidate batches stay
outside the repository. Only a tiny explicitly licensed fixture and bounded
human-readable evidence may later be admitted through ordinary content policy.

### Score layers

| Layer | Measurements | Role |
| --- | --- | --- |
| Signal validity | PCM shape/rate, onset window, silence, DC, clipping, nonfinite, peak/RMS/crest, deterministic repeat | Hard reject only |
| Matched response | multi-resolution log-spectrum/log-mel distance, modal peak assignment in log-frequency, per-band decay slopes/T20, attack and temporal centroid, spectral-centroid trajectory, bandwidth/roughness/flatness | Diagnose object response against the exact reference condition |
| Relational physics | force-to-energy ordering, modal-frequency stability across force, participation change across impact location, take variance, material/geometry separability | Hard or Pareto constraints; catches metric-improving but physically incoherent candidates |
| Learned representation | matched-reference PANN/CLAP distance, material/action retrieval, equal-count bootstrapped FAD on larger sets | Secondary perceptual/semantic evidence |
| Broad artifact critic | optional Audiobox Aesthetics and optional audio-language tags | Non-gating warning until locally calibrated |
| Human preference | randomized pairwise/MUSHRA-like ratings and real-versus-synth holdout | Calibration and final audit authority |
| Product cost | modes, model bytes, render time, active-voice and whole-mixer cost when applicable | Separate Pareto axis; quality cannot hide unbounded cost |

No raw weighted sum is accepted initially. Signal failures reject. Remaining
candidates form a Pareto set across matched fidelity, control coherence, human-
calibrated preference and cost. A learned scalar rank may be added only after it
beats every single-metric baseline on a held-out object/position split and
reports calibrated uncertainty.

### Machine-readable output

One candidate report should contain:

- exact input/evaluator/candidate hashes and split identity;
- every raw submetric plus direction and calibration range;
- per-condition and aggregate bootstrap intervals;
- detected failure tags such as `EXCESSIVE_NARROW_RINGING`,
  `DECAY_TOO_LONG_HIGH_BANDS`, `MATERIAL_CLUSTER_COLLAPSE`,
  `FORCE_ORDER_INVERTED` or `POSITION_RESPONSE_MISSING`;
- nearest/better/worse reference and candidate IDs;
- evaluator disagreement and out-of-distribution flags;
- a decision of `Reject`, `ParetoCandidate`, `NeedsHumanAudit` or
  `HoldoutImprovement`, never an unconditional production promotion.

Codex can read that report, inspect spectral/envelope plots and listen through a
human-facing comparison file, but candidate retention is reproducible from the
report rather than a prose impression.

## Autonomous iteration protocol

### Q0 — freeze the failed baseline

Keep the existing nine WAVs, comparison digest and parameter manifests as the
negative baseline. Add no synth change. Measure all signal and descriptor layers
to prove that the evaluator can reproduce its own reports.

### Q1 — obtain matched references and anchors

Use a reviewed external RealImpact subset or record three engine-owned objects
under fixed source/microphone conditions. Preserve raw gain for force/dynamics
tests and create a separate loudness-matched view for timbre judgments. Add a
high-quality modal/offline reconstruction if available and deliberately degraded
anchors that isolate missing modes, wrong damping and excessive strike noise.

### Q2 — calibrate the oracle

Render a diverse bounded parameter batch around and beyond the current presets.
The product owner performs a randomized seed comparison without seeing parameter
names. Fit a small regularized pairwise ranker over interpretable descriptors
plus frozen embeddings. Measure leave-one-object or leave-one-position-out
agreement, uncertainty and inversions against each component metric. If the
ensemble does not generalize better than simple baselines, it is not yet an
oracle and autonomous tuning does not start.

### Q3 — bounded search

Use a derivative-free optimizer over a declared small parameter space: modal
frequency correction, per-band damping, participation gains, strike-envelope/
noise balance and radiation gain. CMA-ES, Bayesian optimization or a stratified
candidate search are all viable; the first implementation should select the
simplest method supported by the measured dimension and evaluation cost. Render
fixed-size batches, retain the Pareto frontier, and change the search region only
from held-out evidence. Never optimize contact-proxy heuristics and acoustic
parameters simultaneously.

### Q4 — audit and stop

Ask for another human comparison only for close frontier candidates, evaluator
disagreement, high uncertainty or a new object/material domain. Stop and reject
the current model family if two calibrated cycles improve training metrics but
not held-out preference, if simple control relations regress, or if no bounded
quality/cost point beats the current baseline. A successful P0 quality oracle
still does not promote SPEC-45 or authorize production contact schemas.

## Anti-gaming controls

- Never train and report on the same object/impact-position closure.
- Freeze preprocessing and reference hashes before candidate search.
- Keep raw dynamics and loudness-normalized timbre views separate.
- Report per-material/per-force results; an aggregate cannot hide one collapsed
  profile.
- Require improvement under at least one matched descriptor and the calibrated
  preference model without violating relational controls.
- Keep deliberately bad anchors and silent/clipped/noisy adversarial controls.
- Treat metric disagreement as evidence, not average it away.
- Do not let an LLM explanation repair or override a numeric/human failure.

## Smallest recommended next implementation

Implement evaluator Q0/Q1 before changing the synthesizer:

1. external corpus/manifest and exact onset/loudness preprocessing;
2. deterministic signal, modal-peak, decay/envelope and spectral descriptor
   report for the current nine WAVs and matched references;
3. a static randomized browser or file bundle for paired human labels;
4. held-out split and report comparison command;
5. only then add frozen PANN/CLAP or aesthetic-model adapters if they improve
   agreement enough to justify their weight and dependency cost.

This gives Codex a falsifiable ear with a classical offline fallback. It avoids
committing model weights or datasets, adds no runtime dependency and prevents
another round of tuning to whichever artifact happens to sound less bad once.

## Local implementation checkpoint

`cargo run -p xtask -- physical-sound-eval --manifest <external-json>
--output <external-empty-directory>` now implements the classical Q0/Q1 cut:

- the physical-sound laboratory emits a sorted external manifest with exact WAV
  hashes and the nine failed-baseline condition identities;
- the evaluator accepts bounded PCM 8/16/24/32-bit and IEEE float32 WAV,
  downmixes deterministically, aligns onset and reports hard signal checks,
  multiresolution gain-matched log spectra, modal-peak assignment, attack,
  temporal/spectral centroid, bandwidth/flatness and per-band T20;
- matched entries report raw level separately from timbre/decay distances and
  retain `NeedsHumanAudit`; no uncalibrated weighted quality scalar exists;
- a fixed seed produces renamed A/B WAVs, a static local rating browser and a
  separate hidden answer key;
- external paths, hashes, entry order, counts, sizes and durations fail closed;
  raw corpora and generated reports remain outside the repository.

The first Q0 run analyzed all nine baseline WAVs. A repeated two-pair Q1
control report was byte-identical: self-match returned zero for every distance,
while steel-center against glass-corner produced `28.7939 dB` multiresolution
spectrum RMSE, modal cost `0.642964` and spectral/modal/high-band-decay
diagnostics. This proves determinism and basic sensitivity only. No reviewed
real reference, human-preference calibration, relational-physics gate,
embedding adapter or held-out ranker has run, so autonomous tuning remains
blocked at Q2.

## Steel reference-screen checkpoint

A subsequent bounded P0 cycle used seven hash-frozen CC0 metal-impact WAVs as
a broad timbral screen, not as a controlled matched corpus. The selected
12-mode candidate reduced median multiresolution spectrum RMSE from
`44.5951 dB` to `21.9281 dB` and median absolute mean-T20 delta from
`6606.82 ms` to `49.17 ms`. Median modal cost worsened from `1.04514` to
`1.36658`, consistent with the pack containing several different bodies rather
than one plate identity. The exact provenance, hashes, candidates and known
edge-decay diagnostic are recorded in the
[steel calibration report](physical-sound-steel-calibration-2026-08-26.md).

This result permits the selected candidate to replace the failed steel preset
inside the off-by-default laboratory/demo. It does not calibrate the evaluator,
establish autonomous quality ranking, or authorize a generic steel/P1 claim.
The next discriminator remains a blind product-owner audition, followed by a
controlled small object corpus if the candidate is worth pursuing.

## Decision

Treat the current P0 audition as `PERCEPTUAL_FAIL`. Preserve modal impact as a
candidate model family, but block further quality claims and blind coefficient
tuning. The next coherent work package is an external, human-calibrated
`PhysicalSoundQualityOracleP0`; its success means reliable held-out ranking of
candidate sounds, not production promotion.
