# Speech timeline service and emotion2vec+ microphone probe

This is an isolated developer PoC for the Proposed `audio-understanding` role.
It is not a gameplay dependency, does not mutate engine state, and does not
make model output authoritative.

The project is the model-neutral home for the resident speech timeline service.
`next-speech-timeline` is the service entry point; the existing
`next-emotion-probe` command and JSON schema remain available for direct model
diagnostics. Model weights, runtime builds, ready files, tokens, and captured
audio stay outside the repository.

The backwards-compatible default profile selects
`emotion2vec/emotion2vec_plus_base`, pinned to Hugging Face revision
`b318240bfe67db81a8c572ecb37ce9c3759b81c9`. The facade can also select the
Russian RESD WavLM candidate, `Aniemore/wavlm-emotion-russian-resd`, without
changing the timeline protocol. Model weights and the Python environment belong
outside the repository under `~/.cache/nextengine/`.

## Install

```bash
uv venv --python 3.12 ~/.cache/nextengine/emotion2vec-plus-base/venv
source ~/.cache/nextengine/emotion2vec-plus-base/venv/bin/activate
uv sync \
  --project tools/speech-timeline \
  --active
deactivate
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-emotion-probe download
```

## Record the default PipeWire microphone

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-emotion-probe \
  record --seconds 5
```

Preserve the captured WAV only when it is explicitly needed:

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-emotion-probe \
  record --seconds 5 --keep-audio /tmp/emotion-sample.wav
```

Analyze an existing file:

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-emotion-probe \
  analyze /path/to/audio.wav
```

The JSON `scores` are upstream model scores, not calibrated probabilities or
facts about a speaker's internal state. The wrapper reports them as observed
vocal expression only.

## Resident service

The service accepts one explicit-finish utterance at a time over an
authenticated `ws://127.0.0.1` listener. It loads and warms both pinned models
before publishing its ready file. The explicit JSON profile, GGUF, emotion
cache, `transcribe.cpp` checkout/library, ready file, and any diagnostic output
must all live outside the repository.

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-speech-timeline \
  serve --profile /path/outside/repository/speech-timeline-profile.v1.json
```

Once model warm-up completes, the ready JSON printed to stdout contains both
`uri` and `dashboard_uri`. Open the latter (for example,
`http://127.0.0.1:43721/`) in a browser, grant microphone access, select the
exact input, and press **Начать запись**. The Vue dashboard is served by the
same loopback process as the WebSocket, obtains its short-lived token from a
same-origin `no-store` bootstrap response, and never writes raw audio.
The protocol defaults to `raw`. If the profile contains an audio preprocessor,
the dashboard exposes four explicit per-utterance choices: **RAW**,
**Gain only**, the previous full **DPDFNet** route, and the new
whisper-preserving **Whisper** route. The dashboard preselects Whisper for the
current microphone trial; unsupported routes never fall back silently.

The dashboard deliberately keeps these representations separate:

- stable and tentative Voxtral transcript revisions;
- VAD speech/no-speech segments on the same 16 kHz sample clock;
- raw score windows from the selected vocal-affect adapter on the 16 kHz sample
  clock;
- smoothed observed-expression segments;
- the final utterance-level fusion result and exact model lineage.

Timeline updates carry only newly observed affect windows after the first
revision (`raw_observations_mode: "append"`); the dashboard merges them by
`observation_id`. Smoothed segments remain a replaceable projection. This
keeps long-turn event payloads bounded instead of serializing the full affect
history on every ASR revision. Use `NEXTENGINE_SPEECH_LOG_LEVEL=DEBUG` (or
`serve --log-level DEBUG`) to inspect periodic ingress, slow model jobs, event
serialization/send timings, and browser-side slow-event diagnostics.

