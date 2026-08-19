# Emotion timeline quality research — 2026-08-18

## Scope and decision

This note concerns only the optional Phase 1 vocal-expression diagnostic.
It neither promotes SPEC-16/ADR-017 nor changes the later LLM, TTS or
FunctionGemma scope.

**Decision:** the service now exposes the pinned Russian WavLM candidate behind
the replaceable affect facade. It passes the frozen RESD quality gate and the
one-process Voxtral+WavLM screen below, but is still a local diagnostic profile
rather than an unqualified shipped default. Keep `emotion2vec_plus_base` as
the rollback profile; do not choose a model from a model-card headline or one
subjective microphone recording.

## Observed failure and correction

The dashboard may show a last raw model window with a high `other` score while
the smoothed timeline shows `unknown`. This is expected during a one-hop label
change: the live policy requires two hops before admitting a new expression.
The error was using that *current, provisional* state as the final utterance
summary. It let a final silence/transition erase earlier confirmed speech.

The final event now selects the duration-weighted dominant label across
admitted primary-expression segments only. `unknown` and model bucket `other`
remain visible in the raw and timeline diagnostics but cannot overwrite a
confirmed final expression. If there is no such evidence, the final result
honestly abstains as `unknown`. Overlapping raw windows contribute only through
their non-overlapping smoothed segments.

## Noise and silence policy

- A user explicitly presses **"Калибровать тишину"** before a recording. The
  browser listens for two seconds, transmits no calibration audio and retains
  only bounded RMS statistics.
- It estimates the 90th-percentile background level and rejects a calibration
  containing large energy variation, which normally means speech or a transient
  noise occurred during the sample.
- The local service validates the numeric result and raises only the energy-VAD
  speech/silence thresholds. Emotion2Vec still receives the original signal:
  denoising, AGC and level normalization can remove the loudness and spectral
  cues that carry prosody.
- The calibration is bound to the selected input device and is discarded if the
  device changes. It is per-browser-session diagnostic state, not a profile or
  a stored biometric record.

This improves no-speech rejection but is not a claim that RMS VAD is robust to
all noise types. A neural VAD remains a separately measurable replacement
behind `VoiceActivityDetector`.

## Programmatic acceptance measurements

The final event now exposes content-free diagnostic counters: raw observation
count, confirmed segment count and samples, `unknown` share, label-transition
churn and per-label final support. They reveal the failure mode without saving
audio or transcript.

Before a model swap, prepare an external, permission-cleared evaluation manifest
with only hashed local audio paths and human labels. Include at least quiet
silence, fan/keyboard/background speech without the target speaker, neutral
speech, deliberate basic expressions and within-turn transitions from several
speakers. Report:

| Question | Metric |
| --- | --- |
| Does silence leak into emotion inference? | VAD false-positive rate and emotion-observation count per no-speech minute |
| Does VAD lose actual speech? | speech recall and affect coverage of labelled speech |
| Is the final tag correct? | macro-F1 and balanced accuracy on a declared common label map |
| Is the emotion timeline usable? | segment overlap/boundary error plus transitions per voiced minute |
| Is the result stable and timely? | final-label flip rate, first affect latency, inference p50/p95, RTF and peak VRAM |

Use the same clips, windows, VAD decisions, label map and pinned model revisions
for every candidate. Score `other`/`unknown` as abstentions, not as successful
basic-emotion classifications. A candidate must improve the agreed quality
measure without violating the resident latency/VRAM envelope; otherwise retain
the base model.

## First held-out Russian calibration screen

`Aniemore/resd` is now used only as an external, held-out calibration artifact,
not as a training input. The external manifest
`/home/kaifaty/.cache/nextengine/emotion-calibration/resd-70-v2/manifest.json`
selects ten deterministic, actor-stratified test clips for each of the seven
RESD labels. It contains 70 normalised 16 kHz mono WAV files (6 m 13 s total),
the source revision `8db7068a7717e48d829c2baa32e4908972611138`, file hashes,
an explicit label map, and a minimum duration of one second. The dataset's card
declares MIT; raw files and the manifest remain outside Git. The prior v1
selection contained one 720 ms neutral clip and is retained only as a
diagnostic artifact, not the comparison baseline.

The pinned `emotion2vec_plus_base` checkpoint was loaded once on the local RTX
3080 and measured against whole normalised v2 utterances. Of the 60 clips with
an exact class mapping, top-1 matched 29 (`48.3%`): angry `7/10`, disgusted
`5/10`, fearful `6/10`, happy `3/10`, neutral `4/10`, sad `4/10`. The ten RESD
`enthusiasm` clips are intentionally unscored because the checkpoint exposes no
equivalent label; it predicted `other` for six of them. Mean model-only
inference was 28.0 ms per clip after load, maximum 321 ms. The complete,
content-free report is
`/home/kaifaty/.cache/nextengine/emotion-calibration/resd-70-v2/emotion2vec-plus-base-report.v1.json`
with SHA-256
`4dc1ef57ceb39c6ec21f4b9507dfaf104e32a4928ad755ae41c4bce954d569de`.

