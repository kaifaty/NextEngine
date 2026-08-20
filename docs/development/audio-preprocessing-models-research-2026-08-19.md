# Streaming speech-enhancement models for embedded capture — 2026-08-19

> **Selection update (2026-08-20):** DPDFNet remains a reproducible diagnostic
> candidate, but its current pre/post-gain route is no longer the leading default
> after the user preferred raw capture. The raw-first, multi-branch evaluation
> and next experiment are defined in
> [the speech front-end follow-up](speech-input-front-end-research-2026-08-20.md).

## Scope and decision boundary

This is a bounded research result for the optional `SpeechTimelineService`
prototype.  It does not promote SPEC-16/ADR-017 or select a shipped game
dependency.

The user requires preprocessing of the game's captured PCM stream itself.
The product path must not install, create or depend on virtual microphones,
PipeWire graphs or other system audio devices.  A usable candidate must be
resident, causal/stateful, locally runnable, provenance-pinned and able to
preserve one acoustic sample clock through its algorithmic delay.

The primary purpose is ASR robustness for quiet/whispered Russian speech.
Vocal-affect inference retains raw PCM by default.  Enhanced PCM may reach the
affect adapter only after a measured A/B gate, because enhancement can remove
emotion-discriminative acoustic features.

## Candidates

| Candidate | Evidence and deployment path | Suitability | Decision |
| --- | --- | --- | --- |
| **DPDFNet** | Causal stateful streaming models at 8/16/48 kHz; 16-kHz `baseline`/`2`/`4`/`8` exports are 2.31–3.54 M parameters and 8.3–13.9 MiB ONNX. Official API buffers one ~20 ms model window then returns subsequent 10 ms hops. Apache-2.0. `sherpa-onnx` exposes local online DPDFNet APIs and a maintained Rust crate/examples. Sources: [project](https://github.com/ceva-ip/DPDFNet), [streaming contract](https://github.com/ceva-ip/DPDFNet), [Rust/runtime support](https://github.com/k2-fsa/sherpa-onnx). | Direct 16-kHz ASR path avoids an unnecessary 48→16 kHz denoise/resample round-trip. State and small ONNX artifacts fit a resident Rust adapter via `sherpa-onnx`; no virtual device is required. | **First measured candidate**: `dpdfnet2` then `dpdfnet4`; start on CPU and preserve raw fail-open bypass. |
| **DeepFilterNet** | Rust codebase, full-band 48-kHz speech enhancement, dual MIT/Apache-2.0. Its zero-lookahead LADSPA variant reports 20 ms minimum STFT latency, but that plugin is not the product integration path. Source: [DeepFilterNet](https://github.com/Rikorose/DeepFilterNet). | A mature direct-Rust alternative, especially if the eventual capture path needs high-resolution 48-kHz output. Current ASR consumes 16 kHz, so this needs more resampling/integration work than DPDFNet. | **Second A/B candidate**, not the first integration. |
| **`nnnoiseless` / RNNoise** | `nnnoiseless` is a BSD-3-Clause Rust RNNoise port with a low-level `DenoiseState` API. Sources: [repository](https://github.com/jneem/nnnoiseless), [crate API](https://docs.rs/nnnoiseless/latest/nnnoiseless/). | Very small fallback for low-spec machines or a baseline. It has less capacity than current DPDFNet candidates. | **Fallback/baseline** only. |
| ClearerVoice FRCRN/MossFormer, FullSubNet(+), Meta Demucs denoiser | ClearerVoice publishes enhancement models but its documented route is Python/PyTorch; FullSubNet is a research PyTorch project. Meta's real-time denoiser repository is archived. Sources: [ClearerVoice](https://github.com/modelscope/ClearerVoice-Studio), [FullSubNet](https://github.com/audio-westlakeu/FullSubNet), [archived Denoiser](https://github.com/facebookresearch/denoiser). | Worth offline quality comparison, but no smaller measured embedded Rust path than DPDFNet currently. | Do not integrate first. |
| DeepFilterGAN | 2025 paper describes a 3.58 M-parameter low-latency model intended to recover speech that predictive denoisers over-suppress. Source: [paper](https://arxiv.org/abs/2505.23515). | Promising, particularly for whispers, but this research found no pinned official inference package/weights or Rust/ONNX path suitable for product evaluation. | Research watchlist; do not select without reproducible artifacts. |

## Why DPDFNet leads the trial

DPDFNet is the only surveyed candidate with all currently needed pieces:

1. causal, stateful online processing rather than file-oriented enhancement;
2. appropriately sized 16-kHz models for the existing ASR contract;
3. published ONNX artifacts and clear model-size/compute choices;
4. an existing local `sherpa-onnx` online denoiser API with C and Rust support;
5. an Apache-2.0 code license (weight provenance still requires exact revision,
   hash and license verification before distribution).

It is a candidate, not an untested quality claim.  Its published streaming
description reports ~20 ms until the first enhanced output and ~10 ms on later
hops; target hardware must measure real callback/process time, queue depth and
end-to-end ASR latency.

## Required adapter contract

```text
host PCM frame (start_sample, end_sample, samples)
  → resident AudioPreprocessor
  → raw_affect frame, asr_enhanced frame, speech/noise metrics,
    algorithmic_delay_samples, exact model/config identity
```

- The model state belongs to one capture stream and resets only at its declared
  boundary.
- Output is ordered and carries the original acoustic interval plus explicit
  delay; no text timing is inferred from enhancement output.
- Bypass, model failure, deadline breach or invalid/non-finite output fail open
  to the raw ASR frame and emit bounded diagnostics.
- Gain follows suppression and is bounded/speech-aware.  It cannot amplify
  silence indefinitely or clip a sample.
- The facade takes no vendor types; `sherpa-onnx`, DeepFilterNet and RNNoise
  remain replaceable adapters.

## Evaluation before selection

Use identical, consented microphone takes in at least: normal quiet speech,
quiet whisper, and whisper plus representative stationary/non-stationary noise.
For each model and bypass, retain no raw audio by default; where the explicit
five-record diagnostic buffer is enabled, compare:

1. ASR transcript/WER against the known prompt and first/final latency;
2. VAD speech admission, false speech during silence, SNR/level and clipping;
3. raw-affect versus enhanced-affect agreement and labelled emotion quality;
4. RTF, p95 processing time, CPU/RAM, algorithmic delay and failure/bypass
   behaviour on the active target;
5. exact model revision, SHA-256, ONNX Runtime/sherpa version and configuration.

Speech enhancement is not automatically safe for SER.  Research shows that
enhancement can be useful when designed/evaluated for noisy SER, but it can
also modify discriminative acoustic features; see [Zhou et al., Interspeech
2020](https://www.isca-archive.org/interspeech_2020/zhou20g_interspeech.html)
and [Tzeng et al., ICASSP 2025](https://lab-msp.com/MSP/publications/Tzeng_2025.pdf).
Therefore quality metrics such as DNSMOS/PESQ alone cannot authorize the affect
branch change.

## Smallest next action

Prototype the replaceable `AudioPreprocessor` seam with an explicit raw bypass,
then use the 16-kHz DPDFNet `baseline` and `dpdfnet2` as the first offline and
resident-streaming A/B trial.  Do not switch default capture or affect input
until the evaluation above is recorded.
