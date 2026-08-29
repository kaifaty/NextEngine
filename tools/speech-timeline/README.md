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
authenticated `ws://127.0.0.1` listener. It loads and warms every configured
ASR adapter plus the selected vocal-affect model before publishing its ready
file. The explicit JSON profile, GGUF, model snapshots, `gigastt` runtime/model
bundle, emotion cache, `transcribe.cpp` checkout/library, ready file, and any
diagnostic output must all live outside the repository.

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-speech-timeline \
  serve --profile /path/outside/repository/speech-timeline-profile.v1.json
```

Once model warm-up completes, the ready JSON printed to stdout contains both
`uri` and `dashboard_uri`. Open the latter (for example,
`http://127.0.0.1:43721/`) in a browser, grant microphone access, select the
exact input, and press **Начать запись**. The Vue dashboard is served by the
same loopback process as the WebSocket, obtains its short-lived token from a
same-origin `no-store` bootstrap response, and never writes raw audio unless
the explicit diagnostic-retention profile is configured. The ASR selector is
locked for the duration of one utterance: Voxtral and NVIDIA Nemotron 3.5 emit
native/cache-aware streaming revisions, GigaSTT exposes bounded rolling-window
re-decode as `buffered_emulation`, and GigaAM-v3 produces one
finalized-utterance result after **Завершить фразу**.
For Voxtral, the dashboard also exposes three per-utterance quality/latency
presets: **480 ms**, **960 ms**, and **2400 ms**. The profile's 480 ms value
remains the default. Selecting another value adds optional `asr_delay_ms` to
`session.start`; the service validates it against the selected adapter's
advertised `supported_delay_ms`, locks it for that utterance, and echoes the
effective value in `session.started` and final ASR metrics. This creates new
stream options on the already-resident model session; it does not reload the
GGUF or duplicate its weights in VRAM. Higher-delay presets remain diagnostic
until paired Russian WER/CER and latency measurements justify a new default.
The same selection is available to repeatable microphone/benchmark runs as
`--asr-delay-ms 480|960|2400`; content-free benchmark reports record the
selected value in `run_configuration`.
The protocol defaults to `raw`. If the profile contains an audio preprocessor,
the dashboard exposes its explicit per-utterance routes: **RAW**, **Gain only**,
full **DPDFNet**, whisper-preserving **Whisper**, and any separately configured
**GTCRN** / **UL-UNAS** models. The dashboard preselects Whisper for the current
microphone trial; unsupported routes never fall back silently and no enhancer
is promoted over RAW without paired WER/CER evidence.

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

The profile is strict schema version 1 and requires the `emotion` plus
`service` objects; the `voxtral` branch itself is optional — when it is
absent, exactly one of the configured ASR branches (`gigaam` / `gigastt` /
`nemotron`) must back `default_asr_model`:

- `voxtral`: exact GGUF path, byte size, `sha256:` digest, `transcribe.cpp`
  root/library/revision, backend, model delay, and partial-decode interval
  (240 ms by default; legacy v1 profiles without the field keep this default);
- `emotion`: pinned model ID/revision, existing cache directory, device, and
  local-only classification. `adapter_id` is optional for legacy v1 profiles
  and defaults to `emotion2vec-plus/1`.
- `service`: loopback port (`0` selects an ephemeral port), external ready-file
  path, and aligned frame/turn byte ceilings.
- `gigaam` (optional): a local pinned `ai-sage/GigaAM-v3` `e2e_rnnt` snapshot,
  exact hashes for weights, executable model code, config and tokenizer, device,
  and local-only classification. `default_asr_model` may select it; otherwise
  Voxtral remains the backwards-compatible default.
- `gigastt` (optional): an exact external `gigastt` binary plus the four pinned
  GigaAM-v3 RNNT INT8 ONNX/vocabulary artifacts, runtime version/source
  revision and local-only classification. It is advertised as bounded buffered
  emulation, never as native streaming.
- `nemotron` (optional): the pinned NVIDIA Nemotron 3.5 streaming GGUF, pinned
  NeMo-Speech.cpp checkout and both hash-closed ASR libraries, GPU index,
  trained right-context mode and local-only classification. The current
  comparison profile uses `right_context: 1`, or 160 ms of configured model
  lookahead.
- `audio_preprocessor` (optional): a separately pinned, hash-validated
  streaming ONNX model and explicit routing. The current