The same v2 clips were then replayed through the public resident WebSocket
path with 80 ms logical frames. All 70 clips were admitted as speech and all
70 yielded at least one affect observation, so this screen provides no evidence
that the current VAD loses voiced RESD material. The final timeline result was
23/60 exact mapped top-1 (`38.3%`), with eight final abstentions. Per mapped
class it was angry `5/10`, disgusted `2/10`, fearful `6/10`, happy `3/10`,
neutral `4/10` and sad `3/10`. Thus the VAD/window/smoothing path reduced this
small-screen score by ten percentage points relative to whole-utterance model
inference; do not attribute that gap to VAD. The external privacy-preserving
report is
`/home/kaifaty/.cache/nextengine/emotion-calibration/resd-70-v2/timeline-base-report.v1.json`
with SHA-256
`726c58c1540547f012c1d8b34cee85a338bcbd1ebd7b9affd62fba44b50e024d`.
It omits PCM and transcript text. Its 3.44 s / 7.45 s p50/p95
finish-to-final times are **not** Live latency: this quality run used unpaced
burst ingress, which deliberately queues all 80 ms ASR frames before each
finish.

This is a small acted-speech screen, not a deployment-quality or probability
calibration claim: it does not exercise natural microphone speech, and the
unpaced path is not a latency test. It does establish that the base checkpoint
must not be treated as a reliable Russian basic-emotion classifier, especially
for happy, sad and disgusted speech; it also separates the current VAD result
from the later temporal-aggregation loss. Do not retune VAD thresholds,
smoothing thresholds or public label semantics to improve this held-out score.
Every subsequent candidate must first use the frozen v2 clips for direct
comparison with base; run its full-path measurement only after it wins that
cheaper gate. Report abstentions separately and only then decide whether a
bounded smoothing-policy experiment is justified.

## `emotion2vec_plus_large` A/B result

The larger official checkpoint was downloaded outside Git at pinned revision
`6c303ba987b86b93193de93e34bb2b077a6bedc4` and loaded successfully beside the
resident base service. It was evaluated directly against exactly the same
RESD-70 v2 manifest and label map as base. It scored 26/60 (`43.3%`) mapped
top-1 versus base's 29/60 (`48.3%`): angry `5/10`, disgusted `3/10`, fearful
`3/10`, happy `6/10`, neutral `4/10`, sad `5/10`. It improves happy and sad but
loses too much on fear, angry and disgusted for the aggregate result to pass.
The external report is
`/home/kaifaty/.cache/nextengine/emotion-calibration/resd-70-v2/emotion2vec-plus-large-report.v1.json`
with SHA-256
`152eaed20f0a7d6f36ae6f9ddbc52c67215099f9d4f79812a7736b6818e02a07`.

**Decision:** retain the base checkpoint and do not run a full resident
timeline/latency profile or swap the browser service to this candidate. The
candidate already fails the cheaper whole-utterance quality gate; a full path
run would interrupt the current resident service while adding no evidence that
the model itself is better. Reconsider only if a declared downstream weighting
values happiness/sadness enough to outweigh aggregate loss, or a new candidate
first beats the base direct score on the frozen manifest.

## `Aniemore/wavlm-emotion-russian-resd` A/B result

The Russian audio-only WavLM candidate was downloaded outside Git at pinned
revision `7a4ca18b34adff59b56b451acc7ff44fc43a12dc`. Only the standard
Transformers `config.json`, `preprocessor_config.json` and `model.safetensors`
were fetched; no remote model code was enabled. The 1,266,099,454-byte weights
have SHA-256
`dabf15d84b451195346b8050102a7243b3f92276064195f6e34b65aaa06a12ab`.

On the frozen RESD-70 v2 clips, with the model card's 16 kHz mono,
per-utterance-normalized and maximum-12-second preprocessing, it scored 45/60
exactly mapped classes (`75.0%`) versus base's 29/60 (`48.3%`). Per mapped
class: angry `8/10`, disgusted `7/10`, fearful `7/10`, happy `8/10`, neutral
`8/10`, sad `7/10`. The candidate also has a native `enthusiasm` class and got
7/10 on the otherwise-unscored RESD enthusiasm clips. The direct report is
`/home/kaifaty/.cache/nextengine/emotion-calibration/resd-70-v2/aniemore-wavlm-russian-resd-direct-report.v1.json`,
SHA-256 `ed66c8e250695daf13805ef61888ed976f1aa35562ce1ab96fa93db1834c3489`.
GPU-synchronized inference averaged 34.8 ms per utterance (maximum 423.8 ms,
including the first invocation); it is not a live service latency claim.

