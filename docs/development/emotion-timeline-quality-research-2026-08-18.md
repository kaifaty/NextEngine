# Emotion timeline quality research — 2026-08-18

## Scope and decision

This note concerns only the optional Phase 1 vocal-expression diagnostic.
It neither promotes SPEC-16/ADR-017 nor changes the later LLM, TTS or
FunctionGemma scope.

**Decision:** retain `emotion2vec_plus_base` for the current live path, add a
speech-only final aggregation and an explicit quiet-room VAD calibration, then
run a pinned base-versus-large comparison. Do not replace a model based on a
model-card headline or a subjective single microphone recording.

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

2. **Russian-specialized A/B baseline, not a default.**
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

Restart the resident service with this build, calibrate the actual microphone
while quiet, then record a small labelled live checklist. Use the emitted final
diagnostics to establish the base line. Only then download and benchmark the
pinned `emotion2vec_plus_large` candidate through the existing replaceable
adapter boundary.