`dpdfnet-streaming/1` adapter is trial-only and permits only `asr_only`;
  VAD, vocal affect and the original sample clock remain on raw PCM. When
  explicit diagnostics are enabled, raw and ASR-enhanced WAV are retained as
  separate, clearly labelled variants.
- `audio_enhancers` (optional): at most one pinned GTCRN and one pinned UL-UNAS
  streaming ONNX graph. Both are ASR-only, resident CPU routes behind the same
  facade; the profile validates exact file size and SHA-256 before startup.

### Selectable GigaAM-v3 ASR

Download the immutable `e2e_rnnt` snapshot outside the repository. Startup is
offline-only and validates all four artifacts before executing the pinned model
code:

```bash
hf download ai-sage/GigaAM-v3 \
  --revision 7655ad717f8122257385bb4b2f373db3697e8680 \
  --cache-dir ~/.cache/nextengine/gigaam-v3-e2e-rnnt/models
```

Add the following top-level profile object. Keep
`default_asr_model: "voxtral-realtime"` to preserve the current default while
exposing both choices in the dashboard.

```json
"default_asr_model": "voxtral-realtime",
"gigaam": {
  "model_id": "ai-sage/GigaAM-v3",
  "model_revision": "7655ad717f8122257385bb4b2f373db3697e8680",
  "snapshot_path": "/home/you/.cache/nextengine/gigaam-v3-e2e-rnnt/models/models--ai-sage--GigaAM-v3/snapshots/7655ad717f8122257385bb4b2f373db3697e8680",
  "device": "cuda",
  "classification": "unclassified_local_only",
  "weights_sha256": "sha256:afc6dcbae8320ea56f2cddebc0f13fbf62c9d59b6ddcad899782623c8610826a",
  "modeling_sha256": "sha256:269be43b635b1e510115baa2a843c5cbaa052e8adf0be30dc133a2ba5b5f2d86",
  "config_sha256": "sha256:02361ba9cafd6c3ec66fcdd73494c3b562a60eb2a2d1b13f3cb04ae440d93e52",
  "tokenizer_sha256": "sha256:828c12c991019eef952a960661f25a92d6ad279591e2ea466b4aeddf1d20a18a"
}
```

The current GigaAM adapter is intentionally honest about its batch behavior:
PCM chunks are accumulated under the existing bounded utterance clock, no
intermediate transcript is fabricated, and inference runs once at finish.
GigaAM's short-form limit is 25 seconds, so the browser and CLI use the lower
per-model bound when it is selected; Voxtral keeps the service's 30-second
ceiling. Both routes return the same transcript/timeline schema and report the
selected `asr_model` in `session.started`, `utterance.final`, metrics and saved
diagnostic-record metadata.

### Selectable GigaSTT buffered GigaAM baseline

The local comparison route uses the upstream `gigastt` `2.18.0` offline Linux
x86-64 release at source revision
`7bc17f438ebd4daaf8956aa2373cf77635bff41c`. Keep the release bundle outside
the repository. The profile validates the executable and all four RNNT model
artifacts by SHA-256 and verifies `gigastt --version` before starting it.

```json
"gigastt": {
  "model_id": "GigaAM-v3-rnnt-int8",
  "model_revision": "gigastt-v2.18.0-offline",
  "model_dir": "/home/you/.cache/nextengine/gigastt/2.18.0/offline/models",
  "runtime_path": "/home/you/.cache/nextengine/gigastt/2.18.0/offline/bin/gigastt",
  "runtime_version": "2.18.0",
  "runtime_revision": "7bc17f438ebd4daaf8956aa2373cf77635bff41c",
  "runtime_sha256": "sha256:35fecb26b1e4d97b55ad2ae8f33a6893b796638df7dcffa50993f8ba255696e4",
  "encoder_sha256": "sha256:c52665e9d96c4ca3a153c063d2ee9af6c567fe2975ca50fd038b75bbf2f60e7f",
  "decoder_sha256": "sha256:443c3b7bd42b453611618135d6b1e7d9467e5dd97c8a68501da4aa355750c0da",
  "joint_sha256": "sha256:fd1d02f45c2ad3d6b67cc149811ad794ab4b020ed49a0a9e2790a8619d1cddd8",
  "vocab_sha256": "sha256:a9143c30844d3c0bee3e9e927e4084774eb1b9eeaafc473b2c4521e4911a7c07",
  "classification": "unclassified_local_only"
}
```