Emotion scores remain uncalibrated observations. The resident service applies
an energy-VAD gate (20 ms frames, 2-frame start hysteresis, 400 ms hangover,
200 ms pre-roll) and runs the selected affect model only when a window contains at least
600 ms and 35% voiced coverage. Silence is emitted as `no_speech`, never as a
model-derived `neutral` label. The gate is a replaceable baseline; a Silero or
WebRTC adapter can be evaluated behind the same activity contract later.
Phase 1 doesn't fabricate word timestamps, so emotional tags aren't attached
to individual words in this view. The final tagged-text form is a later derived
consumer view.

The profile is strict schema version 1 and contains three required objects plus
an optional preprocessing branch:

- `voxtral`: exact GGUF path, byte size, `sha256:` digest, `transcribe.cpp`
  root/library/revision, backend, model delay, and partial-decode interval
  (240 ms by default; legacy v1 profiles without the field keep this default);
- `emotion`: pinned model ID/revision, existing cache directory, device, and
  local-only classification. `adapter_id` is optional for legacy v1 profiles
  and defaults to `emotion2vec-plus/1`.
- `service`: loopback port (`0` selects an ephemeral port), external ready-file
  path, and aligned frame/turn byte ceilings.
- `audio_preprocessor` (optional): a separately pinned, hash-validated
  streaming ONNX model and explicit routing. The current
`dpdfnet-streaming/1` adapter is trial-only and permits only `asr_only`;
  VAD, vocal affect and the original sample clock remain on raw PCM. When
  explicit diagnostics are enabled, raw and ASR-enhanced WAV are retained as
  separate, clearly labelled variants.

### DPDFNet ASR-only trial

`dpdfnet-streaming/1` is a local CPU streaming preprocessor for a controlled
ASR A/B trial. It does not create a PipeWire/PulseAudio virtual device: the
service accepts PCM from the host and keeps exactly one causal DPDFNet stream
per utterance on a dedicated CPU worker. The model is loaded once at startup,
but RAW remains the selected control. Only an utterance that requests the
processed ASR route resets its resident state; its causal tail is drained at
finish so the ASR branch keeps the captured 16 kHz sample count.
VAD and vocal affect always receive raw PCM in both modes.

Download the exact model outside the repository, record both the Hugging Face
commit and digest, then add this object to the existing profile:

```json
"audio_preprocessor": {
  "adapter_id": "dpdfnet-streaming/1",
  "model_id": "Ceva-IP/DPDFNet",
  "model_revision": "dd6818d00f50c836fed43a6243ebe49116de5964",
  "model_name": "dpdfnet2",
  "model_path": "/home/you/.cache/nextengine/dpdfnet-0.6.0/onnx/dpdfnet2.onnx",
  "model_size_bytes": 10178747,
  "model_sha256": "sha256:4f0ee28935b4a32abecc717d745416976565834d839601acf43031094b4dc94c",
  "routing": "asr_only",
  "whisper_attenuation_limit_db": 12.0,
  "gain_placement": "pre_and_post_denoise",
  "gain": {
    "enabled": true,
    "activation_threshold_dbfs": -75.0,
    "target_dbfs": -26.0,
    "max_gain_db": 32.0,
    "attack_ms": 40,
    "release_ms": 160,
    "limiter_peak_dbfs": -1.0
  }
}
```

The adapter never auto-downloads a model at service startup. Its ONNX session
uses CPU execution with a reported 20 ms causal delay. The optional gain is a
20-ms speech-aware stage: it raises only frames above the gate, smooths
attack/release and hard-limits peaks. `enhanced` retains the earlier configured
DPDFNet/gain placement for regression comparison; `gain_only` isolates level
normalization; `whisper` applies one adaptive gain before DPDFNet, mixes an
aligned dry safety floor back into the denoised stream and applies only a peak
limiter afterwards. The default 12 dB attenuation limit retains about 25% of
the pre-gained dry signal, following the attenuation-limit form used by
DeepFilterNet. All routes preserve the exact sample clock and leave VAD/affect
on raw PCM. Final metrics expose raw/ASR RMS, peak, nonzero ratio, per-chunk
p50/p95 preprocessing time, input/output counts and flush time; capabilities
report each route's stages and exact attenuation setting. An unavailable route
fails before capture with `AUDIO_ROUTE_UNAVAILABLE`.

