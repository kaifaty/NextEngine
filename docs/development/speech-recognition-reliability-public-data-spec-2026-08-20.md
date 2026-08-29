# Speech recognition reliability from public data — development specification

| Field | Value |
| --- | --- |
| ID | `DEV-SPEECH-RELIABILITY-001` |
| Status | `Proposed experiment` |
| Date | `2026-08-20` |
| Scope | `tools/speech-timeline`; utterance-level Russian ASR reliability |
| Primary route | Pinned Voxtral Realtime profile; one calibrator per exact ASR/audio route |
| Data constraint | No first-party voice-recording campaign and no default player-voice collection |
| Authority | Development specification only; not an Accepted architecture SPEC or a `crates/contracts` contract |

## 1. Decision and claim boundary

Build a small, offline-trained `RecognitionReliabilityEstimator` from public
Russian speech corpora, deterministic audio degradations and the real
`SpeechTimelineService` revision trace. The estimator predicts the risk that
the finalized, normalized ASR transcript differs from a known reference.

The estimator does **not** measure intelligence, accent quality, speech health,
pronunciation skill or a person's ability to speak clearly. Its user-facing
meaning is only:

> The current speech pipeline is or is not sufficiently confident that it
> recovered this utterance correctly.

The stable name is `recognition_reliability`, not `diction_score`,
`pronunciation_score` or `speech_defect`. A clean recording can still contain
an ASR error, and a noisy or quiet recording can still be transcribed exactly.

The first candidate is generic-public calibration. It MUST advertise
`calibration_domain = generic_public_ru_v0` and MUST NOT claim calibration for
all player microphones. Unknown or out-of-distribution input widens the
uncertainty result instead of producing confident acceptance.

This proposal remains below the Deferred Proposed dialogue/model-pack boundary
in [SPEC-16](../architecture/16-text-canonical-multimodal-dialogue-and-model-packs.md)
and [ADR-017](../architecture/adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md).
Accepted [ADR-005](../architecture/adr/005-offline-first-ai-process-boundary.md)
continues to own isolation, untrusted output and offline fallback semantics.

## 2. Goals

1. Produce an utterance-level calibrated `p_exact` for an exact pinned ASR,
   runtime, delay and preprocessing route.
2. Distinguish `clear`, `uncertain`, `unheard`, `no_speech` and
   `technical_failure` without asking an LLM to judge plausibility.
3. Use evidence already available or cheaply collectable in the live path:
   transcript revisions, stable prefix, final text, VAD, signal diagnostics
   and terminal status.
4. Add no second ASR pass, neural quality model or GPU work to the live route.
5. Make corpus acquisition, augmentation, replay, training and evaluation
   reproducible from exact manifests while keeping all heavy data outside Git.
6. Permit a deterministic NPC policy to ask for repetition or confirmation;
   generated wording may vary, but the semantic action does not.

## 3. Non-goals

- Fine-tuning Voxtral, GigaAM or another ASR.
- Word-level confidence or uncertain word spans. Current Voxtral events expose
  utterance text revisions but no verified lexical timestamps or calibrated
  token probabilities.
- Forced-alignment or Goodness of Pronunciation for arbitrary dialogue.
- Diagnosing articulation, accent, disability, intoxication or medical state.
- Treating RMS, SNR proxy, DNSMOS, a denoiser score or VAD probability as the
  sole quality oracle.
- Runtime learning, weight mutation, silent telemetry collection or upload of
  player microphone audio.
- Making partial ASR authoritative or allowing it to commit dialogue/gameplay
  state.
- Selecting a shipping model or changing Accepted engine architecture.

## 4. Output taxonomy

| Result | Meaning | Required default behavior |
| --- | --- | --- |
| `clear` | Final transcript passed the calibrated selective-risk threshold and is not OOD | Admit only as finalized ASR text to the existing bounded dialogue path |
| `uncertain` | Speech and text exist, but exactness evidence is insufficient | Ask for confirmation or repetition |
| `unheard` | Speech evidence exists, but final ASR text is empty or coverage is unusable | Ask the player to repeat; do not create an empty dialogue turn |
| `no_speech` | No admitted speech evidence | End capture without a player utterance |
| `technical_failure` | Scheduler, adapter, protocol, model or feature extraction failed | Use the declared text/input fallback and report a technical reason |