The adapter owns one loopback-only CPU sidecar for the lifetime of the speech
service and opens one internal WebSocket per outer utterance. It disables
GigaSTT VAD, punctuation and ITN, selects manual endpointing, and forwards the
outer explicit finish as `stop`. GigaSTT re-decodes every 800 ms over a window
capped at 2.5 s with 1.5 s retained left context. Because its public WebSocket
payload does not expose the internal committed/live split, every partial is
reported as replaceable tentative text; only the final response becomes the
stable prefix. The CPU runtime avoids competing with Voxtral/GigaAM for VRAM.

This is a latency/quality baseline, not the canonical game ASR. The upstream
paired benchmark reports a sizeable stream-vs-batch WER regression, so local
WER/CER, empty-rate and revision churn on reference-transcribed Russian
normal/whisper/noise clips remain mandatory before any promotion.

### Selectable NVIDIA Nemotron 3.5 streaming ASR

The third comparison route is
`nvidia/nemotron-3.5-asr-streaming-0.6b` at Hugging Face revision
`1c8deaecc64b91f034d73e08dd8b64625eb3395d`. The current Linux trial uses the
official Q8 GGUF (`741548352` bytes,
`sha256:a5c435f294eea8f88ce68dd27b8c3bfea7f777cb2fbba04fcd30eaa555f429ae`)
and NeMo-Speech.cpp revision `4f9676226f667d14608487df744f375db87127f8`.
Build and install the official `cuda-asr` preset outside the repository, then
record the hashes of the installed implementation and C-ABI libraries.

```json
"nemotron": {
  "model_id": "nvidia/nemotron-3.5-asr-streaming-0.6b",
  "model_revision": "1c8deaecc64b91f034d73e08dd8b64625eb3395d",
  "model_path": "/home/you/.cache/nextengine/nemotron-3.5-streaming/model/nemotron-3.5-asr-streaming-0.6b.q8_0.gguf",
  "model_size_bytes": 741548352,
  "model_sha256": "sha256:a5c435f294eea8f88ce68dd27b8c3bfea7f777cb2fbba04fcd30eaa555f429ae",
  "runtime_root": "/home/you/.cache/nextengine/nemotron-3.5-streaming/nemo-speech-cpp",
  "runtime_revision": "4f9676226f667d14608487df744f375db87127f8",
  "implementation_library": "/home/you/.cache/nextengine/nemotron-3.5-streaming/runtime/lib/libnemo_speech_asr.so",
  "implementation_library_sha256": "sha256:<64 lowercase hex>",
  "abi_library": "/home/you/.cache/nextengine/nemotron-3.5-streaming/runtime/lib/libnemo_speech_asr_c.so.1",
  "abi_library_sha256": "sha256:<64 lowercase hex>",
  "gpu": 0,
  "right_context": 1,
  "classification": "unclassified_local_only"
}
```

Nemotron keeps its cache-aware RNNT state between 80 ms input frames and emits
replaceable partial hypotheses; only the explicit finish result is committed.
Its patched GGML and Voxtral's GGML have colliding shared-library names, so the
adapter owns one isolated resident worker process. This is process isolation,
not per-turn execution: the model loads and warms once at service startup and
all turns reuse it. Calls still pass through the facade's bounded priority
scheduler, and a worker failure is surfaced as a typed model operation failure.
The worker ignores terminal `SIGINT`; the parent service owns orderly shutdown.

The configured delay is model lookahead, not end-to-end text latency. The ready
capabilities expose supported trained modes (`80`, `160`, `560`, `1120` ms),
the active mode, 80 ms partial cadence, backend and exact model/runtime lineage.
The current Q8/local-runtime redistribution status remains
`unclassified_local_only`; do not copy the external artifacts into the
repository or a game package without a separate license/provenance review.

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

### GTCRN and UL-UNAS universal enhancement trial