For a whisper test, first use **Калибровать тишину** in the dashboard while
remaining silent. The calibrated VAD and gain activation gates become relative
to that measured noise floor plus their declared margins, so background is not
amplified merely because the microphone is quiet. Each completed diagnostic
turn exposes both **Raw микрофон** and a route-labelled **ASR** WAV when it used
processing. Select **RAW · контроль** to bypass preprocessing immediately;
removing the whole `audio_preprocessor` object removes all three candidates
after restart.

The ready file is created with mode `0600`, contains the random session token,
and is removed on clean shutdown. Raw PCM, transcripts, and model outputs are
not written by default.

For an explicit local microphone investigation, an operator may add the
following `service` profile field. The service then writes only completed
utterances as 16 kHz mono raw WAV to the external directory, retains at most
five, and serves them only through the same loopback dashboard. When an
ASR-only preprocessor is enabled, each raw WAV may also receive a matching
ASR-enhanced WAV for listening comparison; no transcript or model output is
written beside either file.

```json
"diagnostic_audio": {
  "root": "/home/you/.cache/nextengine/speech-timeline-diagnostics",
  "max_records": 5
}
```

### Russian WavLM candidate profile

`transformers-wavlm-russian-ser/1` is a second, replaceable adapter for
`Aniemore/wavlm-emotion-russian-resd`. It uses the stock Transformers WavLM
implementation with `trust_remote_code=False`; it never downloads at service
startup. The cache must already contain the exact `config.json`,
`preprocessor_config.json`, and `model.safetensors` at the pinned revision.
The profile validates the weight digest before Voxtral or WavLM loads, so an
incomplete or altered cache fails closed.

```json
{
  "adapter_id": "transformers-wavlm-russian-ser/1",
  "model_id": "Aniemore/wavlm-emotion-russian-resd",
  "model_revision": "7a4ca18b34adff59b56b451acc7ff44fc43a12dc",
  "cache_dir": "/home/you/.cache/nextengine/aniemore-wavlm-russian-resd/models",
  "device": "cuda",
  "classification": "unclassified_local_only",
  "weights_sha256": "sha256:dabf15d84b451195346b8050102a7243b3f92276064195f6e34b65aaa06a12ab"
}
```

The adapter normalizes WavLM's seven native head labels to the timeline
vocabulary: `angry`, `disgusted`, `enthusiasm`, `fearful`, `happy`, `neutral`,
and `sad`. These remain uncalibrated observations of vocal expression, not
probabilities of a speaker's internal state. To roll back, remove `adapter_id`
and `weights_sha256` (or set `adapter_id` to `emotion2vec-plus/1`) in an
otherwise valid base profile, then restart the resident service.

For a trial of another stock `WavLMForSequenceClassification` checkpoint, use
`transformers-wavlm-audio-classification/1`. For the explicit bounded trial
set of stock `WavLM` or `Wav2Vec2` classification heads, use
`transformers-audio-classification/1`. Both require a one-to-one `label_map`
from the checkpoint's exact `config.json` labels to lowercase timeline labels.
The service validates the pinned model family, head and map before loading; it
does not guess label order or use model-repository Python.

```json
{
  "adapter_id": "transformers-audio-classification/1",
  "label_map": {
    "Angry": "angry",
    "Disgusted": "disgusted",
    "Happy": "happy",
    "Neutral": "neutral",
    "Sad": "sad",
    "Scared": "fearful",
    "Surprised": "surprised"
  }
}
```

In a second terminal, list inputs and connect the microphone client:

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-speech-timeline \
  microphone --list-inputs