`clear` is not permission for direct gameplay mutation. It only allows the
final transcript to proceed through the same canonical-text and validation
path as typed input. The result is a non-authoritative annotation.

## 5. Dataset profiles

Two profiles MUST remain separate. Their manifests, derived runs and trained
artifacts cannot be merged implicitly.

### 5.1 `public-safe-v0`

Candidate profile for an artifact that may later undergo normal redistribution
review. Every source still requires exact revision/hash and a repository
license decision before artifact publication.

| Source | Intended use | Published terms | Admission |
| --- | --- | --- | --- |
| [Common Voice Scripted Speech 26.0 — Russian](https://mozilladatacollective.com/datasets/cmqinj9g500vsnr07qf4hmr3j) | Main multi-speaker train/calibration source | CC0; source also forbids re-identification and re-hosting | Allowed locally; never republish raw clips |
| [Common Voice Spontaneous Speech 4.0 — Russian](https://mozilladatacollective.com/datasets/cmqi2c2eu0062o5075atr17rs) | Source-disjoint conversational held-out suite | CC0; same privacy/re-hosting restrictions | Evaluation-first; no train leakage |
| [Google FLEURS](https://huggingface.co/datasets/google/fleurs) `ru_ru` | Independent read-speech held-out suite | CC BY 4.0 | Evaluation-first with attribution closure |
| [MUSAN / OpenSLR 17](https://www.openslr.org/17/) | Additive noise, music and competing speech | CC BY 4.0 | Augmentation source with notice closure |
| [RIRS_NOISES / OpenSLR 28](https://www.openslr.org/28/) | Real/simulated room impulse responses and noise | Apache-2.0 | Augmentation source |

The default candidate MUST be trainable without Golos, Dialogs RU or Russian
LibriSpeech. This preserves a bounded fallback when their terms are not
admitted for a distributable artifact.

### 5.2 `whisper-research-v0`

Adds [Dialogs RU](https://huggingface.co/datasets/langswap/dialogs-ru-emotional-conversations)
for genuine Russian `whisper` and expressive/quiet speech. Lowering the gain of
ordinary voiced speech is not a substitute for whisper because it does not
reproduce whisper acoustics.

Dialogs RU uses OpenRAIL terms with use restrictions. Under
[SPEC-11](../architecture/11-security-licensing-and-governance.md), this profile
is research-only until an explicit license/provenance review admits both the
dataset use and the derived artifact. A model trained with this profile MUST be
marked `redistribution_status = blocked_pending_review` and MUST NOT replace a
public-safe artifact.

### 5.3 Excluded by default

| Source | Reason |
| --- | --- |
| [Golos](https://github.com/salute-developers/golos/tree/master/golos) | Custom attribution/share-alike-like license; exclude until derived-model and redistribution consequences are reviewed |
| [Russian LibriSpeech / OpenSLR 96](https://www.openslr.org/96/) | Described as public domain in the USA; exclude from the default global shipping closure until jurisdictional reuse is reviewed |
| Web/video/podcast scraping | Transcript, speaker consent, source rights and redistribution closure are not reliable enough for this experiment |
| Existing player microphone turns | No silent future-training consent; diagnostic retention is not training consent |

## 6. External storage and provenance

Dataset audio, derived audio, ASR traces, feature matrices, checkpoints and
reports MUST remain outside the repository and shipping package. A tool MUST
receive an explicit external `--store` directory; no machine-local absolute
path enters a committed manifest.

The repository MAY contain this specification, schemas, acquisition scripts,
normalizers, feature code and tiny generated/CC0 fixtures.

The experiment-local manifest is not a public engine contract:

```text
SpeechReliabilityDatasetManifestV0 {
  schema_version,
  dataset_id,
  dataset_revision,
  calibration_domain,
  source_entries[],        // source URL, exact release/revision, hashes
  rights_entries[],        // license class, notices, restrictions, status
  sample_index_root,
  speaker_partition_root,
  split_assignment_root,
  text_normalizer_profile,
  augmentation_profile_hash,
  replay_profile_hash,
  feature_schema_hash,
  label_schema_hash,
  manifest_hash,
}
```

Unknown terms, missing hashes, duplicate clip identity or a source/speaker
split conflict excludes the affected data before replay or training. It does
not downgrade to a warning.

## 7. Split and leakage rules

1. Assign `train`, `calibration` and `held_out` before creating any
   degradation.
2. Split by the best available stable speaker identity and by source corpus.
3. Every derivative of one clean clip remains in the same split.
4. Noise and RIR recordings also use disjoint train/held-out partitions.
5. Identical normalized text MAY occur across splits only when the audio and
   speaker identities differ; duplicate audio content is forbidden.
6. Common Voice Spontaneous and FLEURS Russian are held out from the first
   training run to expose source/domain shift.
7. Whisper train/dev/test membership follows the source split and is reported
   only inside `whisper-research-v0`.
8. Split membership and every random selection are derived from a fixed seed
   and stored by content hash, never from filesystem or worker order.

## 8. Text normalization and labels

`ru-asr-normalize-v0` is the only scoring normalizer for the first run:

1. validate UTF-8 and convert to NFC;
2. locale-aware lowercase for Russian;
3. map `ё` to `е`;
4. replace punctuation and non-letter separators with one space;
5. collapse whitespace and trim;
6. exclude empty references and references containing digits from V0 rather
   than introducing an implicit number verbalizer.

The current dashboard WER/CER display is diagnostic and MUST NOT become the
training oracle until it uses the same versioned normalizer.

For each final ASR result compute:

- `wer` with deterministic Levenshtein alignment over normalized words;
- `cer` over normalized non-space Unicode scalars;
- `exact_match = normalized_reference == normalized_final`;
- insertion, deletion and substitution counts;
- `empty_final` and `speech_detected_but_empty`.

The first learned target is `exact_match`. WER/CER remain reported secondary
targets and error-analysis metrics. `no_speech`, `unheard` and
`technical_failure` are typed deterministic outcomes, not labels guessed by
the classifier.

## 9. Deterministic degradation generation

Each admitted clean clip receives the unchanged control plus at most five
derived conditions sampled from a manifest-fixed grid:

- level attenuation without calling it whisper;
- additive MUSAN noise at declared SNR values;
- convolution with a declared RIRS_NOISES impulse response;
- combined noise plus room response;
- bounded microphone-band/EQ coloration;
- bounded compression, clipping or PCM-frame loss.

All derived audio is converted to 16 kHz mono PCM S16LE using one pinned
resampler profile. A derivation record stores clean-clip hash, noise/RIR hash,
seed, transform order and exact parameters. Augmentation MUST NOT modify the
known transcript.

The same derived RAW PCM is replayed separately through each candidate ASR
preprocessing route. RAW, gain, DPDFNet, GTCRN and UL-UNAS results are paired by
source/derivation ID; their samples are never generated independently.

## 10. Replay profile and trace collection

V0 targets the real resident service, not an offline mock:

- 16 kHz mono PCM S16LE;
- paced 80 ms ingress;
- explicit finish;
- pinned ASR model/runtime/revision;
- pinned model delay and partial-decode interval;
- selected `asr_audio_route` locked for the turn;
- diagnostic retention disabled for corpus replay;
- exact ready/capability payload and terminal lineage retained in the run
  manifest.

One reliability artifact binds exactly:

```text
(asr_adapter_id, model_id, model_revision, runtime_revision,
 model_delay, partial_decode_interval, asr_audio_route,
 feature_schema, normalizer_profile)
```

A change to any bound identity requires a new dataset replay and candidate
artifact. Scores from Voxtral, GigaAM, GigaSTT or another route are not pooled
into one calibrator. Final-only adapters require their own feature schema.

The trace collector records every transcript revision as bounded metadata:

```text
TranscriptRevisionTraceV0 {
  revision,
  observed_audio_end_sample,
  service_elapsed_ms,
  text,
  stable_prefix,
  final,
}
```

`service_elapsed_ms` is diagnostic wall time, not acoustic word time. No word
span is inferred from revision arrival.

## 11. V0 feature schema

All features MUST be computable in one streaming pass without retaining the
whole waveform for the estimator.

### Transcript-revision features

- revision count before final;
- time/audio duration to first non-empty text;
- time/audio duration to last text change;
- normalized Levenshtein distance between consecutive revision texts;
- cumulative and maximum revision churn;
- stable-prefix length divided by current/final normalized text length;
- final-to-previous-revision edit distance;
- final word/character count and speech duration;
- empty partial/final flags.

### Acoustic/front-end features

- calibrated noise floor when available and its source;
- VAD speech sample count, speech ratio and segment count;
- raw and selected-ASR RMS/peak dBFS;
- RMS/peak delta between raw and selected route;
- non-zero sample ratio;
- clipping ratio and dropped/discontinuous-frame count;
- preprocessing route, active/bypass state and declared algorithmic delay.

### Runtime validity features

- terminal completion versus typed error;
- scheduler overload/rejection and ASR job failure count;
- ingress sample/frame counts and sample-clock consistency;
- feature completeness and finite-value validation.

V0 MUST NOT use transcript semantic plausibility, an LLM judgement, vocabulary
rarity, sentiment, emotion label, speaker demographic field or device serial
identity. Those inputs can create circular confidence or unjustified bias.

Offline-only diagnostics such as DNSMOS, a second ASR, RAW-versus-processed
transcript agreement or GigaAM reference output MAY be reported in analysis,
but MUST NOT become required runtime features for the V0 artifact.

## 12. Baseline estimator and artifact

The first learned candidate is deliberately small:

1. finite-value validation and feature-schema check;
2. fixed training-set mean/scale normalization;
3. L2-regularized logistic regression for `exact_match`;
4. optional isotonic calibration fitted only on the disjoint calibration
   split when it improves both Brier score and calibration error there;
5. a manifest-fixed threshold selected only from the calibration split by the
   declared selective-risk rule;
6. robust feature quantiles for an OOD guard.

The candidate MUST be compared with:

- a constant base-rate predictor;
- deterministic `speech_detected_but_empty` handling;
- a simple revision-stability rule with no learned weights.

A more complex GBDT/neural estimator is authorized only after the logistic
baseline passes end-to-end plumbing and a pre-registered equal-data comparison
shows a material held-out calibration/coverage gain. Model size alone is not a
promotion signal.

The exported candidate contains only bounded coefficients, feature
normalization constants, optional monotonic calibration knots, OOD bounds,
thresholds and exact lineage. It requires no Python training framework or GPU
at runtime. Training and intermediate checkpoints remain external.

## 13. OOD and automatic microphone calibration

The public-only constraint makes OOD handling mandatory.

- The artifact stores robust train/calibration quantiles for every critical
  acoustic and revision feature.
- Missing required features, non-finite values or manifest-defined excessive
  quantile violations force `uncertain`/`technical_failure`; they can never
  produce `clear`.
- OOD parameters are fixed in the artifact manifest and evaluated on held-out
  corpora; runtime does not tune them from accepted/rejected player text.

The existing explicit quiet-room calibration MAY provide a 0.5–10 s local
noise-floor observation. A recommended UI capture is 2 s. This value may
normalize signal features and VAD/gain gates, but it does not retrain weights
and is not evidence that transcript text is correct.

Session-local signal quantiles MAY update from ordinary turns only as bounded
non-persisted diagnostics. They MUST NOT alter model weights, create training
data or make a failed turn become `clear` by self-confirmation.

## 14. Prototype result schema

This is an internal `tools/speech-timeline` schema, not yet a public engine
contract:

```text
RecognitionReliabilityResultV0 {
  schema_version: 0,
  transcript_revision: u64,
  scope: Utterance,
  final: bool,
  decision: Clear | Uncertain | Unheard | NoSpeech | TechnicalFailure,
  p_exact: Option<float>,
  probability_kind: "calibrated_normalized_exact_match" | "unavailable",
  calibration_domain: "generic_public_ru_v0" | "whisper_research_ru_v0",
  estimator_artifact_hash: Option<Hash256>,
  asr_model_id,
  asr_model_revision,
  asr_audio_route,
  feature_schema_hash,
  out_of_distribution: bool,
  reason_codes: BoundedList<RecognitionReliabilityReasonV0>,
}
```

`p_exact` is present only for a complete compatible feature vector. It is
bounded to `[0, 1]`, finite and never described as calibrated outside the
declared domain. Partial results, if exposed to the UI, set `final = false` and
remain provisional; only the final result can be used by clarification policy.

Reason codes are closed and bounded, initially:

- `RELIABILITY_CLEAR_THRESHOLD_MET`;
- `RELIABILITY_REVISION_UNSTABLE`;
- `RELIABILITY_SPEECH_WITH_EMPTY_FINAL`;
- `RELIABILITY_NO_SPEECH_EVIDENCE`;
- `RELIABILITY_SIGNAL_OUT_OF_DISTRIBUTION`;
- `RELIABILITY_FEATURE_INCOMPLETE`;
- `RELIABILITY_PROFILE_MISMATCH`;
- `RELIABILITY_ASR_TECHNICAL_FAILURE`.

## 15. Dialogue/NPC integration boundary

The first implementation stops at service/UI output. A later consumer may map
the final result through a deterministic policy:

```text
Clear       -> AcceptFinalTranscript
Uncertain   -> ConfirmTranscript or RequestRepeat
Unheard     -> RequestRepeat
NoSpeech    -> NoTurn
Technical   -> TextInputFallback or AuthoredTechnicalResponse
```

The policy chooses the semantic action. An LLM may render that already-selected
action in the NPC's authored tone, but it cannot override `uncertain`, invent
what was said or decide that the player has a speech defect. A neutral
accessible wording remains available even when a character pack provides a
rude or humorous variant.

Partials remain limited to cancelable immutable prewarm as specified by the
current speech task state. Gameplay, tool calls, canonical subtitles and TTS
commitment wait for final admission and their normal validators.

## 16. Evaluation protocol

Every report MUST identify corpus releases/hashes, split roots, augmentation
profile, ASR/runtime/audio-route identity, hardware, feature schema, estimator
artifact and threshold-selection rule.

Report at least:

- exact-match rate, WER and CER;
- Brier score, negative log loss and expected calibration error;
- AUROC and precision/recall for transcript error;
- clear coverage and error rate among `clear` results;
- Wilson 95% upper confidence bound for false-clear rate;
- empty-rate and `speech_detected_but_empty` rate;
- revision churn and time to stable/final text;
- results by corpus, clean/degradation bucket, duration band and audio route;
- OOD rate and the number of OOD samples incorrectly marked `clear`;
- scoring CPU latency and memory overhead.

First candidate gates:

| Gate | Required result |
| --- | --- |
| Data closure | Exact source/revision/hash/rights/split/augmentation/replay manifests; zero cross-split derivative leakage |
| Calibration | Overall ECE ≤ 0.05 and every named bucket with ≥100 samples ECE ≤ 0.10 |
| Selective safety | Overall Wilson 95% upper bound for false-clear ≤ 5%; per named bucket with ≥100 samples ≤ 10% |
| Utility | `clear` coverage ≥30% overall and ≥50% on the clean held-out bucket; otherwise report-only |
| OOD | Zero OOD examples classified `clear` by construction |
| Baseline | Brier score improves by at least 10% relative to the constant-prior predictor |
| Runtime cost | Estimator p95 ≤2 ms on the active Linux CPU, ≤16 MiB resident memory, no extra GPU/ASR invocation |
| Failure behavior | Missing/mismatched/non-finite features fail closed to typed uncertainty/failure |

These are development gates, not Accepted ProductChecks. Failure leaves the
existing finalized transcript plus deterministic clarification/text fallback;
it does not block the offline game or weaken input validation.

`whisper-research-v0` is always reported separately. It cannot establish
shipping admission and cannot hide regression in `public-safe-v0`.

## 17. Failure semantics

| Failure | Required outcome |
| --- | --- |
| Dataset/source hash mismatch | Reject the affected source before feature generation |
| Unknown/incompatible rights or missing notice | Exclude the source and every derived artifact |
| Split leakage or duplicate audio across prohibited splits | Invalidate the complete dataset revision |
| Replay profile/model mismatch | Reject the trace; never score it with another route's estimator |
| Missing/non-finite/out-of-range feature | `uncertain` or `technical_failure`, never `clear` |
| Estimator artifact absent/corrupt/incompatible | Typed failure and existing text-input/authored fallback |
| ASR timeout/OOM/scheduler/protocol failure | `technical_failure`; do not reinterpret as poor diction |
| VAD says speech and final text is empty | `unheard`; do not call the learned classifier |
| No speech evidence | `no_speech`; do not create a canonical empty utterance |
| Runtime score differs on replay | Retain recorded normalized result; report `NONDETERMINISTIC_RESULT`, do not rerun to green |

## 18. Privacy and accessibility

- Player microphone data is not training data by default.
- Diagnostic retention and corpus ingestion are separate explicit
  capabilities and consents.
- Raw capture remains non-retained unless the user explicitly enables the
  bounded local diagnostic archive.
- No speaker re-identification, demographic inference or device fingerprint is
  part of the estimator.
- Default logs contain hashes, bounded metrics and reason codes, not audio or
  transcript content.
- Thresholds MUST be evaluated for false rejection as well as false acceptance
  across available speakers and conditions. Higher uncertainty may change NPC
  wording, but never penalizes gameplay skill or progression by default.

## 19. Implementation increments

### R0 — Data closure and runner

- Implement exact source/rights/split/augmentation/replay manifests.
- Add a downloader/importer that writes only to an explicit external store.
- Add `ru-asr-normalize-v0` and deterministic WER/CER labels.
- Produce a small manifest-only dry run before downloading the full subset.

### R1 — Revision feature capture

- Accumulate bounded transcript-revision statistics in the service/benchmark
  path without serializing unbounded text history into terminal events.
- Add clipping/discontinuity metrics and feature-schema validation.
- Replay identical PCM through one exact ASR/audio route at a time.

### R2 — Train, calibrate and evaluate

- Fit constant/rule/logistic baselines on frozen train/calibration splits.
- Export the small immutable candidate and its complete lineage.
- Run source-disjoint and degradation-disjoint held-out evaluation.
- Keep `public-safe-v0` and `whisper-research-v0` reports/artifacts separate.

### R3 — Resident scoring and Vue diagnostics

- Load the estimator once with the resident speech service.
- Emit provisional/final reliability results with typed reasons and OOD state.
- Show probability kind/domain, decision, reason and calibration limits; never
  label the player as speaking badly.
- Prove scoring adds no ASR call and remains within the CPU/memory bound.

NPC dialogue behavior is a later consumer and is not required to close R0–R3.

## 20. Definition of done

This experiment is complete when:

1. a hash-closed public-only corpus subset can be reproduced outside Git;
2. the real Voxtral streaming trace is collected without unbounded event or
   audio retention;
3. the V0 logistic artifact and its baselines have source-disjoint held-out
   reports;
4. the public-safe candidate either passes every gate or remains explicitly
   report-only with the first failing gate named;
5. the whisper candidate remains separately licensed and classified;
6. the resident service/UI exposes honest utterance-level reliability with
   typed failure/OOD behavior and deterministic fallback;
7. no result is presented as word timing, pronunciation quality or gameplay
   authority.

## 21. Reconsideration conditions

Revisit this design when any of the following becomes true:

- the selected ASR exposes calibrated token/word confidence and verified
  lexical timestamps;
- an admissible multi-speaker Russian whisper corpus becomes available;
- a future opt-in player-data program supplies target-domain evidence;
- public-only held-out coverage is too low to be useful even at the allowed
  false-clear risk;
- a simpler revision rule matches the learned estimator, making the model
  unnecessary;
- a replacement ASR changes the available feature/capability envelope.

Until then, the cheapest safe fallback is finalized ASR plus a conservative
`uncertain` result and an authored request to repeat or use text input.