An ephemeral WavLM adapter then exercised the production loopback WebSocket,
PCM framing, energy VAD, 1/2-second affect cadence, scheduler, smoothing and
final aggregation. It scored 44/60 (`73.3%`): angry `8/10`, disgusted `7/10`,
fearful `9/10`, happy `6/10`, neutral `7/10`, sad `7/10`. All 70 clips were
speech-admitted and affect-observed; final `unknown` was returned for one
neutral and one enthusiasm clip because no segment reached the existing
two-hop smoothing admission. The report is
`/home/kaifaty/.cache/nextengine/emotion-calibration/resd-70-v2/aniemore-wavlm-russian-resd-affect-timeline-report.v1.json`,
SHA-256 `8fdfc51f47474c1888b575f2d5c62b9e238a21bb60db071bb065c0106c8e572f`.

The actual adapter is now `transformers-wavlm-russian-ser/1`. It loads only the
pinned local files with stock Transformers and `trust_remote_code=False`,
checks `model.safetensors` against the digest above, and fails preflight if the
snapshot/config/feature extractor is missing or outside the declared cache.
Its seven native labels are normalized to the timeline vocabulary, preserving
`enthusiasm` rather than silently treating it as `happy`.

A real one-process Voxtral+WavLM service then replayed the exact same 70 clips
through public WebSocket framing, VAD, serialized scheduler, ASR and final
smoothing. All 70 were admitted and observed; it scored 45/60 (`75.0%`) on the
six-class common map, with two final abstentions (one neutral, one enthusiasm).
Its readiness report measured 2.6 s Voxtral load + 4.4 s WavLM load and 4,924
MiB resident process VRAM, leaving 3,382 MiB free on the RTX 3080. The
privacy-preserving report is
`/home/kaifaty/.cache/nextengine/emotion-calibration/resd-70-v2/aniemore-wavlm-russian-resd-joint-timeline-report.v1.json`,
SHA-256 `5add6a8be7b10c18245f6c24a3e481b3c7d54e2d1148fdcf14645ef59cf090e7`.
Its 3.136/6.805 s finish-to-final p50/p95 uses unpaced burst ingress and must
not be presented as Live latency. A separate two-run paced 5 s joint check
measured worker-busy RTF p95 `0.609`, finish-to-final p95 `761 ms`, but first
affect p95 `2.352 s` (first transcript `2.276 s`). That is capture-start UX on
this WAV, including VAD/window accumulation and shared scheduling, not WavLM
inference alone: stable emotion jobs measured 21/24 ms p50/p95. Report
`/tmp/nextengine-speech-timeline-wavlm-paced-5s.json`, SHA-256
`371a2a9012d07abc9b382bc86d9f992e052d3a2dc35ed9a6925f2f7687352582`.
The remaining acceptance evidence is a labelled user-microphone checklist.
[model card](https://huggingface.co/Aniemore/wavlm-emotion-russian-resd)

## Next Russian SER candidates — 2026-08-19 research

`nikatonika/aniemore-audio-finetuned` is the first direct A/B candidate. It is
a standard local `WavLMForSequenceClassification` checkpoint at revision
`3928ed3f29ffd09a4249d00b76ea44c9c8b7bb6d`, publishes a safetensors weight
file, needs no remote code, accepts 16 kHz normalized audio, and exposes
`Angry`, `Disgusted`, `Happy`, `Neutral`, `Sad`, `Scared`, `Surprised`. Its
training claim is Russian Dusha + EmoGator with a natural class distribution,
which is relevant because RESD is acted while Dusha includes real podcast
speech. Its reported 0.860 accuracy / 0.858 macro-F1 is only a self-reported
1,575-sample validation split, so it is not comparable to the frozen RESD
screen or a generalization claim. The card is sparse (missing YAML metadata and
formal citation); moreover it calls the base `wavlm-base-plus`, while its own
config has WavLM Large dimensions (1,024 hidden, 24 layers), identical to the
current candidate. Treat that as a provenance/documentation warning, not proof
of incorrect weights. Its CC-BY-4.0 licence requires attribution and the
claimed Dusha/EmoGator upstream terms still need classification before any
distribution. [model card](https://huggingface.co/nikatonika/aniemore-audio-finetuned)

The controlled screen is: fetch only `config.json`,
`preprocessor_config.json` and `model.safetensors` at that revision outside
Git; hash the weights; then run the existing RESD-70 v2 direct harness over the
six common classes. `Scared → fearful` and the other five common labels map
directly; RESD `enthusiasm` and the candidate's `Surprised` remain explicit
non-comparable labels. Do not alter VAD, smoothing, thresholds or the current
WavLM profile until this cheap gate passes. If it does, generalize the WavLM
adapter's declared profile label map rather than hard-coding this new head.

`Aniemore/wav2vec2-emotion-v1-crosslingual` (revision
`6634a4213bbc63f2e9613f4d827a1d29e3b5e0da`) is the secondary generalization
candidate. It is a standard no-remote-code safetensors Wav2Vec2 model under
Apache-2.0 and keeps the seven Aniemore labels. Its self-reported RESD macro-F1
falls to 0.534, but mapped Dusha podcast macro-F1 rises to 0.345 from the
current WavLM's 0.112; the card explicitly says these models trade acted
accuracy for spontaneous speech and need neutral-threshold calibration. It is
worth testing only after `nikatonika` or against a new natural-microphone set,
not as an automatic replacement. [model card](https://huggingface.co/Aniemore/wav2vec2-emotion-v1-crosslingual)

Do not test the apparently stronger Aniemore audio-text fusion variants in this
increment: their `auto_map` points to repository Python (`custom_code`), which
violates the current no-remote-code local-model boundary, and their reported
Dusha macro-F1 is lower than the current audio-only WavLM. HuBERT and legacy
Wav2Vec2 RESD checkpoints are also lower on their own spontaneous-Dusha rows.

## Model shortlist (research, not an approval to install)

1. **`emotion2vec/emotion2vec_plus_large` — first A/B candidate.** It preserves
   the exact nine output categories and 16 kHz utterance/frame contract used by
   the current adapter, while the publisher describes the base as about 90M and
   large as about 300M parameters, trained from the larger pseudo-labelled
   corpus. Its API compatibility makes it the smallest controlled model swap.
   The price is roughly a threefold model footprint and unknown live latency on
   the RTX 3080, so benchmark it before changing the profile. [base model
   card](https://huggingface.co/emotion2vec/emotion2vec_plus_base), [large model
   card](https://huggingface.co/emotion2vec/emotion2vec_plus_large)

2. **Russian-specialized Wav2Vec2 baseline, lower priority.**
   `Aniemore/wav2vec2-xlsr-53-russian-emotion-recognition` has seven Russian
   labels and 316M parameters. Its own card reports macro-F1 0.721 on RESD but
   only 0.095 on Dusha podcast and 0.225 on CAMEO; that is useful evidence of
   domain sensitivity rather than a basis for replacement. It should be loaded
   only at a pinned revision after eliminating `trust_remote_code` or auditing
   the exact code. [model card](https://huggingface.co/Aniemore/wav2vec2-xlsr-53-russian-emotion-recognition)

3. **`KELONMYOSA/wav2vec2-xls-r-300m-emotion-ru` — exploratory only.** It is
   trained on roughly 125k Russian DUSHA dialog recordings and has a compact
   five-label mapping, but has no comparable reported evaluation in its card
   and asks for remote code. It is a hypothesis to test on the same manifest,
   not evidence of superiority. [model card](https://huggingface.co/KELONMYOSA/wav2vec2-xls-r-300m-emotion-ru)

4. **Rejected for this Russian live path.** SenseVoiceSmall combines ASR,
   emotion and acoustic-event tags, but its released checkpoint documents ASR
   language support only for Chinese, Cantonese, English, Japanese and Korean;
   it is not a sound Russian SER upgrade. MERaLiON-SER-v1 is similarly
   multilingual but does not list Russian. The audEERING dimensional
   arousal/valence model is English/MSP-Podcast and research-only under its
   displayed license. [SenseVoice scope](https://github.com/QwenAudio/SenseVoice),
   [MERaLiON scope](https://huggingface.co/MERaLiON/MERaLiON-SER-v1),
   [audEERING card](https://huggingface.co/audeering/wav2vec2-large-robust-12-ft-emotion-msp-dim)

The broader model claim is also intentionally bounded: the emotion2vec paper
reports cross-language representation results, but does not certify this
specific microphone, noise profile, label ontology or user population. The
evaluation manifest is therefore the release criterion. [ACL paper](https://aclanthology.org/2024.findings-acl.931.pdf)

## Next smallest action

Do not retry `emotion2vec_plus_large` on the full path unless the product
weighting changes. Keep the WavLM profile running for actual-microphone
quiet-room calibration and a labelled checklist: the acted RESD screen does
not replace them. Before that checklist, the user-requested `nikatonika`
candidate may take one exact direct RESD-70 v2 A/B; retain the current WavLM
profile unless it wins the declared comparable label screen.