~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-speech-timeline \
  microphone \
  --ready-file /path/outside/repository/speech-timeline-ready.json \
  --device 'pw:<exact-node-name>' \
  --locale ru \
  --asr-audio-route raw
```

The client performs the same microphone preflight as the direct Voxtral probe,
sends 80 ms PCM chunks by default, and renders replaceable transcript and
emotion timeline updates until `utterance.final`. Use Ctrl-C to cancel by
disconnecting. `--save-wav` remains an explicit diagnostic exception and
refuses repository paths or overwrite.

One Phase 1 turn is bounded to 960,000 bytes: 30 seconds of 16 kHz mono signed
16-bit PCM. Both the CLI and browser now stop capture at the advertised server
limit and finalize the turn automatically. `TURN_TOO_LARGE` means a client sent
beyond that boundary; the service treats it as one terminal input error. It is
not a model failure. Longer conversation must be split into utterances instead
of increasing an unbounded in-memory turn.
The browser asks for a 16 kHz audio context, uses a stateful fallback resampler,
disables browser AEC/noise suppression/AGC for ASR fidelity, batches worklet
messages to 20 ms, and flushes the final partial block before `session.finish`.
Transient browser/audio bursts are flow-controlled at the service's bounded
ASR ingress instead of being expanded into an unbounded model queue.

## Rebuild the Vue dashboard

The production bundle is committed inside the Python package so the installed
service has no Node.js runtime dependency. After changing `web/src`, rebuild it
with the pinned lockfile:

```bash
cd tools/speech-timeline/web
pnpm install --frozen-lockfile
pnpm run build
```

Measure sequential resident sessions with an external 16 kHz mono PCM WAV:

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-speech-timeline \
  benchmark \
  --ready-file /path/outside/repository/speech-timeline-ready.json \
  --audio /path/outside/repository/sample.wav \
  --mode unpaced \
  --runs 2 \
  --asr-audio-route raw \
  --out /path/outside/repository/speech-timeline-benchmark.json
```

For a comparable A/B, run the same hashed WAV and settings again with
`--asr-audio-route enhanced` and a different output path. The report records
the selected route under `run_configuration`; compare transcript quality
outside this content-free timing report, then compare route latency and
preprocessor metrics without relying on loudness as a quality oracle.

The report is atomically replaced with mode `0600` and contains model identity,
load counts, queue/inference latency, end-to-end RTF, and p50/p95 summaries. It
omits audio and transcript content. The terminal `metrics` payload keeps the
last 64 job records on the wire, while `jobs_total` and `job_summary` retain
exact counts/totals/percentiles for long turns; any future oversized event is
returned as one terminal protocol error rather than leaving the client waiting.

## Held-out affect evaluation

`evaluate-affect` replays a previously prepared external calibration manifest
through the public resident WebSocket path. This exercises the production VAD,
emotion-window cadence and final smoothing rather than calling an adapter
directly. It verifies every local WAV hash before use and writes only clip IDs,
source labels, final expression/admission diagnostics, timing and model lineage;
audio and ASR transcripts are omitted. The command is evaluation only: it never
trains a model or adjusts model/VAD thresholds.

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-speech-timeline \
  evaluate-affect \
  --ready-file /path/outside/repository/speech-timeline-ready.json \
  --manifest /path/outside/repository/resd-70-v1/manifest.json \
  --mode unpaced \
  --chunk-ms 80 \
  --out /path/outside/repository/resd-70-v1/timeline-base-report.v1.json
```

`unpaced` is the default for a compact model-quality screen: it preserves each
clip's 16 kHz sample order and the configured 80 ms logical ingress cadence, but
does not claim realtime wall latency. Use `--mode paced` when latency is part of
the question. A manifest must be outside the repository and use the
`nextengine.speech-timeline.manual-affect-calibration-set` schema; all selected
audio must be external, 16 kHz mono signed-16-bit PCM WAV, at most 30 seconds
per clip, and match the manifest's `normalized_audio_sha256`.