The same facade can host the official streaming ONNX graphs from
[`Xiaobin-Rong/gtcrn`](https://github.com/Xiaobin-Rong/gtcrn) and
[`Xiaobin-Rong/ul-unas`](https://github.com/Xiaobin-Rong/ul-unas). They remain
CPU-resident beside DPDFNet and own distinct `gtcrn` / `ul_unas` routes. The
adapter validates the published tensor/cache shapes, executes one complex STFT
frame at a time and reconstructs PCM with normalized overlap-add. Its 512-sample
window and 256-sample hop report 32 ms algorithmic latency and preserve the
captured sample count exactly after flush. VAD and vocal affect still receive
RAW PCM.

Copy the exact upstream ONNX files outside the repository and add this bounded
top-level list to the profile:

```json
"audio_enhancers": [
  {
    "adapter_id": "gtcrn-onnx-streaming/1",
    "model_id": "Xiaobin-Rong/gtcrn",
    "model_revision": "502ebfab64da7c4a9af78dcb9c6ceef1ebb01c73",
    "model_path": "/home/you/.cache/nextengine/audio-enhancers/gtcrn/502ebfab64da7c4a9af78dcb9c6ceef1ebb01c73/gtcrn_simple.onnx",
    "model_size_bytes": 535190,
    "model_sha256": "sha256:b4718df6228e7bdf1a8a435cf98f838636eb2fd331acabf86ba87c5192ebcb87",
    "routing": "asr_only"
  },
  {
    "adapter_id": "ul-unas-onnx-streaming/1",
    "model_id": "Xiaobin-Rong/ul-unas",
    "model_revision": "00f7c700da43d38347f30a6ccebd86fcbc798e07",
    "model_path": "/home/you/.cache/nextengine/audio-enhancers/ul-unas/00f7c700da43d38347f30a6ccebd86fcbc798e07/ulunas_stream_simple.onnx",
    "model_size_bytes": 788967,
    "model_sha256": "sha256:f2e804d54d6a88f4f82f44d86c9f1cf646db2509bfca935cfbfc5fcd8cbfac3b",
    "routing": "asr_only"
  }
]
```

The dashboard's diagnostic list can replay one retained RAW WAV sequentially
through RAW, DPDFNet, GTCRN and UL-UNAS using the selected ASR and identical
80-ms PCM chunks. Replay is real-time paced to keep scheduler load comparable
and explicitly disables diagnostic retention, so it cannot evict the source
recording. The result table reports transcript, first-partial and finalization
latency, preprocessing p95 and signal levels. Supplying the known transcript
also computes WER and CER in the browser. This is an evaluation surface, not an
automatic route selector.

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
  --asr-model gigaam-v3-e2e-rnnt \
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
  --asr-model gigaam-v3-e2e-rnnt \
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

## Public-data ASR reliability corpus (R0)

The `reliability-corpus` commands implement the data-closure runner described
by
[`DEV-SPEECH-RELIABILITY-001`](../../docs/development/speech-recognition-reliability-public-data-spec-2026-08-20.md).
They do not train or load a reliability model, collect microphone turns, or
download a dataset implicitly. Corpus audio, source indexes and generated
reports must remain in an explicit directory outside this repository.

The committed `public-safe-v0` recipe is deliberately `planned`, without fake
artifact hashes. A manifest-only dry run reports every missing closure and does
not read audio or transcripts:

```bash
mkdir -p /path/outside/repository/speech-reliability

~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-speech-timeline \
  reliability-corpus dry-run \
  --manifest tools/speech-timeline/examples/speech-reliability-public-safe-v0.recipe.json \
  --store /path/outside/repository/speech-reliability \
  --out /path/outside/repository/speech-reliability/dry-run.json
```

Acquisition is an explicit operator step because Common Voice may require an
authenticated download and its raw clips must not be re-hosted. After acquiring
and normalizing an allowed source, create a JSONL source index in the external
store. Every speech line has exactly these fields:

```json
{
  "schema_version": 0,
  "clip_id": "source-stable-clip-id",
  "speaker_id": "source-stable-speaker-id",
  "relative_audio_path": "audio/source/clip.wav",
  "audio_sha256": "sha256:<64-lowercase-hex>",
  "samples": 32000,
  "sample_rate_hz": 16000,
  "channels": 1,
  "encoding": "pcm_s16le_wav",
  "transcript": "Эталонная русская фраза"
}
```

Noise and room-response sources use a content-free asset index. Segment long
upstream recordings into bounded normalized assets first; `partition_group_id`
keeps related segments in one split:

```json
{
  "schema_version": 0,
  "asset_id": "source-stable-asset-id",
  "partition_group_id": "source-stable-recording-id",
  "relative_audio_path": "audio/noise/asset.wav",
  "audio_sha256": "sha256:<64-lowercase-hex>",
  "samples": 960000,
  "sample_rate_hz": 16000,
  "channels": 1,
  "encoding": "pcm_s16le_wav"
}
```

Copy the recipe to the external store, replace each admitted source's status,
relative index path and SHA-256 with verified `closed` values, and bind the
exact resident replay identity. `prepare` then verifies the source-index hash,
every WAV hash and audio contract, rejects duplicate audio/path escape, applies
`ru-asr-normalize-v0`, assigns deterministic source/speaker and noise/RIR
partition groups, and atomically publishes a private (`0600`) prepared index:

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-speech-timeline \
  reliability-corpus prepare \
  --manifest /path/outside/repository/speech-reliability/closed-recipe.json \
  --store /path/outside/repository/speech-reliability \
  --out-index /path/outside/repository/speech-reliability/prepared.jsonl \
  --out /path/outside/repository/speech-reliability/prepare-report.json
```

The report contains hashes, counts, split roots and blockers, but no transcript,
raw speaker/partition ID or audio. The external prepared index contains
normalized references because later replay and scoring need them. References
containing digits and references that normalize to empty are excluded in V0.

Use the exact same normalizer and deterministic alignment for a quick scoring
check:

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-speech-timeline \
  reliability-corpus score \
  --reference "Привет, мир!" \
  --hypothesis "привет"
```

### Importing acquired sources into source indexes

After an operator downloads raw material into the explicit store (archives
stay outside Git; record their SHA-256 alongside), three commands turn it
into the JSONL source-index format consumed by `prepare`:

```bash
next-speech-timeline reliability-corpus import-fleurs \
  --store $STORE --fleurs-root $STORE/raw/fleurs \
  --out-index $STORE/indexes/fleurs-ru.jsonl --workers 12
next-speech-timeline reliability-corpus import-musan-noise \
  --store $STORE --noise-root $STORE/raw/musan/musan/noise \
  --out-index $STORE/indexes/musan-noise.jsonl --workers 8
next-speech-timeline reliability-corpus import-rirs \
  --store $STORE --rirs-root $STORE/raw/rirs_noises/RIRS_NOISES \
  --out-index $STORE/indexes/rirs.jsonl
```

Behavior notes:

- FLEURS ships IEEE-float WAVs; every clip is converted to the store contract
  (16 kHz mono pcm_s16le) with system ffmpeg, re-read through the standard
  library, hashed, and capped at the 30-second utterance limit — longer clips
  are counted under `skips.too_long`, never truncated.
- MUSAN noise files that already match the contract are referenced in place;
  only non-conforming ones are converted. The raw tree must live under the
  store root.
- RIRS_NOISES assets are referenced in place grouped by room prefix. They are
  expected to ship contract-conforming; any drift is a typed skip, not a
  silent conversion.
- Index rows carry pinned SHA-256 hashes; indexes publish atomically with
  private permissions outside this repository.

Common Voice releases are imported with one bounded command per release:

```bash
next-speech-timeline reliability-corpus import-common-voice \
  --store $STORE --cv-root $STORE/raw/common-voice/scripted/extracted/cv-corpus-26.0-2026-06-12/ru \
  --out-index $STORE/indexes/cv-scripted.jsonl --kind scripted \
  --max-rows 60000 --workers 12
```

- ``--kind scripted`` parses the headerless-style ``validated.tsv`` of
  release 26 (``client_id | path | sentence_id | sentence | …``) over
  ``clips/*.mp3``; ``--kind spontaneous`` parses the single
  ``ss-corpus-<locale>.tsv`` over ``audios/``.
- Admission takes the first ``--max-rows`` rows in file order — a
  documented deterministic selection — converts every clip from MP3 to the
  store contract with ffmpeg, and skips typed buckets for clips over 30 s,
  empty transcripts or missing audio (counts included in the report).
- Long SHA-256 ``client_id`` speakers are folded into bounded stable group
  ids so ``prepare`` can split by speaker without leaking raw identifiers.

### Deterministic degradations

`reliability-corpus augment` implements the §9 grid over a prepared index:
every selected clip gets one unchanged control reference plus up to
``max_variants_per_clip`` derived conditions (attenuation, additive MUSAN
noise at pinned SNR, RIRS room response with wet-mix permyriad, combined
room+noise, brickwall band EQ, peak-scale clipping with seeded 20 ms frame
loss).  All math runs in float64 on a [-1, 1) domain with a single pinned
round-and-clamp int16 conversion; noise/RIR pools are drawn only from asset
rows whose split equals the parent's, derivatives inherit that split and
copy the parent reference verbatim, and every derivation record carries the
subseed, transform order, picked parameters and both audio hashes.

```bash
next-speech-timeline reliability-corpus augment \
  --manifest $STORE/partial-public-safe.recipe.json \
  --store $STORE --prepared-index $STORE/prepared.jsonl \
  --out-index $STORE/augmented/train-calib.jsonl \
  --split train --split calibration --limit 500
```

The output is a replay-compatible speech index, so the same
``reliability-corpus replay`` command scores it against any bound route;
identical inputs reproduce identical bytes by construction.

## Recognition reliability feature capture (R1)

Every completed utterance now carries a bounded internal diagnostic payload at
`utterance.final` → `metrics.recognition_reliability_features`
(`nextengine.speech-reliability.features`, schema
`speech-reliability-features-v0`).  It folds the whole turn into fixed-size
numeric features; no transcript history is serialized into any event.

- Transcript revisions are folded by `TranscriptRevisionState`
  (`reliability_features.py`) with O(1) memory: revision/non-empty counts,
  audio and wall time of first text and last change, normalized Levenshtein
  churn between consecutive revisions (`lev/max(1,len,len)`, initial
  previous state is the empty string), cumulative/maximum churn,
  stable-prefix ratio over the current normalized text, final-to-previous
  edit distance, final word/character counts, utterance duration and typed
  flags (`all_revisions_empty`, `final_empty`,
  `text_appeared_then_vanished`, `final_matches_previous`).  Texts are
  normalized with the same `ru-asr-normalize-v0` as corpus scoring.
- Acoustic features keep raw PCM and the selected ASR route explicitly
  separate (`acoustic.raw` / `acoustic.asr_route`): samples, RMS/peak dBFS,
  non-zero ratio and clipping ratio (int16 saturation ≥ 32767).  Neither
  branch retains audio.  VAD speech samples/ratio/segment count, calibrated
  noise floor plus its source, route-minus-raw RMS/peak deltas complete the
  group.
- Runtime features record ingress frames, discontinuous-frame count,
  ASR-route sample deficit, scheduler overload delta, ASR job failures,
  preprocessor active/bypass state, algorithmic delay and exact identity:
  selected model, adapter/runtime IDs, delay, partial-decode interval,
  audio route and enhancer model identity.

The payload fails closed: NaN/infinite values, unknown schema versions,
missing identity fields or an unexpected adapter identity produce a typed
incomplete payload (`completeness: "incomplete"`, stable `invalid_reason`)
instead of a silently degraded vector.  It remains a tool-local experiment:
no public engine contract changes, and intermediate ASR text is never
gameplay authority.

## Prepared-corpus replay through the resident service

`reliability-corpus replay` sends every speech clip of one prepared
hash-closed index through the real resident WebSocket path exactly once per
clip — 16 kHz mono PCM in production 80 ms chunks, paced or unpaced, explicit
finish, one locked ASR model/route for the whole run.  The runner refuses
planned recipes, unclosed sources, tampered audio (every WAV is re-hashed)
and any mismatch between the recipe's pinned adapter identity and the live
service before touching the first clip.  Overload, turn overflow, timeout,
no-speech, speech-but-empty and technical failures are recorded as typed
outcomes; nothing is retried to green.

```bash
~/.cache/nextengine/emotion2vec-plus-base/venv/bin/next-speech-timeline \
  reliability-corpus replay \
  --manifest /path/outside/repository/speech-reliability/closed-recipe.json \
  --store /path/outside/repository/speech-reliability \
  --prepared-index /path/outside/repository/speech-reliability/prepared.jsonl \
  --ready-file /path/outside/repository/speech-timeline-ready.json \
  --out-dir /path/outside/repository/speech-reliability/replay-runs/run-001 \
  --asr-model voxtral-realtime \
  --asr-audio-route raw \
  --mode paced
```

Outputs stay in the external directory with private (0600) permissions and
are replaced atomically: `results.jsonl` (typed outcome, timing, deterministic
WER/CER/exact-match against the prepared normalized reference),
`features.jsonl` (the terminal reliability feature payload per clip),
`trace.jsonl` (bounded per-revision metadata: delivery sample cursor, service
elapsed ms, text and stable prefix, capped at 512 revisions per utterance
with a truncation counter) and `report.json` (hashes, identities, run
configuration and outcome counts; no transcript content).
